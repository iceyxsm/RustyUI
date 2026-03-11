//! Example demonstrating egui component integration with hot reload support
//! 
//! This example shows how to use the egui components with the RustyUI hot reload system,
//! including state preservation, runtime updates, and component composition.

use rustyui_adapters::{
    EguiAdapter, EguiButton, EguiText, EguiInput, EguiLayout, LayoutDirection,
    UIFrameworkAdapter, RenderContext, UIComponent, FrameworkConfig, RuntimeUpdate, UpdateType
};
use rustyui_core::DualModeEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting egui component integration example");
    
    // Initialize the dual-mode engine for development
    let config = create_development_config();
    let _engine = DualModeEngine::new(config)?;
    
    // Create egui adapter
    let mut adapter = EguiAdapter::new();
    let framework_config = FrameworkConfig::default();
    adapter.initialize(&framework_config)?;
    
    // Create example application with various components
    let mut app = ExampleEguiApp::new();
    
    // Demonstrate component rendering
    println!("\nRendering initial UI components...");
    app.render_ui(&mut adapter)?;
    
    // Demonstrate hot reload state preservation
    println!("\nDemonstrating state preservation...");
    app.demonstrate_state_preservation(&mut adapter)?;
    
    // Demonstrate runtime updates
    println!("\nDemonstrating runtime updates...");
    app.demonstrate_runtime_updates(&mut adapter)?;
    
    // Demonstrate component composition
    println!("\nDemonstrating component composition...");
    app.demonstrate_component_composition(&mut adapter)?;
    
    println!("\negui component integration example completed successfully!");
    
    Ok(())
}

/// Example application demonstrating egui component integration
struct ExampleEguiApp {
    /// Main button component
    main_button: EguiButton,
    
    /// Status text component
    status_text: EguiText,
    
    /// User input component
    user_input: EguiInput,
    
    /// Main layout container
    main_layout: EguiLayout,
    
    /// Button layout container
    button_layout: EguiLayout,
    
    /// Application state
    click_count: u32,
    user_message: String,
}

impl ExampleEguiApp {
    fn new() -> Self {
        // Create main button with click handler
        let main_button = EguiButton::new("main_button".to_string(), "Click Me!".to_string())
            .with_click_handler(|| {
                println!("Main button clicked!");
            });
        
        // Create status text
        let status_text = EguiText::new("status_text".to_string(), "Ready".to_string());
        
        // Create user input with change handler
        let user_input = EguiInput::new("user_input".to_string(), "Enter your message...".to_string())
            .with_change_handler(|value| {
                println!("Input changed to: {}", value);
            });
        
        // Create layout containers
        let main_layout = EguiLayout::new("main_layout".to_string(), LayoutDirection::Vertical);
        let button_layout = EguiLayout::new("button_layout".to_string(), LayoutDirection::Horizontal);
        
        Self {
            main_button,
            status_text,
            user_input,
            main_layout,
            button_layout,
            click_count: 0,
            user_message: String::new(),
        }
    }
    
    /// Render the UI components
    fn render_ui(&mut self, adapter: &mut EguiAdapter) -> Result<(), Box<dyn std::error::Error>> {
        // Create render context
        let mut render_context = adapter.create_render_context()?;
        
        // Render individual components
        println!("Rendering main button...");
        self.main_button.render(render_context.as_mut())?;
        
        println!("Rendering status text...");
        self.status_text.render(render_context.as_mut())?;
        
        println!("Rendering user input...");
        self.user_input.render(render_context.as_mut())?;
        
        // Render layout with nested components
        println!("Rendering layout structure...");
        self.render_layout_structure(render_context.as_mut())?;
        
        Ok(())
    }
    
    /// Render the layout structure with nested components
    fn render_layout_structure(&mut self, ctx: &mut dyn RenderContext) -> Result<(), Box<dyn std::error::Error>> {
        // Begin main vertical layout
        ctx.begin_vertical_layout();
        
        // Render title text
        ctx.render_text("egui Component Integration Example");
        
        // Render user input
        ctx.render_input(&self.user_message, Box::new(|new_value| {
            println!("User input updated: {}", new_value);
        }));
        
        // Begin horizontal button layout
        ctx.begin_horizontal_layout();
        
        // Render multiple buttons
        ctx.render_button("Button 1", Box::new(|| println!("Button 1 clicked")));
        ctx.render_button("Button 2", Box::new(|| println!("Button 2 clicked")));
        ctx.render_button("Reset", Box::new(|| println!("Reset clicked")));
        
        // End horizontal layout
        ctx.end_horizontal_layout();
        
        // Render status text
        let status = format!("Clicks: {} | Message: '{}'", self.click_count, self.user_message);
        ctx.render_text(&status);
        
        // End main vertical layout
        ctx.end_vertical_layout();
        
        Ok(())
    }
    
