//! Tiered compilation system for progressive optimization
//! 
//! Based on 2024-2026 industry best practices:
//! - Java HotSpot 4-tier compilation model
//! - .NET Dynamic PGO tiered approach
//! - V8 TurboFan optimization pipeline
//! - GraalVM Native Image PGO

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Compilation tier levels for progressive optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompilationTier {
    /// Tier 0: Interpreter (Rhai/AST) - No compilation
    Interpreter = 0,
    
    /// Tier 1: Quick JIT - Minimal optimizations, fast compilation (<5ms)
    QuickJIT = 1,
    
    /// Tier 2: Optimized JIT - Moderate optimizations (<20ms)
    OptimizedJIT = 2,
    
    /// Tier 3: Aggressive JIT - Full optimizations (<100ms)
    AggressiveJIT = 3,
}

impl CompilationTier {
    /// Get the next higher tier
    pub fn next_tier(self) -> Option<Self> {
        match self {
            CompilationTier::Interpreter => Some(CompilationTier::QuickJIT),
            CompilationTier::QuickJIT => Some(CompilationTier::OptimizedJIT),
            CompilationTier::OptimizedJIT => Some(CompilationTier::AggressiveJIT),
            CompilationTier::AggressiveJIT => None,
        }
    }
    
    /// Get compilation time budget for this tier
    pub fn compilation_time_budget(self) -> Duration {
        match self {
            CompilationTier::Interpreter => Duration::from_millis(0),
            CompilationTier::QuickJIT => Duration::from_millis(5),
            CompilationTier::OptimizedJIT => Duration::from_millis(20),
            CompilationTier::AggressiveJIT => Duration::from_millis(100),
        }
    }
    
    /// Get Cranelift optimization level for this tier
    #[cfg(feature = "dev-ui")]
    pub fn cranelift_opt_level(self) -> cranelift_codegen::settings::OptLevel {
        match self {
            CompilationTier::Interpreter => cranelift_codegen::settings::OptLevel::None,
            CompilationTier::QuickJIT => cranelift_codegen::settings::OptLevel::Speed,
            CompilationTier::OptimizedJIT => cranelift_codegen::settings::OptLevel::Speed,
            CompilationTier::AggressiveJIT => cranelift_codegen::settings::OptLevel::SpeedAndSize,
        }
    }
}

/// Configuration for tiered compilation
#[derive(Debug, Clone)]
pub struct TieredCompilationConfig {
    /// Execution count threshold for Tier 1 (Quick JIT)
    pub tier1_threshold: u32,
    
    /// Execution count threshold for Tier 2 (Optimized JIT)
    pub tier2_threshold: u32,
    
    /// Execution count threshold for Tier 3 (Aggressive JIT)
    pub tier3_threshold: u32,
    
    /// Enable background recompilation
    pub background_recompilation: bool,
    
    /// Maximum concurrent recompilation tasks
    pub max_concurrent_recompilations: usize,
    
    /// Enable tier statistics collection
    pub collect_statistics: bool,
    
    /// Profiling configuration
    pub profiling: crate::profiling::ProfilingConfig,
}

impl Default for TieredCompilationConfig {
    fn default() -> Self {
        Self {
            tier1_threshold: 10,      // Compile after 10 executions
            tier2_threshold: 100,     // Optimize after 100 executions
            tier3_threshold: 1000,    // Aggressively optimize after 1000 executions
            background_recompilation: true,
            max_concurrent_recompilations: 4,
            collect_statistics: true,
            profiling: crate::profiling::ProfilingConfig::default(),
        }
    }
}

/// Function metadata for tiered compilation
#[derive(Debug, Clone)]
pub struct FunctionMetadata {
    /// Function identifier (code hash)
    pub function_id: String,
    
    /// Current compilation tier
    pub current_tier: CompilationTier,
    
    /// Execution count
    pub execution_count: u32,
    
    /// Total execution time
    pub total_execution_time: Duration,
    
    /// Last execution timestamp
    pub last_execution: Instant,
    
    /// Compilation time for current tier
    pub compilation_time: Duration,
    
    /// Whether function is currently being recompiled
    pub recompiling: bool,
    
    /// Number of times recompiled
    pub recompilation_count: u32,
}

impl FunctionMetadata {
    pub fn new(function_id: String) -> Self {
        Self {
            function_id,
            current_tier: CompilationTier::Interpreter,
            execution_count: 0,
            total_execution_time: Duration::from_nanos(0),
            last_execution: Instant::now(),
            compilation_time: Duration::from_nanos(0),
            recompiling: false,
            recompilation_count: 0,
        }
    }
    
    /// Record an execution
    pub fn record_execution(&mut self, execution_time: Duration) {
        self.execution_count += 1;
        self.total_execution_time += execution_time;
        self.last_execution = Instant::now();
    }
    
