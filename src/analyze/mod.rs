//! Read-only project analysis: which languages are present and how big they are,
//! how the project is laid out, where execution starts, and what it declares.
//!
//! Offline and read-only by construction: it only reads files, and it reuses the
//! audit walkers (`audit::walk_files`, `audit::find_named_files`,
//! `audit::detect_scopes`) so traversal and exclusions cannot drift from the
//! audit's own view of the project.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::audit;

/// Extension (without the dot) to language. One row per language family, so
/// adding a language is a row rather than a branch.
const LANGUAGES: &[(&str, &str)] = &[
    ("rs", "Rust"),
    ("ts", "TypeScript"),
    ("tsx", "TypeScript"),
    ("mts", "TypeScript"),
    ("cts", "TypeScript"),
    ("js", "JavaScript"),
    ("jsx", "JavaScript"),
    ("mjs", "JavaScript"),
    ("cjs", "JavaScript"),
    ("vue", "Vue"),
    ("php", "PHP"),
    ("go", "Go"),
    ("java", "Java"),
    ("cs", "C#"),
    ("py", "Python"),
    ("rb", "Ruby"),
    ("sh", "Shell"),
    ("ps1", "PowerShell"),
    ("sql", "SQL"),
    ("css", "CSS"),
    ("scss", "Sass"),
    ("html", "HTML"),
    ("md", "Markdown"),
    ("json", "JSON"),
    ("yaml", "YAML"),
    ("yml", "YAML"),
    ("toml", "TOML"),
    ("xml", "XML"),
];

/// Entry-point candidates per project part, checked in order. The first existing
/// path is the part's entry point.
const ENTRY_CANDIDATES: &[(&str, &[&str])] = &[
    (
        "frontend",
        &[
            "src/main.tsx",
            "src/main.ts",
            "src/main.jsx",
            "src/main.js",
            "src/App.vue",
            "app.vue",
            "pages/index.vue",
            "index.html",
        ],
    ),
    (
        "backend",
        &[
            "main.go",
            "src/main.ts",
            "Program.cs",
            "artisan",
            "public/index.php",
        ],
    ),
    (
        ".",
        &[
            "src/main.rs",
            "src/main.ts",
            "src/main.js",
            "src/index.ts",
            "src/index.js",
            "main.go",
            "Program.cs",
            "index.html",
        ],
    ),
];

/// A language and the size of its content, in files and lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LanguageSize {
    pub language: String,
    pub files: usize,
    pub lines: usize,
}

/// Top-level entries and the project parts recognized by the audit detector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Structure {
    pub top_level: Vec<String>,
    pub parts: Vec<String>,
}

/// Where execution starts for a part.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EntryPoint {
    pub part: String,
    pub path: String,
}

/// Dependencies declared by one manifest. `parsed` is false when the manifest was
/// scanned for dependency element names instead of parsed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestDependencies {
    pub manifest: String,
    pub parsed: bool,
    pub dependencies: Vec<String>,
}

/// What the tool can say about a project, plus what it could not analyze.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisReport {
    pub root: String,
    pub languages: Vec<LanguageSize>,
    pub structure: Structure,
    pub entry_points: Vec<EntryPoint>,
    pub dependencies: Vec<ManifestDependencies>,
    pub limits: Vec<String>,
}

/// Analyzes `root`. Read-only: nothing in the target project is written.
pub fn analyze(root: &Path) -> Result<AnalysisReport, String> {
    if !root.exists() {
        return Err(format!("Folder '{}' does not exist", root.display()));
    }
    if !root.is_dir() {
        return Err(format!("'{}' is not a folder", root.display()));
    }

    let files = audit::walk_files(root);
    let manifests = manifest_paths(root);
    let mut limits = Vec::new();
    let languages = count_languages(&files, &manifests, &mut limits);
    let structure = Structure {
        top_level: top_level_entries(root),
        parts: audit::detect_scopes(root),
    };
    let entry_points = entry_points(root);
    if entry_points.is_empty() {
        limits.push("no entry point recognized for any part".to_string());
    }
    let dependencies = read_dependencies(root, &manifests);
    if dependencies.iter().any(|manifest| !manifest.parsed) {
        limits.push(
            "XML manifests (pom.xml, *.csproj) are scanned for dependency element names, not parsed"
                .to_string(),
        );
    }
    limits.push(
        "figures are line counts: blank lines and comments are included, there is no code-versus-comment split"
            .to_string(),
    );
    limits.push(
        "dependency, build-output, version-control, and hidden directories are skipped by the walk"
            .to_string(),
    );

    Ok(AnalysisReport {
        root: root.display().to_string(),
        languages,
        structure,
        entry_points,
        dependencies,
        limits,
    })
}

