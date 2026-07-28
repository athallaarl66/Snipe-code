use std::fs;
use std::path::Path;
use std::io::BufWriter;

use printpdf::*;
use docx_rs::*;

pub fn run_audit(project_path: &str, export_format: &str) -> Result<Vec<String>, String> {
    let path = Path::new(project_path);
    if !path.exists() {
        return Err(format!("Folder '{}' does not exist", project_path));
    }

    let mut findings = Vec::new();

    // Run all checks
    check_hardcoded_secrets(path, &mut findings);
    check_env_in_git(path, &mut findings);
    check_sql_injection(path, &mut findings);
    check_xss_risks(path, &mut findings);
    check_security_configs(path, &mut findings);

    let mut exported_files = Vec::new();
    let formats: Vec<&str> = export_format.split(',').map(|s| s.trim()).collect();

    for format in &formats {
        match *format {
            "md" => {
                let report = generate_markdown(&findings);
                let report_path = path.join("Audit_Report.md");
                fs::write(&report_path, &report).map_err(|e| e.to_string())?;
                exported_files.push("Audit_Report.md".to_string());
            }
            "pdf" => {
                let report_path = path.join("Audit_Report.pdf");
                generate_pdf(&findings, &report_path)?;
                exported_files.push("Audit_Report.pdf".to_string());
            }
            "docx" => {
                let report_path = path.join("Audit_Report.docx");
                generate_docx(&findings, &report_path)?;
                exported_files.push("Audit_Report.docx".to_string());
            }
            _ => {}
        }
    }

    Ok(exported_files)
}

#[derive(Debug)]
struct Finding {
    severity: String,
    category: String,
    description: String,
    file: String,
    line: usize,
}

// === CHECKS ===

