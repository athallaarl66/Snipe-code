# US-002: Init Tanpa Template — Unit Tests

### test_no_template_generates_clean_structure
```rust
#[test]
fn test_no_template_generates_clean_structure() {
    let result = generate_project("none", &stack_config, &project_name);
    assert!(result.folders.contains(&"frontend/src/components/".to_string()));
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
    assert!(!result.files.iter().any(|f| f.path.contains("dashboard/")));
    assert!(!result.files.iter().any(|f| f.path.contains("blog/")));
}
```

### test_no_template_still_has_security_config
```rust
#[test]
fn test_no_template_still_has_security_config() {
    let result = generate_project("none", &stack_config, &project_name);
    assert!(result.files.iter().any(|f| f.path == ".env.example"));
    assert!(result.files.iter().any(|f| f.path == ".gitignore"));
}
```
