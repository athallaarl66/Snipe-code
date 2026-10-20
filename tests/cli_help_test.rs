use std::fs;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_snipe-code");

fn run(args: &[&str]) -> (String, bool) {
    let out = Command::new(BIN)
        .args(args)
        .output()
        .expect("failed to run snipe-code");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        out.status.success(),
    )
}

/// Runs the binary and reports stdout plus the exact exit status.
fn run_with_status(args: &[&str]) -> (String, Option<i32>) {
    let out = Command::new(BIN)
        .args(args)
        .output()
        .expect("failed to run snipe-code");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        out.status.code(),
    )
}

fn is_major_minor_patch(value: &str) -> bool {
    value.split('.').count() == 3 && value.chars().all(|c| c.is_ascii_digit() || c == '.')
}

#[test]
fn test_version_command_and_flag_report_crate_version() {
    let (stdout, code) = run_with_status(&["version"]);
    assert_eq!(code, Some(0), "version command must exit 0");
    assert_eq!(
        stdout.lines().count(),
        1,
        "version output must be a single line, got {:?}",
        stdout
    );
    let printed = stdout.trim();
    assert!(
        is_major_minor_patch(printed),
        "version must be major.minor.patch, got {:?}",
        printed
    );
    assert_eq!(printed, env!("CARGO_PKG_VERSION"));

    let (flag_stdout, flag_code) = run_with_status(&["--version"]);
    assert_eq!(flag_code, Some(0), "--version must exit 0");
    assert_eq!(
        flag_stdout, stdout,
        "--version must print the same string as the version command"
    );
}

#[test]
fn test_help_documents_version_and_unknown_tokens_exit_2() {
    let (stdout, ok) = run(&["--help"]);
    assert!(ok);
    assert!(
        stdout.contains("version"),
        "help harus mendokumentasikan command version"
    );
    assert!(
        stdout.contains("--version"),
        "help harus mendokumentasikan flag --version"
    );

    let (_, command_code) = run_with_status(&["nonexistent-cmd"]);
    assert_eq!(command_code, Some(2), "unknown command harus exit 2");

    let (_, option_code) = run_with_status(&["--nonexistent-option"]);
    assert_eq!(option_code, Some(2), "unknown option harus exit 2");
}

#[test]
fn test_help_lists_commands_and_faq() {
    let (stdout, ok) = run(&["--help"]);
    assert!(ok);
    assert!(stdout.contains("new"), "help harus menyebut command new");
    assert!(
        stdout.contains("audit"),
        "help harus menyebut command audit"
    );
    assert!(stdout.contains("FAQ"), "help harus menyertakan FAQ");
    assert!(
        stdout.contains("--dry-run"),
        "help harus menyebut opsi --dry-run"
    );
    assert!(
        stdout.contains("--export"),
        "help harus menyebut opsi --export"
    );
}

#[test]
fn test_help_command_same_as_flag() {
    let (stdout, ok) = run(&["help"]);
    assert!(ok);
    assert!(stdout.contains("COMMANDS"));
    assert!(stdout.contains("audit"));
}

#[test]
fn test_help_audit_lists_export_options() {
    let (stdout, ok) = run(&["help", "audit"]);
    assert!(ok);
    assert!(stdout.contains("--export"));
    assert!(stdout.contains("md,pdf,docx"));
    assert!(stdout.contains("snipe-code audit . --export pdf,docx"));
}

#[test]
fn test_faq_covers_install_and_uninstall() {
    let (stdout, ok) = run(&["faq"]);
    assert!(ok);
    assert!(stdout.contains("cargo install --path ."));
    assert!(stdout.contains("cargo uninstall snipe-code"));
    assert!(stdout.contains("not recognized"));
}

#[test]
fn test_unknown_command_friendly_error() {
    let (stdout, ok) = run(&["nonexistent-cmd"]);
    // Exit code non-zero (error), tapi pesannya ramah
    assert!(!ok);
    assert!(stdout.contains("Unknown command"));
    assert!(stdout.contains("snipe-code help"));
}

#[test]
fn test_unknown_help_topic() {
    let (stdout, _) = run(&["help", "bogus"]);
    assert!(stdout.contains("Unknown topic"));
    assert!(stdout.contains("snipe-code help"));
}

#[test]
fn test_new_rejects_invalid_options_before_mutation() {
    let dir = std::env::temp_dir().join(format!("snipe-new-cli-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for args in [
        vec!["new", "--unknown"],
        vec!["new", "--name"],
        vec!["new", "unexpected"],
        vec!["new", "--frontend", "missing-stack"],
        vec!["new", "--frontend", "none", "--backend", "none"],
    ] {
        let out = Command::new(BIN)
            .args(&args)
            .current_dir(&dir)
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "args: {args:?}");
        assert!(
            !dir.join("my-project").exists(),
            "usage error mutated project: {args:?}"
        );
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_new_accepts_explicit_selection_and_dry_run_alias() {
    let dir = std::env::temp_dir().join(format!("snipe-new-valid-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let out = Command::new(BIN)
        .args([
            "new",
            "--template",
            "none",
            "--frontend",
            "nextjs",
            "--backend",
            "none",
            "--database",
            "none",
            "--name",
            "demo",
            "--dry-run",
        ])
        .current_dir(&dir)
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(!dir.join("demo").exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_audit_non_interactive_defaults_to_markdown() {
    // Dir temp dengan file minimal
    let dir = std::env::temp_dir().join(format!("snipe_audit_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(".env.example"), "FOO=bar\n").unwrap();
    fs::write(dir.join(".gitignore"), "node_modules\n.env\n").unwrap();

    // stdin dinonaktifkan → console::Term::is_term() == false → default md
    let out = Command::new(BIN)
        .arg("audit")
        .arg(dir.to_str().unwrap())
        .stdin(std::process::Stdio::null())
        .output()
        .expect("failed to run audit");

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        stdout.contains("Audit complete"),
        "audit harus selesai tanpa prompt: {}",
        stdout
    );
    assert!(
        dir.join("Audit_Report.md").exists(),
        "default export harus markdown"
    );
    assert!(!dir.join("Audit_Report.pdf").exists());

    let _ = fs::remove_dir_all(&dir);
}
