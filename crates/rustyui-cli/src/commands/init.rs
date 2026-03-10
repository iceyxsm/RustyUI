//! Initialize RustyUI in an existing project

use crate::config::ConfigManager;
use crate::error::{CliError, CliResult};
use crate::project::ProjectManager;
use crate::template::TemplateManager;
use console::style;
use rustyui_core::config::UIFramework;
use std::path::PathBuf;

/// Command to initialize RustyUI in an existing project
pub struct InitCommand {
    framework: String,
    path: PathBuf,
    force: bool,
    yes: bool,
}

impl InitCommand {
    /// Create a new init command
    pub fn new(framework: String, path: PathBuf, force: bool, yes: bool) -> Self {
        Self {
            framework,
            path,
            force,
            yes,
        }
    }
    
    /// Execute the init command
    pub fn execute(&mut self) -> CliResult<()> {
        println!("{} Initializing RustyUI project...", style("🚀").blue());
        
        // Analyze the project
        let project_manager = ProjectManager::new(self.path.clone())?;
        let analysis = project_manager.analyze_project()?;
        
        // Check if it's a valid Rust project
        if !analysis.is_rust_project {
            return Err(CliError::project(
                "Not a Rust project. Please run this command in a directory with Cargo.toml"
            ));
        }
        
        // Check if RustyUI is already initialized
        if analysis.is_rustyui_project && !self.force {
            return Err(CliError::project(
                "RustyUI is already initialized. Use --force to reinitialize."
            ));
        }
        
        // Validate framework
        let ui_framework = self.parse_framework(&self.framework)?;
        
        // Show analysis results
        self.show_analysis(&analysis)?;
        
        // Confirm initialization if not using --yes
        if !self.yes && !self.confirm_initialization(&analysis)? {
            println!("Initialization cancelled.");
            return Ok(());
        }
        
        // Create configuration with comprehensive dual-mode setup
        let config_manager = ConfigManager::new(self.path.clone())?;
        let config = config_manager.create_default_config(ui_framework.clone())?;
        
        // Validate and save main configuration
        config_manager.validate_config(&config)?;
        config_manager.save_config(&config)?;
        
        // Generate comprehensive hot reload configuration files
        config_manager.generate_hot_reload_config_files(&ui_framework, &analysis)?;
        
        // Add RustyUI dependencies to Cargo.toml with dual-mode support
        project_manager.add_rustyui_dependencies(&self.framework)?;
        
        // Generate example code and templates
        let template_manager = TemplateManager::new(self.path.clone());
        template_manager.generate_example_code(&self.framework)?;
        template_manager.generate_gitignore()?;
        template_manager.generate_readme(
            &self.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("rustyui-project"),
            &self.framework
        )?;
        
        // Show success message
        self.show_success_message()?;
        
        Ok(())
    }
    
    /// Parse framework string to UIFramework enum
    fn parse_framework(&self, framework: &str) -> CliResult<UIFramework> {
        match framework.to_lowercase().as_str() {
            "egui" => Ok(UIFramework::Egui),
            "iced" => Ok(UIFramework::Iced),
            "slint" => Ok(UIFramework::Slint),
            "tauri" => Ok(UIFramework::Tauri),
            _ => Err(CliError::unsupported_framework(framework)),
        }
    }
    
