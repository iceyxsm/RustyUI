//! Error handling for the RustyUI CLI

use thiserror::Error;

/// Result type for CLI operations
pub type CliResult<T> = Result<T, CliError>;

/// CLI-specific errors
#[derive(Error, Debug)]
pub enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(#[from] toml::de::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::ser::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Project error: {0}")]
    Project(String),
    
    #[error("Framework error: {0}")]
    Framework(String),
    
    #[error("Template error: {0}")]
    Template(String),
    
    #[error("Command execution failed: {0}")]
    Command(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Directory already exists: {0}")]
    DirectoryExists(String),
    
    #[error("Unsupported framework: {0}")]
    UnsupportedFramework(String),
    
    #[error("Development mode error: {0}")]
    DevMode(String),
    
    #[error("Build error: {0}")]
    Build(String),
}

impl CliError {
    /// Create a new project error
    pub fn project<S: Into<String>>(msg: S) -> Self {
        Self::Project(msg.into())
    }
    
    /// Create a new framework error
    pub fn framework<S: Into<String>>(msg: S) -> Self {
        Self::Framework(msg.into())
    }
    
    /// Create a new template error
    pub fn template<S: Into<String>>(msg: S) -> Self {
        Self::Template(msg.into())
    }
    
    /// Create a new command error
    pub fn command<S: Into<String>>(msg: S) -> Self {
        Self::Command(msg.into())
    }
    
    /// Create a new invalid config error
    pub fn invalid_config<S: Into<String>>(msg: S) -> Self {
        Self::InvalidConfig(msg.into())
    }
    
    /// Create a new file not found error
    pub fn file_not_found<S: Into<String>>(path: S) -> Self {
        Self::FileNotFound(path.into())
    }
    
    /// Create a new directory exists error
    pub fn directory_exists<S: Into<String>>(path: S) -> Self {
        Self::DirectoryExists(path.into())
    }
    
    /// Create a new unsupported framework error
    pub fn unsupported_framework<S: Into<String>>(framework: S) -> Self {
        Self::UnsupportedFramework(framework.into())
    }
    
    /// Create a new dev mode error
    pub fn dev_mode<S: Into<String>>(msg: S) -> Self {
        Self::DevMode(msg.into())
    }
    
    /// Create a new build error
    pub fn build<S: Into<String>>(msg: S) -> Self {
        Self::Build(msg.into())
    }
}

/// Display user-friendly error messages
impl CliError {
    /// Display error with styling
    pub fn display_styled(&self) -> String {
        use console::style;
        
        match self {
            CliError::Io(err) => format!("{} {}", style("IO Error:").red().bold(), err),
            CliError::Config(err) => format!("{} {}", style("Config Error:").red().bold(), err),
            CliError::Serialization(err) => format!("{} {}", style("Serialization Error:").red().bold(), err),
            CliError::Json(err) => format!("{} {}", style("JSON Error:").red().bold(), err),
            CliError::Project(msg) => format!("{} {}", style("Project Error:").red().bold(), msg),
            CliError::Framework(msg) => format!("{} {}", style("Framework Error:").red().bold(), msg),
            CliError::Template(msg) => format!("{} {}", style("Template Error:").red().bold(), msg),
            CliError::Command(msg) => format!("{} {}", style("Command Error:").red().bold(), msg),
            CliError::InvalidConfig(msg) => format!("{} {}", style("Invalid Config:").red().bold(), msg),
            CliError::FileNotFound(path) => format!("{} {}", style("File Not Found:").red().bold(), path),
            CliError::DirectoryExists(path) => format!("{} {}", style("Directory Exists:").red().bold(), path),
            CliError::UnsupportedFramework(framework) => format!("{} {}", style("Unsupported Framework:").red().bold(), framework),
            CliError::DevMode(msg) => format!("{} {}", style("Dev Mode Error:").red().bold(), msg),
            CliError::Build(msg) => format!("{} {}", style("Build Error:").red().bold(), msg),
        }
    }
}