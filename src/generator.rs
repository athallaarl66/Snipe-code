use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct GenerateConfig {
    pub project_name: String,
    pub template_id: String,
    pub frontend: String,
    pub backend: String,
    pub database: String,
}

#[derive(Debug)]
pub struct GenerateResult {
    pub folders: Vec<String>,
    pub files: Vec<String>,
}

pub fn folder_exists(project_name: &str) -> bool {
    Path::new(project_name).exists()
}

/// Validates `project_name` as a single, safe relative directory name.
pub fn validate_project_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Project name cannot be empty".to_string());
    }
    if name == "." || name == ".." {
        return Err(format!("'{}' is not a valid project name", name));
    }
    if name.contains('\0') {
        return Err("Project name contains a NUL byte".to_string());
    }
    // Path separators, drive prefixes, and Windows-invalid filename characters
    for c in name.chars() {
        if c == '/'
            || c == '\\'
            || c == ':'
            || c == '*'
            || c == '?'
            || c == '"'
            || c == '<'
            || c == '>'
            || c == '|'
        {
            return Err(format!(
                "'{}' is not a valid project name (contains '{}')",
                name, c
            ));
        }
    }
    if name.starts_with('.') {
        return Err(format!(
            "'{}' is not a valid project name (must not start with '.')",
            name
        ));
    }
    // Trailing dot or space is invalid on Windows and rejected by most tools
    if name.ends_with('.') || name.ends_with(' ') {
        return Err(format!(
            "'{}' is not a valid project name (trailing '.' or space)",
            name
        ));
    }
    // Windows reserved device names (case-insensitive, optional extension)
    let stem = name.split('.').next().unwrap_or("");
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if reserved.iter().any(|r| stem.eq_ignore_ascii_case(r)) {
        return Err(format!("'{}' is a reserved name on this platform", name));
    }
    Ok(())
}

/// Resolves the canonical destination for a project name against the current
/// directory. Rejects a final destination that is a symlink or junction.
pub fn resolve_destination(project_name: &str) -> Result<PathBuf, String> {
    let cwd =
        std::env::current_dir().map_err(|e| format!("Cannot read current directory: {}", e))?;
    let dest = cwd.join(project_name);
    match fs::symlink_metadata(&dest) {
        Ok(meta) if meta.file_type().is_symlink() => Err(format!(
            "Destination '{}' is a symlink/junction — refusing to touch it",
            dest.display()
        )),
        _ => Ok(dest),
    }
}

/// Creates a unique sibling staging directory next to the destination.
fn staging_dir(dest: &Path) -> Result<PathBuf, String> {
    let parent = dest
        .parent()
        .ok_or_else(|| "Destination has no parent directory".to_string())?;
    let name = dest
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    let pid = std::process::id();
    for attempt in 0..100u32 {
        let candidate = parent.join(format!(".{}.snipe-staging-{}-{}", name, pid, attempt));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("Cannot create staging directory: {}", e)),
        }
    }
    Err("Could not allocate a unique staging directory".to_string())
}

/// Generates the project inside a sibling staging directory, validates the
/// staged output, then publishes it to the destination.
///
/// `overwrite: true` allows replacement of an existing ordinary directory
/// (after successful staging). Existing symlinks/junctions are always rejected.
pub fn generate(config: &GenerateConfig, overwrite: bool) -> Result<GenerateResult, String> {
    validate_project_name(&config.project_name)?;
    let dest = resolve_destination(&config.project_name)?;
    if dest.exists() && !overwrite {
        return Err(format!(
            "Destination '{}' already exists (use overwrite to replace)",
            dest.display()
        ));
    }

    let staging = staging_dir(&dest)?;
    let result = match write_all(&staging, config) {
        Ok(r) => r,
        Err(e) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(e);
        }
    };
    if let Err(e) = validate_staged(&staging, config) {
        let _ = fs::remove_dir_all(&staging);
        return Err(e);
    }
    if let Err(e) = publish(&staging, &dest, overwrite) {
        let _ = fs::remove_dir_all(&staging);
        return Err(e);
    }
    Ok(result)
}

