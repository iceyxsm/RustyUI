//! Framework adapter traits and interfaces for RustyUI
//! 
//! Implements the adapter pattern following 2026 best practices for UI framework integration.
//! Provides a clean abstraction layer that allows RustyUI to work with multiple UI frameworks
//! (egui, iced, tauri, slint) without tight coupling.

use serde::{Serialize, Deserialize};

/// Result type for framework adapter operations
pub type AdapterResult<T> = Result<T, AdapterError>;

/// Errors that can occur during framework adapter operations
#[derive(Debug, Clone)]
pub enum AdapterError {
    /// Framework-specific initialization error
    InitializationFailed(String),
    /// Component rendering error
    RenderingFailed(String),
    /// State preservation/restoration error
    StateFailed(String),
    /// Hot reload operation error
    HotReloadFailed(String),
    /// Framework not supported
    UnsupportedFramework(String),
    /// Invalid component configuration
    InvalidComponent(String),
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            AdapterError::RenderingFailed(msg) => write!(f, "Rendering failed: {}", msg),
            AdapterError::StateFailed(msg) => write!(f, "State operation failed: {}", msg),
            AdapterError::HotReloadFailed(msg) => write!(f, "Hot reload failed: {}", msg),
            AdapterError::UnsupportedFramework(name) => write!(f, "Unsupported framework: {}", name),
            AdapterError::InvalidComponent(msg) => write!(f, "Invalid component: {}", msg),
        }
    }
}

impl std::error::Error for AdapterError {}

/// Core trait for UI framework adapters
/// 
/// This trait defines the contract that all framework adapters must implement.
/// It follows the adapter pattern to provide a unified interface across different
/// UI frameworks while maintaining framework-specific optimizations.
/// 
/// # 2026 Design Principles
/// - Framework-agnostic interface for unified development experience
/// - Conditional compilation for zero production overhead
/// - Runtime interpretation support for instant development feedback
/// - State preservation across hot reload cycles
/// - Performance-first design with minimal abstraction cost
pub trait UIFrameworkAdapter: Send + Sync {
    /// Returns the name of the UI framework this adapter supports
    fn framework_name(&self) -> &'static str;
    
    /// Initialize the framework adapter with the given configuration
    fn initialize(&mut self, config: &FrameworkConfig) -> AdapterResult<()>;
    
    /// Render a UI component using the framework's rendering system
    fn render_component(&mut self, component: &dyn UIComponent, ctx: &mut dyn RenderContext) -> AdapterResult<()>;
    
    /// Create a new render context for this framework
    fn create_render_context(&self) -> AdapterResult<Box<dyn RenderContext>>;
    
    /// Check if the adapter requires modifications to the underlying framework
    fn requires_framework_modifications(&self) -> bool {
        false
    }
    
    // Development-only methods for runtime interpretation and hot reload
    
    /// Handle runtime updates during development mode
    #[cfg(feature = "dev-ui")]
    fn handle_runtime_update(&mut self, update: &RuntimeUpdate) -> AdapterResult<()>;
    
    /// Preserve framework-specific state before hot reload
    #[cfg(feature = "dev-ui")]
    fn preserve_framework_state(&self) -> AdapterResult<FrameworkState>;
    
    /// Restore framework-specific state after hot reload
    #[cfg(feature = "dev-ui")]
    fn restore_framework_state(&mut self, state: FrameworkState) -> AdapterResult<()>;
    
    /// Check if runtime interpretation is supported for this framework
    #[cfg(feature = "dev-ui")]
    fn supports_runtime_interpretation(&self) -> bool {
        true
    }
    
    /// Apply a runtime update to a specific component
    #[cfg(feature = "dev-ui")]
    fn apply_component_update(&mut self, component_id: &str, update_data: &serde_json::Value) -> AdapterResult<()>;
}

/// Trait for framework-agnostic rendering context
/// 
/// Provides a unified interface for rendering UI components across different frameworks.
/// Each framework adapter implements this trait to provide framework-specific rendering
/// while maintaining a consistent API for UI components.
pub trait RenderContext: Send + Sync {
    /// Render a button with the given text and callback
    fn render_button(&mut self, text: &str, callback: Box<dyn Fn() + Send + Sync>);
    
    /// Render static text
    fn render_text(&mut self, text: &str);
    
    /// Render an input field with the given value and change callback
    fn render_input(&mut self, value: &str, on_change: Box<dyn Fn(String) + Send + Sync>);
    
    /// Render a checkbox with the given state and change callback
    fn render_checkbox(&mut self, checked: bool, on_change: Box<dyn Fn(bool) + Send + Sync>);
    