    /// Demonstrate state preservation across hot reloads
    fn demonstrate_state_preservation(&mut self, _adapter: &mut EguiAdapter) -> Result<(), Box<dyn std::error::Error>> {
        println!("Setting up initial state...");
        
        // Modify component states
        self.main_button.click();
        self.main_button.click();
        self.main_button.set_text("Clicked 2 times!".to_string());
        
        self.status_text.set_content("State modified".to_string());
        
        self.user_input.set_value("Test message".to_string());
        
        // Save states
        println!("Saving component states...");
        let button_state = self.main_button.hot_reload_state();
        let text_state = self.status_text.hot_reload_state();
        let input_state = self.user_input.hot_reload_state();
        
        println!("Button state: {}", serde_json::to_string_pretty(&button_state)?);
        println!("Text state: {}", serde_json::to_string_pretty(&text_state)?);
        println!("Input state: {}", serde_json::to_string_pretty(&input_state)?);
        
        // Create new components and restore states
        println!("Creating new components and restoring states...");
        let mut new_button = EguiButton::new("main_button".to_string(), "Original".to_string());
        let mut new_text = EguiText::new("status_text".to_string(), "Original".to_string());
        let mut new_input = EguiInput::new("user_input".to_string(), "Original".to_string());
        
        new_button.restore_state(button_state)?;
        new_text.restore_state(text_state)?;
        new_input.restore_state(input_state)?;
        
        // Verify state restoration
        println!("Verifying state restoration...");
        println!("Button click count: {}", new_button.get_click_count());
        println!("Text content: '{}'", new_text.get_content());
        println!("Input value: '{}'", new_input.get_value());
        
        // Update our components
        self.main_button = new_button;
        self.status_text = new_text;
        self.user_input = new_input;
        
        Ok(())
    }
    
    /// Demonstrate runtime updates
    fn demonstrate_runtime_updates(&mut self, adapter: &mut EguiAdapter) -> Result<(), Box<dyn std::error::Error>> {
        println!("Applying runtime updates to components...");
        
        // Create runtime updates for different components
        let button_update = RuntimeUpdate {
            component_id: "main_button".to_string(),
            update_type: UpdateType::ComponentChange,
            data: serde_json::json!({
                "text": "Updated via Runtime!",
                "enabled": true,
                "style": {
                    "width": 200.0,
                    "height": 40.0,
                    "color": [0.2, 0.6, 0.8, 1.0]
                }
            }),
            timestamp: std::time::SystemTime::now(),
        };
        
        let text_update = RuntimeUpdate {
            component_id: "status_text".to_string(),
            update_type: UpdateType::ComponentChange,
            data: serde_json::json!({
                "content": "Runtime update applied!",
                "style": {
                    "size": 16.0,
                    "bold": true,
                    "color": [0.0, 0.8, 0.0, 1.0]
                }
            }),
            timestamp: std::time::SystemTime::now(),
        };
        
        let input_update = RuntimeUpdate {
            component_id: "user_input".to_string(),
            update_type: UpdateType::ComponentChange,
            data: serde_json::json!({
                "placeholder": "Updated placeholder...",
                "enabled": true,
                "style": {
                    "background_color": [0.95, 0.95, 0.95, 1.0],
                    "text_color": [0.1, 0.1, 0.1, 1.0]
                }
            }),
            timestamp: std::time::SystemTime::now(),
        };
        
        // Apply updates through adapter
        adapter.handle_runtime_update(&button_update)?;
        adapter.handle_runtime_update(&text_update)?;
        adapter.handle_runtime_update(&input_update)?;
        
        // Apply updates to components
        self.main_button.handle_runtime_update(&button_update)?;
        self.status_text.handle_runtime_update(&text_update)?;
        self.user_input.handle_runtime_update(&input_update)?;
        
        // Process queued updates
        adapter.process_update_queue()?;
        
        println!("Runtime updates applied successfully!");
        
        Ok(())
    }
    
