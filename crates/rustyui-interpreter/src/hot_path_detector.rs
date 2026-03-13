//! Hot path detection for profile-guided optimization
//! 
//! Based on 2024-2026 industry best practices:
//! - Priority-based hot path scoring
//! - Configurable thresholds for optimization candidates
//! - Cached analysis results with TTL
//! - Loop and call site detection algorithms

use crate::profiling::{ProfilingInfrastructure, ProfileData};
use crate::tiered_compilation::CompilationTier;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Hot function information
#[derive(Debug, Clone)]
pub struct HotFunction {
    /// Function identifier
    pub function_id: String,
    
    /// Current execution count
    pub execution_count: u64,
    
    /// Total execution time
    pub total_time: Duration,
    
    /// Current compilation tier
    pub current_tier: CompilationTier,
    
    /// Recommended next tier
    pub recommended_tier: CompilationTier,
    
    /// Priority score for optimization
    pub priority_score: f64,
}

/// Hot loop information
#[derive(Debug, Clone)]
pub struct HotLoop {
    /// Loop identifier within function
    pub loop_id: u32,
    
    /// Loop execution count
    pub execution_count: u64,
    
    /// Average iteration count
    pub avg_iterations: f64,
    
    /// Optimization opportunities
    pub optimization_opportunities: Vec<LoopOptimization>,
}

/// Loop optimization opportunities
#[derive(Debug, Clone)]
pub enum LoopOptimization {
    /// Unroll loop with predictable iteration count
    Unrolling { factor: u32 },
    
    /// Vectorize loop operations
    Vectorization,
    
    /// Hoist invariant computations
    LoopInvariantCodeMotion,
    
    /// Strength reduction for induction variables
    StrengthReduction,
}

/// Hot call site for inlining
#[derive(Debug, Clone)]
pub struct HotCallSite {
    /// Call site identifier within function
    pub call_site_id: u32,
    
    /// Total call count
    pub call_count: u64,
    
    /// Target function name
    pub target_function: String,
    
    /// Whether call site is monomorphic (single target)
    pub is_monomorphic: bool,
    
    /// Inlining benefit score
    pub inline_benefit_score: f64,
}

/// Configuration for hot path detection
#[derive(Debug, Clone)]
pub struct HotPathConfig {
    /// Minimum execution count to consider function hot
    pub min_execution_count: u64,
    
    /// Minimum priority score for optimization
    pub min_priority_score: f64,
    
    /// Hot path cache TTL
    pub cache_ttl: Duration,
    
    /// Reanalysis interval
    pub reanalysis_interval: Duration,
    
    /// Loop hot threshold
    pub loop_hot_threshold: u64,
    
    /// Call site hot threshold
    pub call_site_hot_threshold: u64,
    
    /// Minimum inline benefit score
    pub min_inline_benefit_score: f64,
}

impl Default for HotPathConfig {
    fn default() -> Self {
        Self {
            min_execution_count: 10,
            min_priority_score: 100.0,
            cache_ttl: Duration::from_secs(60), // 1 minute
            reanalysis_interval: Duration::from_secs(30), // 30 seconds
            loop_hot_threshold: 50,
            call_site_hot_threshold: 20,
            min_inline_benefit_score: 50.0,
        }
    }
}

/// Cached hot path analysis results
#[derive(Debug, Clone)]
pub struct HotPathCache {
    /// Hot functions cache
    hot_functions: HashMap<String, (HotFunction, Instant)>,
    
    /// Hot loops cache
    hot_loops: HashMap<String, (Vec<HotLoop>, Instant)>,
    
    /// Hot call sites cache
    hot_call_sites: HashMap<String, (Vec<HotCallSite>, Instant)>,
    
    /// Last analysis timestamp
    last_analysis: Instant,
}

impl HotPathCache {
    /// Create new cache
    pub fn new() -> Self {
        Self {
            hot_functions: HashMap::new(),
            hot_loops: HashMap::new(),
            hot_call_sites: HashMap::new(),
            last_analysis: Instant::now(),
        }
    }
    
    /// Check if cache entry is expired
    fn is_expired(&self, timestamp: Instant, ttl: Duration) -> bool {
        timestamp.elapsed() > ttl
    }
    
    /// Clean expired entries
    pub fn clean_expired(&mut self, ttl: Duration) {
        let now = Instant::now();
        
        self.hot_functions.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp) <= ttl
        });
        
        self.hot_loops.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp) <= ttl
        });
        
        self.hot_call_sites.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp) <= ttl
        });
    }
}

/// Hot path detector for identifying optimization candidates
pub struct HotPathDetector {
    /// Reference to profiling infrastructure
    profiling: Arc<ProfilingInfrastructure>,
    
    /// Hot path detection configuration
    config: HotPathConfig,
    
