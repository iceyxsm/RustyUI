//! Error handling for the interpreter

use thiserror::Error;

/// Result type for interpreter operations
pub type Result<T> = std::result::Result<T, InterpreterError>;

/// Interpreter error types
#[derive(Error, Debug)]
pub enum InterpreterError {
    /// Rhai script interpretation errors
    #[cfg(feature = "dev-ui")]
    #[error("Rhai interpretation error: {0}")]
    Rhai(String),
    
    /// AST interpretation errors
    #[cfg(feature = "dev-ui")]
    #[error("AST interpretation error: {0}")]
    AST(String),
    
    /// JIT compilation errors
    #[cfg(feature = "dev-ui")]
    #[error("JIT compilation error: {0}")]
    JIT(String),
    
    /// Recovery error - when fallback mechanisms fail
    #[cfg(feature = "dev-ui")]
    #[error("Recovery error: {0}")]
    Recovery(String),
    
    /// Unsupported feature error
    #[cfg(feature = "dev-ui")]
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
    
    /// Generic interpretation errors
    #[error("Interpretation error: {0}")]
    Generic(String),
    
    /// Resource limit exceeded errors
    #[cfg(feature = "dev-ui")]
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
}

impl InterpreterError {
    /// Create a new generic error
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }
    
    /// Create a new Rhai error (development only)
    #[cfg(feature = "dev-ui")]
    pub fn rhai(msg: impl Into<String>) -> Self {
        Self::Rhai(msg.into())
    }
    
    /// Create a new AST error (development only)
    #[cfg(feature = "dev-ui")]
    pub fn ast(msg: impl Into<String>) -> Self {
        Self::AST(msg.into())
    }
    
    /// Create a new JIT error (development only)
    #[cfg(feature = "dev-ui")]
    pub fn jit(msg: impl Into<String>) -> Self {
        Self::JIT(msg.into())
    }
    
    /// Create a new recovery error (development only)
    #[cfg(feature = "dev-ui")]
    pub fn recovery(msg: impl Into<String>) -> Self {
        Self::Recovery(msg.into())
    }
    
    /// Create a new unsupported feature error (development only)
    #[cfg(feature = "dev-ui")]
    pub fn unsupported_feature(msg: impl Into<String>) -> Self {
        Self::UnsupportedFeature(msg.into())
    }
    
    /// Create a new execution error
    pub fn execution(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }
    
    /// Create a new compilation error
    pub fn compilation(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }
    
    /// Create a new resource limit error (development only)
    #[cfg(feature = "dev-ui")]
    pub fn resource_limit(msg: impl Into<String>) -> Self {
        Self::ResourceLimit(msg.into())
    }
    
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            #[cfg(feature = "dev-ui")]
            InterpreterError::Rhai(_) => true,  // Can fallback to AST
            #[cfg(feature = "dev-ui")]
            InterpreterError::AST(_) => true,   // Can fallback to Rhai
            #[cfg(feature = "dev-ui")]
            InterpreterError::JIT(_) => true,   // Can fallback to AST
            #[cfg(feature = "dev-ui")]
            InterpreterError::UnsupportedFeature(_) => true, // Can disable feature
            #[cfg(feature = "dev-ui")]
            InterpreterError::ResourceLimit(_) => true, // Can reduce resource usage
            #[cfg(feature = "dev-ui")]
            InterpreterError::Recovery(_) => false, // Recovery itself failed
            InterpreterError::Generic(_) => true,   // Can try isolation
        }
    }
    
    /// Get suggested recovery strategy name
    pub fn suggested_recovery_strategy(&self) -> &'static str {
        match self {
            #[cfg(feature = "dev-ui")]
            InterpreterError::Rhai(_) => "FallbackToAST",
            #[cfg(feature = "dev-ui")]
            InterpreterError::AST(_) => "FallbackToRhai",
            #[cfg(feature = "dev-ui")]
            InterpreterError::JIT(_) => "FallbackToAST",
            #[cfg(feature = "dev-ui")]
            InterpreterError::UnsupportedFeature(_) => "DisableFeature",
            #[cfg(feature = "dev-ui")]
            InterpreterError::ResourceLimit(_) => "ReduceResourceUsage",
            #[cfg(feature = "dev-ui")]
            InterpreterError::Recovery(_) => "IsolateAndContinue",
            InterpreterError::Generic(_) => "IsolateAndContinue",
        }
    }
}