    /// Begin a horizontal layout container
    fn begin_horizontal_layout(&mut self);
    
    /// End the current horizontal layout container
    fn end_horizontal_layout(&mut self);
    
    /// Begin a vertical layout container
    fn begin_vertical_layout(&mut self);
    
    /// End the current vertical layout container
    fn end_vertical_layout(&mut self);
    
    /// Apply custom styling to the next rendered element
    fn apply_style(&mut self, style: &ComponentStyle);
    
    /// Get the current rendering bounds/area
    fn get_available_rect(&self) -> Rect;
    
    /// Check if the context supports a specific rendering feature
    fn supports_feature(&self, feature: RenderFeature) -> bool;
    
    // Development-only methods for runtime updates
    
    /// Handle runtime updates during development mode
    #[cfg(feature = "dev-ui")]
    fn handle_runtime_update(&mut self, update: &RuntimeUpdate) -> AdapterResult<()>;
    
    /// Mark a component for hot reload tracking
    #[cfg(feature = "dev-ui")]
    fn mark_component_for_tracking(&mut self, component_id: &str);
}

/// Trait for UI components that can be rendered and hot-reloaded
/// 
/// All UI components must implement this trait to work with the RustyUI system.
/// The trait provides both production rendering and development-time hot reload capabilities.
pub trait UIComponent: Send + Sync {
    /// Render the component using the provided render context
    fn render(&mut self, ctx: &mut dyn RenderContext) -> AdapterResult<()>;
    
    /// Get a unique identifier for this component instance
    fn component_id(&self) -> &str;
    
    /// Get the component type name for debugging and hot reload
    fn component_type(&self) -> &'static str;
    
    // Development-only methods for hot reload support
    
    /// Extract the current state for hot reload preservation
    #[cfg(feature = "dev-ui")]
    fn hot_reload_state(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
    
    /// Restore state after hot reload
    #[cfg(feature = "dev-ui")]
    fn restore_state(&mut self, _state: serde_json::Value) -> AdapterResult<()> {
        Ok(())
    }
    
    /// Get interpretation hint for runtime updates
    #[cfg(feature = "dev-ui")]
    fn interpretation_hint(&self) -> InterpretationHint {
        InterpretationHint::Auto
    }
    
    /// Handle runtime component updates
    #[cfg(feature = "dev-ui")]
    fn handle_runtime_update(&mut self, _update: &RuntimeUpdate) -> AdapterResult<()> {
        // Default implementation does nothing
        Ok(())
    }
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
    
    /// Timestamp of the update
    pub timestamp: std::time::SystemTime,
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

/// Interpretation hints for runtime updates (development mode only)
#[cfg(feature = "dev-ui")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpretationHint {
    /// Automatically choose the best interpretation strategy
    Auto,
    /// Prefer Rhai script interpretation for fast updates
    PreferRhai,
    /// Prefer AST interpretation for better performance
    PreferAST,
    /// Require JIT compilation for performance-critical code
    RequireJIT,
    /// Disable interpretation for this component
    NoInterpretation,
}

/// Component styling information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStyle {
    /// Background color (RGBA)
    pub background_color: Option<[f32; 4]>,
    
    /// Text color (RGBA)
    pub text_color: Option<[f32; 4]>,
    
    /// Font size in points
    pub font_size: Option<f32>,
    
    /// Padding around the component
    pub padding: Option<Padding>,
    
    /// Margin around the component
    pub margin: Option<Margin>,
    
    /// Border configuration
    pub border: Option<Border>,
}

/// Padding configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

/// Margin configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Margin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

/// Border configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Border {
    pub width: f32,
    pub color: [f32; 4], // RGBA
    pub style: BorderStyle,
}

/// Border style options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BorderStyle {
    Solid,
    Dashed,
    Dotted,
}

/// Rectangle representing rendering bounds
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

/// Rendering features that may be supported by different frameworks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderFeature {
    /// Support for custom fonts
    CustomFonts,
    /// Support for animations
    Animations,
    /// Support for 3D rendering
    ThreeDRendering,
    /// Support for custom shaders
    CustomShaders,
    /// Support for vector graphics
    VectorGraphics,
    /// Support for image rendering
    ImageRendering,
    /// Support for text input
    TextInput,
    /// Support for drag and drop
    DragAndDrop,
}

/// Layout types for organizing UI components
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LayoutType {
    /// Horizontal layout (left to right)
    Horizontal,
    /// Vertical layout (top to bottom)
    Vertical,
    /// Grid layout with specified columns and rows
    Grid { columns: u32, rows: u32 },
    /// Absolute positioning
    Absolute,
    /// Flexible layout with grow factors
    Flex,
}

