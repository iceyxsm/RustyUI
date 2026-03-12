//! Performance monitoring and metrics collection system
//! 
//! This module provides comprehensive performance monitoring capabilities for the dual-mode
//! engine, tracking interpretation times, memory usage, and validating against performance
//! targets as specified in requirements 7.1-7.4.

use std::time::{Duration, Instant, SystemTime};
use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};

/// Performance targets as defined in requirements 7.2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    /// Maximum interpretation time (Requirement 7.2: <100ms)
    pub max_interpretation_time_ms: u64,
    /// Maximum file change processing time (Requirement 7.2: <50ms)
    pub max_file_change_time_ms: u64,
    /// Maximum memory overhead in MB (Requirement 7.3)
    pub max_memory_overhead_mb: u64,
    /// Maximum JIT compilation time when needed
    pub max_jit_compilation_time_ms: u64,
    /// Maximum state preservation time
    pub max_state_preservation_time_ms: u64,
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            max_interpretation_time_ms: 100,  // Requirement 7.2
            max_file_change_time_ms: 50,      // Requirement 7.2
            max_memory_overhead_mb: 50,       // Reasonable default for Requirement 7.3
            max_jit_compilation_time_ms: 200, // Reasonable for JIT operations
            max_state_preservation_time_ms: 10, // Should be very fast
        }
    }
}

/// Individual performance measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMeasurement {
    pub operation: String,
    pub duration: Duration,
    pub timestamp: SystemTime,
    pub component_id: Option<String>,
    pub memory_usage_bytes: Option<u64>,
    pub success: bool,
    pub metadata: HashMap<String, String>,
}

/// Aggregated performance metrics for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total number of operations measured
    pub total_operations: u64,
    /// Average interpretation time across all operations
    pub average_interpretation_time: Duration,
    /// Maximum interpretation time recorded
    pub max_interpretation_time: Duration,
    /// Average file change processing time
    pub average_file_change_time: Duration,
    /// Current memory usage in bytes
    pub current_memory_usage_bytes: u64,
    /// Peak memory usage recorded
    pub peak_memory_usage_bytes: u64,
    /// Number of operations that exceeded targets
    pub target_violations: u64,
    /// Performance trend over time
    pub trend_analysis: TrendAnalysis,
    /// Last updated timestamp
    pub last_updated: SystemTime,
}

/// Performance trend analysis for development feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Whether performance is improving, stable, or degrading
    pub trend: PerformanceTrend,
    /// Percentage change from baseline
    pub change_percentage: f64,
    /// Number of samples used for trend calculation
    pub sample_count: usize,
    /// Confidence level of the trend analysis (0.0 to 1.0)
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Degrading,
    InsufficientData,
}

/// Performance monitoring system with real-time tracking
pub struct PerformanceMonitor {
    /// Performance targets for validation
    targets: PerformanceTargets,
    /// Recent measurements for trend analysis (limited size for memory efficiency)
    recent_measurements: VecDeque<PerformanceMeasurement>,
    /// Aggregated metrics
    metrics: PerformanceMetrics,
    /// Maximum number of measurements to keep in memory
    max_measurements: usize,
    /// Start time for overall session tracking
    session_start: Instant,
    /// Whether monitoring is enabled
    enabled: bool,
}

impl PerformanceMonitor {
    /// Create a new performance monitor with default targets
    pub fn new() -> Self {
        Self::with_targets(PerformanceTargets::default())
    }
    
    /// Create a new performance monitor with custom targets
    pub fn with_targets(targets: PerformanceTargets) -> Self {
        Self {
            targets,
            recent_measurements: VecDeque::new(),
            metrics: PerformanceMetrics {
                total_operations: 0,
                average_interpretation_time: Duration::from_millis(0),
                max_interpretation_time: Duration::from_millis(0),
                average_file_change_time: Duration::from_millis(0),
                current_memory_usage_bytes: 0,
                peak_memory_usage_bytes: 0,
                target_violations: 0,
                trend_analysis: TrendAnalysis {
                    trend: PerformanceTrend::InsufficientData,
                    change_percentage: 0.0,
                    sample_count: 0,
                    confidence: 0.0,
                },
                last_updated: SystemTime::now(),
            },
            max_measurements: 1000, // Keep last 1000 measurements
            session_start: Instant::now(),
            enabled: true,
        }
    }
    
    /// Enable or disable performance monitoring
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Check if monitoring is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Record a performance measurement
    pub fn record_measurement(&mut self, measurement: PerformanceMeasurement) {
        if !self.enabled {
            return;
        }
        
        // Check for target violations
        let violates_target = self.check_target_violation(&measurement);
        if violates_target {
            self.metrics.target_violations += 1;
        }
        
        // Update metrics
        self.update_metrics(&measurement);
        
        // Add to recent measurements (with size limit)
        self.recent_measurements.push_back(measurement);
        if self.recent_measurements.len() > self.max_measurements {
            self.recent_measurements.pop_front();
        }
        
        // Update trend analysis
        self.update_trend_analysis();
        
        self.metrics.last_updated = SystemTime::now();
    }
    