/// Writes every deterministic project file into `root`, including CI, Docker,
/// and AI workspace output (best-effort external Git runs later, after publish).
fn write_all(root: &Path, config: &GenerateConfig) -> Result<GenerateResult, String> {
    let mut result = GenerateResult {
        folders: Vec::new(),
        files: Vec::new(),
    };

    // Template folders and files
    let template_folders = crate::template::TemplateRegistry::get(&config.template_id)
        .map(|t| t.structure.folders.clone())
        .unwrap_or_default();
    for folder in &template_folders {
        fs::create_dir_all(root.join(folder)).map_err(|e| e.to_string())?;
        result.folders.push(folder.to_string());
    }
    if let Some(template) = crate::template::TemplateRegistry::get(&config.template_id) {
        for file in &template.structure.files {
            let path = root.join(&file.path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::write(&path, template_file_content(&file.template_source))
                .map_err(|e| e.to_string())?;
            result.files.push(file.path.clone());
        }
    }

    // Frontend
    if config.frontend != "none" {
        let frontend_root = root.join("frontend/");
        if let Some(folders) = crate::stack::get_frontend_folders(&config.frontend) {
            for folder in folders {
                fs::create_dir_all(frontend_root.join(folder)).map_err(|e| e.to_string())?;
                result.folders.push(format!("frontend/{}", folder));
            }
        }
        if let Some(files) = crate::stack::get_frontend_files(&config.frontend) {
            for file in files {
                let path = frontend_root.join(file.path);
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                fs::write(&path, file.content).map_err(|e| e.to_string())?;
                result.files.push(format!("frontend/{}", file.path));
            }
        }
    }

    // Backend
    if config.backend != "none" {
        let backend_root = root.join("backend/");
        if let Some(folders) = crate::stack::get_backend_folders(&config.backend) {
            for folder in folders {
                fs::create_dir_all(backend_root.join(folder)).map_err(|e| e.to_string())?;
                result.folders.push(format!("backend/{}", folder));
            }
        }
        if let Some(files) = crate::stack::get_backend_files(&config.backend) {
            for file in files {
                let path = backend_root.join(file.path);
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                fs::write(&path, file.content).map_err(|e| e.to_string())?;
                result.files.push(format!("backend/{}", file.path));
            }
        }
    }

    // Root security/support files
    fs::write(
        root.join(".env.example"),
        env_example_content(&config.database),
    )
    .map_err(|e| e.to_string())?;
    result.files.push(".env.example".to_string());
    fs::write(root.join(".gitignore"), gitignore_content()).map_err(|e| e.to_string())?;
    result.files.push(".gitignore".to_string());
    fs::write(root.join("README.md"), readme_content(config)).map_err(|e| e.to_string())?;
    result.files.push("README.md".to_string());

    // CI/CD (when a stack is selected), Docker (when a database/backend is), AI workspace
    if config.frontend != "none" || config.backend != "none" {
        crate::cicd::generate_ci_cd(&root.to_string_lossy(), &config.frontend, &config.backend)?;
        result.files.push(".github/workflows/main.yml".to_string());
    }
    if config.database != "none" || config.backend != "none" {
        crate::docker::generate_docker_compose(
            &root.to_string_lossy(),
            &config.database,
            &config.backend,
        )?;
        result.files.push("docker-compose.yml".to_string());
    }
    crate::ai_workspace::generate_ai_workspace(&root.to_string_lossy())?;
    result.files.push(".ai-workspace/rules.md".to_string());
    result.files.push(".ai-workspace/rules.mdc".to_string());

    Ok(result)
}

/// Validates that staged output satisfies the minimal contracts before publish.
fn validate_staged(root: &Path, config: &GenerateConfig) -> Result<(), String> {
    let required = [".env.example", ".gitignore", "README.md"];
    for rel in required {
        if !root.join(rel).exists() {
            return Err(format!("Staged output missing required file '{}'", rel));
        }
    }
    let manifests: &[&str] = if config.frontend != "none" {
        &["frontend/package.json"]
    } else {
        &[]
    };
    for rel in manifests {
        if !root.join(rel).exists() {
            return Err(format!("Staged output missing required file '{}'", rel));
        }
    }
    let backend_manifests = match config.backend.as_str() {
        "nestjs" => &["backend/package.json", "backend/nest-cli.json"][..],
        "go-gin" => &["backend/go.mod"][..],
        "spring-boot" => &["backend/pom.xml"][..],
        "dotnet8" => &["backend/*.csproj"][..],
        "laravel" => &["backend/composer.json", "backend/artisan"][..],
        _ => &[][..],
    };
    for rel in backend_manifests {
        let joined = root.join(rel);
        let exists = if rel.ends_with('*') {
            root.join(rel.trim_end_matches('*'))
                .read_dir()
                .map(|mut d| d.any(|e| e.is_ok()))
                .unwrap_or(false)
        } else {
            joined.exists()
        };
        if !exists {
            return Err(format!("Staged output missing required file '{}'", rel));
        }
    }
    Ok(())
}

/// Publishes the staged directory to the destination. New destinations are
/// published by rename; existing ordinary directories are replaced only after
/// explicit overwrite authorization (already enforced by caller).
fn publish(staging: &Path, dest: &Path, overwrite: bool) -> Result<(), String> {
    if dest.exists() {
        if !overwrite {
            return Err(format!(
                "Destination '{}' already exists (use overwrite to replace)",
                dest.display()
            ));
        }
        match fs::symlink_metadata(dest) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(format!(
                    "Destination '{}' is a symlink/junction — refusing to replace it",
                    dest.display()
                ));
            }
            Ok(_) => {
                fs::remove_dir_all(dest).map_err(|e| {
                    format!(
                        "Cannot remove existing destination '{}': {}",
                        dest.display(),
                        e
                    )
                })?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(format!(
                    "Cannot inspect destination '{}': {}",
                    dest.display(),
                    e
                ));
            }
        }
    }
    fs::rename(staging, dest)
        .map_err(|e| format!("Cannot publish project to '{}': {}", dest.display(), e))?;
    Ok(())
}

