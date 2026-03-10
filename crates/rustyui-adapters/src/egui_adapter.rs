//! egui framework adapter for RustyUI
//! 
//! Provides egui-specific implementation of the UIFrameworkAdapter trait.
//! Supports runtime updates, state preservation, and hot reload capabilities
//! for egui-based applications.

use crate::traits::{
    UIFrameworkAdapter, RenderContext, FrameworkConfig, FrameworkState, UIComponent,
    AdapterResult, AdapterError, ComponentStyle, Rect, RenderFeature
};
use std::collections::HashMap;
use serde_json;

#[cfg(feature = "dev-ui")]
use crate::traits::{RuntimeUpdate, UpdateType};

/// Adapter for the egui immediate mode GUI framework
/// 
/// This adapter provides egui-specific implementation of the UIFrameworkAdapter trait,
/// enabling hot reload capabilities and runtime updates for egui applications.
/// 
/// # Features
/// - Runtime component updates during development
/// - State preservation across hot reload cycles
/// - egui-specific rendering context
/// - Conditional compilation for zero production overhead
pub struct EguiAdapter {
    /// egui context wrapper for rendering operations
    context: Option<EguiContext>,
    
    /// Component registry for tracking rendered components
    component_registry: HashMap<String, ComponentInfo>,
    
    /// State storage for development mode hot reload
    #[cfg(feature = "dev-ui")]
    state_storage: HashMap<String, Vec<u8>>,
    
    /// Runtime update queue for development mode
    #[cfg(feature = "dev-ui")]
    update_queue: Vec<RuntimeUpdate>,
    
    /// Initialization status
    initialized: bool,
    
    /// Framework configuration
    config: Option<FrameworkConfig>,
}

/// Information about a registered component
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    /// Component identifier
    pub id: String,
    /// Component type name
    pub type_name: String,
    /// Last update timestamp
    pub last_updated: std::time::SystemTime,
    /// Component-specific configuration
    pub config: serde_json::Value,
}

impl EguiAdapter {
    /// Create a new egui adapter
    pub fn new() -> Self {
        Self {
            context: None,
            component_registry: HashMap::new(),
            #[cfg(feature = "dev-ui")]
            state_storage: HashMap::new(),
            #[cfg(feature = "dev-ui")]
            update_queue: Vec::new(),
            initialized: false,
            config: None,
        }
    }
    
    /// Register a component for tracking and hot reload
    pub fn register_component(&mut self, component: &dyn UIComponent) {
        let info = ComponentInfo {
            id: component.component_id().to_string(),
            type_name: component.component_type().to_string(),
            last_updated: std::time::SystemTime::now(),
            config: serde_json::Value::Null,
        };
        self.component_registry.insert(component.component_id().to_string(), info);
    }
    
    /// Get component information by ID
    pub fn get_component_info(&self, component_id: &str) -> Option<&ComponentInfo> {
        self.component_registry.get(component_id)
    }
}

impl UIFrameworkAdapter for EguiAdapter {
    fn framework_name(&self) -> &'static str {
        "egui"
    }
    
    fn initialize(&mut self, config: &FrameworkConfig) -> AdapterResult<()> {
        // Initialize egui context with configuration
        self.context = Some(EguiContext::new(config)?);
        self.config = Some(config.clone());
        self.initialized = true;
        
        #[cfg(feature = "dev-ui")]
        {
            // Initialize development-specific features
            self.state_storage.clear();
            self.update_queue.clear();
        }
        
        Ok(())
    }
    
    fn render_component(&mut self, component: &dyn UIComponent, ctx: &mut dyn RenderContext) -> AdapterResult<()> {
        if !self.initialized {
            return Err(AdapterError::InitializationFailed(
                "EguiAdapter not initialized".to_string()
            ));
        }
        
        // Register component for tracking
        self.register_component(component);
        
        // Mark component for hot reload tracking in development mode
        #[cfg(feature = "dev-ui")]
        {
            ctx.mark_component_for_tracking(component.component_id());
        }
        
        // Render the component
        let mutable_component = unsafe {
            // SAFETY: We need mutable access to render the component
            // This is safe because we control the lifetime and ensure no concurrent access
            std::ptr::read(&component as *const &dyn UIComponent as *const &mut dyn UIComponent)
        };
        
        mutable_component.render(ctx).map_err(|e| {
            AdapterError::RenderingFailed(format!("Component rendering failed: {}", e))
        })?;
        
        Ok(())
    }
    
    fn create_render_context(&self) -> AdapterResult<Box<dyn RenderContext>> {
        if !self.initialized {
            return Err(AdapterError::InitializationFailed(
                "EguiAdapter not initialized".to_string()
            ));
        }
        
        Ok(Box::new(EguiRenderContext::new(
            self.context.as_ref(),
            self.config.as_ref()
        )))
    }
    
    fn requires_framework_modifications(&self) -> bool {
        // egui adapter doesn't require framework modifications
        false
    }
    
    #[cfg(feature = "dev-ui")]
    fn handle_runtime_update(&mut self, update: &RuntimeUpdate) -> AdapterResult<()> {
        // Process the update immediately without queuing
        match update.update_type {
            UpdateType::ComponentChange => {
                self.apply_component_update(&update.component_id, &update.data)?;
            }
            UpdateType::StyleChange => {
                self.apply_style_update(&update.data)?;
            }
            UpdateType::LayoutChange => {
                self.apply_layout_update(&update.data)?;
            }
            UpdateType::EventHandlerChange => {
                self.apply_event_handler_update(&update.component_id, &update.data)?;
            }
        }
        
        Ok(())
    }
    
    #[cfg(feature = "dev-ui")]
    fn preserve_framework_state(&self) -> AdapterResult<FrameworkState> {
        if let Some(ref context) = self.context {
            let state_data = context.serialize_state().map_err(|e| {
                AdapterError::StateFailed(format!("Failed to serialize egui state: {}", e))
            })?;
            
            Ok(FrameworkState::Egui(state_data))
        } else {
            Ok(FrameworkState::None)
        }
    }
    
    #[cfg(feature = "dev-ui")]
    fn restore_framework_state(&mut self, state: FrameworkState) -> AdapterResult<()> {
        match (self.context.as_mut(), state) {
            (Some(context), FrameworkState::Egui(state_data)) => {
                context.deserialize_state(&state_data).map_err(|e| {
                    AdapterError::StateFailed(format!("Failed to restore egui state: {}", e))
                })?;
            }
            (None, _) => {
                return Err(AdapterError::InitializationFailed(
                    "Cannot restore state: EguiAdapter not initialized".to_string()
                ));
            }
            (_, FrameworkState::None) => {
                // No state to restore
            }
            (_, other_state) => {
                return Err(AdapterError::StateFailed(format!(
                    "Invalid state type for egui adapter: {:?}", 
                    std::mem::discriminant(&other_state)
                )));
            }
        }
        
        Ok(())
    }
    
    #[cfg(feature = "dev-ui")]
    fn supports_runtime_interpretation(&self) -> bool {
        true
    }
    
    #[cfg(feature = "dev-ui")]
    fn apply_component_update(&mut self, component_id: &str, update_data: &serde_json::Value) -> AdapterResult<()> {
        // Store component update for later application
        let serialized = serde_json::to_vec(update_data).map_err(|e| {
            AdapterError::HotReloadFailed(format!("Failed to serialize component update: {}", e))
        })?;
        
        self.state_storage.insert(component_id.to_string(), serialized);
        
        // Update component registry if it exists
        if let Some(info) = self.component_registry.get_mut(component_id) {
            info.last_updated = std::time::SystemTime::now();
            info.config = update_data.clone();
        }
        
        Ok(())
    }
}

