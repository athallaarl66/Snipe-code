use snipe_code::generator::{self, GenerateConfig};

fn default_config() -> GenerateConfig {
    GenerateConfig {
        project_name: "my-project".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "nestjs".to_string(),
        database: "none".to_string(),
    }
}

#[test]
fn test_dry_run_does_not_create_files() {
    let config = default_config();
    let output = generator::dry_run_preview(&config);

    // Output is a string, no actual files created
    assert!(output.contains("my-project/"));
    assert!(output.contains("(0 files created — dry run mode)"));
}

#[test]
fn test_dry_run_output_contains_frontend_folders() {
    let config = default_config();
    let output = generator::dry_run_preview(&config);

    assert!(output.contains("frontend/src/components/ui/"));
    assert!(output.contains("frontend/src/pages/"));
    assert!(output.contains("frontend/src/hooks/"));
}

#[test]
fn test_dry_run_output_contains_backend_folders() {
    let config = default_config();
    let output = generator::dry_run_preview(&config);

    assert!(output.contains("backend/src/controllers/"));
    assert!(output.contains("backend/src/services/"));
    assert!(output.contains("backend/src/repositories/"));
}

#[test]
fn test_dry_run_output_contains_root_files() {
    let config = default_config();
    let output = generator::dry_run_preview(&config);

    assert!(output.contains(".env.example"));
    assert!(output.contains(".gitignore"));
    assert!(output.contains("README.md"));
}

#[test]
fn test_dry_run_with_template_folders() {
    let config = GenerateConfig {
        project_name: "company-site".to_string(),
        template_id: "company-profile".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let output = generator::dry_run_preview(&config);

    assert!(output.contains("frontend/src/pages/home/"));
    assert!(output.contains("frontend/src/pages/about/"));
    assert!(output.contains("frontend/src/pages/services/"));
}

#[test]
fn test_dry_run_no_backend_when_none() {
    let config = GenerateConfig {
        project_name: "fe-only".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let output = generator::dry_run_preview(&config);

    assert!(!output.contains("backend/"));
}

#[test]
fn test_dry_run_no_frontend_when_none() {
    let config = GenerateConfig {
        project_name: "be-only".to_string(),
        template_id: "none".to_string(),
        frontend: "none".to_string(),
        backend: "nestjs".to_string(),
        database: "none".to_string(),
    };
    let output = generator::dry_run_preview(&config);

    assert!(!output.contains("frontend/"));
}

#[test]
fn test_dry_run_empty_name_returns_error() {
    let config = GenerateConfig {
        project_name: "".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let output = generator::dry_run_preview(&config);

    assert!(output.contains("Error"));
}

#[test]
fn test_dry_run_tree_structure_format() {
    let config = default_config();
    let output = generator::dry_run_preview(&config);

    // Verify tree characters present
    assert!(output.contains("├──") || output.contains("└──"));
}
