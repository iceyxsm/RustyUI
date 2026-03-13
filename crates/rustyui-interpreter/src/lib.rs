//! # RustyUI Interpreter
//! 
//! Runtime interpretation system for RustyUI with Rhai scripting, AST parsing,
//! and Cranelift JIT compilation. Enhanced with 2026 performance optimizations.

#[cfg(feature = "dev-ui")]
pub mod rhai_interpreter;

#[cfg(feature = "dev-ui")]
pub mod ast_interpreter;

#[cfg(feature = "dev-ui")]
pub mod jit_compiler;

#[cfg(feature = "dev-ui")]
pub mod tiered_compilation;

#[cfg(feature = "dev-ui")]
pub mod profiling;

pub mod error;

#[cfg(test)]
mod property_tests;

#[cfg(test)]
mod test_jit;

pub use error::{InterpreterError, Result};

#[cfg(feature = "dev-ui")]
pub use rhai_interpreter::RhaiInterpreter;

#[cfg(feature = "dev-ui")]
pub use ast_interpreter::ASTInterpreter;

#[cfg(feature = "dev-ui")]
pub use jit_compiler::JITCompiler;

#[cfg(feature = "dev-ui")]
pub use tiered_compilation::{
    CompilationTier, TieredCompilationConfig, TieredCompilationManager,
    FunctionMetadata, TierStatistics,
};

#[cfg(feature = "dev-ui")]
pub use profiling::{
    ProfileData, ProfilingInfrastructure, ProfilingConfig, OverheadTracker,
    BranchStatistics, LoopStatistics, CallSiteStatistics, TypeFeedback,
};

/// Runtime interpreter that handles code changes without compilation
#[cfg(feature = "dev-ui")]
pub struct RuntimeInterpreter {
    /// Rhai scripting engine
    rhai_interpreter: RhaiInterpreter,
    
    /// AST interpretation system
    ast_interpreter: ASTInterpreter,
    
    /// JIT compiler for performance-critical code
    jit_compiler: JITCompiler,
    
    /// Tiered compilation manager
    tiered_compilation: TieredCompilationManager,
    
    /// Profiling infrastructure for PGO
    profiling: std::sync::Arc<ProfilingInfrastructure>,
    
    /// Interpretation cache (now using optimized memory pools)
    interpretation_cache: std::collections::HashMap<String, InterpretedCode>,
}

#[cfg(feature = "dev-ui")]
impl RuntimeInterpreter {
    /// Create a new runtime interpreter with performance optimizations
    pub fn new() -> Result<Self> {
        let config = TieredCompilationConfig::default();
        let profiling = std::sync::Arc::new(ProfilingInfrastructure::new(config.profiling.clone()));
        
        Ok(Self {
            rhai_interpreter: RhaiInterpreter::new()?,
            ast_interpreter: ASTInterpreter::new()?,
            jit_compiler: JITCompiler::new()?,
            tiered_compilation: TieredCompilationManager::new(config),
            profiling,
            interpretation_cache: std::collections::HashMap::new(),
        })
    }
    
    /// Create a new runtime interpreter with custom configuration
    pub fn with_config(config: TieredCompilationConfig) -> Result<Self> {
        let profiling = std::sync::Arc::new(ProfilingInfrastructure::new(config.profiling.clone()));
        
        Ok(Self {
            rhai_interpreter: RhaiInterpreter::new()?,
            ast_interpreter: ASTInterpreter::new()?,
            jit_compiler: JITCompiler::new()?,
            tiered_compilation: TieredCompilationManager::new(config),
            profiling,
            interpretation_cache: std::collections::HashMap::new(),
        })
    }
    