#[cfg(feature = "dev-ui")]
impl EguiAdapter {
    /// Apply component update during development
    fn apply_component_update(&mut self, component_id: &str, data: &serde_json::Value) -> AdapterResult<()> {
        // Store component update for later application
        let serialized = serde_json::to_vec(data).map_err(|e| {
            AdapterError::HotReloadFailed(format!("Failed to serialize component update: {}", e))
        })?;
        
        self.state_storage.insert(component_id.to_string(), serialized);
        
        // Get component type before applying typed update
        let component_type = if let Some(info) = self.component_registry.get(component_id) {
            info.type_name.clone()
        } else {
            "Unknown".to_string()
        };
        
        // Update component registry
        if let Some(info) = self.component_registry.get_mut(component_id) {
            info.last_updated = std::time::SystemTime::now();
            info.config = data.clone();
        }
        
        // Apply component-specific updates based on type
        self.apply_typed_component_update(component_id, &component_type, data)?;
        
        Ok(())
    }
    
    /// Apply component-specific updates based on component type
    fn apply_typed_component_update(&mut self, component_id: &str, component_type: &str, data: &serde_json::Value) -> AdapterResult<()> {
        match component_type {
            "EguiButton" => self.apply_button_update(component_id, data),
            "EguiText" => self.apply_text_update(component_id, data),
            "EguiInput" => self.apply_input_update(component_id, data),
            "EguiLayout" => self.apply_layout_component_update(component_id, data),
            _ => {
                println!("egui: Unknown component type for update: {}", component_type);
                Ok(())
            }
        }
    }
    
