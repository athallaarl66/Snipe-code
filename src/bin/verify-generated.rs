//! Generates a project for one stack and verifies it through the shared
//! verification engine, so CI exercises the same code path users run.

use snipe_code::verify;
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    // SAFETY: this binary is single-threaded at startup, before project code runs.
    for key in [
        "DATABASE_URL",
        "DB_URL",
        "DB_CONNECTION",
        "DB_HOST",
        "DB_DATABASE",
        "PGHOST",
        "PGDATABASE",
        "PGHOSTADDR",
        "PGSERVICE",
        "PGSERVICEFILE",
    ] {
        unsafe { env::remove_var(key) };
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("generated verification failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    if env::var_os(snipe_code::template::OVERRIDE_ENV).is_some() {
        return Err("verify-generated refuses caller-controlled template overrides".to_string());
    }
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

    verify::run_generated_project(frontend, backend, database)
}