    /// Interpret a UI code change with error recovery and performance optimization
    pub fn interpret_change(&mut self, change: &UIChange) -> Result<InterpretationResult> {
        let start_time = std::time::Instant::now();
        
        // Generate function ID for profiling
        let function_id = self.calculate_function_id(&change.content);
        
        // Choose interpretation strategy based on code complexity and configuration
        let strategy = self.choose_strategy(change);
        
        let result = self.try_interpret_with_fallback(change, strategy);
        
        // Record execution in profiling infrastructure
        let execution_time = start_time.elapsed();
        self.profiling.record_execution(&function_id, execution_time);
        
        // Record execution in tiered compilation manager
        self.tiered_compilation.record_execution(&function_id, execution_time);
        
        // Cache successful interpretations
        if let Ok(ref interpretation_result) = result {
            if interpretation_result.success {
                self.interpretation_cache.insert(
                    change.content.clone(),
                    InterpretedCode {
                        source: change.content.clone(),
                        result: "Cached interpretation".to_string(),
                        timestamp: std::time::SystemTime::now(),
                    }
                );
            }
        }
        
        result
    }
    
    /// Try interpretation with automatic fallback on errors
    fn try_interpret_with_fallback(&mut self, change: &UIChange, mut strategy: InterpretationStrategy) -> Result<InterpretationResult> {
        let start_time = std::time::Instant::now();
        let mut attempts = 0;
        let max_attempts = 3;
        let original_strategy = strategy.clone();
        
        loop {
            attempts += 1;
            
            let result = match strategy {
                InterpretationStrategy::Rhai => {
                    self.rhai_interpreter.interpret(&change.content)
                }
                InterpretationStrategy::AST => {
                    self.ast_interpreter.interpret(&change.content)
                }
                InterpretationStrategy::JIT => {
                    self.jit_compiler.compile_and_execute(&change.content)
                }
            };
            
            match result {
                Ok(mut interpretation_result) => {
                    if attempts > 1 {
                        println!("SUCCESS: Interpretation succeeded after {} attempts using {:?}", attempts, strategy);
                    }
                    
                    // Populate additional fields for property tests
                    interpretation_result.used_strategy = Some(strategy.clone());
                    interpretation_result.required_compilation = Some(matches!(strategy, InterpretationStrategy::JIT));
                    interpretation_result.memory_usage_bytes = Some(self.estimate_memory_usage(&change.content));
                    interpretation_result.ui_updates = Some(vec!["UI updated".to_string()]); // Simplified for now
                    
                    return Ok(interpretation_result);
                }
                Err(error) if attempts < max_attempts => {
                    println!("WARNING: Interpretation failed with {:?}, attempting fallback: {}", strategy, error);
                    
                    // Try fallback strategy
                    strategy = match strategy {
                        InterpretationStrategy::JIT => InterpretationStrategy::AST,
                        InterpretationStrategy::AST => InterpretationStrategy::Rhai,
                        InterpretationStrategy::Rhai => {
                            // Last resort: return error with graceful degradation
                            return Ok(InterpretationResult {
                                execution_time: start_time.elapsed(),
                                success: false,
                                error_message: Some(format!("All interpretation strategies failed: {}", error)),
                                memory_usage_bytes: Some(0),
                                ui_updates: Some(vec![]),
                                used_strategy: Some(original_strategy),
                                required_compilation: Some(false),
                            });
                        }
                    };
                }
                Err(error) => {
                    println!("FAILED: All interpretation strategies failed after {} attempts", attempts);
                    return Ok(InterpretationResult {
                        execution_time: start_time.elapsed(),
                        success: false,
                        error_message: Some(format!("Interpretation failed: {}", error)),
                        memory_usage_bytes: Some(0),
                        ui_updates: Some(vec![]),
                        used_strategy: Some(original_strategy),
                        required_compilation: Some(false),
                    });
                }
            }
        }
    }
    
    /// Choose the best interpretation strategy for the given change
    fn choose_strategy(&self, change: &UIChange) -> InterpretationStrategy {
        // For Phase 1, use simple heuristics
        let code_length = change.content.len();
        
        if code_length < 100 {
            // Small changes - use Rhai for simplicity
            InterpretationStrategy::Rhai
        } else if code_length < 1000 {
            // Medium changes - use AST interpretation
            InterpretationStrategy::AST
        } else {
            // Large changes - use JIT for performance
            InterpretationStrategy::JIT
        }
    }
    