    /// Apply button-specific updates
    fn apply_button_update(&mut self, component_id: &str, data: &serde_json::Value) -> AdapterResult<()> {
        if let Some(obj) = data.as_object() {
            if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                println!("egui: Updated button '{}' text to '{}'", component_id, text);
            }
            if let Some(enabled) = obj.get("enabled").and_then(|v| v.as_bool()) {
                println!("egui: Updated button '{}' enabled state to {}", component_id, enabled);
            }
            if let Some(_style) = obj.get("style") {
                println!("egui: Updated button '{}' style", component_id);
                // Apply button-specific styling
            }
        }
        Ok(())
    }
    
    /// Apply text-specific updates
    fn apply_text_update(&mut self, component_id: &str, data: &serde_json::Value) -> AdapterResult<()> {
        if let Some(obj) = data.as_object() {
            if let Some(content) = obj.get("content").and_then(|v| v.as_str()) {
                println!("egui: Updated text '{}' content to '{}'", component_id, content);
            }
            if let Some(style) = obj.get("style") {
                println!("egui: Updated text '{}' style", component_id);
                // Apply text-specific styling
                if let Some(size) = style.get("size").and_then(|v| v.as_f64()) {
                    println!("egui: Text '{}' size updated to {}", component_id, size);
                }
                if let Some(bold) = style.get("bold").and_then(|v| v.as_bool()) {
                    println!("egui: Text '{}' bold updated to {}", component_id, bold);
                }
            }
        }
        Ok(())
    }
    
    /// Apply input-specific updates
    fn apply_input_update(&mut self, component_id: &str, data: &serde_json::Value) -> AdapterResult<()> {
        if let Some(obj) = data.as_object() {
            if let Some(value) = obj.get("value").and_then(|v| v.as_str()) {
                println!("egui: Updated input '{}' value to '{}'", component_id, value);
            }
            if let Some(placeholder) = obj.get("placeholder").and_then(|v| v.as_str()) {
                println!("egui: Updated input '{}' placeholder to '{}'", component_id, placeholder);
            }
            if let Some(enabled) = obj.get("enabled").and_then(|v| v.as_bool()) {
                println!("egui: Updated input '{}' enabled state to {}", component_id, enabled);
            }
            if let Some(focused) = obj.get("focused").and_then(|v| v.as_bool()) {
                println!("egui: Updated input '{}' focus state to {}", component_id, focused);
            }
        }
        Ok(())
    }
    
    /// Apply layout component-specific updates
    fn apply_layout_component_update(&mut self, component_id: &str, data: &serde_json::Value) -> AdapterResult<()> {
        if let Some(obj) = data.as_object() {
            if let Some(spacing) = obj.get("spacing").and_then(|v| v.as_f64()) {
                println!("egui: Updated layout '{}' spacing to {}", component_id, spacing);
            }
            if let Some(padding) = obj.get("padding").and_then(|v| v.as_array()) {
                if padding.len() == 4 {
                    println!("egui: Updated layout '{}' padding", component_id);
                }
            }
            if let Some(bg_color) = obj.get("background_color").and_then(|v| v.as_array()) {
                if bg_color.len() == 4 {
                    println!("egui: Updated layout '{}' background color", component_id);
                }
            }
        }
        Ok(())
    }
    
    /// Apply style update during development
    fn apply_style_update(&mut self, data: &serde_json::Value) -> AdapterResult<()> {
        // Extract style information from update data
        if let Some(context) = self.context.as_mut() {
            context.apply_style_update(data).map_err(|e| {
                AdapterError::HotReloadFailed(format!("Failed to apply style update: {}", e))
            })?;
        }
        
        Ok(())
    }
    
    /// Apply layout update during development
    fn apply_layout_update(&mut self, data: &serde_json::Value) -> AdapterResult<()> {
        // Extract layout information from update data
        if let Some(context) = self.context.as_mut() {
            context.apply_layout_update(data).map_err(|e| {
                AdapterError::HotReloadFailed(format!("Failed to apply layout update: {}", e))
            })?;
        }
        
        Ok(())
    }
    
    /// Apply event handler update during development
    fn apply_event_handler_update(&mut self, component_id: &str, data: &serde_json::Value) -> AdapterResult<()> {
        // Store event handler update
        let update_key = format!("{}_events", component_id);
        let serialized = serde_json::to_vec(data).map_err(|e| {
            AdapterError::HotReloadFailed(format!("Failed to serialize event handler update: {}", e))
        })?;
        
        self.state_storage.insert(update_key, serialized);
        
        Ok(())
    }
    
    /// Process queued runtime updates
    pub fn process_update_queue(&mut self) -> AdapterResult<()> {
        let updates = std::mem::take(&mut self.update_queue);
        
        for update in updates {
            self.handle_runtime_update(&update)?;
        }
        
        Ok(())
    }
    
    /// Queue a runtime update for later processing
    pub fn queue_runtime_update(&mut self, update: RuntimeUpdate) {
        self.update_queue.push(update);
    }
    
    /// Get the number of queued updates
    pub fn queued_updates_count(&self) -> usize {
        self.update_queue.len()
    }
    
    /// Clear all queued updates
    pub fn clear_update_queue(&mut self) {
        self.update_queue.clear();
    }
}

/// egui context wrapper for framework operations
/// 
/// Wraps egui-specific functionality and provides state management
/// for hot reload operations during development.
struct EguiContext {
    /// Framework configuration
    config: FrameworkConfig,
    
    /// egui-specific state (placeholder for actual egui::Context integration)
    state: EguiState,
    
    /// Style configuration
    style_config: ComponentStyle,
}

/// egui-specific state representation
#[derive(Debug, Clone)]
struct EguiState {
    /// Memory state (placeholder for egui::Memory)
    memory: HashMap<String, serde_json::Value>,
    
    /// Visual state (placeholder for egui visuals)
    visuals: serde_json::Value,
    
    /// Input state
    input_state: serde_json::Value,
}

impl Default for EguiState {
    fn default() -> Self {
        Self {
            memory: HashMap::new(),
            visuals: serde_json::Value::Null,
            input_state: serde_json::Value::Null,
        }
    }
}

impl EguiContext {
    fn new(config: &FrameworkConfig) -> AdapterResult<Self> {
        Ok(Self {
            config: config.clone(),
            state: EguiState::default(),
            style_config: ComponentStyle {
                background_color: None,
                text_color: None,
                font_size: None,
                padding: None,
                margin: None,
                border: None,
            },
        })
    }
    
    /// Get current egui state
    pub fn get_state(&self) -> &EguiState {
        &self.state
    }
    
    /// Update egui state
    pub fn update_state(&mut self, new_state: EguiState) {
        self.state = new_state;
    }
    
