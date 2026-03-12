//! Example egui components with hot reload support
//! 
//! This module provides concrete implementations of UI components for the egui framework,
//! demonstrating hot reload capabilities and state preservation.

use crate::traits::{UIComponent, RenderContext, AdapterResult};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};

/// A button component with hot reload support
#[derive(Clone)]
pub struct EguiButton {
    id: String,
    state: EguiButtonState,
    click_handler: Option<Arc<Mutex<dyn Fn() + Send + Sync>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EguiButtonState {
    pub text: String,
    pub enabled: bool,
    pub click_count: u32,
    pub style: ButtonStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonStyle {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub color: Option<[f32; 4]>, // RGBA
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            color: None,
        }
    }
}

impl EguiButton {
    pub fn new(id: String, text: String) -> Self {
        Self {
            id,
            state: EguiButtonState {
                text,
                enabled: true,
                click_count: 0,
                style: ButtonStyle::default(),
            },
            click_handler: None,
        }
    }

    pub fn with_click_handler<F>(mut self, handler: F) -> Self 
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.click_handler = Some(Arc::new(Mutex::new(handler)));
        self
    }

    pub fn set_text(&mut self, text: String) {
        self.state.text = text;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.state.enabled = enabled;
    }

    pub fn set_style(&mut self, style: ButtonStyle) {
        self.state.style = style;
    }

    pub fn click(&mut self) {
        if self.state.enabled {
            self.state.click_count += 1;
            if let Some(handler) = &self.click_handler {
                if let Ok(handler) = handler.lock() {
                    handler();
                }
            }
        }
    }

    pub fn get_click_count(&self) -> u32 {
        self.state.click_count
    }
}

impl UIComponent for EguiButton {
    fn render(&mut self, ctx: &mut dyn RenderContext) -> AdapterResult<()> {
        // Create a callback that captures the button's click behavior
        let click_count = self.state.click_count;
        let _enabled = self.state.enabled;
        
        ctx.render_button(&self.state.text, Box::new(move || {
            // This would be handled by the actual egui integration
            println!("Button clicked! Count: {}", click_count + 1);
        }));
        
        Ok(())
    }

    fn component_id(&self) -> &str {
        &self.id
    }

    fn component_type(&self) -> &'static str {
        "EguiButton"
    }

    #[cfg(feature = "dev-ui")]
    fn hot_reload_state(&self) -> serde_json::Value {
        serde_json::to_value(&self.state).unwrap_or(serde_json::Value::Null)
    }

    #[cfg(feature = "dev-ui")]
    fn restore_state(&mut self, state: serde_json::Value) -> AdapterResult<()> {
        if let Ok(button_state) = serde_json::from_value::<EguiButtonState>(state) {
            self.state = button_state;
        }
        Ok(())
    }

    #[cfg(feature = "dev-ui")]
    fn interpretation_hint(&self) -> InterpretationHint {
        InterpretationHint::PreferRhai // Buttons are simple and work well with Rhai
    }

    #[cfg(feature = "dev-ui")]
    fn handle_runtime_update(&mut self, update: &RuntimeUpdate) -> AdapterResult<()> {
        if let Some(data) = update.data.as_object() {
            if let Some(text) = data.get("text").and_then(|v| v.as_str()) {
                self.state.text = text.to_string();
            }
            if let Some(enabled) = data.get("enabled").and_then(|v| v.as_bool()) {
                self.state.enabled = enabled;
            }
            if let Some(style_data) = data.get("style") {
                if let Ok(style) = serde_json::from_value::<ButtonStyle>(style_data.clone()) {
                    self.state.style = style;
                }
            }
        }
        Ok(())
    }
}

/// A text display component with hot reload support
#[derive(Debug, Clone)]
pub struct EguiText {
    id: String,
    state: EguiTextState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EguiTextState {
    pub content: String,
    pub style: TextStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub size: Option<f32>,
    pub color: Option<[f32; 4]>, // RGBA
    pub bold: bool,
    pub italic: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: None,
            color: None,
            bold: false,
            italic: false,
        }
    }
}

impl EguiText {
    pub fn new(id: String, content: String) -> Self {
        Self {
            id,
            state: EguiTextState {
                content,
                style: TextStyle::default(),
            },
        }
    }

    pub fn set_content(&mut self, content: String) {
        self.state.content = content;
    }

