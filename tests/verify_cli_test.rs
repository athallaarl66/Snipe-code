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
