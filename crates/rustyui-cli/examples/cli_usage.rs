//! Example demonstrating RustyUI CLI library usage

use rustyui_cli::{ConfigManager, ProjectManager, TemplateManager};
use rustyui_core::config::UIFramework;
use std::path::PathBuf;
use tempfile::TempDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RustyUI CLI Library Usage Example");
    println!("==================================");
    
    // Create a temporary directory for demonstration
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    
    println!("\n1. Creating project manager...");
    let project_manager = ProjectManager::new(project_path.clone())?;
    
    // Analyze the project
    println!("2. Analyzing project structure...");
    let analysis = project_manager.analyze_project()?;
    println!("   - Is Rust project: {}", analysis.is_rust_project);
    println!("   - Is RustyUI project: {}", analysis.is_rustyui_project);
    println!("   - Detected framework: {:?}", analysis.detected_framework);
    println!("   - UI directories: {:?}", analysis.ui_directories);
    
    // Create a basic Cargo.toml to make it a Rust project
    println!("\n3. Creating basic Rust project structure...");
    let cargo_toml_content = r#"[package]
name = "example-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
    std::fs::write(project_path.join("Cargo.toml"), cargo_toml_content)?;
    std::fs::create_dir_all(project_path.join("src"))?;
    std::fs::write(project_path.join("src").join("main.rs"), "fn main() { println!(\"Hello, world!\"); }")?;
    
    // Re-analyze after creating Rust project
    let analysis = project_manager.analyze_project()?;
    println!("   - Is Rust project: {}", analysis.is_rust_project);
    
    // Create configuration manager
    println!("\n4. Creating configuration manager...");
    let config_manager = ConfigManager::new(project_path.clone())?;
    println!("   - Config exists: {}", config_manager.config_exists());
    
    // Create default configuration
    println!("5. Creating default configuration for egui...");
    let config = config_manager.create_default_config(UIFramework::Egui)?;
    println!("   - Framework: {:?}", config.framework);
    println!("   - Watch paths: {:?}", config.watch_paths);
    
    // Validate and save configuration
    println!("6. Validating and saving configuration...");
    config_manager.validate_config(&config)?;
    config_manager.save_config(&config)?;
    println!("   - Config saved successfully");
    
    // Load configuration back
    println!("7. Loading configuration...");
    let loaded_config = config_manager.load_config()?;
    println!("   - Loaded framework: {:?}", loaded_config.framework);
    
    // Create template manager
    println!("\n8. Creating template manager...");
    let template_manager = TemplateManager::new(project_path.clone());
    
    // Generate example code
    println!("9. Generating egui example code...");
    template_manager.generate_example_code("egui")?;
    println!("   - Example code generated");
    
    // Generate additional files
    println!("10. Generating additional project files...");
    template_manager.generate_gitignore()?;
    template_manager.generate_readme("example-project", "egui")?;
    
    // Final analysis
    println!("\n11. Final project analysis...");
    let final_analysis = project_manager.analyze_project()?;
    println!("   - Is Rust project: {}", final_analysis.is_rust_project);
    println!("   - Is RustyUI project: {}", final_analysis.is_rustyui_project);
    println!("   - Has RustyUI config: {}", final_analysis.has_rustyui_config);
    
    println!("\n✅ CLI library demonstration completed successfully!");
    println!("📁 Project created at: {}", project_path.display());
    
    // List generated files
    println!("\n📄 Generated files:");
    for entry in std::fs::read_dir(&project_path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        println!("   - {}", file_name.to_string_lossy());
    }
    
    Ok(())
}