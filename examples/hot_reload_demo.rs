//! Hot Reload Demo Application
//! 
//! This example demonstrates RustyUI's hot reload capabilities with egui.
//! It showcases various UI components, state preservation, and real-time updates.

use eframe::egui;
use rustyui_core::{DualModeEngine, DualModeConfig, UIFramework};
use rustyui_adapters::{EguiAdapter, UIFrameworkAdapter};
use std::collections::HashMap;

/// Main application state that will be preserved during hot reload
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppState {
    /// Counter value that persists across reloads
    pub counter: i32,
    /// Text input that maintains its content
    pub text_input: String,
    /// Checkbox state
    pub checkbox_enabled: bool,
    /// Slider value
    pub slider_value: f32,
    /// Selected tab
    pub selected_tab: usize,
    /// Color picker value
    pub color: [f32; 3],
    /// List of items
    pub items: Vec<String>,
    /// Current item being added
    pub new_item: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            counter: 0,
            text_input: "Hello, RustyUI!".to_string(),
            checkbox_enabled: true,
            slider_value: 50.0,
            selected_tab: 0,
            color: [0.5, 0.7, 1.0],
            items: vec![
                "Item 1".to_string(),
                "Item 2".to_string(),
                "Item 3".to_string(),
            ],
            new_item: String::new(),
        }
    }
}

/// Hot reload demo application
pub struct HotReloadDemo {
    /// Application state
    state: AppState,
    /// RustyUI dual-mode engine
    #[cfg(feature = "dev-ui")]
    engine: Option<DualModeEngine>,
    /// Framework adapter
    #[cfg(feature = "dev-ui")]
    adapter: Option<EguiAdapter>,
    /// Component IDs for tracking
    component_ids: HashMap<String, String>,
}