/// Files and lines per language, sorted by language name. Content whose extension
/// is not in the language table and that is not read as a manifest is named in
/// `limits` instead of being dropped.
fn count_languages(
    files: &[PathBuf],
    manifests: &[PathBuf],
    limits: &mut Vec<String>,
) -> Vec<LanguageSize> {
    let mut totals: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    let mut unknown: BTreeMap<String, usize> = BTreeMap::new();
    let mut unreadable = 0usize;

    for file in files {
        let extension = file
            .extension()
            .map(|value| value.to_string_lossy().to_lowercase());
        let language = extension.as_deref().and_then(language_for);
        let Some(language) = language else {
            // A manifest that is not a language either (a .csproj) was read as a
            // manifest, so it is not "content that was not analyzed".
            if manifests.contains(file) {
                continue;
            }
            let label = match extension {
                Some(value) => format!(".{value}"),
                None => "(no extension)".to_string(),
            };
            *unknown.entry(label).or_insert(0) += 1;
            continue;
        };
        let Ok(content) = fs::read_to_string(file) else {
            unreadable += 1;
            continue;
        };
        let entry = totals.entry(language).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += content.lines().count();
    }

    if !unknown.is_empty() {
        let described = unknown
            .iter()
            .map(|(label, count)| format!("{label} ({count} file(s))"))
            .collect::<Vec<_>>()
            .join(", ");
        limits.push(format!("content not analyzed: {described}"));
    }
    if unreadable > 0 {
        limits.push(format!(
            "{unreadable} file(s) could not be read as text; their lines are not counted"
        ));
    }

    totals
        .into_iter()
        .map(|(language, (files, lines))| LanguageSize {
            language: language.to_string(),
            files,
            lines,
        })
        .collect()
}

fn language_for(extension: &str) -> Option<&'static str> {
    LANGUAGES
        .iter()
        .find(|(known, _)| *known == extension)
        .map(|(_, language)| *language)
}

/// Top-level directory and file names, skipping hidden and excluded directories.
fn top_level_entries(root: &Path) -> Vec<String> {
    let mut entries = Vec::new();
    let Ok(reader) = fs::read_dir(root) else {
        return entries;
    };
    for entry in reader.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false);
        if is_dir && audit::is_excluded_dir(&entry.path()) {
            continue;
        }
        entries.push(name);
    }
    entries.sort();
    entries
}

fn entry_points(root: &Path) -> Vec<EntryPoint> {
    let mut found = Vec::new();
    for (part, candidates) in ENTRY_CANDIDATES {
        let part_dir = root.join(part);
        if !part_dir.is_dir() {
            continue;
        }
        let label = if *part == "." { "root" } else { part };
        let hit = candidates
            .iter()
            .map(|candidate| part_dir.join(candidate))
            .find(|candidate| candidate.is_file())
            .or_else(|| java_entry_point(&part_dir));
        if let Some(hit) = hit {
            found.push(EntryPoint {
                part: label.to_string(),
                path: relative_path(root, &hit),
            });
        }
    }
    found
}

/// A Spring Boot application class is the entry point of a Java part; its name is
/// project-specific, so it is found by suffix rather than by a fixed path.
fn java_entry_point(part_dir: &Path) -> Option<PathBuf> {
    audit::find_files(&part_dir.join("src/main/java"), ".java")
        .into_iter()
        .find(|file| {
            file.file_name()
                .is_some_and(|name| name.to_string_lossy().ends_with("Application.java"))
        })
}

fn relative_path(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Every dependency manifest under `root`, sorted by path.
fn manifest_paths(root: &Path) -> Vec<PathBuf> {
    let mut manifests = audit::find_named_files(
        root,
        &[
            "package.json",
            "composer.json",
            "go.mod",
            "Cargo.toml",
            "pom.xml",
        ],
    );
    manifests.extend(
        audit::walk_files(root)
            .into_iter()
            .filter(|file| file.extension().is_some_and(|value| value == "csproj")),
    );
    manifests.sort();
    manifests.dedup();
    manifests
}

/// Dependencies declared by each manifest. A manifest that cannot be read is
/// reported with no dependencies rather than omitted.
fn read_dependencies(root: &Path, manifests: &[PathBuf]) -> Vec<ManifestDependencies> {
    manifests
        .iter()
        .filter_map(|manifest| read_manifest(root, manifest))
        .collect()
}

fn read_manifest(root: &Path, manifest: &Path) -> Option<ManifestDependencies> {
    let file_name = manifest.file_name()?.to_string_lossy().to_string();
    let text = fs::read_to_string(manifest).ok()?;
    let (parsed, dependencies) = match file_name.as_str() {
        "package.json" => (true, json_keys(&text, &["dependencies", "devDependencies"])),
        "composer.json" => (true, json_keys(&text, &["require", "require-dev"])),
        "go.mod" => (true, go_dependencies(&text)),
        "Cargo.toml" => (true, cargo_dependencies(&text)),
        "pom.xml" => (
            false,
            xml_dependencies(&text, "<artifactId>", "</artifactId>"),
        ),
        _ => (false, csproj_dependencies(&text)),
    };
    Some(ManifestDependencies {
        manifest: relative_path(root, manifest),
        parsed,
        dependencies,
    })
}

/// Top-level keys of the named objects, read with the JSON parser.
fn json_keys(text: &str, sections: &[&str]) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return Vec::new();
    };
    let mut keys = Vec::new();
    for section in sections {
        let Some(object) = value.get(section).and_then(|value| value.as_object()) else {
            continue;
        };
        keys.extend(object.keys().cloned());
    }
    keys.sort();
    keys.dedup();
    keys
}

