//! Start development mode with hot reload

use crate::error::{CliError, CliResult};
use crate::project::ProjectManager;
use console::style;
use std::path::PathBuf;

#[cfg(feature = "dev-ui")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "dev-ui")]
use std::sync::mpsc;
#[cfg(feature = "dev-ui")]
use std::thread;

/// Command to start development mode
pub struct DevCommand {
    path: PathBuf,
    port: Option<u16>,
    watch: bool,
}

impl DevCommand {
    /// Create a new dev command
    pub fn new(path: PathBuf, port: Option<u16>, watch: bool) -> Self {
        Self { path, port, watch }
    }
    
    /// Execute the dev command
    pub fn execute(&mut self) -> CliResult<()> {
        println!("{} Starting RustyUI development mode...", style("").blue());
        
        // Initialize workflow manager for seamless integration
        let mut workflow_manager = crate::workflow::WorkflowManager::new(self.path.clone())?;
        
        // Check if it's a RustyUI project
        let project_manager = ProjectManager::new(self.path.clone())?;
        if !project_manager.is_rustyui_project() {
            return Err(CliError::dev_mode(
                "Not a RustyUI project. Run 'rustyui init' first."
            ));
        }
        
        // Auto-configure runtime settings for optimal performance
        let config = workflow_manager.auto_configure_runtime()?;
        
        // Show development mode info
        self.show_dev_info(&config)?;
        
        // Handle seamless mode transition to development
        workflow_manager.handle_mode_transition(crate::workflow::DevelopmentMode::Development)?;
        
        // Start development mode with dual-mode engine integration
        self.start_development_mode_with_engine(&config)?;
        
        Ok(())
    }
    
    /// Show development mode information
    fn show_dev_info(&self, config: &rustyui_core::DualModeConfig) -> CliResult<()> {
        println!("\n{}", style("Development Mode Configuration:").bold());
        println!("Framework: {}", style(&format!("{:?}", config.framework)).cyan());
        println!("Watch paths: {:?}", config.watch_paths);
        
        #[cfg(feature = "dev-ui")]
        {
            println!("Interpretation strategy: {:?}", config.development_settings.interpretation_strategy);
            println!("JIT threshold: {}ms", config.development_settings.jit_compilation_threshold);
            println!("State preservation: {}", 
                if config.development_settings.state_preservation { 
                    style("enabled").green() 
                } else { 
                    style("disabled").red() 
                }
            );
            println!("Performance monitoring: {}", 
                if config.development_settings.performance_monitoring { 
                    style("enabled").green() 
                } else { 
                    style("disabled").red() 
                }
            );
            println!("Change detection delay: {}ms", config.development_settings.change_detection_delay_ms);
        }
        
        if let Some(port) = self.port {
            println!("Port: {}", style(port.to_string()).cyan());
        }
        
        println!("File watching: {}", 
            if self.watch { 
                style("enabled").green() 
            } else { 
                style("disabled").red() 
            }
        );
        
        println!("\n{}", style("Features:").bold());
        println!("Instant hot reload for UI changes");
        println!("💾 State preservation across code changes");
        println!("Performance monitoring and metrics");
        println!("File watching for automatic updates");
        println!("Safe runtime code evaluation");
        
        println!("\n{}", style("Controls:").bold());
        println!("Ctrl+C to stop development mode");
        println!("Edit your UI code and see instant changes!");
        
        Ok(())
    }
    
    /// Start development mode with dual-mode engine integration
    fn start_development_mode_with_engine(&self, config: &rustyui_core::DualModeConfig) -> CliResult<()> {
        println!("\n{} Initializing dual-mode engine...", style("⚙️").blue());
        
        #[cfg(feature = "dev-ui")]
        {
            self.start_dual_mode_engine(config)
        }
        
        #[cfg(not(feature = "dev-ui"))]
        {
            // Fallback to basic cargo run in production builds
            println!("{} Development features not available in production build", style("").yellow());
            println!("Run with: cargo run --features dev-ui");
            
            let project_manager = ProjectManager::new(self.path.clone())?;
            project_manager.run_development(self.watch)
        }
    }
    