    /// Start timing an operation and return a timer
    pub fn start_timer(&self, operation: String) -> PerformanceTimer {
        PerformanceTimer::new(operation)
    }
    
    /// Record an interpretation operation timing
    pub fn record_interpretation(&mut self, duration: Duration, component_id: Option<String>, success: bool) {
        let measurement = PerformanceMeasurement {
            operation: "interpretation".to_string(),
            duration,
            timestamp: SystemTime::now(),
            component_id,
            memory_usage_bytes: None,
            success,
            metadata: HashMap::new(),
        };
        
        self.record_measurement(measurement);
    }
    
    /// Record a file change processing timing
    pub fn record_file_change(&mut self, duration: Duration, file_path: Option<String>, success: bool) {
        let mut metadata = HashMap::new();
        if let Some(path) = file_path {
            metadata.insert("file_path".to_string(), path);
        }
        
        let measurement = PerformanceMeasurement {
            operation: "file_change".to_string(),
            duration,
            timestamp: SystemTime::now(),
            component_id: None,
            memory_usage_bytes: None,
            success,
            metadata,
        };
        
        self.record_measurement(measurement);
    }
    
    /// Record memory usage measurement
    pub fn record_memory_usage(&mut self, memory_bytes: u64) {
        self.metrics.current_memory_usage_bytes = memory_bytes;
        if memory_bytes > self.metrics.peak_memory_usage_bytes {
            self.metrics.peak_memory_usage_bytes = memory_bytes;
        }
    }
    
