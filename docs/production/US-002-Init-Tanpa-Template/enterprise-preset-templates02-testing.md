# US-002: Init Tanpa Template — Unit Tests

> Status: these are proposed cases, not executed tests. The executed suite lives in
> `tests/`, and `docs/uat/UAT-MATRIX.md` maps each case to the suite that covers it.

### test_no_template_generates_clean_structure
```rust
#[test]
fn test_no_template_generates_clean_structure() {
    let result = generate_project("none", &stack_config, &project_name);
    assert!(result.folders.contains(&"frontend/src/components/ui/".to_string()));
    assert!(result.folders.contains(&"backend/src/controllers/".to_string()));
    assert!(result.folders.contains(&"backend/src/services/".to_string()));
    assert!(result.folders.contains(&"backend/src/repositories/".to_string()));
}
```

### test_no_template_no_template_specific_files
```rust
#[test]
fn test_no_template_no_template_specific_files() {
    let result = generate_project("none", &stack_config, &project_name);
    assert!(!result.files.iter().any(|f| f.contains("posts/")));
}
```

### test_no_template_still_has_security_config
```rust
#[test]
fn test_no_template_still_has_security_config() {
    let result = generate_project("none", &stack_config, &project_name);
    assert!(result.files.iter().any(|f| f == ".env.example"));
    assert!(result.files.iter().any(|f| f == ".gitignore"));
}
```
