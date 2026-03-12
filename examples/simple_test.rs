//! Simple test to verify Rhai interpreter and Egui adapter implementations work

use std::time::Duration;

// Test Rhai interpreter
fn test_rhai_interpreter() -> Result<(), Box<dyn std::error::Error>> {
    use rustyui_interpreter::RhaiInterpreter;
    
    println!("Testing Rhai Interpreter...");
    
    let mut interpreter = RhaiInterpreter::new()?;
    
    // Test simple script
    let script = "let x = 42; x * 2";
    let result = interpreter.interpret(script)?;
    
    println!("Rhai Result: success={}, time={:?}", result.success, result.execution_time);
    assert!(result.success);
    
    // Test caching
    let result2 = interpreter.interpret(script)?;
    println!("Rhai Cached Result: success={}, time={:?}", result2.success, result2.execution_time);
    
    let hit_rate = interpreter.cache_hit_rate();
    println!("Rhai Cache Hit Rate: {:.2}%", hit_rate * 100.0);
    
    Ok(())
}

// Test Egui adapter
fn test_egui_adapter() -> Result<(), Box<dyn std::error::Error>> {
    use rustyui_adapters::{EguiAdapter, traits::{UIFrameworkAdapter, FrameworkConfig}};
    
    println!("Testing Egui Adapter...");
    
    let mut adapter = EguiAdapter::new();
    
    // Test initialization
    let config = FrameworkConfig::default();
    adapter.initialize(&config)?;
    
    println!("Egui Adapter: framework={}", adapter.framework_name());
    assert_eq!(adapter.framework_name(), "egui");
    
    // Test render context creation
    let _ctx = adapter.create_render_context()?;
    println!("Egui render context created successfully");
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing RustyUI Implementations (Simple) ===\n");
    
    test_rhai_interpreter()?;
    println!();
    
    test_egui_adapter()?;
    println!();
    
    println!("=== All tests completed successfully! ===");
    Ok(())
}