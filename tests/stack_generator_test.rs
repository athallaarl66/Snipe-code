use snipe_code::stack;
use snipe_code::generator::{self, GenerateConfig};

#[test]
fn test_frontend_stacks_count() {
    assert_eq!(stack::FRONTEND_STACKS.len(), 5);
}

#[test]
fn test_backend_stacks_count() {
    assert_eq!(stack::BACKEND_STACKS.len(), 6);
}

#[test]
fn test_database_stacks_count() {
    assert_eq!(stack::DATABASE_STACKS.len(), 3);
}

#[test]
fn test_get_frontend_folders_nextjs() {
    let folders = stack::get_frontend_folders("nextjs").unwrap();
    assert!(folders.contains(&"src/components/ui/"));
    assert!(folders.contains(&"src/pages/"));
}

#[test]
fn test_get_backend_folders_nestjs() {
    let folders = stack::get_backend_folders("nestjs").unwrap();
    assert!(folders.contains(&"src/controllers/"));
    assert!(folders.contains(&"src/services/"));
}

#[test]
fn test_validate_selection_both_none() {
    let result = stack::validate_selection("none", "none");
    assert!(result.is_err());
}

#[test]
fn test_validate_selection_frontend_only() {
    let result = stack::validate_selection("nextjs", "none");
    assert!(result.is_ok());
}

#[test]
fn test_validate_selection_backend_only() {
    let result = stack::validate_selection("none", "nestjs");
    assert!(result.is_ok());
}

#[test]
fn test_generate_creates_folders() {
    let config = GenerateConfig {
        project_name: "test-project-gen".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config).unwrap();
    assert!(result.folders.iter().any(|f| f.contains("components/ui/")));
    assert!(result.folders.iter().any(|f| f.contains("pages/")));

    // Cleanup
    let _ = std::fs::remove_dir_all("test-project-gen");
}

#[test]
fn test_generate_creates_security_files() {
    let config = GenerateConfig {
        project_name: "test-project-security".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config).unwrap();
    assert!(result.files.contains(&".env.example".to_string()));
    assert!(result.files.contains(&".gitignore".to_string()));
    assert!(result.files.contains(&"README.md".to_string()));

    // Verify files exist
    assert!(std::path::Path::new("test-project-security/.env.example").exists());
    assert!(std::path::Path::new("test-project-security/.gitignore").exists());

    // Cleanup
    let _ = std::fs::remove_dir_all("test-project-security");
}

#[test]
fn test_generate_frontend_only_no_backend_folders() {
    let config = GenerateConfig {
        project_name: "test-project-fe".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config).unwrap();
    assert!(!result.folders.iter().any(|f| f.starts_with("backend/")));

    // Cleanup
    let _ = std::fs::remove_dir_all("test-project-fe");
}

#[test]
fn test_generate_backend_only_no_frontend_folders() {
    let config = GenerateConfig {
        project_name: "test-project-be".to_string(),
        template_id: "none".to_string(),
        frontend: "none".to_string(),
        backend: "nestjs".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config).unwrap();
    assert!(!result.folders.iter().any(|f| f.starts_with("frontend/")));
    assert!(result.folders.iter().any(|f| f.contains("controllers/")));

    // Cleanup
    let _ = std::fs::remove_dir_all("test-project-be");
}

#[test]
fn test_generate_empty_name_fails() {
    let config = GenerateConfig {
        project_name: "".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config);
    assert!(result.is_err());
}
