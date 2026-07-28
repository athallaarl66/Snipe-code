use dialoguer::{Select, Input};
use console::Style;

use crate::template::TemplateRegistry;
use crate::stack::{self, FRONTEND_STACKS, BACKEND_STACKS, DATABASE_STACKS};
use crate::generator::{GenerateConfig, generate, dry_run_preview};

pub fn run(dry_run: bool) {
    let header_style = Style::new().bold().cyan();
    let success_style = Style::new().bold().green();

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

    // Build config
    let config = GenerateConfig {
        project_name,
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
            println!("{}", success_style.apply_to(format!("Project created!")));
            println!("  Folders: {}", result.folders.len());
            println!("  Files: {}", result.files.len());
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
