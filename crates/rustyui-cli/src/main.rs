//! RustyUI CLI tool for project initialization and development

use clap::{Parser, Subcommand};
use rustyui_core::{DualModeConfig, config::UIFramework};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rustyui")]
#[command(about = "Revolutionary dual-mode UI development system for Rust")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize RustyUI in an existing project
    Init {
        /// UI framework to use
        #[arg(long, default_value = "egui")]
        framework: String,
        
        /// Project directory
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    
    /// Create a new RustyUI project
    New {
        /// Project name
        name: String,
        
        /// UI framework to use
        #[arg(long, default_value = "egui")]
        framework: String,
    },
    
    /// Start development mode with hot reload
    Dev {
        /// Project directory
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init { framework, path } => {
            println!("Initializing RustyUI project with {} framework at {:?}", framework, path);
            init_project(&framework, &path)?;
        }
        Commands::New { name, framework } => {
            println!("Creating new RustyUI project '{}' with {} framework", name, framework);
            create_new_project(&name, &framework)?;
        }
        Commands::Dev { path } => {
            println!("Starting development mode at {:?}", path);
            start_dev_mode(&path)?;
        }
    }
    
    Ok(())
}

/// Initialize RustyUI in an existing project
fn init_project(framework: &str, path: &PathBuf) -> anyhow::Result<()> {
    let ui_framework = parse_framework(framework)?;
    
    // Create configuration
    let config = DualModeConfig {
        framework: ui_framework,
        watch_paths: vec![path.join("src")],
        ..Default::default()
    };
    
    // Write configuration file
    let config_path = path.join("rustyui.toml");
    let config_content = toml::to_string_pretty(&config)?;
    std::fs::write(&config_path, config_content)?;
    
    println!("✓ Created rustyui.toml configuration");
    
    // TODO: Modify Cargo.toml to add RustyUI dependencies
    // TODO: Generate example hot reload code
    
    println!("✓ RustyUI initialization complete!");
    println!("  Run 'cargo run --features dev-ui' to start development mode");
    
    Ok(())
}

/// Create a new RustyUI project
fn create_new_project(name: &str, framework: &str) -> anyhow::Result<()> {
    let project_path = PathBuf::from(name);
    
    if project_path.exists() {
        return Err(anyhow::anyhow!("Directory '{}' already exists", name));
    }
    
    // Create project directory
    std::fs::create_dir_all(&project_path)?;
    
    // Initialize as Rust project
    std::process::Command::new("cargo")
        .args(&["init", "--name", name])
        .current_dir(&project_path)
        .status()?;
    
    // Initialize RustyUI
    init_project(framework, &project_path)?;
    
    println!("✓ Created new RustyUI project '{}'", name);
    
    Ok(())
}

/// Start development mode with hot reload
fn start_dev_mode(path: &PathBuf) -> anyhow::Result<()> {
    // Check if rustyui.toml exists
    let config_path = path.join("rustyui.toml");
    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "No rustyui.toml found. Run 'rustyui init' first."
        ));
    }
    
    // Load configuration
    let config_content = std::fs::read_to_string(&config_path)?;
    let _config: DualModeConfig = toml::from_str(&config_content)?;
    
    println!("🚀 Starting RustyUI development mode...");
    
    // TODO: Start dual-mode engine
    // TODO: Begin file watching
    // TODO: Initialize runtime interpreter
    
    println!("✓ Development mode active - edit your UI code for instant updates!");
    
    // For Phase 1, just run cargo with dev-ui feature
    let status = std::process::Command::new("cargo")
        .args(&["run", "--features", "dev-ui"])
        .current_dir(path)
        .status()?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("Failed to start development mode"));
    }
    
    Ok(())
}

/// Parse framework string to UIFramework enum
fn parse_framework(framework: &str) -> anyhow::Result<UIFramework> {
    match framework.to_lowercase().as_str() {
        "egui" => Ok(UIFramework::Egui),
        "iced" => Ok(UIFramework::Iced),
        "slint" => Ok(UIFramework::Slint),
        "tauri" => Ok(UIFramework::Tauri),
        _ => Err(anyhow::anyhow!("Unsupported framework: {}", framework)),
    }
}