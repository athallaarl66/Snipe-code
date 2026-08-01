use snipe_code::generator::{self, GenerateConfig};
use std::fs;

#[test]
fn test_folder_exists_returns_false_for_nonexistent() {
    assert!(!generator::folder_exists("nonexistent-folder-xyz"));
}

#[test]
fn test_folder_exists_returns_true_for_existing() {
    let project_name = "test-collision-exists";
    fs::create_dir_all(project_name).unwrap();
    
    assert!(generator::folder_exists(project_name));
    
    fs::remove_dir_all(project_name).unwrap();
}

#[test]
fn test_generate_overwrites_existing_folder() {
    let project_name = "test-collision-overwrite";
    
    // Create folder with a file
    fs::create_dir_all(project_name).unwrap();
    fs::write(format!("{}/old-file.txt", project_name), "old content").unwrap();
    
    // Generate should overwrite
    let config = GenerateConfig {
        project_name: project_name.to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    
    let result = generator::generate(&config).unwrap();
    
    // Old file should be gone
    assert!(!fs::exists(format!("{}/old-file.txt", project_name)).unwrap());
    
    // New files should exist
    assert!(result.files.iter().any(|f| f.contains("package.json")));
    assert!(result.files.iter().any(|f| f.contains(".gitignore")));
    
    fs::remove_dir_all(project_name).unwrap();
}

#[test]
fn test_generate_overwrites_frontend_only() {
    let project_name = "test-collision-fe";
    
    fs::create_dir_all(project_name).unwrap();
    fs::write(format!("{}/old-fe.txt", project_name), "old").unwrap();
    
    let config = GenerateConfig {
        project_name: project_name.to_string(),
        template_id: "none".to_string(),
        frontend: "vue".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    
    let result = generator::generate(&config).unwrap();
    assert!(result.folders.iter().any(|f| f.contains("components/ui/")));
    assert!(result.files.iter().any(|f| f.contains("App.vue")));
    
    fs::remove_dir_all(project_name).unwrap();
}

#[test]
fn test_generate_overwrites_backend_only() {
    let project_name = "test-collision-be";
    
    fs::create_dir_all(project_name).unwrap();
    
    let config = GenerateConfig {
        project_name: project_name.to_string(),
        template_id: "none".to_string(),
        frontend: "none".to_string(),
        backend: "go-gin".to_string(),
        database: "none".to_string(),
    };
    
    let result = generator::generate(&config).unwrap();
    assert!(result.files.iter().any(|f| f.contains("go.mod")));
    assert!(result.files.iter().any(|f| f.contains("main.go")));
    
    fs::remove_dir_all(project_name).unwrap();
}

#[test]
fn test_generate_overwrites_fullstack() {
    let project_name = "test-collision-full";
    
    fs::create_dir_all(project_name).unwrap();
    fs::write(format!("{}/old.txt", project_name), "old").unwrap();
    
    let config = GenerateConfig {
        project_name: project_name.to_string(),
        template_id: "company-profile".to_string(),
        frontend: "nextjs".to_string(),
        backend: "nestjs".to_string(),
        database: "postgresql".to_string(),
    };
    
    let result = generator::generate(&config).unwrap();
    
    // Should have both frontend and backend
    assert!(result.folders.iter().any(|f| f.starts_with("frontend/")));
    assert!(result.folders.iter().any(|f| f.starts_with("backend/")));
    assert!(result.files.iter().any(|f| f.contains("package.json")));
    
    fs::remove_dir_all(project_name).unwrap();
}

#[test]
fn test_dry_run_shows_collision_warning() {
    let project_name = "test-collision-dry";
    
    fs::create_dir_all(project_name).unwrap();
    
    let config = GenerateConfig {
        project_name: project_name.to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    
    let output = generator::dry_run_preview(&config);
    assert!(output.contains("WARNING"));
    assert!(output.contains("already exists"));
    
    fs::remove_dir_all(project_name).unwrap();
}

#[test]
fn test_dry_run_no_warning_for_new_folder() {
    let config = GenerateConfig {
        project_name: "nonexistent-folder-xyz".to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    
    let output = generator::dry_run_preview(&config);
    assert!(!output.contains("WARNING"));
}