    /// Cached hot path analysis results
    hot_paths: Arc<RwLock<HotPathCache>>,
}

impl HotPathDetector {
    /// Create new hot path detector
    pub fn new(profiling: Arc<ProfilingInfrastructure>, config: HotPathConfig) -> Self {
        Self {
            profiling,
            config,
            hot_paths: Arc::new(RwLock::new(HotPathCache::new())),
        }
    }
    
    /// Create with default configuration
    pub fn with_defaults(profiling: Arc<ProfilingInfrastructure>) -> Self {
        Self::new(profiling, HotPathConfig::default())
    }
    
    /// Identify hot functions based on execution count and timing
    pub fn detect_hot_functions(&self) -> Vec<HotFunction> {
        let mut hot_functions = Vec::new();
        
        // Get all profiles from profiling infrastructure
        let profiles = self.profiling.get_all_profiles();
        
        for (function_id, profile) in profiles {
            let execution_count = profile.get_execution_count();
            
            // Skip functions that don't meet minimum execution threshold
            if execution_count < self.config.min_execution_count {
                continue;
            }
            
            let avg_execution_time = profile.get_average_execution_time();
            let priority_score = self.calculate_priority(&function_id, execution_count, avg_execution_time);
            
            // Skip functions that don't meet minimum priority score
            if priority_score < self.config.min_priority_score {
                continue;
            }
            
            // Determine current and recommended tier
            let current_tier = CompilationTier::Interpreter; // TODO: Get from tiered compilation manager
            let recommended_tier = self.recommend_tier(execution_count, priority_score);
            
            let hot_function = HotFunction {
                function_id: function_id.clone(),
                execution_count,
                total_time: Duration::from_nanos((avg_execution_time.as_nanos() as u64) * execution_count),
                current_tier,
                recommended_tier,
                priority_score,
            };
            
            hot_functions.push(hot_function);
        }
        
        // Sort by priority score (highest first)
        hot_functions.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Cache results
        {
            let mut cache = self.hot_paths.write().unwrap();
            cache.clean_expired(self.config.cache_ttl);
            
            for hot_function in &hot_functions {
                cache.hot_functions.insert(
                    hot_function.function_id.clone(),
                    (hot_function.clone(), Instant::now())
                );
            }
        }
        
        hot_functions
    }
    
    /// Identify hot loops within functions
    pub fn detect_hot_loops(&self, function_id: &str) -> Vec<HotLoop> {
        let mut hot_loops = Vec::new();
        
        // Check cache first
        {
            let cache = self.hot_paths.read().unwrap();
            if let Some((cached_loops, timestamp)) = cache.hot_loops.get(function_id) {
                if !cache.is_expired(*timestamp, self.config.cache_ttl) {
                    return cached_loops.clone();
                }
            }
        }
        
        // Get profile data for function
        if let Some(profile) = self.profiling.get_profile(function_id) {
            // Analyze loop statistics
            for entry in profile.loop_stats.iter() {
                let loop_id = *entry.key();
                let stats = entry.value();
                
                let execution_count = stats.execution_count.load(std::sync::atomic::Ordering::Relaxed);
                
                // Skip loops that don't meet hot threshold
                if execution_count < self.config.loop_hot_threshold {
                    continue;
                }
                
                let avg_iterations = stats.average_iterations();
                let optimization_opportunities = self.identify_loop_optimizations(&stats);
                
                let hot_loop = HotLoop {
                    loop_id,
                    execution_count,
                    avg_iterations,
                    optimization_opportunities,
                };
                
                hot_loops.push(hot_loop);
            }
        }
        
        // Sort by execution count (highest first)
        hot_loops.sort_by(|a, b| b.execution_count.cmp(&a.execution_count));
        
        // Cache results
        {
            let mut cache = self.hot_paths.write().unwrap();
            cache.hot_loops.insert(function_id.to_string(), (hot_loops.clone(), Instant::now()));
        }
        
        hot_loops
    }
    