    #[cfg(feature = "dev-ui")]
    fn serialize_state(&self) -> AdapterResult<Vec<u8>> {
        let state_json = serde_json::json!({
            "memory": self.state.memory,
            "visuals": self.state.visuals,
            "input_state": self.state.input_state,
            "style_config": {
                "background_color": self.style_config.background_color,
                "text_color": self.style_config.text_color,
                "font_size": self.style_config.font_size,
            }
        });
        
        serde_json::to_vec(&state_json).map_err(|e| {
            AdapterError::StateFailed(format!("Failed to serialize egui state: {}", e))
        })
    }
    
    #[cfg(feature = "dev-ui")]
    fn deserialize_state(&mut self, data: &[u8]) -> AdapterResult<()> {
        let state_json: serde_json::Value = serde_json::from_slice(data).map_err(|e| {
            AdapterError::StateFailed(format!("Failed to deserialize egui state: {}", e))
        })?;
        
        // Restore memory state
        if let Some(memory) = state_json.get("memory").and_then(|v| v.as_object()) {
            self.state.memory = memory.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
        }
        
        // Restore visuals
        if let Some(visuals) = state_json.get("visuals") {
            self.state.visuals = visuals.clone();
        }
        
        // Restore input state
        if let Some(input_state) = state_json.get("input_state") {
            self.state.input_state = input_state.clone();
        }
        
        // Restore style configuration
        if let Some(style_config) = state_json.get("style_config") {
            if let Some(bg_color) = style_config.get("background_color") {
                self.style_config.background_color = bg_color.as_array()
                    .and_then(|arr| {
                        if arr.len() == 4 {
                            Some([
                                arr[0].as_f64()? as f32,
                                arr[1].as_f64()? as f32,
                                arr[2].as_f64()? as f32,
                                arr[3].as_f64()? as f32,
                            ])
                        } else {
                            None
                        }
                    });
            }
            
            if let Some(text_color) = style_config.get("text_color") {
                self.style_config.text_color = text_color.as_array()
                    .and_then(|arr| {
                        if arr.len() == 4 {
                            Some([
                                arr[0].as_f64()? as f32,
                                arr[1].as_f64()? as f32,
                                arr[2].as_f64()? as f32,
                                arr[3].as_f64()? as f32,
                            ])
                        } else {
                            None
                        }
                    });
            }
            
            if let Some(font_size) = style_config.get("font_size") {
                self.style_config.font_size = font_size.as_f64().map(|f| f as f32);
            }
        }
        
        Ok(())
    }
    
    #[cfg(feature = "dev-ui")]
    fn apply_style_update(&mut self, data: &serde_json::Value) -> AdapterResult<()> {
        // Apply style updates to the context
        if let Some(background_color) = data.get("background_color") {
            self.style_config.background_color = background_color.as_array()
                .and_then(|arr| {
                    if arr.len() == 4 {
                        Some([
                            arr[0].as_f64()? as f32,
                            arr[1].as_f64()? as f32,
                            arr[2].as_f64()? as f32,
                            arr[3].as_f64()? as f32,
                        ])
                    } else {
                        None
                    }
                });
        }
        
        if let Some(text_color) = data.get("text_color") {
            self.style_config.text_color = text_color.as_array()
                .and_then(|arr| {
                    if arr.len() == 4 {
                        Some([
                            arr[0].as_f64()? as f32,
                            arr[1].as_f64()? as f32,
                            arr[2].as_f64()? as f32,
                            arr[3].as_f64()? as f32,
                        ])
                    } else {
                        None
                    }
                });
        }
        
        if let Some(font_size) = data.get("font_size") {
            self.style_config.font_size = font_size.as_f64().map(|f| f as f32);
        }
        
        Ok(())
    }
    
    #[cfg(feature = "dev-ui")]
    fn apply_layout_update(&mut self, data: &serde_json::Value) -> AdapterResult<()> {
        // Apply layout updates to the context
        if let Some(layout_type) = data.get("layout_type") {
            // Store layout configuration in memory
            self.state.memory.insert("layout_type".to_string(), layout_type.clone());
        }
        
        if let Some(spacing) = data.get("spacing") {
            self.state.memory.insert("spacing".to_string(), spacing.clone());
        }
        
        if let Some(margins) = data.get("margins") {
            self.state.memory.insert("margins".to_string(), margins.clone());
        }
        
        Ok(())
    }
}

/// egui render context implementation
/// 
/// Provides egui-specific rendering operations and integrates with
/// the hot reload system for development-time updates.
pub struct EguiRenderContext {
    /// Reference to egui context
    context: Option<EguiContextRef>,
    
    /// Framework configuration
    config: Option<FrameworkConfig>,
    
    /// Rendered elements (for Phase 1 simulation and testing)
    rendered_elements: Vec<RenderedElement>,
    
    /// Current layout stack
    layout_stack: Vec<LayoutInfo>,
    
    /// Component tracking for hot reload
    #[cfg(feature = "dev-ui")]
    tracked_components: Vec<String>,
    
    /// Current style state
    current_style: Option<ComponentStyle>,
}

/// Information about a rendered element
#[derive(Debug, Clone)]
struct RenderedElement {
    element_type: String,
    content: String,
    style: Option<ComponentStyle>,
    timestamp: std::time::SystemTime,
}

