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

#[test]
fn scan_json_is_read_only_and_export_requires_versioned_input() {
    let fixture = audited_project("scan-export");
    let before: Vec<_> = fs::read_dir(&fixture.dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    let (stdout, _, status) = run(&["scan", fixture.path(), "--json"]);
    let after: Vec<_> = fs::read_dir(&fixture.dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(
        before, after,
        "scan must not change project directory entries"
    );
    assert_eq!(status, Some(0));
    let scan: serde_json::Value = serde_json::from_str(&stdout).expect("scan JSON");
    assert_eq!(scan["schema_version"], 1);
    assert!(!fixture.dir.join("Audit_Report.md").exists());

    let scan_file = fixture.write("scan.json", &stdout);
    let (export_stdout, _, export_status) = run(&[
        "export",
        scan_file.to_str().unwrap(),
        "--format",
        "md",
        "--output",
        fixture.path(),
    ]);
    assert_eq!(export_status, Some(0), "{export_stdout}");
    assert!(fixture.dir.join("Audit_Report.md").exists());

    let bad = fixture.write("bad.json", "{}");
    let (_, _, bad_status) = run(&[
        "export",
        bad.to_str().unwrap(),
        "--format",
        "md",
        "--output",
        fixture.path(),
    ]);
    assert_eq!(bad_status, Some(3));
}

#[test]
fn export_rejects_unknown_format_and_missing_output_without_creating_files() {
    let fixture = audited_project("export-errors");
    let (scan_stdout, _, scan_status) = run(&["scan", fixture.path(), "--json"]);
    assert_eq!(scan_status, Some(0));
    let scan_file = fixture.write("scan.json", &scan_stdout);

    let (_, unknown_stderr, unknown_status) = run(&[
        "export",
        scan_file.to_str().unwrap(),
        "--format",
        "exe",
        "--output",
        fixture.path(),
    ]);
    assert_eq!(unknown_status, Some(3));
    assert!(unknown_stderr.contains("unsupported export format"));
    assert!(!fixture.dir.join("Audit_Report.md").exists());

    let missing = fixture.dir.join("missing-output");
    let (_, missing_stderr, missing_status) = run(&[
        "export",
        scan_file.to_str().unwrap(),
        "--format",
        "md",
        "--output",
        missing.to_str().unwrap(),
    ]);
    assert_eq!(missing_status, Some(3));
    assert!(missing_stderr.contains("output directory does not exist"));
    assert!(!missing.exists());
}

#[test]
fn audit_rejects_ambiguous_options_before_target_io() {
    let fixture = Fixture::new("strict-audit");
    let (stdout, stderr, status) = run(&["audit", fixture.path(), "--bogus"]);
    assert_eq!(status, Some(2));
    assert!(!fixture.dir.join("Audit_Report.md").exists());
    assert!(stdout.is_empty());
    assert!(stderr.contains("unknown option"));
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