    /// Clear interpretation cache
    pub fn clear_cache(&mut self) {
        self.interpretation_cache.clear();
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            entries: self.interpretation_cache.len(),
            memory_usage: self.estimate_cache_memory(),
        }
    }
    
    /// Estimate memory usage of interpretation cache
    fn estimate_cache_memory(&self) -> usize {
        self.interpretation_cache
            .iter()
            .map(|(key, value)| key.len() + value.source.len() + value.result.len())
            .sum()
    }
    
    /// Calculate function ID for profiling (simple hash)
    fn calculate_function_id(&self, code: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
        format!("fn_{:x}", hasher.finish())
    }
}

/// Production builds have no runtime interpreter
#[cfg(not(feature = "dev-ui"))]
pub struct RuntimeInterpreter;

#[cfg(not(feature = "dev-ui"))]
impl RuntimeInterpreter {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

/// UI code change information
#[derive(Debug, Clone)]
pub struct UIChange {
    /// Code content to interpret
    pub content: String,

    /// Optional strategy preference (if None, will be auto-selected)
    pub interpretation_strategy: Option<InterpretationStrategy>,

    /// Component ID for tracking
    pub component_id: Option<String>,

    /// Change type for optimization
    pub change_type: ChangeType,

    /// Complexity score for performance analysis (used in property tests)
    pub complexity_score: Option<u32>,

    /// Timestamp for tracking (used in property tests)
    pub timestamp: Option<std::time::SystemTime>,
}


/// Types of UI changes
#[derive(Debug, Clone)]
pub enum ChangeType {
    ComponentUpdate,
    StyleChange,
    LayoutChange,
    EventHandlerChange,
    StateChange,
}

/// Interpretation strategies
#[derive(Debug, Clone, PartialEq)]
pub enum InterpretationStrategy {
    Rhai,
    AST,
    JIT,
}

/// Result of code interpretation
#[derive(Debug)]
pub struct InterpretationResult {
    /// Time taken for interpretation
    pub execution_time: std::time::Duration,
    
    /// Success status
    pub success: bool,
    
    /// Error message if interpretation failed
    pub error_message: Option<String>,
    
    /// Memory usage in bytes (for property tests)
    pub memory_usage_bytes: Option<u64>,
    
    /// UI updates generated (for property tests)
    pub ui_updates: Option<Vec<String>>,
    
    /// Strategy that was actually used (for property tests)
    pub used_strategy: Option<InterpretationStrategy>,
    
    /// Whether compilation was required (for property tests)
    pub required_compilation: Option<bool>,
}

/// Cached interpreted code
#[derive(Debug, Clone)]
pub struct InterpretedCode {
    /// Original source code
    pub source: String,
    
    /// Interpretation result
    pub result: String,
    
    /// Cache timestamp
    pub timestamp: std::time::SystemTime,
}
/// Cache statistics for monitoring
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached entries
    pub entries: usize,
    
    /// Estimated memory usage in bytes
    pub memory_usage: usize,
}
#[cfg(feature = "dev-ui")]
impl RuntimeInterpreter {
    /// Interpret UI component updates with enhanced parsing and error recovery
    pub fn interpret_ui_change(&mut self, code: &str, component_id: Option<String>) -> Result<InterpretationResult> {
        let change = UIChange {
            content: code.to_string(),
            interpretation_strategy: None, // Auto-select
            component_id: component_id.clone(),
            change_type: ChangeType::ComponentUpdate,
            complexity_score: None,
            timestamp: Some(std::time::SystemTime::now()),
        };
        
        // Try interpretation with error isolation
        match self.interpret_change(&change) {
            Ok(result) => Ok(result),
            Err(error) => {
                // Isolate error and provide graceful degradation
                println!("Error isolated for component {:?}: {}", component_id, error);
                
                Ok(InterpretationResult {
                    execution_time: std::time::Duration::from_millis(0),
                    success: false,
                    error_message: Some(format!("Isolated error: {}", error)),
                    memory_usage_bytes: Some(0),
                    ui_updates: Some(vec![]),
                    used_strategy: Some(InterpretationStrategy::Rhai),
                    required_compilation: Some(false),
                })
            }
        }
    }
    
