//! Error Reporting and Logging System for RustyUI
//! 
//! Provides comprehensive error reporting, logging, and metrics collection
//! for development feedback and debugging.

use crate::error::RustyUIError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Comprehensive error reporting system
#[cfg(feature = "dev-ui")]
pub struct ErrorReporter {
    /// Error logs with detailed context
    error_logs: Arc<Mutex<Vec<ErrorLog>>>,
    
    /// Error metrics for analysis
    metrics: ErrorMetrics,
    
    /// Configuration for reporting behavior
    config: ReportingConfig,
    
    /// Error patterns for intelligent analysis
    error_patterns: HashMap<String, ErrorPattern>,
}

#[cfg(feature = "dev-ui")]
impl ErrorReporter {
    /// Create a new error reporter
    pub fn new() -> Self {
        Self {
            error_logs: Arc::new(Mutex::new(Vec::new())),
            metrics: ErrorMetrics::default(),
            config: ReportingConfig::default(),
            error_patterns: HashMap::new(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: ReportingConfig) -> Self {
        let mut reporter = Self::new();
        reporter.config = config;
        reporter
    }
    
    /// Report an error with full context
    pub fn report_error(&mut self, error: &RustyUIError, context: ErrorReportContext) {
        let error_log = ErrorLog {
            timestamp: SystemTime::now(),
            error_type: self.classify_error_type(error),
            error_message: error.to_string(),
            context: context.clone(),
            stack_trace: self.capture_stack_trace(),
            severity: self.determine_severity(error, &context),
        };
        
        // Update metrics
        self.update_metrics(&error_log);
        
        // Store error log
        if let Ok(mut logs) = self.error_logs.lock() {
            logs.push(error_log.clone());
            
            // Limit log size
            if logs.len() > self.config.max_error_logs {
                logs.remove(0);
            }
        }
        
        // Analyze error patterns
        self.analyze_error_pattern(&error_log);
        
        // Output error based on configuration
        self.output_error(&error_log);
    }
    
    /// Classify error type for reporting
    fn classify_error_type(&self, error: &RustyUIError) -> ErrorType {
        match error {
            RustyUIError::Interpretation { .. } => ErrorType::Interpretation,
            RustyUIError::StatePreservation { .. } => ErrorType::StatePreservation,
            RustyUIError::FrameworkAdapter { .. } => ErrorType::FrameworkAdapter,
            #[cfg(feature = "dev-ui")]
            RustyUIError::FileWatcher(_) => ErrorType::FileWatching,
            RustyUIError::Configuration { .. } => ErrorType::Configuration,
            RustyUIError::Initialization { .. } => ErrorType::Initialization,
            _ => ErrorType::Generic,
        }
    }
    
    /// Determine error severity
    fn determine_severity(&self, error: &RustyUIError, context: &ErrorReportContext) -> ErrorSeverity {
        match error {
            RustyUIError::Initialization { .. } => ErrorSeverity::Critical,
            RustyUIError::Configuration { .. } => ErrorSeverity::High,
            RustyUIError::Interpretation { .. } => {
                if context.affects_core_functionality {
                    ErrorSeverity::High
                } else {
                    ErrorSeverity::Medium
                }
            }
            RustyUIError::StatePreservation { .. } => ErrorSeverity::Medium,
            RustyUIError::FrameworkAdapter { .. } => ErrorSeverity::Medium,
            #[cfg(feature = "dev-ui")]
            RustyUIError::FileWatcher(_) => ErrorSeverity::Low,
            _ => ErrorSeverity::Low,
        }
    }
    
    /// Capture stack trace for debugging
    fn capture_stack_trace(&self) -> Option<String> {
        if self.config.capture_stack_traces {
            // In a real implementation, this would capture the actual stack trace
            Some("Stack trace capture not implemented in Phase 1".to_string())
        } else {
            None
        }
    }
    
    /// Update error metrics
    fn update_metrics(&mut self, error_log: &ErrorLog) {
        self.metrics.total_errors += 1;
        
        match error_log.severity {
            ErrorSeverity::Critical => self.metrics.critical_errors += 1,
            ErrorSeverity::High => self.metrics.high_errors += 1,
            ErrorSeverity::Medium => self.metrics.medium_errors += 1,
            ErrorSeverity::Low => self.metrics.low_errors += 1,
        }
        
        // Update error type counts
        *self.metrics.error_type_counts.entry(error_log.error_type.clone()).or_insert(0) += 1;
        
        // Update component error counts
        if let Some(ref component_id) = error_log.context.component_id {
            *self.metrics.component_error_counts.entry(component_id.clone()).or_insert(0) += 1;
        }
    }
    
    /// Analyze error patterns for intelligent reporting
    fn analyze_error_pattern(&mut self, error_log: &ErrorLog) {
        let pattern_key = format!("{}:{}", error_log.error_type.as_str(), 
                                 error_log.context.operation.as_str());
        
        // Check if we need to generate fixes before modifying the pattern
        let should_generate_fixes = {
            if let Some(existing_pattern) = self.error_patterns.get(&pattern_key) {
                existing_pattern.occurrence_count >= 2 && existing_pattern.suggested_fixes.is_empty()
            } else {
                false
            }
        };
        
        // Generate fixes if needed
        let new_fixes = if should_generate_fixes {
            self.generate_suggested_fixes_for_pattern(&error_log.error_type, &error_log.context.operation)
        } else {
            Vec::new()
        };
        
        // Now modify the pattern
        let pattern = self.error_patterns.entry(pattern_key.clone()).or_insert_with(|| {
            ErrorPattern {
                pattern_id: pattern_key,
                error_type: error_log.error_type.clone(),
                operation: error_log.context.operation.clone(),
                occurrence_count: 0,
                first_seen: error_log.timestamp,
                last_seen: error_log.timestamp,
                suggested_fixes: Vec::new(),
            }
        });
        
        pattern.occurrence_count += 1;
        pattern.last_seen = error_log.timestamp;
        
        // Apply fixes if we generated them
        if !new_fixes.is_empty() {
            pattern.suggested_fixes = new_fixes;
        }
    }
    
    /// Generate suggested fixes for error patterns
    fn generate_suggested_fixes_for_pattern(&self, error_type: &ErrorType, operation: &ErrorOperation) -> Vec<String> {
        match (error_type, operation) {
            (ErrorType::Interpretation, _) => {
                vec![
                    "Try using a different interpretation strategy".to_string(),
                    "Check code syntax for errors".to_string(),
                ]
            }
            (ErrorType::FileWatching, _) => {
                vec![
                    "Check file permissions".to_string(),
                    "Verify file paths exist".to_string(),
                ]
            }
            (ErrorType::StatePreservation, _) => {
                vec![
                    "Ensure state data is serializable".to_string(),
                    "Check for circular references in state".to_string(),
                ]
            }
            _ => {
                vec!["Check system logs for more details".to_string()]
            }
        }
    }
    
    /// Output error based on configuration
    fn output_error(&self, error_log: &ErrorLog) {
        match self.config.output_format {
            OutputFormat::Console => self.output_to_console(error_log),
            OutputFormat::Json => self.output_to_json(error_log),
            OutputFormat::Structured => self.output_structured(error_log),
        }
    }
    
    /// Output error to console
    fn output_to_console(&self, error_log: &ErrorLog) {
        let severity_icon = match error_log.severity {
            ErrorSeverity::Critical => "",
            ErrorSeverity::High => "",
            ErrorSeverity::Medium => "",
            ErrorSeverity::Low => "",
        };
        
        println!("{} {} Error: {}", 
                severity_icon, 
                error_log.error_type.as_str(), 
                error_log.error_message);
        
        if let Some(ref component_id) = error_log.context.component_id {
            println!("Component: {}", component_id);
        }
        
        println!("Operation: {}", error_log.context.operation.as_str());
        
        if self.config.show_suggestions {
            self.show_suggestions_for_error(error_log);
        }
    }
    
    /// Output error as JSON
    fn output_to_json(&self, error_log: &ErrorLog) {
        if let Ok(json) = serde_json::to_string_pretty(error_log) {
            println!("{}", json);
        }
    }
    
    /// Output structured error information
    fn output_structured(&self, error_log: &ErrorLog) {
        println!("┌─ Error Report ─────────────────────────────────────");
        println!("│ Type: {}", error_log.error_type.as_str());
        println!("│ Severity: {:?}", error_log.severity);
        println!("│ Message: {}", error_log.error_message);
        println!("│ Operation: {}", error_log.context.operation.as_str());
        
        if let Some(ref component_id) = error_log.context.component_id {
            println!("│ Component: {}", component_id);
        }
        
        if let Some(ref stack_trace) = error_log.stack_trace {
            println!("│ Stack Trace: {}", stack_trace);
        }
        
        println!("└────────────────────────────────────────────────────");
        
        if self.config.show_suggestions {
            self.show_suggestions_for_error(error_log);
        }
    }
    
    /// Show suggestions for fixing the error
    fn show_suggestions_for_error(&self, error_log: &ErrorLog) {
        let pattern_key = format!("{}:{}", error_log.error_type.as_str(), 
                                 error_log.context.operation.as_str());
        
        if let Some(pattern) = self.error_patterns.get(&pattern_key) {
            if !pattern.suggested_fixes.is_empty() {
                println!("Suggestions:");
                for (i, fix) in pattern.suggested_fixes.iter().enumerate() {
                    println!("{}. {}", i + 1, fix);
                }
            }
        }
    }
    
    /// Get error metrics
    pub fn get_metrics(&self) -> &ErrorMetrics {
        &self.metrics
    }
    
    /// Get error logs
    pub fn get_error_logs(&self) -> Vec<ErrorLog> {
        self.error_logs.lock().unwrap().clone()
    }
    
    /// Get error patterns
    pub fn get_error_patterns(&self) -> Vec<ErrorPattern> {
        self.error_patterns.values().cloned().collect()
    }
    
    /// Clear error logs and reset metrics
    pub fn clear_logs(&mut self) {
        if let Ok(mut logs) = self.error_logs.lock() {
            logs.clear();
        }
        self.metrics = ErrorMetrics::default();
        self.error_patterns.clear();
    }
    
    /// Generate error report
    pub fn generate_report(&self) -> ErrorReport {
        ErrorReport {
            timestamp: SystemTime::now(),
            metrics: self.metrics.clone(),
            recent_errors: self.get_recent_errors(10),
            error_patterns: self.get_error_patterns(),
            recommendations: self.generate_recommendations(),
        }
    }
    
    /// Get recent errors
    fn get_recent_errors(&self, count: usize) -> Vec<ErrorLog> {
        if let Ok(logs) = self.error_logs.lock() {
            logs.iter()
                .rev()
                .take(count)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Generate recommendations based on error patterns
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Analyze error frequency
        if self.metrics.total_errors > 50 {
            recommendations.push("Consider reviewing code quality - high error frequency detected".to_string());
        }
        
        // Analyze error types
        if self.metrics.critical_errors > 0 {
            recommendations.push("Address critical errors immediately to prevent system instability".to_string());
        }
        
        // Analyze component errors
        let problematic_components: Vec<_> = self.metrics.component_error_counts
            .iter()
            .filter(|(_, &count)| count > 5)
            .map(|(component, count)| format!("{} ({} errors)", component, count))
            .collect();
        
        if !problematic_components.is_empty() {
            recommendations.push(format!("Review these components with frequent errors: {}", 
                                       problematic_components.join(",")));
        }
        
        recommendations
    }
}

/// Production builds have minimal error reporting
#[cfg(not(feature = "dev-ui"))]
pub struct ErrorReporter;

#[cfg(not(feature = "dev-ui"))]
impl ErrorReporter {
    pub fn new() -> Self {
        Self
    }
    
    pub fn report_error(&mut self, _error: &RustyUIError, _context: ErrorReportContext) {
        // Minimal error reporting in production
    }
}

/// Error log entry with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLog {
    pub timestamp: SystemTime,
    pub error_type: ErrorType,
    pub error_message: String,
    pub context: ErrorReportContext,
    pub stack_trace: Option<String>,
    pub severity: ErrorSeverity,
}

/// Context information for error reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReportContext {
    pub operation: ErrorOperation,
    pub component_id: Option<String>,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub affects_core_functionality: bool,
    pub user_action: Option<String>,
    pub system_state: HashMap<String, String>,
}

/// Types of errors for classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorType {
    Interpretation,
    StatePreservation,
    FileWatching,
    FrameworkAdapter,
    Configuration,
    Initialization,
    Generic,
}

