use std::fs;
use std::io::BufWriter;
use std::path::Path;

use dialoguer::MultiSelect;
use docx_rs::*;
use printpdf::*;
use serde::{Deserialize, Serialize};

use crate::date;

/// Directories no traversal descends into: dependencies, build output, generated
/// reports. Hidden directories are skipped as well. Shared by the audit walkers and
/// the analysis walkers so one edit keeps both views of a project in step.
pub(crate) const EXCLUDED_DIRS: &[&str] = &[
    "node_modules",
    "vendor",
    "target",
    "dist",
    "build",
    "bin",
    "obj",
    "out",
    "coverage",
    "reports",
];

/// True for a directory every traversal skips. Hidden directories are skipped,
/// and `bin`/`obj` are build output everywhere except below a source directory:
/// `src/bin` is where a Rust crate keeps its additional binaries.
pub(crate) fn is_excluded_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().map(|name| name.to_string_lossy()) else {
        return false;
    };
    if name.starts_with('.') {
        return true;
    }
    if !EXCLUDED_DIRS.contains(&name.as_ref()) {
        return false;
    }
    let under_source = path
        .parent()
        .and_then(|parent| parent.file_name())
        .is_some_and(|parent| parent == "src");
    !(under_source && matches!(name.as_ref(), "bin" | "obj"))
}

fn is_within_root(root: &Path, candidate: &Path) -> bool {
    let Ok(root) = fs::canonicalize(root) else {
        return false;
    };
    fs::canonicalize(candidate).is_ok_and(|resolved| resolved.starts_with(root))
}

fn read_within_root(root: &Path, file: &Path) -> Option<String> {
    if !is_within_root(root, file) {
        return None;
    }
    fs::read_to_string(file).ok()
}

#[cfg(not(windows))]
fn is_link_or_unverified(entry: &fs::DirEntry) -> bool {
    entry
        .file_type()
        .map(|kind| kind.is_symlink())
        .unwrap_or(true)
}

#[cfg(windows)]
fn is_link_or_unverified(entry: &fs::DirEntry) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    entry
        .metadata()
        .map(|metadata| metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0)
        .unwrap_or(true)
}

