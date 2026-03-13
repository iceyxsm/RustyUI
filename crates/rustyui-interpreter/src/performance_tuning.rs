//! Performance tuning and optimization for Phase 2 Enhanced JIT PGO
//! 
//! This module provides utilities for measuring and optimizing the performance
//! of the PGO system to meet the <5% overhead requirement.

use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};

/// Performance tuner for PGO system
pub struct PerformanceTuner {
    /// Baseline execution times (without PGO)
    baseline_times: Vec<Duration>,
    
    /// PGO execution times
    pgo_times: Vec<Duration>,
    
    /// Profiling overhead measurements
    profiling_overhead: AtomicU64,
    
    /// Total measurements taken
    measurement_count: AtomicU64,
}

impl PerformanceTuner {
    /// Create new performance tuner
    pub fn new() -> Self {
        Self {
            baseline_times: Vec::new(),
            pgo_times: Vec::new(),
            profiling_overhead: AtomicU64::new(0),
            measurement_count: AtomicU64::new(0),
        }
    }
    
    /// Measure baseline execution time (without PGO)
    pub fn measure_baseline<F, R>(&mut self, operation: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        
        self.baseline_times.push(duration);
        result
    }
    
    /// Measure PGO execution time
    pub fn measure_pgo<F, R>(&mut self, operation: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        
        self.pgo_times.push(duration);
        result
    }
    
    /// Calculate profiling overhead percentage
    pub fn calculate_overhead_percentage(&self) -> f64 {
        if self.baseline_times.is_empty() || self.pgo_times.is_empty() {
            return 0.0;
        }
        
        let avg_baseline = self.average_duration(&self.baseline_times);
        let avg_pgo = self.average_duration(&self.pgo_times);
        
        if avg_baseline.as_nanos() == 0 {
            return 0.0;
        }
        
        let overhead = avg_pgo.as_nanos() as f64 - avg_baseline.as_nanos() as f64;
        (overhead / avg_baseline.as_nanos() as f64) * 100.0
    }
    
    /// Get performance recommendations
    pub fn get_recommendations(&self) -> Vec<PerformanceRecommendation> {
        let mut recommendations = Vec::new();
        
        let overhead = self.calculate_overhead_percentage();
        
        if overhead > 5.0 {
            recommendations.push(PerformanceRecommendation {
                category: RecommendationCategory::ProfilingOverhead,
                severity: RecommendationSeverity::High,
                description: format!("Profiling overhead is {:.2}%, exceeds 5% target", overhead),
                suggestion: "Consider increasing sampling rate or reducing profiling frequency".to_string(),
            });
        }
        
        if overhead > 2.0 && overhead <= 5.0 {
            recommendations.push(PerformanceRecommendation {
                category: RecommendationCategory::ProfilingOverhead,
                severity: RecommendationSeverity::Medium,
                description: format!("Profiling overhead is {:.2}%, approaching 5% limit", overhead),
                suggestion: "Monitor overhead and consider tuning if it increases".to_string(),
            });
        }
        
        // Check for compilation time budget violations
        if let Some(max_pgo_time) = self.pgo_times.iter().max() {
            if max_pgo_time > &Duration::from_millis(100) {
                recommendations.push(PerformanceRecommendation {
                    category: RecommendationCategory::CompilationTime,
                    severity: RecommendationSeverity::High,
                    description: format!("Maximum compilation time is {:?}, exceeds 100ms budget", max_pgo_time),
                    suggestion: "Reduce optimization level or implement compilation timeout".to_string(),
                });
            }
        }
        
        recommendations
    }
    
    /// Tune sampling rate to achieve target overhead
    pub fn tune_sampling_rate(&self, current_rate: u32, target_overhead: f64) -> u32 {
        let current_overhead = self.calculate_overhead_percentage();
        
        if current_overhead <= target_overhead {
            return current_rate;
        }
        
        // Simple linear adjustment - in practice, this could be more sophisticated
        let adjustment_factor = current_overhead / target_overhead;
        let new_rate = (current_rate as f64 * adjustment_factor).ceil() as u32;
        
        // Clamp to reasonable bounds
        new_rate.max(1).min(1000)
    }
    
    /// Generate performance report
    pub fn generate_report(&self) -> PerformanceReport {
        PerformanceReport {
            overhead_percentage: self.calculate_overhead_percentage(),
            baseline_avg: self.average_duration(&self.baseline_times),
            pgo_avg: self.average_duration(&self.pgo_times),
            measurement_count: self.baseline_times.len().min(self.pgo_times.len()),
            recommendations: self.get_recommendations(),
        }
    }
    
    /// Calculate average duration
    fn average_duration(&self, durations: &[Duration]) -> Duration {
        if durations.is_empty() {
            return Duration::from_nanos(0);
        }
        
        let total_nanos: u64 = durations.iter().map(|d| d.as_nanos() as u64).sum();
        Duration::from_nanos(total_nanos / durations.len() as u64)
    }
}

/// Performance recommendation
#[derive(Debug, Clone)]
pub struct PerformanceRecommendation {
    pub category: RecommendationCategory,
    pub severity: RecommendationSeverity,
    pub description: String,
    pub suggestion: String,
}

/// Recommendation category
#[derive(Debug, Clone, PartialEq)]
pub enum RecommendationCategory {
    ProfilingOverhead,
    CompilationTime,
    MemoryUsage,
    TierThresholds,
}

