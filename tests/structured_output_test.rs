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

fn run_with_data_dir(args: &[&str], data_dir: &std::path::Path) -> (String, String, Option<i32>) {
    let out = Command::new(BIN)
        .args(args)
        .env("SNIPE_CODE_DATA_DIR", data_dir)
        .env(
            "DATABASE_URL",
            "postgresql://history-user:history-db-canary@127.0.0.1/target",
        )
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
        "const api_key = \"scan-canary-7f93a\";\nexport default api_key;\n",
    );
    fixture.write(".gitignore", "node_modules\n.env\n");
    fixture
}

fn canary_envelope() -> snipe_code::history::ScanEnvelope {
    use snipe_code::history::{
        RULE_VERSION, RuleStatus, SCHEMA_VERSION, ScanEnvelope, ScanFinding, ScanResult,
        finding_fingerprint,
    };

    let rule_id = "hardcoded-secret";
    let file = "src/config.js";
    let severity = "HIGH";
    ScanEnvelope {
        schema_version: SCHEMA_VERSION,
        result: ScanResult {
            run_id: "run-canary".to_string(),
            scanned_at_utc: "2026-11-10T00:00:00Z".to_string(),
            project_id: "project-canary".to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            rule_version: RULE_VERSION.to_string(),
            detected_scopes: vec!["javascript".to_string()],
            evaluated_rule_groups: vec![RuleStatus {
                rule_group: "common".to_string(),
                status: "evaluated".to_string(),
            }],
            exclusions: vec![],
            findings: vec![ScanFinding {
                rule_id: rule_id.to_string(),
                fingerprint_version: 1,
                fingerprint: finding_fingerprint(RULE_VERSION, rule_id, file, 4, severity),
                severity: severity.to_string(),
                category: "Hardcoded Secret".to_string(),
                description: "password=history-canary-7f93a".to_string(),
                file: file.to_string(),
                line: 4,
            }],
            high: 1,
            medium: 0,
            low: 0,
            highest_severity: Some(severity.to_string()),
            policy_failed: true,
        },
    }
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
    assert_eq!(scan["schema_version"], snipe_code::history::SCHEMA_VERSION);
    assert!(
        scan["result"]["evaluated_rule_groups"]
            .as_array()
            .unwrap()
            .iter()
            .any(|status| status["status"] == "unknown")
    );
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
fn scan_history_is_opt_in_and_supports_list_show_export_delete() {
    let fixture = audited_project("scan-history");
    let data_dir = std::env::temp_dir().join(format!(
        "snipe-history-cli-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::SeqCst)
    ));
    let output = std::env::temp_dir().join(format!(
        "snipe-history-export-{}-{}.json",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::SeqCst)
    ));
    let before: Vec<_> = fs::read_dir(&fixture.dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();

    let (scan, stderr, status) = run_with_data_dir(&["scan", fixture.path(), "--json"], &data_dir);
    assert_eq!(status, Some(0), "{stderr}");
    assert!(!scan.contains("scan-canary-7f93a"));
    assert!(!stderr.contains("scan-canary-7f93a"));
    assert!(!scan.contains("history-db-canary"));
    assert!(
        !data_dir.exists(),
        "scan without --save must not create history"
    );

    let (saved, stderr, status) =
        run_with_data_dir(&["scan", fixture.path(), "--json", "--save"], &data_dir);
    assert_eq!(status, Some(0), "{stderr}");
    let result: serde_json::Value = serde_json::from_str(&saved).unwrap();
    let run_id = result["result"]["run_id"].as_str().unwrap();
    let project_id = result["result"]["project_id"].as_str().unwrap();
    assert!(!saved.contains("scan-canary-7f93a"));
    assert!(!stderr.contains("scan-canary-7f93a"));
    assert!(!saved.contains("history-db-canary"));
    assert!(!stderr.contains("history-db-canary"));

    let (listed, stderr, status) = run_with_data_dir(&["history", "list"], &data_dir);
    assert_eq!(status, Some(0), "{stderr}");
    assert!(listed.contains(run_id));
    let (shown, stderr, status) = run_with_data_dir(&["history", "show", run_id], &data_dir);
    assert_eq!(status, Some(0), "{stderr}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&shown).unwrap(),
        result
    );

    let output_text = output.to_string_lossy().into_owned();
    let (exported, stderr, status) = run_with_data_dir(
        &["history", "export", run_id, "--output", &output_text],
        &data_dir,
    );
    assert_eq!(status, Some(0), "{exported} {stderr}");
    assert_eq!(fs::read_to_string(&output).unwrap(), shown);
    assert!(!exported.contains("12345"));

    let (deleted, stderr, status) =
        run_with_data_dir(&["history", "delete", project_id], &data_dir);
    assert_eq!(status, Some(0), "{deleted} {stderr}");
    let (listed, stderr, status) = run_with_data_dir(&["history", "list"], &data_dir);
    assert_eq!(status, Some(0), "{stderr}");
    assert!(listed.is_empty());
    let after: Vec<_> = fs::read_dir(&fixture.dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(before, after, "history must not write into source project");

    let bytes = fs::read(data_dir.join("history.sqlite3")).unwrap();
    let raw_db = String::from_utf8_lossy(&bytes);
    assert!(!raw_db.contains("scan-canary-7f93a"));
    assert!(!raw_db.contains("history-db-canary"));
    let _ = fs::remove_file(output);
    let _ = fs::remove_dir_all(data_dir);
}

#[test]
fn canary_stays_redacted_in_history_output_database_and_report_export() {
    let fixture = Fixture::new("history-canary");
    let data_dir = std::env::temp_dir().join(format!(
        "snipe-history-canary-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::SeqCst)
    ));
    let json_path = fixture.dir.join("history-export.json");
    let scan_path = fixture.dir.join("scan-canary.json");
    fs::write(&scan_path, serde_json::to_vec(&canary_envelope()).unwrap()).unwrap();
    let scanned = snipe_code::history::read_scan(&scan_path).unwrap();
    let scanned_json = serde_json::to_string(&scanned).unwrap();
    assert!(!scanned_json.contains("history-canary-7f93a"));
    assert!(scanned_json.contains("[REDACTED]"));

    let mut store = snipe_code::history::HistoryStore::open_in(&data_dir).unwrap();
    store.save(&canary_envelope()).unwrap();
    drop(store);

    let canary = "history-canary-7f93a";
    let (shown, stderr, status) = run_with_data_dir(&["history", "show", "run-canary"], &data_dir);
    assert_eq!(status, Some(0), "{stderr}");
    assert!(!shown.contains(canary));
    assert!(!stderr.contains(canary));

    let json_path_text = json_path.to_string_lossy().into_owned();
    let (stdout, stderr, status) = run_with_data_dir(
        &[
            "history",
            "export",
            "run-canary",
            "--output",
            &json_path_text,
        ],
        &data_dir,
    );
    assert_eq!(status, Some(0), "{stdout} {stderr}");
    assert!(!stdout.contains(canary));
    assert!(!stderr.contains(canary));
    let json = fs::read_to_string(&json_path).unwrap();
    assert!(!json.contains(canary));
    assert!(json.contains("[REDACTED]"));

    let (_, stderr, status) = run(&[
        "export",
        json_path_text.as_str(),
        "--format",
        "md",
        "--output",
        fixture.path(),
    ]);
    assert_eq!(status, Some(0), "{stderr}");
    let report = fs::read_to_string(fixture.dir.join("Audit_Report.md")).unwrap();
    assert!(!report.contains(canary));
    assert!(!stderr.contains(canary));

    let bytes = fs::read(data_dir.join("history.sqlite3")).unwrap();
    assert!(!String::from_utf8_lossy(&bytes).contains(canary));
    let _ = fs::remove_dir_all(data_dir);
}

#[test]
fn export_preserves_existing_report() {
    let fixture = audited_project("existing-report");
    let (scan, _, status) = run(&["scan", fixture.path(), "--json"]);
    assert_eq!(status, Some(0));
    let input = fixture.write("scan.json", &scan);
    let report = fixture.write("Audit_Report.md", "existing content");
    let (_, _, status) = run(&[
        "export",
        input.to_str().unwrap(),
        "--format",
        "md",
        "--output",
        fixture.path(),
    ]);
    assert_eq!(status, Some(3));
    assert_eq!(fs::read_to_string(report).unwrap(), "existing content");
}

#[test]
fn history_rejects_option_as_id_without_creating_database() {
    let fixture = Fixture::new("history-invalid");
    let data_dir = fixture.dir.join("history");
    let (_, _, status) = run_with_data_dir(&["history", "show", "--help"], &data_dir);
    assert_eq!(status, Some(2));
    assert!(!data_dir.exists());
}

#[test]
fn audit_rejects_ambiguous_options_before_target_io() {
    let fixture = Fixture::new("strict-audit");
    let target = fixture.path();
    for args in [
        vec!["audit", target, "--bogus"],
        vec!["audit", target, "--json", "--json"],
        vec!["audit", target, "--export", "md", "--export", "pdf"],
        vec!["audit", target, "another-path"],
        vec!["audit", target, "--export"],
        vec!["audit", target, "--export", ""],
    ] {
        let (stdout, stderr, status) = run(&args);
        assert_eq!(status, Some(2), "{args:?}: {stderr}");
        assert!(stdout.is_empty(), "{args:?}: {stdout}");
        assert!(!stderr.is_empty(), "{args:?}");
        for report in ["Audit_Report.md", "Audit_Report.pdf", "Audit_Report.docx"] {
            assert!(!fixture.dir.join(report).exists(), "{args:?}: {report}");
        }
    }
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
    assert!(
        help.contains("--save") && help.contains("history"),
        "{help}"
    );

    let (history_help, _, status) = run(&["help", "history"]);
    assert_eq!(status, Some(0));
    for expected in [
        "list",
        "show <run-id>",
        "export <run-id>",
        "delete <project-id>",
    ] {
        assert!(history_help.contains(expected), "{history_help}");
    }

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
