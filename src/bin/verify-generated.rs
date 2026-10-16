//! Generates a project for one stack and verifies it through the shared
//! verification engine, so CI exercises the same code path users run.

use snipe_code::generator::{GenerateConfig, generate};
use snipe_code::verify::{self, Options};
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("generated verification failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut frontend = "none";
    let mut backend = "none";
    let mut database = "none";
    let mut i = 1;
    let args: Vec<String> = env::args().collect();
    while i < args.len() {
        let value = args
            .get(i + 1)
            .ok_or_else(|| format!("missing value for {}", args[i]))?;
        match args[i].as_str() {
            "--frontend" => frontend = value,
            "--backend" => backend = value,
            "--database" => database = value,
            flag => return Err(format!("unknown option {flag}")),
        }
        i += 2;
    }
    if frontend == "none" && backend == "none" {
        return Err("select --frontend or --backend".to_string());
    }

    let workspace = verify::exec::temp_workspace("snipe-verify")?;
    let result = generate_and_verify(&workspace, frontend, backend, database);
    let cleanup = fs::remove_dir_all(&workspace).map_err(|e| e.to_string());
    result.and(cleanup)
}

fn generate_and_verify(
    workspace: &Path,
    frontend: &str,
    backend: &str,
    database: &str,
) -> Result<(), String> {
    let config = GenerateConfig {
        project_name: "generated-app".to_string(),
        template_id: "none".to_string(),
        frontend: frontend.to_string(),
        backend: backend.to_string(),
        database: database.to_string(),
        ai_tools: Vec::new(),
    };
    let repo = env::current_dir().map_err(|e| e.to_string())?;
    env::set_current_dir(workspace).map_err(|e| e.to_string())?;
    let generated = generate(&config, false).map_err(|e| e.to_string());
    let result =
        generated.and_then(|_| verify_generated(Path::new("generated-app"), frontend, backend));
    env::set_current_dir(repo).map_err(|e| e.to_string())?;
    result
}

/// Runs the generated project through the engine and keeps the requested stack
/// in the failure message.
fn verify_generated(root: &Path, frontend: &str, backend: &str) -> Result<(), String> {
    let options = Options {
        path: root.to_path_buf(),
        only: None,
        security: false,
    };
    let report = verify::run(root, &options, &verify::exec::run)?;
    let stack = if backend == "none" { frontend } else { backend };
    if report.parts.is_empty() {
        return Err(format!(
            "unsupported stack {stack}: no verification gates found in {}",
            root.display()
        ));
    }
    verify::print_report(&report);
    match report.first_failure() {
        Some((_, _, _, message)) => Err(format!("{stack}: {message}")),
        None => match verify::exit_status(&report) {
            0 => Ok(()),
            status => Err(format!("{stack}: no gate executed ({status})")),
        },
    }
}
