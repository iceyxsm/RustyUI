//! Configuration management for RustyUI CLI

use crate::error::{CliError, CliResult};
use console::style;
use rustyui_core::{DualModeConfig, config::UIFramework};
use std::path::{Path, PathBuf};

/// Configuration file name
pub const CONFIG_FILE_NAME: &str = "rustyui.toml";

/// Configuration manager for RustyUI projects
pub struct ConfigManager {
    project_path: PathBuf,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(project_path: PathBuf) -> CliResult<Self> {
        let config_path = project_path.join(CONFIG_FILE_NAME);
        
        Ok(Self {
            project_path,
            config_path,
        })
    }
    
    /// Check if configuration file exists
    pub fn config_exists(&self) -> bool {
        self.config_path.exists()
    }
    
    /// Load configuration from file
    pub fn load_config(&self) -> CliResult<DualModeConfig> {
        if !self.config_exists() {
            return Err(CliError::file_not_found(
                format!("{} not found. Run 'rustyui init' first.", CONFIG_FILE_NAME)
            ));
        }
        
        let config_content = std::fs::read_to_string(&self.config_path)?;
        let config: DualModeConfig = toml::from_str(&config_content)?;
        
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save_config(&self, config: &DualModeConfig) -> CliResult<()> {
        let config_content = toml::to_string_pretty(config)?;
        std::fs::write(&self.config_path, config_content)?;
        
        println!("{} Created {}", style("✓").green(), CONFIG_FILE_NAME);
        
        Ok(())
    }
    
    /// Create default configuration for a framework with comprehensive dual-mode settings
    pub fn create_default_config(&self, framework: UIFramework) -> CliResult<DualModeConfig> {
        let mut config = DualModeConfig::default();
        config.framework = framework.clone();
        
        // Set up watch paths based on project structure
        config.watch_paths = self.detect_watch_paths()?;
        
        // Configure framework-specific settings
        self.configure_framework_specific_settings(&mut config, &framework)?;
        
        Ok(config)
    }
    
    /// Configure framework-specific settings
    fn configure_framework_specific_settings(&self, config: &mut DualModeConfig, framework: &UIFramework) -> CliResult<()> {
        #[cfg(feature = "dev-ui")]
        {
            match framework {
                UIFramework::Egui => {
                    config.development_settings.interpretation_strategy = 
                        rustyui_core::config::InterpretationStrategy::Hybrid { 
                            rhai_threshold: 5, 
                            jit_threshold: 50 
                        };
                    config.development_settings.change_detection_delay_ms = 25; // Fast for immediate UI
                }
                UIFramework::Iced => {
                    config.development_settings.interpretation_strategy = 
                        rustyui_core::config::InterpretationStrategy::ASTOnly; // Better for Iced's architecture
                    config.development_settings.change_detection_delay_ms = 50;
                }
                UIFramework::Slint => {
                    config.development_settings.interpretation_strategy = 
                        rustyui_core::config::InterpretationStrategy::JITPreferred; // Slint benefits from JIT
                    config.development_settings.change_detection_delay_ms = 100; // Allow for compilation
                }
                UIFramework::Tauri => {
                    config.development_settings.interpretation_strategy = 
                        rustyui_core::config::InterpretationStrategy::Hybrid { 
                            rhai_threshold: 10, 
                            jit_threshold: 100 
                        };
                    config.development_settings.change_detection_delay_ms = 75; // Account for web context
                }
                UIFramework::Custom { .. } => {
                    // Use conservative defaults for custom frameworks
                    config.development_settings.interpretation_strategy = 
                        rustyui_core::config::InterpretationStrategy::Hybrid { 
                            rhai_threshold: 10, 
                            jit_threshold: 100 
                        };
                }
            }
        }
        
        // Configure production settings based on framework
        match framework {
            UIFramework::Tauri => {
                config.production_settings.binary_size_optimization = true; // Important for web distribution
                config.production_settings.security_hardening = true; // Critical for web apps
            }
            UIFramework::Slint => {
                config.production_settings.optimization_level = rustyui_core::config::OptimizationLevel::ReleaseLTO;
            }
            _ => {
                // Use defaults for other frameworks
            }
        }
        
        Ok(())
    }
    
    /// Generate comprehensive hot reload configuration files
    pub fn generate_hot_reload_config_files(&self, framework: &UIFramework, project_analysis: &crate::project::ProjectAnalysis) -> CliResult<()> {
        // Generate main rustyui.toml configuration
        self.generate_main_config_file(framework, project_analysis)?;
        
        // Generate development-specific configuration
        self.generate_development_config(framework)?;
        
        // Generate framework-specific configuration files
        self.generate_framework_config_files(framework)?;
        
        // Generate build configuration
        self.generate_build_config(project_analysis)?;
        
        // Generate IDE integration files
        self.generate_ide_integration_files()?;
        
        Ok(())
    }
    
    /// Generate main rustyui.toml configuration file
    fn generate_main_config_file(&self, framework: &UIFramework, project_analysis: &crate::project::ProjectAnalysis) -> CliResult<()> {
        let config = self.create_default_config(framework.clone())?;
        
        // Create enhanced configuration with project-specific settings
        let mut config_content = toml::to_string_pretty(&config)?;
        
        // Add project-specific comments and documentation
        let header_comment = format!(r#"# RustyUI Configuration File
# Generated for {} framework
# Project type: {:?}
# 
# This file configures dual-mode operation:
# - Development mode: Instant hot reload with runtime interpretation
# - Production mode: Zero-overhead native Rust performance
#
# For more information, visit: https://github.com/rustyui/rustyui

"#, 
            match framework {
                UIFramework::Egui => "egui",
                UIFramework::Iced => "iced", 
                UIFramework::Slint => "slint",
                UIFramework::Tauri => "tauri",
                UIFramework::Custom { name, .. } => name,
            },
            project_analysis.project_type
        );
        
        config_content = format!("{}{}", header_comment, config_content);
        
        // Add additional configuration sections
        config_content.push_str(&self.generate_advanced_config_sections(framework, project_analysis)?);
        
        std::fs::write(&self.config_path, config_content)?;
        
        println!("{} Generated comprehensive rustyui.toml configuration", style("✓").green());
        
        Ok(())
    }
    
    /// Generate advanced configuration sections
    fn generate_advanced_config_sections(&self, framework: &UIFramework, _project_analysis: &crate::project::ProjectAnalysis) -> CliResult<String> {
        let mut sections = String::new();
        
        // Hot reload specific configuration
        sections.push_str(r#"
# Hot Reload Configuration
[hot_reload]
enabled = true
auto_save = true
preserve_state = true
debounce_ms = 50

# File watching patterns
watch_patterns = [
    "**/*.rs",
    "**/*.toml",
"#);
        
        // Add framework-specific patterns
        match framework {
            UIFramework::Slint => {
                sections.push_str(r#"    "**/*.slint",
"#);
            }
            UIFramework::Tauri => {
                sections.push_str(r#"    "**/*.html",
    "**/*.css",
    "**/*.js",
    "**/*.json",
"#);
            }
            _ => {}
        }
        
        sections.push_str(r#"]

# Ignore patterns
ignore_patterns = [
    "target/**",
    "**/.git/**",
    "**/node_modules/**",
    "**/*.tmp",
    "**/*.bak",
]
"#);
        
        // Performance configuration
        sections.push_str(r#"
# Performance Configuration
[performance]
max_memory_mb = 50
interpretation_timeout_ms = 5000
jit_compilation_timeout_ms = 10000
state_preservation_timeout_ms = 1000

# Monitoring
enable_metrics = true
log_performance = false
benchmark_mode = false
"#);
        
        // Development tools integration
        sections.push_str(r#"
# Development Tools Integration
[dev_tools]
rust_analyzer_integration = true
vscode_extension = true
intellij_plugin = false

# Debugging
debug_mode = false
verbose_logging = false
trace_interpretation = false
"#);
        
        // Framework-specific configuration
        match framework {
            UIFramework::Tauri => {
                sections.push_str(r#"
# Tauri-specific Configuration
[tauri]
dev_path = "http://localhost:3000"
dist_dir = "../dist"
with_global_tauri = false
"#);
            }
            UIFramework::Slint => {
                sections.push_str(r#"
# Slint-specific Configuration
[slint]
include_paths = ["ui/", "assets/"]
library_paths = []
"#);
            }
            _ => {}
        }
        
        Ok(sections)
    }
    
    /// Generate development-specific configuration
    fn generate_development_config(&self, framework: &UIFramework) -> CliResult<()> {
        let dev_config_path = self.project_path.join(".rustyui").join("dev.toml");
        
        // Create .rustyui directory if it doesn't exist
        if let Some(parent) = dev_config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let dev_config = format!(r#"# Development-only configuration
# This file is automatically generated and can be customized

[runtime]
interpretation_engine = "hybrid"
jit_compiler = "cranelift"
script_engine = "rhai"

[state_preservation]
strategy = "json"
backup_interval_ms = 1000
max_backups = 10

[change_detection]
polling_interval_ms = 100
batch_changes = true
smart_filtering = true

[ui_framework]
name = "{}"
adapter_path = "auto"
custom_renderer = false

[logging]
level = "info"
file_path = ".rustyui/logs/dev.log"
max_file_size_mb = 10
"#, match framework {
    UIFramework::Egui => "egui",
    UIFramework::Iced => "iced",
    UIFramework::Slint => "slint", 
    UIFramework::Tauri => "tauri",
    UIFramework::Custom { name, .. } => name,
});
        
        std::fs::write(&dev_config_path, dev_config)?;
        
        println!("{} Generated development configuration", style("✓").green());
        
        Ok(())
    }
    
    /// Generate framework-specific configuration files
    fn generate_framework_config_files(&self, framework: &UIFramework) -> CliResult<()> {
        match framework {
            UIFramework::Tauri => {
                self.generate_tauri_config()?;
            }
            UIFramework::Slint => {
                self.generate_slint_config()?;
            }
            _ => {
                // No additional config files needed for egui/iced
            }
        }
        
        Ok(())
    }
    
    /// Generate Tauri-specific configuration
    fn generate_tauri_config(&self) -> CliResult<()> {
        let tauri_dir = self.project_path.join("src-tauri");
        if !tauri_dir.exists() {
            std::fs::create_dir_all(&tauri_dir)?;
        }
        
        let tauri_config_path = tauri_dir.join("tauri.conf.json");
        if !tauri_config_path.exists() {
            let tauri_config = r#"{
  "build": {
    "beforeDevCommand": "",
    "beforeBuildCommand": "",
    "devPath": "http://localhost:3000",
    "distDir": "../dist"
  },
  "package": {
    "productName": "RustyUI App",
    "version": "0.1.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      }
    },
    "bundle": {
      "active": true,
      "targets": "all",
      "identifier": "com.rustyui.app",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ]
    },
    "security": {
      "csp": null
    },
    "windows": [
      {
        "fullscreen": false,
        "resizable": true,
        "title": "RustyUI App",
        "width": 800,
        "height": 600
      }
    ]
  }
}
"#;
            std::fs::write(&tauri_config_path, tauri_config)?;
            println!("{} Generated Tauri configuration", style("✓").green());
        }
        
        Ok(())
    }
    
    /// Generate Slint-specific configuration
    fn generate_slint_config(&self) -> CliResult<()> {
        let slint_config_path = self.project_path.join("slint.toml");
        if !slint_config_path.exists() {
            let slint_config = r#"# Slint Configuration
[build]
include_paths = ["ui/", "assets/"]
library_paths = []

[hot_reload]
enabled = true
watch_paths = ["ui/", "src/"]

[compiler]
optimize = false
debug_info = true
"#;
            std::fs::write(&slint_config_path, slint_config)?;
            println!("{} Generated Slint configuration", style("✓").green());
        }
        
        Ok(())
    }
    
    /// Generate build configuration
    fn generate_build_config(&self, project_analysis: &crate::project::ProjectAnalysis) -> CliResult<()> {
        let build_config_path = self.project_path.join(".rustyui").join("build.toml");
        
        // Create .rustyui directory if it doesn't exist
        if let Some(parent) = build_config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let build_config = format!(r#"# Build Configuration for RustyUI
# Project type: {:?}

[development]
features = ["dev-ui"]
profile = "dev"
target_dir = "target/dev"

[production]
features = []
profile = "release"
target_dir = "target/release"
strip_symbols = true
optimize_size = true

[dual_mode]
conditional_compilation = true
feature_gates = ["dev-ui", "production"]
cfg_attributes = ["cfg(feature = \"dev-ui\")", "cfg(not(feature = \"dev-ui\"))"]

[workspace]
is_workspace = {}
members = [{}]
"#, 
            project_analysis.project_type,
            project_analysis.workspace_info.as_ref().map_or(false, |w| w.is_workspace_root),
            project_analysis.workspace_info.as_ref()
                .map(|w| w.workspace_members.iter().map(|m| format!("\"{}\"", m)).collect::<Vec<_>>().join(","))
                .unwrap_or_default()
        );
        
        std::fs::write(&build_config_path, build_config)?;
        
        println!("{} Generated build configuration", style("✓").green());
        
        Ok(())
    }
    
    /// Generate IDE integration files
    fn generate_ide_integration_files(&self) -> CliResult<()> {
        let vscode_dir = self.project_path.join(".vscode");
        
        // Generate VS Code settings
        if !vscode_dir.exists() {
            std::fs::create_dir_all(&vscode_dir)?;
        }
        
        let settings_path = vscode_dir.join("settings.json");
        if !settings_path.exists() {
            let settings = r#"{
    "rust-analyzer.cargo.features": ["dev-ui"],
    "rust-analyzer.checkOnSave.command": "check",
    "rust-analyzer.checkOnSave.extraArgs": ["--features", "dev-ui"],
    "files.watcherExclude": {
        "**/target/**": true,
        "**/.rustyui/logs/**": true
    },
    "files.associations": {
        "*.rustyui": "toml"
    }
}
"#;
            std::fs::write(&settings_path, settings)?;
            println!("{} Generated VS Code settings", style("✓").green());
        }
        
        // Generate launch configuration
        let launch_path = vscode_dir.join("launch.json");
        if !launch_path.exists() {
            let launch_config = r#"{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug RustyUI (Development Mode)",
            "cargo": {
                "args": [
                    "build",
                    "--features",
                    "dev-ui"
                ],
                "filter": {
                    "name": "${workspaceFolderBasename}",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        },
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug RustyUI (Production Mode)",
            "cargo": {
                "args": [
                    "build",
                    "--release"
                ],
                "filter": {
                    "name": "${workspaceFolderBasename}",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
"#;
            std::fs::write(&launch_path, launch_config)?;
            println!("{} Generated VS Code launch configuration", style("✓").green());
        }
        
        Ok(())
    }
    
    /// Detect appropriate watch paths for the project
    fn detect_watch_paths(&self) -> CliResult<Vec<PathBuf>> {
        let mut watch_paths = Vec::new();
        
        // Always watch src directory if it exists
        let src_path = self.project_path.join("src");
        if src_path.exists() {
            watch_paths.push(PathBuf::from("src"));
        }
        
        // Check for common UI directories
        let ui_dirs = ["ui", "components", "widgets", "views"];
        for dir in &ui_dirs {
            let dir_path = self.project_path.join(dir);
            if dir_path.exists() {
                watch_paths.push(PathBuf::from(dir));
            }
        }
        
        // If no directories found, default to src
        if watch_paths.is_empty() {
            watch_paths.push(PathBuf::from("src"));
        }
        
        Ok(watch_paths)
    }
    
    /// Update configuration with new settings
    pub fn update_config<F>(&self, updater: F) -> CliResult<()>
    where
        F: FnOnce(&mut DualModeConfig) -> CliResult<()>,
    {
        let mut config = self.load_config()?;
        updater(&mut config)?;
        self.save_config(&config)?;
        
        Ok(())
    }
    
    /// Validate configuration
    pub fn validate_config(&self, config: &DualModeConfig) -> CliResult<()> {
        // Validate watch paths exist
        for watch_path in &config.watch_paths {
            let full_path = self.project_path.join(watch_path);
            if !full_path.exists() {
                return Err(CliError::invalid_config(
                    format!("Watch path does not exist: {}", watch_path.display())
                ));
            }
        }
        
        // Validate framework-specific requirements
        match config.framework {
            UIFramework::Egui => self.validate_egui_config()?,
            UIFramework::Iced => self.validate_iced_config()?,
            UIFramework::Slint => self.validate_slint_config()?,
            UIFramework::Tauri => self.validate_tauri_config()?,
            UIFramework::Custom { ref name, ref adapter_path } => {
                self.validate_custom_config(name, adapter_path)?;
            }
        }
        
        Ok(())
    }
    
    /// Validate egui-specific configuration
    fn validate_egui_config(&self) -> CliResult<()> {
        // Check if egui dependencies are present in Cargo.toml
        let cargo_toml_path = self.project_path.join("Cargo.toml");
        if cargo_toml_path.exists() {
            let cargo_content = std::fs::read_to_string(&cargo_toml_path)?;
            if !cargo_content.contains("egui") {
                println!("{} Consider adding egui dependencies to Cargo.toml", style("⚠").yellow());
            }
        }
        
        Ok(())
    }
    
    /// Validate iced-specific configuration
    fn validate_iced_config(&self) -> CliResult<()> {
        let cargo_toml_path = self.project_path.join("Cargo.toml");
        if cargo_toml_path.exists() {
            let cargo_content = std::fs::read_to_string(&cargo_toml_path)?;
            if !cargo_content.contains("iced") {
                println!("{} Consider adding iced dependencies to Cargo.toml", style("⚠").yellow());
            }
        }
        
        Ok(())
    }
    
    /// Validate slint-specific configuration
    fn validate_slint_config(&self) -> CliResult<()> {
        let cargo_toml_path = self.project_path.join("Cargo.toml");
        if cargo_toml_path.exists() {
            let cargo_content = std::fs::read_to_string(&cargo_toml_path)?;
            if !cargo_content.contains("slint") {
                println!("{} Consider adding slint dependencies to Cargo.toml", style("⚠").yellow());
            }
        }
        
        Ok(())
    }
    
    /// Validate tauri-specific configuration
    fn validate_tauri_config(&self) -> CliResult<()> {
        let cargo_toml_path = self.project_path.join("Cargo.toml");
        if cargo_toml_path.exists() {
            let cargo_content = std::fs::read_to_string(&cargo_toml_path)?;
            if !cargo_content.contains("tauri") {
                println!("{} Consider adding tauri dependencies to Cargo.toml", style("⚠").yellow());
            }
        }
        
        // Check for tauri.conf.json
        let tauri_config_path = self.project_path.join("src-tauri").join("tauri.conf.json");
        if !tauri_config_path.exists() {
            println!("{} Tauri configuration file not found at src-tauri/tauri.conf.json", style("⚠").yellow());
        }
        
        Ok(())
    }
    
    /// Validate custom framework configuration
    fn validate_custom_config(&self, name: &str, adapter_path: &str) -> CliResult<()> {
        let adapter_full_path = self.project_path.join(adapter_path);
        if !adapter_full_path.exists() {
            return Err(CliError::invalid_config(
                format!("Custom framework adapter not found: {} ({})", name, adapter_path)
            ));
        }
        
        Ok(())
    }
    
    /// Get project path
    pub fn project_path(&self) -> &Path {
        &self.project_path
    }
    
    /// Get config file path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }
}