//! Dual-mode engine implementation with cross-platform optimization

use crate::{config::DualModeConfig, error::{Result, RustyUIError}, platform::{Platform, PlatformConfig, PlatformCapabilities}};
use std::time::{Duration, Instant};

#[cfg(feature = "dev-ui")]
use crate::{ChangeMonitor, ChangeAnalyzer, StatePreservor, error_recovery::{ErrorRecoveryManager, ErrorContext, Operation}, error_reporting::{ErrorReporter, ErrorReportContext, ErrorOperation}, performance::{PerformanceMonitor, PerformanceTargets}};

// Production-compatible stub types
#[cfg(not(feature = "dev-ui"))]
#[derive(Debug, Clone)]
pub enum Operation {
    Interpretation,
    StatePreservation,
    FileWatching,
    ComponentRendering,
    JITCompilation,
    RhaiExecution,
    ASTParsing,
    FrameworkIntegration,
}

#[cfg(not(feature = "dev-ui"))]
pub mod error_recovery {
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum RecoveryResult {
        Failed { strategy: RecoveryStrategy, message: String },
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum RecoveryStrategy {
        IsolateAndContinue,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum HealthStatus {
        Healthy,
    }
}

// Production-compatible InterpretationResult
#[derive(Debug)]
pub struct InterpretationResult {
    pub execution_time: std::time::Duration,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Core dual-mode engine that manages runtime interpretation and production compilation
pub struct DualModeEngine {
    /// Configuration for dual-mode operation
    config: DualModeConfig,
    
    /// Platform-specific configuration and capabilities
    platform_config: PlatformConfig,
    
    /// Change monitor for file system watching (development only)
    #[cfg(feature = "dev-ui")]
    change_monitor: Option<ChangeMonitor>,
    
    /// Intelligent change analyzer for 2026 classification (development only)
    #[cfg(feature = "dev-ui")]
    change_analyzer: Option<ChangeAnalyzer>,
    
    /// State preservation system (development only)
    #[cfg(feature = "dev-ui")]
    state_preservor: Option<StatePreservor>,
    
    /// Error recovery manager (development only)
    #[cfg(feature = "dev-ui")]
    error_recovery: Option<ErrorRecoveryManager>,
    
    /// Error reporter for comprehensive logging (development only)
    #[cfg(feature = "dev-ui")]
    error_reporter: Option<ErrorReporter>,
    
    /// Performance monitor for tracking metrics and targets (development only)
    #[cfg(feature = "dev-ui")]
    performance_monitor: Option<PerformanceMonitor>,
    
    /// Component lifecycle manager (development only)
    #[cfg(feature = "dev-ui")]
    component_manager: Option<crate::component_lifecycle::ComponentLifecycleManager>,
    
    /// Engine initialization state
    initialized: bool,
}

impl DualModeEngine {
    /// Create a new dual-mode engine with the given configuration
    pub fn new(config: DualModeConfig) -> Result<Self> {
        // Check platform requirements
        PlatformCapabilities::check_minimum_requirements()
            .map_err(|e| RustyUIError::initialization(format!("Platform requirements not met: {}", e)))?;
        
        // Auto-detect platform configuration
        let platform_config = PlatformConfig::auto_detect();
        platform_config.validate()
            .map_err(|e| RustyUIError::initialization(format!("Platform configuration invalid: {}", e)))?;
        
        println!("🚀 Initializing RustyUI on {} with {} file watcher", 
            platform_config.platform, 
            match platform_config.file_watcher_backend {
                crate::platform::FileWatcherBackend::ReadDirectoryChanges => "ReadDirectoryChanges",
                crate::platform::FileWatcherBackend::FSEvents => "FSEvents",
                crate::platform::FileWatcherBackend::INotify => "inotify",
                crate::platform::FileWatcherBackend::Poll => "polling",
            }
        );
        
        Ok(Self {
            config,
            platform_config,
            #[cfg(feature = "dev-ui")]
            change_monitor: None,
            #[cfg(feature = "dev-ui")]
            change_analyzer: None,
            #[cfg(feature = "dev-ui")]
            state_preservor: None,
            #[cfg(feature = "dev-ui")]
            error_recovery: None,
            #[cfg(feature = "dev-ui")]
            error_reporter: None,
            #[cfg(feature = "dev-ui")]
            performance_monitor: None,
            #[cfg(feature = "dev-ui")]
            component_manager: None,
            initialized: false,
        })
    }
    
    /// Create a new dual-mode engine with custom platform configuration
    pub fn with_platform_config(config: DualModeConfig, platform_config: PlatformConfig) -> Result<Self> {
        // Validate platform configuration
        platform_config.validate()
            .map_err(|e| RustyUIError::initialization(format!("Platform configuration invalid: {}", e)))?;
        
        Ok(Self {
            config,
            platform_config,
            #[cfg(feature = "dev-ui")]
            change_monitor: None,
            #[cfg(feature = "dev-ui")]
            change_analyzer: None,
            #[cfg(feature = "dev-ui")]
            state_preservor: None,
            #[cfg(feature = "dev-ui")]
            error_recovery: None,
            #[cfg(feature = "dev-ui")]
            error_reporter: None,
            #[cfg(feature = "dev-ui")]
            performance_monitor: None,
            #[cfg(feature = "dev-ui")]
            component_manager: None,
            initialized: false,
        })
    }
    
    /// Initialize the dual-mode engine with platform-specific optimizations
    pub fn initialize(&mut self) -> Result<()> {
        #[cfg(feature = "dev-ui")]
        {
            // Check development feature requirements
            PlatformCapabilities::check_dev_features()
                .map_err(|e| RustyUIError::initialization(format!("Development features not available: {}", e)))?;
            
            // Initialize development-only components with platform optimization
            self.change_monitor = Some(ChangeMonitor::with_platform_config(
                &self.config.watch_paths, 
                self.platform_config.clone()
            )?);
            self.change_analyzer = Some(ChangeAnalyzer::new());
            self.state_preservor = Some(StatePreservor::new());
            self.error_recovery = Some(ErrorRecoveryManager::new());
            self.error_reporter = Some(ErrorReporter::new());
            
            // Initialize performance monitor with targets from config
            let performance_targets = PerformanceTargets {
                max_interpretation_time_ms: 100,  // Requirement 7.2
                max_file_change_time_ms: 50,      // Requirement 7.2
                max_memory_overhead_mb: self.config.development_settings.max_memory_overhead_mb.unwrap_or(50),
                max_jit_compilation_time_ms: self.config.development_settings.jit_compilation_threshold as u64,
                max_state_preservation_time_ms: 10,
            };
            self.performance_monitor = Some(PerformanceMonitor::with_targets(performance_targets));
            
            // Initialize component lifecycle manager
            self.component_manager = Some(crate::component_lifecycle::ComponentLifecycleManager::new());
            
            println!("✅ Development mode initialized with platform optimizations");
            println!("  File watcher: {} (expected latency: {}ms)", 
                match self.platform_config.file_watcher_backend {
                    crate::platform::FileWatcherBackend::ReadDirectoryChanges => "ReadDirectoryChanges",
                    crate::platform::FileWatcherBackend::FSEvents => "FSEvents", 
                    crate::platform::FileWatcherBackend::INotify => "inotify",
                    crate::platform::FileWatcherBackend::Poll => "polling",
                },
                self.platform_config.file_watcher_backend.performance_characteristics().latency_ms
            );
            println!("  JIT compilation: {}", 
                if self.platform_config.use_jit_compilation { "enabled" } else { "disabled" }
            );
            println!("  Thread count: {}", self.platform_config.thread_count);
        }
        
        self.initialized = true;
        Ok(())
    }
    
    /// Check if the engine is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Start development mode with runtime interpretation (development only)
    #[cfg(feature = "dev-ui")]
    pub fn start_development_mode(&mut self) -> Result<()> {
        if !self.initialized {
            self.initialize()?;
        }
        
        if let Some(ref mut monitor) = self.change_monitor {
            monitor.start_watching()?;
        }
        
        Ok(())
    }
    
    /// Start development mode (no-op in production builds)
    #[cfg(not(feature = "dev-ui"))]
    pub fn start_development_mode(&mut self) -> Result<()> {
        // Development features are stripped in production
        Ok(())
    }
    
    /// Check if runtime interpreter is available (development only)
    #[cfg(feature = "dev-ui")]
    pub fn has_runtime_interpreter(&self) -> bool {
        self.change_monitor.is_some()
    }
    
    /// Check if runtime interpreter is available (always false in production)
    #[cfg(not(feature = "dev-ui"))]
    pub fn has_runtime_interpreter(&self) -> bool {
        false
    }
    
    /// Check if the engine can interpret changes (development only)
    #[cfg(feature = "dev-ui")]
    pub fn can_interpret_changes(&self) -> bool {
        self.initialized && self.change_monitor.is_some()
    }
    
    /// Check if the engine can interpret changes (always false in production)
    #[cfg(not(feature = "dev-ui"))]
    pub fn can_interpret_changes(&self) -> bool {
        false
    }
    
    /// Get memory overhead in bytes (development only)
    #[cfg(feature = "dev-ui")]
    pub fn memory_overhead(&self) -> usize {
        // Estimate memory overhead from development components
        let mut overhead = 0;
        
        if self.change_monitor.is_some() {
            overhead += 1024 * 1024; // ~1MB for file watching
        }
        
        if self.state_preservor.is_some() {
            overhead += 512 * 1024; // ~512KB for state preservation
        }
        
        overhead
    }
    
    /// Get file watching performance statistics (development only)
    #[cfg(feature = "dev-ui")]
    pub fn get_file_watching_stats(&self) -> Option<&crate::change_monitor::ChangeStats> {
        self.change_monitor.as_ref().map(|monitor| monitor.get_stats())
    }
    
    /// Get change analysis performance statistics (development only)
    #[cfg(feature = "dev-ui")]
    pub fn get_analysis_stats(&self) -> Option<&crate::change_analyzer::AnalysisStats> {
        self.change_analyzer.as_ref().map(|analyzer| analyzer.get_stats())
    }
    
    /// Process pending file changes with intelligent analysis (development only)
    #[cfg(feature = "dev-ui")]
    pub fn process_file_changes(&mut self) -> Result<Vec<crate::change_monitor::FileChange>> {
        if let Some(ref mut monitor) = self.change_monitor {
            Ok(monitor.check_changes())
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Process and analyze file changes with 2026 intelligent classification (development only)
    #[cfg(feature = "dev-ui")]
    pub fn process_and_analyze_changes(&mut self) -> Result<Option<crate::change_analyzer::AnalysisResult>> {
        if let (Some(ref mut monitor), Some(ref mut analyzer)) = 
            (&mut self.change_monitor, &mut self.change_analyzer) {
            
            let changes = monitor.check_changes();
            if !changes.is_empty() {
                println!("🔍 Analyzing {} file changes with 2026 intelligent classification", changes.len());
                let analysis = analyzer.analyze_changes(changes);
                
                // Log analysis results
                println!("📊 Analysis completed in {:?}", analysis.analysis_time);
                println!("  Priority changes: {}", 
                    analysis.analyzed_changes.iter()
                        .filter(|c| matches!(c.classification.priority, 
                            crate::change_analyzer::ChangePriority::Critical | 
                            crate::change_analyzer::ChangePriority::High))
                        .count()
                );
                println!("  Requires full reload: {}", analysis.requires_full_reload);
                println!("  Cascade updates: {}", analysis.cascade_updates.len());
                
                return Ok(Some(analysis));
            }
        }
        Ok(None)
    }
    
    /// Get memory overhead in bytes (always 0 in production)
    #[cfg(not(feature = "dev-ui"))]
    pub fn memory_overhead(&self) -> usize {
        0
    }
    
    /// Interpret a UI change during development (development only)
    #[cfg(feature = "dev-ui")]
    pub fn interpret_ui_change(&mut self, code: &str, component_id: Option<String>) -> Result<InterpretationResult> {
        if !self.can_interpret_changes() {
            return Err(RustyUIError::interpretation("Engine not initialized for interpretation"));
        }

        // For Phase 1, create a simple mock interpretation result
        // In a full implementation, this would use the RuntimeInterpreter
        let start_time = std::time::Instant::now();
        
        // Simulate interpretation work
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        let execution_time = start_time.elapsed();
        
        println!("🔄 Interpreted UI change ({} chars) in {:?}", code.len(), execution_time);
        if let Some(id) = component_id {
            println!("  Component: {}", id);
        }
        
        Ok(InterpretationResult {
            execution_time,
            success: true,
            error_message: None,
        })
    }
    
    /// Interpret a UI change (no-op in production builds)
    #[cfg(not(feature = "dev-ui"))]
    pub fn interpret_ui_change(&mut self, _code: &str, _component_id: Option<String>) -> Result<InterpretationResult> {
        // Return a dummy success result in production
        Ok(InterpretationResult {
            execution_time: std::time::Duration::from_nanos(0),
            success: true,
            error_message: None,
        })
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &DualModeConfig {
        &self.config
    }
    
    /// Get the platform configuration
    pub fn platform_config(&self) -> &PlatformConfig {
        &self.platform_config
    }
    
    /// Get the current platform
    pub fn platform(&self) -> Platform {
        self.platform_config.platform
    }
    
    /// Check if the engine is using platform-native optimizations
    pub fn is_using_native_optimizations(&self) -> bool {
        self.platform_config.use_native_apis
    }
    
    /// Get expected memory overhead based on platform capabilities
    pub fn expected_memory_overhead(&self) -> usize {
        #[cfg(feature = "dev-ui")]
        {
            let base_overhead = self.memory_overhead();
            let platform_overhead = self.platform_config.file_watcher_backend
                .performance_characteristics().memory_overhead_bytes as usize;
            base_overhead + platform_overhead
        }
        
        #[cfg(not(feature = "dev-ui"))]
        {
            0
        }
    }
    
    /// Check if JIT compilation is available and enabled
    pub fn jit_compilation_available(&self) -> bool {
        #[cfg(feature = "dev-ui")]
        {
            self.platform_config.use_jit_compilation && 
            self.platform_config.platform.jit_capabilities().supports_cranelift
        }
        
        #[cfg(not(feature = "dev-ui"))]
        {
            false
        }
    }
    
    /// Handle an error with recovery mechanisms (development only)
    #[cfg(feature = "dev-ui")]
    pub fn handle_error(&mut self, error: &RustyUIError, operation: Operation, component_id: Option<String>) -> crate::error_recovery::RecoveryResult {
        // Collect data needed for reporting before borrowing mutably
        let error_operation = self.map_operation_to_error_operation(&operation);
        let affects_core = self.is_core_functionality_affected(&operation);
        let system_state = self.get_system_state();
        
        // Report error with full context
        if let Some(ref mut reporter) = self.error_reporter {
            let report_context = ErrorReportContext {
                operation: error_operation,
                component_id: component_id.clone(),
                file_path: None,
                line_number: None,
                affects_core_functionality: affects_core,
                user_action: None,
                system_state,
            };
            
            reporter.report_error(error, report_context);
        }
        
        // Attempt recovery
        if let Some(ref mut recovery_manager) = self.error_recovery {
            let context = ErrorContext {
                operation,
                component_id,
                context_data: std::collections::HashMap::new(),
            };
            
            let result = recovery_manager.handle_error(error, context);
            
            // Log recovery attempt
            match &result {
                crate::error_recovery::RecoveryResult::Success { strategy, message, .. } => {
                    println!("✅ Error recovery successful: {} (strategy: {:?})", message, strategy);
                }
                crate::error_recovery::RecoveryResult::PartialRecovery { strategy, message, limitations } => {
                    println!("⚠️ Partial error recovery: {} (strategy: {:?})", message, strategy);
                    for limitation in limitations {
                        println!("  - {}", limitation);
                    }
                }
                crate::error_recovery::RecoveryResult::Failed { strategy, message } => {
                    println!("❌ Error recovery failed: {} (strategy: {:?})", message, strategy);
                }
            }
            
            result
        } else {
            crate::error_recovery::RecoveryResult::Failed {
                strategy: crate::error_recovery::RecoveryStrategy::IsolateAndContinue,
                message: "Error recovery manager not initialized".to_string(),
            }
        }
    }
    
    /// Map Operation to ErrorOperation for reporting
    #[cfg(feature = "dev-ui")]
    fn map_operation_to_error_operation(&self, operation: &Operation) -> ErrorOperation {
        match operation {
            Operation::Interpretation => ErrorOperation::RhaiInterpretation,
            Operation::StatePreservation => ErrorOperation::StatePreservation,
            Operation::FileWatching => ErrorOperation::FileWatching,
            Operation::ComponentRendering => ErrorOperation::ComponentRendering,
            Operation::JITCompilation => ErrorOperation::JITCompilation,
            Operation::RhaiExecution => ErrorOperation::RhaiInterpretation,
            Operation::ASTParsing => ErrorOperation::ASTInterpretation,
            Operation::FrameworkIntegration => ErrorOperation::FrameworkIntegration,
        }
    }
    
    /// Check if operation affects core functionality
    #[cfg(feature = "dev-ui")]
    fn is_core_functionality_affected(&self, operation: &Operation) -> bool {
        matches!(operation, 
            Operation::Interpretation | 
            Operation::ComponentRendering | 
            Operation::FrameworkIntegration
        )
    }
    
    /// Get current system state for error reporting
    #[cfg(feature = "dev-ui")]
    fn get_system_state(&self) -> std::collections::HashMap<String, String> {
        let mut state = std::collections::HashMap::new();
        
        state.insert("platform".to_string(), format!("{:?}", self.platform_config.platform));
        state.insert("initialized".to_string(), self.initialized.to_string());
        state.insert("jit_available".to_string(), self.jit_compilation_available().to_string());
        state.insert("memory_overhead".to_string(), self.memory_overhead().to_string());
        
        if let Some(ref stats) = self.get_file_watching_stats() {
            state.insert("file_changes_processed".to_string(), stats.total_events.to_string());
        }
        
        state
    }
    
    /// Handle an error (no-op in production builds)
    #[cfg(not(feature = "dev-ui"))]
    pub fn handle_error(&mut self, _error: &RustyUIError, _operation: Operation, _component_id: Option<String>) -> error_recovery::RecoveryResult {
        error_recovery::RecoveryResult::Failed {
            strategy: error_recovery::RecoveryStrategy::IsolateAndContinue,
            message: "Error recovery not available in production builds".to_string(),
        }
    }
    
    /// Get error recovery metrics (development only)
    #[cfg(feature = "dev-ui")]
    pub fn get_error_recovery_metrics(&self) -> Option<&crate::error_recovery::RecoveryMetrics> {
        self.error_recovery.as_ref().map(|manager| manager.get_metrics())
    }
    
    /// Get system health status (development only)
    #[cfg(feature = "dev-ui")]
    pub fn get_health_status(&self) -> crate::error_recovery::HealthStatus {
        if let Some(ref recovery_manager) = self.error_recovery {
            recovery_manager.get_health_status()
        } else {
            crate::error_recovery::HealthStatus::Healthy
        }
    }
    
    /// Get system health status (always healthy in production)
    #[cfg(not(feature = "dev-ui"))]
    pub fn get_health_status(&self) -> error_recovery::HealthStatus {
        error_recovery::HealthStatus::Healthy
    }
    
    /// Store fallback state for error recovery (development only)
    #[cfg(feature = "dev-ui")]
    pub fn store_fallback_state(&mut self, component_id: String, state_data: serde_json::Value) {
        if let Some(ref mut recovery_manager) = self.error_recovery {
            recovery_manager.store_fallback_state(component_id, state_data);
        }
    }
    
    /// Store fallback state (no-op in production)
    #[cfg(not(feature = "dev-ui"))]
    pub fn store_fallback_state(&mut self, _component_id: String, _state_data: serde_json::Value) {
        // No-op in production builds
    }
    
    /// Get error report (development only)
    #[cfg(feature = "dev-ui")]
    pub fn get_error_report(&self) -> Option<crate::error_reporting::ErrorReport> {
        self.error_reporter.as_ref().map(|reporter| reporter.generate_report())
    }
    
    /// Get error metrics (development only)
    #[cfg(feature = "dev-ui")]
    pub fn get_error_metrics(&self) -> Option<&crate::error_reporting::ErrorMetrics> {
        self.error_reporter.as_ref().map(|reporter| reporter.get_metrics())
    }
    
    /// Clear error logs (development only)
    #[cfg(feature = "dev-ui")]
    pub fn clear_error_logs(&mut self) {
        if let Some(ref mut reporter) = self.error_reporter {
            reporter.clear_logs();
        }
        if let Some(ref mut recovery_manager) = self.error_recovery {
            recovery_manager.clear_error_history();
        }
    }
    
    // Component Lifecycle Management Methods
    
    /// Register a new UI component (development only)
    #[cfg(feature = "dev-ui")]
    pub fn register_component(&mut self, id: String, type_name: String) -> Result<()> {
        if let Some(ref mut manager) = self.component_manager {
            manager.register_component(id.clone(), type_name.clone())?;
            
            // Record performance measurement
            if let Some(ref mut perf_monitor) = self.performance_monitor {
                let measurement = crate::performance::PerformanceMeasurement {
                    operation: "component_registration".to_string(),
                    duration: Duration::from_millis(1), // Minimal overhead
                    timestamp: std::time::SystemTime::now(),
                    component_id: Some(id),
                    memory_usage_bytes: Some(0),
                    success: true,
                    metadata: std::collections::HashMap::new(),
                };
                perf_monitor.record_measurement(measurement);
            }
        }
        Ok(())
    }
    
    /// Update component state (development only)
    #[cfg(feature = "dev-ui")]
    pub fn update_component_state(&mut self, id: &str, state: crate::component_lifecycle::ComponentState) -> Result<()> {
        if let Some(ref mut manager) = self.component_manager {
            manager.update_component_state(id, state)?;
        }
        Ok(())
    }
    
    /// Preserve component state for hot reload (development only)
    #[cfg(feature = "dev-ui")]
    pub fn preserve_component_state(&mut self, component_id: &str, state_data: serde_json::Value) -> Result<()> {
        // Store in component manager
        if let Some(ref mut manager) = self.component_manager {
            manager.preserve_component_state(component_id, state_data.clone())?;
        }
        
        // Store in state preservor
        if let Some(ref mut preservor) = self.state_preservor {
            preservor.save_global_state(component_id, &state_data)?;
        }
        
        // Store in error recovery for fallback
        if let Some(ref mut recovery) = self.error_recovery {
            recovery.store_fallback_state(component_id.to_string(), state_data);
        }
        
        Ok(())
    }
    
    /// Restore component state after hot reload (development only)
    #[cfg(feature = "dev-ui")]
    pub fn restore_component_state(&mut self, component_id: &str) -> Option<serde_json::Value> {
        // Try component manager first
        if let Some(ref manager) = self.component_manager {
            if let Some(state) = manager.restore_component_state(component_id) {
                return Some(state.clone());
            }
        }
        
        // Fallback to state preservor
        if let Some(ref mut preservor) = self.state_preservor {
            if let Ok(Some(state)) = preservor.restore_global_state::<serde_json::Value>(component_id) {
                return Some(state);
            }
        }
        
        None
    }
    
    /// Get component lifecycle information (development only)
    #[cfg(feature = "dev-ui")]
    pub fn get_component_info(&self, id: &str) -> Option<&crate::component_lifecycle::ComponentLifecycle> {
        self.component_manager.as_ref()?.get_component(id)
    }
    
    /// Get all active components (development only)
    #[cfg(feature = "dev-ui")]
    pub fn get_active_components(&self) -> Vec<&crate::component_lifecycle::ComponentLifecycle> {
        self.component_manager.as_ref()
            .map(|manager| manager.get_active_components())
            .unwrap_or_default()
    }
    
    /// Get component statistics (development only)
    #[cfg(feature = "dev-ui")]
    pub fn get_component_statistics(&self) -> Option<crate::component_lifecycle::ComponentStatistics> {
        self.component_manager.as_ref().map(|manager| manager.get_statistics())
    }
    
    // Integrated Hot Reload Workflow
    
    /// Process file change and update components (development only)
    #[cfg(feature = "dev-ui")]
    pub fn process_file_change_and_update(&mut self, file_path: &str) -> Result<Vec<String>> {
        let start_time = Instant::now();
        let mut updated_components = Vec::new();
        
        // Process file change
        let _changes = self.process_file_changes()?;
        
        // Analyze changes
        if let Some(analysis_result) = self.process_and_analyze_changes()? {
            // Extract affected components from analyzed changes
            let affected_components: Vec<String> = analysis_result.analyzed_changes
                .iter()
                .filter_map(|change| {
                    // Extract component ID from file path or change metadata
                    // This is a simplified approach - in practice, you'd have more sophisticated mapping
                    change.original_change.path.file_stem()
                        .and_then(|stem| stem.to_str())
                        .map(|s| s.to_string())
                })
                .collect();
            
            // Update affected components
            for component_id in &affected_components {
                // Preserve current state
                if let Some(current_state) = self.restore_component_state(component_id) {
                    self.preserve_component_state(component_id, current_state)?;
                }
                
                // Update component state to updating
                self.update_component_state(component_id, crate::component_lifecycle::ComponentState::Updating)?;
                
                // Apply hot reload (this would integrate with framework adapter)
                // For now, just mark as updated
                updated_components.push(component_id.to_string());
                
                // Update component state back to active
                self.update_component_state(component_id, crate::component_lifecycle::ComponentState::Active)?;
            }
        }
        
        // Record performance
        if let Some(ref mut perf_monitor) = self.performance_monitor {
            let measurement = crate::performance::PerformanceMeasurement {
                operation: "hot_reload_cycle".to_string(),
                duration: start_time.elapsed(),
                timestamp: std::time::SystemTime::now(),
                component_id: None,
                memory_usage_bytes: Some(0),
                success: true,
                metadata: {
                    let mut map = std::collections::HashMap::new();
                    map.insert("file_path".to_string(), file_path.to_string());
                    map.insert("components_updated".to_string(), updated_components.len().to_string());
                    map
                },
            };
            perf_monitor.record_measurement(measurement);
        }
        
        Ok(updated_components)
    }
    
    /// Cleanup destroyed components (development only)
    #[cfg(feature = "dev-ui")]
    pub fn cleanup_components(&mut self) {
        if let Some(ref mut manager) = self.component_manager {
            manager.cleanup();
        }
    }
}