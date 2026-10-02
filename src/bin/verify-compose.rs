use snipe_code::generator::{GenerateConfig, generate};
use std::env;
use std::fs;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("compose verification failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let database = env::args().nth(1).ok_or("database required")?;
    let backend = env::args().nth(2).ok_or("backend required")?;
    let workspace = env::temp_dir().join(format!("snipe-compose-{}", std::process::id()));
    let _ = fs::remove_dir_all(&workspace);
    fs::create_dir_all(&workspace).map_err(|e| e.to_string())?;
    let result = (|| {
        let repo = env::current_dir().map_err(|e| e.to_string())?;
        env::set_current_dir(&workspace).map_err(|e| e.to_string())?;
        let config = GenerateConfig {
            project_name: "generated-compose".to_string(),
            template_id: "none".to_string(),
            frontend: "none".to_string(),
            backend: backend.clone(),
            database: database.clone(),
        };
        let generated = generate(&config, false).map_err(|e| e.to_string());
        let status = generated.and_then(|_| {
            let compose = if cfg!(windows) {
                "docker.exe"
            } else {
                "docker"
            };
            Command::new(compose)
                .args([
                    "compose",
                    "-f",
                    "generated-compose/docker-compose.yml",
                    "config",
                ])
                .status()
                .map_err(|e| format!("docker unavailable: {e}"))
                .and_then(|status| {
                    if status.success() {
                        Ok(())
                    } else {
                        Err(format!("docker compose config exited {status}"))
                    }
                })
        });
        env::set_current_dir(repo).map_err(|e| e.to_string())?;
        status
    })();
    fs::remove_dir_all(&workspace).map_err(|e| e.to_string())?;
    result
}
