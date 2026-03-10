//! End-to-end tests for RustyUI CLI functionality

use rustyui_cli::{ConfigManager, ProjectManager, TemplateManager};
use rustyui_core::config::UIFramework;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test complete project initialization workflow
#[test]
fn test_complete_project_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    // Step 1: Create basic Rust project
    let cargo_toml_content = r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
    
    std::fs::write(project_path.join("Cargo.toml"), cargo_toml_content).unwrap();
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(project_path.join("src").join("main.rs"), "fn main() {}").unwrap();
    
    // Step 2: Initialize project manager and analyze
    let project_manager = ProjectManager::new(project_path.clone()).unwrap();
    let analysis = project_manager.analyze_project().unwrap();
    
    assert!(analysis.is_rust_project);
    assert!(!analysis.is_rustyui_project);
    
    // Step 3: Create and configure RustyUI
    let config_manager = ConfigManager::new(project_path.clone()).unwrap();
    let config = config_manager.create_default_config(UIFramework::Egui).unwrap();
    
    config_manager.validate_config(&config).unwrap();
    config_manager.save_config(&config).unwrap();
    
    // Step 4: Generate templates
    let template_manager = TemplateManager::new(project_path.clone());
    template_manager.generate_example_code("egui").unwrap();
    template_manager.generate_gitignore().unwrap();
    template_manager.generate_readme("test-project", "egui").unwrap();
    
    // Step 5: Verify final state
    let final_analysis = project_manager.analyze_project().unwrap();
    assert!(final_analysis.is_rust_project);
    assert!(final_analysis.is_rustyui_project);
    assert!(final_analysis.has_rustyui_config);
    
    // Verify files exist
    assert!(project_path.join("rustyui.toml").exists());
    assert!(project_path.join(".gitignore").exists());
    assert!(project_path.join("README.md").exists());
    assert!(project_path.join("src").join("main.rs").exists());
}

/// Test configuration management workflow
#[test]
fn test_configuration_management() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    // Create src directory for validation
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    
    let config_manager = ConfigManager::new(project_path.clone()).unwrap();
    
    // Test different frameworks
    let frameworks = [
        UIFramework::Egui,
        UIFramework::Iced,
        UIFramework::Slint,
        UIFramework::Tauri,
    ];
    
    for framework in &frameworks {
        let config = config_manager.create_default_config(framework.clone()).unwrap();
        
        // Validate configuration
        config_manager.validate_config(&config).unwrap();
        
        // Save and load configuration
        config_manager.save_config(&config).unwrap();
        let loaded_config = config_manager.load_config().unwrap();
        
        // Verify framework matches
        match (framework, &loaded_config.framework) {
            (UIFramework::Egui, UIFramework::Egui) => {},
            (UIFramework::Iced, UIFramework::Iced) => {},
            (UIFramework::Slint, UIFramework::Slint) => {},
            (UIFramework::Tauri, UIFramework::Tauri) => {},
            _ => panic!("Framework mismatch: expected {:?}, got {:?}", framework, loaded_config.framework),
        }
    }
}

/// Test template generation for all frameworks
#[test]
fn test_template_generation() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    // Test all supported frameworks
    let frameworks = ["egui", "iced", "slint", "tauri"];
    
    for framework in &frameworks {
        // Create separate directory for each framework
        let framework_path = project_path.join(framework);
        std::fs::create_dir_all(&framework_path).unwrap();
        
        // Create src directory structure first
        let src_path = if *framework == "tauri" {
            framework_path.join("src-tauri").join("src")
        } else {
            framework_path.join("src")
        };
        std::fs::create_dir_all(&src_path).unwrap();
        
        let framework_template_manager = TemplateManager::new(framework_path.clone());
        
        // Generate example code
        let result = framework_template_manager.generate_example_code(framework);
        assert!(result.is_ok(), "Failed to generate example code for {}: {:?}", framework, result.err());
        
        // Verify main.rs was created
        let main_rs_path = src_path.join("main.rs");
        assert!(main_rs_path.exists(), "main.rs not created for {} at path {:?}", framework, main_rs_path);
        
        // Verify content contains framework-specific code
        let content = std::fs::read_to_string(&main_rs_path)
            .unwrap_or_else(|e| panic!("Failed to read main.rs for {}: {}", framework, e));
        assert!(content.contains(framework), "Content doesn't contain framework name for {}", framework);
        assert!(content.contains("RustyUI"), "Content doesn't contain RustyUI for {}", framework);
        
        // Generate additional files
        framework_template_manager.generate_gitignore().unwrap();
        framework_template_manager.generate_readme(&format!("test-{}", framework), framework).unwrap();
        
        // Verify additional files
        assert!(framework_path.join(".gitignore").exists());
        assert!(framework_path.join("README.md").exists());
    }
}

/// Test error handling scenarios
#[test]
fn test_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    let config_manager = ConfigManager::new(project_path.clone()).unwrap();
    
    // Test loading non-existent config
    let result = config_manager.load_config();
    assert!(result.is_err());
    
    // Test validation with missing watch paths
    let mut config = config_manager.create_default_config(UIFramework::Egui).unwrap();
    config.watch_paths = vec![PathBuf::from("non-existent-path")];
    
    let result = config_manager.validate_config(&config);
    assert!(result.is_err());
    
    // Test template generation with invalid framework
    let template_manager = TemplateManager::new(project_path.clone());
    let result = template_manager.generate_example_code("invalid-framework");
    assert!(result.is_err());
}

/// Test project analysis with different project structures
#[test]
fn test_project_analysis_variations() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    let project_manager = ProjectManager::new(project_path.clone()).unwrap();
    
    // Test empty directory
    let analysis = project_manager.analyze_project().unwrap();
    assert!(!analysis.is_rust_project);
    assert!(!analysis.is_rustyui_project);
    assert!(analysis.detected_framework.is_none());
    
    // Test with Cargo.toml but no dependencies
    let cargo_toml_content = r#"[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#;
    std::fs::write(project_path.join("Cargo.toml"), cargo_toml_content).unwrap();
    
    let analysis = project_manager.analyze_project().unwrap();
    assert!(analysis.is_rust_project);
    assert!(analysis.detected_framework.is_none());
    
    // Test with egui dependency
    let cargo_toml_with_egui = r#"[package]
name = "test"
version = "0.1.0"
edition = "2021"

[dependencies]
egui = "0.24"
"#;
    std::fs::write(project_path.join("Cargo.toml"), cargo_toml_with_egui).unwrap();
    
    let analysis = project_manager.analyze_project().unwrap();
    assert!(analysis.is_rust_project);
    assert_eq!(analysis.detected_framework, Some("egui".to_string()));
    
    // Test with RustyUI config
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    let config_manager = ConfigManager::new(project_path.clone()).unwrap();
    let config = config_manager.create_default_config(UIFramework::Egui).unwrap();
    config_manager.save_config(&config).unwrap();
    
    let analysis = project_manager.analyze_project().unwrap();
    assert!(analysis.is_rust_project);
    assert!(analysis.is_rustyui_project);
    assert!(analysis.has_rustyui_config);
}