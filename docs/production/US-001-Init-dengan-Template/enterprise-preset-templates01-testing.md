# US-001: Init dengan Template — Unit Tests

## Test Cases

### test_template_selector_loads_all_templates
```rust
#[test]
fn test_template_selector_loads_all_templates() {
    let templates = TemplateRegistry::load_all();
    assert!(templates.len() >= 5); // none + 4 templates
    assert!(templates.iter().any(|t| t.id == "none"));
    assert!(templates.iter().any(|t| t.id == "iot-dashboard"));
    assert!(templates.iter().any(|t| t.id == "company-profile"));
    assert!(templates.iter().any(|t| t.id == "portfolio"));
    assert!(templates.iter().any(|t| t.id == "blog"));
}
```

### test_template_has_required_fields
```rust
#[test]
fn test_template_has_required_fields() {
    let template = TemplateRegistry::get("iot-dashboard").unwrap();
    assert_eq!(template.name, "Industrial IoT Dashboard");
    assert!(!template.description.is_empty());
    assert!(!template.icon.is_empty());
}
```

### test_template_structure_has_folders
```rust
#[test]
fn test_template_structure_has_folders() {
    let template = TemplateRegistry::get("iot-dashboard").unwrap();
    let folders = template.structure.folders;
    assert!(folders.contains(&"frontend/src/components/charts/".to_string()));
    assert!(folders.contains(&"backend/src/websocket/".to_string()));
}
```

### test_template_none_has_no_specific_files
```rust
#[test]
fn test_template_none_has_no_specific_files() {
    let template = TemplateRegistry::get("none").unwrap();
    assert!(template.structure.folders.is_empty());
    assert!(template.structure.files.is_empty());
}
```
