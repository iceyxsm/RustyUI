//! Error handling for the interpreter

use thiserror::Error;

/// Result type for interpreter operations
pub type Result<T> = std::result::Result<T, InterpreterError>;

/// Interpreter error types
#[derive(Error, Debug, Clone)]
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
        #[cfg(feature = "dev-ui")]
        {
            Self::JIT(msg.into())
        }
        #[cfg(not(feature = "dev-ui"))]
        {
            Self::Generic(msg.into())
        }
    }
    
    /// Create a new validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }
    
    /// Create a new initialization error
    pub fn initialization(msg: impl Into<String>) -> Self {
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
    
    /// Check if this error requires compilation (for property tests)
    pub fn requires_compilation(&self) -> bool {
        match self {
            #[cfg(feature = "dev-ui")]
            InterpreterError::JIT(_) => true,
            _ => false,
        }
    }
    
    /// Check if this error causes system instability (for property tests)
    pub fn causes_system_instability(&self) -> bool {
        match self {
            #[cfg(feature = "dev-ui")]
            InterpreterError::Recovery(_) => true, // Recovery failure is serious
            #[cfg(feature = "dev-ui")]
            InterpreterError::ResourceLimit(_) => true, // Resource exhaustion
            _ => false,
        }
    }
    
    /// Check if this is a resource limit error (for property tests)
    pub fn is_resource_limit_error(&self) -> bool {
        match self {
            #[cfg(feature = "dev-ui")]
            InterpreterError::ResourceLimit(_) => true,
            _ => false,
        }
    }
}