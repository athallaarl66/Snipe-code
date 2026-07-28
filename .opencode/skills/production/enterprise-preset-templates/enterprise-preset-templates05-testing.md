# US-005: Template Company Profile — Unit Tests

### test_company_profile_has_page_structure
```rust
#[test]
fn test_company_profile_has_page_structure() {
    let template = TemplateRegistry::get("company-profile").unwrap();
    let folders = &template.structure.folders;
    assert!(folders.iter().any(|f| f.contains("home") || f.contains("about")));
    assert!(folders.iter().any(|f| f.contains("services") || f.contains("contact")));
}
```

### test_company_profile_backend_optional
```rust
#[test]
fn test_company_profile_backend_optional() {
    let template = TemplateRegistry::get("company-profile").unwrap();
    assert!(!template.structure.require_backend);
}
```
