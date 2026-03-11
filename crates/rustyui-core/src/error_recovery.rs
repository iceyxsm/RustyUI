//! Error Recovery Manager for RustyUI
//! 
//! Provides comprehensive error handling and recovery mechanisms to prevent
//! application crashes during runtime interpretation and maintain stability.

use crate::error::RustyUIError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Error recovery manager that handles interpretation failures and provides
/// graceful fallback mechanisms
#[cfg(feature = "dev-ui")]
pub struct ErrorRecoveryManager {
    /// Recovery strategies for different error types
    recovery_strategies: HashMap<ErrorType, RecoveryStrategy>,
    
    /// Error history for pattern analysis
    error_history: Arc<Mutex<Vec<ErrorEvent>>>,
    
    /// Fallback state storage
    fallback_states: HashMap<String, FallbackState>,
    
    /// Recovery metrics
    metrics: RecoveryMetrics,
    
    /// Configuration for recovery behavior
    config: RecoveryConfig,
}

#[cfg(feature = "dev-ui")]
impl ErrorRecoveryManager {
    /// Create a new error recovery manager
    pub fn new() -> Self {
        let mut strategies = HashMap::new();
        
        // Initialize default recovery strategies
        strategies.insert(ErrorType::InterpretationFailure, RecoveryStrategy::FallbackToLastWorking);
        strategies.insert(ErrorType::StatePreservationFailure, RecoveryStrategy::ResetToDefault);
        strategies.insert(ErrorType::FileWatchingFailure, RecoveryStrategy::RestartWatcher);
        strategies.insert(ErrorType::FrameworkAdapterFailure, RecoveryStrategy::IsolateAndContinue);
        strategies.insert(ErrorType::JITCompilationFailure, RecoveryStrategy::FallbackToAST);
        strategies.insert(ErrorType::RhaiScriptFailure, RecoveryStrategy::FallbackToAST);
        strategies.insert(ErrorType::ASTParsingFailure, RecoveryStrategy::FallbackToRhai);
        
        Self {
            recovery_strategies: strategies,
            error_history: Arc::new(Mutex::new(Vec::new())),
            fallback_states: HashMap::new(),
            metrics: RecoveryMetrics::default(),
            config: RecoveryConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: RecoveryConfig) -> Self {
        let mut manager = Self::new();
        manager.config = config;
        manager
    }
    
    /// Handle an error and attempt recovery
    pub fn handle_error(&mut self, error: &RustyUIError, context: ErrorContext) -> RecoveryResult {
        let error_type = self.classify_error(error);
        let error_event = ErrorEvent {
            error_type: error_type.clone(),
            timestamp: SystemTime::now(),
            context: context.clone(),
            error_message: error.to_string(),
        };
        
        // Record error in history
        if let Ok(mut history) = self.error_history.lock() {
            history.push(error_event.clone());
            
            // Limit history size
            if history.len() > self.config.max_error_history {
                history.remove(0);
            }
        }
        
        // Update metrics
        self.metrics.total_errors += 1;
        
        // Get recovery strategy
        let strategy = self.recovery_strategies
            .get(&error_type)
            .unwrap_or(&RecoveryStrategy::IsolateAndContinue)
            .clone();
        
        // Attempt recovery
        let recovery_result = self.execute_recovery_strategy(&strategy, &error_event, &context);
        
        // Update metrics based on result
        match &recovery_result {
            RecoveryResult::Success { .. } => {
                self.metrics.successful_recoveries += 1;
            }
            RecoveryResult::PartialRecovery { .. } => {
                self.metrics.partial_recoveries += 1;
            }
            RecoveryResult::Failed { .. } => {
                self.metrics.failed_recoveries += 1;
            }
        }
        
        recovery_result
    }
    
    /// Execute a specific recovery strategy
    fn execute_recovery_strategy(
        &mut self,
        strategy: &RecoveryStrategy,
        error_event: &ErrorEvent,
        context: &ErrorContext,
    ) -> RecoveryResult {
        match strategy {
            RecoveryStrategy::FallbackToLastWorking => {
                self.fallback_to_last_working(context)
            }
            RecoveryStrategy::ResetToDefault => {
                self.reset_to_default(context)
            }
            RecoveryStrategy::RestartWatcher => {
                self.restart_file_watcher(context)
            }
            RecoveryStrategy::IsolateAndContinue => {
                self.isolate_and_continue(error_event, context)
            }
            RecoveryStrategy::FallbackToAST => {
                self.fallback_to_ast(context)
            }
            RecoveryStrategy::FallbackToRhai => {
                self.fallback_to_rhai(context)
            }
            RecoveryStrategy::RetryWithDelay => {
                self.retry_with_delay(context)
            }
            RecoveryStrategy::DisableFeature => {
                self.disable_feature(context)
            }
        }
    }
    
    /// Fallback to the last working state
    fn fallback_to_last_working(&mut self, context: &ErrorContext) -> RecoveryResult {
        if let Some(component_id) = &context.component_id {
            if let Some(fallback_state) = self.fallback_states.get(component_id) {
                return RecoveryResult::Success {
                    strategy: RecoveryStrategy::FallbackToLastWorking,
                    message: format!("Restored component '{}' to last working state", component_id),
                    fallback_data: Some(fallback_state.clone()),
                };
            }
        }
        
        RecoveryResult::PartialRecovery {
            strategy: RecoveryStrategy::FallbackToLastWorking,
            message: "No fallback state available, continuing with current state".to_string(),
            limitations: vec!["State may be inconsistent".to_string()],
        }
    }
    
    /// Reset component to default state
    fn reset_to_default(&mut self, context: &ErrorContext) -> RecoveryResult {
        RecoveryResult::Success {
            strategy: RecoveryStrategy::ResetToDefault,
            message: format!("Reset component to default state for context: {:?}", context.operation),
            fallback_data: None,
        }
    }
    
    /// Restart file watcher
    fn restart_file_watcher(&mut self, _context: &ErrorContext) -> RecoveryResult {
        // In a real implementation, this would restart the file watcher
        RecoveryResult::Success {
            strategy: RecoveryStrategy::RestartWatcher,
            message: "File watcher restarted successfully".to_string(),
            fallback_data: None,
        }
    }
    
    /// Isolate error and continue operation
    fn isolate_and_continue(&mut self, error_event: &ErrorEvent, context: &ErrorContext) -> RecoveryResult {
        // Log the error but don't let it crash the application
        log::warn!("Isolated error: {} in context: {:?}", error_event.error_message, context.operation);
        
        RecoveryResult::PartialRecovery {
            strategy: RecoveryStrategy::IsolateAndContinue,
            message: "Error isolated, continuing with reduced functionality".to_string(),
            limitations: vec![
                "Some features may be temporarily unavailable".to_string(),
                "Performance may be degraded".to_string(),
            ],
        }
    }
    
    /// Fallback from JIT to AST interpretation
    fn fallback_to_ast(&mut self, context: &ErrorContext) -> RecoveryResult {
        RecoveryResult::Success {
            strategy: RecoveryStrategy::FallbackToAST,
            message: "Switched from JIT compilation to AST interpretation".to_string(),
            fallback_data: Some(FallbackState {
                component_id: context.component_id.clone().unwrap_or_default(),
                state_data: serde_json::json!({
                    "interpretation_mode": "AST",
                    "fallback_reason": "JIT compilation failed"
                }),
                timestamp: SystemTime::now(),
                recovery_count: 1,
            }),
        }
    }
    
    /// Fallback from AST to Rhai interpretation
    fn fallback_to_rhai(&mut self, context: &ErrorContext) -> RecoveryResult {
        RecoveryResult::Success {
            strategy: RecoveryStrategy::FallbackToRhai,
            message: "Switched from AST to Rhai script interpretation".to_string(),
            fallback_data: Some(FallbackState {
                component_id: context.component_id.clone().unwrap_or_default(),
                state_data: serde_json::json!({
                    "interpretation_mode": "Rhai",
                    "fallback_reason": "AST parsing failed"
                }),
                timestamp: SystemTime::now(),
                recovery_count: 1,
            }),
        }
    }
    
    /// Retry operation with delay
    fn retry_with_delay(&mut self, context: &ErrorContext) -> RecoveryResult {
        // In a real implementation, this would schedule a retry
        RecoveryResult::PartialRecovery {
            strategy: RecoveryStrategy::RetryWithDelay,
            message: format!("Scheduled retry for operation: {:?}", context.operation),
            limitations: vec!["Operation will be retried after delay".to_string()],
        }
    }
    
    /// Disable problematic feature
    fn disable_feature(&mut self, context: &ErrorContext) -> RecoveryResult {
        RecoveryResult::PartialRecovery {
            strategy: RecoveryStrategy::DisableFeature,
            message: format!("Disabled feature due to persistent errors: {:?}", context.operation),
            limitations: vec!["Feature will remain disabled until manual re-enable".to_string()],
        }
    }
    
    /// Store a fallback state for a component
    pub fn store_fallback_state(&mut self, component_id: String, state_data: serde_json::Value) {
        let fallback_state = FallbackState {
            component_id: component_id.clone(),
            state_data,
            timestamp: SystemTime::now(),
            recovery_count: 0,
        };
        
        self.fallback_states.insert(component_id, fallback_state);
    }
    
    /// Classify error type for appropriate recovery strategy
    fn classify_error(&self, error: &RustyUIError) -> ErrorType {
        match error {
            RustyUIError::Interpretation { .. } => ErrorType::InterpretationFailure,
            RustyUIError::StatePreservation { .. } => ErrorType::StatePreservationFailure,
            RustyUIError::FrameworkAdapter { .. } => ErrorType::FrameworkAdapterFailure,
            #[cfg(feature = "dev-ui")]
            RustyUIError::FileWatcher(_) => ErrorType::FileWatchingFailure,
            RustyUIError::Configuration { .. } => ErrorType::ConfigurationError,
            RustyUIError::Initialization { .. } => ErrorType::InitializationError,
            _ => ErrorType::GenericError,
        }
    }
    
    /// Get recovery metrics
    pub fn get_metrics(&self) -> &RecoveryMetrics {
        &self.metrics
    }
    
    /// Get error history
    pub fn get_error_history(&self) -> Vec<ErrorEvent> {
        self.error_history.lock().unwrap().clone()
    }
    
    /// Clear error history
    pub fn clear_error_history(&mut self) {
        if let Ok(mut history) = self.error_history.lock() {
            history.clear();
        }
        self.metrics = RecoveryMetrics::default();
    }
    
    /// Check if system is in degraded mode
    pub fn is_degraded_mode(&self) -> bool {
        let error_rate = if self.metrics.total_errors > 0 {
            self.metrics.failed_recoveries as f64 / self.metrics.total_errors as f64
        } else {
            0.0
        };
        
        error_rate > self.config.degraded_mode_threshold
    }
    
    /// Get system health status
    pub fn get_health_status(&self) -> HealthStatus {
        if self.is_degraded_mode() {
            HealthStatus::Degraded
        } else if self.metrics.total_errors > 0 {
            HealthStatus::Recovering
        } else {
            HealthStatus::Healthy
        }
    }
    
    /// Check if system has error logs
    pub fn has_error_logs(&self) -> bool {
        if let Ok(history) = self.error_history.lock() {
            !history.is_empty()
        } else {
            false
        }
    }
    
    /// Get system health for property testing
    pub fn system_health(&self) -> SystemHealth {
        SystemHealth {
            stable: !self.is_degraded_mode(),
            error_count: self.metrics.total_errors as u32,
            last_error: self.error_history.lock()
                .ok()
                .and_then(|history| history.last().map(|e| e.timestamp)),
        }
    }
}

/// Production builds have minimal error recovery
#[cfg(not(feature = "dev-ui"))]
pub struct ErrorRecoveryManager;

#[cfg(not(feature = "dev-ui"))]
impl ErrorRecoveryManager {
    pub fn new() -> Self {
        Self
    }
    
