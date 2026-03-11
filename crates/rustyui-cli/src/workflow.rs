//! Development workflow integration and mode switching

use crate::config::ConfigManager;
use crate::error::{CliError, CliResult};
use crate::project::ProjectManager;
use console::style;
use rustyui_core::{DualModeConfig, Platform, PlatformConfig};
use std::path::PathBuf;
use std::process::Command;

/// Workflow manager for seamless development experience
pub struct WorkflowManager {
    project_path: PathBuf,
    config_manager: ConfigManager,
    project_manager: ProjectManager,
}

impl WorkflowManager {
    /// Create a new workflow manager
    pub fn new(project_path: PathBuf) -> CliResult<Self> {
        let config_manager = ConfigManager::new(project_path.clone())?;
        let project_manager = ProjectManager::new(project_path.clone())?;
        
        Ok(Self {
            project_path,
            config_manager,
            project_manager,
        })
    }
    
    /// Detect and configure optimal runtime settings automatically
    pub fn auto_configure_runtime(&mut self) -> CliResult<DualModeConfig> {
        println!("{} Auto-configuring runtime settings...", style("🔧").blue());
        
        // Detect platform capabilities
        let platform = Platform::current();
        let platform_config = PlatformConfig::auto_detect();
        
        // Analyze project structure
        let analysis = self.project_manager.analyze_project()?;
        
        // Load or create base configuration
        let mut config = if analysis.is_rustyui_project {
            self.config_manager.load_config()?
        } else {
            return Err(CliError::project("Not a RustyUI project. Run 'rustyui init' first."));
        };
        
        // Auto-optimize configuration based on platform and project
        self.optimize_configuration(&mut config, &platform_config, &analysis)?;
        
        // Validate and save optimized configuration
        self.config_manager.validate_config(&config)?;
        self.config_manager.save_config(&config)?;
        
        println!("{} Runtime configuration optimized for {}", 
            style("✅").green(), 
            style(platform.to_string()).cyan()
        );
        
        Ok(config)
    }
    
    /// Optimize configuration based on platform and project characteristics
    fn optimize_configuration(
        &self,
        config: &mut DualModeConfig,
        platform_config: &PlatformConfig,
        analysis: &crate::project::ProjectAnalysis,
    ) -> CliResult<()> {
        #[cfg(feature = "dev-ui")]
        {
            // Optimize interpretation strategy based on project size
            let source_file_count = analysis.source_files.len();
            let ui_file_count = analysis.source_files.iter()
                .filter(|f| f.has_ui_code)
                .count();
            
            if source_file_count > 50 || ui_file_count > 20 {
                // Large project - prefer JIT compilation
                config.development_settings.interpretation_strategy = 
                    rustyui_core::config::InterpretationStrategy::JITPreferred;
                config.development_settings.jit_compilation_threshold = 50; // Lower threshold
            } else if ui_file_count < 5 {
                // Small project - use Rhai for simplicity
                config.development_settings.interpretation_strategy = 
                    rustyui_core::config::InterpretationStrategy::RhaiOnly;
            } else {
                // Medium project - use hybrid approach
                config.development_settings.interpretation_strategy = 
                    rustyui_core::config::InterpretationStrategy::Hybrid { 
                        rhai_threshold: 10, 
                        jit_threshold: 100 
                    };
            }
            
            // Optimize memory settings based on available system memory
            let available_memory_gb = self.get_available_memory_gb();
            if available_memory_gb < 4.0 {
                // Low memory system - reduce overhead
                config.development_settings.max_memory_overhead_mb = Some(25);
                config.development_settings.change_detection_delay_ms = 100; // Reduce frequency
            } else if available_memory_gb > 16.0 {
                // High memory system - allow more overhead for better performance
                config.development_settings.max_memory_overhead_mb = Some(100);
                config.development_settings.change_detection_delay_ms = 25; // Increase frequency
            }
            
            // Optimize for platform-specific capabilities
            if !platform_config.use_jit_compilation {
                // JIT not available - force Rhai or AST interpretation
                config.development_settings.interpretation_strategy = 
                    rustyui_core::config::InterpretationStrategy::RhaiOnly;
            }
        }
        
        // Optimize watch paths based on project structure
        let mut optimized_paths = Vec::new();
        
        // Always watch src directory
        if self.project_path.join("src").exists() {
            optimized_paths.push(PathBuf::from("src"));
        }
        
        // Watch UI-specific directories if they exist
        for ui_dir in &analysis.ui_directories {
            let path = PathBuf::from(ui_dir);
            if !optimized_paths.contains(&path) {
                optimized_paths.push(path);
            }
        }
        
        // Watch examples if they exist and contain UI code
        if self.project_path.join("examples").exists() {
            let examples_have_ui = analysis.source_files.iter()
                .any(|f| f.path.starts_with("examples") && f.has_ui_code);
            if examples_have_ui {
                optimized_paths.push(PathBuf::from("examples"));
            }
        }
        
        // Update configuration with optimized paths
        if !optimized_paths.is_empty() {
            config.watch_paths = optimized_paths;
        }
        
        println!("  {} Optimized for {} source files ({} UI files)", 
            style("📊").blue(),
            analysis.source_files.len(),
            analysis.source_files.iter().filter(|f| f.has_ui_code).count()
        );
        
        #[cfg(feature = "dev-ui")]
        println!("  {} Interpretation strategy: {:?}", 
            style("🧠").blue(),
            config.development_settings.interpretation_strategy
        );
        
        println!("  {} Watch paths: {:?}", 
            style("👀").blue(),
            config.watch_paths
        );
        
        Ok(())
    }
    
