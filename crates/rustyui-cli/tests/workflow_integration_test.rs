//! Integration tests for workflow management

use rustyui_cli::{CliResult, WorkflowManager};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_workflow_manager_creation() -> CliResult<()> {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    // Create a basic Cargo.toml to make it a Rust project
    std::fs::write(
        project_path.join("Cargo.toml"),
        r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
rustyui-core = { path = "../../../crates/rustyui-core", features = ["dev-ui"] }
"#,
    ).unwrap();
    
    // Create src directory
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(
        project_path.join("src/main.rs"),
        "fn main() { println!(\"Hello, world!\"); }",
    ).unwrap();
    
    // Create rustyui.toml to make it a RustyUI project
    std::fs::write(
        project_path.join("rustyui.toml"),
        r#"
[framework]
type = "egui"

[development]
interpretation_strategy = "hybrid"
state_preservation = true
performance_monitoring = true

[production]
strip_dev_features = true
optimization_level = "release"
"#,
    ).unwrap();
    
    // Test workflow manager creation
    let workflow_manager = WorkflowManager::new(project_path);
    assert!(workflow_manager.is_ok(), "Failed to create workflow manager: {:?}", workflow_manager.err());
    
    Ok(())
}

#[test]
fn test_development_mode_detection() -> CliResult<()> {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    // Create minimal project structure
    create_test_project(&project_path)?;
    
    let workflow_manager = WorkflowManager::new(project_path)?;
    let current_mode = workflow_manager.get_current_mode()?;
    
    // Should detect development mode when dev-ui feature is enabled
    #[cfg(feature = "dev-ui")]
    {
        use rustyui_cli::workflow::DevelopmentMode;
        assert_eq!(current_mode, DevelopmentMode::Development);
    }
    
    #[cfg(not(feature = "dev-ui"))]
    {
        use rustyui_cli::workflow::DevelopmentMode;
        assert_eq!(current_mode, DevelopmentMode::Production);
    }
    
    Ok(())
}

#[test]
fn test_workflow_status_display() -> CliResult<()> {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    create_test_project(&project_path)?;
    
    let workflow_manager = WorkflowManager::new(project_path)?;
    
    // This should not panic and should complete successfully
    let result = workflow_manager.show_workflow_status();
    assert!(result.is_ok(), "Failed to show workflow status: {:?}", result.err());
    
    Ok(())
}

#[cfg(feature = "dev-ui")]
#[test]
fn test_auto_configuration() -> CliResult<()> {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    create_test_project(&project_path)?;
    
    let mut workflow_manager = WorkflowManager::new(project_path)?;
    
    // Test auto-configuration
    let result = workflow_manager.auto_configure_runtime();
    assert!(result.is_ok(), "Failed to auto-configure runtime: {:?}", result.err());
    
    let config = result.unwrap();
    
    // Verify configuration has been optimized
    assert!(!config.watch_paths.is_empty(), "Watch paths should not be empty");
    assert!(config.development_settings.state_preservation, "State preservation should be enabled");
    
    Ok(())
}

#[test]
fn test_cargo_integration() -> CliResult<()> {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    create_test_project(&project_path)?;
    
    let workflow_manager = WorkflowManager::new(project_path)?;
    
    // Test cargo command integration (without actually executing cargo)
    // This tests the parsing and mode detection logic
    let args = vec!["--features".to_string(), "dev-ui".to_string()];
    
    // This should not panic and should handle the integration logic
    // Note: We're not actually executing cargo commands in tests
    let result = std::panic::catch_unwind(|| {
        // Just test that the method exists and can be called
        // In a real test environment, we'd mock the cargo execution
        workflow_manager.integrate_cargo_workflow("run", &args)
    });
    
    assert!(result.is_ok(), "Cargo integration should not panic");
    
    Ok(())
}

/// Helper function to create a test project structure
fn create_test_project(project_path: &PathBuf) -> CliResult<()> {
    // Create Cargo.toml
    std::fs::write(
        project_path.join("Cargo.toml"),
        r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
rustyui-core = { path = "../../../crates/rustyui-core", features = ["dev-ui"] }

[features]
dev-ui = ["rustyui-core/dev-ui"]
"#,
    ).map_err(|e| rustyui_cli::CliError::project(format!("Failed to create Cargo.toml: {}", e)))?;
    
    // Create src directory and main.rs
    std::fs::create_dir_all(project_path.join("src"))
        .map_err(|e| rustyui_cli::CliError::project(format!("Failed to create src directory: {}", e)))?;
    
    std::fs::write(
        project_path.join("src/main.rs"),
        r#"
use rustyui_core::{DualModeEngine, DualModeConfig, UIFramework};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DualModeConfig {
        framework: UIFramework::Egui,
        watch_paths: vec![std::path::PathBuf::from("src")],
        production_settings: Default::default(),
        conditional_compilation: Default::default(),
        #[cfg(feature = "dev-ui")]
        development_settings: Default::default(),
    };
    
    let mut engine = DualModeEngine::new(config)?;
    engine.initialize()?;
    
    println!("RustyUI test application running!");
    
    Ok(())
}
"#,
    ).map_err(|e| rustyui_cli::CliError::project(format!("Failed to create main.rs: {}", e)))?;
    
    // Create rustyui.toml
    std::fs::write(
        project_path.join("rustyui.toml"),
        r#"
[framework]
type = "egui"

[development]
interpretation_strategy = "hybrid"
jit_compilation_threshold = 100
state_preservation = true
performance_monitoring = true
change_detection_delay_ms = 50
max_memory_overhead_mb = 50

[production]
strip_dev_features = true
optimization_level = "release"
binary_size_optimization = true
security_hardening = true

[conditional_compilation]
dev_feature_flag = "dev-ui"
cfg_attributes = ["feature = \"dev-ui\""]

[[watch_paths]]
path = "src"

[[watch_paths]]
path = "examples"
"#,
    ).map_err(|e| rustyui_cli::CliError::project(format!("Failed to create rustyui.toml: {}", e)))?;
    
    Ok(())
}