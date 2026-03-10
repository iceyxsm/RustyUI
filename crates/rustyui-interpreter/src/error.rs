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
    
    /// Generic interpretation errors
    #[error("Interpretation error: {0}")]
    Generic(String),
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
}