    /// Start the dual-mode engine (enhanced implementation)
    #[cfg(feature = "dev-ui")]
    fn start_dual_mode_engine(&self, config: &rustyui_core::DualModeConfig) -> CliResult<()> {
        use rustyui_core::DualModeEngine;
        use std::sync::{Arc, Mutex};
        use std::sync::mpsc;
        
        println!("{} Initializing dual-mode engine...", style("⚙️").blue());
        
        // Create and configure the dual-mode engine
        let mut engine = DualModeEngine::new(config.clone())
            .map_err(|e| CliError::dev_mode(format!("Failed to create dual-mode engine: {}", e)))?;
        
        // Initialize the engine
        engine.initialize()
            .map_err(|e| CliError::dev_mode(format!("Failed to initialize dual-mode engine: {}", e)))?;
        
        // Start development mode
        engine.start_development_mode()
            .map_err(|e| CliError::dev_mode(format!("Failed to start development mode: {}", e)))?;
        
        println!("{} Development mode active!", style("").green());
        println!("Hot reload enabled - edit your UI code and see instant changes!");
        println!("Performance monitoring: {}", 
            if config.development_settings.performance_monitoring { 
                style("enabled").green() 
            } else { 
                style("disabled").yellow() 
            }
        );
        
        if self.watch {
            println!("👀 File watching active on {} paths", config.watch_paths.len());
            for path in &config.watch_paths {
                println!("- {}", style(path.display()).dim());
            }
        }
        
        // Show mode-specific information
        self.show_mode_information(&engine)?;
        
        // Create shared engine for thread communication
        let engine_arc = Arc::new(Mutex::new(engine));
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        
        // Set up Ctrl+C handler
        let shutdown_tx_clone = shutdown_tx.clone();
        ctrlc::set_handler(move || {
            println!("\n{} Received shutdown signal...", style("🛑").yellow());
            let _ = shutdown_tx_clone.send(());
        }).map_err(|e| CliError::dev_mode(format!("Failed to set up signal handler: {}", e)))?;
        
        // Start the development server loop
        self.run_development_loop(engine_arc, shutdown_rx)?;
        
        Ok(())
    }
    
    /// Show mode-specific information and capabilities
    #[cfg(feature = "dev-ui")]
    fn show_mode_information(&self, engine: &rustyui_core::DualModeEngine) -> CliResult<()> {
        println!("\n{} Mode Information:", style("").blue());
        
        // Current mode
        println!("Current mode: {}", style("Development").green());
        println!("Runtime interpreter: {}", 
            if engine.has_runtime_interpreter() { 
                style("active").green() 
            } else { 
                style("inactive").red() 
            }
        );
        
        // Platform capabilities
        println!("Platform: {:?}", engine.platform());
        println!("Native optimizations: {}", 
            if engine.is_using_native_optimizations() { 
                style("enabled").green() 
            } else { 
                style("disabled").yellow() 
            }
        );
        println!("JIT compilation: {}", 
            if engine.jit_compilation_available() { 
                style("available").green() 
            } else { 
                style("unavailable").yellow() 
            }
        );
        
        // Memory and performance
        let memory_mb = engine.memory_overhead() as f64 / (1024.0 * 1024.0);
        println!("Memory overhead: {:.1} MB", memory_mb);
        
        let expected_mb = engine.expected_memory_overhead() as f64 / (1024.0 * 1024.0);
        println!("Expected overhead: {:.1} MB", expected_mb);
        
        // Health status
        let health = engine.get_health_status();
        println!("System health: {:?}", health);
        
        // Configuration validation
        if engine.can_interpret_changes() {
            println!("{} Ready for hot reload", style("✓").green());
        } else {
            println!("{} Hot reload not available", style("⚠").yellow());
        }
        
        Ok(())
    }
    
    /// Run the main development loop with file watching and hot reload
    #[cfg(feature = "dev-ui")]
    fn run_development_loop(
        &self, 
        engine: Arc<Mutex<rustyui_core::DualModeEngine>>, 
        shutdown_rx: mpsc::Receiver<()>
    ) -> CliResult<()> {
        use std::time::{Duration, Instant};
        
        println!("\n{} Development server running...", style("").green());
        println!("Press Ctrl+C to stop");
        
        let mut last_stats_report = Instant::now();
        let stats_interval = Duration::from_secs(30); // Report stats every 30 seconds
        
        loop {
            // Check for shutdown signal (non-blocking)
            if shutdown_rx.try_recv().is_ok() {
                break;
            }
            
            // Process file changes and perform hot reload
            if let Ok(mut engine_guard) = engine.lock() {
                // Check for file changes
                if let Ok(Some(analysis)) = engine_guard.process_and_analyze_changes() {
                    self.handle_file_changes(&mut engine_guard, analysis)?;
                }
                
                // Report performance statistics periodically
                if last_stats_report.elapsed() >= stats_interval {
                    self.report_performance_stats(&engine_guard);
                    last_stats_report = Instant::now();
                }
            }
            
            // Sleep briefly to avoid busy waiting
            thread::sleep(Duration::from_millis(50));
        }
        
        // Graceful shutdown
        self.handle_shutdown()?;
        
        Ok(())
    }
    