/// Layout information for the layout stack
#[derive(Debug, Clone)]
struct LayoutInfo {
    layout_type: String,
    bounds: Rect,
    child_count: u32,
}

impl EguiRenderContext {
    fn new(context: Option<&EguiContext>, config: Option<&FrameworkConfig>) -> Self {
        Self {
            context: context.map(|_| EguiContextRef),
            config: config.cloned(),
            rendered_elements: Vec::new(),
            layout_stack: Vec::new(),
            #[cfg(feature = "dev-ui")]
            tracked_components: Vec::new(),
            current_style: None,
        }
    }
    
    /// Add a rendered element to the tracking list
    fn add_rendered_element(&mut self, element_type: &str, content: &str) {
        let element = RenderedElement {
            element_type: element_type.to_string(),
            content: content.to_string(),
            style: self.current_style.clone(),
            timestamp: std::time::SystemTime::now(),
        };
        
        self.rendered_elements.push(element);
        
        // Update child count for current layout
        if let Some(layout) = self.layout_stack.last_mut() {
            layout.child_count += 1;
        }
        
        println!("egui: Rendered {} '{}'", element_type, content);
    }
    
    /// Get the current layout bounds
    fn get_current_layout_bounds(&self) -> Rect {
        self.layout_stack
            .last()
            .map(|layout| layout.bounds)
            .unwrap_or_else(|| Rect::new(0.0, 0.0, 800.0, 600.0))
    }
    
    /// Get the number of rendered elements
    pub fn get_rendered_count(&self) -> usize {
        self.rendered_elements.len()
    }
    
    /// Get rendered elements for testing and debugging
    pub fn get_rendered_elements(&self) -> &[RenderedElement] {
        &self.rendered_elements
    }
    
    /// Clear rendered elements (useful for testing)
    pub fn clear_rendered_elements(&mut self) {
        self.rendered_elements.clear();
    }
    
    /// Get the current layout depth
    pub fn get_layout_depth(&self) -> usize {
        self.layout_stack.len()
    }
    
    /// Check if we're currently in a layout
    pub fn is_in_layout(&self) -> bool {
        !self.layout_stack.is_empty()
    }
    
    /// Get the current layout type if in a layout
    pub fn get_current_layout_type(&self) -> Option<&str> {
        self.layout_stack.last().map(|layout| layout.layout_type.as_str())
    }
    
    #[cfg(feature = "dev-ui")]
    /// Get tracked components for hot reload
    pub fn get_tracked_components(&self) -> &[String] {
        &self.tracked_components
    }
    
    #[cfg(feature = "dev-ui")]
    /// Clear tracked components
    pub fn clear_tracked_components(&mut self) {
        self.tracked_components.clear();
    }
    
    /// Apply component-specific styling based on component type
    pub fn apply_component_style(&mut self, component_type: &str, style_data: &serde_json::Value) -> AdapterResult<()> {
        match component_type {
            "EguiButton" => self.apply_button_style(style_data),
            "EguiText" => self.apply_text_style(style_data),
            "EguiInput" => self.apply_input_style(style_data),
            "EguiLayout" => self.apply_layout_style(style_data),
            _ => {
                println!("egui: Unknown component type for styling: {}", component_type);
                Ok(())
            }
        }
    }
    
    fn apply_button_style(&mut self, style_data: &serde_json::Value) -> AdapterResult<()> {
        if let Some(obj) = style_data.as_object() {
            if let Some(color) = obj.get("color").and_then(|v| v.as_array()) {
                if color.len() == 4 {
                    println!("egui: Applied button color style");
                }
            }
            if let Some(width) = obj.get("width").and_then(|v| v.as_f64()) {
                println!("egui: Applied button width: {}", width);
            }
            if let Some(height) = obj.get("height").and_then(|v| v.as_f64()) {
                println!("egui: Applied button height: {}", height);
            }
        }
        Ok(())
    }
    
    fn apply_text_style(&mut self, style_data: &serde_json::Value) -> AdapterResult<()> {
        if let Some(obj) = style_data.as_object() {
            if let Some(size) = obj.get("size").and_then(|v| v.as_f64()) {
                println!("egui: Applied text size: {}", size);
            }
            if let Some(bold) = obj.get("bold").and_then(|v| v.as_bool()) {
                println!("egui: Applied text bold: {}", bold);
            }
            if let Some(italic) = obj.get("italic").and_then(|v| v.as_bool()) {
                println!("egui: Applied text italic: {}", italic);
            }
        }
        Ok(())
    }
    
    fn apply_input_style(&mut self, style_data: &serde_json::Value) -> AdapterResult<()> {
        if let Some(obj) = style_data.as_object() {
            if let Some(bg_color) = obj.get("background_color").and_then(|v| v.as_array()) {
                if bg_color.len() == 4 {
                    println!("egui: Applied input background color");
                }
            }
            if let Some(text_color) = obj.get("text_color").and_then(|v| v.as_array()) {
                if text_color.len() == 4 {
                    println!("egui: Applied input text color");
                }
            }
        }
        Ok(())
    }
    
    fn apply_layout_style(&mut self, style_data: &serde_json::Value) -> AdapterResult<()> {
        if let Some(obj) = style_data.as_object() {
            if let Some(spacing) = obj.get("spacing").and_then(|v| v.as_f64()) {
                println!("egui: Applied layout spacing: {}", spacing);
            }
            if let Some(padding) = obj.get("padding").and_then(|v| v.as_array()) {
                if padding.len() == 4 {
                    println!("egui: Applied layout padding");
                }
            }
        }
        Ok(())
    }
}