/// Recommendation severity
#[derive(Debug, Clone, PartialEq)]
pub enum RecommendationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Performance report
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub overhead_percentage: f64,
    pub baseline_avg: Duration,
    pub pgo_avg: Duration,
    pub measurement_count: usize,
    pub recommendations: Vec<PerformanceRecommendation>,
}

impl PerformanceReport {
    /// Check if performance meets requirements
    pub fn meets_requirements(&self) -> bool {
        self.overhead_percentage <= 5.0 && 
        self.recommendations.iter().all(|r| r.severity != RecommendationSeverity::Critical)
    }
    
    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "Overhead: {:.2}%, Baseline: {:?}, PGO: {:?}, Measurements: {}, Recommendations: {}",
            self.overhead_percentage,
            self.baseline_avg,
            self.pgo_avg,
            self.measurement_count,
            self.recommendations.len()
        )
    }
}

/// Tier threshold optimizer
pub struct TierThresholdOptimizer {
    /// Execution count samples per tier
    tier_samples: std::collections::HashMap<crate::tiered_compilation::CompilationTier, Vec<u32>>,
    
    /// Performance improvements per tier
    tier_improvements: std::collections::HashMap<crate::tiered_compilation::CompilationTier, Vec<f64>>,
}

impl TierThresholdOptimizer {
    /// Create new optimizer
    pub fn new() -> Self {
        Self {
            tier_samples: std::collections::HashMap::new(),
            tier_improvements: std::collections::HashMap::new(),
        }
    }
    
    /// Record tier promotion data
    pub fn record_tier_promotion(
        &mut self,
        tier: crate::tiered_compilation::CompilationTier,
        execution_count: u32,
        improvement: f64,
    ) {
        self.tier_samples.entry(tier).or_insert_with(Vec::new).push(execution_count);
        self.tier_improvements.entry(tier).or_insert_with(Vec::new).push(improvement);
    }
    
    /// Optimize tier thresholds based on collected data
    pub fn optimize_thresholds(&self) -> crate::tiered_compilation::TieredCompilationConfig {
        let mut config = crate::tiered_compilation::TieredCompilationConfig::default();
        
        // Analyze tier 1 promotions
        if let Some(samples) = self.tier_samples.get(&crate::tiered_compilation::CompilationTier::QuickJIT) {
            if !samples.is_empty() {
                let avg = samples.iter().sum::<u32>() / samples.len() as u32;
                config.tier1_threshold = avg.max(5).min(50); // Reasonable bounds
            }
        }
        
        // Analyze tier 2 promotions
        if let Some(samples) = self.tier_samples.get(&crate::tiered_compilation::CompilationTier::OptimizedJIT) {
            if !samples.is_empty() {
                let avg = samples.iter().sum::<u32>() / samples.len() as u32;
                config.tier2_threshold = avg.max(config.tier1_threshold + 10).min(500);
            }
        }
        
        // Analyze tier 3 promotions
        if let Some(samples) = self.tier_samples.get(&crate::tiered_compilation::CompilationTier::AggressiveJIT) {
            if !samples.is_empty() {
                let avg = samples.iter().sum::<u32>() / samples.len() as u32;
                config.tier3_threshold = avg.max(config.tier2_threshold + 50).min(5000);
            }
        }
        
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_performance_tuner_overhead_calculation() {
        let mut tuner = PerformanceTuner::new();
        
        // Simulate baseline measurements
        for _ in 0..10 {
            tuner.measure_baseline(|| {
                thread::sleep(Duration::from_millis(1));
            });
        }
        
        // Simulate PGO measurements with overhead
        for _ in 0..10 {
            tuner.measure_pgo(|| {
                thread::sleep(Duration::from_millis(1));
                // Simulate profiling overhead
                thread::sleep(Duration::from_micros(50));
            });
        }
        
        let overhead = tuner.calculate_overhead_percentage();
        assert!(overhead > 0.0, "Should detect overhead");
        assert!(overhead < 50.0, "Overhead should be reasonable for test");
    }
    
    #[test]
    fn test_sampling_rate_tuning() {
        let mut tuner = PerformanceTuner::new();
        
        // Add some measurements that exceed target overhead
        tuner.baseline_times.push(Duration::from_millis(1));
        tuner.pgo_times.push(Duration::from_millis(2)); // 100% overhead
        
        let current_rate = 10;
        let target_overhead = 5.0;
        let new_rate = tuner.tune_sampling_rate(current_rate, target_overhead);
        
        assert!(new_rate > current_rate, "Should increase sampling rate to reduce overhead");
    }
    
    #[test]
    fn test_tier_threshold_optimizer() {
        let mut optimizer = TierThresholdOptimizer::new();
        
        // Record some tier promotion data
        optimizer.record_tier_promotion(
            crate::tiered_compilation::CompilationTier::QuickJIT,
            15,
            1.2
        );
        optimizer.record_tier_promotion(
            crate::tiered_compilation::CompilationTier::OptimizedJIT,
            120,
            1.8
        );
        
        let optimized_config = optimizer.optimize_thresholds();
        
        // Should adjust thresholds based on recorded data
        assert!(optimized_config.tier1_threshold >= 5);
        assert!(optimized_config.tier2_threshold > optimized_config.tier1_threshold);
    }
}