/// Performance metrics for framework operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkMetrics {
    /// Time taken for rendering operations
    pub render_time_ms: f64,
    
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    
    /// Number of components rendered
    pub components_rendered: u32,
    
    /// Frame rate (frames per second)
    pub fps: f32,
    
    /// Development-only metrics
    #[cfg(feature = "dev-ui")]
    pub hot_reload_metrics: Option<HotReloadMetrics>,
}

/// Hot reload performance metrics (development mode only)
#[cfg(feature = "dev-ui")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadMetrics {
    /// Time taken for state preservation
    pub state_preservation_time_ms: f64,
    
    /// Time taken for component updates
    pub update_time_ms: f64,
    
    /// Time taken for state restoration
    pub state_restoration_time_ms: f64,
    
    /// Total hot reload cycle time
    pub total_reload_time_ms: f64,
}

/// Factory trait for creating framework adapters
pub trait AdapterFactory: Send + Sync {
    /// Create a new adapter instance for the specified framework
    fn create_adapter(&self, framework: &str, config: &FrameworkConfig) -> AdapterResult<Box<dyn UIFrameworkAdapter>>;
    
    /// List supported frameworks
    fn supported_frameworks(&self) -> Vec<&'static str>;
    
    /// Check if a framework is supported
    fn supports_framework(&self, framework: &str) -> bool {
        self.supported_frameworks().contains(&framework)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Mock implementation of UIFrameworkAdapter for testing
    struct MockAdapter {
        name: &'static str,
        initialized: bool,
    }
    
    impl MockAdapter {
        fn new(name: &'static str) -> Self {
            Self {
                name,
                initialized: false,
            }
        }
    }
    
    impl UIFrameworkAdapter for MockAdapter {
        fn framework_name(&self) -> &'static str {
            self.name
        }
        
        fn initialize(&mut self, _config: &FrameworkConfig) -> AdapterResult<()> {
            self.initialized = true;
            Ok(())
        }
        
        fn render_component(&mut self, _component: &dyn UIComponent, _ctx: &mut dyn RenderContext) -> AdapterResult<()> {
            if !self.initialized {
                return Err(AdapterError::InitializationFailed("Adapter not initialized".to_string()));
            }
            Ok(())
        }
        
        fn create_render_context(&self) -> AdapterResult<Box<dyn RenderContext>> {
            Ok(Box::new(MockRenderContext::new()))
        }
        
        #[cfg(feature = "dev-ui")]
        fn handle_runtime_update(&mut self, _update: &RuntimeUpdate) -> AdapterResult<()> {
            Ok(())
        }
        
        #[cfg(feature = "dev-ui")]
        fn preserve_framework_state(&self) -> AdapterResult<FrameworkState> {
            Ok(FrameworkState::None)
        }
        
        #[cfg(feature = "dev-ui")]
        fn restore_framework_state(&mut self, _state: FrameworkState) -> AdapterResult<()> {
            Ok(())
        }
        
        #[cfg(feature = "dev-ui")]
        fn apply_component_update(&mut self, _component_id: &str, _update_data: &serde_json::Value) -> AdapterResult<()> {
            Ok(())
        }
    }
    
    /// Mock implementation of RenderContext for testing
    struct MockRenderContext {
        button_count: u32,
        text_count: u32,
    }
    
    impl MockRenderContext {
        fn new() -> Self {
            Self {
                button_count: 0,
                text_count: 0,
            }
        }
    }
    
    impl RenderContext for MockRenderContext {
        fn render_button(&mut self, _text: &str, _callback: Box<dyn Fn() + Send + Sync>) {
            self.button_count += 1;
        }
        
        fn render_text(&mut self, _text: &str) {
            self.text_count += 1;
        }
        
        fn render_input(&mut self, _value: &str, _on_change: Box<dyn Fn(String) + Send + Sync>) {}
        
        fn render_checkbox(&mut self, _checked: bool, _on_change: Box<dyn Fn(bool) + Send + Sync>) {}
        
        fn begin_horizontal_layout(&mut self) {}
        
        fn end_horizontal_layout(&mut self) {}
        
        fn begin_vertical_layout(&mut self) {}
        
        fn end_vertical_layout(&mut self) {}
        
        fn apply_style(&mut self, _style: &ComponentStyle) {}
        
        fn get_available_rect(&self) -> Rect {
            Rect::new(0.0, 0.0, 800.0, 600.0)
        }
        
