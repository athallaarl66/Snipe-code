//! Command execution shared by the `verify` command and the CI harness.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::thread;

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
    println!(
        "  $ {}  (in {})",
        redact(&spec.display(), &database_credentials(&spec.dir)),
        redact(label, &database_credentials(&spec.dir))
    );
    run_command(spec)
}

/// Runs a gate while keeping its output off stdout, which is reserved for
/// machine-readable CLI results.
pub fn run_stderr(spec: &CommandSpec, label: &str) -> Result<(), RunError> {
    eprintln!(
        "  $ {}  (in {})",
        redact(&spec.display(), &database_credentials(&spec.dir)),
        redact(label, &database_credentials(&spec.dir))
    );
    run_process(&mut command(spec), spec, true)
}

/// Run a harness-owned generated project without inherited database targets.
pub fn run_generated(spec: &CommandSpec, label: &str) -> Result<(), RunError> {
    println!(
        "  $ {}  (in {})",
        redact(&spec.display(), &database_credentials(&spec.dir)),
        redact(label, &database_credentials(&spec.dir))
    );
    run_process(&mut generated_command(spec), spec, false)
}

pub(super) const GENERATED_CLEARED_DB_KEYS: &[&str] = &[
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
];

fn generated_command(spec: &CommandSpec) -> Command {
    let mut child = command(spec);
    for key in GENERATED_CLEARED_DB_KEYS {
        child.env_remove(key);
    }
    child
}

fn run_command(spec: &CommandSpec) -> Result<(), RunError> {
    run_process(&mut command(spec), spec, false)
}

fn run_process(child: &mut Command, spec: &CommandSpec, stderr_only: bool) -> Result<(), RunError> {
    let secrets = database_credentials(&spec.dir);
    let mut process = child
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            RunError::Unavailable(redact(
                &format!("{} unavailable: {error}", spec.program),
                &secrets,
            ))
        })?;
    let stdout = process.stdout.take().expect("piped stdout");
    let stderr = process.stderr.take().expect("piped stderr");
    let stdout_secrets = secrets.clone();
    let out = thread::spawn(move || forward(stdout, &stdout_secrets, stderr_only));
    let stderr_secrets = secrets.clone();
    let err = thread::spawn(move || forward(stderr, &stderr_secrets, true));
    let status = process.wait();
    let out = out.join();
    let err = err.join();
    if !matches!(out, Ok(Ok(()))) || !matches!(err, Ok(Ok(()))) {
        return Err(RunError::Failed("could not forward child output".into()));
    }
    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(RunError::Failed(format!(
            "{} {} exited with {status}",
            redact(&spec.program, &secrets),
            redact(&format!("{:?}", spec.args), &secrets)
        ))),
        Err(error) => Err(RunError::Unavailable(format!(
            "{} unavailable: {error}",
            redact(&spec.program, &secrets)
        ))),
    }
}

// Hold back enough bytes to match literals split across pipe reads.
fn forward(mut input: impl Read, secrets: &[Vec<u8>], stderr: bool) -> io::Result<()> {
    let output: Box<dyn Write> = if stderr {
        Box::new(io::stderr())
    } else {
        Box::new(io::stdout())
    };
    let mut output = io::BufWriter::new(output);
    let mut failed = false;
    let mut pending = Vec::new();
    let mut buffer = [0; 8192];
    loop {
        let count = input.read(&mut buffer)?;
        pending.extend_from_slice(&buffer[..count]);
        let mut pos = 0;
        while pos < pending.len() {
            if count != 0
                && secrets.iter().any(|secret| {
                    secret.len() > pending.len() - pos && secret.starts_with(&pending[pos..])
                })
            {
                break;
            }
            if let Some(secret) = secrets
                .iter()
                .find(|secret| pending[pos..].starts_with(secret))
            {
                failed |= output.write_all(b"[redacted]").is_err();
                pos += secret.len();
            } else {
                failed |= output.write_all(&pending[pos..pos + 1]).is_err();
                pos += 1;
            }
        }
        pending.drain(..pos);
        failed |= output.flush().is_err();
        if count == 0 {
            return if failed {
                Err(io::Error::other("output forwarding failed"))
            } else {
                Ok(())
            };
        }
    }
}