    /// Demonstrate component composition and complex layouts
    fn demonstrate_component_composition(&mut self, adapter: &mut EguiAdapter) -> Result<(), Box<dyn std::error::Error>> {
        println!("Creating complex component composition...");
        
        // Create a complex layout with nested components
        let mut main_container = EguiLayout::new("main_container".to_string(), LayoutDirection::Vertical);
        main_container.set_spacing(8.0);
        main_container.set_padding([10.0, 10.0, 10.0, 10.0]);
        
        // Create header section
        let mut header_layout = EguiLayout::new("header_layout".to_string(), LayoutDirection::Horizontal);
        let title_text = Box::new(EguiText::new("title".to_string(), "Component Composition Demo".to_string()));
        let close_button = Box::new(EguiButton::new("close_btn".to_string(), "×".to_string()));
        
        header_layout.add_child(title_text);
        header_layout.add_child(close_button);
        
        // Create content section
        let mut content_layout = EguiLayout::new("content_layout".to_string(), LayoutDirection::Vertical);
        let description_text = Box::new(EguiText::new("description".to_string(), 
            "This demonstrates complex component composition with nested layouts.".to_string()));
        let name_input = Box::new(EguiInput::new("name_input".to_string(), "Enter your name...".to_string()));
        
        content_layout.add_child(description_text);
        content_layout.add_child(name_input);
        
        // Create button section
        let mut button_section = EguiLayout::new("button_section".to_string(), LayoutDirection::Horizontal);
        button_section.set_spacing(5.0);
        
        let save_button = Box::new(EguiButton::new("save_btn".to_string(), "Save".to_string()));
        let cancel_button = Box::new(EguiButton::new("cancel_btn".to_string(), "Cancel".to_string()));
        let help_button = Box::new(EguiButton::new("help_btn".to_string(), "Help".to_string()));
        
        button_section.add_child(save_button);
        button_section.add_child(cancel_button);
        button_section.add_child(help_button);
        
        // Compose the main layout
        main_container.add_child(Box::new(header_layout));
        main_container.add_child(Box::new(content_layout));
        main_container.add_child(Box::new(button_section));
        
        // Render the composed layout
        let mut render_context = adapter.create_render_context()?;
        main_container.render(render_context.as_mut())?;
        
        println!("Complex composition rendered with {} top-level children", main_container.get_children_count());
        
        // Demonstrate state preservation for complex layouts
        println!("Testing state preservation for complex layout...");
        let layout_state = main_container.hot_reload_state();
        println!("Layout state preserved: {} bytes", serde_json::to_string(&layout_state)?.len());
        
        Ok(())
    }
}

/// Create development configuration for the example
fn create_development_config() -> rustyui_core::DualModeConfig {
    rustyui_core::DualModeConfig {
        framework: rustyui_core::config::UIFramework::Egui,
        #[cfg(feature = "dev-ui")]
        development_settings: rustyui_core::config::DevelopmentSettings::default(),
        production_settings: rustyui_core::config::ProductionSettings::default(),
        watch_paths: vec![
            std::path::PathBuf::from("src"),
            std::path::PathBuf::from("examples"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_example_app_creation() {
        let app = ExampleEguiApp::new();
        assert_eq!(app.main_button.component_id(), "main_button");
        assert_eq!(app.status_text.component_id(), "status_text");
        assert_eq!(app.user_input.component_id(), "user_input");
    }
    
    #[test]
    fn test_component_rendering() {
        let mut app = ExampleEguiApp::new();
        let mut adapter = EguiAdapter::new();
        let config = FrameworkConfig::default();
        
        assert!(adapter.initialize(&config).is_ok());
        assert!(app.render_ui(&mut adapter).is_ok());
    }
    
    #[test]
    #[cfg(feature = "dev-ui")]
    fn test_state_preservation() {
        let mut app = ExampleEguiApp::new();
        let mut adapter = EguiAdapter::new();
        let config = FrameworkConfig::default();
        
        adapter.initialize(&config).unwrap();
        assert!(app.demonstrate_state_preservation(&mut adapter).is_ok());
    }
    
    #[test]
    #[cfg(feature = "dev-ui")]
    fn test_runtime_updates() {
        let mut app = ExampleEguiApp::new();
        let mut adapter = EguiAdapter::new();
        let config = FrameworkConfig::default();
        
        adapter.initialize(&config).unwrap();
        assert!(app.demonstrate_runtime_updates(&mut adapter).is_ok());
    }
    
    #[test]
    fn test_component_composition() {
        let mut app = ExampleEguiApp::new();
        let mut adapter = EguiAdapter::new();
        let config = FrameworkConfig::default();
        
        adapter.initialize(&config).unwrap();
        assert!(app.demonstrate_component_composition(&mut adapter).is_ok());
    }
}