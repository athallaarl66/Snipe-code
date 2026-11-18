use snipe_code::generator::{GenerateConfig, generate};
use std::{fs, path::Path, process::Command};

#[test]
fn generation_child() {
    if std::env::var_os("SNIPE_SAFETY_CHILD").is_none() {
        return;
    }
    let config = GenerateConfig {
        project_name: "project".into(),
        template_id: "blog".into(),
        frontend: std::env::var("SNIPE_TEST_FRONTEND").unwrap_or_else(|_| "none".into()),
        backend: "none".into(),
        database: "none".into(),
        ai_tools: vec![],
    };
    let result = generate(&config, true);
    if std::env::var_os("SNIPE_SAFETY_ACCEPT").is_some() {
        assert!(result.is_ok(), "valid override rejected: {result:?}");
    } else {
        assert!(result.is_err(), "unsafe override accepted: {result:?}");
    }
}

fn reject(root: &Path, overrides: &Path) {
    let output = Command::new(std::env::current_exe().unwrap())
        .current_dir(root)
        .env("SNIPE_SAFETY_CHILD", "1")
        .env("SNIPE_CODE_TEMPLATES", overrides)
        .args(["--exact", "generation_child", "--nocapture"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(
        fs::read_to_string(root.join("canary.txt")).unwrap(),
        "canary"
    );
    assert_eq!(
        fs::read_to_string(root.join("project/old.txt")).unwrap(),
        "old"
    );
    assert_eq!(
        fs::read_dir(root).unwrap().count(),
        3,
        "staging artifact remained"
    );
}

#[test]
fn malicious_preset_and_links_leave_destination_and_canary_unchanged() {
    let root = std::env::var_os("TMPDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(format!("snipe-template-safety-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let overrides = root.join("overrides");
    fs::create_dir_all(overrides.join("presets/blog")).unwrap();
    fs::create_dir(root.join("project")).unwrap();
    fs::write(root.join("project/old.txt"), "old").unwrap();
    fs::write(root.join("canary.txt"), "canary").unwrap();
    let preset = overrides.join("presets/blog/preset.json");
    for value in [
        "../canary.txt",
        "/absolute",
        "C:/drive",
        "C:drive",
        "\\\\server\\share",
        "a/../../canary.txt",
        "a\\..\\canary.txt",
    ] {
        for field in ["folder", "file", "source"] {
            let (folders, files) = match field {
                "folder" => (vec![value], vec![]),
                "file" => (
                    vec![],
                    vec![serde_json::json!({"path":value,"template_source":"blog-sample-post"})],
                ),
                _ => (
                    vec![],
                    vec![serde_json::json!({"path":"safe.txt","template_source":value})],
                ),
            };
            let json = serde_json::json!({"id":"blog","name":"Blog","description":"","icon":"", "structure":{"folders":folders,"files":files}});
            fs::write(&preset, json.to_string()).unwrap();
            reject(&root, &overrides);
        }
    }
    fs::remove_file(&preset).unwrap();
    let stack_folders = overrides.join("frontends/nextjs/folders.txt");
    fs::create_dir_all(stack_folders.parent().unwrap()).unwrap();
    fs::write(&stack_folders, "../../canary.txt\n").unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .current_dir(&root)
        .env("SNIPE_SAFETY_CHILD", "1")
        .env("SNIPE_TEST_FRONTEND", "nextjs")
        .env("SNIPE_CODE_TEMPLATES", &overrides)
        .args(["--exact", "generation_child", "--nocapture"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(
        fs::read_to_string(root.join("project/old.txt")).unwrap(),
        "old"
    );
    assert_eq!(
        fs::read_to_string(root.join("canary.txt")).unwrap(),
        "canary"
    );
    assert_eq!(fs::read_dir(&root).unwrap().count(), 3);
    fs::remove_file(&stack_folders).unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("canary.txt"), &preset).unwrap();
        reject(&root, &overrides);
        fs::remove_file(&preset).unwrap();
        std::os::unix::fs::symlink(&root, overrides.join("outside")).unwrap();
        reject(&root, &overrides);
        fs::remove_file(overrides.join("outside")).unwrap();
    }
    #[cfg(windows)]
    {
        match std::os::windows::fs::symlink_file(root.join("canary.txt"), &preset) {
            Ok(()) => {
                reject(&root, &overrides);
                fs::remove_file(&preset).unwrap();
            }
            Err(error) if error.raw_os_error() == Some(1314) => {
                eprintln!("SKIPPED file symlink: privilege unavailable")
            }
            Err(error) => panic!("file symlink: {error}"),
        }
        let link = overrides.join("outside");
        let status = Command::new("cmd.exe")
            .args(["/C", "mklink", "/J"])
            .arg(&link)
            .arg(&root)
            .status()
            .unwrap();
        assert!(status.success(), "junction creation failed");
        reject(&root, &overrides);
        fs::remove_dir(link).unwrap();
    }
    let valid = serde_json::json!({"id":"blog","name":"Blog","description":"","icon":"", "structure":{"folders":["safe/nested/"],"files":[{"path":"safe/post.txt","template_source":"blog-sample-post"}]}});
    fs::write(&preset, valid.to_string()).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .current_dir(&root)
        .env("SNIPE_SAFETY_CHILD", "1")
        .env("SNIPE_SAFETY_ACCEPT", "1")
        .env("SNIPE_CODE_TEMPLATES", &overrides)
        .args(["--exact", "generation_child", "--nocapture"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(root.join("project/safe/post.txt").is_file());
    assert_eq!(
        fs::read_to_string(root.join("canary.txt")).unwrap(),
        "canary"
    );
    fs::remove_dir_all(root).unwrap();
}
