//! Error types for RustyUI core

use thiserror::Error;

/// Result type for RustyUI operations
pub type Result<T> = std::result::Result<T, RustyUIError>;

/// Error types for RustyUI operations
#[derive(Error, Debug)]
pub enum RustyUIError {
    /// File system watcher error (development only)
    #[cfg(feature = "dev-ui")]
    #[error("File watcher error: {0}")]
    FileWatcher(#[from] notify::Error),
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// Configuration error
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    /// Engine initialization error
    #[error("Engine initialization failed: {message}")]
    Initialization { message: String },
    
    /// Runtime interpretation error
    #[error("Runtime interpretation error: {message}")]
    Interpretation { message: String },
    
    /// State preservation error
    #[error("State preservation error: {message}")]
    StatePreservation { message: String },
    
    /// Framework adapter error
    #[error("Framework adapter error: {message}")]
    FrameworkAdapter { message: String },
    
    /// Generic error with custom message
    #[error("{message}")]
    Generic { message: String },
}

impl RustyUIError {
    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }
    
    /// Create an initialization error
    pub fn initialization(message: impl Into<String>) -> Self {
        Self::Initialization {
            message: message.into(),
        }
    }
    
    /// Create an interpretation error
    pub fn interpretation(message: impl Into<String>) -> Self {
        Self::Interpretation {
            message: message.into(),
        }
    }
    
    /// Create a state preservation error
    pub fn state_preservation(message: impl Into<String>) -> Self {
        Self::StatePreservation {
            message: message.into(),
        }
    }
    
    /// Create a framework adapter error
    pub fn framework_adapter(message: impl Into<String>) -> Self {
        Self::FrameworkAdapter {
            message: message.into(),
        }
    }
    
    /// Create a generic error
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }
    
    /// Create a component not found error
    pub fn component_not_found(message: impl Into<String>) -> Self {
        Self::Generic {
            message: format!("Component not found: {}", message.into()),
        }
    }
    
    /// Create a file watching error (development only)
    #[cfg(feature = "dev-ui")]
    pub fn file_watching(message: impl Into<String>) -> Self {
        Self::Generic {
            message: format!("File watching error: {}", message.into()),
        }
    }
    
    /// Create a file watching error (no-op in production)
    #[cfg(not(feature = "dev-ui"))]
    pub fn file_watching(message: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }
}