    /// Get current performance metrics
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }
    
    /// Get performance targets
    pub fn get_targets(&self) -> &PerformanceTargets {
        &self.targets
    }
    
    /// Update performance targets
    pub fn update_targets(&mut self, targets: PerformanceTargets) {
        self.targets = targets;
    }
    
    /// Check if current performance meets all targets
    pub fn meets_performance_targets(&self) -> bool {
        let interpretation_ok = self.metrics.average_interpretation_time.as_millis() as u64 
            <= self.targets.max_interpretation_time_ms;
        let file_change_ok = self.metrics.average_file_change_time.as_millis() as u64 
            <= self.targets.max_file_change_time_ms;
        let memory_ok = (self.metrics.current_memory_usage_bytes / (1024 * 1024)) 
            <= self.targets.max_memory_overhead_mb;
        
        interpretation_ok && file_change_ok && memory_ok
    }
    
    /// Get performance violations summary
    pub fn get_violations_summary(&self) -> PerformanceViolationsSummary {
        let mut violations = Vec::new();
        
        if self.metrics.average_interpretation_time.as_millis() as u64 > self.targets.max_interpretation_time_ms {
            violations.push(format!(
                "Interpretation time: {}ms (target: {}ms)",
                self.metrics.average_interpretation_time.as_millis(),
                self.targets.max_interpretation_time_ms
            ));
        }
        
        if self.metrics.average_file_change_time.as_millis() as u64 > self.targets.max_file_change_time_ms {
            violations.push(format!(
                "File change processing: {}ms (target: {}ms)",
                self.metrics.average_file_change_time.as_millis(),
                self.targets.max_file_change_time_ms
            ));
        }
        
        let memory_mb = self.metrics.current_memory_usage_bytes / (1024 * 1024);
        if memory_mb > self.targets.max_memory_overhead_mb {
            violations.push(format!(
                "Memory usage: {}MB (target: {}MB)",
                memory_mb,
                self.targets.max_memory_overhead_mb
            ));
        }
        
        PerformanceViolationsSummary {
            total_violations: self.metrics.target_violations,
            current_violations: violations,
            meets_targets: self.meets_performance_targets(),
        }
    }
    
    /// Generate a performance report for development feedback
    pub fn generate_report(&self) -> PerformanceReport {
        PerformanceReport {
            session_duration: self.session_start.elapsed(),
            metrics: self.metrics.clone(),
            targets: self.targets.clone(),
            violations_summary: self.get_violations_summary(),
            recommendations: self.generate_recommendations(),
        }
    }
    
    /// Clear all measurements and reset metrics
    pub fn reset(&mut self) {
        self.recent_measurements.clear();
        self.metrics = PerformanceMetrics {
            total_operations: 0,
            average_interpretation_time: Duration::from_millis(0),
            max_interpretation_time: Duration::from_millis(0),
            average_file_change_time: Duration::from_millis(0),
            current_memory_usage_bytes: 0,
            peak_memory_usage_bytes: 0,
            target_violations: 0,
            trend_analysis: TrendAnalysis {
                trend: PerformanceTrend::InsufficientData,
                change_percentage: 0.0,
                sample_count: 0,
                confidence: 0.0,
            },
            last_updated: SystemTime::now(),
        };
        self.session_start = Instant::now();
    }
    
    /// Check if a measurement violates performance targets
    fn check_target_violation(&self, measurement: &PerformanceMeasurement) -> bool {
        match measurement.operation.as_str() {
            "interpretation" => {
                measurement.duration.as_millis() as u64 > self.targets.max_interpretation_time_ms
            }
            "file_change" => {
                measurement.duration.as_millis() as u64 > self.targets.max_file_change_time_ms
            }
            "jit_compilation" => {
                measurement.duration.as_millis() as u64 > self.targets.max_jit_compilation_time_ms
            }
            "state_preservation" => {
                measurement.duration.as_millis() as u64 > self.targets.max_state_preservation_time_ms
            }
            _ => false,
        }
    }
    
    /// Update aggregated metrics with new measurement
    fn update_metrics(&mut self, measurement: &PerformanceMeasurement) {
        self.metrics.total_operations += 1;
        
        // Update max interpretation time
        if measurement.duration > self.metrics.max_interpretation_time {
            self.metrics.max_interpretation_time = measurement.duration;
        }
        
        // Update averages based on operation type
        let total_ops = self.metrics.total_operations;
        match measurement.operation.as_str() {
            "interpretation" => {
                Self::update_average_duration_static(&mut self.metrics.average_interpretation_time, measurement.duration, total_ops);
            }
            "file_change" => {
                Self::update_average_duration_static(&mut self.metrics.average_file_change_time, measurement.duration, total_ops);
            }
            _ => {}
        }
    }
    
    /// Update running average duration (static version to avoid borrowing issues)
    fn update_average_duration_static(current_avg: &mut Duration, new_duration: Duration, total_ops: u64) {
        if total_ops == 1 {
            *current_avg = new_duration;
        } else {
            let current_total = current_avg.as_nanos() * (total_ops - 1) as u128;
            let new_total = current_total + new_duration.as_nanos();
            let new_avg_nanos = new_total / total_ops as u128;
            *current_avg = Duration::from_nanos(new_avg_nanos as u64);
        }
    }
    
    /// Update running average duration (kept for compatibility)
    fn _update_average_duration(&self, current_avg: &mut Duration, new_duration: Duration) {
        let total_ops = self.metrics.total_operations;
        Self::update_average_duration_static(current_avg, new_duration, total_ops);
    }
    
    /// Update trend analysis based on recent measurements
    fn update_trend_analysis(&mut self) {
        if self.recent_measurements.len() < 10 {
            self.metrics.trend_analysis.trend = PerformanceTrend::InsufficientData;
            return;
        }
        
        // Simple trend analysis: compare recent vs older measurements
        let recent_count = self.recent_measurements.len().min(20);
        let older_count = (self.recent_measurements.len() / 2).max(10);
        
        let recent_avg = self.calculate_average_duration(&self.recent_measurements, recent_count);
        let older_avg = self.calculate_average_duration(&self.recent_measurements, older_count);
        
        let change_ratio = recent_avg.as_nanos() as f64 / older_avg.as_nanos() as f64;
        let change_percentage = (change_ratio - 1.0) * 100.0;
        
        self.metrics.trend_analysis.change_percentage = change_percentage;
        self.metrics.trend_analysis.sample_count = recent_count;
        self.metrics.trend_analysis.confidence = (recent_count as f64 / 50.0).min(1.0);
        
        self.metrics.trend_analysis.trend = if change_percentage < -5.0 {
            PerformanceTrend::Improving
        } else if change_percentage > 5.0 {
            PerformanceTrend::Degrading
        } else {
            PerformanceTrend::Stable
        };
    }
    
    /// Calculate average duration for a subset of measurements
    fn calculate_average_duration(&self, measurements: &VecDeque<PerformanceMeasurement>, count: usize) -> Duration {
        let relevant_measurements: Vec<_> = measurements.iter().rev().take(count).collect();
        if relevant_measurements.is_empty() {
            return Duration::from_millis(0);
        }
        
        let total_nanos: u128 = relevant_measurements.iter()
            .map(|m| m.duration.as_nanos())
            .sum();
        
        Duration::from_nanos((total_nanos / relevant_measurements.len() as u128) as u64)
    }
    
    /// Generate performance improvement recommendations
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.metrics.average_interpretation_time.as_millis() as u64 > self.targets.max_interpretation_time_ms {
            recommendations.push("Consider optimizing interpretation logic or using JIT compilation for complex operations".to_string());
        }
        
        if self.metrics.average_file_change_time.as_millis() as u64 > self.targets.max_file_change_time_ms {
            recommendations.push("File change processing is slow - consider optimizing file watching or debouncing".to_string());
        }
        
        let memory_mb = self.metrics.current_memory_usage_bytes / (1024 * 1024);
        if memory_mb > self.targets.max_memory_overhead_mb {
            recommendations.push("Memory usage is high - consider reducing cache sizes or optimizing data structures".to_string());
        }
        
        match self.metrics.trend_analysis.trend {
            PerformanceTrend::Degrading => {
                recommendations.push("Performance is degrading over time - investigate recent changes".to_string());
            }
            PerformanceTrend::Improving => {
                recommendations.push("Performance is improving - current optimizations are working well".to_string());
            }
            _ => {}
        }
        
        if recommendations.is_empty() {
            recommendations.push("Performance is within targets - no immediate action needed".to_string());
        }
        
        recommendations
    }
}

