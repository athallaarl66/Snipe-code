//! The `verify` command as a user sees it: help output, usage errors, the
//! nothing-to-verify exit status, and the README contract.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_snipe-code");

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Fixture {
    path: PathBuf,
}

impl Fixture {
    fn new() -> Fixture {
        let path = std::env::temp_dir().join(format!(
            "snipe-verify-cli-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&path).expect("create fixture");
        Fixture { path }
    }

    fn write(&self, relative: &str, content: &str) -> &Fixture {
        fs::write(self.path.join(relative), content).expect("write fixture file");
        self
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn run(args: &[&str]) -> (String, Option<i32>) {
    let out = Command::new(BIN)
        .args(args)
        .output()
        .expect("failed to run snipe-code");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    (text, out.status.code())
}

fn run_output(args: &[&str], dir: &PathBuf) -> (String, String, Option<i32>) {
    let out = Command::new(BIN)
        .args(args)
        .current_dir(dir)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("failed to run snipe-code");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code(),
    )
}

fn run_verify_json(dir: &PathBuf, env: &[(&str, &str)]) -> std::process::Output {
    let mut command = Command::new(BIN);
    command
        .args(["verify", ".", "--only", "test", "--json"])
        .current_dir(dir)
        .stdin(std::process::Stdio::null());
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
        command.env_remove(key);
    }
    for (key, value) in env {
        command.env(key, value);
    }
    command.output().expect("run verify preflight")
}

#[test]
fn main_help_lists_verify() {
    let (stdout, status) = run(&["help"]);
    assert_eq!(status, Some(0));
    assert!(stdout.contains("verify"), "{stdout}");
}

#[test]
fn per_command_help_documents_options_and_exit_status() {
    let (stdout, status) = run(&["help", "verify"]);
    assert_eq!(status, Some(0));
    for expected in [
        "USAGE",
        "snipe-code verify [PATH] [--only <gate>] [--security] [--no-install]",
        "--only <gate>",
        "--security",
        "--no-install",
        "--json",
        "SAFETY",
        "block explicit remote, production-like, or ambiguous database targets",
        "An undetected target is unknown and requires explicit approval",
        "blocked",
        "EXIT STATUS",
        "4   incomplete",
        "not-applicable",
        "audit",
    ] {
        assert!(
            stdout.contains(expected),
            "help is missing {expected}:\n{stdout}"
        );
    }
}

#[test]
fn unknown_option_and_unknown_gate_are_usage_errors() {
    let fixture = Fixture::new();
    fixture.write(
        "package.json",
        r#"{"name":"app","scripts":{"test":"vitest run"}}"#,
    );
    let path = fixture.path.to_string_lossy().to_string();

    let (_, unknown_option) = run(&["verify", &path, "--nope"]);
    assert_eq!(unknown_option, Some(2));

    let (_, unknown_gate) = run(&["verify", &path, "--only", "deploy"]);
    assert_eq!(unknown_gate, Some(2));

    let (stdout, rejected) = run(&["verify", &path, "--only", "build"]);
    assert_eq!(rejected, Some(2), "{stdout}");
    assert!(stdout.contains("does not apply"), "{stdout}");

    let (_, missing_value) = run(&["verify", &path, "--only"]);
    assert_eq!(missing_value, Some(2));
}

#[test]
fn unrecognized_project_reports_no_gates_and_exits_three() {
    let fixture = Fixture::new();
    fixture.write("notes.txt", "nothing to verify here\n");
    let path = fixture.path.to_string_lossy().to_string();
    let (stdout, status) = run(&["verify", &path]);
    assert_eq!(status, Some(3), "{stdout}");
    assert!(stdout.contains("no gates found"), "{stdout}");
    assert!(stdout.contains("NOTHING EXECUTED"), "{stdout}");
}

#[test]
fn no_install_leaves_nothing_to_run_when_install_is_the_only_gate() {
    let fixture = Fixture::new();
    fixture.write("package.json", r#"{"name":"app"}"#);
    let path = fixture.path.to_string_lossy().to_string();
    let (stdout, status) = run(&["verify", &path, "--no-install"]);
    assert_eq!(status, Some(3), "{stdout}");
    assert!(stdout.contains("not-applicable"), "{stdout}");
    assert!(
        stdout.contains("skipped by --no-install"),
        "the report must say why install did not run:\n{stdout}"
    );
}

#[test]
fn no_install_with_only_install_is_rejected_by_the_cli() {
    let fixture = Fixture::new();
    fixture.write("package.json", r#"{"name":"app"}"#);
    let path = fixture.path.to_string_lossy().to_string();
    let (stdout, status) = run(&["verify", &path, "--no-install", "--only", "install"]);
    assert_eq!(status, Some(2), "{stdout}");
    assert!(stdout.contains("--no-install"), "{stdout}");
}

#[test]
fn report_export_writes_json_and_keeps_stdout_to_report_path() {
    let fixture = Fixture::new();
    fixture.write(
        "package.json",
        r#"{"name":"app","scripts":{"test":"echo test"}}"#,
    );
    let (stdout, stderr, status) = run_output(
        &[
            "verify",
            ".",
            "--only",
            "test",
            "--report",
            "json",
            "--output",
            "verify.json",
        ],
        &fixture.path,
    );
    assert_eq!(status, Some(3), "{stdout}\n{stderr}");
    assert_eq!(stdout.trim(), "verify.json");
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.path.join("verify.json")).unwrap()).unwrap();
    assert_eq!(report["exit_code"], 3);
    assert_eq!(report["counts"]["blocked"], 1);
    assert!(
        report["parts"][0]["gates"][0]["detail"]
            .as_str()
            .unwrap()
            .contains("database target unknown")
    );
}

#[test]
fn inherited_database_environment_overrides_dotenv_for_preflight() {
    let fixture = Fixture::new();
    fixture.write(
        "package.json",
        r#"{"name":"app","scripts":{"test":"vitest run"}}"#,
    );
    fixture.write(
        ".env",
        "DB_HOST=db.production.example\nDB_DATABASE=app_test\nDB_PASSWORD=dotenv-secret-canary\n",
    );
    let output = run_verify_json(&fixture.path, &[("DB_HOST", "localhost")]);
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["counts"]["blocked"], 1);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("database target: local database app_test at localhost"),
        "{stderr}"
    );
    assert!(!stderr.contains("db.production.example"), "{stderr}");
    assert!(!stderr.contains("dotenv-secret-canary"), "{stderr}");

