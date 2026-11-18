//! Command execution shared by the `verify` command and the CI harness.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;

/// A command and the directory it runs in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub dir: PathBuf,
}

impl CommandSpec {
    pub fn new(program: &str, args: &[&str], dir: &Path) -> Self {
        Self {
            program: program.to_string(),
            args: args.iter().map(|arg| arg.to_string()).collect(),
            dir: dir.to_path_buf(),
        }
    }

    /// The command as a user would type it.
    pub fn display(&self) -> String {
        if self.args.is_empty() {
            self.program.clone()
        } else {
            format!("{} {}", self.program, self.args.join(" "))
        }
    }
}

/// Why a gate could not complete. The message never repeats the part it belongs
/// to, so a caller can prefix its own label and keep its wording stable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    /// The tool is not installed on this machine.
    Unavailable(String),
    /// The tool ran and exited non-zero.
    Failed(String),
}

impl RunError {
    pub fn message(&self) -> &str {
        match self {
            RunError::Unavailable(message) | RunError::Failed(message) => message,
        }
    }
}

/// Runs one command, printing it first so the run can be repeated by hand.
pub fn run(spec: &CommandSpec, label: &str) -> Result<(), RunError> {
    println!("  $ {}  (in {})", spec.display(), label);
    run_command(spec)
}

/// Runs a gate while keeping its output off stdout, which is reserved for
/// machine-readable CLI results.
pub fn run_stderr(spec: &CommandSpec, label: &str) -> Result<(), RunError> {
    eprintln!("  $ {}  (in {})", spec.display(), label);
    match command(spec)
        .stdout(Stdio::from(std::io::stderr()))
        .status()
    {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(RunError::Failed(format!(
            "{} {:?} exited with {status}",
            spec.program, spec.args
        ))),
        Err(error) => Err(RunError::Unavailable(format!(
            "{} unavailable: {error}",
            spec.program
        ))),
    }
}

/// Run a harness-owned generated project without inherited database targets.
pub fn run_generated(spec: &CommandSpec, label: &str) -> Result<(), RunError> {
    println!("  $ {}  (in {})", spec.display(), label);
    run_process(&mut generated_command(spec), spec)
}

fn generated_command(spec: &CommandSpec) -> Command {
    let mut child = command(spec);
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
        child.env_remove(key);
    }
    child
}

fn run_command(spec: &CommandSpec) -> Result<(), RunError> {
    run_process(&mut command(spec), spec)
}

fn run_process(child: &mut Command, spec: &CommandSpec) -> Result<(), RunError> {
    match child.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(RunError::Failed(format!(
            "{} {:?} exited with {status}",
            spec.program, spec.args
        ))),
        Err(error) => Err(RunError::Unavailable(format!(
            "{} unavailable: {error}",
            spec.program
        ))),
    }
}

/// `npm`, `mvn` and `composer` are `.cmd` shims on Windows, which
/// `CreateProcess` cannot start directly, so they go through `cmd.exe /C`.
/// Go, dotnet, php and cargo are real executables and are spawned directly.
fn command(spec: &CommandSpec) -> Command {
    let mut command =
        if cfg!(windows) && matches!(spec.program.as_str(), "npm" | "mvn" | "composer") {
            let mut command = Command::new("cmd.exe");
            command.args(["/C", &spec.program]);
            command
        } else {
            Command::new(&spec.program)
        };
    command.args(&spec.args).current_dir(&spec.dir);
    command
}

/// A fresh temp workspace. Windows shells hand out the 8.3 short form of the
/// temp path (`ATHALL~1`); Vite canonicalises it and then fails to load its own
/// files, so the long form is used to keep local verification equal to CI.
pub fn temp_workspace(prefix: &str) -> Result<PathBuf, String> {
    let path = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    Ok(long_path(path))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_command_removes_inherited_database_targets() {
        let spec = CommandSpec::new("tool", &[], Path::new("."));
        let command = generated_command(&spec);
        let removed: Vec<_> = command
            .get_envs()
            .filter(|(_, value)| value.is_none())
            .map(|(key, _)| key.to_string_lossy().into_owned())
            .collect();
        for key in [
            "DATABASE_URL",
            "DB_URL",
            "DB_CONNECTION",
            "DB_HOST",
            "DB_DATABASE",
            "PGHOST",
            "PGDATABASE",
        ] {
            assert!(removed.iter().any(|name| name == key), "{key}");
        }
    }
}
