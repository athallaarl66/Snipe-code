use snipe_code::template::TemplateRegistry;

#[test]
fn test_load_all_returns_4_templates() {
    let templates = TemplateRegistry::load_all();
    assert_eq!(templates.len(), 4);
}

#[test]
fn test_get_company_profile() {
    let template = TemplateRegistry::get("company-profile").unwrap();
    assert_eq!(template.id, "company-profile");
    assert_eq!(template.name, "Company Profile");
}

#[test]
fn test_get_nonexistent() {
    let result = TemplateRegistry::get("nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_all_templates_have_metadata() {
    let templates = TemplateRegistry::load_all();
    for t in templates {
        assert!(!t.name.is_empty(), "name is empty for {}", t.id);
        assert!(
            !t.description.is_empty(),
            "description is empty for {}",
            t.id
        );
        assert!(!t.icon.is_empty(), "icon is empty for {}", t.id);
    }
}

#[test]
fn test_none_template_has_empty_structure() {
    let template = TemplateRegistry::get("none").unwrap();
    assert!(template.structure.folders.is_empty());
    assert!(template.structure.files.is_empty());
}