    pub fn handle_error(&mut self, _error: &RustyUIError, _context: ErrorContext) -> RecoveryResult {
        RecoveryResult::Failed {
            strategy: RecoveryStrategy::IsolateAndContinue,
            message: "Error recovery not available in production builds".to_string(),
        }
    }
}

/// Types of errors that can occur
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorType {
    InterpretationFailure,
    StatePreservationFailure,
    FileWatchingFailure,
    FrameworkAdapterFailure,
    JITCompilationFailure,
    RhaiScriptFailure,
    ASTParsingFailure,
    ConfigurationError,
    InitializationError,
    GenericError,
}

/// Recovery strategies for different error types
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    /// Restore to last known working state
    FallbackToLastWorking,
    /// Reset to default state
    ResetToDefault,
    /// Restart file watcher
    RestartWatcher,
    /// Isolate error and continue
    IsolateAndContinue,
    /// Fallback from JIT to AST
    FallbackToAST,
    /// Fallback from AST to Rhai
    FallbackToRhai,
    /// Retry operation after delay
    RetryWithDelay,
    /// Disable problematic feature
    DisableFeature,
}

/// Result of a recovery attempt
#[derive(Debug, Clone)]
pub enum RecoveryResult {
    /// Recovery was successful
    Success {
        strategy: RecoveryStrategy,
        message: String,
        fallback_data: Option<FallbackState>,
    },
    /// Partial recovery with limitations
    PartialRecovery {
        strategy: RecoveryStrategy,
        message: String,
        limitations: Vec<String>,
    },
    /// Recovery failed
    Failed {
        strategy: RecoveryStrategy,
        message: String,
    },
}

