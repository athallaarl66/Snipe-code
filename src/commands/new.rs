use dialoguer::{Select, Input};
use console::Style;

use crate::template::TemplateRegistry;
use crate::stack::{self, FRONTEND_STACKS, BACKEND_STACKS, DATABASE_STACKS};
use crate::generator::{GenerateConfig, generate, dry_run_preview, folder_exists};
use crate::git;
use crate::cicd;
use crate::docker;
use crate::ai_workspace;

pub fn run(dry_run: bool) {
    let header_style = Style::new().bold().cyan();
    let success_style = Style::new().bold().green();
    let warning_style = Style::new().bold().yellow();

    println!("{}", header_style.apply_to("SNIPE-CODE CLI"));
    println!("Enterprise Dev-Forge\n");

    // 1. Select Template
    let templates = TemplateRegistry::load_all();
    let template_names: Vec<String> = templates.iter()
        .map(|t| format!("{} {} — {}", t.icon, t.name, t.description))
        .collect();

    let template_idx = Select::new()
        .with_prompt("Pilih Template")
        .items(&template_names)
        .default(0)
        .interact()
        .expect("Failed to read template selection");

    let selected_template = &templates[template_idx];

    // 2. Select Frontend
    let frontend_names: Vec<&str> = FRONTEND_STACKS.iter().map(|s| s.name).collect();
    let frontend_idx = Select::new()
        .with_prompt("Pilih Frontend")
        .items(&frontend_names)
        .default(0)
        .interact()
        .expect("Failed to read frontend selection");

    let selected_frontend = &FRONTEND_STACKS[frontend_idx];

    // 3. Select Backend
    let backend_names: Vec<&str> = BACKEND_STACKS.iter().map(|s| s.name).collect();
    let backend_idx = Select::new()
        .with_prompt("Pilih Backend")
        .items(&backend_names)
        .default(0)
        .interact()
        .expect("Failed to read backend selection");

    let selected_backend = &BACKEND_STACKS[backend_idx];

    // 4. Validate selection
    if let Err(e) = stack::validate_selection(selected_frontend.id, selected_backend.id) {
        println!("Error: {}", e);
        return;
    }

    // 5. Select Database
    let database_names: Vec<&str> = DATABASE_STACKS.iter().map(|s| s.name).collect();
    let database_idx = Select::new()
        .with_prompt("Pilih Database")
        .items(&database_names)
        .default(0)
        .interact()
        .expect("Failed to read database selection");

    let selected_database = &DATABASE_STACKS[database_idx];

    // 6. Project Name
    let project_name: String = Input::new()
        .with_prompt("Project Name")
        .default("my-project".to_string())
        .interact_text()
        .expect("Failed to read project name");

    // 7. Check folder collision
    if folder_exists(&project_name) {
        println!("{}", warning_style.apply_to(format!(
            "\nError: Folder \"{}\" already exists", project_name
        )));
        
        let options = vec!["Overwrite (delete existing)", "Cancel"];
        let choice = Select::new()
            .with_prompt("What to do?")
            .items(&options)
            .default(1)
            .interact()
            .expect("Failed to read choice");

        if choice == 1 {
            println!("Cancelled. No changes made.");
            return;
        }
        println!("Overwriting existing folder...");
    }

    // Build config
    let config = GenerateConfig {
        project_name: project_name.clone(),
        template_id: selected_template.id.to_string(),
        frontend: selected_frontend.id.to_string(),
        backend: selected_backend.id.to_string(),
        database: selected_database.id.to_string(),
    };

    // Dry-run mode
    if dry_run {
        println!("\n[DRY RUN] Would create:\n");
        println!("{}", dry_run_preview(&config));
        return;
    }

    // Generate
    println!("\nGenerating project structure...");
    match generate(&config) {
        Ok(result) => {
            println!("{}", success_style.apply_to("Project created!"));
            println!("  Folders: {}", result.folders.len());
            println!("  Files: {}", result.files.len());
        }
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    }

    // Post-generation steps
    println!("\nPost-generation setup...");

    // CI/CD
    if let Err(e) = cicd::generate_ci_cd(&project_name, selected_frontend.id, selected_backend.id) {
        println!("  CI/CD warning: {}", e);
    } else if selected_frontend.id != "none" || selected_backend.id != "none" {
        println!("  ✓ CI/CD workflow generated");
    }

    // Docker Compose
    if let Err(e) = docker::generate_docker_compose(&project_name, selected_database.id, selected_backend.id) {
        println!("  Docker warning: {}", e);
    } else if selected_database.id != "none" {
        println!("  ✓ Docker Compose generated");
    }

    // AI Workspace
    if let Err(e) = ai_workspace::generate_ai_workspace(&project_name) {
        println!("  AI Workspace warning: {}", e);
    } else {
        println!("  ✓ AI Workspace created");
    }

    // Git
    if git::is_git_installed() {
        if let Err(e) = git::init(&project_name) {
            println!("  Git init warning: {}", e);
        } else {
            println!("  ✓ Git initialized");
            if let Err(e) = git::add_and_commit(&project_name, &selected_template.name, selected_frontend.name, selected_backend.name) {
                println!("  Git commit warning: {}", e);
            } else {
                println!("  ✓ Initial commit made");
            }
        }
    } else {
        println!("  ⚠ Git not installed — skipping git init");
    }

    println!("\n{}", success_style.apply_to(format!("Done! Project '{}' is ready.", project_name)));
}
