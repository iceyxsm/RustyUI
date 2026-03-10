//! Integration tests for RustyUI CLI

use rustyui_cli::{ConfigManager, ProjectManager, TemplateManager};
use std::path::PathBuf;
use tempfile::TempDir;

/// Test configuration manager creation and basic operations
#[test]
fn test_config_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    let config_manager = ConfigManager::new(project_path).unwrap();
    
    // Should not have config initially
    assert!(!config_manager.config_exists());
}

/// Test project manager creation and analysis
#[test]
fn test_project_manager_analysis() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    let project_manager = ProjectManager::new(project_path).unwrap();
    let analysis = project_manager.analyze_project().unwrap();
    
    // Should not be a Rust project initially
    assert!(!analysis.is_rust_project);
    assert!(!analysis.is_rustyui_project);
    assert!(analysis.detected_framework.is_none());
}

/// Test template manager creation
#[test]
fn test_template_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    let _template_manager = TemplateManager::new(project_path);
    // Template manager should always be creatable
}

/// Test project validation
#[test]
fn test_project_validation() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    // Create a basic Cargo.toml to make it a Rust project
    let cargo_toml_content = r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
    
    std::fs::write(project_path.join("Cargo.toml"), cargo_toml_content).unwrap();
    
    let project_manager = ProjectManager::new(project_path).unwrap();
    let analysis = project_manager.analyze_project().unwrap();
    
    // Should now be a Rust project
    assert!(analysis.is_rust_project);
    assert!(!analysis.is_rustyui_project);
}

/// Test configuration creation and validation
#[test]
fn test_config_creation_and_validation() {
    use rustyui_core::config::UIFramework;
    
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    // Create src directory
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    
    let config_manager = ConfigManager::new(project_path).unwrap();
    let config = config_manager.create_default_config(UIFramework::Egui).unwrap();
    
    // Validate the configuration
    config_manager.validate_config(&config).unwrap();
    
    // Save and load configuration
    config_manager.save_config(&config).unwrap();
    assert!(config_manager.config_exists());
    
    let loaded_config = config_manager.load_config().unwrap();
    assert!(matches!(loaded_config.framework, UIFramework::Egui));
}

/// Test error handling
#[test]
fn test_error_handling() {
    use rustyui_cli::CliError;
    
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    let config_manager = ConfigManager::new(project_path).unwrap();
    
    // Should fail to load non-existent config
    let result = config_manager.load_config();
    assert!(result.is_err());
    
    match result.unwrap_err() {
        CliError::FileNotFound(_) => {
            // Expected error type
        }
        _ => panic!("Expected FileNotFound error"),
    }
}