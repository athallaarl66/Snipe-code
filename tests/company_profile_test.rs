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
fn test_company_profile_generates_nextjs_files() {
    let config = GenerateConfig {
        project_name: "test-cp-nextjs".to_string(),
        template_id: "company-profile".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config).unwrap();

    assert!(result.files.iter().any(|f| f.contains("package.json")));
    assert!(result.files.iter().any(|f| f.contains("tsconfig.json")));
    assert!(result.files.iter().any(|f| f.contains("next.config.js")));
    assert!(result.files.iter().any(|f| f.contains("src/app/layout.tsx")));
    assert!(result.files.iter().any(|f| f.contains("src/app/page.tsx")));

    let _ = std::fs::remove_dir_all("test-cp-nextjs");
}

#[test]
fn test_company_profile_generates_vue_files() {
    let config = GenerateConfig {
        project_name: "test-cp-vue".to_string(),
        template_id: "company-profile".to_string(),
        frontend: "vue".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config).unwrap();

    assert!(result.files.iter().any(|f| f.contains("package.json")));
    assert!(result.files.iter().any(|f| f.contains("vite.config.ts")));
    assert!(result.files.iter().any(|f| f.contains("src/App.vue")));
    assert!(result.files.iter().any(|f| f.contains("src/main.ts")));

    let _ = std::fs::remove_dir_all("test-cp-vue");
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

    assert!(result.folders.iter().any(|f| f.contains("header")));
    assert!(result.folders.iter().any(|f| f.contains("footer")));

    let _ = std::fs::remove_dir_all("test-cp-layout");
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

    assert!(output.contains("package.json"));
    assert!(output.contains("tsconfig.json"));
    assert!(output.contains("next.config.js"));
    assert!(output.contains("README.md"));
}