/// Tampilkan MultiSelect interaktif untuk memilih format export audit.
/// Markdown pre-selected. Loop sampai minimal satu format dipilih.
pub fn select_export_formats() -> Vec<String> {
    let options = ["Markdown (.md)", "PDF (.pdf)", "DOCX (.docx)"];
    let ids = ["md", "pdf", "docx"];

    loop {
        let picked = MultiSelect::new()
            .with_prompt("Pilih format export (spasi = pilih, Enter = lanjut)")
            .items(&options)
            .defaults(&[true, false, false])
            .interact()
            .expect("Failed to read format selection");

        if picked.is_empty() {
            println!("Minimal satu format harus dipilih.");
            continue;
        }
        return picked.iter().map(|&i| ids[i].to_string()).collect();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditResult {
    #[serde(default)]
    pub exported_files: Vec<String>,
    pub findings: Vec<Finding>,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub highest_severity: Option<String>,
    pub policy_failed: bool,
    pub detected_scopes: Vec<String>,
    pub evaluated_rule_groups: Vec<String>,
    pub exclusions: Vec<String>,
}

pub fn run_audit(project_path: &str, export_format: &str) -> Result<Vec<String>, String> {
    Ok(run_audit_report(project_path, export_format)?.exported_files)
}

pub fn run_audit_scan(project_path: &str) -> Result<AuditResult, String> {
    let path = Path::new(project_path);
    if !path.is_dir() {
        return Err(format!("Folder '{}' does not exist", project_path));
    }
    let root = fs::canonicalize(path).map_err(|error| error.to_string())?;
    let mut findings = Vec::new();
    check_hardcoded_secrets(&root, &mut findings);
    check_env_in_git(&root, &mut findings);
    check_sql_injection(&root, &mut findings);
    check_xss_risks(&root, &mut findings);
    check_security_configs(&root, &mut findings);
    check_supported_stack_sources(&root, &mut findings);
    let detected_scopes = detect_scopes(&root);
    let high = findings.iter().filter(|f| f.severity == "HIGH").count();
    let medium = findings.iter().filter(|f| f.severity == "MEDIUM").count();
    let low = findings.iter().filter(|f| f.severity == "LOW").count();
    let highest_severity = if high > 0 {
        Some("HIGH")
    } else if medium > 0 {
        Some("MEDIUM")
    } else if low > 0 {
        Some("LOW")
    } else {
        None
    };
    Ok(AuditResult {
        exported_files: Vec::new(),
        findings,
        high,
        medium,
        low,
        highest_severity: highest_severity.map(str::to_string),
        policy_failed: high > 0,
        detected_scopes,
        evaluated_rule_groups: vec![
            "common".to_string(),
            "stack-specific heuristic rules".to_string(),
        ],
        exclusions: vec![
            ".git, node_modules, vendor, target, dist, build, reports, symlinked directories"
                .to_string(),
        ],
    })
}

pub fn run_audit_report(project_path: &str, export_format: &str) -> Result<AuditResult, String> {
    let formats = validate_formats(export_format)?;
    let path = Path::new(project_path);
    let mut result = run_audit_scan(project_path)?;
    for format in formats {
        let filename = match format.as_str() {
            "md" => "Audit_Report.md",
            "pdf" => "Audit_Report.pdf",
            "docx" => "Audit_Report.docx",
            _ => unreachable!("formats validated before export"),
        };
        let output = path.join(filename);
        match format.as_str() {
            "md" => fs::write(
                &output,
                generate_markdown(&result.findings, &result.detected_scopes),
            )
            .map_err(|e| e.to_string())?,
            "pdf" => generate_pdf(&result.findings, &output, &result.detected_scopes)?,
            "docx" => generate_docx(&result.findings, &output, &result.detected_scopes)?,
            _ => unreachable!("formats validated before export"),
        }
        result.exported_files.push(filename.to_string());
    }
    Ok(result)
}

fn validate_formats(value: &str) -> Result<Vec<String>, String> {
    let mut formats = Vec::new();
    for token in value.split(',').map(str::trim) {
        if token.is_empty() {
            return Err("export format contains an empty token".to_string());
        }
        if !matches!(token, "md" | "pdf" | "docx") {
            return Err(format!("unsupported export format: {token}"));
        }
        if !formats.iter().any(|format| format == token) {
            formats.push(token.to_string());
        }
    }
    if formats.is_empty() {
        return Err("at least one export format is required".to_string());
    }
    Ok(formats)
}

pub(crate) fn detect_scopes(path: &Path) -> Vec<String> {
    let mut scopes = vec!["common".to_string()];
    if path.join("frontend/package.json").exists() {
        scopes.push("frontend".to_string());
    }
    if path.join("backend/go.mod").exists() {
        scopes.push("backend:go".to_string());
    }
    if path.join("backend/package.json").exists() {
        scopes.push("backend:node".to_string());
    }
    if path.join("backend/pom.xml").exists() {
        scopes.push("backend:java".to_string());
    }
    if fs::read_dir(path.join("backend"))
        .map(|entries| {
            entries.flatten().any(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "csproj")
            })
        })
        .unwrap_or(false)
    {
        scopes.push("backend:dotnet".to_string());
    }
    if path.join("backend/composer.json").exists() {
        scopes.push("backend:php".to_string());
    }
    for manifest in find_named_files(
        path,
        &[
            "package.json",
            "composer.json",
            "go.mod",
            "pom.xml",
            "Cargo.toml",
        ],
    ) {
        let text = read_within_root(path, &manifest)
            .unwrap_or_default()
            .to_lowercase();
        let identity = if manifest.file_name().is_some_and(|name| name == "go.mod") {
            "go"
        } else if manifest.file_name().is_some_and(|name| name == "pom.xml") {
            if text.contains("spring") {
                "java:spring"
            } else {
                "java"
            }
        } else if manifest
            .file_name()
            .is_some_and(|name| name == "composer.json")
        {
            if text.contains("laravel") {
                "php:laravel"
            } else {
                "php"
            }
        } else if manifest
            .file_name()
            .is_some_and(|name| name == "Cargo.toml")
        {
            "rust"
        } else if text.contains("nuxt") {
            "javascript:nuxt"
        } else if text.contains("vue") {
            "javascript:vue"
        } else if text.contains("react") {
            "javascript:react"
        } else {
            "javascript/typescript"
        };
        let label = format!(
            "{} ({})",
            identity,
            manifest.parent().unwrap_or(path).display()
        );
        if !scopes.contains(&label) {
            scopes.push(label);
        }
    }
    scopes
}