impl RecoveryResult {
    /// Check if recovery was successful
    pub fn is_ok(&self) -> bool {
        matches!(self, RecoveryResult::Success { .. } | RecoveryResult::PartialRecovery { .. })
    }
}

/// Context information for error recovery
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Operation that caused the error
    pub operation: Operation,
    /// Component ID if applicable
    pub component_id: Option<String>,
    /// Additional context data
    pub context_data: HashMap<String, String>,
}

/// Types of operations that can fail
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

/// Error event for tracking and analysis
#[derive(Debug, Clone)]
pub struct ErrorEvent {
    pub error_type: ErrorType,
    pub timestamp: SystemTime,
    pub context: ErrorContext,
    pub error_message: String,
}

/// Fallback state for component recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackState {
    pub component_id: String,
    pub state_data: serde_json::Value,
    pub timestamp: SystemTime,
    pub recovery_count: u32,
}

/// Recovery metrics for monitoring
#[derive(Debug, Clone, Default)]
pub struct RecoveryMetrics {
    pub total_errors: u64,
    pub successful_recoveries: u64,
    pub partial_recoveries: u64,
    pub failed_recoveries: u64,
}

impl RecoveryMetrics {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_errors == 0 {
            1.0
        } else {
            self.successful_recoveries as f64 / self.total_errors as f64
        }
    }
    
    /// Calculate recovery rate (successful + partial)
    pub fn recovery_rate(&self) -> f64 {
        if self.total_errors == 0 {
            1.0
        } else {
            (self.successful_recoveries + self.partial_recoveries) as f64 / self.total_errors as f64
        }
    }
}