    /// Identify hot call sites for inlining
    pub fn detect_hot_call_sites(&self, function_id: &str) -> Vec<HotCallSite> {
        let mut hot_call_sites = Vec::new();
        
        // Check cache first
        {
            let cache = self.hot_paths.read().unwrap();
            if let Some((cached_sites, timestamp)) = cache.hot_call_sites.get(function_id) {
                if !cache.is_expired(*timestamp, self.config.cache_ttl) {
                    return cached_sites.clone();
                }
            }
        }
        
        // Get profile data for function
        if let Some(profile) = self.profiling.get_profile(function_id) {
            // Analyze call site statistics
            for entry in profile.call_site_stats.iter() {
                let call_site_id = *entry.key();
                let stats = entry.value();
                
                let call_count = stats.call_count.load(std::sync::atomic::Ordering::Relaxed);
                
                // Skip call sites that don't meet hot threshold
                if call_count < self.config.call_site_hot_threshold {
                    continue;
                }
                
                let is_monomorphic = stats.is_monomorphic();
                let target_function = stats.hot_target().unwrap_or_else(|| "unknown".to_string());
                let inline_benefit_score = self.calculate_inline_benefit_score(call_count, is_monomorphic);
                
                // Skip call sites with low inline benefit
                if inline_benefit_score < self.config.min_inline_benefit_score {
                    continue;
                }
                
                let hot_call_site = HotCallSite {
                    call_site_id,
                    call_count,
                    target_function,
                    is_monomorphic,
                    inline_benefit_score,
                };
                
                hot_call_sites.push(hot_call_site);
            }
        }
        
        // Sort by inline benefit score (highest first)
        hot_call_sites.sort_by(|a, b| b.inline_benefit_score.partial_cmp(&a.inline_benefit_score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Cache results
        {
            let mut cache = self.hot_paths.write().unwrap();
            cache.hot_call_sites.insert(function_id.to_string(), (hot_call_sites.clone(), Instant::now()));
        }
        
        hot_call_sites
    }
    
    /// Calculate optimization priority score
    pub fn calculate_priority(&self, function_id: &str, execution_count: u64, avg_execution_time: Duration) -> f64 {
        // Priority formula: (execution_count * avg_execution_time) / compilation_cost_estimate
        let execution_weight = execution_count as f64;
        let time_weight = avg_execution_time.as_nanos() as f64 / 1_000_000.0; // Convert to milliseconds
        let compilation_cost = self.estimate_compilation_cost(function_id);
        
        if compilation_cost > 0.0 {
            (execution_weight * time_weight) / compilation_cost
        } else {
            execution_weight * time_weight
        }
    }
    
    /// Check if function is optimization candidate
    pub fn is_optimization_candidate(&self, function_id: &str, current_tier: CompilationTier) -> bool {
        if let Some(profile) = self.profiling.get_profile(function_id) {
            let execution_count = profile.get_execution_count();
            let avg_execution_time = profile.get_average_execution_time();
            let priority_score = self.calculate_priority(function_id, execution_count, avg_execution_time);
            
            execution_count >= self.config.min_execution_count &&
            priority_score >= self.config.min_priority_score &&
            current_tier < CompilationTier::AggressiveJIT
        } else {
            false
        }
    }
    
    /// Recommend compilation tier based on execution patterns
    fn recommend_tier(&self, execution_count: u64, priority_score: f64) -> CompilationTier {
        if execution_count >= 1000 && priority_score >= 1000.0 {
            CompilationTier::AggressiveJIT
        } else if execution_count >= 100 && priority_score >= 500.0 {
            CompilationTier::OptimizedJIT
        } else if execution_count >= 10 && priority_score >= 100.0 {
            CompilationTier::QuickJIT
        } else {
            CompilationTier::Interpreter
        }
    }
    
    /// Estimate compilation cost for priority calculation
    fn estimate_compilation_cost(&self, function_id: &str) -> f64 {
        // Simple heuristic based on function ID length (proxy for complexity)
        // In a real implementation, this would analyze the function's AST
        let base_cost = 10.0; // Base compilation cost in milliseconds
        let complexity_factor = (function_id.len() as f64 / 10.0).max(1.0);
        base_cost * complexity_factor
    }
    
    /// Identify loop optimization opportunities
    fn identify_loop_optimizations(&self, stats: &crate::profiling::LoopStatistics) -> Vec<LoopOptimization> {
        let mut optimizations = Vec::new();
        
        // Check for unrolling opportunity
        if stats.is_predictable() && stats.average_iterations() <= 8.0 {
            let factor = (stats.average_iterations() as u32).min(4);
            optimizations.push(LoopOptimization::Unrolling { factor });
        }
        
        // Check for vectorization opportunity (simplified heuristic)
        if stats.average_iterations() >= 4.0 {
            optimizations.push(LoopOptimization::Vectorization);
        }
        
        // Always consider loop invariant code motion and strength reduction
        optimizations.push(LoopOptimization::LoopInvariantCodeMotion);
        optimizations.push(LoopOptimization::StrengthReduction);
        
        optimizations
    }
    
    /// Calculate inline benefit score
    fn calculate_inline_benefit_score(&self, call_count: u64, is_monomorphic: bool) -> f64 {
        let base_score = call_count as f64;
        let monomorphic_bonus = if is_monomorphic { 2.0 } else { 1.0 };
        base_score * monomorphic_bonus
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &HotPathConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: HotPathConfig) {
        self.config = config;
        
        // Clear cache to force reanalysis with new config
        let mut cache = self.hot_paths.write().unwrap();
        *cache = HotPathCache::new();
    }
}