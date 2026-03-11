//! Demonstration of UI code interpretation capabilities

use rustyui_core::{DualModeEngine, DualModeConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RustyUI UI Code Interpretation Demo");
    println!("Demonstrating runtime interpretation of UI changes\n");
    
    // Create dual-mode configuration
    let config = DualModeConfig::default();
    
    // Create and initialize the engine
    let mut engine = DualModeEngine::new(config)?;
    engine.initialize()?;
    
    println!("Engine initialized with runtime interpreter");
    
    #[cfg(feature = "dev-ui")]
    {
        // Test basic UI component interpretation
        println!("\n=== Testing Basic UI Components ===");
        
        let button_code = r#"
            button("Click Me")
        "#;
        
        match engine.interpret_ui_change(button_code, Some("button1".to_string())) {
            Ok(result) => {
                println!("✓ Button interpretation: {:?}", result.execution_time);
                if !result.success {
                    println!("Error: {:?}", result.error_message);
                }
            }
            Err(e) => println!("✗ Button interpretation failed: {}", e),
        }
        
        let text_code = r#"
            text("Hello, RustyUI!")
        "#;
        
        match engine.interpret_ui_change(text_code, Some("text1".to_string())) {
            Ok(result) => {
                println!("✓ Text interpretation: {:?}", result.execution_time);
            }
            Err(e) => println!("✗ Text interpretation failed: {}", e),
        }
        
        // Test layout interpretation
        println!("\n=== Testing Layout Components ===");
        
        let layout_code = r#"
            vertical_layout();
            horizontal_layout();
            grid_layout(3, 2)
        "#;
        
        match engine.interpret_ui_change(layout_code, Some("layout1".to_string())) {
            Ok(result) => {
                println!("✓ Layout interpretation: {:?}", result.execution_time);
            }
            Err(e) => println!("✗ Layout interpretation failed: {}", e),
        }
        
        // Test styling interpretation
        println!("\n=== Testing Styling ===");
        
        let style_code = r#"
            style("background-color", "blue");
            color(255, 0, 0);
            padding(10);
            margin(5)
        "#;
        
        match engine.interpret_ui_change(style_code, Some("style1".to_string())) {
            Ok(result) => {
                println!("✓ Style interpretation: {:?}", result.execution_time);
            }
            Err(e) => println!("✗ Style interpretation failed: {}", e),
        }
        
        // Test state management
        println!("\n=== Testing State Management ===");
        
        let state_code = r#"
            set_state("counter", 42);
            let value = get_state("counter");
            update_component("counter_display", "text", value)
        "#;
        
        match engine.interpret_ui_change(state_code, Some("state1".to_string())) {
            Ok(result) => {
                println!("✓ State interpretation: {:?}", result.execution_time);
            }
            Err(e) => println!("✗ State interpretation failed: {}", e),
        }
        
        // Test event handling
        println!("\n=== Testing Event Handling ===");
        
        let event_code = r#"
            on_click("handle_button_click");
            on_change("handle_input_change");
            on_hover("handle_mouse_hover")
        "#;
        
        match engine.interpret_ui_change(event_code, Some("events1".to_string())) {
            Ok(result) => {
                println!("✓ Event interpretation: {:?}", result.execution_time);
            }
            Err(e) => println!("✗ Event interpretation failed: {}", e),
        }
        
        // Test complex UI composition
        println!("\n=== Testing Complex UI Composition ===");
        
        let complex_code = r#"
            // Create a login form
            vertical_layout();
            text("Login Form");
            input("Username");
            input("Password");
            
            horizontal_layout();
            button("Login");
            button("Cancel");
            
            // Style the form
            style("padding", "20px");
            style("border", "1px solid #ccc");
            color(50, 50, 50);
            
            // Add event handlers
            on_click("handle_login");
            
            // Log completion
            log("Login form created successfully")
        "#;
        
        match engine.interpret_ui_change(complex_code, Some("login_form".to_string())) {
            Ok(result) => {
                println!("✓ Complex UI interpretation: {:?}", result.execution_time);
                if result.execution_time > std::time::Duration::from_millis(100) {
                    println!("WARNING: Interpretation took longer than 100ms target");
                } else {
                    println!("✓ Performance target met (<100ms)");
                }
            }
            Err(e) => println!("✗ Complex UI interpretation failed: {}", e),
        }
        
        // Test error handling
        println!("\n=== Testing Error Handling ===");
        
        let invalid_code = r#"
            invalid_function_call();
            syntax error here
        "#;
        
        match engine.interpret_ui_change(invalid_code, Some("error_test".to_string())) {
            Ok(result) => {
                if result.success {
                    println!("✗ Expected error but interpretation succeeded");
                } else {
                    println!("✓ Error handling works: {:?}", result.error_message);
                }
            }
            Err(e) => println!("✓ Error caught at engine level: {}", e),
        }
        
        println!("\n=== Performance Summary ===");
        println!("All UI interpretation tests completed");
        println!("Target: <100ms for complex UI changes");
        println!("Target: <10ms for simple UI changes");
        println!("Memory overhead: {} bytes", engine.memory_overhead());
    }
    
    #[cfg(not(feature = "dev-ui"))]
    {
        println!("Production mode - UI interpretation features disabled");
        println!("This demonstrates zero-overhead production builds");
        println!("Memory overhead: {} bytes", engine.memory_overhead());
    }
    
    Ok(())
}