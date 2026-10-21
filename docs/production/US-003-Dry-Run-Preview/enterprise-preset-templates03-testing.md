# US-003: Dry-Run Preview — Unit Tests

> Status: these are proposed cases, not executed tests. The executed suite lives in
> `tests/`, and `docs/uat/UAT-MATRIX.md` maps each case to the suite that covers it.

### test_dry_run_does_not_create_files
```rust
#[test]
fn test_dry_run_does_not_create_files() {
    let dir = tempfile::tempdir().unwrap();
    let out = Command::new(BIN)
        .args(["new", "--template", "none", "--frontend", "nextjs",
               "--backend", "none", "--database", "none", "--name", "demo",
               "--dry-run"])
        .current_dir(dir.path())
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(!dir.path().join("demo").exists());
}
```

### test_dry_run_output_contains_folders
```rust
#[test]
fn test_dry_run_output_contains_folders() {
    let output = dry_run_preview(&config);
    assert!(output.contains("frontend/src/components/ui/"));
    assert!(output.contains("backend/src/controllers/"));
}
```