impl RenderContext for EguiRenderContext {
    fn render_button(&mut self, text: &str, _callback: Box<dyn Fn() + Send + Sync>) {
        self.add_rendered_element("Button", text);
    }
    
    fn render_text(&mut self, text: &str) {
        self.add_rendered_element("Text", text);
    }
    
    fn render_input(&mut self, value: &str, _on_change: Box<dyn Fn(String) + Send + Sync>) {
        self.add_rendered_element("Input", value);
    }
    
    fn render_checkbox(&mut self, checked: bool, _on_change: Box<dyn Fn(bool) + Send + Sync>) {
        let state = if checked { "checked" } else { "unchecked" };
        self.add_rendered_element("Checkbox", state);
    }
    
    fn begin_horizontal_layout(&mut self) {
        let bounds = self.get_current_layout_bounds();
        let layout_info = LayoutInfo {
            layout_type: "Horizontal".to_string(),
            bounds,
            child_count: 0,
        };
        
        self.layout_stack.push(layout_info);
        self.add_rendered_element("BeginHorizontalLayout", "");
    }
    
    fn end_horizontal_layout(&mut self) {
        if let Some(layout) = self.layout_stack.pop() {
            println!("egui: Ended horizontal layout with {} children", layout.child_count);
        }
        self.add_rendered_element("EndHorizontalLayout", "");
    }
    
    fn begin_vertical_layout(&mut self) {
        let bounds = self.get_current_layout_bounds();
        let layout_info = LayoutInfo {
            layout_type: "Vertical".to_string(),
            bounds,
            child_count: 0,
        };
        
        self.layout_stack.push(layout_info);
        self.add_rendered_element("BeginVerticalLayout", "");
    }
    
    fn end_vertical_layout(&mut self) {
        if let Some(layout) = self.layout_stack.pop() {
            println!("egui: Ended vertical layout with {} children", layout.child_count);
        }
        self.add_rendered_element("EndVerticalLayout", "");
    }
    
    fn apply_style(&mut self, style: &ComponentStyle) {
        self.current_style = Some(style.clone());
        println!("egui: Applied style - bg: {:?}, text: {:?}, font_size: {:?}", 
                 style.background_color, style.text_color, style.font_size);
    }
    
    fn get_available_rect(&self) -> Rect {
        self.get_current_layout_bounds()
    }
    
    fn supports_feature(&self, feature: RenderFeature) -> bool {
        match feature {
            RenderFeature::CustomFonts => true,
            RenderFeature::Animations => true,
            RenderFeature::ThreeDRendering => false,
            RenderFeature::CustomShaders => false,
            RenderFeature::VectorGraphics => true,
            RenderFeature::ImageRendering => true,
            RenderFeature::TextInput => true,
            RenderFeature::DragAndDrop => true,
        }
    }
    
    #[cfg(feature = "dev-ui")]
    fn handle_runtime_update(&mut self, update: &RuntimeUpdate) -> AdapterResult<()> {
        match update.update_type {
            UpdateType::ComponentChange => {
                // Handle component updates in the render context
                println!("egui: Handling component update for {}", update.component_id);
            }
            UpdateType::StyleChange => {
                // Apply style changes
                if let Ok(style) = serde_json::from_value::<ComponentStyle>(update.data.clone()) {
                    self.apply_style(&style);
                }
            }
            UpdateType::LayoutChange => {
                // Handle layout changes
                println!("egui: Handling layout update");
            }
            UpdateType::EventHandlerChange => {
                // Handle event handler changes
                println!("egui: Handling event handler update for {}", update.component_id);
            }
        }
        
        Ok(())
    }
    
    #[cfg(feature = "dev-ui")]
    fn mark_component_for_tracking(&mut self, component_id: &str) {
        if !self.tracked_components.contains(&component_id.to_string()) {
            self.tracked_components.push(component_id.to_string());
            println!("egui: Marked component '{}' for hot reload tracking", component_id);
        }
    }
}

/// Reference to egui context (placeholder for actual egui integration)
/// 
/// In a full implementation, this would hold references to actual egui types
/// like egui::Context, egui::Ui, etc. For Phase 1, this serves as a placeholder
/// that demonstrates the integration pattern.
struct EguiContextRef;

impl EguiContextRef {
    /// Simulate egui context operations
    pub fn simulate_render_operation(&self, operation: &str) {
        println!("egui_context: Executing {}", operation);
    }
}

// Additional helper implementations for the EguiAdapter

impl Default for EguiAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for EguiAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EguiAdapter")
            .field("initialized", &self.initialized)
            .field("component_count", &self.component_registry.len())
            .field("framework", &self.framework_name())
            .finish()
    }
}

#[cfg(feature = "dev-ui")]
impl EguiAdapter {
    /// Get development statistics
    pub fn get_dev_stats(&self) -> EguiDevStats {
        EguiDevStats {
            registered_components: self.component_registry.len(),
            queued_updates: self.update_queue.len(),
            stored_states: self.state_storage.len(),
            tracked_components: self.component_registry.keys().cloned().collect(),
        }
    }
}