pub(super) fn redact(text: &str, secrets: &[Vec<u8>]) -> String {
    let mut bytes = text.as_bytes().to_vec();
    for secret in secrets {
        let mut result = Vec::new();
        let mut pos = 0;
        while pos < bytes.len() {
            if bytes[pos..].starts_with(secret) {
                result.extend_from_slice(b"[redacted]");
                pos += secret.len();
            } else {
                result.push(bytes[pos]);
                pos += 1;
            }
        }
        bytes = result;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

pub(super) fn database_credentials(directory: &Path) -> Vec<Vec<u8>> {
    fn add(key: &str, value: &str, secrets: &mut Vec<Vec<u8>>) {
        if matches!(
            key,
            "DB_PASSWORD" | "DB_USERNAME" | "PGPASSWORD" | "PGUSER" | "MYSQL_PWD"
        ) {
            if !value.is_empty() {
                secrets.push(value.as_bytes().to_vec());
            }
        } else if matches!(key, "DATABASE_URL" | "DB_URL") {
            if let Some((_, rest)) = value.split_once("://") {
                let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
                if let Some((userinfo, _)) = authority.rsplit_once('@') {
                    for part in userinfo.splitn(2, ':') {
                        if !part.is_empty() {
                            secrets.push(part.as_bytes().to_vec());
                        }
                    }
                }
            }
        }
    }
    let mut secrets = Vec::new();
    for (key, value) in std::env::vars_os() {
        if let (Some(key), Some(value)) = (key.to_str(), value.to_str()) {
            add(key, value, &mut secrets);
        }
    }
    for name in [
        ".env",
        ".env.local",
        ".env.test",
        ".env.test.local",
        ".env.testing",
        ".env.testing.local",
    ] {
        if let Ok(text) = fs::read_to_string(directory.join(name)) {
            for line in text.lines() {
                let line = line.trim().strip_prefix("export ").unwrap_or(line.trim());
                if let Some((key, value)) = line.split_once('=') {
                    let value = value.trim();
                    let value = value
                        .strip_prefix('"')
                        .and_then(|v| v.strip_suffix('"'))
                        .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                        .unwrap_or(value);
                    add(key.trim(), value, &mut secrets);
                }
            }
        }
    }
    // Cover percent-encoded URL userinfo, decoded values, and JSON string escapes.
    let originals = secrets.clone();
    for value in originals {
        let mut decoded = Vec::new();
        let mut i = 0;
        while i < value.len() {
            if value[i] == b'%' && i + 2 < value.len() {
                if let (Some(a), Some(b)) = (
                    (value[i + 1] as char).to_digit(16),
                    (value[i + 2] as char).to_digit(16),
                ) {
                    decoded.push((a * 16 + b) as u8);
                    i += 3;
                    continue;
                }
            }
            decoded.push(value[i]);
            i += 1;
        }
        for item in [value, decoded] {
            if let Ok(text) = std::str::from_utf8(&item) {
                if let Ok(json) = serde_json::to_string(text) {
                    secrets.push(json.as_bytes()[1..json.len() - 1].to_vec());
                }
            }
            for lower in [false, true] {
                let mut encoded = String::new();
                for &byte in &item {
                    if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
                        encoded.push(byte as char);
                    } else if lower {
                        encoded.push_str(&format!("%{byte:02x}"));
                    } else {
                        encoded.push_str(&format!("%{byte:02X}"));
                    }
                }
                secrets.push(encoded.into_bytes());
            }
            secrets.push(item);
        }
    }
    secrets.retain(|value| !value.is_empty());
    secrets.sort_by_key(|value| std::cmp::Reverse(value.len()));
    secrets.dedup();
    secrets
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
    create_workspace(&path)?;
    Ok(long_path(path))
}

fn create_workspace(path: &Path) -> Result<(), String> {
    fs::create_dir(path).map_err(|error| format!("cannot create fresh workspace: {error}"))
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
    fn authority_ends_before_path_query_and_fragment() {
        let dir = temp_workspace("redaction-authority").unwrap();
        fs::write(dir.join(".env"), "DATABASE_URL=postgres://fixtureAuthUser:fixtureAuthPass@localhost/db?mail=other@elsewhere\n").unwrap();
        let secrets = database_credentials(&dir);
        assert!(secrets.contains(&b"fixtureAuthUser".to_vec()));
        assert!(secrets.contains(&b"fixtureAuthPass".to_vec()));
        assert!(!secrets.contains(&b"localhost/db?mail=other".to_vec()));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn dotenv_preserves_quoted_credential_boundaries() {
        let dir = temp_workspace("dotenv-exact").unwrap();
        let secret = "'fixture inner credential'";
        fs::write(
            dir.join(".env"),
            format!(
                "DB_PASSWORD=\"{secret}\"
"
            ),
        )
        .unwrap();
        let secrets = database_credentials(&dir);
        assert_eq!(redact(secret, &secrets), "[redacted]");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn workspace_collision_preserves_existing_contents() {
        let workspace = temp_workspace("snipe-workspace-collision").unwrap();
        let canary = workspace.join("canary");
        fs::write(&canary, b"preserve").unwrap();
        assert!(create_workspace(&workspace).is_err());
        assert_eq!(fs::read(&canary).unwrap(), b"preserve");
        fs::remove_dir_all(workspace).unwrap();
    }

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
            "PGHOSTADDR",
            "PGSERVICE",
            "PGSERVICEFILE",
        ] {
            assert!(removed.iter().any(|name| name == key), "{key}");
        }
    }
}