pub(crate) fn find_named_files(root: &Path, names: &[&str]) -> Vec<std::path::PathBuf> {
    find_named_files_from(root, root, names)
}

fn find_named_files_from(root: &Path, path: &Path, names: &[&str]) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(path) else {
        return files;
    };
    for entry in entries.flatten() {
        let candidate = entry.path();
        if is_link_or_unverified(&entry) || !is_within_root(root, &candidate) {
            continue;
        }
        if candidate.is_dir() {
            if is_excluded_dir(&candidate) {
                continue;
            }
            files.extend(find_named_files_from(root, &candidate, names));
        } else if candidate
            .file_name()
            .is_some_and(|name| names.contains(&name.to_string_lossy().as_ref()))
        {
            files.push(candidate);
        }
    }
    files
}

/// One audit result. Public because the structured report serializes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub severity: String,
    pub category: String,
    pub description: String,
    pub file: String,
    pub line: usize,
}

/// The audit result as structured data: contexts, findings with severity and
/// location, totals, and the files the human-readable report was written to.
pub fn render_json(result: &AuditResult) -> Result<String, String> {
    serde_json::to_string_pretty(result).map_err(|error| error.to_string())
}

pub fn render_scan_json(root: &Path, result: AuditResult) -> Result<String, String> {
    let envelope = crate::history::ScanEnvelope::from_audit(root, result)?;
    serde_json::to_string_pretty(&envelope).map_err(|error| error.to_string())
}

pub fn export_scan_json(input: &Path, formats: &str, output: &Path) -> Result<Vec<String>, String> {
    let formats = validate_formats(formats)?;
    let mut result = crate::history::read_scan(input)?.to_audit_result();
    if !output.is_dir() {
        return Err(format!(
            "output directory does not exist: {}",
            output.display()
        ));
    }
    for format in &formats {
        let filename = match format.as_str() {
            "md" => "Audit_Report.md",
            "pdf" => "Audit_Report.pdf",
            "docx" => "Audit_Report.docx",
            _ => unreachable!("formats validated"),
        };
        let path = output.join(filename);
        if path.symlink_metadata().is_ok() {
            return Err(format!("output already exists: {}", path.display()));
        }
    }
    let mut exported = Vec::new();
    for format in formats {
        let filename = match format.as_str() {
            "md" => "Audit_Report.md",
            "pdf" => "Audit_Report.pdf",
            "docx" => "Audit_Report.docx",
            _ => unreachable!("formats validated"),
        };
        let path = output.join(filename);
        match format.as_str() {
            "md" => fs::write(
                &path,
                generate_markdown(&result.findings, &result.detected_scopes),
            )
            .map_err(|error| error.to_string())?,
            "pdf" => generate_pdf(&result.findings, &path, &result.detected_scopes)?,
            "docx" => generate_docx(&result.findings, &path, &result.detected_scopes)?,
            _ => unreachable!("formats validated"),
        }
        exported.push(path.display().to_string());
    }
    result.exported_files = exported.clone();
    Ok(exported)
}

pub fn validate_export_formats(value: &str) -> Result<(), String> {
    validate_formats(value).map(|_| ())
}

pub fn validate_audit_args(args: &[String]) -> Result<(), String> {
    let mut path_seen = false;
    let mut export_seen = false;
    let mut json_seen = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--export" => {
                if export_seen {
                    return Err("duplicate --export".to_string());
                }
                export_seen = true;
                let Some(value) = args.get(i + 1) else {
                    return Err("missing value for --export".to_string());
                };
                validate_formats(value)?;
                i += 2;
            }
            "--json" => {
                if json_seen {
                    return Err("duplicate --json".to_string());
                }
                json_seen = true;
                i += 1;
            }
            token if token.starts_with('-') => return Err(format!("unknown option {token}")),
            _ if path_seen => return Err("multiple audit paths".to_string()),
            _ => {
                path_seen = true;
                i += 1;
            }
        }
    }
    Ok(())
}

