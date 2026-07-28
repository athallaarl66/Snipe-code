use std::process::Command;
use std::path::Path;

pub fn init(project_path: &str) -> Result<(), String> {
    let path = Path::new(project_path);
    if !path.exists() {
        return Err(format!("Folder '{}' does not exist", project_path));
    }

    // git init
    let output = Command::new("git")
        .arg("init")
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to run git init: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git init failed: {}", stderr));
    }

    Ok(())
}

pub fn add_and_commit(project_path: &str, template_name: &str, frontend: &str, backend: &str) -> Result<(), String> {
    let path = Path::new(project_path);
    if !path.exists() {
        return Err(format!("Folder '{}' does not exist", project_path));
    }

    // git add .
    let output = Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to run git add: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git add failed: {}", stderr));
    }

    // git commit
    let commit_msg = format!("Initial commit: {} + {} + {}", template_name, frontend, backend);
    let output = Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to run git commit: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git commit failed: {}", stderr));
    }

    Ok(())
}

pub fn is_git_installed() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