/// Module names from `require` blocks and single-line require directives.
fn go_dependencies(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_block = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("require (") {
            in_block = true;
            continue;
        }
        if in_block && line == ")" {
            in_block = false;
            continue;
        }
        if in_block {
            if let Some(name) = line.split_whitespace().next() {
                names.push(name.to_string());
            }
        } else if let Some(rest) = line.strip_prefix("require ") {
            if let Some(name) = rest.split_whitespace().next() {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

/// Keys of the dependency tables, including per-target dependency tables.
fn cargo_dependencies(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_dependencies = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            let header = line.trim_start_matches('[').trim_end_matches(']');
            in_dependencies = header == "dependencies"
                || header == "dev-dependencies"
                || header == "build-dependencies"
                || header.ends_with(".dependencies")
                || header.ends_with(".dev-dependencies")
                || header.ends_with(".build-dependencies");
            continue;
        }
        if !in_dependencies || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, _)) = line.split_once('=') {
            let name = key.trim().trim_matches('"');
            if !name.is_empty() {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

/// Dependency artifact names from an XML manifest: everything between the opening
/// and closing element name, from the first dependency block onwards, so the
/// project's own artifact name is not counted. Scanned, not parsed.
fn xml_dependencies(text: &str, open: &str, close: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_dependencies = false;
    for line in text.lines() {
        if line.contains("<dependencies>") {
            in_dependencies = true;
            continue;
        }
        if !in_dependencies {
            continue;
        }
        let Some(start) = line.find(open) else {
            continue;
        };
        let rest = &line[start + open.len()..];
        let Some(end) = rest.find(close) else {
            continue;
        };
        let name = rest[..end].trim();
        if !name.is_empty() {
            names.push(name.to_string());
        }
    }
    names.sort();
    names.dedup();
    names
}

/// Package references from a .csproj: scanned, not parsed, since no XML parser is
/// a dependency of this tool.
fn csproj_dependencies(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in text.lines() {
        let Some(start) = line.find("<PackageReference") else {
            continue;
        };
        let rest = &line[start..];
        let Some(attribute) = rest.find("Include=\"") else {
            continue;
        };
        let rest = &rest[attribute + "Include=\"".len()..];
        let Some(end) = rest.find('"') else {
            continue;
        };
        names.push(rest[..end].to_string());
    }
    names.sort();
    names.dedup();
    names
}

/// The human-readable report. States its own limits rather than looking complete.
pub fn render_human(report: &AnalysisReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("Project analysis: {}\n", report.root));

    out.push_str("\nLanguages\n");
    if report.languages.is_empty() {
        out.push_str("  no recognized language files\n");
    }
    let width = report
        .languages
        .iter()
        .map(|entry| entry.language.len())
        .max()
        .unwrap_or(0);
    for entry in &report.languages {
        out.push_str(&format!(
            "  {:<width$}  {} file(s)  {} lines\n",
            entry.language,
            entry.files,
            entry.lines,
            width = width
        ));
    }

    out.push_str("\nStructure\n");
    out.push_str(&format!(
        "  top level: {}\n",
        join_or_none(&report.structure.top_level)
    ));
    out.push_str(&format!(
        "  parts: {}\n",
        join_or_none(&report.structure.parts)
    ));

    out.push_str("\nEntry points\n");
    if report.entry_points.is_empty() {
        out.push_str("  none recognized\n");
    }
    for entry in &report.entry_points {
        out.push_str(&format!("  {}: {}\n", entry.part, entry.path));
    }

    out.push_str("\nDependencies\n");
    if report.dependencies.is_empty() {
        out.push_str("  no dependency manifest found\n");
    }
    for manifest in &report.dependencies {
        let form = if manifest.parsed { "parsed" } else { "scanned" };
        out.push_str(&format!("  {} ({})\n", manifest.manifest, form));
        if manifest.dependencies.is_empty() {
            out.push_str("    declares none\n");
        }
        for dependency in &manifest.dependencies {
            out.push_str(&format!("    {dependency}\n"));
        }
    }

    out.push_str("\nLimits\n");
    for limit in &report.limits {
        out.push_str(&format!("  - {limit}\n"));
    }

    out
}

/// The same report as structured data, for a script or an agent.
pub fn render_json(report: &AnalysisReport) -> Result<String, String> {
    serde_json::to_string_pretty(report).map_err(|error| error.to_string())
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}