// === CHECKS ===

fn check_hardcoded_secrets(path: &Path, findings: &mut Vec<Finding>) {
    let secret_patterns = vec![
        "password",
        "secret",
        "api_key",
        "apikey",
        "token",
        "private_key",
    ];
    let extensions = vec![".ts", ".tsx", ".js", ".jsx", ".json"];

    let mut files: Vec<std::path::PathBuf> = Vec::new();
    for ext in &extensions {
        files.extend(find_files(path, ext));
    }
    files.extend(find_dotfiles(path, &[".env", ".env.local"]));

    for file in files {
        if let Some(content) = read_within_root(path, &file) {
            for (line_num, line) in content.lines().enumerate() {
                for pattern in &secret_patterns {
                    let lower = line.to_lowercase();
                    let value = line.split_once('=').map(|(_, value)| value.trim());
                    let literal_value = value.is_some_and(|value| {
                        (!value.is_empty())
                            && (value.contains('"')
                                || value.contains('\'')
                                || file
                                    .file_name()
                                    .is_some_and(|name| name == ".env" || name == ".env.local"))
                            && !value.contains("process.env")
                            && !value.contains("your-")
                            && !value.contains("placeholder")
                            && !value.contains("example")
                    });
                    if lower.contains(pattern)
                        && !line.trim().starts_with('#')
                        && line.contains('=')
                        && literal_value
                    {
                        findings.push(Finding {
                            severity: "HIGH".to_string(),
                            category: "Hardcoded Secret".to_string(),
                            description: format!("Potential hardcoded secret: {}", pattern),
                            file: file.display().to_string(),
                            line: line_num + 1,
                        });
                    }
                }
            }
        }
    }
}

fn check_env_in_git(path: &Path, findings: &mut Vec<Finding>) {
    let gitignore = path.join(".gitignore");
    if let Some(content) = read_within_root(path, &gitignore) {
        if !content.contains(".env") {
            findings.push(Finding {
                severity: "MEDIUM".to_string(),
                category: "Git Security".to_string(),
                description: ".env files not in .gitignore".to_string(),
                file: ".gitignore".to_string(),
                line: 0,
            });
        }
    }
}

fn check_sql_injection(path: &Path, findings: &mut Vec<Finding>) {
    let extensions = vec![".ts", ".tsx", ".js", ".jsx"];
    for ext in &extensions {
        let files = find_files(path, ext);
        for file in files {
            if let Some(content) = read_within_root(path, &file) {
                for (line_num, line) in content.lines().enumerate() {
                    let lower = line.to_lowercase();
                    let executable_query = (lower.contains("query") || lower.contains("sql"))
                        && line.contains('+')
                        && (line.contains("SELECT")
                            || line.contains("INSERT")
                            || line.contains("UPDATE")
                            || line.contains("DELETE")
                            || line.contains("select")
                            || line.contains("insert")
                            || line.contains("update")
                            || line.contains("delete"));
                    if executable_query {
                        findings.push(Finding {
                            severity: "HIGH".to_string(),
                            category: "SQL Injection".to_string(),
                            description: "Potential SQL injection: string concatenation in query"
                                .to_string(),
                            file: file.display().to_string(),
                            line: line_num + 1,
                        });
                    }
                }
            }
        }
    }
}

fn check_xss_risks(path: &Path, findings: &mut Vec<Finding>) {
    let extensions = vec![".tsx", ".jsx", ".vue"];
    for ext in &extensions {
        let files = find_files(path, ext);
        for file in files {
            if let Some(content) = read_within_root(path, &file) {
                for (line_num, line) in content.lines().enumerate() {
                    if line.contains("dangerouslySetInnerHTML") || line.contains("v-html") {
                        findings.push(Finding {
                            severity: "MEDIUM".to_string(),
                            category: "XSS Risk".to_string(),
                            description: "Direct HTML injection detected".to_string(),
                            file: file.display().to_string(),
                            line: line_num + 1,
                        });
                    }
                }
            }
        }
    }
}