    pub fn set_style(&mut self, style: TextStyle) {
        self.state.style = style;
    }

    pub fn get_content(&self) -> &str {
        &self.state.content
    }
}

impl UIComponent for EguiText {
    fn render(&mut self, ctx: &mut dyn RenderContext) -> AdapterResult<()> {
        ctx.render_text(&self.state.content);
        Ok(())
    }

    fn component_id(&self) -> &str {
        &self.id
    }

    fn component_type(&self) -> &'static str {
        "EguiText"
    }

    #[cfg(feature = "dev-ui")]
    fn hot_reload_state(&self) -> serde_json::Value {
        serde_json::to_value(&self.state).unwrap_or(serde_json::Value::Null)
    }

    #[cfg(feature = "dev-ui")]
    fn restore_state(&mut self, state: serde_json::Value) -> AdapterResult<()> {
        if let Ok(text_state) = serde_json::from_value::<EguiTextState>(state) {
            self.state = text_state;
        }
        Ok(())
    }

    #[cfg(feature = "dev-ui")]
    fn interpretation_hint(&self) -> InterpretationHint {
        InterpretationHint::PreferRhai // Text is simple and works well with Rhai
    }

    #[cfg(feature = "dev-ui")]
    fn handle_runtime_update(&mut self, update: &RuntimeUpdate) -> AdapterResult<()> {
        if let Some(data) = update.data.as_object() {
            if let Some(content) = data.get("content").and_then(|v| v.as_str()) {
                self.state.content = content.to_string();
            }
            if let Some(style_data) = data.get("style") {
                if let Ok(style) = serde_json::from_value::<TextStyle>(style_data.clone()) {
                    self.state.style = style;
                }
            }
        }
        Ok(())
    }
}

/// An input field component with hot reload support
#[derive(Clone)]
pub struct EguiInput {
    id: String,
    state: EguiInputState,
    change_handler: Option<Arc<Mutex<dyn Fn(String) + Send + Sync>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EguiInputState {
    pub value: String,
    pub placeholder: String,
    pub focused: bool,
    pub enabled: bool,
    pub style: InputStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputStyle {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub background_color: Option<[f32; 4]>,
    pub text_color: Option<[f32; 4]>,
}

impl Default for InputStyle {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            background_color: None,
            text_color: None,
        }
    }
}

impl EguiInput {
    pub fn new(id: String, placeholder: String) -> Self {
        Self {
            id,
            state: EguiInputState {
                value: String::new(),
                placeholder,
                focused: false,
                enabled: true,
                style: InputStyle::default(),
            },
            change_handler: None,
        }
    }

    pub fn with_change_handler<F>(mut self, handler: F) -> Self 
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.change_handler = Some(Arc::new(Mutex::new(handler)));
        self
    }

    pub fn set_value(&mut self, value: String) {
        let old_value = self.state.value.clone();
        self.state.value = value.clone();
        
        if old_value != value {
            if let Some(handler) = &self.change_handler {
                if let Ok(handler) = handler.lock() {
                    handler(value);
                }
            }
        }
    }

    pub fn set_placeholder(&mut self, placeholder: String) {
        self.state.placeholder = placeholder;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.state.focused = focused;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.state.enabled = enabled;
    }

    pub fn set_style(&mut self, style: InputStyle) {
        self.state.style = style;
    }

    pub fn get_value(&self) -> &str {
        &self.state.value
    }
}

impl UIComponent for EguiInput {
    fn render(&mut self, ctx: &mut dyn RenderContext) -> AdapterResult<()> {
        let current_value = self.state.value.clone();
        
        ctx.render_input(&current_value, Box::new(move |new_value| {
            // This would be handled by the actual egui integration
            println!("Input changed to: {}", new_value);
        }));
        
        Ok(())
    }

    fn component_id(&self) -> &str {
        &self.id
    }

