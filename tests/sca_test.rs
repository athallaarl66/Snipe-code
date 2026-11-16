use serde_json::{Value, json};
use snipe_code::sca;
use std::{fs, path::PathBuf};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let base = std::env::var_os("TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let p = base.join(format!("snipe-sca-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }
    fn lock(&self, v: Value) {
        fs::write(
            self.0.join("package-lock.json"),
            serde_json::to_vec(&v).unwrap(),
        )
        .unwrap();
    }
    fn cache(&self) -> PathBuf {
        self.0.join("cache")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn graph() -> Value {
    json!({"lockfileVersion":3,"packages":{"":{"dependencies":{"first":"1.0.0"}},"node_modules/first":{"version":"1.0.0","dependencies":{"second":"2.0.0"}},"node_modules/second":{"version":"2.0.0"}}})
}
// Synthetic advisory: no bundled live advisory data or downloaded prose.
fn response(name: &str) -> Vec<u8> {
    if name == "first" {
        return b"{}".to_vec();
    }
    serde_json::to_vec(&json!({"vulns":[{"id":"GHSA-2345-6789-cfgh","published":"2026-01-01T00:00:00Z","modified":"2026-01-02T00:00:00Z","references":[{"url":"https://github.com/advisories/GHSA-2345-6789-cfgh"}],"affected":[{"package":{"ecosystem":"npm","name":"second"}}],"database_specific":{"severity":"HIGH"},"summary":"DO_NOT_EMIT_ADVISORY_PROSE"}]})).unwrap()
}
fn fake(endpoint: &str, body: &[u8]) -> Result<Vec<u8>, String> {
    assert_eq!(endpoint, sca::ENDPOINT);
    let v: Value = serde_json::from_slice(body).unwrap();
    assert_eq!(v.as_object().unwrap().len(), 2);
    assert_eq!(v["package"]["ecosystem"], "npm");
    assert_eq!(v["package"].as_object().unwrap().len(), 2);
    Ok(response(v["package"]["name"].as_str().unwrap()))
}
#[test]
fn offline_direct_transitive_and_provenance() {
    let f = Fixture::new();
    f.lock(graph());
    let snapshot = sca::refresh(&f.0, &f.cache(), true, fake, 100).unwrap();
    assert_eq!(sca::active_snapshot(&f.cache()).unwrap(), snapshot);
    let r = sca::analyze(&f.0, Some(&f.cache()), 101);
    assert_eq!(r.status, "vulnerable");
    assert_eq!(r.packages.len(), 2);
    assert!(r.packages[0].direct);
    assert!(!r.packages[1].direct);
    assert_eq!(r.packages[0].status, "clean");
    assert_eq!(
        r.packages[1].advisory_id.as_deref(),
        Some("GHSA-2345-6789-cfgh")
    );
    assert_eq!(r.packages[1].severity, "high");
    let p = r.snapshot.as_ref().unwrap();
    assert_eq!(p.license, "CC-BY-4.0");
    assert_eq!(p.payload_sha256.len(), 64);
    assert!(!serde_json::to_string(&r).unwrap().contains("DO_NOT_EMIT"));
    assert_eq!(
        sca::analyze(&f.0, Some(&snapshot), 100 + sca::MAX_AGE - 1).status,
        "vulnerable"
    );
    assert_eq!(
        sca::analyze(&f.0, Some(&snapshot), 100 + sca::MAX_AGE).status,
        "stale"
    );
    assert_eq!(sca::analyze(&f.0, Some(&snapshot), 99).status, "unknown");
}
#[test]
fn refusal_makes_zero_requests_and_writes() {
    let f = Fixture::new();
    let result = sca::refresh(
        &f.0,
        &f.cache(),
        false,
        |_, _| panic!("network forbidden"),
        100,
    );
    assert!(result.is_err());
    assert!(!f.cache().exists());
}
#[test]
fn invalid_refresh_preserves_previous_and_replacement_works() {
    let f = Fixture::new();
    f.lock(graph());
    sca::refresh(&f.0, &f.cache(), true, fake, 100).unwrap();
    let before = fs::read(f.cache().join("current")).unwrap();
    for bytes in [
        b"invalid".to_vec(),
        b"{\"vulns\":[{\"id\":\"CVE-2026-1234\"}]}".to_vec(),
        b"{\"next_page_token\":\"secret\"}".to_vec(),
    ] {
        assert!(sca::refresh(&f.0, &f.cache(), true, |_, _| Ok(bytes.clone()), 101).is_err());
        assert_eq!(fs::read(f.cache().join("current")).unwrap(), before);
    }
    let path = sca::refresh(&f.0, &f.cache(), true, fake, 102).unwrap();
    assert_ne!(fs::read(f.cache().join("current")).unwrap(), before);
    assert_eq!(sca::active_snapshot(&f.cache()).unwrap(), path);
}
#[test]
fn missing_unsupported_and_malformed_graphs_unknown_without_fetch() {
    let f = Fixture::new();
    assert_eq!(sca::analyze(&f.0, None, 100).status, "unknown");
    for lock in [
        json!({"lockfileVersion":2}),
        json!({"lockfileVersion":3,"packages":{}}),
        json!({"lockfileVersion":3,"packages":{"":{"dependencies":{"missing":"1.0.0"}}}}),
    ] {
        f.lock(lock);
        assert_eq!(sca::analyze(&f.0, None, 100).status, "unknown");
        assert!(
            sca::refresh(
                &f.0,
                &f.cache(),
                true,
                |_, _| panic!("invalid graph fetched"),
                100
            )
            .is_err()
        );
    }
    for patch in [
        json!({"link":true}),
        json!({"version":"git+https://example.invalid/a"}),
        json!({"version":"1.0.0","resolved":"file:../local"}),
    ] {
        let mut lock = graph();
        lock["packages"]["node_modules/first"] = patch;
        f.lock(lock);
        assert!(
            sca::refresh(
                &f.0,
                &f.cache(),
                true,
                |_, _| panic!("unsupported package fetched"),
                100
            )
            .is_err()
        );
    }
}
#[test]
fn corruption_partial_coverage_and_missing_snapshot_unknown() {
    let f = Fixture::new();
    f.lock(graph());
    assert_eq!(sca::analyze(&f.0, None, 100).status, "unknown");
    let path = sca::refresh(&f.0, &f.cache(), true, fake, 100).unwrap();
    let mut lock = graph();
    lock["packages"]["node_modules/third"] = json!({"version":"3.0.0"});
    f.lock(lock);
    assert_eq!(sca::analyze(&f.0, Some(&path), 101).status, "unknown");
    let mut v: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    v["payload"][0]["version"] = json!("9.9.9");
    fs::write(&path, serde_json::to_vec(&v).unwrap()).unwrap();
    assert_eq!(sca::analyze(&f.0, Some(&path), 101).status, "unknown");
    assert!(sca::active_snapshot(&f.cache()).is_err());
}
#[test]
fn transport_rejects_other_endpoint_without_spawn() {
    assert!(sca::fetch_osv("http://example.invalid", b"secret").is_err());
}

#[test]
fn invalid_timestamps_and_unicode_pointer_are_unknown() {
    let f = Fixture::new();
    f.lock(graph());
    let path = sca::refresh(&f.0, &f.cache(), true, fake, 100).unwrap();
    let original = fs::read(&path).unwrap();
    for stamp in [Value::Null, json!(0), json!(102), json!("invalid")] {
        let mut v: Value = serde_json::from_slice(&original).unwrap();
        v["provenance"]["fetched_at"] = stamp;
        fs::write(&path, serde_json::to_vec(&v).unwrap()).unwrap();
        assert_eq!(sca::analyze(&f.0, Some(&path), 101).status, "unknown");
    }
    let mut v: Value = serde_json::from_slice(&original).unwrap();
    v["provenance"]
        .as_object_mut()
        .unwrap()
        .remove("expires_at");
    fs::write(&path, serde_json::to_vec(&v).unwrap()).unwrap();
    assert_eq!(sca::analyze(&f.0, Some(&path), 101).status, "unknown");
    fs::write(
        f.cache().join("current"),
        format!("{}é.json", "a".repeat(62)),
    )
    .unwrap();
    assert!(sca::active_snapshot(&f.cache()).is_err());
}

#[test]
fn cli_offline_snapshot_history_and_consent() {
    use std::process::{Command, Stdio};
    let f = Fixture::new();
    f.lock(graph());
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_snipe-code"))
            .args(args)
            .env("SNIPE_CODE_DATA_DIR", f.0.join("data"))
            .stdin(Stdio::null())
            .output()
            .unwrap()
    };
    let root = f.0.to_str().unwrap();
    let cache = f.cache();
    let cache_str = cache.to_str().unwrap();
    for args in [
        vec!["advisory", "refresh", root, "--cache", cache_str],
        vec![
            "advisory", "refresh", root, "--cache", cache_str, "--yes", "extra",
        ],
        vec!["scan", root, "--snapshot"],
        vec!["scan", root, "--snapshot", "x", "--snapshot", "y"],
    ] {
        assert_eq!(run(&args).status.code(), Some(2));
        assert!(!cache.exists());
        assert!(!f.0.join("data").exists());
    }
    let help = run(&["help", "advisory"]);
    assert!(String::from_utf8_lossy(&help.stdout).contains("--yes"));
    let unknown = run(&["scan", root, "--json"]);
    assert!(unknown.status.success());
    assert!(!f.0.join("data").exists());
    let unknown: Value = serde_json::from_slice(&unknown.stdout).unwrap();
    assert_eq!(unknown["result"]["sca"]["status"], "unknown");
    for (stamp, expected) in [(sca::now(), "vulnerable"), (100, "stale")] {
        sca::refresh(&f.0, &cache, true, fake, stamp).unwrap();
        let scan = run(&["scan", root, "--snapshot", cache_str, "--json", "--save"]);
        assert!(
            scan.status.success(),
            "{}",
            String::from_utf8_lossy(&scan.stderr)
        );
        assert!(!String::from_utf8_lossy(&scan.stdout).contains("DO_NOT_EMIT"));
        let envelope: snipe_code::history::ScanEnvelope =
            serde_json::from_slice(&scan.stdout).unwrap();
        let report = envelope.result.sca.as_ref().unwrap();
        assert_eq!(report.status, expected);
        assert_eq!(
            report.packages[1].advisory_id.as_deref(),
            Some("GHSA-2345-6789-cfgh")
        );
        let shown = run(&["history", "show", &envelope.result.run_id]);
        assert!(shown.status.success());
        let shown: snipe_code::history::ScanEnvelope =
            serde_json::from_slice(&shown.stdout).unwrap();
        assert_eq!(shown.result.sca, envelope.result.sca);
        assert_eq!(shown.to_audit_result().sca, envelope.result.sca);
        let mut bad = serde_json::to_value(&envelope).unwrap();
        bad["result"]["sca"]["packages"][0]["status"] = json!("passed");
        let bad_path = f.0.join("bad.json");
        fs::write(&bad_path, serde_json::to_vec(&bad).unwrap()).unwrap();
        assert!(snipe_code::history::read_scan(&bad_path).is_err());
    }
}