        fn supports_feature(&self, _feature: RenderFeature) -> bool {
            true
        }
        
        #[cfg(feature = "dev-ui")]
        fn handle_runtime_update(&mut self, _update: &RuntimeUpdate) -> AdapterResult<()> {
            Ok(())
        }
        
        #[cfg(feature = "dev-ui")]
        fn mark_component_for_tracking(&mut self, _component_id: &str) {}
    }
    
    /// Mock implementation of UIComponent for testing
    struct MockComponent {
        id: String,
        render_count: u32,
    }
    
    impl MockComponent {
        fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                render_count: 0,
            }
        }
    }
    
    impl UIComponent for MockComponent {
        fn render(&mut self, ctx: &mut dyn RenderContext) -> AdapterResult<()> {
            self.render_count += 1;
            ctx.render_text(&format!("Component {}", self.id));
            Ok(())
        }
        
        fn component_id(&self) -> &str {
            &self.id
        }
        
        fn component_type(&self) -> &'static str {
            "MockComponent"
        }
        
        #[cfg(feature = "dev-ui")]
        fn hot_reload_state(&self) -> serde_json::Value {
            serde_json::json!({
                "id": self.id,
                "render_count": self.render_count
            })
        }
        
        #[cfg(feature = "dev-ui")]
        fn restore_state(&mut self, state: serde_json::Value) -> AdapterResult<()> {
            if let Some(count) = state.get("render_count").and_then(|v| v.as_u64()) {
                self.render_count = count as u32;
            }
            Ok(())
        }
    }
    
    #[test]
    fn test_adapter_initialization() {
        let mut adapter = MockAdapter::new("test");
        assert_eq!(adapter.framework_name(), "test");
        assert!(!adapter.initialized);
        
        let config = FrameworkConfig::default();
        adapter.initialize(&config).unwrap();
        assert!(adapter.initialized);
    }
    
    #[test]
    fn test_component_rendering() {
        let mut adapter = MockAdapter::new("test");
        let config = FrameworkConfig::default();
        adapter.initialize(&config).unwrap();
        
        let mut component = MockComponent::new("test-component");
        let mut ctx = MockRenderContext::new();
        
        assert_eq!(component.render_count, 0);
        component.render(&mut ctx).unwrap();
        assert_eq!(component.render_count, 1);
        assert_eq!(ctx.text_count, 1);
    }
    
    #[test]
    fn test_render_context_features() {
        let ctx = MockRenderContext::new();
        assert!(ctx.supports_feature(RenderFeature::CustomFonts));
        
        let rect = ctx.get_available_rect();
        assert_eq!(rect.width, 800.0);
        assert_eq!(rect.height, 600.0);
    }
    
    #[test]
    fn test_framework_config_default() {
        let config = FrameworkConfig::default();
        assert!(!config.development_mode);
        assert_eq!(config.optimization_level, 2);
        assert_eq!(config.settings, serde_json::Value::Null);
    }
    
    #[test]
    fn test_rect_operations() {
        let rect = Rect::new(10.0, 20.0, 100.0, 200.0);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 200.0);
        
        let zero_rect = Rect::zero();
        assert_eq!(zero_rect.x, 0.0);
        assert_eq!(zero_rect.y, 0.0);
        assert_eq!(zero_rect.width, 0.0);
        assert_eq!(zero_rect.height, 0.0);
    }
    
    #[cfg(feature = "dev-ui")]
    #[test]
    fn test_hot_reload_state_preservation() {
        let mut component = MockComponent::new("hot-reload-test");
        component.render_count = 5;
        
        // Save state
        let state = component.hot_reload_state();
        assert_eq!(state["render_count"], 5);
        
        // Reset component
        component.render_count = 0;
        
        // Restore state
        component.restore_state(state).unwrap();
        assert_eq!(component.render_count, 5);
    }
    
    #[cfg(feature = "dev-ui")]
    #[test]
    fn test_runtime_update_handling() {
        let mut adapter = MockAdapter::new("test");
        let config = FrameworkConfig::default();
        adapter.initialize(&config).unwrap();
        
        let update = RuntimeUpdate {
            component_id: "test-component".to_string(),
            update_type: UpdateType::ComponentChange,
            data: serde_json::json!({"property": "value"}),
            timestamp: std::time::SystemTime::now(),
        };
        
        // Should not panic or return error
        adapter.handle_runtime_update(&update).unwrap();
    }
    
    #[test]
    fn test_adapter_error_display() {
        let error = AdapterError::InitializationFailed("Test error".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Initialization failed"));
        assert!(error_string.contains("Test error"));
    }
}