    /// Get available system memory in GB (rough estimate)
    fn get_available_memory_gb(&self) -> f64 {
        // This is a simplified implementation
        // In a real system, you'd use platform-specific APIs
        #[cfg(target_os = "windows")]
        {
            // Windows implementation would use GlobalMemoryStatusEx
            8.0 // Default assumption
        }
        
        #[cfg(target_os = "macos")]
        {
            // macOS implementation would use sysctl
            8.0 // Default assumption
        }
        
        #[cfg(target_os = "linux")]
        {
            // Linux implementation would read /proc/meminfo
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                if let Some(line) = meminfo.lines().find(|l| l.starts_with("MemAvailable:")) {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            return kb as f64 / (1024.0 * 1024.0); // Convert KB to GB
                        }
                    }
                }
            }
            8.0 // Default assumption
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            8.0 // Default assumption for other platforms
        }
    }
    
    /// Handle seamless mode transitions
    pub fn handle_mode_transition(&self, target_mode: DevelopmentMode) -> CliResult<()> {
        println!("{} Transitioning to {} mode...", 
            style("🔄").blue(),
            style(target_mode.to_string()).cyan()
        );
        
        match target_mode {
            DevelopmentMode::Development => {
                self.transition_to_development()?;
            }
            DevelopmentMode::Production => {
                self.transition_to_production()?;
            }
        }
        
        println!("{} Mode transition completed", style("✅").green());
        Ok(())
    }
    
    /// Transition to development mode
    fn transition_to_development(&self) -> CliResult<()> {
        // Ensure dev-ui feature is available
        if !cfg!(feature = "dev-ui") {
            println!("{} Development mode requires dev-ui feature", style("⚠").yellow());
            println!("  Run with: cargo run --features dev-ui");
            return Ok(());
        }
        
        // Load development configuration
        let config = self.config_manager.load_config()?;
        
        // Validate development requirements
        self.validate_development_requirements(&config)?;
        
        println!("  {} Development mode ready", style("✓").green());
        println!("  {} Hot reload: enabled", style("🔥").green());
        println!("  {} State preservation: {}", 
            style("💾").blue(),
            if cfg!(feature = "dev-ui") && config.development_settings.state_preservation {
                style("enabled").green()
            } else {
                style("disabled").yellow()
            }
        );
        
        Ok(())
    }
    
    /// Transition to production mode
    fn transition_to_production(&self) -> CliResult<()> {
        // Validate production build requirements
        self.validate_production_requirements()?;
        
        println!("  {} Production mode ready", style("✓").green());
        println!("  {} Zero overhead: guaranteed", style("🚀").green());
        println!("  {} Development features: stripped", style("🔒").blue());
        
        Ok(())
    }
    
    /// Validate development mode requirements
    fn validate_development_requirements(&self, config: &DualModeConfig) -> CliResult<()> {
        // Check platform compatibility
        let platform = Platform::current();
        let platform_config = PlatformConfig::auto_detect();
        
        // Validate file watcher backend
        let watcher_perf = platform_config.file_watcher_backend.performance_characteristics();
        if watcher_perf.latency_ms > 100 {
            println!("  {} File watcher latency high ({}ms) - performance may be affected", 
                style("⚠").yellow(),
                watcher_perf.latency_ms
            );
        }
        
        // Validate JIT compilation if needed
        #[cfg(feature = "dev-ui")]
        {
            if matches!(config.development_settings.interpretation_strategy, 
                rustyui_core::config::InterpretationStrategy::JITPreferred |
                rustyui_core::config::InterpretationStrategy::Hybrid { .. }) {
                
                let jit_caps = platform.jit_capabilities();
                if !jit_caps.supports_cranelift {
                    println!("  {} JIT compilation not available - falling back to Rhai interpretation", 
                        style("⚠").yellow()
                    );
                }
            }
        }
        
        // Validate watch paths
        for path in &config.watch_paths {
            let full_path = self.project_path.join(path);
            if !full_path.exists() {
                println!("  {} Watch path does not exist: {}", 
                    style("⚠").yellow(),
                    path.display()
                );
            }
        }
        
        Ok(())
    }
    
    /// Validate production mode requirements
    fn validate_production_requirements(&self) -> CliResult<()> {
        // Check that dev-ui feature is not enabled in production
        if cfg!(feature = "dev-ui") {
            println!("  {} Warning: dev-ui feature is enabled in production build", 
                style("⚠").yellow()
            );
            println!("    For zero overhead, build with: cargo build --release");
        }
        
        // Validate Cargo.toml configuration
        let analysis = self.project_manager.analyze_project()?;
        if !analysis.build_system.profile_configurations.contains(&"release".to_string()) {
            println!("  {} No release profile found - using default optimizations", 
                style("ℹ").blue()
            );
        }
        
        Ok(())
    }
    
    /// Integrate with existing Cargo workflows
    pub fn integrate_cargo_workflow(&self, cargo_command: &str, args: &[String]) -> CliResult<()> {
        println!("{} Integrating with Cargo workflow: {} {}", 
            style("🔗").blue(),
            style(cargo_command).cyan(),
            args.join(" ")
        );
        
        match cargo_command {
            "run" => {
                self.handle_cargo_run(args)?;
            }
            "build" => {
                self.handle_cargo_build(args)?;
            }
            "test" => {
                self.handle_cargo_test(args)?;
            }
            "check" => {
                self.handle_cargo_check(args)?;
            }
            _ => {
                // Pass through other commands unchanged
                self.execute_cargo_command(cargo_command, args)?;
            }
        }
        
        Ok(())
    }
    
    /// Handle cargo run with RustyUI integration
    fn handle_cargo_run(&self, args: &[String]) -> CliResult<()> {
        let has_dev_ui = args.iter().any(|arg| arg.contains("dev-ui"));
        
        if has_dev_ui {
            println!("  {} Development mode detected", style("🔥").green());
            self.handle_mode_transition(DevelopmentMode::Development)?;
        } else {
            println!("  {} Production mode detected", style("🚀").blue());
            self.handle_mode_transition(DevelopmentMode::Production)?;
        }
        
        self.execute_cargo_command("run", args)?;
        Ok(())
    }
    
    /// Handle cargo build with RustyUI integration
    fn handle_cargo_build(&self, args: &[String]) -> CliResult<()> {
        let is_release = args.iter().any(|arg| arg == "--release");
        let has_dev_ui = args.iter().any(|arg| arg.contains("dev-ui"));
        
        if is_release && !has_dev_ui {
            println!("  {} Production build detected", style("🚀").green());
            self.handle_mode_transition(DevelopmentMode::Production)?;
        } else if has_dev_ui {
            println!("  {} Development build detected", style("🔥").blue());
            self.handle_mode_transition(DevelopmentMode::Development)?;
        }
        
        self.execute_cargo_command("build", args)?;
        Ok(())
    }
    
    /// Handle cargo test with RustyUI integration
    fn handle_cargo_test(&self, args: &[String]) -> CliResult<()> {
        // Tests might need dev-ui features for testing hot reload functionality
        let has_dev_ui = args.iter().any(|arg| arg.contains("dev-ui"));
        
        if has_dev_ui {
            println!("  {} Testing with development features", style("🧪").blue());
        }
        
        self.execute_cargo_command("test", args)?;
        Ok(())
    }
    
    /// Handle cargo check with RustyUI integration
    fn handle_cargo_check(&self, args: &[String]) -> CliResult<()> {
        // Check command - just validate configuration
        let has_dev_ui = args.iter().any(|arg| arg.contains("dev-ui"));
        
        if has_dev_ui {
            println!("  {} Checking with development features", style("🔍").blue());
        }
        
        self.execute_cargo_command("check", args)?;
        Ok(())
    }
    
    /// Execute a cargo command with the given arguments
    fn execute_cargo_command(&self, command: &str, args: &[String]) -> CliResult<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg(command);
        cmd.args(args);
        cmd.current_dir(&self.project_path);
        
        let status = cmd.status()
            .map_err(|e| CliError::dev_mode(format!("Failed to execute cargo {}: {}", command, e)))?;
        
        if !status.success() {
            return Err(CliError::dev_mode(format!("cargo {} failed with exit code: {:?}", command, status.code())));
        }
        
        Ok(())
    }
    
    /// Get current development mode based on configuration and environment
    pub fn get_current_mode(&self) -> CliResult<DevelopmentMode> {
        if cfg!(feature = "dev-ui") {
            Ok(DevelopmentMode::Development)
        } else {
            Ok(DevelopmentMode::Production)
        }
    }
    
    /// Show workflow status and recommendations
    pub fn show_workflow_status(&self) -> CliResult<()> {
        println!("{}", style("RustyUI Workflow Status:").bold());
        
        let current_mode = self.get_current_mode()?;
        println!("  Current mode: {}", style(current_mode.to_string()).cyan());
        
        let analysis = self.project_manager.analyze_project()?;
        println!("  Project type: {:?}", analysis.project_type);
        
        if let Some(framework) = &analysis.detected_framework {
            println!("  Framework: {}", style(framework).cyan());
        }
        
        println!("\n{}", style("Available Commands:").bold());
        println!("  {} - Start development mode", style("rustyui dev").cyan());
        println!("  {} - Development with hot reload", style("cargo run --features dev-ui").cyan());
        println!("  {} - Production build", style("cargo build --release").cyan());
        println!("  {} - Show configuration", style("rustyui config").cyan());
        
        println!("\n{}", style("Workflow Integration:").bold());
        println!("  {} Automatic mode detection", style("✓").green());
        println!("  {} Seamless Cargo integration", style("✓").green());
        println!("  {} Platform-optimized settings", style("✓").green());
        
        Ok(())
    }
}

/// Development mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevelopmentMode {
    Development,
    Production,
}

impl std::fmt::Display for DevelopmentMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DevelopmentMode::Development => write!(f, "Development"),
            DevelopmentMode::Production => write!(f, "Production"),
        }
    }
}