    /// Get average execution time
    pub fn average_execution_time(&self) -> Duration {
        if self.execution_count == 0 {
            Duration::from_nanos(0)
        } else {
            self.total_execution_time / self.execution_count
        }
    }
    
    /// Check if function should be promoted to next tier
    pub fn should_promote(&self, config: &TieredCompilationConfig) -> bool {
        if self.recompiling {
            return false;
        }
        
        match self.current_tier {
            CompilationTier::Interpreter => {
                self.execution_count >= config.tier1_threshold
            }
            CompilationTier::QuickJIT => {
                self.execution_count >= config.tier2_threshold
            }
            CompilationTier::OptimizedJIT => {
                self.execution_count >= config.tier3_threshold
            }
            CompilationTier::AggressiveJIT => false,
        }
    }
}

/// Tiered compilation manager
pub struct TieredCompilationManager {
    /// Configuration
    config: TieredCompilationConfig,
    
    /// Function metadata indexed by function ID
    functions: Arc<Mutex<HashMap<String, FunctionMetadata>>>,
    
    /// Tier statistics
    stats: Arc<Mutex<TierStatistics>>,
    
    /// Hot path detector for optimization candidates
    #[cfg(feature = "dev-ui")]
    hot_path_detector: Option<Arc<crate::hot_path_detector::HotPathDetector>>,
}

