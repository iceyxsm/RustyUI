//! Demonstration of UI component state preservation during hot reload

use rustyui_core::{DualModeEngine, DualModeConfig, UIComponent, ComponentStateManager};
use rustyui_core::ui_component::{ButtonComponent, InputComponent};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RustyUI State Preservation Demo");
    println!("Demonstrating component state preservation during hot reload\n");
    
    // Create dual-mode configuration
    let config = DualModeConfig::default();
    
    // Create and initialize the engine
    let mut engine = DualModeEngine::new(config)?;
    engine.initialize()?;
    
    println!("Engine initialized with state preservation system");
    
    #[cfg(feature = "dev-ui")]
    {
        // Create component state manager
        let mut state_manager = ComponentStateManager::new();
        
        println!("\n=== Creating UI Components ===");
        
        // Create some UI components
        let mut button1 = ButtonComponent::new("login_button".to_string(), "Login".to_string());
        let mut button2 = ButtonComponent::new("cancel_button".to_string(), "Cancel".to_string());
        let mut input1 = InputComponent::new("username_input".to_string(), "Enter username".to_string());
        let mut input2 = InputComponent::new("password_input".to_string(), "Enter password".to_string());
        
        println!("Created components:");
        println!("  - Button: {} ({})", button1.component_id(), button1.component_type());
        println!("  - Button: {} ({})", button2.component_id(), button2.component_type());
        println!("  - Input: {} ({})", input1.component_id(), input1.component_type());
        println!("  - Input: {} ({})", input2.component_id(), input2.component_type());
        
        println!("\n=== Simulating User Interaction ===");
        
        // Simulate user interactions
        button1.click();
        button1.click();
        button1.click();
        println!("Login button clicked {} times", button1.get_click_count());
        
        button2.click();
        println!("Cancel button clicked {} times", button2.get_click_count());
        
        input1.set_value("john_doe".to_string());
        input1.set_focused(true);
        println!("Username input: '{}' (focused: {})", input1.get_value(), input1.focused);
        
        input2.set_value("secret123".to_string());
        println!("Password input: '{}' characters", input2.get_value().len());
        
        println!("\n=== Saving Component States ===");
        
        // Save all component states
        state_manager.save_component_state(&mut button1)?;
        state_manager.save_component_state(&mut button2)?;
        state_manager.save_component_state(&mut input1)?;
        state_manager.save_component_state(&mut input2)?;
        
        let stats = state_manager.get_stats();
        println!("States saved: {}", stats.total_saves);
        println!("Memory usage: {} bytes", stats.memory_usage);
        
        println!("\n=== Simulating Hot Reload (Component Recreation) ===");
        
        // Simulate hot reload by creating new component instances
        let mut new_button1 = ButtonComponent::new("login_button".to_string(), "Login".to_string());
        let mut new_button2 = ButtonComponent::new("cancel_button".to_string(), "Cancel".to_string());
        let mut new_input1 = InputComponent::new("username_input".to_string(), "Enter username".to_string());
        let mut new_input2 = InputComponent::new("password_input".to_string(), "Enter password".to_string());
        
        println!("New components created (simulating hot reload)");
        println!("Initial states:");
        println!("  - Login button clicks: {}", new_button1.get_click_count());
        println!("  - Cancel button clicks: {}", new_button2.get_click_count());
        println!("  - Username input: '{}'", new_input1.get_value());
        println!("  - Password input: '{}'", new_input2.get_value());
        
        println!("\n=== Restoring Component States ===");
        
        // Restore states
        let restored1 = state_manager.restore_component_state(&mut new_button1)?;
        let restored2 = state_manager.restore_component_state(&mut new_button2)?;
        let restored3 = state_manager.restore_component_state(&mut new_input1)?;
        let restored4 = state_manager.restore_component_state(&mut new_input2)?;
        
        println!("State restoration results:");
        println!("  - Login button: {} (clicks: {})", restored1, new_button1.get_click_count());
        println!("  - Cancel button: {} (clicks: {})", restored2, new_button2.get_click_count());
        println!("  - Username input: {} (value: '{}')", restored3, new_input1.get_value());
        println!("  - Password input: {} (value length: {})", restored4, new_input2.get_value().len());
        
        let final_stats = state_manager.get_stats();
        println!("\nFinal statistics:");
        println!("  - Total saves: {}", final_stats.total_saves);
        println!("  - Total restores: {}", final_stats.total_restores);
        println!("  - Serialization failures: {}", final_stats.serialization_failures);
        println!("  - Deserialization failures: {}", final_stats.deserialization_failures);
        
        println!("\n=== Testing State Validation ===");
        
        // Test state validation with invalid data
        let mut test_input = InputComponent::new("test_input".to_string(), "Test".to_string());
        test_input.set_value("x".repeat(15000)); // Very long value
        
        match state_manager.save_component_state(&mut test_input) {
            Ok(_) => {
                println!("✓ Long input value saved successfully");
                
                // Try to restore it
                let mut restore_input = InputComponent::new("test_input".to_string(), "Test".to_string());
                match state_manager.restore_component_state(&mut restore_input) {
                    Ok(restored) => {
                        if restored {
                            println!("✗ Long input value should have failed validation but was restored");
                        } else {
                            println!("✓ Long input value correctly not restored");
                        }
                    }
                    Err(e) => {
                        println!("✓ Long input value validation failed as expected: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("✓ Long input value correctly rejected: {}", e);
            }
        }
        
        println!("\n=== Testing Component Type Validation ===");
        
        // Test component type mismatch
        let mut button_for_mismatch = ButtonComponent::new("mismatch_test".to_string(), "Test".to_string());
        button_for_mismatch.click();
        state_manager.save_component_state(&mut button_for_mismatch)?;
        
        // Try to restore button state as input
        let mut input_for_mismatch = InputComponent::new("mismatch_test".to_string(), "Test".to_string());
        match state_manager.restore_component_state(&mut input_for_mismatch) {
            Ok(_) => println!("✗ Component type mismatch should have failed"),
            Err(e) => println!("✓ Component type mismatch correctly detected: {}", e),
        }
        
        println!("\n=== Testing Priority-Based State Preservation ===");
        
        // Demonstrate priority ordering
        println!("Component priorities:");
        println!("  - Button priority: {}", new_button1.state_preservation_priority());
        println!("  - Input priority: {}", new_input1.state_preservation_priority());
        println!("  - Input has higher priority: {}", 
            new_input1.state_preservation_priority() > new_button1.state_preservation_priority());
        
        println!("\n=== State Preservation Demo Completed ===");
        println!("Key features demonstrated:");
        println!("  ✓ Component state serialization and deserialization");
        println!("  ✓ Type-safe state restoration with validation");
        println!("  ✓ Component ID and type validation");
        println!("  ✓ Priority-based state preservation");
        println!("  ✓ Memory usage tracking and statistics");
        println!("  ✓ Error handling for invalid states");
        println!("  ✓ Hot reload simulation with state preservation");
        
        println!("\nMemory overhead: {} bytes", engine.memory_overhead());
    }
    
    #[cfg(not(feature = "dev-ui"))]
    {
        println!("Production mode - state preservation features disabled");
        println!("This demonstrates zero-overhead production builds");
        println!("Memory overhead: {} bytes", engine.memory_overhead());
    }
    
    Ok(())
}