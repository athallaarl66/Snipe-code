# US-007: Template Blog/Posts — Unit Tests

> Status: these are proposed cases, not executed tests. The executed suite lives in
> `tests/`, and `docs/uat/UAT-MATRIX.md` maps each case to the suite that covers it.

### test_blog_has_content_structure
```rust
#[test]
fn test_blog_has_content_structure() {
    let template = TemplateRegistry::get("blog").unwrap();
    let folders = &template.structure.folders;
    assert!(folders.iter().any(|f| f.contains("posts")));
    assert!(folders.iter().any(|f| f.contains("categories") || f.contains("tags")));
    assert_eq!(folders.len(), 6);
}
```

### test_blog_backend_optional
```rust
#[test]
fn test_blog_backend_optional() {
    let template = TemplateRegistry::get("blog").unwrap();
    assert!(!template.rules.require_backend);
}
```