fn check_hardcoded_secrets(path: &Path, findings: &mut Vec<Finding>) {
    let secret_patterns = vec!["password", "secret", "api_key", "apikey", "token", "private_key"];
    let extensions = vec![".ts", ".tsx", ".js", ".jsx", ".env", ".json"];

    for ext in &extensions {
        let files = find_files(path, ext);
        for file in files {
            if let Ok(content) = fs::read_to_string(&file) {
                for (line_num, line) in content.lines().enumerate() {
                    for pattern in &secret_patterns {
                        if line.to_lowercase().contains(pattern) && !line.trim().starts_with('#') {
                            if line.contains('=') && !line.contains("process.env") && !line.contains("your-") {
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
    }
}

fn check_env_in_git(path: &Path, findings: &mut Vec<Finding>) {
    let gitignore = path.join(".gitignore");
    if gitignore.exists() {
        if let Ok(content) = fs::read_to_string(&gitignore) {
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
}

fn check_sql_injection(path: &Path, findings: &mut Vec<Finding>) {
    let extensions = vec![".ts", ".tsx", ".js", ".jsx"];
    for ext in &extensions {
        let files = find_files(path, ext);
        for file in files {
            if let Ok(content) = fs::read_to_string(&file) {
                for (line_num, line) in content.lines().enumerate() {
                    if (line.contains("query") || line.contains("sql")) && line.contains('+') {
                        findings.push(Finding {
                            severity: "HIGH".to_string(),
                            category: "SQL Injection".to_string(),
                            description: "Potential SQL injection: string concatenation in query".to_string(),
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
            if let Ok(content) = fs::read_to_string(&file) {
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

fn find_files(path: &Path, extension: &str) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && !path.file_name().map_or(false, |n| {
                n.to_string_lossy().starts_with('.') || n.to_string_lossy() == "node_modules" || n.to_string_lossy() == "target"
            }) {
                files.extend(find_files(&path, extension));
            } else if path.extension().map_or(false, |e| e.to_string_lossy() == &extension[1..]) {
                files.push(path);
            }
        }
    }
    files
}

// === EXPORT: MARKDOWN ===

fn generate_markdown(findings: &[Finding]) -> String {
    let mut report = String::new();

    report.push_str("# Security Audit Report\n\n");
    report.push_str("**Generated by:** SNIPE-CODE CLI\n");
    report.push_str(&format!("**Date:** {}\n", chrono_placeholder()));
    report.push_str(&format!("**Total Findings:** {}\n\n", findings.len()));

    let high = findings.iter().filter(|f| f.severity == "HIGH").count();
    let medium = findings.iter().filter(|f| f.severity == "MEDIUM").count();
    let low = findings.iter().filter(|f| f.severity == "LOW").count();

    report.push_str("## Summary\n\n");
    report.push_str(&format!("- HIGH: {}\n", high));
    report.push_str(&format!("- MEDIUM: {}\n", medium));
    report.push_str(&format!("- LOW: {}\n\n", low));

    if findings.is_empty() {
        report.push_str("## No Issues Found\n\n");
        report.push_str("Your codebase looks clean! No security issues detected.\n");
    } else {
        report.push_str("## Findings\n\n");
        for (i, finding) in findings.iter().enumerate() {
            report.push_str(&format!("### {}. {} [{}]\n", i + 1, finding.category, finding.severity));
            report.push_str(&format!("**File:** `{}` (line {})\n", finding.file, finding.line));
            report.push_str(&format!("**Description:** {}\n\n", finding.description));
        }
    }

    report.push_str("---\n");
    report.push_str("*Report generated by SNIPE-CODE CLI*\n");

    report
}

// === EXPORT: PDF ===

fn generate_pdf(findings: &[Finding], output_path: &Path) -> Result<(), String> {
    let (doc, page1, layer1) = PdfDocument::new("Security Audit Report", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();

    let mut y = 750.0;

    // Title
    current_layer.use_text("Security Audit Report", 24.0, Mm(20.0), Mm(y), &font_bold);
    y -= 15.0;

    // Metadata
    current_layer.use_text(&format!("Generated by: SNIPE-CODE CLI"), 10.0, Mm(20.0), Mm(y), &font);
    y -= 8.0;
    current_layer.use_text(&format!("Date: {}", chrono_placeholder()), 10.0, Mm(20.0), Mm(y), &font);
    y -= 8.0;
    current_layer.use_text(&format!("Total Findings: {}", findings.len()), 10.0, Mm(20.0), Mm(y), &font);
    y -= 20.0;

    // Summary
    let high = findings.iter().filter(|f| f.severity == "HIGH").count();
    let medium = findings.iter().filter(|f| f.severity == "MEDIUM").count();
    let low = findings.iter().filter(|f| f.severity == "LOW").count();

    current_layer.use_text("Summary", 16.0, Mm(20.0), Mm(y), &font_bold);
    y -= 12.0;
    current_layer.use_text(&format!("HIGH: {}  |  MEDIUM: {}  |  LOW: {}", high, medium, low), 10.0, Mm(20.0), Mm(y), &font);
    y -= 20.0;

    // Findings
    if findings.is_empty() {
        current_layer.use_text("No Issues Found", 14.0, Mm(20.0), Mm(y), &font_bold);
        y -= 12.0;
        current_layer.use_text("Your codebase looks clean!", 10.0, Mm(20.0), Mm(y), &font);
    } else {
        current_layer.use_text("Findings", 16.0, Mm(20.0), Mm(y), &font_bold);
        y -= 15.0;

        for (i, finding) in findings.iter().enumerate() {
            if y < 50.0 {
                break; // Simple pagination: stop if page is full
            }

            let title = format!("{}. {} [{}]", i + 1, finding.category, finding.severity);
            current_layer.use_text(&title, 11.0, Mm(20.0), Mm(y), &font_bold);
            y -= 10.0;

            current_layer.use_text(&format!("File: {} (line {})", finding.file, finding.line), 9.0, Mm(25.0), Mm(y), &font);
            y -= 8.0;

            current_layer.use_text(&format!("Description: {}", finding.description), 9.0, Mm(25.0), Mm(y), &font);
            y -= 15.0;
        }
    }

    // Footer
    current_layer.use_text("Report generated by SNIPE-CODE CLI", 8.0, Mm(20.0), Mm(20.0), &font);

    let file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer).map_err(|e| e.to_string())?;

    Ok(())
}

// === EXPORT: DOCX ===

fn generate_docx(findings: &[Finding], output_path: &Path) -> Result<(), String> {
    let high = findings.iter().filter(|f| f.severity == "HIGH").count();
    let medium = findings.iter().filter(|f| f.severity == "MEDIUM").count();
    let low = findings.iter().filter(|f| f.severity == "LOW").count();

    let doc = Docx::new()
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("Security Audit Report").bold().size(36))
        )
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(format!("Generated by: SNIPE-CODE CLI")).size(20))
        )
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(format!("Date: {}", chrono_placeholder())).size(20))
        )
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(format!("Total Findings: {}", findings.len())).size(20))
        )
        .add_paragraph(Paragraph::new())
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("Summary").bold().size(28))
        )
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(format!("HIGH: {}  |  MEDIUM: {}  |  LOW: {}", high, medium, low)).size(20))
        )
        .add_paragraph(Paragraph::new());

    let doc = if findings.is_empty() {
        doc.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("No Issues Found").bold().size(24))
        )
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("Your codebase looks clean!").size(20))
        )
    } else {
        let mut doc = doc.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("Findings").bold().size(28))
        );

        for (i, finding) in findings.iter().enumerate() {
            doc = doc.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new()
                        .add_text(format!("{}. {} [{}]", i + 1, finding.category, finding.severity))
                        .bold()
                        .size(22))
            )
            .add_paragraph(
                Paragraph::new()
                    .add_run(Run::new()
                        .add_text(format!("File: {} (line {})", finding.file, finding.line))
                        .size(18))
            )
            .add_paragraph(
                Paragraph::new()
                    .add_run(Run::new()
                        .add_text(format!("Description: {}", finding.description))
                        .size(18))
            )
            .add_paragraph(Paragraph::new());
        }

        doc
    };

    let doc = doc.add_paragraph(
        Paragraph::new()
            .add_run(Run::new().add_text("Report generated by SNIPE-CODE CLI").size(16).italic())
    );

    let file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    doc.build().pack(file).map_err(|e| e.to_string())?;

    Ok(())
}

fn chrono_placeholder() -> String {
    "2026-07-28".to_string()
}
