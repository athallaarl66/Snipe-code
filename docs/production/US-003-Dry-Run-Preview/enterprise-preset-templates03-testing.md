# US-003: Dry-Run Preview — Unit Tests

### test_dry_run_does_not_create_files
```rust
#[test]
fn test_dry_run_does_not_create_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = ProjectConfig::new("test-project", "none", stack_config);
    dry_run(&config, temp_dir.path());
    assert!(temp_dir.path().read_dir().unwrap().count() == 0);
}
```

### test_dry_run_output_contains_folders
```rust
#[test]
fn test_dry_run_output_contains_folders() {
    let output = dry_run_preview("none", &stack_config);
    assert!(output.contains("frontend/src/components/"));
    assert!(output.contains("backend/src/controllers/"));
}
```
