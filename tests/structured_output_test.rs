//! Structured output: `analyze --json`, `audit --json`, and the help text that
//! documents both.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_snipe-code");
static SEQ: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let seq = SEQ.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "snipe-structured-{label}-{}-{seq}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create fixture");
        Fixture {
            dir: fs::canonicalize(&dir).expect("canonicalise fixture"),
        }
    }

    fn write(&self, relative: &str, content: &str) -> PathBuf {
        let path = self.dir.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(&path, content).expect("write fixture file");
        path
    }

    fn path(&self) -> &str {
        self.dir.to_str().expect("fixture path is utf-8")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn run(args: &[&str]) -> (String, String, Option<i32>) {
    let out = Command::new(BIN)
        .args(args)
        .output()
        .expect("run snipe-code");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.code(),
    )
}

/// A project the audit has something to say about: one HIGH hardcoded secret.
fn audited_project(label: &str) -> Fixture {
    let fixture = Fixture::new(label);
    fixture.write(
        "src/config.js",
        "const api_key = \"12345\";\nexport default api_key;\n",
    );
    fixture.write(".gitignore", "node_modules\n.env\n");
    fixture
}

// --- 2.2 structured audit --------------------------------------------------

#[test]
fn test_audit_structured_output_agrees_with_the_exported_report() {
    let fixture = audited_project("audit");
    let (stdout, stderr, status) = run(&["audit", fixture.path(), "--export", "md", "--json"]);

    assert_eq!(
        status,
        Some(1),
        "a HIGH finding must still exit 1: {stderr}"
    );
    let structured: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("stdout is not JSON ({error}): {stdout}"));

    let contexts = structured["detected_scopes"]
        .as_array()
        .expect("detected_scopes array");
    assert!(!contexts.is_empty(), "contexts must be reported: {stdout}");

    let findings = structured["findings"].as_array().expect("findings array");
    assert!(!findings.is_empty(), "the fixture has a finding: {stdout}");
    let high = structured["high"].as_u64().expect("high total");
    assert!(high >= 1, "the hardcoded secret is HIGH: {stdout}");
    for finding in findings {
        for field in ["severity", "category", "description", "file"] {
            assert!(
                finding[field].is_string(),
                "finding is missing {field}: {finding}"
            );
        }
        assert!(
            finding["line"].is_u64(),
            "finding is missing line: {finding}"
        );
    }

    assert_eq!(
        structured["exported_files"],
        serde_json::json!(["Audit_Report.md"])
    );

    // The exported report stays intact and agrees with the structured data.
    let report = fs::read_to_string(fixture.dir.join("Audit_Report.md")).expect("read report");
    assert!(
        report.contains(&format!("- HIGH: {high}")),
        "report totals disagree with the structured output:\n{report}"
    );
    for finding in findings {
        let category = finding["category"].as_str().expect("category");
        let line = finding["line"].as_u64().expect("line");
        assert!(
            report.contains(category),
            "report is missing category {category}:\n{report}"
        );
        assert!(
            report.contains(&format!("(line {line})")),
            "report is missing line {line}:\n{report}"
        );
    }

    // stdout stays parseable, so the human-facing prose moves to stderr.
    assert!(
        !stdout.contains("Exported files"),
        "stdout must stay machine-readable: {stdout}"
    );
    assert!(
        stderr.contains("Exported files") && stderr.contains("Audit_Report.md"),
        "the exported-files summary must move to stderr: {stderr}"
    );
}

#[test]
fn test_audit_without_json_keeps_the_human_summary_on_stdout() {
    let fixture = audited_project("audit-human");
    let (stdout, _, status) = run(&["audit", fixture.path(), "--export", "md"]);

    assert_eq!(status, Some(1));
    assert!(
        stdout.contains("Audit complete! Exported files:") && stdout.contains("Audit_Report.md"),
        "{stdout}"
    );
}

// --- 2.3 documented in the help text ---------------------------------------

#[test]
fn test_help_documents_the_structured_output_options() {
    let (help, _, status) = run(&["help"]);
    assert_eq!(status, Some(0));
    assert!(
        help.contains("--json"),
        "help must document --json:\n{help}"
    );
    assert!(
        help.contains("analyze"),
        "help must list the analyze command:\n{help}"
    );

    let (analyze_help, _, status) = run(&["help", "analyze"]);
    assert_eq!(status, Some(0));
    for expected in ["snipe-code analyze", "--json", "Limits", "read-only"] {
        assert!(
            analyze_help.contains(expected),
            "analyze help is missing {expected}:\n{analyze_help}"
        );
    }

    let (audit_help, _, status) = run(&["help", "audit"]);
    assert_eq!(status, Some(0));
    for expected in ["--json", "stderr", "stdout"] {
        assert!(
            audit_help.contains(expected),
            "audit help is missing {expected}:\n{audit_help}"
        );
    }
}
