mod common;

use std::fs;
use std::path::PathBuf;

use snipe_code::audit;
use snipe_code::cicd;
use snipe_code::generator::{self, GenerateConfig};

#[test]
fn test_nest_cli_json_is_valid() {
    let _ws = common::Workspace::new("bf-nest");
    let config = GenerateConfig {
        project_name: "app".to_string(),
        template_id: "none".to_string(),
        frontend: "none".to_string(),
        backend: "nestjs".to_string(),
        database: "none".to_string(),
        ai_tools: Vec::new(),
    };
    generator::generate(&config, false).unwrap();
    let content = fs::read_to_string("app/backend/nest-cli.json").unwrap();
    let json: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|_| panic!("nest-cli.json must be valid JSON, got: {}", content));
    assert!(json.get("$schema").is_some());
    assert_eq!(json.get("sourceRoot").and_then(|v| v.as_str()), Some("src"));
}

#[test]
fn test_laravel_env_example_has_app_key_placeholder() {
    let _ws = common::Workspace::new("bf-laravel");
    let config = GenerateConfig {
        project_name: "app".to_string(),
        template_id: "none".to_string(),
        frontend: "none".to_string(),
        backend: "laravel".to_string(),
        database: "none".to_string(),
        ai_tools: Vec::new(),
    };
    generator::generate(&config, false).unwrap();
    // No real .env is generated — only an environment template (safe default)
    assert!(
        !fs::exists("app/backend/.env").unwrap(),
        "must not generate a real .env"
    );
    let content = fs::read_to_string("app/backend/.env.example").unwrap();
    assert!(content.contains("APP_KEY="));
    assert!(
        content.contains("CHANGE_ME"),
        ".env.example must use replaceable placeholders:\n{}",
        content
    );
}

#[test]
fn test_gitignore_no_duplicate_vendor() {
    let _ws = common::Workspace::new("bf-gi");
    let config = GenerateConfig {
        project_name: "app".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
        ai_tools: Vec::new(),
    };
    generator::generate(&config, false).unwrap();
    let content = fs::read_to_string("app/.gitignore").unwrap();
    let count = content.lines().filter(|l| l.trim() == "vendor/").count();
    assert_eq!(count, 1, "vendor/ must appear exactly once, got {}", count);
    for expected in [
        "node_modules/",
        "bin/",
        "obj/",
        "Audit_Report.md",
        "Audit_Report.pdf",
        "Audit_Report.docx",
    ] {
        assert!(
            content.contains(expected),
            "generated .gitignore must ignore {expected}:\n{content}"
        );
    }
}

#[test]
fn test_gitignore_ignores_verify_and_audit_output() {
    let _ws = common::Workspace::new("bf-gi-out");
    let config = GenerateConfig {
        project_name: "app".to_string(),
        template_id: "none".to_string(),
        frontend: "none".to_string(),
        backend: "dotnet8".to_string(),
        database: "none".to_string(),
        ai_tools: Vec::new(),
    };
    generator::generate(&config, false).unwrap();
    let content = fs::read_to_string("app/.gitignore").unwrap();
    assert!(
        content.contains("obj/") && content.contains("bin/"),
        "a .NET project must ignore its build output:\n{content}"
    );
}

#[test]
fn test_workflow_uses_npm_install_not_ci() {
    let dir = temp_project("workflow");
    cicd::generate_ci_cd(&dir.to_string_lossy(), "nextjs", "none").unwrap();
    let content = fs::read_to_string(dir.join(".github/workflows/main.yml")).unwrap();
    assert!(!content.contains("npm ci"), "must not use npm ci");
    assert!(content.contains("npm install"), "must use npm install");
    assert!(
        !content.contains("npm run lint"),
        "must not invoke undeclared lint script"
    );
}

#[test]
fn test_non_container_backend_omits_backend_compose_service() {
    let dir = temp_project("compose-go");
    snipe_code::docker::generate_docker_compose(&dir.to_string_lossy(), "postgresql", "go-gin")
        .unwrap();
    let content = fs::read_to_string(dir.join("docker-compose.yml")).unwrap();
    assert!(content.contains("postgres:"));
    assert!(!content.contains("  backend:"));
    let _ = fs::remove_dir_all(dir);
}

