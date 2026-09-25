mod common;

use snipe_code::generator::{self, GenerateConfig};
use snipe_code::stack;

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
    assert!(folders.contains(&"src/app/"));
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
    let _ws = common::Workspace::new("gen-folders");
    let config = GenerateConfig {
        project_name: "app".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config, false).unwrap();
    assert!(result.folders.iter().any(|f| f.contains("components/ui/")));
    assert!(result.folders.iter().any(|f| f.contains("app/")));
}

#[test]
fn test_generate_creates_security_files() {
    let _ws = common::Workspace::new("gen-security");
    let config = GenerateConfig {
        project_name: "app".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config, false).unwrap();
    assert!(result.files.contains(&".env.example".to_string()));
    assert!(result.files.contains(&".gitignore".to_string()));
    assert!(result.files.contains(&"README.md".to_string()));

    // Verify files exist
    assert!(std::path::Path::new("app/.env.example").exists());
    assert!(std::path::Path::new("app/.gitignore").exists());
}

#[test]
fn test_generate_frontend_only_no_backend_folders() {
    let _ws = common::Workspace::new("gen-fe");
    let config = GenerateConfig {
        project_name: "app".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config, false).unwrap();
    assert!(!result.folders.iter().any(|f| f.starts_with("backend/")));
}

#[test]
fn test_generate_backend_only_no_frontend_folders() {
    let _ws = common::Workspace::new("gen-be");
    let config = GenerateConfig {
        project_name: "app".to_string(),
        template_id: "none".to_string(),
        frontend: "none".to_string(),
        backend: "nestjs".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config, false).unwrap();
    assert!(!result.folders.iter().any(|f| f.starts_with("frontend/")));
    assert!(result.folders.iter().any(|f| f.contains("controllers/")));
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
    let result = generator::generate(&config, false);
    assert!(result.is_err());
}

#[test]
fn test_generate_rejects_unsafe_names() {
    for bad in [
        "a/b",
        "a\\b",
        "..",
        ".",
        ".hidden",
        "con",
        "CON",
        "nul.txt",
        "my-project.",
        "my project ",
        "a:b",
        "a*b",
        "/abs",
    ] {
        let config = GenerateConfig {
            project_name: bad.to_string(),
            template_id: "none".to_string(),
            frontend: "none".to_string(),
            backend: "none".to_string(),
            database: "none".to_string(),
        };
        assert!(
            generator::generate(&config, false).is_err(),
            "name '{}' should be rejected",
            bad
        );
    }
}

#[test]
fn test_generate_accepts_safe_names() {
    for good in ["my-project", "blog", "app2", "portfolio_site"] {
        let _ws = common::Workspace::new(&format!("gen-ok-{}", good));
        let config = GenerateConfig {
            project_name: good.to_string(),
            template_id: "none".to_string(),
            frontend: "none".to_string(),
            backend: "none".to_string(),
            database: "none".to_string(),
        };
        assert!(
            generator::generate(&config, false).is_ok(),
            "name '{}' should be accepted",
            good
        );
    }
}
