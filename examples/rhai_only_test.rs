//! Simple test to verify Rhai interpreter implementation works

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
    
    // Test more complex script
    let complex_script = r#"
        fn fibonacci(n) {
            if n <= 1 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        }
        
        let result = fibonacci(10);
        result
    "#;
    
    let result3 = interpreter.interpret(complex_script)?;
    println!("Complex Rhai Result: success={}, time={:?}", result3.success, result3.execution_time);
    assert!(result3.success);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Rhai Interpreter Only ===\n");
    
    test_rhai_interpreter()?;
    println!();
    
    println!("=== Rhai test completed successfully! ===");
    Ok(())
}