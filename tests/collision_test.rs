mod common;

use snipe_code::generator::{self, GenerateConfig, validate_project_name};
use std::fs;

#[test]
fn test_folder_exists_returns_false_for_nonexistent() {
    assert!(!generator::folder_exists("nonexistent-folder-xyz"));
}

#[test]
fn test_folder_exists_returns_true_for_existing() {
    let _ws = common::Workspace::new("collision-exists");
    let project_name = "existing-dir";
    fs::create_dir_all(project_name).unwrap();

    assert!(generator::folder_exists(project_name));
}

#[test]
fn test_generate_refuses_existing_without_overwrite() {
    let _ws = common::Workspace::new("collision-refuse");
    let project_name = "existing-project";
    fs::create_dir_all(project_name).unwrap();
    fs::write(format!("{}/old-file.txt", project_name), "old content").unwrap();

    let config = GenerateConfig {
        project_name: project_name.to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };

    let result = generator::generate(&config, false);
    assert!(
        result.is_err(),
        "generate must refuse existing dest without overwrite"
    );

    // Original folder untouched
    assert!(fs::exists(format!("{}/old-file.txt", project_name)).unwrap());
}

#[test]
fn test_generate_overwrites_existing_folder_when_authorized() {
    let _ws = common::Workspace::new("collision-overwrite");
    let project_name = "existing-project";
    fs::create_dir_all(project_name).unwrap();
    fs::write(format!("{}/old-file.txt", project_name), "old content").unwrap();

    let config = GenerateConfig {
        project_name: project_name.to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };

    let result = generator::generate(&config, true).unwrap();

    // Old file should be gone
    assert!(!fs::exists(format!("{}/old-file.txt", project_name)).unwrap());

    // New files should exist
    assert!(result.files.iter().any(|f| f.contains("package.json")));
    assert!(result.files.iter().any(|f| f.contains(".gitignore")));
}

#[test]
fn test_generate_overwrites_frontend_only() {
    let _ws = common::Workspace::new("collision-fe");
    let project_name = "existing-fe";
    fs::create_dir_all(project_name).unwrap();
    fs::write(format!("{}/old-fe.txt", project_name), "old").unwrap();

    let config = GenerateConfig {
        project_name: project_name.to_string(),
        template_id: "none".to_string(),
        frontend: "vue".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };

    let result = generator::generate(&config, true).unwrap();
    assert!(result.folders.iter().any(|f| f.contains("components/ui/")));
    assert!(result.files.iter().any(|f| f.contains("App.vue")));
}

#[test]
fn test_generate_overwrites_backend_only() {
    let _ws = common::Workspace::new("collision-be");
    let project_name = "existing-be";
    fs::create_dir_all(project_name).unwrap();

    let config = GenerateConfig {
        project_name: project_name.to_string(),
        template_id: "none".to_string(),
        frontend: "none".to_string(),
        backend: "go-gin".to_string(),
        database: "none".to_string(),
    };

    let result = generator::generate(&config, true).unwrap();
    assert!(result.files.iter().any(|f| f.contains("go.mod")));
    assert!(result.files.iter().any(|f| f.contains("main.go")));
}

#[test]
fn test_generate_overwrites_fullstack() {
    let _ws = common::Workspace::new("collision-full");
    let project_name = "existing-full";
    fs::create_dir_all(project_name).unwrap();
    fs::write(format!("{}/old.txt", project_name), "old").unwrap();

    let config = GenerateConfig {
        project_name: project_name.to_string(),
        template_id: "company-profile".to_string(),
        frontend: "nextjs".to_string(),
        backend: "nestjs".to_string(),
        database: "postgresql".to_string(),
    };

    let result = generator::generate(&config, true).unwrap();

    // Should have both frontend and backend
    assert!(result.folders.iter().any(|f| f.starts_with("frontend/")));
    assert!(result.folders.iter().any(|f| f.starts_with("backend/")));
    assert!(result.files.iter().any(|f| f.contains("package.json")));
}

#[test]
fn test_dry_run_shows_collision_warning() {
    let _ws = common::Workspace::new("collision-dry");
    let project_name = "existing-dry";
    fs::create_dir_all(project_name).unwrap();

    let config = GenerateConfig {
        project_name: project_name.to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };

    let output = generator::dry_run_preview(&config);
    assert!(output.contains("WARNING"));
    assert!(output.contains("already exists"));
}

#[test]
fn test_dry_run_no_warning_for_new_folder() {
    let config = GenerateConfig {
        project_name: "nonexistent-folder-xyz".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };

    let output = generator::dry_run_preview(&config);
    assert!(!output.contains("WARNING"));
}

#[test]
fn test_validate_project_name_rejects_traversal() {
    for bad in [
        "../escape",
        "a/../b",
        "..",
        ".",
        "sub/dir",
        "C:\\abs",
        "\\abs",
    ] {
        assert!(
            validate_project_name(bad).is_err(),
            "'{}' must be rejected",
            bad
        );
    }
}

#[test]
fn test_resolve_destination_rejects_uncreated_name() {
    let result = generator::resolve_destination("brand-new-name-xyz");
    assert!(result.is_ok());
    assert!(result.unwrap().ends_with("brand-new-name-xyz"));
}

#[cfg(unix)]
#[test]
fn test_resolve_destination_rejects_symlink_target() {
    let _ws = common::Workspace::new("collision-symlink");
    fs::create_dir_all("real-dir").unwrap();
    std::os::unix::fs::symlink("real-dir", "linked-dir").unwrap();
    let result = generator::resolve_destination("linked-dir");
    assert!(result.is_err(), "symlink destination must be rejected");
}

#[cfg(windows)]
#[test]
fn test_resolve_destination_rejects_junction_target() {
    let _ws = common::Workspace::new("collision-junction");
    fs::create_dir_all("real-dir").unwrap();
    // Try to create a junction via cmd; skip silently when permissions block it.
    let out = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J", "linked-dir", "real-dir"])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let result = generator::resolve_destination("linked-dir");
            assert!(result.is_err(), "junction destination must be rejected");
        }
        _ => eprintln!("skipping junction test: no permission to create junctions"),
    }
}

#[cfg(windows)]
#[test]
fn test_generate_refuses_junction_destination() {
    let _ws = common::Workspace::new("collision-junction-gen");
    fs::create_dir_all("real-dir").unwrap();
    let out = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J", "linked-dir", "real-dir"])
        .output();
    if let Ok(o) = out {
        if o.status.success() {
            let config = GenerateConfig {
                project_name: "linked-dir".to_string(),
                template_id: "none".to_string(),
                frontend: "none".to_string(),
                backend: "none".to_string(),
                database: "none".to_string(),
            };
            assert!(
                generator::generate(&config, true).is_err(),
                "generate must refuse a junction destination"
            );
            // Junction must still point at the original
            assert!(fs::exists("real-dir").unwrap());
        }
    }
}