fn check_security_configs(path: &Path, findings: &mut Vec<Finding>) {
    if !path.join(".env.example").exists() {
        findings.push(Finding {
            severity: "LOW".to_string(),
            category: "Security Config".to_string(),
            description: "Missing .env.example file".to_string(),
            file: ".env.example".to_string(),
            line: 0,
        });
    }

    if !path.join(".gitignore").exists() {
        findings.push(Finding {
            severity: "MEDIUM".to_string(),
            category: "Security Config".to_string(),
            description: "Missing .gitignore file".to_string(),
            file: ".gitignore".to_string(),
            line: 0,
        });
    }
}

fn check_supported_stack_sources(path: &Path, findings: &mut Vec<Finding>) {
    for extension in ["php", "go", "java", "cs", "rs"] {
        for file in find_files(path, &format!(".{extension}")) {
            let Some(content) = read_within_root(path, &file) else {
                continue;
            };
            for (line_num, line) in content.lines().enumerate() {
                let lower = line.to_lowercase();
                let sql = (lower.contains("query") || lower.contains("sql"))
                    && (line.contains('+') || line.contains(" . "))
                    && ["select", "insert", "update", "delete"]
                        .iter()
                        .any(|keyword| lower.contains(keyword));
                let secret = ["password=", "api_key=", "apikey=", "token=", "private_key="]
                    .iter()
                    .any(|pattern| lower.contains(pattern))
                    && (line.contains('"') || line.contains('\''))
                    && !lower.contains("process.env")
                    && !lower.contains("your-")
                    && !lower.contains("placeholder")
                    && !lower.contains("example");
                if sql || secret {
                    findings.push(Finding {
                        severity: "HIGH".to_string(),
                        category: "Stack Source Risk".to_string(),
                        description:
                            "Heuristic unsafe query or secret pattern in supported stack source"
                                .to_string(),
                        file: file.display().to_string(),
                        line: line_num + 1,
                    });
                }
            }
        }
    }
}

/// Every file under `path` that is not inside an excluded directory. The single
/// traversal shared by the audit and by the analysis report; callers filter.
pub(crate) fn walk_files(root: &Path) -> Vec<std::path::PathBuf> {
    walk_files_from(root, root)
}

fn walk_files_from(root: &Path, path: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(path) else {
        return files;
    };
    for entry in entries.flatten() {
        let candidate = entry.path();
        if is_link_or_unverified(&entry) || !is_within_root(root, &candidate) {
            continue;
        }
        if candidate.is_dir() {
            if !is_excluded_dir(&candidate) {
                files.extend(walk_files_from(root, &candidate));
            }
        } else {
            files.push(candidate);
        }
    }
    files
}

pub(crate) fn find_files(path: &Path, extension: &str) -> Vec<std::path::PathBuf> {
    let wanted = extension.trim_start_matches('.');
    walk_files(path)
        .into_iter()
        .filter(|file| {
            file.extension()
                .is_some_and(|found| found.to_string_lossy() == wanted)
        })
        .collect()
}

fn find_dotfiles(root: &Path, names: &[&str]) -> Vec<std::path::PathBuf> {
    find_dotfiles_from(root, root, names)
}

fn find_dotfiles_from(root: &Path, path: &Path, names: &[&str]) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(path) else {
        return files;
    };
    for entry in entries.flatten() {
        let candidate = entry.path();
        if is_link_or_unverified(&entry) || !is_within_root(root, &candidate) {
            continue;
        }
        if candidate.is_dir() {
            if !is_excluded_dir(&candidate) {
                files.extend(find_dotfiles_from(root, &candidate, names));
            }
        } else if candidate
            .file_name()
            .is_some_and(|name| names.contains(&name.to_string_lossy().as_ref()))
        {
            files.push(candidate);
        }
    }
    files
}

// === EXPORT: MARKDOWN ===

