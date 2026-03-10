//! Core traits for UI framework adapters

use serde::{Deserialize, Serialize};

/// Trait for framework-agnostic UI framework adapters
pub trait UIFrameworkAdapter: Send + Sync {
    /// Get the name of the UI framework
    fn framework_name(&self) -> &'static str;
    
    /// Initialize the adapter with configuration
    fn initialize(&mut self, config: &FrameworkConfig) -> anyhow::Result<()>;
    
    /// Create a render context for component rendering
    fn create_render_context(&self) -> Box<dyn RenderContext>;
    
    /// Handle runtime updates during development (development only)
    #[cfg(feature = "dev-ui")]
    fn handle_runtime_update(&mut self, update: &RuntimeUpdate) -> anyhow::Result<()> {
        let _ = update;
        Ok(())
    }
    
    /// Preserve framework-specific state (development only)
    #[cfg(feature = "dev-ui")]
    fn preserve_framework_state(&self) -> anyhow::Result<FrameworkState> {
        Ok(FrameworkState::None)
    }
    
    /// Restore framework-specific state (development only)
    #[cfg(feature = "dev-ui")]
    fn restore_framework_state(&mut self, _state: FrameworkState) -> anyhow::Result<()> {
        Ok(())
    }
    
    /// Check if the adapter requires framework modifications
    fn requires_framework_modifications(&self) -> bool {
        false
    }
}

/// Trait for framework-agnostic rendering context
pub trait RenderContext: Send + Sync {
    /// Render a button with the given text and callback
    fn render_button(&mut self, text: &str, callback: Box<dyn Fn()>);
    
    /// Render text with the given content
    fn render_text(&mut self, text: &str);
    
    /// Render a horizontal layout with children
    fn render_horizontal_layout(&mut self, children: Vec<Box<dyn RenderContext>>);
    
    /// Render a vertical layout with children
    fn render_vertical_layout(&mut self, children: Vec<Box<dyn RenderContext>>);
}

/// Configuration for framework adapters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkConfig {
    /// Framework-specific settings
    pub settings: serde_json::Value,
    
    /// Enable development features
    pub development_mode: bool,
    
    /// Performance optimization level
    pub optimization_level: u8,
}

impl Default for FrameworkConfig {
    fn default() -> Self {
        Self {
            settings: serde_json::Value::Null,
            development_mode: false,
            optimization_level: 2,
        }
    }
}

/// Framework-specific state for preservation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrameworkState {
    /// No state to preserve
    None,
    
    /// egui-specific state
    #[cfg(feature = "egui-adapter")]
    Egui(Vec<u8>),
    
    /// iced-specific state
    #[cfg(feature = "iced-adapter")]
    Iced(Vec<u8>),
    
    /// slint-specific state
    #[cfg(feature = "slint-adapter")]
    Slint(Vec<u8>),
    
    /// tauri-specific state
    #[cfg(feature = "tauri-adapter")]
    Tauri(Vec<u8>),
    
    /// Custom framework state
    Custom { framework: String, data: Vec<u8> },
}

/// Runtime update information for development mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeUpdate {
    /// Component identifier
    pub component_id: String,
    
    /// Type of update
    pub update_type: UpdateType,
    
    /// Update data
    pub data: serde_json::Value,
}

/// Types of runtime updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateType {
    /// Component structure changed
    ComponentChange,
    
    /// Styling changed
    StyleChange,
    
    /// Layout changed
    LayoutChange,
    
    /// Event handler changed
    EventHandlerChange,
}