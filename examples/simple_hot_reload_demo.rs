//! Simple Hot Reload Demo
//! 
//! This example demonstrates RustyUI's hot reload capabilities without external UI frameworks.
//! It shows component registration, state preservation, and file change monitoring.

use rustyui_core::{
    DualModeEngine, DualModeConfig, UIFramework, UIComponent, 
    component_lifecycle::ComponentState
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

/// Simple component state that can be preserved
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ButtonState {
    text: String,
    click_count: u32,
    enabled: bool,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            text: "Click me!".to_string(),
            click_count: 0,
            enabled: true,
        }
    }
}

/// Simple button component for demonstration
#[derive(Debug)]
struct SimpleButton {
    id: String,
    state: ButtonState,
}

impl SimpleButton {
    fn new(id: String) -> Self {
        Self {
            id,
            state: ButtonState::default(),
        }
    }
    
    fn click(&mut self) {
        if self.state.enabled {
            self.state.click_count += 1;
            println!("Button '{}' clicked! Count: {}", self.id, self.state.click_count);
        }
    }
    
    fn set_text(&mut self, text: String) {
        self.state.text = text;
        println!("Button '{}' text changed to: '{}'", self.id, self.state.text);
    }
    
    fn toggle_enabled(&mut self) {
        self.state.enabled = !self.state.enabled;
        println!("Button '{}' enabled: {}", self.id, self.state.enabled);
    }
}

impl UIComponent for SimpleButton {
    type State = ButtonState;
    
    fn component_id(&self) -> &str {
        &self.id
    }
    
    fn component_type(&self) -> &str {
        "SimpleButton"
    }
    
    #[cfg(feature = "dev-ui")]
    fn hot_reload_state(&self) -> rustyui_core::Result<Self::State> {
        Ok(self.state.clone())
    }
    
    #[cfg(feature = "dev-ui")]
    fn restore_state(&mut self, state: Self::State) -> rustyui_core::Result<()> {
        self.state = state;
        Ok(())
    }
    
    fn render(&mut self, _ctx: &mut dyn rustyui_core::RenderContext) {
        println!("Rendering SimpleButton: {} (clicked {} times)", 
                 self.id, self.state.click_count);
    }
    
    #[cfg(feature = "dev-ui")]
    fn state_preservation_priority(&self) -> u32 {
        100 // High priority for buttons
    }
}

/// Demo application showcasing hot reload
struct HotReloadApp {
    engine: DualModeEngine,
    components: HashMap<String, SimpleButton>,
}

impl HotReloadApp {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create dual-mode configuration
        let config = DualModeConfig {
            framework: UIFramework::Custom { 
                name: "SimpleDemo".to_string(), 
                adapter_path: "examples/simple_hot_reload_demo.rs".to_string() 
            },
            watch_paths: vec![
                PathBuf::from("examples/"),
                PathBuf::from("src/"),
                PathBuf::from("crates/rustyui-core/src/"),
            ],
            production_settings: Default::default(),
            conditional_compilation: Default::default(),
            #[cfg(feature = "dev-ui")]
            development_settings: Default::default(),
        };
        
        // Initialize dual-mode engine
        let mut engine = DualModeEngine::new(config)?;
        engine.initialize()?;
        
        println!("RustyUI Hot Reload Demo initialized!");
        println!("Watching paths: {:?}", engine.config().watch_paths);
        
        let mut app = Self {
            engine,
            components: HashMap::new(),
        };
        
        // Create demo components
        app.create_demo_components()?;
        
        // Start development mode
        app.engine.start_development_mode()?;
        
