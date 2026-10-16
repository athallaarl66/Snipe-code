use console::Style;
use dialoguer::{Input, MultiSelect, Select};

use crate::ai_tools;
use crate::generator::{
    GenerateConfig, GenerateResult, dry_run_preview, folder_exists, generate, resolve_destination,
    validate_project_name,
};
use crate::git;
use crate::stack::{self, DATABASE_STACKS};
use crate::template::TemplateRegistry;

/// Inputs for `new` that come from the command line.
#[derive(Debug, Default)]
pub struct Options {
    pub dry_run: bool,
    /// Emit the generation result as JSON on stdout, progress on stderr.
    pub json: bool,
    pub name: Option<String>,
    /// Comma-separated AI tool ids, or `none`. When set, no AI tool prompt is
    /// shown — scripts and CI can select tools without a terminal.
    pub ai_tools: Option<String>,
}

pub fn run(options: Options) -> Result<(), String> {
    let header_style = Style::new().bold().cyan();
    let success_style = Style::new().bold().green();
    let warning_style = Style::new().bold().yellow();

    // Progress goes to stderr when structured output is requested, so stdout
    // carries the result alone.
    let emit = |line: &str| {
        if options.json {
            eprintln!("{}", line);
        } else {
            println!("{}", line);
        }
    };
    let interactive = console::Term::stdout().is_term();

    emit(&header_style.apply_to("SNIPE-CODE CLI").to_string());
    emit("Enterprise Dev-Forge");
    emit("");

    let registry = ai_tools::load()?;
    let templates = TemplateRegistry::load_all();
    let frontend_stacks = stack::frontend_stacks();
    let backend_stacks = stack::backend_stacks();

    // Steps 1-5: prompts, or deterministic defaults when there is no terminal.
    let (template_id, frontend_id, backend_id, database_id, project_name) = if interactive {
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

        let frontend_names: Vec<&str> = frontend_stacks.iter().map(|s| s.name.as_str()).collect();
        let frontend_idx = Select::new()
            .with_prompt("Pilih Frontend")
            .items(&frontend_names)
            .default(0)
            .interact()
            .expect("Failed to read frontend selection");

        let backend_names: Vec<&str> = backend_stacks.iter().map(|s| s.name.as_str()).collect();
        let backend_idx = Select::new()
            .with_prompt("Pilih Backend")
            .items(&backend_names)
            .default(0)
            .interact()
            .expect("Failed to read backend selection");

        let database_names: Vec<&str> = DATABASE_STACKS.iter().map(|s| s.name).collect();
        let database_idx = Select::new()
            .with_prompt("Pilih Database")
            .items(&database_names)
            .default(0)
            .interact()
            .expect("Failed to read database selection");

        let name = match options.name.clone() {
            Some(name) => name,
            None => Input::new()
                .with_prompt("Project Name")
                .default("my-project".to_string())
                .interact_text()
                .expect("Failed to read project name"),
        };

        (
            templates[template_idx].id.to_string(),
            frontend_stacks[frontend_idx].id.clone(),
            backend_stacks[backend_idx].id.clone(),
            DATABASE_STACKS[database_idx].id.to_string(),
            name,
        )
    } else {
        emit(
            "Terminal non-interaktif — default: template none, frontend nextjs, \
             backend none, database none.",
        );
        (
            "none".to_string(),
            "nextjs".to_string(),
            "none".to_string(),
            "none".to_string(),
            options
                .name
                .clone()
                .unwrap_or_else(|| "my-project".to_string()),
        )
    };

    if let Err(e) = stack::validate_selection(&frontend_id, &backend_id) {
        emit(&format!("Error: {}", e));
        return Err(e.to_string());
    }

    if let Err(e) = validate_project_name(&project_name) {
        emit(&format!("Error: {}", e));
        return Err(e);
    }

    // One selection value, whichever producer it came from: generation only ever
    // sees this vector.
    let ai_selection: Vec<String> = match &options.ai_tools {
        Some(spec) => ai_tools::parse_selection(spec),
        None if interactive => {
            let names = ai_tools::display_names(&registry);
            let defaults = vec![false; names.len()];
            let picked = MultiSelect::new()
                .with_prompt("Pilih AI Agent Tools (opsional, spasi untuk memilih)")
                .items(&names)
                .defaults(&defaults)
                .interact()
                .expect("Failed to read AI tool selection");
            picked
                .into_iter()
                .map(|index| registry[index].id.clone())
                .collect()
        }
        None => Vec::new(),
    };
    // Resolving the plan here means an unknown tool or format fails before the
    // wizard shows a collision prompt or writes anything.
    ai_tools::plan(&registry, &ai_selection)?;

    let dest = resolve_destination(&project_name)?;
    let mut overwrite = false;

    // Step 6: folder collision — show the canonical destination.
    if folder_exists(&project_name) {
        if !interactive {
            let message = format!("Folder \"{}\" already exists", dest.display());
            emit(&format!("Error: {}", message));
            return Err(message);
        }
        emit(
            &warning_style
                .apply_to(format!(
                    "\nError: Folder \"{}\" already exists",
                    dest.display()
                ))
                .to_string(),
        );

        let choices = vec!["Overwrite (replace existing)", "Cancel"];
        let choice = Select::new()
            .with_prompt("What to do?")
            .items(&choices)
            .default(1)
            .interact()
            .expect("Failed to read choice");

        if choice == 1 {
            emit("Cancelled. No changes made.");
            return Ok(());
        }
        emit("Overwriting existing folder...");
        overwrite = true;
    }

    let config = GenerateConfig {
        project_name: project_name.clone(),
        template_id: template_id.clone(),
        frontend: frontend_id.clone(),
        backend: backend_id.clone(),
        database: database_id.clone(),
        ai_tools: ai_selection,
    };

    // Dry-run mode — preview only, no filesystem mutation
    if options.dry_run {
        emit("\n[DRY RUN] Would create:\n");
        emit(&dry_run_preview(&config));
        if options.json {
            println!("{}", structured_result(&config, None));
        }
        return Ok(());
    }

    // Generate (staged, published only after successful validation)
    emit("\nGenerating project structure...");
    let result = match generate(&config, overwrite) {
        Ok(result) => result,
        Err(e) => {
            emit(&format!("Error: {}", e));
            return Err(e);
        }
    };

    emit(&success_style.apply_to("Project created!").to_string());
    emit(&format!("  Folders: {}", result.folders.len()));
    emit(&format!("  Files: {}", result.files.len()));
    emit(&format!("  Templates: {}", crate::template::source_label()));
    if result.ai_tools.is_empty() {
        emit("  AI tools: none selected — no agent artifact written");
    } else {
        emit(&format!("  AI tools: {}", result.ai_tools.join(", ")));
        emit(&format!(
            "  AI artifacts: {}",
            result.ai_artifacts.join(", ")
        ));
    }
    for overlap in &result.ai_overlaps {
        emit(&format!("  Overlap: {}", overlap));
    }

    // Post-generation steps
    emit("\nPost-generation setup...");

    // Git (best-effort external step, after successful publication)
    if git::is_git_installed() {
        if let Err(e) = git::init(&project_name) {
            emit(&format!("  Git init warning: {}", e));
        } else {
            emit("  ✓ Git initialized");
            let template_name = templates
                .iter()
                .find(|t| t.id == template_id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| template_id.clone());
            if let Err(e) = git::add_and_commit(
                &project_name,
                &template_name,
                &stack_name(&frontend_stacks, &frontend_id),
                &stack_name(&backend_stacks, &backend_id),
            ) {
                emit(&format!("  Git commit warning: {}", e));
            } else {
                emit("  ✓ Initial commit made");
            }
        }
    } else {
        emit("  ⚠ Git not installed — skipping git init");
    }

    emit(
        &success_style
            .apply_to(format!("Done! Project '{}' is ready.", project_name))
            .to_string(),
    );

    if options.json {
        println!("{}", structured_result(&config, Some(&result)));
    }
    Ok(())
}

fn stack_name(stacks: &[stack::StackOption], id: &str) -> String {
    stacks
        .iter()
        .find(|s| s.id == id)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| id.to_string())
}

/// The generation result as JSON: selected tools, artifacts written, outcome.
fn structured_result(config: &GenerateConfig, result: Option<&GenerateResult>) -> String {
    match result {
        Some(result) => serde_json::json!({
            "project": config.project_name,
            "ai_tools": result.ai_tools,
            "ai_artifacts": result.ai_artifacts,
            "ai_overlaps": result.ai_overlaps,
            "files": result.files,
            "outcome": "created",
        }),
        None => serde_json::json!({
            "project": config.project_name,
            "ai_tools": config.ai_tools,
            "ai_artifacts": Vec::<String>::new(),
            "ai_overlaps": Vec::<String>::new(),
            "files": Vec::<String>::new(),
            "outcome": "dry-run",
        }),
    }
    .to_string()
}
