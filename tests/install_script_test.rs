//! The POSIX installer: it installs a verified archive, and it fails loudly
//! instead of placing an unverified or partial binary.
//!
//! The tests serve a locally built archive over a `file://` release URL, so they
//! exercise the real download, checksum and extraction paths without a network.

#![cfg(all(unix, target_os = "linux"))]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQ: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let seq = SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "snipe-install-{label}-{}-{seq}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create fixture");
        Fixture { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn install_dir(&self) -> PathBuf {
        self.root.join("bin")
    }

    fn installed_binary(&self) -> PathBuf {
        self.install_dir().join("snipe-code")
    }

    /// A release directory holding `v0.1.0`'s archive and its published checksum.
    /// Returns the base URL directory the installer appends `<version>/` to.
    fn stage_release(&self) -> PathBuf {
        let stage = self.root.join("stage");
        let base = self.root.join("assets");
        let assets = base.join("v0.1.0");
        fs::create_dir_all(&stage).expect("create stage");
        fs::create_dir_all(&assets).expect("create assets");
        fs::write(
            stage.join("snipe-code"),
            "#!/bin/sh\necho 'snipe-code 0.1.0'\n",
        )
        .expect("write binary");
        fs::write(stage.join("LICENSE"), "MIT License\n").expect("write license");

        let archive = assets.join("snipe-code-v0.1.0-x86_64-unknown-linux-gnu.tar.gz");
        let status = Command::new("tar")
            .args(["-czf"])
            .arg(&archive)
            .args(["-C"])
            .arg(&stage)
            .arg(".")
            .status()
            .expect("run tar");
        assert!(status.success(), "tar failed");
        self.publish_checksum(&archive);
        base
    }

    fn publish_checksum(&self, archive: &Path) {
        let out = Command::new("sha256sum")
            .arg(archive)
            .output()
            .expect("run sha256sum");
        assert!(out.status.success(), "sha256sum failed");
        let digest = String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .next()
            .expect("digest")
            .to_string();
        let checksum = archive.with_file_name(format!(
            "{}.sha256",
            archive.file_name().expect("file name").to_string_lossy()
        ));
        fs::write(
            &checksum,
            format!(
                "{digest}  {}\n",
                archive.file_name().unwrap().to_string_lossy()
            ),
        )
        .expect("write checksum");
    }

    fn run_installer(&self, base_url: &str) -> (String, String, Option<i32>) {
        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("install.sh");
        let out = Command::new("bash")
            .arg(script)
            .env("SNIPE_INSTALL_VERSION", "v0.1.0")
            .env("SNIPE_INSTALL_DIR", self.install_dir())
            .env("SNIPE_RELEASE_BASE_URL", base_url)
            .output()
            .expect("run install.sh");
        (
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
            out.status.code(),
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn test_installer_installs_a_verified_archive() {
    let fixture = Fixture::new("success");
    let assets = fixture.stage_release();
    let base_url = format!("file://{}", assets.display());

    let (stdout, stderr, status) = fixture.run_installer(&base_url);

    assert_eq!(status, Some(0), "stdout: {stdout}\nstderr: {stderr}");
    let installed = fixture.installed_binary();
    assert!(
        installed.is_file(),
        "the binary was not installed: {stderr}"
    );
    assert!(
        fs::read_to_string(&installed)
            .expect("read installed")
            .contains("snipe-code 0.1.0"),
        "the installed file is not the released binary"
    );
    assert!(
        stdout.contains("Installed snipe-code v0.1.0"),
        "the installer must report what it installed: {stdout}"
    );
    assert!(
        stdout.contains("PATH"),
        "the installer must print the PATH hint: {stdout}"
    );
}

#[test]
fn test_installer_aborts_on_a_checksum_mismatch() {
    let fixture = Fixture::new("checksum");
    let assets = fixture.stage_release();
    let archive = assets.join("snipe-code-v0.1.0-x86_64-unknown-linux-gnu.tar.gz");
    fs::write(
        archive.with_file_name(format!(
            "{}.sha256",
            archive.file_name().unwrap().to_string_lossy()
        )),
        "0000000000000000000000000000000000000000000000000000000000000000  tampered.tar.gz\n",
    )
    .expect("write wrong checksum");
    let base_url = format!("file://{}", assets.display());

    let (stdout, stderr, status) = fixture.run_installer(&base_url);

    assert_ne!(status, Some(0), "a checksum mismatch must fail: {stdout}");
    assert!(
        stderr.contains("checksum mismatch"),
        "the failure must be explained: {stderr}"
    );
    assert!(
        !fixture.installed_binary().exists(),
        "no binary may be left on the install path"
    );
}

#[test]
fn test_installer_aborts_when_the_asset_cannot_be_fetched() {
    let fixture = Fixture::new("missing");
    let empty = fixture.path().join("empty");
    fs::create_dir_all(&empty).expect("create empty release dir");
    let base_url = format!("file://{}", empty.display());

    let (stdout, stderr, status) = fixture.run_installer(&base_url);

    assert_ne!(status, Some(0), "a failed download must fail: {stdout}");
    assert!(
        stderr.contains("could not download"),
        "the failure must be explained: {stderr}"
    );
    assert!(
        !fixture.installed_binary().exists(),
        "no binary may be left on the install path"
    );
}