        Ok(app)
    }
    
    fn create_demo_components(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Create various button components
        let button_configs = vec![
            ("primary_button", "Primary Action"),
            ("secondary_button", "Secondary Action"),
            ("toggle_button", "Toggle Me"),
            ("counter_button", "Count: 0"),
        ];
        
        for (id, text) in button_configs {
            let mut button = SimpleButton::new(id.to_string());
            button.set_text(text.to_string());
            
            // Register with engine
            self.engine.register_component(id.to_string(), "SimpleButton".to_string())?;
            self.engine.update_component_state(id, ComponentState::Active)?;
            
            // Preserve initial state
            let state_json = serde_json::to_value(&button.state)?;
            self.engine.preserve_component_state(id, state_json)?;
            
            self.components.insert(id.to_string(), button);
        }
        
        println!("Created {} demo components", self.components.len());
        Ok(())
    }
    
    fn simulate_user_interactions(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\nSimulating user interactions...");
        
        // Simulate clicking buttons - need to separate the operations to avoid borrow checker issues
        if self.components.contains_key("primary_button") {
            self.components.get_mut("primary_button").unwrap().click();
            self.preserve_component_state("primary_button")?;
        }
        
        if self.components.contains_key("counter_button") {
            for i in 1..=3 {
                let component = self.components.get_mut("counter_button").unwrap();
                component.click();
                component.set_text(format!("Count: {}", i));
                drop(component); // Release the borrow before calling preserve_component_state
                self.preserve_component_state("counter_button")?;
            }
        }
        
        if self.components.contains_key("toggle_button") {
            self.components.get_mut("toggle_button").unwrap().toggle_enabled();
            self.preserve_component_state("toggle_button")?;
        }
        
        Ok(())
    }
    
    fn preserve_component_state(&mut self, component_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(component) = self.components.get(component_id) {
            let state_json = serde_json::to_value(&component.state)?;
            self.engine.preserve_component_state(component_id, state_json)?;
            self.engine.update_component_state(component_id, ComponentState::Updating)?;
            
            // Simulate processing time
            thread::sleep(Duration::from_millis(10));
            
            self.engine.update_component_state(component_id, ComponentState::Active)?;
        }
        Ok(())
    }
    
    fn simulate_hot_reload(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\nSimulating hot reload...");
        
        // Simulate file change detection
        let updated_components = self.engine.process_file_change_and_update("examples/simple_hot_reload_demo.rs")?;
        
        if !updated_components.is_empty() {
            println!("File change detected, updating components: {:?}", updated_components);
            
            // Restore state for affected components
            for component_id in &updated_components {
                if let Some(restored_state) = self.engine.restore_component_state(component_id) {
                    if let Ok(button_state) = serde_json::from_value::<ButtonState>(restored_state) {
                        if let Some(component) = self.components.get_mut(component_id) {
                            let _ = component.restore_state(button_state);
                        }
                    }
                }
            }
        } else {
            println!("File change processed, no components affected");
        }
        
        Ok(())
    }
    
    fn display_statistics(&self) {
        println!("\nRustyUI Statistics:");
        
        // Component statistics
        if let Some(stats) = self.engine.get_component_statistics() {
            println!("Components: {} total, {} active", stats.total_components, stats.active_components);
            println!("Total updates: {}", stats.total_updates);
            if let Some(avg_age) = stats.average_age {
                println!("Average component age: {:.2}s", avg_age.as_secs_f32());
            }
        }
        
        // Memory usage
        let memory_overhead = self.engine.memory_overhead();
        println!("Memory overhead: {:.1} KB", memory_overhead as f32 / 1024.0);
        
        // Engine status
        println!("Hot reload: {}", if self.engine.has_runtime_interpreter() { "Active" } else { "Inactive" });
        println!("Platform: {:?}", self.engine.platform());
        println!("Native optimizations: {}", self.engine.is_using_native_optimizations());
        
        // Active components
        let active_components = self.engine.get_active_components();
        println!("Active components:");
        for component in active_components {
            println!("- {} ({})", component.id, component.type_name);
        }
    }
    
    fn run_demo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\nStarting Hot Reload Demo...");
        
        // Initial state
        self.display_statistics();
        
        // Simulate user interactions
        thread::sleep(Duration::from_millis(500));
        self.simulate_user_interactions()?;
        
        // Show updated statistics
        thread::sleep(Duration::from_millis(500));
        self.display_statistics();
        
        // Simulate hot reload
        thread::sleep(Duration::from_millis(500));
        self.simulate_hot_reload()?;
        
        // Final statistics
        thread::sleep(Duration::from_millis(500));
        self.display_statistics();
        
        println!("\nDemo completed successfully!");
        println!("In a real application, file changes would trigger automatic reloads");
        println!("Component state would be preserved across code changes");
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RustyUI Simple Hot Reload Demo");
    println!("==================================");
    
    let mut app = HotReloadApp::new()?;
    app.run_demo()?;
    
    Ok(())
}