use console::Style;
use dialoguer::{Input, Select};

use crate::generator::{
    GenerateConfig, dry_run_preview, folder_exists, generate, resolve_destination,
    validate_project_name,
};
use crate::git;
use crate::stack::{self, BACKEND_STACKS, DATABASE_STACKS, FRONTEND_STACKS};
use crate::template::TemplateRegistry;

pub fn run(dry_run: bool) -> Result<(), String> {
    let header_style = Style::new().bold().cyan();
    let success_style = Style::new().bold().green();
    let warning_style = Style::new().bold().yellow();

    println!("{}", header_style.apply_to("SNIPE-CODE CLI"));
    println!("Enterprise Dev-Forge\n");

    // 1. Select Template
    let templates = TemplateRegistry::load_all();
    let template_names: Vec<String> = templates
        .iter()
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
        return Err(e.to_string());
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

    // 6. Project Name — validate before any filesystem use
    let project_name: String = Input::new()
        .with_prompt("Project Name")
        .default("my-project".to_string())
        .interact_text()
        .expect("Failed to read project name");

    if let Err(e) = validate_project_name(&project_name) {
        println!("Error: {}", e);
        return Err(e);
    }

    let dest = resolve_destination(&project_name)?;
    let mut overwrite = false;

    // 7. Check folder collision — show canonical destination
    if folder_exists(&project_name) {
        println!(
            "{}",
            warning_style.apply_to(format!(
                "\nError: Folder \"{}\" already exists",
                dest.display()
            ))
        );

        let options = vec!["Overwrite (replace existing)", "Cancel"];
        let choice = Select::new()
            .with_prompt("What to do?")
            .items(&options)
            .default(1)
            .interact()
            .expect("Failed to read choice");

        if choice == 1 {
            println!("Cancelled. No changes made.");
            return Ok(());
        }
        println!("Overwriting existing folder...");
        overwrite = true;
    }

    // Build config
    let config = GenerateConfig {
        project_name: project_name.clone(),
        template_id: selected_template.id.to_string(),
        frontend: selected_frontend.id.to_string(),
        backend: selected_backend.id.to_string(),
        database: selected_database.id.to_string(),
    };

    // Dry-run mode — preview only, no filesystem mutation
    if dry_run {
        println!("\n[DRY RUN] Would create:\n");
        println!("{}", dry_run_preview(&config));
        return Ok(());
    }

    // Generate (staged, published only after successful validation)
    println!("\nGenerating project structure...");
    match generate(&config, overwrite) {
        Ok(result) => {
            println!("{}", success_style.apply_to("Project created!"));
            println!("  Folders: {}", result.folders.len());
            println!("  Files: {}", result.files.len());
        }
        Err(e) => {
            println!("Error: {}", e);
            return Err(e);
        }
    }

    // Post-generation steps
    println!("\nPost-generation setup...");

    // Git (best-effort external step, after successful publication)
    if git::is_git_installed() {
        if let Err(e) = git::init(&project_name) {
            println!("  Git init warning: {}", e);
        } else {
            println!("  ✓ Git initialized");
            if let Err(e) = git::add_and_commit(
                &project_name,
                &selected_template.name,
                selected_frontend.name,
                selected_backend.name,
            ) {
                println!("  Git commit warning: {}", e);
            } else {
                println!("  ✓ Initial commit made");
            }
        }
    } else {
        println!("  ⚠ Git not installed — skipping git init");
    }

    println!(
        "\n{}",
        success_style.apply_to(format!("Done! Project '{}' is ready.", project_name))
    );
    Ok(())
}
