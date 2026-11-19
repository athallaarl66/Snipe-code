use snipe_code::{audit, history, quality::*};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "snipe-quality-integration-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }
    fn write(&self, path: &str, text: &str) {
        fs::write(self.0.join(path), text).unwrap();
    }
    fn scan(&self) -> history::ScanEnvelope {
        let mut envelope = history::ScanEnvelope::from_audit(
            &self.0,
            audit::run_audit_scan(self.0.to_str().unwrap()).unwrap(),
        )
        .unwrap();
        // Synthetic evaluated-scope fixture; real legacy heuristic scope stays unknown.
        envelope.result.evaluated_rule_groups = vec![history::RuleStatus {
            rule_group: "fixture-reviewed".into(),
            status: "evaluated".into(),
        }];
        envelope
    }
    fn git(&self, args: &[&str]) -> String {
        let result = Command::new("git")
            .args(args)
            .current_dir(&self.0)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env(
                "GIT_CONFIG_GLOBAL",
                if cfg!(windows) { "NUL" } else { "/dev/null" },
            )
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        String::from_utf8(result.stdout).unwrap()
    }
    fn repo() -> Self {
        let f = Self::new();
        f.write("a.rs", "fn main() {}\n");
        f.git(&["init", "--initial-branch=main"]);
        f.git(&["add", "a.rs"]);
        f.git(&[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            "fixture",
        ]);
        f
    }
    fn cli(&self, args: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_snipe-code"))
            .args(args)
            .current_dir(&self.0)
            .env("SNIPE_CODE_DATA_DIR", self.0.join("data"))
            .stdin(Stdio::null())
            .output()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn policy(id: &str, unit: &str, threshold: u64) -> Policy {
    Policy {
        id: id.into(),
        version: 1,
        unit: unit.into(),
        required: true,
        comparison: if id == "lcov-line-coverage" {
            Comparison::Minimum
        } else {
            Comparison::Maximum
        },
        threshold,
        unknown: Missing::Unknown,
        exclusions: vec![],
    }
}
fn config(metrics: Vec<Policy>) -> Config {
    Config {
        version: 1,
        enabled: true,
        metrics,
    }
}

#[test]
fn baseline_reference_persists_exact_metadata_and_old_envelope_still_reads() {
    let f = Fixture::repo();
    let base = f.scan();
    let mut current = f.scan();
    let c = config(vec![policy("new-findings", "findings", 0)]);
    let report = evaluate(&f.0, &c, None, &current, Some(&base), Some("HEAD")).unwrap();
    assert_eq!(report.state, State::Pass, "{:?}", report.delta);
    let metadata = report.delta.as_ref().unwrap().baseline.clone().unwrap();
    assert_eq!(metadata.run_id, base.result.run_id);
    assert_eq!(metadata.schema_version, base.schema_version);
    assert_eq!(metadata.rule_version, base.result.rule_version);
    current.result.quality = Some(report);
    let data = Fixture::new();
    let mut store = history::HistoryStore::open_in(&data.0).unwrap();
    store.save(&current).unwrap();
    let saved = store.get(&current.result.run_id).unwrap().unwrap();
    assert_eq!(saved.result.quality, current.result.quality);
    assert_eq!(
        saved
            .result
            .quality
            .unwrap()
            .delta
            .unwrap()
            .baseline
            .unwrap(),
        metadata
    );
    let mut old = serde_json::to_value(&current).unwrap();
    old["result"].as_object_mut().unwrap().remove("quality");
    f.write("old.json", &old.to_string());
    assert!(
        history::read_scan(&f.0.join("old.json"))
            .unwrap()
            .result
            .quality
            .is_none()
    );
}

#[test]
fn delta_stable_new_resolved_and_version_mismatch() {
    let f = Fixture::repo();
    // Add a deterministic legacy finding to both validated envelopes.
    f.write(
        "a.rs",
        "fn main() { let password = \"delta-fixture-value\"; }\n",
    );
    let base = f.scan();
    assert!(!base.result.findings.is_empty());
    let mut current = base.clone();
    let stable = compare(&f.0, &current, Some(&base), Some("HEAD"));
    assert_eq!(stable.state, State::Pass);
    assert_eq!(stable.unchanged.len(), base.result.findings.len());
    current.result.findings.clear();
    let resolved = compare(&f.0, &current, Some(&base), Some("HEAD"));
    assert_eq!(resolved.resolved.len(), base.result.findings.len());
    let new = compare(&f.0, &base, Some(&current), Some("HEAD"));
    assert_eq!(new.new.len(), base.result.findings.len());
    current = base.clone();
    current.result.rule_version = "heuristic-2".into();
    let unknown = compare(&f.0, &current, Some(&base), Some("HEAD"));
    assert_eq!(unknown.state, State::Unknown);
    assert!(unknown.resolved.is_empty());
    current = base.clone();
    current.result.findings[0].fingerprint_version = 2;
    assert_eq!(
        compare(&f.0, &current, Some(&base), Some("HEAD")).state,
        State::Unknown
    );
    current = base.clone();
    current.result.findings[0].rule_version = Some("2.0.0".into());
    assert_eq!(
        compare(&f.0, &current, Some(&base), Some("HEAD")).state,
        State::Unknown
    );
}

#[test]
fn missing_base_rename_and_merge_conflict_never_resolve() {
    let f = Fixture::repo();
    let base = f.scan();
    for d in [
        compare(&f.0, &base, None, Some("HEAD")),
        compare(&f.0, &base, Some(&base), None),
        compare(&f.0, &base, Some(&base), Some("absent-ref")),
    ] {
        assert_eq!(d.state, State::Unknown);
        assert!(d.resolved.is_empty());
        assert!(d.new.is_empty());
    }
    f.git(&["mv", "a.rs", "renamed.rs"]);
    let d = compare(&f.0, &base, Some(&base), Some("HEAD"));
    assert_eq!(d.state, State::Unknown);
    assert!(d.resolved.is_empty());
    f.git(&["reset", "--hard", "HEAD"]);
    let blob = f.git(&["rev-parse", "HEAD:a.rs"]).trim().to_string();
    f.git(&["update-index", "--force-remove", "a.rs"]);
    let info = format!("100644 {blob} 1\ta.rs\n100644 {blob} 2\ta.rs\n100644 {blob} 3\ta.rs\n");
    use std::io::Write;
    let mut child = Command::new("git")
        .args(["update-index", "--index-info"])
        .current_dir(&f.0)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(info.as_bytes())
        .unwrap();
    assert!(child.wait().unwrap().success());
    let d = compare(&f.0, &base, Some(&base), Some("HEAD"));
    assert_eq!(d.state, State::Unknown);
    assert!(d.resolved.is_empty());
}

#[test]
fn gate_required_missing_mixed_disabled_and_explicit_failure() {
    let f = Fixture::new();
    f.write("a.rs", "fn main() {}\n");
    let current = f.scan();
    let mut c = config(vec![policy("lcov-line-coverage", "basis-points", 0)]);
    let r = evaluate(&f.0, &c, None, &current, None, None).unwrap();
    assert_eq!(r.state, State::Unknown);
    assert_eq!(exit_status(&r), 3);
    validate_report(&r).unwrap();
    c.metrics.push(policy("rust-function-lines", "lines", 1));
    let r = evaluate(&f.0, &c, None, &current, None, None).unwrap();
    assert_eq!(r.state, State::Unknown);
    assert_eq!(exit_status(&r), 4);
    c.metrics[1].threshold = 0;
    let r = evaluate(&f.0, &c, None, &current, None, None).unwrap();
    assert_eq!(r.state, State::Fail);
    assert_eq!(exit_status(&r), 1);
    c.metrics[0].unknown = Missing::Fail;
    assert_eq!(
        evaluate(&f.0, &c, None, &current, None, None)
            .unwrap()
            .metrics[0]
            .state,
        State::Fail
    );
    c.enabled = false;
    let r = evaluate(&f.0, &c, None, &current, None, None).unwrap();
    assert_eq!(r.state, State::NotApplicable);
    assert_eq!(exit_status(&r), 0);
    validate_report(&r).unwrap();
}

#[test]
fn quality_cli_unknown_policy_and_runtime_errors_are_non_success() {
    let f = Fixture::new();
    f.write("a.rs", "fn main() {}\n");
    for (missing, required, expected, state) in [
        (Missing::Unknown, true, 3, "unknown"),
        (Missing::Unknown, false, 3, "unknown"),
        (Missing::Fail, true, 1, "fail"),
    ] {
        let mut p = policy("lcov-line-coverage", "basis-points", 0);
        p.unknown = missing;
        p.required = required;
        f.write(
            "config.json",
            &serde_json::to_string(&config(vec![p])).unwrap(),
        );
        for json in [false, true] {
            let mut args = vec!["quality", "--config", "config.json"];
            if json {
                args.push("--json");
            }
            let out = f.cli(&args);
            assert_eq!(out.status.code(), Some(expected));
            assert!(out.stderr.is_empty());
            let stdout = String::from_utf8(out.stdout).unwrap();
            if json {
                let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
                assert_eq!(value["result"]["quality"]["state"], state);
                assert!(value["result"]["quality"]["metrics"][0]["value"].is_null());
            } else {
                assert!(stdout.contains(&format!("Quality gate: {state}\n")));
                assert!(stdout.contains("coverage report missing"));
            }
            assert!(!f.0.join("data").exists());
        }
    }
    f.write("config.json", "{}");
    let out = f.cli(&["quality", "--config", "config.json", "--save"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());
    assert!(!f.0.join("data").exists());
    let c = config(vec![policy("rust-function-lines", "lines", 1)]);
    f.write("config.json", &serde_json::to_string(&c).unwrap());
    let out = f.cli(&[
        "quality",
        "missing-root",
        "--config",
        "config.json",
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(3));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("source root unavailable"));
    f.write("data", "history target is a file");
    let out = f.cli(&["quality", "--config", "config.json", "--json", "--save"]);
    assert_eq!(out.status.code(), Some(3));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("cannot save history"));
    assert_eq!(
        fs::read_to_string(f.0.join("data")).unwrap(),
        "history target is a file"
    );
}

#[test]
fn quality_cli_missing_baseline_preserves_reason_in_text_and_json() {
    let f = Fixture::new();
    f.write("a.rs", "fn main() {}\n");
    let c = config(vec![policy("new-findings", "findings", 0)]);
    f.write("config.json", &serde_json::to_string(&c).unwrap());
    for json in [false, true] {
        let mut args = vec![
            "quality",
            "--config",
            "config.json",
            "--baseline",
            "missing.json",
            "--base-ref",
            "HEAD",
        ];
        if json {
            args.push("--json");
        }
        let out = f.cli(&args);
        assert_eq!(out.status.code(), Some(3));
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert!(stdout.contains("baseline missing or invalid"), "{stdout}");
        assert!(out.stderr.is_empty());
        if json {
            let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
            assert_eq!(value["result"]["quality"]["delta"]["state"], "unknown");
            assert_eq!(
                value["result"]["quality"]["delta"]["new"],
                serde_json::json!([])
            );
            assert_eq!(
                value["result"]["quality"]["delta"]["resolved"],
                serde_json::json!([])
            );
        }
    }
    assert!(!f.0.join("data").exists());
}

#[test]
fn documented_quality_config_runs_with_imported_not_computed_coverage() {
    let docs = include_str!("../README.md").replace("\r\n", "\n");
    let example = docs
        .split_once("<!-- quality-config-example -->\n```json\n")
        .unwrap()
        .1
        .split_once("\n```")
        .unwrap()
        .0;
    let c = parse_config(example).unwrap();
    assert_eq!(c.metrics.len(), 4);
    let f = Fixture::new();
    f.write("a.rs", "fn main() {}\n");
    f.write("config.json", example);
    f.write("lcov.info", "SF:a.rs\nDA:1,1\nend_of_record\n");
    for coverage in [false, true] {
        let mut args = vec!["quality", "--config", "config.json", "--json"];
        if coverage {
            args.extend(["--coverage", "lcov.info"]);
        }
        let out = f.cli(&args);
        assert_eq!(out.status.code(), Some(4));
        assert!(out.stderr.is_empty());
        let envelope: history::ScanEnvelope = serde_json::from_slice(&out.stdout).unwrap();
        let report = envelope.result.quality.unwrap();
        validate_report(&report).unwrap();
        assert_eq!(report.config, c);
        assert_eq!(report.metrics[0].value, Some(1));
        assert_eq!(report.metrics[1].value, Some(0));
        assert_eq!(report.metrics[2].value, coverage.then_some(10_000));
        assert_eq!(report.metrics[2].coverage.is_some(), coverage);
        assert_eq!(report.metrics[3].state, State::Unknown);
        assert!(!f.0.join("data").exists());
    }
    assert!(docs.contains("static scan does not compute coverage"));
    assert!(docs.contains("<!-- quality-config-example -->"));
}

#[test]
fn quality_cli_exit_codes_read_only_and_saved_roundtrip() {
    let f = Fixture::new();
    f.write("a.rs", "fn main() {}\n");
    let before = fs::read(f.0.join("a.rs")).unwrap();
    let mut entries = fs::read_dir(&f.0)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    entries.sort();
    for args in [
        vec!["quality"],
        vec!["quality", "--config"],
        vec!["quality", "--config", "absent", "--save", "--save"],
        vec!["quality", "--config", "absent", "--baseline", "x"],
        vec!["quality", "--bogus"],
        vec!["quality", "--config", "absent", "--config", "absent"],
        vec!["quality", "--config", "absent", "--json", "--json"],
        vec!["quality", "--config", "absent", "--coverage"],
        vec!["quality", "--config", "absent", "--base-ref", "HEAD"],
        vec!["quality", "--config", "absent", ".", "."],
        vec!["quality", "--help", "--save"],
    ] {
        let out = f.cli(&args);
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert!(!out.stderr.is_empty());
        let mut after = fs::read_dir(&f.0)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        after.sort();
        assert_eq!(after, entries);
        assert!(!f.0.join("data").exists());
    }
    for args in [["quality", "--help"], ["help", "quality"]] {
        let out = f.cli(&args);
        assert!(out.status.success());
        assert!(String::from_utf8_lossy(&out.stdout).contains("--config"));
    }
    let mut c = config(vec![policy("rust-function-lines", "lines", 1)]);
    for (enabled, coverage, threshold, expected, state) in [
        (true, false, 1, 0, "pass"),
        (true, true, 1, 4, "unknown"),
        (true, true, 0, 1, "fail"),
        (false, true, 0, 0, "not-applicable"),
    ] {
        c.enabled = enabled;
        c.metrics[0].threshold = threshold;
        c.metrics.truncate(1);
        if coverage {
            c.metrics
                .push(policy("lcov-line-coverage", "basis-points", 0));
        }
        f.write("config.json", &serde_json::to_string(&c).unwrap());
        let out = f.cli(&["quality", "--config", "config.json", "--json"]);
        assert_eq!(
            out.status.code(),
            Some(expected),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(value["result"]["quality"]["state"], state);
        assert!(!f.0.join("data").exists());
        let text = f.cli(&["quality", "--config", "config.json"]);
        assert_eq!(text.status.code(), Some(expected));
        assert!(
            String::from_utf8_lossy(&text.stdout).contains(&format!("Quality gate: {state}\n")),
            "{}",
            String::from_utf8_lossy(&text.stdout)
        );
        assert!(out.stderr.is_empty());
        assert!(text.stderr.is_empty());
        f.write("current.json", &String::from_utf8(out.stdout).unwrap());
        let roundtrip = history::read_scan(&f.0.join("current.json")).unwrap();
        assert_eq!(serde_json::to_value(roundtrip).unwrap(), value);
    }
    c = config(vec![policy("lcov-line-coverage", "basis-points", 0)]);
    f.write("config.json", &serde_json::to_string(&c).unwrap());
    let out = f.cli(&["quality", "--config", "config.json", "--json", "--save"]);
    assert_eq!(out.status.code(), Some(3));
    let envelope: history::ScanEnvelope = serde_json::from_slice(&out.stdout).unwrap();
    assert!(String::from_utf8_lossy(&out.stderr).contains("Saved scan history"));
    let shown = f.cli(&["history", "show", &envelope.result.run_id]);
    assert!(shown.status.success());
    let shown: history::ScanEnvelope = serde_json::from_slice(&shown.stdout).unwrap();
    assert_eq!(shown, envelope);
    f.write("scan.json", &String::from_utf8(out.stdout).unwrap());
    assert_eq!(
        history::read_scan(&f.0.join("scan.json")).unwrap(),
        envelope
    );
    let out = f.cli(&[
        "history",
        "export",
        &envelope.result.run_id,
        "--output",
        "export.json",
    ]);
    assert!(out.status.success());
    assert_eq!(
        history::read_scan(&f.0.join("export.json")).unwrap(),
        envelope
    );
    let mut bad = serde_json::to_value(&envelope).unwrap();
    bad["result"]["quality"]["state"] = serde_json::json!("pass");
    f.write("bad.json", &bad.to_string());
    assert!(history::read_scan(&f.0.join("bad.json")).is_err());
    assert_eq!(fs::read(f.0.join("a.rs")).unwrap(), before);
}