fn generate_markdown(findings: &[Finding], scopes: &[String]) -> String {
    let mut report = String::new();

    report.push_str("# Security Audit Report\n\n");
    report.push_str("**Generated by:** SNIPE-CODE CLI\n");
    report.push_str(&format!("**Date:** {}\n", date::real_date_yyyymmdd()));
    report.push_str(&format!("**Total Findings:** {}\n\n", findings.len()));
    report.push_str(&format!(
        "**Detected contexts:** {}\n",
        if scopes.is_empty() {
            "unknown".to_string()
        } else {
            scopes.join(", ")
        }
    ));
    report.push_str("**Rule coverage:** common and stack-specific heuristic checks; excluded dependency/build/report directories; results are not proof of complete security.\n\n");

    let high = findings.iter().filter(|f| f.severity == "HIGH").count();
    let medium = findings.iter().filter(|f| f.severity == "MEDIUM").count();
    let low = findings.iter().filter(|f| f.severity == "LOW").count();

    report.push_str("## Summary\n\n");
    report.push_str(&format!("- HIGH: {}\n", high));
    report.push_str(&format!("- MEDIUM: {}\n", medium));
    report.push_str(&format!("- LOW: {}\n\n", low));

    if findings.is_empty() {
        report.push_str("## No findings detected by evaluated rules\n\n");
        report.push_str("No findings detected by evaluated rules. Heuristic checks do not prove complete security.\n");
    } else {
        report.push_str("## Findings\n\n");
        for (i, finding) in findings.iter().enumerate() {
            report.push_str(&format!(
                "### {}. {} [{}]\n",
                i + 1,
                finding.category,
                finding.severity
            ));
            report.push_str(&format!(
                "**File:** `{}` (line {})\n",
                finding.file, finding.line
            ));
            report.push_str(&format!("**Description:** {}\n\n", finding.description));
        }
    }

    report.push_str("---\n");
    report.push_str("*Report generated by SNIPE-CODE CLI*\n");

    report
}

// === EXPORT: PDF ===