impl ErrorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorType::Interpretation => "Interpretation",
            ErrorType::StatePreservation => "StatePreservation",
            ErrorType::FileWatching => "FileWatching",
            ErrorType::FrameworkAdapter => "FrameworkAdapter",
            ErrorType::Configuration => "Configuration",
            ErrorType::Initialization => "Initialization",
            ErrorType::Generic => "Generic",
        }
    }
}

/// Operations that can generate errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorOperation {
    RhaiInterpretation,
    ASTInterpretation,
    JITCompilation,
    StatePreservation,
    StateRestoration,
    FileWatching,
    ComponentRendering,
    FrameworkIntegration,
    SystemInitialization,
}

impl ErrorOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorOperation::RhaiInterpretation => "RhaiInterpretation",
            ErrorOperation::ASTInterpretation => "ASTInterpretation",
            ErrorOperation::JITCompilation => "JITCompilation",
            ErrorOperation::StatePreservation => "StatePreservation",
            ErrorOperation::StateRestoration => "StateRestoration",
            ErrorOperation::FileWatching => "FileWatching",
            ErrorOperation::ComponentRendering => "ComponentRendering",
            ErrorOperation::FrameworkIntegration => "FrameworkIntegration",
            ErrorOperation::SystemInitialization => "SystemInitialization",
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Critical,  // System-breaking errors
    High,      // Major functionality affected
    Medium,    // Minor functionality affected
    Low,       // Cosmetic or non-critical issues
}