    fixture.write(
        ".env",
        "DB_HOST=localhost\nDB_DATABASE=app_test\nDB_PASSWORD=dotenv-secret-canary\n",
    );
    let output = run_verify_json(&fixture.path, &[("DB_HOST", "db.remote.example")]);
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["counts"]["blocked"], 1);
    assert!(
        report["parts"][0]["gates"][0]["detail"]
            .as_str()
            .unwrap()
            .contains("remote")
    );
    assert!(!stdout.contains("db.remote.example"), "{stdout}");
    assert!(!stdout.contains("dotenv-secret-canary"), "{stdout}");
}

#[test]
fn database_url_credentials_and_ambiguous_values_never_reach_reports() {
    let fixture = Fixture::new();
    fixture.write(
        "package.json",
        r#"{"name":"app","scripts":{"test":"vitest run"}}"#,
    );
    for config in [
        format!(
            "DATABASE_URL=postgres://user-canary:{}@localhost/app_test\n",
            "password-canary"
        ),
        "DATABASE_URL=opaque-secret-canary-value\n".to_string(),
    ] {
        fixture.write(".env", &config);
        let output = run_verify_json(&fixture.path, &[]);
        assert_eq!(output.status.code(), Some(3));
        let stdout = String::from_utf8(output.stdout).unwrap();
        let _: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        for canary in ["user-canary", "password-canary", "secret-canary"] {
            assert!(!stdout.contains(canary), "{stdout}");
            assert!(!stderr.contains(canary), "{stderr}");
        }
    }
}

#[test]
fn noninteractive_cli_does_not_spawn_project_commands_without_approval() {
    let fixture = Fixture::new();
    fixture.write(
        "package.json",
        r#"{"name":"app","scripts":{"test":"echo test"}}"#,
    );
    let shim = fixture.path.join("shim");
    fs::create_dir(&shim).unwrap();
    #[cfg(windows)]
    fs::write(
        shim.join("npm.cmd"),
        "@echo off\r\necho ran>executed-canary\r\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let script = shim.join("npm");
        fs::write(&script, "#!/bin/sh\nprintf ran > executed-canary\n").unwrap();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let separator = if cfg!(windows) { ";" } else { ":" };
    let path = format!(
        "{}{}{}",
        shim.display(),
        separator,
        std::env::var("PATH").unwrap_or_default()
    );
    let output = run_verify_json(&fixture.path, &[("PATH", &path)]);
    assert_eq!(output.status.code(), Some(3));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["counts"]["blocked"], 1);
    assert!(!fixture.path.join("executed-canary").exists());
}

