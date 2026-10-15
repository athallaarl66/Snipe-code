use snipe_code::generator::{GenerateConfig, generate};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

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

    let workspace = env::temp_dir().join(format!(
        "snipe-verify-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    fs::create_dir_all(&workspace).map_err(|e| e.to_string())?;
    // Windows shells hand out the 8.3 short form of the temp path (`ATHALL~1`);
    // Vite canonicalises it and then fails to load its own files, so run from the
    // long form to keep local verification equal to CI.
    let workspace = long_path(workspace);
    let result = verify(&workspace, frontend, backend, database);
    let cleanup = fs::remove_dir_all(&workspace).map_err(|e| e.to_string());
    result.and(cleanup)
}

fn verify(workspace: &Path, frontend: &str, backend: &str, database: &str) -> Result<(), String> {
    let config = GenerateConfig {
        project_name: "generated-app".to_string(),
        template_id: "none".to_string(),
        frontend: frontend.to_string(),
        backend: backend.to_string(),
        database: database.to_string(),
    };
    let repo = env::current_dir().map_err(|e| e.to_string())?;
    env::set_current_dir(workspace).map_err(|e| e.to_string())?;
    let generated = generate(&config, false).map_err(|e| e.to_string());
    let result = generated.and_then(|_| {
        if frontend != "none" {
            run_stack_command(Path::new("generated-app/frontend"), frontend, true)?;
        }
        if backend != "none" {
            run_stack_command(Path::new("generated-app/backend"), backend, false)?;
        }
        Ok(())
    });
    env::set_current_dir(repo).map_err(|e| e.to_string())?;
    result
}

fn run_stack_command(path: &Path, stack: &str, frontend: bool) -> Result<(), String> {
    if frontend || stack == "nestjs" {
        return run_npm_steps(path, stack);
    }
    let commands: &[(&str, &[&str])] = match stack {
        "go-gin" => &[
            ("go", &["mod", "download"]),
            ("go", &["mod", "tidy"]),
            ("go", &["test", "./..."]),
            ("go", &["vet", "./..."]),
            ("go", &["build", "./..."]),
        ],
        "spring-boot" => &[("mvn", &["test", "package"])],
        "dotnet8" => &[
            ("dotnet", &["restore"]),
            ("dotnet", &["test", "--no-restore"]),
            ("dotnet", &["build", "--no-restore"]),
        ],
        "laravel" => &[
            (
                "composer",
                &["install", "--no-interaction", "--no-progress"],
            ),
            ("php", &["artisan", "test"]),
        ],
        other => return Err(format!("unsupported backend stack {other}")),
    };
    for (program, args) in commands {
        run_command(program, args, path, stack)?;
    }
    Ok(())
}

/// Runs install plus every script the generated manifest declares, cheapest
/// first, with the build last so a failure is attributed to the right phase.
fn run_npm_steps(path: &Path, stack: &str) -> Result<(), String> {
    let declared = snipe_code::generator::declared_scripts(path);
    run_command("npm", &["install"], path, stack)?;
    for script in ["typecheck", "lint", "test", "build"] {
        if declared.iter().any(|name| name == script) {
            run_command("npm", &["run", script], path, stack)?;
        }
    }
    Ok(())
}

fn run_command(program: &str, args: &[&str], path: &Path, stack: &str) -> Result<(), String> {
    let status = command(program, args, path)
        .status()
        .map_err(|e| format!("{stack}: {program} unavailable: {e}"))?;
    if !status.success() {
        return Err(format!("{stack}: {program} {args:?} exited with {status}"));
    }
    Ok(())
}
fn command(program: &str, args: &[&str], path: &Path) -> Command {
    let mut command = if cfg!(windows) && matches!(program, "npm" | "mvn" | "composer") {
        let mut command = Command::new("cmd.exe");
        command.args(["/C", program]);
        command
    } else {
        Command::new(program)
    };
    command.args(args).current_dir(path);
    command
}

/// The canonical path without the Windows `\\?\` prefix.
fn long_path(path: PathBuf) -> PathBuf {
    let Ok(canonical) = fs::canonicalize(&path) else {
        return path;
    };
    match canonical.to_string_lossy().strip_prefix(r"\\?\") {
        Some(stripped) => PathBuf::from(stripped),
        None => canonical,
    }
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}
