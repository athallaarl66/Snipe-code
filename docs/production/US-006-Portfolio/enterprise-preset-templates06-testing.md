# US-006: Template Portfolio — Unit Tests

### test_portfolio_has_required_sections
```rust
#[test]
fn test_portfolio_has_required_sections() {
    let template = TemplateRegistry::get("portfolio").unwrap();
    let folders = &template.structure.folders;
    assert!(folders.iter().any(|f| f.contains("hero")));
    assert!(folders.iter().any(|f| f.contains("projects")));
    assert!(folders.iter().any(|f| f.contains("skills")));
    assert!(folders.iter().any(|f| f.contains("contact")));
}
```

### test_portfolio_has_projects_data
```rust
#[test]
fn test_portfolio_has_projects_data() {
    let template = TemplateRegistry::get("portfolio").unwrap();
    assert!(template.structure.files.iter().any(|f| f.path.contains("projects.json")));
}
```
