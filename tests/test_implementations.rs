//! Simple test to verify JIT compiler and Egui adapter implementations work

use std::time::Duration;

// Test JIT compiler
#[cfg(feature = "dev-ui")]
fn test_jit_compiler() -> Result<(), Box<dyn std::error::Error>> {
    use rustyui_interpreter::{JITCompiler, InterpretationResult};
    
    println!("Testing JIT Compiler...");
    
    let mut jit = JITCompiler::new()?;
    jit.initialize()?;
    
    // Test simple function compilation
    let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let result = jit.compile_and_execute(code)?;
    
    println!("JIT Result: success={}, time={:?}", result.success, result.execution_time);
    assert!(result.success);
    assert!(result.execution_time < Duration::from_secs(1));
    
    // Test caching
    let result2 = jit.compile_and_execute(code)?;
    println!("JIT Cached Result: success={}, time={:?}", result2.success, result2.execution_time);
    
    let stats = jit.get_stats();
    println!("JIT Stats: cache_hits={}, cache_misses={}", stats.cache_hits, stats.cache_misses);
    
    Ok(())
}

// Test Rhai interpreter
#[cfg(feature = "dev-ui")]
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
#[cfg(feature = "egui-adapter")]
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
    println!("=== Testing RustyUI Implementations ===\n");
    
    #[cfg(feature = "dev-ui")]
    {
        test_jit_compiler()?;
        println!();
        
        test_rhai_interpreter()?;
        println!();
    }
    
    #[cfg(feature = "egui-adapter")]
    {
        test_egui_adapter()?;
        println!();
    }
    
    #[cfg(not(any(feature = "dev-ui", feature = "egui-adapter")))]
    {
        println!("No features enabled. Enable 'dev-ui' and/or 'egui-adapter' features to test implementations.");
    }
    
    println!("=== All tests completed successfully! ===");
    Ok(())
}