    fn component_type(&self) -> &'static str {
        "EguiInput"
    }

    #[cfg(feature = "dev-ui")]
    fn hot_reload_state(&self) -> serde_json::Value {
        serde_json::to_value(&self.state).unwrap_or(serde_json::Value::Null)
    }

    #[cfg(feature = "dev-ui")]
    fn restore_state(&mut self, state: serde_json::Value) -> AdapterResult<()> {
        if let Ok(input_state) = serde_json::from_value::<EguiInputState>(state) {
            self.state = input_state;
        }
        Ok(())
    }

    #[cfg(feature = "dev-ui")]
    fn interpretation_hint(&self) -> InterpretationHint {
        InterpretationHint::PreferAST // Input handling might benefit from AST interpretation
    }

    #[cfg(feature = "dev-ui")]
    fn handle_runtime_update(&mut self, update: &RuntimeUpdate) -> AdapterResult<()> {
        if let Some(data) = update.data.as_object() {
            if let Some(value) = data.get("value").and_then(|v| v.as_str()) {
                self.state.value = value.to_string();
            }
            if let Some(placeholder) = data.get("placeholder").and_then(|v| v.as_str()) {
                self.state.placeholder = placeholder.to_string();
            }
            if let Some(enabled) = data.get("enabled").and_then(|v| v.as_bool()) {
                self.state.enabled = enabled;
            }
            if let Some(focused) = data.get("focused").and_then(|v| v.as_bool()) {
                self.state.focused = focused;
            }
            if let Some(style_data) = data.get("style") {
                if let Ok(style) = serde_json::from_value::<InputStyle>(style_data.clone()) {
                    self.state.style = style;
                }
            }
        }
        Ok(())
    }
}

