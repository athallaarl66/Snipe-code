use snipe_code::generator::{GenerateConfig, generate};
use std::{fs, path::Path, process::Command};

#[test]
fn generator_child() {
    if std::env::var_os("SNIPE_ANCESTOR_CHILD").is_none() {
        return;
    }
    let config = GenerateConfig {
        project_name: "project".into(),
        template_id: "blog".into(),
        frontend: "none".into(),
        backend: "none".into(),
        database: "none".into(),
        ai_tools: vec![],
    };
    let error = generate(&config, true).unwrap_err();
    assert!(error.contains("template path contains a link"), "{error}");
}

fn check_rejected(work: &Path, overrides: &Path) {
    let generator = Command::new(std::env::current_exe().unwrap())
        .current_dir(work)
        .env("SNIPE_ANCESTOR_CHILD", "1")
        .env("SNIPE_CODE_TEMPLATES", overrides)
        .args(["--exact", "generator_child", "--nocapture"])
        .output()
        .unwrap();
    assert!(
        generator.status.success(),
        "{}",
        String::from_utf8_lossy(&generator.stdout)
    );
    for args in [&["new"][..], &["new", "--dry-run"][..], &["--dry-run"][..]] {
        let output = Command::new(env!("CARGO_BIN_EXE_snipe-code"))
            .current_dir(work)
            .env("SNIPE_CODE_TEMPLATES", overrides)
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{args:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("template path contains a link"),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert_eq!(
        fs::read_to_string(work.join("project/old.txt")).unwrap(),
        "old"
    );
}

#[test]
fn ancestor_link_is_rejected_before_cli_or_generator_reads_overrides() {
    let base = std::env::var_os("TMPDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .canonicalize()
        .unwrap();
    let work = base.join(format!(
        "snipe-ancestor-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&work).unwrap();
    let target = work.join("target");
    fs::create_dir_all(target.join("overrides")).unwrap();
    fs::create_dir(work.join("project")).unwrap();
    fs::write(work.join("project/old.txt"), "old").unwrap();

    // A normal override under the same temp ancestors must not be rejected.
    let output = Command::new(env!("CARGO_BIN_EXE_snipe-code"))
        .current_dir(&work)
        .env("SNIPE_CODE_TEMPLATES", target.join("overrides"))
        .args(["new", "--dry-run", "--name", "new-project"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!work.join("new-project").exists());

    let link = work.join("linked");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, &link).unwrap();
    #[cfg(windows)]
    {
        let status = Command::new("cmd.exe")
            .args(["/C", "mklink", "/J"])
            .arg(&link)
            .arg(&target)
            .status()
            .unwrap();
        assert!(status.success(), "junction creation failed");
    }
    check_rejected(&work, &link.join("overrides"));
    // A missing root below a link must not silently fall back to built-ins.
    check_rejected(&work, &link.join("missing"));
    #[cfg(unix)]
    fs::remove_file(&link).unwrap();
    #[cfg(windows)]
    fs::remove_dir(&link).unwrap();
    fs::remove_dir_all(work).unwrap();
}