/// Error metrics for analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub critical_errors: u64,
    pub high_errors: u64,
    pub medium_errors: u64,
    pub low_errors: u64,
    pub error_type_counts: HashMap<ErrorType, u64>,
    pub component_error_counts: HashMap<String, u64>,
}

/// Error pattern for intelligent analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub pattern_id: String,
    pub error_type: ErrorType,
    pub operation: ErrorOperation,
    pub occurrence_count: u64,
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
    pub suggested_fixes: Vec<String>,
}

/// Configuration for error reporting
#[derive(Debug, Clone)]
pub struct ReportingConfig {
    pub max_error_logs: usize,
    pub output_format: OutputFormat,
    pub capture_stack_traces: bool,
    pub show_suggestions: bool,
    pub auto_generate_reports: bool,
    pub report_interval: Duration,
}

impl Default for ReportingConfig {
    fn default() -> Self {
        Self {
            max_error_logs: 1000,
            output_format: OutputFormat::Structured,
            capture_stack_traces: true,
            show_suggestions: true,
            auto_generate_reports: false,
            report_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Output format for error reporting
#[derive(Debug, Clone)]
pub enum OutputFormat {
    Console,
    Json,
    Structured,
}

/// Comprehensive error report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    pub timestamp: SystemTime,
    pub metrics: ErrorMetrics,
    pub recent_errors: Vec<ErrorLog>,
    pub error_patterns: Vec<ErrorPattern>,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::RustyUIError;
    
    #[test]
    fn test_error_reporter_creation() {
        let reporter = ErrorReporter::new();
        assert_eq!(reporter.get_metrics().total_errors, 0);
    }
    
    #[test]
    fn test_error_reporting() {
        let mut reporter = ErrorReporter::new();
        let error = RustyUIError::interpretation("Test error");
        let context = ErrorReportContext {
            operation: ErrorOperation::RhaiInterpretation,
            component_id: Some("test_component".to_string()),
            file_path: None,
            line_number: None,
            affects_core_functionality: false,
            user_action: None,
            system_state: HashMap::new(),
        };
        
        reporter.report_error(&error, context);
        
        assert_eq!(reporter.get_metrics().total_errors, 1);
        assert_eq!(reporter.get_error_logs().len(), 1);
    }
    
    #[test]
    fn test_error_pattern_analysis() {
        let mut reporter = ErrorReporter::new();
        let error = RustyUIError::interpretation("Test error");
        let context = ErrorReportContext {
            operation: ErrorOperation::RhaiInterpretation,
            component_id: Some("test_component".to_string()),
            file_path: None,
            line_number: None,
            affects_core_functionality: false,
            user_action: None,
            system_state: HashMap::new(),
        };
        
        // Report the same error multiple times to trigger pattern analysis
        for _ in 0..5 {
            reporter.report_error(&error, context.clone());
        }
        
        let patterns = reporter.get_error_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns[0].occurrence_count >= 3);
        assert!(!patterns[0].suggested_fixes.is_empty());
    }
}