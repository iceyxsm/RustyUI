//! Comprehensive RustyUI Demo
//! 
//! This example demonstrates the complete RustyUI workflow including:
//! - Component registration and lifecycle management
//! - State preservation across hot reloads
//! - File change monitoring and analysis
//! - Error recovery and performance monitoring
//! - Integration with multiple UI component types

use rustyui_core::{
    DualModeEngine, DualModeConfig, UIFramework, UIComponent,
    component_lifecycle::{ComponentState, ComponentStatistics},
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

/// Different types of UI components for demonstration
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ComponentData {
    Button { text: String, clicks: u32, enabled: bool },
    TextInput { value: String, placeholder: String, focused: bool },
    Slider { value: f32, min: f32, max: f32, step: f32 },
    Checkbox { checked: bool, label: String },
    List { items: Vec<String>, selected: Option<usize> },
}

impl Default for ComponentData {
    fn default() -> Self {
        ComponentData::Button {
            text: "Default Button".to_string(),
            clicks: 0,
            enabled: true,
        }
    }
}

/// Generic UI component that can represent different types
#[derive(Debug)]
struct DemoComponent {
    id: String,
    component_type: String,
    data: ComponentData,
    created_at: Instant,
}

impl DemoComponent {
    fn new(id: String, component_type: String, data: ComponentData) -> Self {
        Self {
            id,
            component_type,
            data,
            created_at: Instant::now(),
        }
    }
    
    /// Simulate user interaction with the component
    fn interact(&mut self) {
        match &mut self.data {
            ComponentData::Button { clicks, .. } => {
                *clicks += 1;
                println!("🖱️ Button '{}' clicked! Total clicks: {}", self.id, clicks);
            }
            ComponentData::TextInput { value, .. } => {
                value.push_str(" (edited)");
                println!("📝 TextInput '{}' modified: '{}'", self.id, value);
            }
            ComponentData::Slider { value, max, .. } => {
                *value = (*value + 10.0).min(*max);
                println!("🎚️ Slider '{}' adjusted to: {:.1}", self.id, value);
            }
            ComponentData::Checkbox { checked, .. } => {
                *checked = !*checked;
                println!("☑️ Checkbox '{}' toggled: {}", self.id, checked);
            }
            ComponentData::List { items, selected } => {
                items.push(format!("Item {}", items.len() + 1));
                *selected = Some(items.len() - 1);
                println!("📋 List '{}' item added. Total items: {}", self.id, items.len());
            }
        }
    }
    
    fn get_display_info(&self) -> String {
        match &self.data {
            ComponentData::Button { text, clicks, enabled } => {
                format!("Button '{}': '{}' ({} clicks, {})", 
                    self.id, text, clicks, if *enabled { "enabled" } else { "disabled" })
            }
            ComponentData::TextInput { value, .. } => {
                format!("TextInput '{}': '{}'", self.id, value)
            }
            ComponentData::Slider { value, min, max, .. } => {
                format!("Slider '{}': {:.1} ({:.1}-{:.1})", self.id, value, min, max)
            }
            ComponentData::Checkbox { checked, label } => {
                format!("Checkbox '{}': {} '{}'", self.id, if *checked { "✅" } else { "❌" }, label)
            }
            ComponentData::List { items, selected } => {
                let selected_info = selected.map(|i| format!(" (selected: {})", i + 1))
                    .unwrap_or_default();
                format!("List '{}': {} items{}", self.id, items.len(), selected_info)
            }
        }
    }
}

impl UIComponent for DemoComponent {
    type State = ComponentData;
    
    fn component_id(&self) -> &str {
        &self.id
    }
    
    fn component_type(&self) -> &str {
        &self.component_type
    }
    
    #[cfg(feature = "dev-ui")]
    fn hot_reload_state(&self) -> rustyui_core::Result<Self::State> {
        Ok(self.data.clone())
    }
    
    #[cfg(feature = "dev-ui")]
    fn restore_state(&mut self, state: Self::State) -> rustyui_core::Result<()> {
        self.data = state;
        println!("🔄 Component '{}' state restored", self.id);
        Ok(())
    }
    
    #[cfg(feature = "dev-ui")]
    fn state_preservation_priority(&self) -> u32 {
        match self.data {
            ComponentData::Button { .. } => 100,
            ComponentData::TextInput { .. } => 150,
            ComponentData::Slider { .. } => 80,
            ComponentData::Checkbox { .. } => 90,
            ComponentData::List { .. } => 120,
        }
    }
}

/// Comprehensive demo application
struct ComprehensiveDemo {
    engine: DualModeEngine,
    components: HashMap<String, DemoComponent>,
    interaction_count: u32,
    start_time: Instant,
}

impl ComprehensiveDemo {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("🚀 Initializing Comprehensive RustyUI Demo...");
        
        // Create configuration with extensive monitoring
        let config = DualModeConfig {
            framework: UIFramework::Custom { 
                name: "ComprehensiveDemo".to_string(), 
                adapter_path: "examples/comprehensive_demo.rs".to_string() 
            },
            watch_paths: vec![
                std::path::PathBuf::from("examples/"),
                std::path::PathBuf::from("src/"),
                std::path::PathBuf::from("crates/rustyui-core/src/"),
                std::path::PathBuf::from("crates/rustyui-adapters/src/"),
            ],
            production_settings: Default::default(),
            conditional_compilation: Default::default(),
            #[cfg(feature = "dev-ui")]
            development_settings: Default::default(),
        };
        
        // Initialize engine with full monitoring
        let mut engine = DualModeEngine::new(config)?;
        engine.initialize()?;
        engine.start_development_mode()?;
        
        println!("✅ RustyUI engine initialized successfully");
        
        Ok(Self {
            engine,
            components: HashMap::new(),
            interaction_count: 0,
            start_time: Instant::now(),
        })
    }
    
    fn create_demo_components(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🏗️ Creating diverse UI components...");
        
        let component_specs = vec![
            ("main_button", "Button", ComponentData::Button {
                text: "Main Action".to_string(),
                clicks: 0,
                enabled: true,
            }),
            ("search_input", "TextInput", ComponentData::TextInput {
                value: "Search here...".to_string(),
                placeholder: "Enter search term".to_string(),
                focused: false,
            }),
            ("volume_slider", "Slider", ComponentData::Slider {
                value: 50.0,
                min: 0.0,
                max: 100.0,
                step: 1.0,
            }),
            ("enable_notifications", "Checkbox", ComponentData::Checkbox {
                checked: true,
                label: "Enable notifications".to_string(),
            }),
            ("todo_list", "List", ComponentData::List {
                items: vec!["Task 1".to_string(), "Task 2".to_string()],
                selected: Some(0),
            }),
            ("secondary_button", "Button", ComponentData::Button {
                text: "Secondary".to_string(),
                clicks: 0,
                enabled: false,
            }),
            ("settings_input", "TextInput", ComponentData::TextInput {
                value: "Default setting".to_string(),
                placeholder: "Setting value".to_string(),
                focused: false,
            }),
        ];
        
        for (id, comp_type, data) in component_specs {
            let component = DemoComponent::new(id.to_string(), comp_type.to_string(), data);
            
            // Register with engine
            self.engine.register_component(id.to_string(), comp_type.to_string())?;
            self.engine.update_component_state(id, ComponentState::Active)?;
            
            // Preserve initial state
            let state_json = serde_json::to_value(&component.data)?;
            self.engine.preserve_component_state(id, state_json)?;
            
            println!("  ✅ Created: {}", component.get_display_info());
            self.components.insert(id.to_string(), component);
        }
        
        println!("🎉 Created {} components successfully", self.components.len());
        Ok(())
    }
    
    fn simulate_user_session(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🎮 Simulating realistic user session...");
        
        let interaction_sequence = vec![
            "main_button", "search_input", "volume_slider", 
            "enable_notifications", "todo_list", "main_button",
            "settings_input", "secondary_button", "volume_slider",
            "todo_list", "main_button", "search_input"
        ];
        
        for (i, component_id) in interaction_sequence.iter().enumerate() {
            if let Some(component) = self.components.get_mut(*component_id) {
                println!("\n  Step {}: Interacting with {}", i + 1, component.get_display_info());
                
                // Update component state to updating
                self.engine.update_component_state(component_id, ComponentState::Updating)?;
                
                // Simulate interaction
                component.interact();
                self.interaction_count += 1;
                
                // Preserve state after interaction
                let state_json = serde_json::to_value(&component.data)?;
                self.engine.preserve_component_state(component_id, state_json)?;
                
                // Update back to active
                self.engine.update_component_state(component_id, ComponentState::Active)?;
                
                // Simulate realistic timing
                thread::sleep(Duration::from_millis(200 + (i * 50) as u64));
            }
        }
        
        println!("\n✅ User session completed: {} interactions", self.interaction_count);
        Ok(())
    }
    
    fn simulate_hot_reload_cycle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🔥 Simulating hot reload cycle...");
        
        // Simulate multiple file changes
        let files_to_update = vec![
            "examples/comprehensive_demo.rs",
            "crates/rustyui-core/src/engine.rs",
            "crates/rustyui-core/src/component_lifecycle.rs",
        ];
        
        for (_i, file_path) in files_to_update.iter().enumerate() {
            println!("\n  🔄 Processing file change: {}", file_path);
            
            let updated_components = self.engine.process_file_change_and_update(file_path)?;
            
            if !updated_components.is_empty() {
                println!("    📝 Components affected: {:?}", updated_components);
                
                // Restore state for affected components
                for component_id in &updated_components {
                    if let Some(restored_state) = self.engine.restore_component_state(component_id) {
                        if let Ok(component_data) = serde_json::from_value::<ComponentData>(restored_state) {
                            if let Some(component) = self.components.get_mut(component_id) {
                                let _ = component.restore_state(component_data);
                            }
                        }
                    }
                }
            } else {
                println!("    ℹ️ No components affected by this change");
            }
            
            // Simulate processing time
            thread::sleep(Duration::from_millis(100));
        }
        
        println!("✅ Hot reload cycle completed");
        Ok(())
    }
    
    fn demonstrate_error_recovery(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🛡️ Demonstrating error recovery...");
        
        // Simulate various error scenarios
        let error_scenarios = vec![
            ("main_button", "Simulated component error"),
            ("search_input", "State corruption simulation"),
            ("todo_list", "Update failure simulation"),
        ];
        
        for (component_id, error_msg) in error_scenarios {
            println!("  ⚠️ Simulating error in '{}': {}", component_id, error_msg);
            
            // Create a mock error
            let error = rustyui_core::RustyUIError::generic(error_msg);
            let operation = rustyui_core::error_recovery::Operation::ComponentRendering;
            
            // Handle error with recovery
            let recovery_result = self.engine.handle_error(&error, operation, Some(component_id.to_string()));
            
            match recovery_result {
                rustyui_core::error_recovery::RecoveryResult::Success { strategy, message, .. } => {
                    println!("    ✅ Recovery successful: {} (strategy: {:?})", message, strategy);
                }
                rustyui_core::error_recovery::RecoveryResult::PartialRecovery { strategy, message, .. } => {
                    println!("    ⚠️ Partial recovery: {} (strategy: {:?})", message, strategy);
                }
                rustyui_core::error_recovery::RecoveryResult::Failed { strategy, message } => {
                    println!("    ❌ Recovery failed: {} (strategy: {:?})", message, strategy);
                }
            }
            
            thread::sleep(Duration::from_millis(150));
        }
        
        println!("✅ Error recovery demonstration completed");
        Ok(())
    }
    
    fn display_comprehensive_statistics(&self) {
        println!("\n📊 Comprehensive RustyUI Statistics");
        println!("=====================================");
        
        // Runtime statistics
        let runtime = self.start_time.elapsed();
        println!("⏱️ Runtime: {:.2}s", runtime.as_secs_f32());
        println!("🎮 Total interactions: {}", self.interaction_count);
        
        // Component statistics
        if let Some(stats) = self.engine.get_component_statistics() {
            self.display_component_stats(&stats);
        }
        
        // Performance statistics
        self.display_performance_stats();
        
        // Engine status
        self.display_engine_status();
        
        // Component details
        self.display_component_details();
        
        // Error recovery status
        self.display_error_recovery_status();
    }
    
    fn display_component_stats(&self, stats: &ComponentStatistics) {
        println!("\n🧩 Component Statistics:");
        println!("  Total components: {}", stats.total_components);
        println!("  Active components: {}", stats.active_components);
        println!("  Total updates: {}", stats.total_updates);
        
        if let Some(avg_age) = stats.average_age {
            println!("  Average component age: {:.2}s", avg_age.as_secs_f32());
        }
        
        let update_rate = if self.start_time.elapsed().as_secs_f32() > 0.0 {
            stats.total_updates as f32 / self.start_time.elapsed().as_secs_f32()
        } else {
            0.0
        };
        println!("  Update rate: {:.1} updates/sec", update_rate);
    }
    
    fn display_performance_stats(&self) {
        println!("\n⚡ Performance Statistics:");
        let memory_overhead = self.engine.memory_overhead();
        println!("  Memory overhead: {:.1} KB", memory_overhead as f32 / 1024.0);
        println!("  Expected memory overhead: {:.1} KB", 
            self.engine.expected_memory_overhead() as f32 / 1024.0);
        
        if let Some(file_stats) = self.engine.get_file_watching_stats() {
            println!("  File events processed: {}", file_stats.total_events);
            if file_stats.total_processing_time.as_secs_f32() > 0.0 {
                let events_per_second = file_stats.total_events as f32 / file_stats.total_processing_time.as_secs_f32();
                println!("  File processing rate: {:.1} events/sec", events_per_second);
            }
        }
        
        if let Some(analysis_stats) = self.engine.get_analysis_stats() {
            println!("  Changes analyzed: {}", analysis_stats.total_analyses);
            println!("  Analysis cache hit rate: {:.1}%", analysis_stats.cache_hit_rate() * 100.0);
        }
    }
    
    fn display_engine_status(&self) {
        println!("\n🔧 Engine Status:");
        println!("  Initialized: {}", self.engine.is_initialized());
        println!("  Hot reload active: {}", self.engine.has_runtime_interpreter());
        println!("  Can interpret changes: {}", self.engine.can_interpret_changes());
        println!("  Platform: {:?}", self.engine.platform());
        println!("  Native optimizations: {}", self.engine.is_using_native_optimizations());
        println!("  JIT compilation available: {}", self.engine.jit_compilation_available());
        
        let health_status = self.engine.get_health_status();
        println!("  System health: {:?}", health_status);
    }
    
    fn display_component_details(&self) {
        println!("\n📋 Component Details:");
        let active_components = self.engine.get_active_components();
        
        for component_lifecycle in active_components {
            if let Some(demo_component) = self.components.get(&component_lifecycle.id) {
                let age = component_lifecycle.created_at.elapsed().unwrap_or_default();
                println!("  • {} (age: {:.1}s, updates: {})", 
                    demo_component.get_display_info(),
                    age.as_secs_f32(),
                    component_lifecycle.update_count
                );
            }
        }
    }
    
    fn display_error_recovery_status(&self) {
        println!("\n🛡️ Error Recovery Status:");
        
        if let Some(recovery_metrics) = self.engine.get_error_recovery_metrics() {
            println!("  Total errors handled: {}", recovery_metrics.total_errors);
            println!("  Successful recoveries: {}", recovery_metrics.successful_recoveries);
            println!("  Recovery success rate: {:.1}%", recovery_metrics.success_rate() * 100.0);
        }
        
        if let Some(error_metrics) = self.engine.get_error_metrics() {
            println!("  Errors by severity:");
            println!("    Critical: {}", error_metrics.critical_errors);
            println!("    High: {}", error_metrics.high_errors);
            println!("    Medium: {}", error_metrics.medium_errors);
            println!("    Low: {}", error_metrics.low_errors);
        }
    }
    
    fn run_comprehensive_demo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🎬 Starting Comprehensive RustyUI Demo");
        println!("======================================");
        
        // Phase 1: Component creation
        self.create_demo_components()?;
        thread::sleep(Duration::from_millis(500));
        
        // Phase 2: User interaction simulation
        self.simulate_user_session()?;
        thread::sleep(Duration::from_millis(500));
        
        // Phase 3: Hot reload demonstration
        self.simulate_hot_reload_cycle()?;
        thread::sleep(Duration::from_millis(500));
        
        // Phase 4: Error recovery demonstration
        self.demonstrate_error_recovery()?;
        thread::sleep(Duration::from_millis(500));
        
        // Phase 5: Final statistics
        self.display_comprehensive_statistics();
        
        println!("\n🎉 Comprehensive Demo Completed Successfully!");
        println!("💡 This demo showcased:");
        println!("   • Component lifecycle management");
        println!("   • State preservation across hot reloads");
        println!("   • File change monitoring and analysis");
        println!("   • Error recovery mechanisms");
        println!("   • Performance monitoring and metrics");
        println!("   • Cross-platform compatibility");
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 RustyUI Comprehensive Demo");
    println!("=============================");
    println!("This demo showcases the complete RustyUI hot reload system\n");
    
    let mut demo = ComprehensiveDemo::new()?;
    demo.run_comprehensive_demo()?;
    
    Ok(())
}