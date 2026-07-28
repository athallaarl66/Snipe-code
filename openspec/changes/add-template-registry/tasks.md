## 1. Setup Dependencies

- [x] 1.1 Add `serde` with `derive` feature to Cargo.toml
- [x] 1.2 Add `serde_json` to Cargo.toml

## 2. Data Structures

- [x] 2.1 Create `src/template.rs` with `Template` struct (id, name, description, icon, structure)
- [x] 2.2 Add `TemplateStructure` struct (folders, files, rules)
- [x] 2.3 Add `FileDef` struct (path, template_source, condition)
- [x] 2.4 Add `GenerationRules` struct (require_backend, require_database, docker_compose, security_configs)
- [x] 2.5 Add `#[derive(Deserialize)]` to all structs

## 3. Template Definitions

- [x] 3.1 Create `templates/` directory with JSON files for each template
- [x] 3.2 Create `templates/none.json` — empty structure
- [x] 3.4 Create `templates/company-profile.json`
- [x] 3.5 Create `templates/portfolio.json`
- [x] 3.6 Create `templates/blog.json`

## 4. Template Registry

- [x] 4.1 Implement `TemplateRegistry::load_all()` — embed JSON via `include_str!`, deserialize all
- [x] 4.2 Implement `TemplateRegistry::get(id: &str)` — find template by ID
- [x] 4.3 Add `TEMPLATES` constant with all 4 embedded JSON strings

## 5. Unit Tests

- [x] 5.1 Test `load_all()` returns 4 templates
- [x] 5.2 Test `get("company-profile")` returns correct template
- [x] 5.3 Test `get("nonexistent")` returns None
- [x] 5.4 Test all templates have non-empty name, description, icon
- [x] 5.5 Test none template has empty structure

## 6. Integration

- [x] 6.1 Add `mod template;` to `src/main.rs`
- [x] 6.2 Verify `cargo build` succeeds
- [x] 6.3 Verify `cargo test` passes all tests
