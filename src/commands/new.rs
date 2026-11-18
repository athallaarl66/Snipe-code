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
    pub template: Option<String>,
    pub frontend: Option<String>,
    pub backend: Option<String>,
    pub database: Option<String>,
    /// Comma-separated AI tool ids, or `none`. When set, no AI tool prompt is
    /// shown — scripts and CI can select tools without a terminal.
    pub ai_tools: Option<String>,
}

pub fn run(options: Options) -> Result<(), String> {
    crate::template::preflight_override()?;
    if console::Term::stdout().is_term() {
        return interactive_wizard(&options);
    }
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
    let interactive = false;

    emit(&header_style.apply_to("SNIPE-CODE CLI").to_string());
    emit("Project generator");
    emit("");

    let registry = ai_tools::load()?;
    let templates = TemplateRegistry::load_all();
    let frontend_stacks = stack::frontend_stacks();
    let backend_stacks = stack::backend_stacks();

    // Steps 1-5: supplied values skip only their own prompts.
    let template_id = choose_template(&options.template, &templates, interactive)?;
    let frontend_id = choose_stack(&options.frontend, &frontend_stacks, "Frontend", interactive)?;
    let backend_id = choose_stack(&options.backend, &backend_stacks, "Backend", interactive)?;
    let database_id = choose_database(&options.database, interactive)?;
    let project_name = match options.name.clone() {
        Some(name) => name,
        None if interactive => Input::new()
            .with_prompt("Project Name")
            .default("my-project".to_string())
            .interact_text()
            .map_err(|e| e.to_string())?,
        None => "my-project".to_string(),
    };

    if stack::validate_selection(&frontend_id, &backend_id).is_err() {
        let message = "Select at least one frontend or backend";
        emit(&format!("Error: {}", message));
        return Err(message.to_string());
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
                .with_prompt("Select AI integrations (optional; Space to select)")
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

fn choose_template(
    value: &Option<String>,
    templates: &[crate::template::Template],
    interactive: bool,
) -> Result<String, String> {
    if let Some(id) = value {
        return templates
            .iter()
            .find(|t| t.id == *id)
            .map(|t| t.id.clone())
            .ok_or_else(|| format!("unknown template: {}", id));
    }
    if !interactive {
        return Ok("none".to_string());
    }
    let names: Vec<String> = templates
        .iter()
        .map(|t| format!("{} {} — {}", t.icon, t.name, t.description))
        .collect();
    let index = Select::new()
        .with_prompt("Select preset")
        .items(&names)
        .default(0)
        .interact()
        .map_err(|e| e.to_string())?;
    Ok(templates[index].id.clone())
}

fn choose_stack(
    value: &Option<String>,
    stacks: &[stack::StackOption],
    label: &str,
    interactive: bool,
) -> Result<String, String> {
    if let Some(id) = value {
        return stacks
            .iter()
            .find(|s| s.id == *id)
            .map(|s| s.id.clone())
            .ok_or_else(|| format!("unknown {}: {}", label.to_lowercase(), id));
    }
    if !interactive {
        return Ok(if label == "Frontend" {
            "nextjs"
        } else {
            "none"
        }
        .to_string());
    }
    let names: Vec<&str> = stacks.iter().map(|s| s.name.as_str()).collect();
    let index = Select::new()
        .with_prompt(format!("Select {}", label))
        .items(&names)
        .default(0)
        .interact()
        .map_err(|e| e.to_string())?;
    Ok(stacks[index].id.clone())
}

fn choose_database(value: &Option<String>, interactive: bool) -> Result<String, String> {
    if let Some(id) = value {
        return DATABASE_STACKS
            .iter()
            .find(|db| db.id == id)
            .map(|db| db.id.to_string())
            .ok_or_else(|| format!("unknown database: {}", id));
    }
    if !interactive {
        return Ok("none".to_string());
    }
    let names: Vec<&str> = DATABASE_STACKS.iter().map(|db| db.name).collect();
    let index = Select::new()
        .with_prompt("Select database")
        .items(&names)
        .default(0)
        .interact()
        .map_err(|e| e.to_string())?;
    Ok(DATABASE_STACKS[index].id.to_string())
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
            "ai_artifact_types": result.ai_artifact_types,
            "ai_overlaps": result.ai_overlaps,
            "files": result.files,
            "outcome": "created",
        }),
        None => serde_json::json!({
            "project": config.project_name,
            "ai_tools": config.ai_tools,
            "ai_artifacts": Vec::<String>::new(),
            "ai_artifact_types": Vec::<crate::generator::AiArtifactType>::new(),
            "ai_overlaps": Vec::<String>::new(),
            "files": Vec::<String>::new(),
            "outcome": "dry-run",
        }),
    }
    .to_string()
}