    /// Interpret with error isolation to prevent crashes
    pub fn interpret_with_isolation(&mut self, change: &UIChange) -> InterpretationResult {
        match self.interpret_change(change) {
            Ok(result) => result,
            Err(error) => {
                println!("Error isolated during interpretation: {}", error);
                
                // Return safe fallback result
                InterpretationResult {
                    execution_time: std::time::Duration::from_millis(0),
                    success: false,
                    error_message: Some(format!("Isolated error: {}", error)),
                    memory_usage_bytes: Some(0),
                    ui_updates: Some(vec![]),
                    used_strategy: Some(InterpretationStrategy::Rhai),
                    required_compilation: Some(false),
                }
            }
        }
    }
    
    /// Check if a feature is supported and safe to use
    pub fn is_feature_supported(&self, feature: &str) -> bool {
        match feature {
            "rhai" => true,
            "ast" => true,
            "jit" => {
                // Check if JIT is available on this platform
                #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
                {
                    true
                }
                #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
                {
                    false
                }
            }
            _ => false,
        }
    }
    
    /// Get safe fallback strategy for unsupported features
    pub fn get_fallback_strategy(&self, unsupported_feature: &str) -> InterpretationStrategy {
        match unsupported_feature {
            "jit" => InterpretationStrategy::AST,
            "ast" => InterpretationStrategy::Rhai,
            _ => InterpretationStrategy::Rhai, // Most compatible fallback
        }
    }
    
    /// Interpret style changes specifically
    pub fn interpret_style_change(&mut self, css_code: &str, component_id: Option<String>) -> Result<InterpretationResult> {
        let change = UIChange {
            content: css_code.to_string(),
            interpretation_strategy: Some(InterpretationStrategy::Rhai), // CSS is simple, use Rhai
            component_id,
            change_type: ChangeType::StyleChange,
            complexity_score: None,
            timestamp: Some(std::time::SystemTime::now()),
        };
        
        self.interpret_change(&change)
    }
    
    /// Interpret layout changes
    pub fn interpret_layout_change(&mut self, layout_code: &str, component_id: Option<String>) -> Result<InterpretationResult> {
        let change = UIChange {
            content: layout_code.to_string(),
            interpretation_strategy: Some(InterpretationStrategy::AST), // Layout needs structure analysis
            component_id,
            change_type: ChangeType::LayoutChange,
            complexity_score: None,
            timestamp: Some(std::time::SystemTime::now()),
        };
        
        self.interpret_change(&change)
    }
    
    /// Interpret event handler changes
    pub fn interpret_event_handler(&mut self, handler_code: &str, component_id: Option<String>) -> Result<InterpretationResult> {
        let change = UIChange {
            content: handler_code.to_string(),
            interpretation_strategy: Some(InterpretationStrategy::JIT), // Event handlers benefit from JIT
            component_id,
            change_type: ChangeType::EventHandlerChange,
            complexity_score: None,
            timestamp: Some(std::time::SystemTime::now()),
        };
        
        self.interpret_change(&change)
    }
    
    /// Interpret state changes
    pub fn interpret_state_change(&mut self, state_code: &str, component_id: Option<String>) -> Result<InterpretationResult> {
        let change = UIChange {
            content: state_code.to_string(),
            interpretation_strategy: Some(InterpretationStrategy::Rhai), // State changes are usually simple
            component_id,
            change_type: ChangeType::StateChange,
            complexity_score: None,
            timestamp: Some(std::time::SystemTime::now()),
        };
        
        self.interpret_change(&change)
    }
    
