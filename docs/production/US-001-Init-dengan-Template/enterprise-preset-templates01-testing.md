# US-001: Init dengan Template — Unit Tests

> Status: these are proposed cases, not executed tests. The executed suite lives in
> `tests/`, and `docs/uat/UAT-MATRIX.md` maps each case to the suite that covers it.

## Test Cases

### test_template_selector_loads_all_templates
```rust
#[test]
fn test_template_selector_loads_all_templates() {
    let templates = TemplateRegistry::load_all();
    assert_eq!(templates.len(), 4); // none + 3 presets
    assert!(templates.iter().any(|t| t.id == "none"));
    assert!(templates.iter().any(|t| t.id == "company-profile"));
    assert!(templates.iter().any(|t| t.id == "portfolio"));
    assert!(templates.iter().any(|t| t.id == "blog"));
}
```

### test_template_has_required_fields
```rust
#[test]
fn test_template_has_required_fields() {
    let template = TemplateRegistry::get("company-profile").unwrap();
    assert_eq!(template.name, "Company Profile");
    assert!(!template.description.is_empty());
    assert!(!template.icon.is_empty());
}
```

### test_template_structure_has_folders
```rust
#[test]
fn test_template_structure_has_folders() {
    let template = TemplateRegistry::get("portfolio").unwrap();
    let folders = template.structure.folders;
    assert!(folders.contains(&"frontend/src/components/sections/hero/".to_string()));
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