fn env_example_content(database: &str) -> String {
    let db_url = crate::stack::get_database_url(database);
    format!(
        r#"# Database
DATABASE_URL={}

# API
API_PORT=3000
API_SECRET=your-secret-here

# Security
CORS_ORIGIN=http://localhost:3000
RATE_LIMIT_WINDOW=60000
RATE_LIMIT_MAX=100

# [GENERATED BY SNIPE-CODE CLI]
"#,
        db_url
    )
}

fn gitignore_content() -> &'static str {
    r#"# Dependencies
node_modules/
target/
vendor/

# Environment
.env
.env.local
.env.*.local

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Build
dist/
build/
out/
"#
}

fn readme_content(config: &GenerateConfig) -> String {
    let mut sections = Vec::new();

    sections.push(format!("# {}\n", config.project_name));
    sections.push("Generated by SNIPE-CODE CLI\n".to_string());

    // Frontend setup
    if config.frontend != "none" {
        let fe_name = crate::stack::FRONTEND_STACKS
            .iter()
            .find(|s| s.id == config.frontend)
            .map(|s| s.name)
            .unwrap_or("Frontend");
        let init_cmd = crate::stack::get_frontend_init_command(&config.frontend);
        let install_cmd = crate::stack::get_frontend_install_command(&config.frontend);

        sections.push("## Frontend Setup\n".to_string());
        sections.push(format!("Stack: {}\n", fe_name));
        if !init_cmd.is_empty() {
            sections.push(format!(
                "```bash\ncd frontend\n{}\n{}\n```\n",
                init_cmd, install_cmd
            ));
        }
    }

    // Backend setup
    if config.backend != "none" {
        let be_name = crate::stack::BACKEND_STACKS
            .iter()
            .find(|s| s.id == config.backend)
            .map(|s| s.name)
            .unwrap_or("Backend");
        let init_cmd = crate::stack::get_backend_init_command(&config.backend);
        let install_cmd = crate::stack::get_backend_install_command(&config.backend);

        sections.push("## Backend Setup\n".to_string());
        sections.push(format!("Stack: {}\n", be_name));
        if !init_cmd.is_empty() {
            sections.push(format!(
                "```bash\ncd backend\n{}\n{}\n```\n",
                init_cmd, install_cmd
            ));
        }
    }

    sections.join("\n")
}

