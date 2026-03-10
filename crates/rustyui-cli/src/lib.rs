//! RustyUI CLI library
//! 
//! This crate provides the command-line interface for RustyUI projects,
//! including project initialization, development mode, and configuration management.

pub mod commands;
pub mod config;
pub mod error;
pub mod project;
pub mod template;

pub use error::{CliError, CliResult};
pub use config::ConfigManager;
pub use project::ProjectManager;
pub use template::TemplateManager;