/// Regression: npm 10's arborist crashes on the Nuxt 3 peer graph
/// (`Cannot read properties of null (reading 'edgesOut')`), so the generated
/// workflow must pin a Node line that ships npm 11+, and the Laravel job must
/// match the PHP line the generated composer.json requires (`^8.2`).
#[test]
fn test_workflow_pins_npm11_node_and_supported_php() {
    let dir = temp_project("workflow-toolchain");
    cicd::generate_ci_cd(&dir.to_string_lossy(), "nuxtjs", "laravel").unwrap();
    let content = fs::read_to_string(dir.join(".github/workflows/main.yml")).unwrap();
    assert!(
        content.contains("node-version: '24'"),
        "must pin Node 24 (npm 11) for the generated frontend job:\n{}",
        content
    );
    assert!(
        !content.contains("node-version: '20'"),
        "Node 20 ships npm 10, which cannot install the Nuxt 3 peer graph"
    );
    assert!(
        content.contains("php-version: '8.2'"),
        "Laravel job must use the PHP line composer.json requires:\n{}",
        content
    );
    let _ = fs::remove_dir_all(dir);
}

fn temp_project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("snipe-bf-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_audit_scans_dotenv_secret() {
    let dir = temp_project("dotenv-secret");
    fs::write(dir.join(".env"), "DB_PASSWORD=supersecret123\n").unwrap();
    let path = dir.to_string_lossy().to_string();
    let exported = audit::run_audit(&path, "md").unwrap();
    assert!(exported.iter().any(|f| f == "Audit_Report.md"));
    let report = fs::read_to_string(dir.join("Audit_Report.md")).unwrap();
    assert!(
        report.contains("Hardcoded Secret"),
        "report missing finding:\n{}",
        report
    );
    assert!(
        report.contains(".env"),
        "report missing .env ref:\n{}",
        report
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_audit_ignores_placeholder_dotenv() {
    let dir = temp_project("dotenv-placeholder");
    fs::write(dir.join(".env"), "API_KEY=your-key-here\n").unwrap();
    let path = dir.to_string_lossy().to_string();
    audit::run_audit(&path, "md").unwrap();
    let report = fs::read_to_string(dir.join("Audit_Report.md")).unwrap();
    assert!(
        !report.contains("Hardcoded Secret"),
        "placeholder flagged:\n{}",
        report
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_audit_validates_formats_and_reports_scopes() {
    let dir = temp_project("audit-contract");
    fs::create_dir_all(dir.join("frontend/src")).unwrap();
    fs::write(
        dir.join("frontend/package.json"),
        r#"{"dependencies":{"react":"1"}}"#,
    )
    .unwrap();
    fs::write(dir.join("frontend/src/app.ts"), "const value = 1;\n").unwrap();
    let path = dir.to_string_lossy().to_string();
    let report = audit::run_audit_report(&path, "md,md").unwrap();
    assert_eq!(report.exported_files, vec!["Audit_Report.md"]);
    assert!(
        report
            .detected_scopes
            .iter()
            .any(|scope| scope.contains("javascript:react"))
    );
    assert!(audit::run_audit_report(&path, "md,").is_err());
    assert!(audit::run_audit_report(&path, "yaml").is_err());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn test_readme_uses_supported_quality_and_commands() {
    let readme = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md")).unwrap();
    assert!(!readme.contains("Enterprise-Grade"));
    assert!(!readme.contains("50+ automated tests"));
    assert!(readme.contains("cargo clippy --all-targets --all-features -- -D warnings"));
    assert!(readme.contains("cosign verify-blob --bundle"));
}

#[test]
fn test_laravel_scaffold_uses_supported_framework_line() {
    let files = snipe_code::stack::get_backend_files("laravel").unwrap();
    let composer = files
        .iter()
        .find(|file| file.path == "composer.json")
        .unwrap();
    assert!(
        composer
            .content
            .contains("\"laravel/framework\": \"^12.0\"")
    );
    assert!(composer.content.contains("\"php\": \"^8.2\""));
    assert!(
        !composer
            .content
            .contains("\"laravel/framework\": \"^11.0\""),
        "Laravel 11 line is advisory-blocked for Composer"
    );
    assert!(
        !composer
            .content
            .contains("\"laravel/framework\": \"^10.10\"")
    );
}

/// Regression: generating twice into the same workspace must not delete the
/// first generated project when the second run is refused (no overwrite).
#[test]
fn test_repeated_generate_without_overwrite_keeps_first_project() {
    let _ws = common::Workspace::new("reg-repeat");
    let config = GenerateConfig {
        project_name: "app".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
        ai_tools: Vec::new(),
    };

    generator::generate(&config, false).unwrap();
    let marker = "app/frontend/package.json";
    assert!(
        fs::exists(marker).unwrap(),
        "first run must create {}",
        marker
    );

    // Second run without overwrite must fail cleanly and leave the project intact
    assert!(generator::generate(&config, false).is_err());
    assert!(
        fs::exists(marker).unwrap(),
        "first project must survive refused second run"
    );
}

/// Regression: two concurrent generations into distinct names must not collide
/// with each other or with a pre-existing repository fixture file.
#[test]
fn test_parallel_generate_distinct_names_do_not_clobber_fixtures() {
    let _ws = common::Workspace::new("reg-parallel");

    // Repository fixture the generator must never touch
    let fixture = "repo-fixture-keep.txt";
    fs::write(fixture, "do not delete\n").unwrap();

    let cfg_a = GenerateConfig {
        project_name: "proj-a".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
        ai_tools: Vec::new(),
    };
    let cfg_b = GenerateConfig {
        project_name: "proj-b".to_string(),
        template_id: "none".to_string(),
        frontend: "vue".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
        ai_tools: Vec::new(),
    };

    let a = std::thread::spawn(move || generator::generate(&cfg_a, false).map(|_| ()));
    let b = std::thread::spawn(move || generator::generate(&cfg_b, false).map(|_| ()));

    a.join().unwrap().unwrap();
    b.join().unwrap().unwrap();

    assert!(fs::exists("proj-a/frontend/package.json").unwrap());
    assert!(fs::exists("proj-b/frontend/package.json").unwrap());
    assert_eq!(fs::read_to_string(fixture).unwrap(), "do not delete\n");
}

/// Staging directory must not linger next to the published destination.
#[test]
fn test_generate_leaves_no_staging_dir_after_success() {
    let _ws = common::Workspace::new("reg-staging");
    let config = GenerateConfig {
        project_name: "app".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
        ai_tools: Vec::new(),
    };
    generator::generate(&config, false).unwrap();

    assert!(fs::exists("app").unwrap());
    let leftovers: Vec<String> = fs::read_dir(".")
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.contains("snipe-staging"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "staging dir left behind: {:?}",
        leftovers
    );
}

#[test]
fn test_failed_generate_cleans_staging() {
    let _ws = common::Workspace::new("reg-failed");
    // Unsafe name that passes config but fails validation mid-flight would be
    // refused before staging; here we force failure via an already-existing
    // file at the destination path to exercise the cleanup path.
    fs::create_dir_all("blocker").unwrap();
    fs::write("blocker/occupied.txt", "occupied").unwrap();
    let config = GenerateConfig {
        project_name: "blocker".to_string(),
        template_id: "none".to_string(),
        frontend: "none".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
        ai_tools: Vec::new(),
    };
    // generate() refuses existing dest without overwrite BEFORE staging begins,
    // so no staging dir should ever appear either.
    assert!(generator::generate(&config, false).is_err());
    assert!(
        fs::exists("blocker/occupied.txt").unwrap(),
        "blocker must stay untouched"
    );
    let leftovers: Vec<String> = fs::read_dir(".")
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.contains("snipe-staging"))
        .collect();
    assert!(leftovers.is_empty(), "staging dir leaked: {:?}", leftovers);
}

/// Regression: the exported Markdown report must carry the real system date,
/// never a hardcoded placeholder.
#[test]
fn test_markdown_report_uses_real_system_date() {
    let dir = temp_project("report-date");
    fs::write(dir.join(".env.example"), "FOO=bar\n").unwrap();

    let before = snipe_code::date::real_date_yyyymmdd();
    audit::run_audit(&dir.to_string_lossy(), "md").unwrap();
    let after = snipe_code::date::real_date_yyyymmdd();

    let report = fs::read_to_string(dir.join("Audit_Report.md")).unwrap();
    let date_line = report
        .lines()
        .find(|line| line.starts_with("**Date:**"))
        .expect("report must carry a Date line");
    let value = date_line.trim_start_matches("**Date:**").trim();

    assert_eq!(value.len(), 10, "date must be YYYY-MM-DD, got {:?}", value);
    let non_digits: Vec<char> = value.chars().filter(|c| !c.is_ascii_digit()).collect();
    assert_eq!(
        non_digits,
        vec!['-', '-'],
        "date must parse as YYYY-MM-DD, got {:?}",
        value
    );
    assert!(
        value == before || value == after,
        "report date {value} must equal the shared helper output ({before} / {after})"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Regression: generated projects must contain the AI agent files the README
/// advertises, at the project root.
#[test]
fn test_generated_project_contains_advertised_ai_agent_files() {
    let _ws = common::Workspace::new("reg-ai-files");
    let config = GenerateConfig {
        project_name: "app".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
        ai_tools: vec!["agents-md".to_string(), "cursor".to_string()],
    };
    let result = generator::generate(&config, false).unwrap();

    for file in [".cursorrules", "AGENTS.md"] {
        assert!(
            fs::exists(format!("app/{file}")).unwrap(),
            "{file} must exist at the project root"
        );
        assert!(
            result.files.iter().any(|f| f == file),
            "{file} must appear in the generation report: {:?}",
            result.files
        );
    }
}

/// Regression: every reported path must resolve inside the destination project
/// tree — generation must never touch a home-directory agent configuration.
#[test]
fn test_generated_files_stay_inside_project_tree() {
    let _ws = common::Workspace::new("reg-containment");
    let config = GenerateConfig {
        project_name: "app".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "nestjs".to_string(),
        database: "postgresql".to_string(),
        ai_tools: Vec::new(),
    };
    let result = generator::generate(&config, false).unwrap();
    let dest = generator::resolve_destination(&config.project_name).unwrap();

    for file in &result.files {
        let relative = std::path::Path::new(file);
        assert!(
            !relative.is_absolute(),
            "generated path must be project-relative: {file}"
        );
        let has_parent = file.split('/').any(|part| part == "..");
        assert!(
            !has_parent,
            "generated path must not escape the project tree: {file}"
        );
        let joined = dest.join(file);
        assert!(
            joined.starts_with(&dest),
            "generated path escapes the project tree: {file}"
        );
        assert!(joined.exists(), "reported file was not written: {file}");
    }
}

/// Regression: README feature bullets must only name artifacts the generator
/// actually produces.
#[test]
fn test_readme_documented_outputs_are_generated() {
    let readme = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md")).unwrap();
    let documented = [
        ".cursorrules",
        "AGENTS.md",
        ".ai-workspace/rules.md",
        ".ai-workspace/rules.mdc",
        ".env.example",
        ".gitignore",
        "docker-compose.yml",
    ];
    for token in documented {
        assert!(readme.contains(token), "README must document {token}");
    }

    let _ws = common::Workspace::new("reg-readme-outputs");
    let config = GenerateConfig {
        project_name: "app".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "nestjs".to_string(),
        database: "postgresql".to_string(),
        ai_tools: vec!["agents-md".to_string(), "cursor".to_string()],
    };
    let result = generator::generate(&config, false).unwrap();
    for token in documented {
        assert!(
            result.files.iter().any(|f| f == token),
            "README documents {token} but generation never produces it: {:?}",
            result.files
        );
    }
}