fn template_file_content(template_source: &str) -> String {
    match template_source {
        "portfolio-projects" => r#"{
  "projects": [
    {
      "title": "Project Name",
      "description": "Brief description of the project",
      "image": "/images/project.png",
      "tags": ["React", "TypeScript"],
      "link": "https://github.com/..."
    }
  ]
}"#
        .to_string(),
        "blog-sample-post" => {
            let date = real_date_yyyymmdd();
            format!(
                r#"---
title: "Hello World"
date: "{}"
tags: ["rust", "cli"]
category: "tutorial"
author: "SNIPE-CODE Team"
---

# Hello World

Welcome to your new blog! This is a sample post to get you started.

## Getting Started

Edit this file or create new `.mdx` files in the `posts/` directory.

## Features

- Markdown/MDX support
- Automatic RSS feed
- Categories and tags
- Responsive design
"#,
                date
            )
        }
        "blog-rss-feed" => {
            let rfc_date = rss_date();
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>My Blog</title>
    <description>Generated by SNIPE-CODE CLI</description>
    <link>http://localhost:3000</link>
    <atom:link href="http://localhost:3000/rss.xml" rel="self" type="application/rss+xml"/>
    <item>
      <title>Hello World</title>
      <link>http://localhost:3000/posts/hello-world</link>
      <pubDate>{}</pubDate>
      <description>Welcome to your new blog!</description>
    </item>
  </channel>
</rss>
"#,
                rfc_date
            )
        }
        _ => String::new(),
    }
}

pub fn dry_run_preview(config: &GenerateConfig) -> String {
    if let Err(e) = validate_project_name(&config.project_name) {
        return format!("Error: {}", e);
    }

    let mut entries: Vec<String> = Vec::new();

    // Check if folder exists
    match resolve_destination(&config.project_name) {
        Ok(dest) if dest.exists() => {
            entries.push(
                "[WARNING] Folder already exists — requires overwrite confirmation to replace"
                    .to_string(),
            );
        }
        Ok(_) => {}
        Err(e) => return format!("Error: {}", e),
    }

    // Collect template folders
    let template_folders = crate::template::TemplateRegistry::get(&config.template_id)
        .map(|t| t.structure.folders.clone())
        .unwrap_or_default();
    for f in &template_folders {
        entries.push(f.clone());
    }

    // Collect template files
    if let Some(template) = crate::template::TemplateRegistry::get(&config.template_id) {
        for file in &template.structure.files {
            entries.push(file.path.clone());
        }
    }

    // Collect frontend folders
    if config.frontend != "none" {
        let fe_folders = crate::stack::get_frontend_folders(&config.frontend)
            .unwrap_or(&[])
            .to_vec();
        for f in fe_folders {
            entries.push(format!("frontend/{}", f));
        }

        // Add frontend files
        if let Some(files) = crate::stack::get_frontend_files(&config.frontend) {
            for file in files {
                entries.push(format!("frontend/{}", file.path));
            }
        }
    }

    // Collect backend folders
    if config.backend != "none" {
        let be_folders = crate::stack::get_backend_folders(&config.backend)
            .unwrap_or(&[])
            .to_vec();
        for f in be_folders {
            entries.push(format!("backend/{}", f));
        }

        // Add backend files
        if let Some(files) = crate::stack::get_backend_files(&config.backend) {
            for file in files {
                entries.push(format!("backend/{}", file.path));
            }
        }
    }

    // Add root files
    entries.push(".env.example".to_string());
    entries.push(".gitignore".to_string());
    entries.push("README.md".to_string());

    format_tree(&config.project_name, &entries)
}

fn format_tree(project_name: &str, entries: &[String]) -> String {
    let mut lines = Vec::new();
    lines.push(format!("{}/", project_name));

    let total = entries.len();
    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == total - 1;
        let connector = if is_last { "└── " } else { "├── " };
        lines.push(format!("{}{}{}", connector, entry, ""));
    }

    lines.push(String::new());
    lines.push("(0 files created — dry run mode)".to_string());

    lines.join("\n")
}

// ponytail: dedupe with the shared date helper from feat-stack-aware-audit when merged
fn real_date_yyyymmdd() -> String {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() / 86400)
        .unwrap_or(0);
    let (y, m, d) = civil_from_days(days as i64);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn rss_date() -> String {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() / 86400)
        .unwrap_or(0);
    let (y, m, d) = civil_from_days(days as i64);
    let dow = day_of_week(y, m, d);
    let month = MONTHS[m as usize - 1];
    format!("{}, {:02} {} {:04} 00:00:00 GMT", dow, d, month, y)
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn day_of_week(y: i64, m: u32, d: u32) -> &'static str {
    static OFFSETS: [i64; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut yy = y;
    if m < 3 {
        yy -= 1;
    }
    let dow = (yy + yy / 4 - yy / 100 + yy / 400 + OFFSETS[(m as usize) - 1] + d as i64) % 7;
    ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][dow as usize]
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
