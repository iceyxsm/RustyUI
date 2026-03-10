//! # RustyUI Core
//! 
//! Core dual-mode engine for RustyUI that provides runtime interpretation during development
//! and zero-overhead compilation for production builds.

pub mod config;
pub mod engine;
pub mod error;
pub mod build_config;
pub mod ui_component;

#[cfg(feature = "dev-ui")]
pub mod change_monitor;

#[cfg(feature = "dev-ui")]
pub mod change_analyzer;

#[cfg(feature = "dev-ui")]
pub mod state_preservor;

pub use config::DualModeConfig;
pub use engine::DualModeEngine;
pub use error::{RustyUIError, Result};
pub use build_config::{BuildConfig, BuildInfo, OptimizationLevel};
pub use ui_component::{UIComponent, UIComponentExt, UIComponentDyn, ComponentStateManager};

#[cfg(feature = "dev-ui")]
pub use change_monitor::ChangeMonitor;

#[cfg(feature = "dev-ui")]
pub use change_analyzer::ChangeAnalyzer;

#[cfg(feature = "dev-ui")]
pub use state_preservor::StatePreservor;

/// Trait for framework-agnostic rendering context
pub trait RenderContext {
    /// Render a button with the given text and callback
    fn render_button(&mut self, text: &str, callback: Box<dyn Fn()>);
    
    /// Render text with the given content
    fn render_text(&mut self, text: &str);
    
    /// Handle runtime updates during development (development only)
    #[cfg(feature = "dev-ui")]
    fn handle_runtime_update(&mut self, update: &RuntimeUpdate);
}

/// Runtime update information for development mode
#[cfg(feature = "dev-ui")]
#[derive(Debug, Clone)]
pub struct RuntimeUpdate {
    pub component_id: String,
    pub update_type: UpdateType,
    pub data: serde_json::Value,
}

/// Types of runtime updates
#[cfg(feature = "dev-ui")]
#[derive(Debug, Clone)]
pub enum UpdateType {
    ComponentChange,
    StyleChange,
    LayoutChange,
    EventHandlerChange,
}