    /// Show project analysis results
    fn show_analysis(&self, analysis: &crate::project::ProjectAnalysis) -> CliResult<()> {
        println!("\n{}", style("📊 Detailed Project Analysis:").bold());
        
        // Basic project info
        println!("  Rust project: {}", if analysis.is_rust_project { style("✓").green() } else { style("✗").red() });
        println!("  RustyUI project: {}", if analysis.is_rustyui_project { style("✓").green() } else { style("✗").red() });
        println!("  Project type: {}", style(format!("{:?}", analysis.project_type)).cyan());
        
        // Framework detection
        if let Some(ref framework) = analysis.detected_framework {
            println!("  Detected framework: {}", style(framework).cyan());
        } else {
            println!("  Detected framework: {}", style("None").yellow());
        }
        
        // Build system info
        println!("\n{}", style("🔧 Build System:").bold());
        println!("  Cargo.toml: {}", if analysis.build_system.has_cargo_toml { style("✓").green() } else { style("✗").red() });
        println!("  build.rs: {}", if analysis.build_system.has_build_rs { style("✓").green() } else { style("✗").red() });
        
        if !analysis.build_system.cargo_features.is_empty() {
            println!("  Existing features: {}", analysis.build_system.cargo_features.join(", "));
        }
        
        if !analysis.build_system.profile_configurations.is_empty() {
            println!("  Build profiles: {}", analysis.build_system.profile_configurations.join(", "));
        }
        
        // Workspace info
        if let Some(ref workspace) = analysis.workspace_info {
            println!("\n{}", style("📦 Workspace:").bold());
            println!("  Workspace root: {}", if workspace.is_workspace_root { style("✓").green() } else { style("✗").red() });
            if !workspace.workspace_members.is_empty() {
                println!("  Members: {}", workspace.workspace_members.join(", "));
            }
        }
        
        // Dependencies
        if !analysis.existing_dependencies.is_empty() {
            println!("\n{}", style("📚 Dependencies:").bold());
            let dep_count = analysis.existing_dependencies.len().min(5);
            for dep in &analysis.existing_dependencies[..dep_count] {
                println!("  • {}", dep);
            }
            if analysis.existing_dependencies.len() > 5 {
                println!("  ... and {} more", analysis.existing_dependencies.len() - 5);
            }
        }
        
        // UI directories
        if !analysis.ui_directories.is_empty() {
            println!("\n{}", style("🎨 UI Directories:").bold());
            for dir in &analysis.ui_directories {
                println!("  • {}", dir);
            }
        }
        
        // Source files analysis
        let ui_files: Vec<_> = analysis.source_files.iter()
            .filter(|f| f.has_ui_code)
            .collect();
        
        if !ui_files.is_empty() {
            println!("\n{}", style("📄 UI Source Files:").bold());
            for file in ui_files.iter().take(3) {
                let framework_info = file.framework_usage.as_ref()
                    .map(|f| format!(" ({})", f))
                    .unwrap_or_default();
                println!("  • {}{}", file.path.display(), framework_info);
            }
            if ui_files.len() > 3 {
                println!("  ... and {} more UI files", ui_files.len() - 3);
            }
        }
        
        // Configuration files
        if !analysis.configuration_files.is_empty() {
            println!("\n{}", style("⚙️  Configuration Files:").bold());
            let important_configs: Vec<_> = analysis.configuration_files.iter()
                .filter(|f| ["Cargo.toml", "rustyui.toml", "tauri.conf.json", ".gitignore"].contains(&f.as_str()))
                .collect();
            
            for config in important_configs {
                println!("  • {}", config);
            }
        }
        
        // Recommendations
        self.show_recommendations(analysis)?;
        
        println!();
        
        Ok(())
    }
    
    /// Show recommendations based on project analysis
    fn show_recommendations(&self, analysis: &crate::project::ProjectAnalysis) -> CliResult<()> {
        println!("\n{}", style("💡 Recommendations:").bold());
        
        // Framework recommendations
        if analysis.detected_framework.is_none() {
            println!("  • Consider specifying a UI framework for optimal integration");
        }
        
        // Workspace recommendations
        if let Some(ref workspace) = analysis.workspace_info {
            if workspace.is_workspace_root && !workspace.workspace_members.is_empty() {
                println!("  • Workspace detected - RustyUI will be configured for the root crate");
                println!("  • Consider running init in individual member crates for per-crate configuration");
            }
        }
        
        // Build system recommendations
        if !analysis.build_system.has_build_rs && analysis.detected_framework.as_ref().map_or(false, |f| f == "slint") {
            println!("  • Consider adding build.rs for Slint UI compilation");
        }
        
        // Source file recommendations
        let rust_files: Vec<_> = analysis.source_files.iter()
            .filter(|f| matches!(f.file_type, crate::project::SourceFileType::Rust))
            .collect();
        
        if rust_files.is_empty() {
            println!("  • No Rust source files found - consider creating src/main.rs or src/lib.rs");
        }
        
        let ui_files: Vec<_> = analysis.source_files.iter()
            .filter(|f| f.has_ui_code)
            .collect();
        
        if ui_files.is_empty() && analysis.detected_framework.is_some() {
            println!("  • Framework detected but no UI code found - templates will be generated");
        }
        
        // Performance recommendations
        if analysis.existing_dependencies.len() > 20 {
            println!("  • Large number of dependencies detected - consider using workspace dependencies");
        }
        
        Ok(())
    }
    