/// A layout container component with hot reload support
pub struct EguiLayout {
    id: String,
    state: EguiLayoutState,
    children: Vec<Box<dyn UIComponent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EguiLayoutState {
    pub layout_type: LayoutDirection,
    pub spacing: f32,
    pub padding: [f32; 4], // top, right, bottom, left
    pub background_color: Option<[f32; 4]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

impl EguiLayout {
    pub fn new(id: String, layout_type: LayoutDirection) -> Self {
        Self {
            id,
            state: EguiLayoutState {
                layout_type,
                spacing: 4.0,
                padding: [0.0, 0.0, 0.0, 0.0],
                background_color: None,
            },
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: Box<dyn UIComponent>) {
        self.children.push(child);
    }

    pub fn set_spacing(&mut self, spacing: f32) {
        self.state.spacing = spacing;
    }

    pub fn set_padding(&mut self, padding: [f32; 4]) {
        self.state.padding = padding;
    }

    pub fn set_background_color(&mut self, color: Option<[f32; 4]>) {
        self.state.background_color = color;
    }

    pub fn get_children_count(&self) -> usize {
        self.children.len()
    }
}

impl UIComponent for EguiLayout {
    fn render(&mut self, ctx: &mut dyn RenderContext) -> AdapterResult<()> {
        match self.state.layout_type {
            LayoutDirection::Horizontal => {
                ctx.begin_horizontal_layout();
                for child in &mut self.children {
                    child.render(ctx)?;
                }
                ctx.end_horizontal_layout();
            }
            LayoutDirection::Vertical => {
                ctx.begin_vertical_layout();
                for child in &mut self.children {
                    child.render(ctx)?;
                }
                ctx.end_vertical_layout();
            }
        }
        Ok(())
    }

    fn component_id(&self) -> &str {
        &self.id
    }

    fn component_type(&self) -> &'static str {
        "EguiLayout"
    }

    #[cfg(feature = "dev-ui")]
    fn hot_reload_state(&self) -> serde_json::Value {
        let mut state_obj = serde_json::to_value(&self.state).unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        
        // Include children states
        let children_states: Vec<serde_json::Value> = self.children
            .iter()
            .map(|child| child.hot_reload_state())
            .collect();
        
        if let Some(obj) = state_obj.as_object_mut() {
            obj.insert("children".to_string(), serde_json::Value::Array(children_states));
        }
        
        state_obj
    }

    #[cfg(feature = "dev-ui")]
    fn restore_state(&mut self, state: serde_json::Value) -> AdapterResult<()> {
        if let Ok(layout_state) = serde_json::from_value::<EguiLayoutState>(state.clone()) {
            self.state = layout_state;
        }
        
        // Restore children states if available
        if let Some(children_states) = state.get("children").and_then(|v| v.as_array()) {
            for (i, child_state) in children_states.iter().enumerate() {
                if let Some(child) = self.children.get_mut(i) {
                    let _ = child.restore_state(child_state.clone());
                }
            }
        }
        
        Ok(())
    }

    #[cfg(feature = "dev-ui")]
    fn interpretation_hint(&self) -> InterpretationHint {
        InterpretationHint::PreferAST // Layouts can be complex and benefit from AST interpretation
    }

    #[cfg(feature = "dev-ui")]
    fn handle_runtime_update(&mut self, update: &RuntimeUpdate) -> AdapterResult<()> {
        if let Some(data) = update.data.as_object() {
            if let Some(spacing) = data.get("spacing").and_then(|v| v.as_f64()) {
                self.state.spacing = spacing as f32;
            }
            if let Some(padding_array) = data.get("padding").and_then(|v| v.as_array()) {
                if padding_array.len() == 4 {
                    let padding: Result<Vec<f32>, _> = padding_array
                        .iter()
                        .map(|v| v.as_f64().map(|f| f as f32).ok_or("Invalid padding value"))
                        .collect();
                    
                    if let Ok(padding_vec) = padding {
                        self.state.padding = [padding_vec[0], padding_vec[1], padding_vec[2], padding_vec[3]];
                    }
                }
            }
            if let Some(bg_color_array) = data.get("background_color").and_then(|v| v.as_array()) {
                if bg_color_array.len() == 4 {
                    let color: Result<Vec<f32>, _> = bg_color_array
                        .iter()
                        .map(|v| v.as_f64().map(|f| f as f32).ok_or("Invalid color value"))
                        .collect();
                    
                    if let Ok(color_vec) = color {
                        self.state.background_color = Some([color_vec[0], color_vec[1], color_vec[2], color_vec[3]]);
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{ComponentStyle, Rect, RenderFeature};
    
    // Simple mock render context for testing
    struct TestRenderContext {
        rendered_elements: Vec<String>,
    }
    
    impl TestRenderContext {
        fn new() -> Self {
            Self {
                rendered_elements: Vec::new(),
            }
        }
    }
    
    impl RenderContext for TestRenderContext {
        fn render_button(&mut self, text: &str, _callback: Box<dyn Fn() + Send + Sync>) {
            self.rendered_elements.push(format!("Button: {}", text));
        }
        
        fn render_text(&mut self, text: &str) {
            self.rendered_elements.push(format!("Text: {}", text));
        }
        
        fn render_input(&mut self, value: &str, _on_change: Box<dyn Fn(String) + Send + Sync>) {
            self.rendered_elements.push(format!("Input: {}", value));
        }
        
        fn render_checkbox(&mut self, checked: bool, _on_change: Box<dyn Fn(bool) + Send + Sync>) {
            self.rendered_elements.push(format!("Checkbox: {}", checked));
        }
        
        fn begin_horizontal_layout(&mut self) {
            self.rendered_elements.push("BeginHorizontal".to_string());
        }
        
        fn end_horizontal_layout(&mut self) {
            self.rendered_elements.push("EndHorizontal".to_string());
        }
        
        fn begin_vertical_layout(&mut self) {
            self.rendered_elements.push("BeginVertical".to_string());
        }
        
        fn end_vertical_layout(&mut self) {
            self.rendered_elements.push("EndVertical".to_string());
        }
        
        fn apply_style(&mut self, _style: &ComponentStyle) {
            self.rendered_elements.push("ApplyStyle".to_string());
        }
        
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

    #[test]
    fn test_egui_button_creation() {
        let button = EguiButton::new("test_button".to_string(), "Click Me".to_string());
        assert_eq!(button.component_id(), "test_button");
        assert_eq!(button.component_type(), "EguiButton");
        assert_eq!(button.state.text, "Click Me");
        assert!(button.state.enabled);
        assert_eq!(button.state.click_count, 0);
    }

    #[test]
    fn test_egui_button_click() {
        let mut button = EguiButton::new("test_button".to_string(), "Click Me".to_string());
        assert_eq!(button.get_click_count(), 0);
        
        button.click();
        assert_eq!(button.get_click_count(), 1);
        
        button.click();
        assert_eq!(button.get_click_count(), 2);
    }

    #[test]
    fn test_egui_button_disabled_click() {
        let mut button = EguiButton::new("test_button".to_string(), "Click Me".to_string());
        button.set_enabled(false);
        
        button.click();
        assert_eq!(button.get_click_count(), 0); // Should not increment when disabled
    }

    #[test]
    fn test_egui_button_render() {
        let mut button = EguiButton::new("test_button".to_string(), "Click Me".to_string());
        let mut ctx = TestRenderContext::new();
        
        assert!(button.render(&mut ctx).is_ok());
        assert_eq!(ctx.rendered_elements.len(), 1);
        assert!(ctx.rendered_elements[0].contains("Click Me"));
    }

    #[test]
    #[cfg(feature = "dev-ui")]
    fn test_egui_button_state_preservation() {
        let mut button = EguiButton::new("test_button".to_string(), "Click Me".to_string());
        button.click();
        button.click();
        button.set_text("Updated Text".to_string());
        
        let state = button.hot_reload_state();
        assert!(state.is_object());
        
        let mut new_button = EguiButton::new("test_button".to_string(), "Original".to_string());
        assert!(new_button.restore_state(state).is_ok());
        
        assert_eq!(new_button.state.text, "Updated Text");
        assert_eq!(new_button.get_click_count(), 2);
    }

    #[test]
    fn test_egui_text_creation() {
        let text = EguiText::new("test_text".to_string(), "Hello World".to_string());
        assert_eq!(text.component_id(), "test_text");
        assert_eq!(text.component_type(), "EguiText");
        assert_eq!(text.get_content(), "Hello World");
    }

    #[test]
    fn test_egui_text_content_update() {
        let mut text = EguiText::new("test_text".to_string(), "Hello World".to_string());
        text.set_content("Updated Content".to_string());
        assert_eq!(text.get_content(), "Updated Content");
    }

    #[test]
    fn test_egui_input_creation() {
        let input = EguiInput::new("test_input".to_string(), "Enter text...".to_string());
        assert_eq!(input.component_id(), "test_input");
        assert_eq!(input.component_type(), "EguiInput");
        assert_eq!(input.get_value(), "");
        assert_eq!(input.state.placeholder, "Enter text...");
    }

    #[test]
    fn test_egui_input_value_update() {
        let mut input = EguiInput::new("test_input".to_string(), "Enter text...".to_string());
        input.set_value("Hello".to_string());
        assert_eq!(input.get_value(), "Hello");
    }

    #[test]
    fn test_egui_layout_creation() {
        let layout = EguiLayout::new("test_layout".to_string(), LayoutDirection::Horizontal);
        assert_eq!(layout.component_id(), "test_layout");
        assert_eq!(layout.component_type(), "EguiLayout");
        assert_eq!(layout.get_children_count(), 0);
    }

    #[test]
    fn test_egui_layout_with_children() {
        let mut layout = EguiLayout::new("test_layout".to_string(), LayoutDirection::Vertical);
        
        let button = Box::new(EguiButton::new("btn1".to_string(), "Button 1".to_string()));
        let text = Box::new(EguiText::new("txt1".to_string(), "Text 1".to_string()));
        
        layout.add_child(button);
        layout.add_child(text);
        
        assert_eq!(layout.get_children_count(), 2);
    }

    #[test]
    fn test_egui_layout_render() {
        let mut layout = EguiLayout::new("test_layout".to_string(), LayoutDirection::Horizontal);
        let button = Box::new(EguiButton::new("btn1".to_string(), "Button 1".to_string()));
        layout.add_child(button);
        
        let mut ctx = TestRenderContext::new();
        assert!(layout.render(&mut ctx).is_ok());
        
        // Should have begin layout, button, and end layout
        assert!(ctx.rendered_elements.len() >= 3);
        assert!(ctx.rendered_elements.contains(&"BeginHorizontal".to_string()));
        assert!(ctx.rendered_elements.contains(&"EndHorizontal".to_string()));
    }

    #[test]
    #[cfg(feature = "dev-ui")]
    fn test_component_runtime_updates() {
        let mut button = EguiButton::new("test_button".to_string(), "Original".to_string());
        
        let update = RuntimeUpdate {
            component_id: "test_button".to_string(),
            update_type: crate::traits::UpdateType::ComponentChange,
            data: serde_json::json!({
                "text": "Updated Text",
                "enabled": false
            }),
            timestamp: std::time::SystemTime::now(),
        };
        
        assert!(button.handle_runtime_update(&update).is_ok());
        assert_eq!(button.state.text, "Updated Text");
        assert!(!button.state.enabled);
    }
}