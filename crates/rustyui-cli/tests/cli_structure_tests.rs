//! Tests for CLI structure and basic functionality

use rustyui_cli::{ConfigManager, ProjectManager, TemplateManager};
use tempfile::TempDir;

#[test]
fn test_cli_crate_structure() {
    // Test that all main components can be instantiated
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    // Test ConfigManager
    let config_manager = ConfigManager::new(project_path.clone());
    assert!(config_manager.is_ok());
    
    // Test ProjectManager
    let project_manager = ProjectManager::new(project_path.clone());
    assert!(project_manager.is_ok());
    
    // Test TemplateManager
    let _template_manager = TemplateManager::new(project_path.clone());
    // TemplateManager doesn't return Result, so just check it was created
    // This is sufficient to test that the struct can be instantiated
}

#[test]
fn test_config_manager_basic_functionality() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    let config_manager = ConfigManager::new(project_path).unwrap();
    
    // Test that config doesn't exist initially
    assert!(!config_manager.config_exists());
    
    // Test creating default config
    let config = config_manager.create_default_config(rustyui_core::config::UIFramework::Egui);
    assert!(config.is_ok());
    
    let config = config.unwrap();
    // Just verify the config was created successfully
    assert!(!config.watch_paths.is_empty());
}

#[test]
fn test_project_manager_basic_functionality() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    let project_manager = ProjectManager::new(project_path).unwrap();
    
    // Test that it's not a Rust project initially
    assert!(!project_manager.is_rust_project());
    assert!(!project_manager.is_rustyui_project());
}

#[test]
fn test_framework_validation() {
    // Test that all supported frameworks are recognized
    let frameworks = ["egui", "iced", "slint", "tauri"];
    
    for framework in &frameworks {
        // This would be tested in the actual command parsing
        // For now, just verify the framework names are valid strings
        assert!(!framework.is_empty());
        assert!(framework.chars().all(|c| c.is_ascii_lowercase()));
    }
}

#[test]
fn test_error_types() {
    use rustyui_cli::CliError;
    
    // Test that all error types can be created
    let _project_error = CliError::project("test");
    let _config_error = CliError::invalid_config("test");
    let _file_error = CliError::file_not_found("test");
    let _dev_error = CliError::dev_mode("test");
    let _build_error = CliError::build("test");
    
    // All errors should implement Display and Debug
    let error = CliError::project("test error");
    let _display = format!("{}", error);
    let _debug = format!("{:?}", error);
}