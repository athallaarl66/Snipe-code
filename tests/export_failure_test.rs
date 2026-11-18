use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_snipe-code");
static SEQ: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        loop {
            let dir = std::env::temp_dir().join(format!(
                "snipe-export-failure-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&dir) {
                Ok(()) => return Self(dir),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create fixture: {error}"),
            }
        }
    }

    fn scan(&self) -> PathBuf {
        let project = self.0.join("project");
        fs::create_dir(&project).unwrap();
        let output = Command::new(BIN)
            .arg("scan")
            .arg(&project)
            .arg("--json")
            .current_dir(&self.0)
            .output()
            .expect("run scan");
        assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
        let scan = self.0.join("scan.json");
        fs::write(&scan, output.stdout).unwrap();
        scan
    }

    fn export(&self, scan: &Path, format: &str, destination: &Path) -> Output {
        Command::new(BIN)
            .arg("export")
            .arg(scan)
            .arg("--format")
            .arg(format)
            .arg("--output")
            .arg(destination)
            .current_dir(&self.0)
            .output()
            .expect("run export")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn assert_export_failure(output: &Output, message: &str) {
    assert_eq!(output.status.code(), Some(3), "{}", text(&output.stderr));
    assert!(output.stdout.is_empty(), "{}", text(&output.stdout));
    let stderr = text(&output.stderr);
    assert!(stderr.starts_with("export: "), "{stderr}");
    assert!(!stderr.trim_start_matches("export: ").trim().is_empty());
    assert!(stderr.contains(message), "{stderr}");
    assert!(!stderr.contains("Exported report"), "{stderr}");
}

#[test]
fn unknown_format_is_rejected_before_input_or_output_access() {
    let fixture = Fixture::new();
    let output = fixture.export(&fixture.0.join("missing.json"), "md,csv", &fixture.0);
    assert_export_failure(&output, "unsupported export format: csv");
    assert!(!fixture.0.join("Audit_Report.md").exists());
    assert!(!fixture.0.join("Audit_Report.pdf").exists());
    assert!(!fixture.0.join("Audit_Report.docx").exists());
}

#[test]
fn output_file_is_not_a_writable_directory() {
    let fixture = Fixture::new();
    let scan = fixture.scan();
    let destination = fixture.0.join("not-a-directory");
    fs::write(&destination, b"keep this file").unwrap();
    let output = fixture.export(&scan, "md", &destination);
    assert_export_failure(&output, "output directory does not exist");
    assert_eq!(fs::read(&destination).unwrap(), b"keep this file");
    assert!(!fixture.0.join("Audit_Report.md").exists());
}

#[test]
fn existing_report_blocks_all_requested_exports_without_success_claim() {
    let fixture = Fixture::new();
    let scan = fixture.scan();
    let destination = fixture.0.join("output");
    fs::create_dir(&destination).unwrap();
    let existing = destination.join("Audit_Report.pdf");
    fs::write(&existing, b"keep existing report").unwrap();
    let output = fixture.export(&scan, "md,pdf", &destination);
    assert_export_failure(&output, "output already exists");
    assert_eq!(fs::read(existing).unwrap(), b"keep existing report");
    assert!(!destination.join("Audit_Report.md").exists());
}

#[cfg(unix)]
#[test]
fn unwritable_directory_reports_write_failure() {
    use std::os::unix::fs::PermissionsExt;

    struct RestorePermissions(PathBuf);
    impl Drop for RestorePermissions {
        fn drop(&mut self) {
            let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(0o700));
        }
    }

    let fixture = Fixture::new();
    let scan = fixture.scan();
    let destination = fixture.0.join("output");
    fs::create_dir(&destination).unwrap();
    let _restore = RestorePermissions(destination.clone());
    fs::set_permissions(&destination, fs::Permissions::from_mode(0o500)).unwrap();
    let probe = destination.join("probe");
    if fs::write(&probe, b"probe").is_ok() {
        fs::remove_file(probe).unwrap();
        eprintln!("SKIPPED: current user can write despite directory permissions");
        return;
    }
    let output = fixture.export(&scan, "md", &destination);
    assert_export_failure(&output, "export:");
    assert!(!destination.join("Audit_Report.md").exists());
}