impl HotReloadDemo {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            state: AppState::default(),
            #[cfg(feature = "dev-ui")]
            engine: None,
            #[cfg(feature = "dev-ui")]
            adapter: None,
            component_ids: HashMap::new(),
        };
        
        // Initialize RustyUI in development mode
        #[cfg(feature = "dev-ui")]
        {
            app.init_rustyui();
        }
        
        app
    }
    
    #[cfg(feature = "dev-ui")]
    fn init_rustyui(&mut self) {
        // Create dual-mode configuration
        let config = DualModeConfig {
            framework: UIFramework::Egui,
            watch_paths: vec![
                std::path::PathBuf::from("examples/"), 
                std::path::PathBuf::from("src/")
            ],
            production_settings: Default::default(),
            conditional_compilation: Default::default(),
            #[cfg(feature = "dev-ui")]
            development_settings: Default::default(),
        };
        
        // Initialize dual-mode engine
        match DualModeEngine::new(config) {
            Ok(mut engine) => {
                if let Err(e) = engine.initialize() {
                    eprintln!("Failed to initialize RustyUI engine: {}", e);
                    return;
                }
                
                // Register components
                let _ = engine.register_component("counter_button".to_string(), "Button".to_string());
                let _ = engine.register_component("text_input".to_string(), "TextInput".to_string());
                let _ = engine.register_component("checkbox".to_string(), "Checkbox".to_string());
                let _ = engine.register_component("slider".to_string(), "Slider".to_string());
                let _ = engine.register_component("color_picker".to_string(), "ColorPicker".to_string());
                let _ = engine.register_component("item_list".to_string(), "List".to_string());
                
                // Start development mode
                if let Err(e) = engine.start_development_mode() {
                    eprintln!("Failed to start development mode: {}", e);
                }
                
                self.engine = Some(engine);
                println!(" RustyUI hot reload enabled!");
            }
            Err(e) => {
                eprintln!("Failed to create RustyUI engine: {}", e);
            }
        }
        
        // Initialize egui adapter
        let mut adapter = EguiAdapter::new();
        let framework_config = rustyui_adapters::FrameworkConfig::default();
        if let Err(e) = adapter.initialize(&framework_config) {
            eprintln!("Failed to initialize egui adapter: {}", e);
        } else {
            self.adapter = Some(adapter);
        }
    }
    
    /// Preserve current state for hot reload
    #[cfg(feature = "dev-ui")]
    fn preserve_state(&mut self) {
        if let Some(ref mut engine) = self.engine {
            let state_json = serde_json::to_value(&self.state).unwrap_or_default();
            let _ = engine.preserve_component_state("app_state", state_json);
        }
    }
    
    /// Restore state after hot reload
    #[cfg(feature = "dev-ui")]
    fn restore_state(&mut self) {
        if let Some(ref mut engine) = self.engine {
            if let Some(state_json) = engine.restore_component_state("app_state") {
                if let Ok(restored_state) = serde_json::from_value::<AppState>(state_json) {
                    self.state = restored_state;
                    println!("State restored from hot reload");
                }
            }
        }
    }
    
    /// Render the counter section
    fn render_counter(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("Counter Demo");
            ui.horizontal(|ui| {
                if ui.button("➖").clicked() {
                    self.state.counter -= 1;
                    self.on_counter_changed();
                }
                
                ui.label(format!("Count: {}", self.state.counter));
                
                if ui.button("➕").clicked() {
                    self.state.counter += 1;
                    self.on_counter_changed();
                }
            });
            
            if ui.button("Reset Counter").clicked() {
                self.state.counter = 0;
                self.on_counter_changed();
            }
        });
    }
    
    /// Render the text input section
    fn render_text_input(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("Text Input Demo");
            
            let response = ui.text_edit_singleline(&mut self.state.text_input);
            if response.changed() {
                self.on_text_changed();
            }
            
            ui.label(format!("Text length: {}", self.state.text_input.len()));
            
            if ui.button("Clear Text").clicked() {
                self.state.text_input.clear();
                self.on_text_changed();
            }
        });
    }
    
    /// Render the controls section
    fn render_controls(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("Controls Demo");
            
            let checkbox_response = ui.checkbox(&mut self.state.checkbox_enabled, "Enable features");
            if checkbox_response.changed() {
                self.on_checkbox_changed();
            }
            
            let slider_response = ui.add(
                egui::Slider::new(&mut self.state.slider_value, 0.0..=100.0)
                    .text("Value")
                    .suffix("%")
            );
            if slider_response.changed() {
                self.on_slider_changed();
            }
            
            ui.horizontal(|ui| {
                ui.label("Color:");
                let color_response = ui.color_edit_button_rgb(&mut self.state.color);
                if color_response.changed() {
                    self.on_color_changed();
                }
            });
        });
    }
    
    /// Render the item list section
    fn render_item_list(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("Dynamic List Demo");
            
            // Add new item
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.state.new_item);
                if ui.button("Add Item").clicked() && !self.state.new_item.is_empty() {
                    self.state.items.push(self.state.new_item.clone());
                    self.state.new_item.clear();
                    self.on_list_changed();
                }
            });
            
            // Display items
            let mut to_remove = None;
            for (index, item) in self.state.items.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}. {}", index + 1, item));
                    if ui.small_button("X").clicked() {
                        to_remove = Some(index);
                    }
                });
            }
            
            // Remove item if requested
            if let Some(index) = to_remove {
                self.state.items.remove(index);
                self.on_list_changed();
            }
            
            if ui.button("Clear All").clicked() {
                self.state.items.clear();
                self.on_list_changed();
            }
        });
    }
    
    /// Render the tabs section
    fn render_tabs(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("Tabs Demo");
            
            ui.horizontal(|ui| {
                for (i, tab_name) in ["Overview", "Settings", "About"].iter().enumerate() {
                    let selected = self.state.selected_tab == i;
                    if ui.selectable_label(selected, *tab_name).clicked() {
                        self.state.selected_tab = i;
                        self.on_tab_changed();
                    }
                }
            });
            
            ui.separator();
            
            match self.state.selected_tab {
                0 => {
                    ui.label(" Overview Tab");
                    ui.label(format!("Counter: {}", self.state.counter));
                    ui.label(format!("Items: {}", self.state.items.len()));
                    ui.label(format!("Checkbox: {}", if self.state.checkbox_enabled { "ON" } else { "OFF" }));
                }
                1 => {
                    ui.label(" Settings Tab");
                    ui.label(format!("Slider Value: {:.1}%", self.state.slider_value));
                    ui.label(format!("Color: RGB({:.2}, {:.2}, {:.2})", 
                        self.state.color[0], self.state.color[1], self.state.color[2]));
                }
                2 => {
                    ui.label(" About Tab");
                    ui.label("RustyUI Hot Reload Demo");
                    ui.label("This application demonstrates real-time UI updates");
                    ui.label("with state preservation across reloads.");
                }
                _ => {}
            }
        });
    }
    
    /// Render performance info
    fn render_performance_info(&mut self, ui: &mut egui::Ui) {
        #[cfg(feature = "dev-ui")]
        if let Some(ref engine) = self.engine {
            ui.group(|ui| {
                ui.label(" RustyUI Performance");
                
                if let Some(stats) = engine.get_component_statistics() {
                    ui.label(format!("Active Components: {}", stats.active_components));
                    ui.label(format!("Total Updates: {}", stats.total_updates));
                    if let Some(avg_age) = stats.average_age {
                        ui.label(format!("Avg Component Age: {:.1}s", avg_age.as_secs_f32()));
                    }
                }
                
                let memory_overhead = engine.memory_overhead();
                ui.label(format!("Memory Overhead: {:.1} KB", memory_overhead as f32 / 1024.0));
                
                if engine.has_runtime_interpreter() {
                    ui.label(" Hot Reload: Active");
                } else {
                    ui.label(" Hot Reload: Inactive");
                }
            });
        }
        
        #[cfg(not(feature = "dev-ui"))]
        {
            ui.group(|ui| {
                ui.label(" Production Mode");
                ui.label("Hot reload disabled for optimal performance");
            });
        }
    }
    
    // Event handlers for component updates
    
    fn on_counter_changed(&mut self) {
        #[cfg(feature = "dev-ui")]
        {
            self.preserve_state();
            if let Some(ref mut engine) = self.engine {
                let _ = engine.update_component_state("counter_button", 
                    rustyui_core::component_lifecycle::ComponentState::Active);
            }
        }
    }
    
    fn on_text_changed(&mut self) {
        #[cfg(feature = "dev-ui")]
        {
            self.preserve_state();
            if let Some(ref mut engine) = self.engine {
                let _ = engine.update_component_state("text_input", 
                    rustyui_core::component_lifecycle::ComponentState::Active);
            }
        }
    }
    
    fn on_checkbox_changed(&mut self) {
        #[cfg(feature = "dev-ui")]
        {
            self.preserve_state();
            if let Some(ref mut engine) = self.engine {
                let _ = engine.update_component_state("checkbox", 
                    rustyui_core::component_lifecycle::ComponentState::Active);
            }
        }
    }
    
    fn on_slider_changed(&mut self) {
        #[cfg(feature = "dev-ui")]
        {
            self.preserve_state();
            if let Some(ref mut engine) = self.engine {
                let _ = engine.update_component_state("slider", 
                    rustyui_core::component_lifecycle::ComponentState::Active);
            }
        }
    }
    
    fn on_color_changed(&mut self) {
        #[cfg(feature = "dev-ui")]
        {
            self.preserve_state();
            if let Some(ref mut engine) = self.engine {
                let _ = engine.update_component_state("color_picker", 
                    rustyui_core::component_lifecycle::ComponentState::Active);
            }
        }
    }
    
    fn on_list_changed(&mut self) {
        #[cfg(feature = "dev-ui")]
        {
            self.preserve_state();
            if let Some(ref mut engine) = self.engine {
                let _ = engine.update_component_state("item_list", 
                    rustyui_core::component_lifecycle::ComponentState::Active);
            }
        }
    }
    
    fn on_tab_changed(&mut self) {
        #[cfg(feature = "dev-ui")]
        {
            self.preserve_state();
        }
    }
}