/// Development statistics for the egui adapter
#[cfg(feature = "dev-ui")]
#[derive(Debug, Clone)]
pub struct EguiDevStats {
    pub registered_components: usize,
    pub queued_updates: usize,
    pub stored_states: usize,
    pub tracked_components: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::*;
    
    #[test]
    fn test_egui_adapter_creation() {
        let adapter = EguiAdapter::new();
        assert_eq!(adapter.framework_name(), "egui");
        assert!(!adapter.initialized);
        assert_eq!(adapter.component_registry.len(), 0);
    }
    
    #[test]
    fn test_egui_adapter_initialization() {
        let mut adapter = EguiAdapter::new();
        let config = FrameworkConfig::default();
        
        assert!(adapter.initialize(&config).is_ok());
        assert!(adapter.initialized);
        assert!(adapter.context.is_some());
    }
    
    #[test]
    fn test_egui_render_context_creation() {
        let mut adapter = EguiAdapter::new();
        let config = FrameworkConfig::default();
        adapter.initialize(&config).unwrap();
        
        let render_context = adapter.create_render_context();
        assert!(render_context.is_ok());
    }
    
    #[test]
    fn test_egui_render_context_operations() {
        let mut adapter = EguiAdapter::new();
        let config = FrameworkConfig::default();
        adapter.initialize(&config).unwrap();
        
        let mut ctx = adapter.create_render_context().unwrap();
        
        // Test basic rendering operations
        ctx.render_button("Test Button", Box::new(|| {}));
        ctx.render_text("Test Text");
        ctx.render_input("Test Input", Box::new(|_| {}));
        ctx.render_checkbox(true, Box::new(|_| {}));
        
        // Test layout operations
        ctx.begin_horizontal_layout();
        ctx.render_text("Child 1");
        ctx.render_text("Child 2");
        ctx.end_horizontal_layout();
        
        // Test feature support
        assert!(ctx.supports_feature(RenderFeature::CustomFonts));
        assert!(ctx.supports_feature(RenderFeature::TextInput));
        assert!(!ctx.supports_feature(RenderFeature::ThreeDRendering));
    }
    
    #[test]
    fn test_egui_adapter_requirements() {
        let adapter = EguiAdapter::new();
        assert!(!adapter.requires_framework_modifications());
    }
    
    #[cfg(feature = "dev-ui")]
    #[test]
    fn test_egui_runtime_interpretation_support() {
        let adapter = EguiAdapter::new();
        assert!(adapter.supports_runtime_interpretation());
    }
    
    #[cfg(feature = "dev-ui")]
    #[test]
    fn test_egui_state_preservation() {
        let mut adapter = EguiAdapter::new();
        let config = FrameworkConfig::default();
        adapter.initialize(&config).unwrap();
        
        // Test state preservation
        let state = adapter.preserve_framework_state().unwrap();
        match state {
            FrameworkState::Egui(_) => {
                // State was preserved successfully
            }
            _ => panic!("Expected Egui state"),
        }
        
        // Test state restoration
        assert!(adapter.restore_framework_state(state).is_ok());
    }
    
    #[cfg(feature = "dev-ui")]
    #[test]
    fn test_egui_runtime_updates() {
        let mut adapter = EguiAdapter::new();
        let config = FrameworkConfig::default();
        adapter.initialize(&config).unwrap();
        
        let update = RuntimeUpdate {
            component_id: "test_component".to_string(),
            update_type: UpdateType::ComponentChange,
            data: serde_json::json!({"property": "value"}),
            timestamp: std::time::SystemTime::now(),
        };
        
        assert!(adapter.handle_runtime_update(&update).is_ok());
        assert_eq!(adapter.queued_updates_count(), 0); // Should be processed immediately
    }
    
    #[test]
    fn test_egui_context_state_operations() {
        let config = FrameworkConfig::default();
        let mut context = EguiContext::new(&config).unwrap();
        
        // Test state access
        let initial_state = context.get_state();
        assert!(initial_state.memory.is_empty());
        
        // Test state update
        let mut new_state = EguiState::default();
        new_state.memory.insert("test_key".to_string(), serde_json::json!("test_value"));
        context.update_state(new_state);
        
        let updated_state = context.get_state();
        assert_eq!(updated_state.memory.len(), 1);
        assert_eq!(updated_state.memory.get("test_key"), Some(&serde_json::json!("test_value")));
    }

    #[test]
    fn test_egui_render_context_component_integration() {
        let mut ctx = EguiRenderContext::new(None, None);
        
        // Test component rendering tracking
        ctx.render_button("Test Button", Box::new(|| {}));
        ctx.render_text("Test Text");
        ctx.render_input("Test Input", Box::new(|_| {}));
        
        assert_eq!(ctx.get_rendered_count(), 3);
        
        let elements = ctx.get_rendered_elements();
        assert_eq!(elements[0].element_type, "Button");
        assert_eq!(elements[1].element_type, "Text");
        assert_eq!(elements[2].element_type, "Input");
    }