    /// Handle file changes and trigger hot reload
    #[cfg(feature = "dev-ui")]
    fn handle_file_changes(
        &self, 
        engine: &mut rustyui_core::DualModeEngine, 
        analysis: rustyui_core::change_analyzer::AnalysisResult
    ) -> CliResult<()> {
        use console::style;
        
        let start_time = std::time::Instant::now();
        
        println!("\nProcessing {} file changes...", 
            analysis.analyzed_changes.len()
        );
        
        let mut successful_interpretations = 0;
        let mut failed_interpretations = 0;
        
        // Process each analyzed change
        for analyzed_change in &analysis.analyzed_changes {
            let change = &analyzed_change.original_change; // Fixed field name
            
            // Read the file content
            let content = match std::fs::read_to_string(&change.path) {
                Ok(content) => content,
                Err(e) => {
                    println!("{} Failed to read {}: {}", 
                        style("✗").red(), 
                        change.path.display(), 
                        e
                    );
                    failed_interpretations += 1;
                    continue;
                }
            };
            
            // Determine component ID from file path
            let component_id = change.path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string());
            
            // Interpret the change
            match engine.interpret_ui_change(&content, component_id.clone()) {
                Ok(result) => {
                    successful_interpretations += 1;
                    
                    println!("{} {} ({:?})", 
                        style("✓").green(),
                        change.path.display(),
                        result.execution_time
                    );
                    
                    // Log additional details for high-priority changes
                    if matches!(analyzed_change.classification.priority, 
                        rustyui_core::change_analyzer::ChangePriority::Critical | 
                        rustyui_core::change_analyzer::ChangePriority::High) {
                        println!("Priority: {:?}, Impact: {:?}", 
                            analyzed_change.classification.priority,
                            analyzed_change.impact.scope // Fixed field access
                        );
                    }
                }
                Err(e) => {
                    failed_interpretations += 1;
                    
                    println!("{} {} - {}", 
                        style("✗").red(),
                        change.path.display(),
                        e
                    );
                }
            }
        }
        
        let total_time = start_time.elapsed();
        
        // Report results
        println!("\nHot reload completed in {:?}", 
            total_time
        );
        println!("{} successful, {} failed", 
            style(successful_interpretations.to_string()).green(),
            if failed_interpretations > 0 { 
                style(failed_interpretations.to_string()).red() 
            } else { 
                style("0".to_string()).dim() // Fixed type mismatch
            }
        );
        
        // Check if full reload is needed
        if analysis.requires_full_reload {
            println!("{} Full application reload recommended", 
                style("").yellow()
            );
        }
        
        // Report cascade updates
        if !analysis.cascade_updates.is_empty() {
            println!("{} cascade updates triggered", 
                analysis.cascade_updates.len()
            );
        }
        
        Ok(())
    }
    
    /// Report performance statistics
    #[cfg(feature = "dev-ui")]
    fn report_performance_stats(&self, engine: &rustyui_core::DualModeEngine) {
        println!("\n{} Performance Report:", style("").blue());
        
        // Memory overhead
        let memory_mb = engine.memory_overhead() as f64 / (1024.0 * 1024.0);
        println!("Memory overhead: {:.1} MB", memory_mb);
        
        // File watching stats
        if let Some(stats) = engine.get_file_watching_stats() {
            println!("File watching:");
            println!("Events processed: {}", stats.total_events);
            println!("Average response time: {:?}", stats.average_processing_time());
            println!("Performance target: {}", 
                if stats.meets_performance_targets() { 
                    style("✓ Met (<50ms)").green() 
                } else { 
                    style("✗ Exceeded (>50ms)").red() 
                }
            );
            println!("Debounced events: {}", stats.debounced_events);
        }
        
        // Analysis stats
        if let Some(stats) = engine.get_analysis_stats() {
            println!("Change analysis:");
            println!("Total analyses: {}", stats.total_analyses);
            println!("Average analysis time: {:?}", stats.average_analysis_time());
            println!("Cache hit rate: {:.1}%", stats.cache_hit_rate() * 100.0);
        }
    }
    
    /// Handle graceful shutdown
    #[allow(dead_code)]
    fn handle_shutdown(&self) -> CliResult<()> {
        println!("\n{} Shutting down development mode...", style("🛑").yellow());
        
        // Clean up resources
        // Stop file watchers
        // Save any pending state
        // Close connections
        
        println!("{} Development mode stopped", style("✓").green());
        
        Ok(())
    }
}