impl eframe::App for HotReloadDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Restore state on startup
        #[cfg(feature = "dev-ui")]
        {
            static mut FIRST_RUN: bool = true;
            unsafe {
                if FIRST_RUN {
                    self.restore_state();
                    FIRST_RUN = false;
                }
            }
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(" RustyUI Hot Reload Demo");
            ui.separator();
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Counter section
                self.render_counter(ui);
                ui.add_space(10.0);
                
                // Text input section
                self.render_text_input(ui);
                ui.add_space(10.0);
                
                // Controls section
                self.render_controls(ui);
                ui.add_space(10.0);
                
                // Item list section
                self.render_item_list(ui);
                ui.add_space(10.0);
                
                // Tabs section
                self.render_tabs(ui);
                ui.add_space(10.0);
                
                // Performance info
                self.render_performance_info(ui);
            });
        });
        
        // Process file changes in development mode
        #[cfg(feature = "dev-ui")]
        if let Some(ref mut engine) = self.engine {
            // Check for file changes and update components
            if let Ok(updated_components) = engine.process_file_change_and_update("examples/hot_reload_demo.rs") {
                if !updated_components.is_empty() {
                    println!("Hot reload triggered for components: {:?}", updated_components);
                    ctx.request_repaint();
                }
            }
        }
    }
    
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        #[cfg(feature = "dev-ui")]
        self.preserve_state();
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("RustyUI Hot Reload Demo"),
        ..Default::default()
    };
    
    eframe::run_native(
        "RustyUI Hot Reload Demo",
        options,
        Box::new(|cc| Ok(Box::new(HotReloadDemo::new(cc)))),
    )
}