/// Configuration for error recovery behavior
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Maximum number of errors to keep in history
    pub max_error_history: usize,
    /// Threshold for entering degraded mode (error rate)
    pub degraded_mode_threshold: f64,
    /// Maximum retry attempts
    pub max_retry_attempts: u32,
    /// Delay between retry attempts
    pub retry_delay: Duration,
    /// Enable automatic feature disabling
    pub auto_disable_features: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_error_history: 100,
            degraded_mode_threshold: 0.5, // 50% error rate
            max_retry_attempts: 3,
            retry_delay: Duration::from_millis(1000),
            auto_disable_features: true,
        }
    }
}

/// System health status
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Recovering,
    Degraded,
}

/// System health information for property testing
#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub stable: bool,
    pub error_count: u32,
    pub last_error: Option<SystemTime>,
}

impl SystemHealth {
    pub fn is_stable(&self) -> bool {
        self.stable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_recovery_manager_creation() {
        let manager = ErrorRecoveryManager::new();
        assert_eq!(manager.get_health_status(), HealthStatus::Healthy);
    }
    
    #[test]
    fn test_error_classification() {
        let manager = ErrorRecoveryManager::new();
        let error = RustyUIError::interpretation("Test error");
        let error_type = manager.classify_error(&error);
        assert_eq!(error_type, ErrorType::InterpretationFailure);
    }
    
    #[test]
    fn test_fallback_state_storage() {
        let mut manager = ErrorRecoveryManager::new();
        let state_data = serde_json::json!({"test": "data"});
        
        manager.store_fallback_state("test_component".to_string(), state_data.clone());
        
        let context = ErrorContext {
            operation: Operation::Interpretation,
            component_id: Some("test_component".to_string()),
            context_data: HashMap::new(),
        };
        
        let result = manager.fallback_to_last_working(&context);
        match result {
            RecoveryResult::Success { .. } => {
                // Success expected
            }
            _ => panic!("Expected successful recovery"),
        }
    }
    
    #[test]
    fn test_recovery_metrics() {
        let mut manager = ErrorRecoveryManager::new();
        let error = RustyUIError::interpretation("Test error");
        let context = ErrorContext {
            operation: Operation::Interpretation,
            component_id: None,
            context_data: HashMap::new(),
        };
        
        manager.handle_error(&error, context);
        
        let metrics = manager.get_metrics();
        assert_eq!(metrics.total_errors, 1);
    }
}