#[test]
fn verify_generated_rejects_caller_supplied_project_paths() {
    let fixture = Fixture::new();
    fixture.write("generated-canary.txt", "keep");
    let output = Command::new(env!("CARGO_BIN_EXE_verify-generated"))
        .args(["--path", fixture.path.to_str().unwrap()])
        .output()
        .expect("run verify-generated");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown option --path"), "{stderr}");
    assert_eq!(
        fs::read_to_string(fixture.path.join("generated-canary.txt")).unwrap(),
        "keep"
    );
}

#[test]
fn verify_generated_rejects_untrusted_template_override() {
    let fixture = Fixture::new();
    let output = Command::new(env!("CARGO_BIN_EXE_verify-generated"))
        .args(["--frontend", "react-vite"])
        .current_dir(&fixture.path)
        .env("SNIPE_CODE_TEMPLATES", &fixture.path)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("refuses caller-controlled template overrides")
    );
    assert_eq!(fs::read_dir(&fixture.path).unwrap().count(), 0);
}

#[test]
fn generated_harness_preflight_uses_child_database_environment() {
    let fixture = Fixture::new();
    let shim = fixture.path.join("shim");
    fs::create_dir(&shim).unwrap();
    #[cfg(windows)]
    fs::write(
        shim.join("npm.cmd"),
        "@echo off\r\nif \"%1\"==\"install\" (echo DB_HOST=remote.example>.env & echo DB_DATABASE=test>>.env)\r\nif \"%2\"==\"test\" echo ran>\"%SNIPE_TEST_MARKER%\"\r\nexit /b 0\r\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let script = shim.join("npm");
        fs::write(&script, "#!/bin/sh\nif [ \"$1\" = install ]; then printf 'DB_HOST=remote.example\\nDB_DATABASE=test\\n' > .env; fi\nif [ \"$2\" = test ]; then printf ran > \"$SNIPE_TEST_MARKER\"; fi\nexit 0\n").unwrap();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let path = std::env::join_paths(
        std::iter::once(shim).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let marker = fixture.path.join("executed-canary");
    let output = Command::new(env!("CARGO_BIN_EXE_verify-generated"))
        .args(["--frontend", "react-vite"])
        .current_dir(&fixture.path)
        .env("PATH", path)
        .env("DB_HOST", "localhost")
        .env("DB_DATABASE", "test")
        .env("SNIPE_TEST_MARKER", &marker)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success(), "{stdout}");
    assert!(
        stdout.contains("remote or ambiguous database host"),
        "{stdout}"
    );
    assert!(!marker.exists());
}

#[test]
fn report_argument_errors_exit_before_gates() {
    let fixture = Fixture::new();
    fixture.write(
        "package.json",
        r#"{"name":"app","scripts":{"test":"echo test"}}"#,
    );
    let (stdout, stderr, status) = run_output(
        &["verify", ".", "--report", "pdf", "--output", "report.pdf"],
        &fixture.path,
    );
    assert_eq!(status, Some(2), "{stdout}\n{stderr}");
    assert!(stdout.is_empty(), "{stdout}");
    assert!(stderr.contains("json, junit, or md"), "{stderr}");
    assert!(!fixture.path.join("report.pdf").exists());
}

#[test]
fn report_write_failure_is_runtime_error_on_stderr() {
    let fixture = Fixture::new();
    let (stdout, stderr, status) = run_output(
        &[
            "verify",
            ".",
            "--report",
            "json",
            "--output",
            "missing/report.json",
        ],
        &fixture.path,
    );
    assert_eq!(status, Some(3), "{stdout}\n{stderr}");
    assert!(stdout.is_empty(), "{stdout}");
    assert!(stderr.contains("cannot write"), "{stderr}");
}

#[test]
fn readme_documents_verify_and_its_exit_status_contract() {
    let readme = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("README.md");
    let text = fs::read_to_string(&readme).expect("read README");
    for expected in [
        "snipe-code verify",
        "--only",
        "--security",
        "--report json",
        "--report junit",
        "--report md",
        "--output",
        "--overwrite",
        "not-applicable",
        "unavailable",
        "tidak ada gate yang dijalankan",
        "npm audit",
        "composer audit",
        "dotnet list package --vulnerable",
    ] {
        assert!(
            text.contains(expected),
            "README is missing {expected} in its verify section"
        );
    }
}
