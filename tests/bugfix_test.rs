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
    };
    generator::generate(&config, false).unwrap();
    let content = fs::read_to_string("app/.gitignore").unwrap();
    let count = content.lines().filter(|l| l.trim() == "vendor/").count();
    assert_eq!(count, 1, "vendor/ must appear exactly once, got {}", count);
}

#[test]
fn test_workflow_uses_npm_install_not_ci() {
    let dir = temp_project("workflow");
    cicd::generate_ci_cd(&dir.to_string_lossy(), "nextjs", "none").unwrap();
    let content = fs::read_to_string(dir.join(".github/workflows/main.yml")).unwrap();
    assert!(!content.contains("npm ci"), "must not use npm ci");
    assert!(content.contains("npm install"), "must use npm install");
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
    };
    let cfg_b = GenerateConfig {
        project_name: "proj-b".to_string(),
        template_id: "none".to_string(),
        frontend: "vue".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
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
