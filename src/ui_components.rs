use std::fs;
use std::path::Path;
use std::process::Command;

pub fn install_shadcn(project_path: &str, frontend: &str) -> Result<Vec<String>, String> {
    if frontend == "none" {
        return Ok(Vec::new());
    }

    // Only support Node.js based frameworks
    match frontend {
        "nextjs" | "react-vite" | "vue" | "nuxtjs" => {}
        _ => return Ok(Vec::new()),
    }

    let frontend_path = Path::new(project_path).join("frontend");
    if !frontend_path.exists() {
        return Err("Frontend folder not found".to_string());
    }

    let mut installed = Vec::new();

    // Install shadcn/ui
    let output = Command::new("npx")
        .args(["shadcn@latest", "init", "-d"])
        .current_dir(&frontend_path)
        .output()
        .map_err(|e| format!("Failed to init shadcn: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("shadcn init failed: {}", stderr));
    }

    installed.push("shadcn/ui initialized".to_string());

    // Install essential components
    let components = vec!["button", "card", "input", "table", "sidebar"];
    for component in &components {
        let output = Command::new("npx")
            .args(["shadcn@latest", "add", component, "-y"])
            .current_dir(&frontend_path)
            .output()
            .map_err(|e| format!("Failed to add component {}: {}", component, e))?;

        if output.status.success() {
            installed.push(format!("Added {}", component));
        }
    }

    Ok(installed)
}

pub fn generate_ui_components_list(project_path: &str, frontend: &str) -> Result<(), String> {
    if frontend == "none" {
        return Ok(());
    }

    let path = Path::new(project_path);
    let components = vec!["button", "card", "input", "table", "sidebar"];
    let content = format!(
        "# UI Components\n\n\
         Auto-installed by SNIPE-CODE CLI:\n\n\
         {}\n",
        components.iter().map(|c| format!("- {}", c)).collect::<Vec<_>>().join("\n")
    );

    fs::write(path.join("frontend/UI_COMPONENTS.md"), content).map_err(|e| e.to_string())?;

    Ok(())
}