fn generate_pdf(findings: &[Finding], output_path: &Path, scopes: &[String]) -> Result<(), String> {
    let (doc, page1, layer1) =
        PdfDocument::new("Security Audit Report", Mm(210.0), Mm(297.0), "Layer 1");
    let mut current_layer = doc.get_page(page1).get_layer(layer1);

    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();

    let mut y = 750.0;

    // Title
    current_layer.use_text("Security Audit Report", 24.0, Mm(20.0), Mm(y), &font_bold);
    y -= 15.0;

    // Metadata
    current_layer.use_text("Generator: SNIPE-CODE CLI", 10.0, Mm(20.0), Mm(y), &font);
    y -= 8.0;
    current_layer.use_text(
        format!("Date: {}", date::real_date_yyyymmdd()),
        10.0,
        Mm(20.0),
        Mm(y),
        &font,
    );
    y -= 8.0;
    current_layer.use_text(
        format!(
            "Contexts: {}",
            if scopes.is_empty() {
                "unknown".to_string()
            } else {
                scopes.join(", ")
            }
        ),
        9.0,
        Mm(20.0),
        Mm(y),
        &font,
    );
    y -= 8.0;
    current_layer.use_text(
        format!("Total Findings: {}", findings.len()),
        10.0,
        Mm(20.0),
        Mm(y),
        &font,
    );
    y -= 20.0;

    // Summary
    let high = findings.iter().filter(|f| f.severity == "HIGH").count();
    let medium = findings.iter().filter(|f| f.severity == "MEDIUM").count();
    let low = findings.iter().filter(|f| f.severity == "LOW").count();

    current_layer.use_text("Summary", 16.0, Mm(20.0), Mm(y), &font_bold);
    y -= 12.0;
    current_layer.use_text(
        format!("HIGH: {}  |  MEDIUM: {}  |  LOW: {}", high, medium, low),
        10.0,
        Mm(20.0),
        Mm(y),
        &font,
    );
    y -= 20.0;

    // Findings
    if findings.is_empty() {
        current_layer.use_text(
            "No findings detected by evaluated rules",
            14.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 12.0;
        current_layer.use_text(
            "Heuristic checks do not prove complete security.",
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
    } else {
        current_layer.use_text("Findings", 16.0, Mm(20.0), Mm(y), &font_bold);
        y -= 15.0;

        for (i, finding) in findings.iter().enumerate() {
            if y < 50.0 {
                let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Findings");
                current_layer = doc.get_page(page).get_layer(layer);
                y = 800.0;
            }

            let title = format!("{}. {} [{}]", i + 1, finding.category, finding.severity);
            current_layer.use_text(&title, 11.0, Mm(20.0), Mm(y), &font_bold);
            y -= 10.0;

            current_layer.use_text(
                format!("File: {} (line {})", finding.file, finding.line),
                9.0,
                Mm(25.0),
                Mm(y),
                &font,
            );
            y -= 8.0;

            current_layer.use_text(
                format!("Description: {}", finding.description),
                9.0,
                Mm(25.0),
                Mm(y),
                &font,
            );
            y -= 15.0;
        }
    }

    // Footer
    current_layer.use_text(
        "Report generated by SNIPE-CODE CLI",
        8.0,
        Mm(20.0),
        Mm(20.0),
        &font,
    );

    let file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer).map_err(|e| e.to_string())?;

    Ok(())
}

// === EXPORT: DOCX ===

fn generate_docx(
    findings: &[Finding],
    output_path: &Path,
    scopes: &[String],
) -> Result<(), String> {
    let high = findings.iter().filter(|f| f.severity == "HIGH").count();
    let medium = findings.iter().filter(|f| f.severity == "MEDIUM").count();
    let low = findings.iter().filter(|f| f.severity == "LOW").count();

    let doc = Docx::new()
        .add_paragraph(
            Paragraph::new().add_run(Run::new().add_text("Security Audit Report").bold().size(36)),
        )
        .add_paragraph(
            Paragraph::new().add_run(
                Run::new()
                    .add_text("Generated by: SNIPE-CODE CLI".to_string())
                    .size(20),
            ),
        )
        .add_paragraph(
            Paragraph::new().add_run(
                Run::new()
                    .add_text(format!("Date: {}", date::real_date_yyyymmdd()))
                    .size(20),
            ),
        )
        .add_paragraph(
            Paragraph::new().add_run(
                Run::new()
                    .add_text(format!("Total Findings: {}", findings.len()))
                    .size(20),
            ),
        )
        .add_paragraph(
            Paragraph::new().add_run(
                Run::new()
                    .add_text(format!(
                        "Contexts: {}",
                        if scopes.is_empty() {
                            "unknown".to_string()
                        } else {
                            scopes.join(", ")
                        }
                    ))
                    .size(18),
            ),
        )
        .add_paragraph(Paragraph::new())
        .add_paragraph(Paragraph::new().add_run(Run::new().add_text("Summary").bold().size(28)))
        .add_paragraph(
            Paragraph::new().add_run(
                Run::new()
                    .add_text(format!(
                        "HIGH: {}  |  MEDIUM: {}  |  LOW: {}",
                        high, medium, low
                    ))
                    .size(20),
            ),
        )
        .add_paragraph(Paragraph::new());

    let doc = if findings.is_empty() {
        doc.add_paragraph(
            Paragraph::new().add_run(
                Run::new()
                    .add_text("No findings detected by evaluated rules")
                    .bold()
                    .size(24),
            ),
        )
        .add_paragraph(
            Paragraph::new().add_run(
                Run::new()
                    .add_text("Heuristic checks do not prove complete security.")
                    .size(20),
            ),
        )
    } else {
        let mut doc = doc.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text("Findings").bold().size(28)),
        );

        for (i, finding) in findings.iter().enumerate() {
            doc = doc
                .add_paragraph(
                    Paragraph::new().add_run(
                        Run::new()
                            .add_text(format!(
                                "{}. {} [{}]",
                                i + 1,
                                finding.category,
                                finding.severity
                            ))
                            .bold()
                            .size(22),
                    ),
                )
                .add_paragraph(
                    Paragraph::new().add_run(
                        Run::new()
                            .add_text(format!("File: {} (line {})", finding.file, finding.line))
                            .size(18),
                    ),
                )
                .add_paragraph(
                    Paragraph::new().add_run(
                        Run::new()
                            .add_text(format!("Description: {}", finding.description))
                            .size(18),
                    ),
                )
                .add_paragraph(Paragraph::new());
        }

        doc
    };

    let doc = doc.add_paragraph(
        Paragraph::new().add_run(
            Run::new()
                .add_text("Report generated by SNIPE-CODE CLI")
                .size(16)
                .italic(),
        ),
    );

    let file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    doc.build().pack(file).map_err(|e| e.to_string())?;

    Ok(())
}