// Interactive selection remains separate from the script-compatible default path.
fn interactive_wizard(options: &Options) -> Result<(), String> {
    let registry = ai_tools::load()?;
    let templates = TemplateRegistry::load_all();
    let frontends = stack::frontend_stacks();
    let backends = stack::backend_stacks();
    let mut config = GenerateConfig {
        project_name: options.name.clone().unwrap_or_else(|| "my-project".into()),
        template_id: choose_template(&options.template, &templates, false)?,
        frontend: choose_stack(&options.frontend, &frontends, "Frontend", false)?,
        backend: choose_stack(&options.backend, &backends, "Backend", false)?,
        database: choose_database(&options.database, false)?,
        ai_tools: options
            .ai_tools
            .as_deref()
            .map(ai_tools::parse_selection)
            .unwrap_or_default(),
    };
    ai_tools::plan(&registry, &config.ai_tools)?;
    let emit = |s: &str| {
        if options.json {
            eprintln!("{s}");
        } else {
            println!("{s}");
        }
    };
    emit("SNIPE-CODE CLI: project generator");
    emit("Use arrow keys and Enter to select; choose Back or Cancel to navigate.");
    emit("Presets add feature-specific structure; stacks select frameworks.");
    let mut step: i32 = 0;
    let mut git_init = false;
    let mut git_commit = false;
    loop {
        let fixed = match step {
            0 => options.template.is_some(),
            1 => options.frontend.is_some(),
            2 => options.backend.is_some(),
            3 => options.database.is_some(),
            4 => options.name.is_some(),
            5 => options.ai_tools.is_some(),
            _ => false,
        };
        if fixed {
            step += 1;
            continue;
        }
        if step < 0 {
            emit("Cancelled. No changes made.");
            return Ok(());
        }
        if step <= 3 {
            let (prompt, labels, ids): (String, Vec<String>, Vec<String>) = match step {
                0 => (
                    "[1/8] Preset".into(),
                    templates
                        .iter()
                        .map(|t| format!("{} — {}", t.name, t.description))
                        .collect(),
                    templates.iter().map(|t| t.id.clone()).collect(),
                ),
                1 => (
                    "[2/8] Frontend framework".into(),
                    frontends.iter().map(|s| s.name.clone()).collect(),
                    frontends.iter().map(|s| s.id.clone()).collect(),
                ),
                2 => (
                    "[3/8] Backend framework".into(),
                    backends.iter().map(|s| s.name.clone()).collect(),
                    backends.iter().map(|s| s.id.clone()).collect(),
                ),
                _ => (
                    "[4/8] Database".into(),
                    DATABASE_STACKS.iter().map(|s| s.name.to_string()).collect(),
                    DATABASE_STACKS.iter().map(|s| s.id.to_string()).collect(),
                ),
            };
            let current = match step {
                0 => &config.template_id,
                1 => &config.frontend,
                2 => &config.backend,
                _ => &config.database,
            };
            let default = ids.iter().position(|id| id == current).unwrap_or(0);
            let mut choices = labels;
            choices.extend(["Back".into(), "Cancel".into()]);
            let choice = Select::new()
                .with_prompt(prompt)
                .items(&choices)
                .default(default)
                .interact_opt()
                .map_err(|e| e.to_string())?;
            match choice {
                None => {
                    emit("Cancelled. No changes made.");
                    return Ok(());
                }
                Some(n) if n == ids.len() => {
                    step = previous_step(step, options, git_init);
                    continue;
                }
                Some(n) if n == ids.len() + 1 => {
                    emit("Cancelled. No changes made.");
                    return Ok(());
                }
                Some(n) => match step {
                    0 => config.template_id = ids[n].clone(),
                    1 => config.frontend = ids[n].clone(),
                    2 => config.backend = ids[n].clone(),
                    _ => config.database = ids[n].clone(),
                },
            }
        } else if step == 4 {
            let choice = Select::new()
                .with_prompt("[5/8] Project name")
                .items(&["Enter name", "Back", "Cancel"])
                .default(0)
                .interact_opt()
                .map_err(|e| e.to_string())?;
            match choice {
                Some(0) => {}
                Some(1) => {
                    step = previous_step(step, options, git_init);
                    continue;
                }
                _ => {
                    emit("Cancelled. No changes made.");
                    return Ok(());
                }
            }
            config.project_name = Input::new()
                .with_prompt("Project name")
                .default(config.project_name.clone())
                .interact_text()
                .map_err(|e| e.to_string())?;
            if let Err(e) = validate_project_name(&config.project_name) {
                emit(&format!("Error: {e}"));
                continue;
            }
        } else if step == 5 {
            emit(
                "AI integrations write project-local instruction, rule, skill, or workflow files only. They do not run a model, authenticate a provider, store credentials, or require an API key.",
            );
            emit("Use Space to select agents and Enter to continue.");
            let names = ai_tools::display_names(&registry);
            let defaults: Vec<bool> = registry
                .iter()
                .map(|t| config.ai_tools.contains(&t.id))
                .collect();
            let picked = MultiSelect::new()
                .with_prompt("[6/8] AI integrations")
                .items(&names)
                .defaults(&defaults)
                .interact_opt()
                .map_err(|e| e.to_string())?;
            let Some(picked) = picked else {
                emit("Cancelled. No changes made.");
                return Ok(());
            };
            config.ai_tools = picked.into_iter().map(|i| registry[i].id.clone()).collect();
            ai_tools::plan(&registry, &config.ai_tools)?;
            let choice = Select::new()
                .with_prompt("AI selection")
                .items(&["Continue", "Back", "Cancel"])
                .default(0)
                .interact_opt()
                .map_err(|e| e.to_string())?;
            match choice {
                Some(0) => {}
                Some(1) => {
                    step = previous_step(step, options, git_init);
                    continue;
                }
                _ => {
                    emit("Cancelled. No changes made.");
                    return Ok(());
                }
            }
        } else if step == 6 || step == 7 {
            let prompt = if step == 6 {
                "[7/8] Initialize a Git repository after creation?"
            } else {
                "[8/8] Stage files and make an initial Git commit after creation?"
            };
            if step == 7 && !git_init {
                git_commit = false;
                step += 1;
                continue;
            }
            let choice = Select::new()
                .with_prompt(prompt)
                .items(&["No", "Yes", "Back", "Cancel"])
                .default(0)
                .interact_opt()
                .map_err(|e| e.to_string())?;
            match choice {
                Some(0) => {
                    if step == 6 {
                        git_init = false
                    } else {
                        git_commit = false
                    }
                }
                Some(1) => {
                    if step == 6 {
                        git_init = true
                    } else {
                        git_commit = true
                    }
                }
                Some(2) => {
                    step = previous_step(step, options, git_init);
                    continue;
                }
                _ => {
                    emit("Cancelled. No changes made.");
                    return Ok(());
                }
            }
        } else {
            if config.frontend == "none" && config.backend == "none" {
                if options.frontend.is_some() && options.backend.is_some() {
                    return Err("Select at least one frontend or backend.".to_string());
                }
                emit("Error: Select at least one frontend or backend.");
                step = if options.frontend.is_some() { 2 } else { 1 };
                continue;
            }
            validate_project_name(&config.project_name)?;
            let dest = resolve_destination(&config.project_name)?;
            let artifacts = ai_tools::plan(&registry, &config.ai_tools)?;
            emit(&format!(
                "\nSummary\n  Destination: {}\n  Preset: {}\n  Frontend: {}\n  Backend: {}\n  Database: {}\n  AI integrations: {}\n  Git init: {}\n  Initial commit: {}",
                dest.display(),
                config.template_id,
                config.frontend,
                config.backend,
                config.database,
                if config.ai_tools.is_empty() {
                    "none".into()
                } else {
                    config.ai_tools.join(", ")
                },
                if git_init { "yes" } else { "no" },
                if git_commit { "yes" } else { "no" }
            ));
            for artifact in &artifacts.artifacts {
                emit(&format!(
                    "  AI artifact: {} ({})",
                    artifact.path, artifact.kind
                ));
            }
            let choice = Select::new()
                .with_prompt("Review: arrow keys and Enter")
                .items(&["Preview", "Create project", "Back", "Cancel"])
                .default(3)
                .interact_opt()
                .map_err(|e| e.to_string())?;
            match choice {
                Some(0) => {
                    emit(&dry_run_preview(&config));
                    continue;
                }
                Some(1) => {}
                Some(2) => {
                    step = previous_step(step, options, git_init);
                    continue;
                }
                _ => {
                    emit("Cancelled. No changes made.");
                    return Ok(());
                }
            }
            if options.dry_run {
                emit(&dry_run_preview(&config));
                if options.json {
                    println!("{}", structured_result(&config, None));
                }
                return Ok(());
            }
            let overwrite = if dest.exists() {
                emit(&format!("Destination already exists: {}", dest.display()));
                let c = Select::new()
                    .with_prompt("Replace existing project?")
                    .items(&["Cancel", "Overwrite (replace existing)", "Back"])
                    .default(0)
                    .interact_opt()
                    .map_err(|e| e.to_string())?;
                match c {
                    Some(1) => true,
                    Some(2) => continue,
                    _ => {
                        emit("Cancelled. No changes made.");
                        return Ok(());
                    }
                }
            } else {
                false
            };
            let result = generate(&config, overwrite)?;
            emit(&format!("Project created: {}", dest.display()));
            if git_init {
                if !git::is_git_installed() {
                    emit("Git unavailable; project retained.");
                } else if let Err(e) = git::init(&config.project_name) {
                    emit(&format!("Git init warning: {e}; project retained."));
                } else if git_commit {
                    let name = templates
                        .iter()
                        .find(|t| t.id == config.template_id)
                        .map(|t| t.name.as_str())
                        .unwrap_or(&config.template_id);
                    match git::add_and_commit(
                        &config.project_name,
                        name,
                        &stack_name(&frontends, &config.frontend),
                        &stack_name(&backends, &config.backend),
                    ) {
                        Ok(()) => emit("Initial Git commit created."),
                        Err(e) => emit(&format!("Git commit warning: {e}; project retained.")),
                    }
                }
            }
            if options.json {
                println!("{}", structured_result(&config, Some(&result)));
            }
            return Ok(());
        }
        step += 1;
    }
}

fn previous_step(step: i32, options: &Options, git_init: bool) -> i32 {
    let mut n = step - 1;
    while n >= 0
        && match n {
            7 => !git_init,
            0 => options.template.is_some(),
            1 => options.frontend.is_some(),
            2 => options.backend.is_some(),
            3 => options.database.is_some(),
            4 => options.name.is_some(),
            5 => options.ai_tools.is_some(),
            _ => false,
        }
    {
        n -= 1;
    }
    n
}

#[cfg(test)]
mod wizard_navigation_tests {
    use super::{Options, previous_step};

    #[test]
    fn back_skips_fixed_and_inapplicable_steps() {
        let options = Options {
            template: Some("none".into()),
            frontend: Some("nextjs".into()),
            backend: Some("none".into()),
            database: Some("none".into()),
            name: Some("project".into()),
            ai_tools: Some("none".into()),
            ..Options::default()
        };
        assert_eq!(previous_step(8, &options, false), 6);
        assert_eq!(previous_step(8, &options, true), 7);
        assert_eq!(previous_step(6, &options, false), -1);
        assert_eq!(previous_step(7, &options, true), 6);
    }
}