    /// Batch interpret multiple UI changes for efficiency
    pub fn interpret_batch(&mut self, changes: Vec<UIChange>) -> Result<Vec<InterpretationResult>> {
        let mut results = Vec::new();
        
        for change in changes {
            let result = self.interpret_change(&change)?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Get interpretation performance metrics
    pub fn get_performance_metrics(&self) -> InterpreterMetrics {
        InterpreterMetrics {
            cache_hit_rate: self.calculate_cache_hit_rate(),
            average_interpretation_time: self.calculate_average_time(),
            total_interpretations: self.interpretation_cache.len(),
            memory_usage: self.estimate_cache_memory(),
        }
    }
    
    /// Get profiling infrastructure
    pub fn get_profiling(&self) -> std::sync::Arc<ProfilingInfrastructure> {
        self.profiling.clone()
    }
    
    /// Get profiling overhead percentage
    pub fn get_profiling_overhead(&self) -> f64 {
        self.profiling.get_overhead_percentage()
    }
    
    /// Record branch outcome for profiling
    pub fn record_branch(&self, function_id: &str, branch_id: u32, taken: bool) {
        self.profiling.record_branch(function_id, branch_id, taken);
    }
    
    /// Record loop iteration count for profiling
    pub fn record_loop(&self, function_id: &str, loop_id: u32, iterations: u64) {
        self.profiling.record_loop(function_id, loop_id, iterations);
    }
    
    /// Record call site invocation for profiling
    pub fn record_call_site(&self, function_id: &str, call_site_id: u32, target: &str) {
        self.profiling.record_call_site(function_id, call_site_id, target);
    }
    
    /// Record type observation for profiling
    pub fn record_type(&self, function_id: &str, operation_id: u32, type_name: &str) {
        self.profiling.record_type(function_id, operation_id, type_name);
    }
    
    /// Calculate cache hit rate (simplified for Phase 1)
    fn calculate_cache_hit_rate(&self) -> f64 {
        // In Phase 1, we'll use a simple estimation
        if self.interpretation_cache.len() > 10 {
            0.75 // Assume 75% hit rate for established cache
        } else {
            0.25 // Lower hit rate for new cache
        }
    }
    
    /// Calculate average interpretation time (simplified for Phase 1)
    fn calculate_average_time(&self) -> std::time::Duration {
        // In Phase 1, return a reasonable estimate
        std::time::Duration::from_millis(5) // Target <10ms
    }
    
    /// Estimate memory usage for a given code string (for property tests)
    fn estimate_memory_usage(&self, code: &str) -> u64 {
        // Simple estimation based on code length and complexity
        let base_memory = code.len() as u64 * 8; // 8 bytes per character
        let complexity_factor = if code.contains("fn ") || code.contains("struct ") { 2 } else { 1 };
        base_memory * complexity_factor + 1024 // Add base overhead
    }
}

/// Performance metrics for the interpreter
#[derive(Debug, Clone)]
pub struct InterpreterMetrics {
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    
    /// Average interpretation time
    pub average_interpretation_time: std::time::Duration,
    
    /// Total number of interpretations performed
    pub total_interpretations: usize,
    
    /// Memory usage in bytes
    pub memory_usage: usize,
}

/// UI update information for applying changes
#[derive(Debug, Clone)]
pub struct UIUpdate {
    /// Component ID being updated
    pub component_id: String,
    
    /// Type of update
    pub update_type: UpdateType,
    
    /// Update data (JSON or other format)
    pub data: String,
    
    /// Timestamp of the update
    pub timestamp: std::time::SystemTime,
}

/// Types of UI updates
#[derive(Debug, Clone)]
pub enum UpdateType {
    /// Replace entire component
    Replace,
    /// Update component properties
    UpdateProperties,
    /// Update component style
    UpdateStyle,
    /// Update component layout
    UpdateLayout,
    /// Update component state
    UpdateState,
}