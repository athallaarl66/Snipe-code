use snipe_code::template::TemplateRegistry;
use snipe_code::generator::{self, GenerateConfig};

#[test]
fn test_company_profile_has_page_folders() {
    let template = TemplateRegistry::get("company-profile").unwrap();
    let folders = &template.structure.folders;

    assert!(folders.iter().any(|f| f.contains("home")));
    assert!(folders.iter().any(|f| f.contains("about")));
    assert!(folders.iter().any(|f| f.contains("services")));
    assert!(folders.iter().any(|f| f.contains("contact")));
    assert!(folders.iter().any(|f| f.contains("team")));
}

#[test]
fn test_company_profile_has_layout_folders() {
    let template = TemplateRegistry::get("company-profile").unwrap();
    let folders = &template.structure.folders;

    assert!(folders.iter().any(|f| f.contains("header")));
    assert!(folders.iter().any(|f| f.contains("footer")));
}

#[test]
fn test_company_profile_backend_optional() {
    let template = TemplateRegistry::get("company-profile").unwrap();
    assert!(!template.structure.rules.require_backend);
}

#[test]
fn test_company_profile_generates_html_files() {
    let config = GenerateConfig {
        project_name: "test-cp-files".to_string(),
        template_id: "company-profile".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config).unwrap();

    assert!(result.files.iter().any(|f| f.contains("home/index.html")));
    assert!(result.files.iter().any(|f| f.contains("about/index.html")));
    assert!(result.files.iter().any(|f| f.contains("services/index.html")));
    assert!(result.files.iter().any(|f| f.contains("contact/index.html")));
    assert!(result.files.iter().any(|f| f.contains("team/index.html")));

    let _ = std::fs::remove_dir_all("test-cp-files");
}

#[test]
fn test_company_profile_generates_layout_components() {
    let config = GenerateConfig {
        project_name: "test-cp-layout".to_string(),
        template_id: "company-profile".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config).unwrap();

    assert!(result.files.iter().any(|f| f.contains("Header.tsx")));
    assert!(result.files.iter().any(|f| f.contains("Footer.tsx")));

    let _ = std::fs::remove_dir_all("test-cp-layout");
}

#[test]
fn test_company_profile_html_has_seo_tags() {
    let config = GenerateConfig {
        project_name: "test-cp-seo".to_string(),
        template_id: "company-profile".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let _ = generator::generate(&config).unwrap();

    let home_html = std::fs::read_to_string("test-cp-seo/frontend/src/pages/home/index.html").unwrap();
    assert!(home_html.contains("<title>"));
    assert!(home_html.contains("<meta name=\"description\""));
    assert!(home_html.contains("og:title"));
    assert!(home_html.contains("application/ld+json"));

    let _ = std::fs::remove_dir_all("test-cp-seo");
}

#[test]
fn test_company_profile_dry_run_shows_files() {
    let config = GenerateConfig {
        project_name: "cp-dry".to_string(),
        template_id: "company-profile".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let output = generator::dry_run_preview(&config);

    assert!(output.contains("frontend/src/pages/home/index.html"));
    assert!(output.contains("frontend/src/pages/about/index.html"));
    assert!(output.contains("Header.tsx"));
    assert!(output.contains("Footer.tsx"));
}
