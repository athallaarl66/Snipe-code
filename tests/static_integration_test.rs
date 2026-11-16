use snipe_code::{audit, history};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const CANARY: &str = "static-canary-839def";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("snipe-static-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("project/src")).unwrap();
        fs::create_dir_all(root.join("reports")).unwrap();
        fs::write(root.join("project/.gitignore"), ".env\n").unwrap();
        fs::write(root.join("project/.env.example"), "").unwrap();
        fs::write(
            root.join("project/src/main.rs"),
            format!(
                r#"
fn main() {{
    let input = std::env::var("INPUT").unwrap();
    std::process::Command::new("sh").arg("-c").arg(input).status();
    let _literal = "{CANARY}";
}}
"#
            ),
        )
        .unwrap();
        Self(root)
    }
    fn project(&self) -> PathBuf {
        self.0.join("project")
    }
    fn run(&self, args: &[&str]) -> Output {
        let output = Command::new(env!("CARGO_BIN_EXE_snipe-code"))
            .args(args)
            .env("SNIPE_CODE_DATA_DIR", self.0.join("data"))
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(!String::from_utf8_lossy(&output.stdout).contains(CANARY));
        assert!(!String::from_utf8_lossy(&output.stderr).contains(CANARY));
        output
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn text(path: &Path) -> &str {
    path.to_str().unwrap()
}

#[test]
fn static_scan_history_export_roundtrip_redacts_and_keeps_identity() {
    let fixture = Fixture::new();
    let project = fixture.project();
    let scan = fixture.run(&["scan", text(&project), "--json"]);
    assert!(scan.status.success());
    assert!(!fixture.0.join("data").exists());
    let saved = fixture.run(&["scan", text(&project), "--json", "--save"]);
    assert!(
        saved.status.success(),
        "{}",
        String::from_utf8_lossy(&saved.stderr)
    );
    let envelope: history::ScanEnvelope = serde_json::from_slice(&saved.stdout).unwrap();
    assert!(!envelope.result.sast.is_empty());
    let finding = envelope
        .result
        .findings
        .iter()
        .find(|finding| finding.category == "Static Analysis")
        .unwrap();
    assert_eq!(finding.rule_id, "sast-rust-shell-tainted");
    assert_eq!(finding.rule_version.as_deref(), Some("1.0.0"));
    assert_ne!(
        finding.fingerprint,
        history::finding_fingerprint(
            history::RULE_VERSION,
            "static-analysis",
            &finding.file,
            finding.line,
            &finding.severity
        )
    );
    let shown = fixture.run(&["history", "show", &envelope.result.run_id]);
    assert!(shown.status.success());
    let shown: history::ScanEnvelope = serde_json::from_slice(&shown.stdout).unwrap();
    assert_eq!(shown, envelope);
    assert_eq!(shown.to_audit_result().sast, envelope.result.sast);
    let mut injected = envelope.clone();
    injected.result.run_id.push_str("-injected");
    injected.result.sast[0].error = Some(CANARY.to_string());
    injected.result.findings[0].description = CANARY.to_string();
    let mut store = history::HistoryStore::open_in(&fixture.0.join("data")).unwrap();
    store.save(&injected).unwrap();
    let sanitized = store.get(&injected.result.run_id).unwrap().unwrap();
    assert!(!serde_json::to_string(&sanitized).unwrap().contains(CANARY));
    drop(store);
    let shown_injected = fixture.run(&["history", "show", &injected.result.run_id]);
    assert!(shown_injected.status.success());
    let listed = fixture.run(&["history", "list"]);
    assert!(String::from_utf8_lossy(&listed.stdout).contains(&envelope.result.run_id));
    let json = fixture.0.join("saved.json");
    assert!(
        fixture
            .run(&[
                "history",
                "export",
                &injected.result.run_id,
                "--output",
                text(&json)
            ])
            .status
            .success()
    );
    let reports = fixture.0.join("reports");
    assert!(
        fixture
            .run(&[
                "export",
                text(&json),
                "--format",
                "md,pdf,docx",
                "--output",
                text(&reports)
            ])
            .status
            .success()
    );
    for extension in ["md", "pdf", "docx"] {
        let bytes = fs::read(reports.join(format!("Audit_Report.{extension}"))).unwrap();
        assert!(!bytes.is_empty());
        assert!(
            !bytes
                .windows(CANARY.len())
                .any(|part| part == CANARY.as_bytes())
        );
    }
    let db = fs::read(fixture.0.join("data/history.sqlite3")).unwrap();
    assert!(
        !db.windows(CANARY.len())
            .any(|part| part == CANARY.as_bytes())
    );
    let legacy = fixture.run(&["audit", text(&project), "--json", "--export", "md"]);
    assert_eq!(legacy.status.code(), Some(1));
    let legacy: serde_json::Value = serde_json::from_slice(&legacy.stdout).unwrap();
    assert!(
        legacy["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["description"]
                .as_str()
                .unwrap()
                .contains("sast-rust-shell-tainted"))
    );
}

#[test]
fn static_statuses_are_not_silently_promoted_and_old_v2_reads() {
    let fixture = Fixture::new();
    let project = fixture.project();
    fs::write(project.join("src/bad.rs"), format!("fn {{ {CANARY}")).unwrap();
    fs::write(project.join("src/large.rs"), " ".repeat(262_145)).unwrap();
    fs::write(project.join("src/unsupported.py"), "print('safe')").unwrap();
    let result = audit::run_audit_scan(text(&project)).unwrap();
    assert!(
        result
            .sast
            .iter()
            .any(|report| report.status == snipe_code::sast::SastStatus::Error)
    );
    assert!(
        result
            .sast
            .iter()
            .any(|report| report.status == snipe_code::sast::SastStatus::Unknown)
    );
    assert!(result.sast.iter().any(|report| report.file == "src/bad.rs"
        && report.status == snipe_code::sast::SastStatus::Error));
    assert!(
        result
            .sast
            .iter()
            .any(|report| report.file == "src/unsupported.py"
                && report.status == snipe_code::sast::SastStatus::Unknown)
    );
    fs::write(
        project.join("src/large.js"),
        format!(
            "{}\nconst password = \"legacy-literal-risk\";",
            " ".repeat(262_145)
        ),
    )
    .unwrap();
    assert!(
        audit::run_audit_scan(text(&project))
            .unwrap()
            .findings
            .iter()
            .any(|finding| finding.category == "Hardcoded Secret"
                && finding.file.ends_with("large.js"))
    );
    let mut result = result;
    result.sast[0].error = Some(CANARY.to_string());
    let envelope = history::ScanEnvelope::from_audit(&project, result).unwrap();
    assert!(!serde_json::to_string(&envelope).unwrap().contains(CANARY));
    let mut json = serde_json::to_value(&envelope).unwrap();
    json["result"]["sast"][0]["status"] = "passed".into();
    let invalid = fixture.0.join("invalid.json");
    fs::write(&invalid, serde_json::to_vec(&json).unwrap()).unwrap();
    assert!(history::read_scan(&invalid).is_err());
    let mut old = serde_json::to_value(&envelope).unwrap();
    old["result"].as_object_mut().unwrap().remove("sast");
    old["result"]["findings"] = serde_json::json!([]);
    old["result"]["high"] = 0.into();
    old["result"]["medium"] = 0.into();
    old["result"]["low"] = 0.into();
    old["result"]["highest_severity"] = serde_json::Value::Null;
    old["result"]["policy_failed"] = false.into();
    let old_path = fixture.0.join("old.json");
    fs::write(&old_path, serde_json::to_vec(&old).unwrap()).unwrap();
    assert!(
        history::read_scan(&old_path)
            .unwrap()
            .result
            .sast
            .is_empty()
    );
}