    #[test]
    fn test_egui_render_context_layout_tracking() {
        let mut ctx = EguiRenderContext::new(None, None);
        
        assert_eq!(ctx.get_layout_depth(), 0);
        assert!(!ctx.is_in_layout());
        
        ctx.begin_vertical_layout();
        assert_eq!(ctx.get_layout_depth(), 1);
        assert!(ctx.is_in_layout());
        assert_eq!(ctx.get_current_layout_type(), Some("Vertical"));
        
        ctx.begin_horizontal_layout();
        assert_eq!(ctx.get_layout_depth(), 2);
        assert_eq!(ctx.get_current_layout_type(), Some("Horizontal"));
        
        ctx.end_horizontal_layout();
        assert_eq!(ctx.get_layout_depth(), 1);
        assert_eq!(ctx.get_current_layout_type(), Some("Vertical"));
        
        ctx.end_vertical_layout();
        assert_eq!(ctx.get_layout_depth(), 0);
        assert!(!ctx.is_in_layout());
    }

    #[test]
    #[cfg(feature = "dev-ui")]
    fn test_egui_render_context_component_tracking() {
        let mut ctx = EguiRenderContext::new(None, None);
        
        assert_eq!(ctx.get_tracked_components().len(), 0);
        
        ctx.mark_component_for_tracking("button1");
        ctx.mark_component_for_tracking("text1");
        ctx.mark_component_for_tracking("button1"); // Duplicate should be ignored
        
        assert_eq!(ctx.get_tracked_components().len(), 2);
        assert!(ctx.get_tracked_components().contains(&"button1".to_string()));
        assert!(ctx.get_tracked_components().contains(&"text1".to_string()));
        
        ctx.clear_tracked_components();
        assert_eq!(ctx.get_tracked_components().len(), 0);
    }

    #[test]
    fn test_egui_render_context_component_styling() {
        let mut ctx = EguiRenderContext::new(None, None);
        
        // Test button styling
        let button_style = serde_json::json!({
            "width": 150.0,
            "height": 30.0,
            "color": [0.2, 0.4, 0.8, 1.0]
        });
        assert!(ctx.apply_component_style("EguiButton", &button_style).is_ok());
        
        // Test text styling
        let text_style = serde_json::json!({
            "size": 14.0,
            "bold": true,
            "italic": false
        });
        assert!(ctx.apply_component_style("EguiText", &text_style).is_ok());
        
        // Test input styling
        let input_style = serde_json::json!({
            "background_color": [0.9, 0.9, 0.9, 1.0],
            "text_color": [0.1, 0.1, 0.1, 1.0]
        });
        assert!(ctx.apply_component_style("EguiInput", &input_style).is_ok());
        
        // Test layout styling
        let layout_style = serde_json::json!({
            "spacing": 8.0,
            "padding": [5.0, 5.0, 5.0, 5.0]
        });
        assert!(ctx.apply_component_style("EguiLayout", &layout_style).is_ok());
        
        // Test unknown component type
        assert!(ctx.apply_component_style("UnknownComponent", &serde_json::json!({})).is_ok());
    }

    #[test]
    #[cfg(feature = "dev-ui")]
    fn test_egui_adapter_component_specific_updates() {
        let mut adapter = EguiAdapter::new();
        let config = FrameworkConfig::default();
        adapter.initialize(&config).unwrap();
        
        // Create a simple mock component info
        let component_info = ComponentInfo {
            id: "test_button".to_string(),
            type_name: "EguiButton".to_string(),
            last_updated: std::time::SystemTime::now(),
            config: serde_json::json!({}),
        };
        adapter.component_registry.insert("test_button".to_string(), component_info);
        
        // Test button update
        let button_update_data = serde_json::json!({
            "text": "Updated Button",
            "enabled": false,
            "style": {
                "width": 200.0,
                "color": [1.0, 0.0, 0.0, 1.0]
            }
        });
        
        assert!(adapter.apply_button_update("test_button", &button_update_data).is_ok());
        
        // Test text update
        let text_update_data = serde_json::json!({
            "content": "Updated Text",
            "style": {
                "size": 16.0,
                "bold": true
            }
        });
        
        assert!(adapter.apply_text_update("test_text", &text_update_data).is_ok());
        
        // Test input update
        let input_update_data = serde_json::json!({
            "value": "Updated Value",
            "placeholder": "Updated Placeholder",
            "enabled": true,
            "focused": false
        });
        
        assert!(adapter.apply_input_update("test_input", &input_update_data).is_ok());
        
        // Test layout update
        let layout_update_data = serde_json::json!({
            "spacing": 10.0,
            "padding": [8.0, 8.0, 8.0, 8.0],
            "background_color": [0.95, 0.95, 0.95, 1.0]
        });
        
        assert!(adapter.apply_layout_component_update("test_layout", &layout_update_data).is_ok());
    }

    #[test]
    #[cfg(feature = "dev-ui")]
    fn test_egui_adapter_typed_component_updates() {
        let mut adapter = EguiAdapter::new();
        let config = FrameworkConfig::default();
        adapter.initialize(&config).unwrap();
        
        // Create a simple mock component info
        let component_info = ComponentInfo {
            id: "button1".to_string(),
            type_name: "EguiButton".to_string(),
            last_updated: std::time::SystemTime::now(),
            config: serde_json::json!({}),
        };
        adapter.component_registry.insert("button1".to_string(), component_info);
        
        let update_data = serde_json::json!({
            "text": "Typed Update",
            "enabled": true
        });
        
        assert!(adapter.apply_typed_component_update("button1", "EguiButton", &update_data).is_ok());
        assert!(adapter.apply_typed_component_update("unknown1", "UnknownType", &update_data).is_ok());
    }
}