impl TieredCompilationManager {
    /// Create a new tiered compilation manager
    pub fn new(config: TieredCompilationConfig) -> Self {
        Self {
            config,
            functions: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(TierStatistics::new())),
            #[cfg(feature = "dev-ui")]
            hot_path_detector: None,
        }
    }
    
    /// Create with hot path detector
    #[cfg(feature = "dev-ui")]
    pub fn with_hot_path_detector(
        config: TieredCompilationConfig,
        profiling: Arc<crate::profiling::ProfilingInfrastructure>,
    ) -> Self {
        let hot_path_config = crate::hot_path_detector::HotPathConfig::default();
        let hot_path_detector = Arc::new(crate::hot_path_detector::HotPathDetector::new(profiling, hot_path_config));
        
        Self {
            config,
            functions: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(TierStatistics::new())),
            hot_path_detector: Some(hot_path_detector),
        }
    }
    
    /// Record function execution
    pub fn record_execution(&self, function_id: &str, execution_time: Duration) {
        let mut functions = self.functions.lock().unwrap();
        
        let metadata = functions.entry(function_id.to_string())
            .or_insert_with(|| FunctionMetadata::new(function_id.to_string()));
        
        metadata.record_execution(execution_time);
        
        // Update statistics
        if self.config.collect_statistics {
            let mut stats = self.stats.lock().unwrap();
            stats.record_execution(metadata.current_tier);
        }
    }
    
    /// Check if function should be recompiled (enhanced with hot path detection)
    pub fn should_recompile(&self, function_id: &str) -> Option<CompilationTier> {
        let functions = self.functions.lock().unwrap();
        
        if let Some(metadata) = functions.get(function_id) {
            // First check traditional tier promotion
            if metadata.should_promote(&self.config) {
                return metadata.current_tier.next_tier();
            }
            
            // Then check hot path detector if available
            #[cfg(feature = "dev-ui")]
            if let Some(ref detector) = self.hot_path_detector {
                if detector.is_optimization_candidate(function_id, metadata.current_tier) {
                    return metadata.current_tier.next_tier();
                }
            }
        }
        
        None
    }
    
    /// Mark function as being recompiled
    pub fn start_recompilation(&self, function_id: &str, target_tier: CompilationTier) {
        let mut functions = self.functions.lock().unwrap();
        
        if let Some(metadata) = functions.get_mut(function_id) {
            metadata.recompiling = true;
            
            if self.config.collect_statistics {
                let mut stats = self.stats.lock().unwrap();
                stats.record_recompilation(metadata.current_tier, target_tier);
            }
        }
    }
    
    /// Complete recompilation and update tier
    pub fn complete_recompilation(&self, function_id: &str, new_tier: CompilationTier, compilation_time: Duration) {
        let mut functions = self.functions.lock().unwrap();
        
        if let Some(metadata) = functions.get_mut(function_id) {
            metadata.current_tier = new_tier;
            metadata.compilation_time = compilation_time;
            metadata.recompiling = false;
            metadata.recompilation_count += 1;
            
            if self.config.collect_statistics {
                let mut stats = self.stats.lock().unwrap();
                stats.record_tier_promotion(new_tier);
            }
        }
    }
    
    /// Get function metadata
    pub fn get_metadata(&self, function_id: &str) -> Option<FunctionMetadata> {
        let functions = self.functions.lock().unwrap();
        functions.get(function_id).cloned()
    }
    
    /// Get tier statistics
    pub fn get_statistics(&self) -> TierStatistics {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }
    
    /// Get all hot functions (candidates for recompilation)
    pub fn get_hot_functions(&self) -> Vec<(String, CompilationTier)> {
        let functions = self.functions.lock().unwrap();
        
        functions.iter()
            .filter_map(|(id, metadata)| {
                if metadata.should_promote(&self.config) {
                    metadata.current_tier.next_tier().map(|tier| (id.clone(), tier))
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get optimization recommendations from hot path detector
    #[cfg(feature = "dev-ui")]
    pub fn get_optimization_recommendations(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        if let Some(ref detector) = self.hot_path_detector {
            // Get hot functions
            let hot_functions = detector.detect_hot_functions();
            
            for hot_function in hot_functions {
                let recommendation = OptimizationRecommendation {
                    function_id: hot_function.function_id.clone(),
                    current_tier: hot_function.current_tier,
                    recommended_tier: hot_function.recommended_tier,
                    priority_score: hot_function.priority_score,
                    optimization_type: OptimizationType::TierPromotion,
                    estimated_benefit: self.estimate_optimization_benefit(&hot_function),
                };
                
                recommendations.push(recommendation);
            }
        }
        
        // Sort by priority score (highest first)
        recommendations.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap_or(std::cmp::Ordering::Equal));
        
        recommendations
    }
    
    /// Get hot path detector reference
    #[cfg(feature = "dev-ui")]
    pub fn get_hot_path_detector(&self) -> Option<Arc<crate::hot_path_detector::HotPathDetector>> {
        self.hot_path_detector.clone()
    }
    
    /// Estimate optimization benefit
    #[cfg(feature = "dev-ui")]
    fn estimate_optimization_benefit(&self, hot_function: &crate::hot_path_detector::HotFunction) -> f64 {
        // Simple heuristic: benefit is proportional to execution count and time
        let execution_factor = (hot_function.execution_count as f64).log10().max(1.0);
        let time_factor = hot_function.total_time.as_millis() as f64;
        execution_factor * time_factor / 1000.0 // Normalize to reasonable range
    }
}

/// Statistics for tiered compilation
#[derive(Debug, Clone)]
pub struct TierStatistics {
    /// Executions per tier
    pub executions_per_tier: HashMap<CompilationTier, u64>,
    
    /// Functions per tier
    pub functions_per_tier: HashMap<CompilationTier, u32>,
    
    /// Recompilations per tier transition
    pub recompilations: HashMap<(CompilationTier, CompilationTier), u32>,
    
    /// Total recompilation time
    pub total_recompilation_time: Duration,
}

impl TierStatistics {
    pub fn new() -> Self {
        let mut executions_per_tier = HashMap::new();
        let mut functions_per_tier = HashMap::new();
        
        for tier in &[
            CompilationTier::Interpreter,
            CompilationTier::QuickJIT,
            CompilationTier::OptimizedJIT,
            CompilationTier::AggressiveJIT,
        ] {
            executions_per_tier.insert(*tier, 0);
            functions_per_tier.insert(*tier, 0);
        }
        
        Self {
            executions_per_tier,
            functions_per_tier,
            recompilations: HashMap::new(),
            total_recompilation_time: Duration::from_nanos(0),
        }
    }
    
    pub fn record_execution(&mut self, tier: CompilationTier) {
        *self.executions_per_tier.entry(tier).or_insert(0) += 1;
    }
    
    pub fn record_recompilation(&mut self, from_tier: CompilationTier, to_tier: CompilationTier) {
        *self.recompilations.entry((from_tier, to_tier)).or_insert(0) += 1;
    }
    
    pub fn record_tier_promotion(&mut self, new_tier: CompilationTier) {
        *self.functions_per_tier.entry(new_tier).or_insert(0) += 1;
    }
}

/// Optimization recommendation from hot path analysis
#[cfg(feature = "dev-ui")]
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    /// Function identifier
    pub function_id: String,
    
    /// Current compilation tier
    pub current_tier: CompilationTier,
    
    /// Recommended tier
    pub recommended_tier: CompilationTier,
    
    /// Priority score
    pub priority_score: f64,
    
    /// Type of optimization
    pub optimization_type: OptimizationType,
    
    /// Estimated benefit score
    pub estimated_benefit: f64,
}

/// Types of optimizations
#[cfg(feature = "dev-ui")]
#[derive(Debug, Clone)]
pub enum OptimizationType {
    /// Promote to higher tier
    TierPromotion,
    
    /// Inline hot call sites
    Inlining,
    
    /// Unroll hot loops
    LoopUnrolling,
    
    /// Apply speculative optimizations
    SpeculativeOptimization,
}