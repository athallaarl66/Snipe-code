# US-007: Template Blog/Posts — Unit Tests

### test_blog_has_content_structure
```rust
#[test]
fn test_blog_has_content_structure() {
    let template = TemplateRegistry::get("blog").unwrap();
    let folders = &template.structure.folders;
    assert!(folders.iter().any(|f| f.contains("posts")));
    assert!(folders.iter().any(|f| f.contains("categories") || f.contains("tags")));
}
```

### test_blog_backend_optional
```rust
#[test]
fn test_blog_backend_optional() {
    let template = TemplateRegistry::get("blog").unwrap();
    assert!(!template.structure.require_backend);
}
```
