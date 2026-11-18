//! Audit must not read linked entries, even when targets are inside the root.
use std::fs;
#[cfg(windows)]
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_snipe-code");
static SEQ: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "snipe-audit-root-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("create isolated fixture");
        fs::create_dir(path.join("project")).unwrap();
        fs::create_dir(path.join("outside")).unwrap();
        fs::write(
            path.join("project/control.js"),
            "const password = 'control';\n",
        )
        .unwrap();
        fs::write(
            path.join("outside/.env"),
            "password=outside-secret-canary\n",
        )
        .unwrap();
        fs::write(
            path.join("outside/escape.js"),
            "const password = 'outside-secret-canary';\n",
        )
        .unwrap();
        fs::write(path.join("outside/gitignore"), "node_modules\n").unwrap();
        fs::write(
            path.join("outside/go.mod"),
            "module outside-secret-canary\n",
        )
        .unwrap();
        Self(path)
    }

    fn root(&self) -> PathBuf {
        self.0.join("project")
    }
    fn outside(&self) -> PathBuf {
        self.0.join("outside")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_scan_and_report(fixture: &Fixture, blocked_paths: &[&str]) {
    let root = fixture.root();
    let output = Command::new(BIN)
        .arg("scan")
        .arg(&root)
        .arg("--json")
        .output()
        .expect("run scan");
    assert_eq!(
        output.status.code(),
        Some(0),
        "scan should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let scan = String::from_utf8(output.stdout).unwrap();
    let _: serde_json::Value = serde_json::from_str(&scan).expect("valid scan JSON");
    assert!(
        scan.contains("control.js"),
        "positive control absent: {scan}"
    );
    let log = String::from_utf8_lossy(&output.stderr);
    for marker in blocked_paths
        .iter()
        .copied()
        .chain(["outside-secret-canary"])
    {
        assert!(!scan.contains(marker), "linked content in scan: {marker}");
        assert!(
            !log.contains(marker),
            "linked content in scan stderr: {marker}"
        );
    }

    let output = Command::new(BIN)
        .arg("audit")
        .arg(&root)
        .args(["--export", "md", "--json"])
        .output()
        .expect("run audit");
    assert_eq!(
        output.status.code(),
        Some(1),
        "positive control must be detected: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = String::from_utf8(output.stdout).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid audit JSON");
    assert!(
        !value["detected_scopes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|scope| scope == "backend:go"),
        "linked backend was traversed"
    );
    let report = fs::read_to_string(root.join("Audit_Report.md")).expect("audit report");
    let log = String::from_utf8_lossy(&output.stderr);
    assert!(json.contains("control.js") && report.contains("control.js"));
    for marker in blocked_paths
        .iter()
        .copied()
        .chain(["outside-secret-canary"])
    {
        for text in [&json[..], &report[..], &log[..]] {
            assert!(!text.contains(marker), "linked content in output: {marker}");
        }
    }
    assert!(
        !value["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["category"] == "Git Security"),
        "linked .gitignore was read"
    );
    assert!(
        !value["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["category"] == "Hardcoded Secret"
                && f["file"].as_str().is_some_and(|p| p.ends_with(".env"))),
        "linked .env was read"
    );
}

#[cfg(unix)]
#[test]
fn posix_file_and_directory_symlinks_stay_outside_scan() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    symlink(f.outside().join(".env"), f.root().join(".env")).unwrap();
    symlink(f.outside().join("escape.js"), f.root().join("linked.js")).unwrap();
    symlink(f.outside().join("gitignore"), f.root().join(".gitignore")).unwrap();
    symlink(f.outside(), f.root().join("linked-directory")).unwrap();
    symlink(f.outside(), f.root().join("backend")).unwrap();
    assert_scan_and_report(&f, &["linked.js", "linked-directory", "escape.js"]);
}

#[cfg(unix)]
#[test]
fn posix_in_root_gitignore_symlink_is_not_read() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    fs::write(f.root().join("gitignore-target"), "node_modules\n").unwrap();
    symlink(
        f.root().join("gitignore-target"),
        f.root().join(".gitignore"),
    )
    .unwrap();
    assert_scan_and_report(&f, &[]);
}

#[cfg(windows)]
fn symlink_file_or_skip(target: &Path, link: &Path) -> bool {
    match std::os::windows::fs::symlink_file(target, link) {
        Ok(()) => true,
        Err(error) if matches!(error.raw_os_error(), Some(1314) | Some(50)) => {
            eprintln!(
                "SKIPPED Windows file symlink: {error} (os error {:?})",
                error.raw_os_error()
            );
            false
        }
        Err(error) => panic!("creating Windows file symlink failed unexpectedly: {error}"),
    }
}

#[cfg(windows)]
#[test]
fn windows_file_symlinks_stay_outside_scan() {
    let f = Fixture::new();
    if !symlink_file_or_skip(&f.outside().join(".env"), &f.root().join(".env")) {
        return;
    }
    assert!(symlink_file_or_skip(
        &f.outside().join("escape.js"),
        &f.root().join("linked.js")
    ));
    assert!(symlink_file_or_skip(
        &f.outside().join("gitignore"),
        &f.root().join(".gitignore")
    ));
    assert_scan_and_report(&f, &["linked.js", "escape.js"]);
}

#[cfg(windows)]
#[test]
fn windows_junction_stays_outside_scan() {
    let f = Fixture::new();
    let junction = f.root().join("backend");
    let output = Command::new("cmd.exe")
        .args(["/D", "/C", "mklink", "/J"])
        .arg(&junction)
        .arg(f.outside())
        .output()
        .expect("run mklink /J");
    assert!(
        output.status.success(),
        "junction creation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_scan_and_report(&f, &["linked-junction", "escape.js"]);
}
