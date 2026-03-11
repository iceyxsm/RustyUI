//! Basic example demonstrating RustyUI interpretation capabilities

use rustyui_core::{DualModeEngine, DualModeConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RustyUI Basic Interpretation Example");
    
    // Create dual-mode configuration
    let config = DualModeConfig::default();
    
    // Create and initialize the engine
    let mut engine = DualModeEngine::new(config)?;
    engine.initialize()?;
    
    println!("Engine initialized successfully");
    println!("Has runtime interpreter: {}", engine.has_runtime_interpreter());
    println!("Can interpret changes: {}", engine.can_interpret_changes());
    println!("Memory overhead: {} bytes", engine.memory_overhead());
    
    // Test UI code interpretation (only works in development mode)
    #[cfg(feature = "dev-ui")]
    {
        println!("\nTesting UI code interpretation...");
        
        // Start development mode
        engine.start_development_mode()?;
        
        // Test simple UI code
        let ui_code = r#"
            Button {
                text: "Click me!",
                on_click: || println!("Button clicked!")
            }
        "#;
        
        match engine.interpret_ui_change(ui_code, Some("button1".to_string())) {
            Ok(result) => {
                println!("✓ Interpretation successful!");
                println!("Execution time: {:?}", result.execution_time);
                println!("Success: {}", result.success);
                if let Some(error) = result.error_message {
                    println!("Error: {}", error);
                }
            }
            Err(e) => {
                println!("✗ Interpretation failed: {}", e);
            }
        }
        
        // Test more complex UI code
        let complex_ui_code = r#"
            VerticalLayout {
                Text { content: "Welcome to RustyUI!" },
                Button { 
                    text: "Start", 
                    on_click: || start_application() 
                },
                Button { 
                    text: "Settings", 
                    on_click: || open_settings() 
                }
            }
        "#;
        
        match engine.interpret_ui_change(complex_ui_code, Some("main_layout".to_string())) {
            Ok(result) => {
                println!("✓ Complex UI interpretation successful!");
                println!("Execution time: {:?}", result.execution_time);
            }
            Err(e) => {
                println!("✗ Complex UI interpretation failed: {}", e);
            }
        }
    }
    
    #[cfg(not(feature = "dev-ui"))]
    {
        println!("\nProduction mode - interpretation features disabled");
        println!("This is expected behavior for zero-overhead production builds");
    }
    
    println!("\nExample completed successfully!");
    Ok(())
}