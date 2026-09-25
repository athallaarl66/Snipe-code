mod common;

use snipe_code::generator::{self, GenerateConfig};
use snipe_code::template::TemplateRegistry;

#[test]
fn test_portfolio_has_required_sections() {
    let template = TemplateRegistry::get("portfolio").unwrap();
    let folders = &template.structure.folders;

    assert!(folders.iter().any(|f| f.contains("hero")));
    assert!(folders.iter().any(|f| f.contains("projects-grid")));
    assert!(folders.iter().any(|f| f.contains("skills")));
    assert!(folders.iter().any(|f| f.contains("experience")));
    assert!(folders.iter().any(|f| f.contains("contact")));
}

#[test]
fn test_portfolio_has_projects_data() {
    let template = TemplateRegistry::get("portfolio").unwrap();
    assert!(
        template
            .structure
            .files
            .iter()
            .any(|f| f.path.contains("projects.json"))
    );
}

#[test]
fn test_portfolio_backend_optional() {
    let template = TemplateRegistry::get("portfolio").unwrap();
    assert!(!template.structure.rules.require_backend);
}

#[test]
fn test_portfolio_generates_projects_json() {
    let _ws = common::Workspace::new("portfolio-json");
    let config = GenerateConfig {
        project_name: "app".to_string(),
        template_id: "portfolio".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config, false).unwrap();

    assert!(result.files.iter().any(|f| f.contains("projects.json")));

    // Verify file exists and has valid JSON
    let projects_path = "app/frontend/src/data/projects.json";
    assert!(std::path::Path::new(projects_path).exists());
    let content = std::fs::read_to_string(projects_path).unwrap();
    assert!(content.contains("projects"));
    assert!(content.contains("Project Name"));
}

#[test]
fn test_portfolio_generates_with_vue() {
    let _ws = common::Workspace::new("portfolio-vue");
    let config = GenerateConfig {
        project_name: "app".to_string(),
        template_id: "portfolio".to_string(),
        frontend: "vue".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config, false).unwrap();

    assert!(result.files.iter().any(|f| f.contains("projects.json")));
    assert!(result.files.iter().any(|f| f.contains("App.vue")));
}

#[test]
fn test_portfolio_dry_run_shows_files() {
    let config = GenerateConfig {
        project_name: "portfolio-dry".to_string(),
        template_id: "portfolio".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let output = generator::dry_run_preview(&config);

    assert!(output.contains("projects.json"));
    assert!(output.contains("hero"));
    assert!(output.contains("projects-grid"));
    assert!(output.contains("skills"));
}