    /// Confirm initialization with user
    fn confirm_initialization(&self, analysis: &crate::project::ProjectAnalysis) -> CliResult<bool> {
        use std::io::{self, Write};
        
        if analysis.is_rustyui_project {
            println!("{} RustyUI is already initialized in this project.", style("⚠").yellow());
            print!("Do you want to reinitialize? [y/N]: ");
        } else {
            print!("Initialize RustyUI with {} framework? [Y/n]: ", style(&self.framework).cyan());
        }
        
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let input = input.trim().to_lowercase();
        
        if analysis.is_rustyui_project {
            Ok(input == "y" || input == "yes")
        } else {
            Ok(input.is_empty() || input == "y" || input == "yes")
        }
    }
    
    /// Show success message
    fn show_success_message(&self) -> CliResult<()> {
        println!("\n{} RustyUI initialization complete with dual-mode support!", style("✅").green());
        
        println!("\n{}", style("📁 Generated Files:").bold());
        println!("  • rustyui.toml - Main configuration with dual-mode settings");
        println!("  • .rustyui/dev.toml - Development-specific configuration");
        println!("  • .rustyui/build.toml - Build system configuration");
        println!("  • .vscode/settings.json - IDE integration settings");
        println!("  • .vscode/launch.json - Debug configurations");
        println!("  • Updated Cargo.toml with dual-mode dependencies and features");
        
        println!("\n{}", style("🚀 Quick Start:").bold());
        println!("  1. Start development mode with instant hot reload:");
        println!("     {}", style("rustyui dev").cyan());
        println!("     or");
        println!("     {}", style("cargo run --features dev-ui").cyan());
        println!();
        println!("  2. Build for production (zero overhead):");
        println!("     {}", style("cargo build --release").cyan());
        println!();
        println!("  3. Review configuration:");
        println!("     {}", style("cat rustyui.toml").cyan());
        
        println!("\n{}", style("🔥 Dual-Mode Features:").bold());
        println!("  • {} Instant hot reload with 0ms compilation time", style("⚡").yellow());
        println!("  • {} State preservation across code changes", style("💾").blue());
        println!("  • {} Zero overhead in production builds", style("🚀").green());
        println!("  • {} Framework-agnostic architecture", style("🎯").magenta());
        println!("  • {} Runtime interpretation with multiple strategies", style("🧠").cyan());
        println!("  • {} Comprehensive error handling and recovery", style("🛡️").red());
        
        println!("\n{}", style("📚 Documentation:").bold());
        println!("  • Configuration: rustyui.toml");
        println!("  • Development settings: .rustyui/dev.toml");
        println!("  • Build configuration: .rustyui/build.toml");
        println!("  • Online docs: https://github.com/rustyui/rustyui");
        
        println!("\n{}", style("🔧 Advanced Usage:").bold());
        println!("  • Custom interpretation strategy: Edit rustyui.toml");
        println!("  • Performance monitoring: Enable in dev.toml");
        println!("  • Framework-specific settings: Check generated config files");
        println!("  • Workspace integration: Automatic detection and setup");
        
        Ok(())
    }
}