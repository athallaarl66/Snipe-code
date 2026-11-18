use std::fs;
use std::process::{Command, Stdio};

#[test]
fn noninteractive_dry_run_preserves_flags_and_json_without_writes() {
    let root = std::env::temp_dir().join(format!("snipe-wizard-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_snipe-code"))
        .current_dir(&root)
        .args([
            "new",
            "--name",
            "wizard-project",
            "--template",
            "blog",
            "--frontend",
            "nextjs",
            "--backend",
            "none",
            "--database",
            "none",
            "--ai-tools",
            "none",
            "--dry-run",
            "--json",
        ])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["outcome"], "dry-run");
    assert_eq!(json["project"], "wizard-project");
    assert!(fs::read_dir(&root).unwrap().next().is_none());
    fs::remove_dir(root).unwrap();
}

#[test]
fn noninteractive_collision_leaves_existing_files_unchanged() {
    let root = std::env::temp_dir().join(format!("snipe-wizard-collision-{}", std::process::id()));
    let project = root.join("existing");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join("keep.txt"), b"original").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_snipe-code"))
        .current_dir(&root)
        .args(["new", "--name", "existing"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read(project.join("keep.txt")).unwrap(), b"original");
    assert_eq!(fs::read_dir(&project).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preset_descriptions_match_declared_structure() {
    use snipe_code::template::TemplateRegistry;
    let none = TemplateRegistry::get("none").unwrap();
    assert!(none.structure.files.is_empty());
    assert!(none.description.contains("selected stack"));
    let company = TemplateRegistry::get("company-profile").unwrap();
    assert!(company.structure.files.is_empty());
    assert!(company.description.contains("folders"));
    let blog = TemplateRegistry::get("blog").unwrap();
    assert!(blog.description.contains("static RSS"));
    assert!(
        blog.structure
            .files
            .iter()
            .any(|f| f.path.ends_with(".mdx"))
    );
    let portfolio = TemplateRegistry::get("portfolio").unwrap();
    assert!(
        portfolio
            .structure
            .files
            .iter()
            .any(|f| f.path.ends_with("projects.json"))
    );
}
