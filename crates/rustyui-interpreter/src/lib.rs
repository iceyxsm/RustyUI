//! # RustyUI Interpreter
//! 
//! Runtime interpretation system for RustyUI with Rhai scripting, AST parsing,
//! and Cranelift JIT compilation.

#[cfg(feature = "dev-ui")]
pub mod rhai_interpreter;

#[cfg(feature = "dev-ui")]
pub mod ast_interpreter;

#[cfg(feature = "dev-ui")]
pub mod jit_compiler;

pub mod error;

pub use error::{InterpreterError, Result};

#[cfg(feature = "dev-ui")]
pub use rhai_interpreter::RhaiInterpreter;

#[cfg(feature = "dev-ui")]
pub use ast_interpreter::ASTInterpreter;

#[cfg(feature = "dev-ui")]
pub use jit_compiler::JITCompiler;

/// Runtime interpreter that handles code changes without compilation
#[cfg(feature = "dev-ui")]
pub struct RuntimeInterpreter {
    /// Rhai scripting engine
    rhai_interpreter: RhaiInterpreter,
    
    /// AST interpretation system
    ast_interpreter: ASTInterpreter,
    
    /// JIT compiler for performance-critical code
    jit_compiler: JITCompiler,
    
    /// Interpretation cache
    interpretation_cache: std::collections::HashMap<String, InterpretedCode>,
}

#[cfg(feature = "dev-ui")]
impl RuntimeInterpreter {
    /// Create a new runtime interpreter
    pub fn new() -> Result<Self> {
        Ok(Self {
            rhai_interpreter: RhaiInterpreter::new()?,
            ast_interpreter: ASTInterpreter::new()?,
            jit_compiler: JITCompiler::new()?,
            interpretation_cache: std::collections::HashMap::new(),
        })
    }
    
    /// Interpret a UI code change
    pub fn interpret_change(&mut self, change: &UIChange) -> Result<InterpretationResult> {
        let start_time = std::time::Instant::now();
        
        // Choose interpretation strategy based on code complexity and configuration
        let strategy = self.choose_strategy(change);
        
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
#[derive(Debug, Clone)]
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