/// Timer for measuring operation performance
pub struct PerformanceTimer {
    operation: String,
    start_time: Instant,
    metadata: HashMap<String, String>,
}

impl PerformanceTimer {
    /// Create a new performance timer
    pub fn new(operation: String) -> Self {
        Self {
            operation,
            start_time: Instant::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the timer
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Finish timing and create a measurement
    pub fn finish(self, success: bool) -> PerformanceMeasurement {
        PerformanceMeasurement {
            operation: self.operation,
            duration: self.start_time.elapsed(),
            timestamp: SystemTime::now(),
            component_id: None,
            memory_usage_bytes: None,
            success,
            metadata: self.metadata,
        }
    }
    
    /// Finish timing with component ID
    pub fn finish_with_component(self, success: bool, component_id: String) -> PerformanceMeasurement {
        PerformanceMeasurement {
            operation: self.operation,
            duration: self.start_time.elapsed(),
            timestamp: SystemTime::now(),
            component_id: Some(component_id),
            memory_usage_bytes: None,
            success,
            metadata: self.metadata,
        }
    }
}

/// Summary of performance target violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceViolationsSummary {
    pub total_violations: u64,
    pub current_violations: Vec<String>,
    pub meets_targets: bool,
}

/// Comprehensive performance report for development feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub session_duration: Duration,
    pub metrics: PerformanceMetrics,
    pub targets: PerformanceTargets,
    pub violations_summary: PerformanceViolationsSummary,
    pub recommendations: Vec<String>,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        assert!(monitor.is_enabled());
        assert_eq!(monitor.get_metrics().total_operations, 0);
    }
    
    #[test]
    fn test_performance_measurement_recording() {
        let mut monitor = PerformanceMonitor::new();
        
        let measurement = PerformanceMeasurement {
            operation: "test".to_string(),
            duration: Duration::from_millis(50),
            timestamp: SystemTime::now(),
            component_id: None,
            memory_usage_bytes: None,
            success: true,
            metadata: HashMap::new(),
        };
        
        monitor.record_measurement(measurement);
        assert_eq!(monitor.get_metrics().total_operations, 1);
    }
    
    #[test]
    fn test_performance_timer() {
        let timer = PerformanceTimer::new("test_operation".to_string());
        std::thread::sleep(Duration::from_millis(1));
        let measurement = timer.finish(true);
        
        assert_eq!(measurement.operation, "test_operation");
        assert!(measurement.duration.as_millis() >= 1);
        assert!(measurement.success);
    }
    
    #[test]
    fn test_target_violation_detection() {
        let mut monitor = PerformanceMonitor::with_targets(PerformanceTargets {
            max_interpretation_time_ms: 10,
            ..Default::default()
        });
        
        // Record a measurement that violates the target
        monitor.record_interpretation(Duration::from_millis(20), None, true);
        
        assert_eq!(monitor.get_metrics().target_violations, 1);
        assert!(!monitor.meets_performance_targets());
    }
    
    #[test]
    fn test_memory_usage_tracking() {
        let mut monitor = PerformanceMonitor::new();
        
        monitor.record_memory_usage(1024 * 1024); // 1MB
        assert_eq!(monitor.get_metrics().current_memory_usage_bytes, 1024 * 1024);
        assert_eq!(monitor.get_metrics().peak_memory_usage_bytes, 1024 * 1024);
        
        monitor.record_memory_usage(2 * 1024 * 1024); // 2MB
        assert_eq!(monitor.get_metrics().current_memory_usage_bytes, 2 * 1024 * 1024);
        assert_eq!(monitor.get_metrics().peak_memory_usage_bytes, 2 * 1024 * 1024);
        
        monitor.record_memory_usage(512 * 1024); // 512KB
        assert_eq!(monitor.get_metrics().current_memory_usage_bytes, 512 * 1024);
        assert_eq!(monitor.get_metrics().peak_memory_usage_bytes, 2 * 1024 * 1024); // Peak remains
    }
}