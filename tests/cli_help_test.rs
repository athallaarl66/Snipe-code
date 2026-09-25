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
