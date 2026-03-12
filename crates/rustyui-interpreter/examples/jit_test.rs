//! Test the production-grade JIT compiler with real Cranelift integration

use rustyui_interpreter::{JITCompiler, Result};

fn main() -> Result<()> {
    println!("Testing RustyUI JIT Compiler with Cranelift");
    
    // Create and initialize JIT compiler
    let mut jit_compiler = JITCompiler::new()?;
    jit_compiler.initialize()?;
    
    // Test simple addition function
    let add_code = r#"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
    "#;
    
    println!("Compiling and executing add function...");
    let result = jit_compiler.compile_and_execute(add_code)?;
    
    println!("Execution result:");
    println!("  Success: {}", result.success);
    println!("  Execution time: {:?}", result.execution_time);
    println!("  Memory usage: {:?} bytes", result.memory_usage_bytes);
    println!("  Strategy used: {:?}", result.used_strategy);
    println!("  Required compilation: {:?}", result.required_compilation);
    
    if let Some(error) = result.error_message {
        println!("  Error: {}", error);
    }
    
    // Test cache hit by running the same function again
    println!("\nTesting cache hit...");
    let cached_result = jit_compiler.compile_and_execute(add_code)?;
    
    println!("Cached execution result:");
    println!("  Success: {}", cached_result.success);
    println!("  Execution time: {:?}", cached_result.execution_time);
    
    // Show compilation statistics
    let stats = jit_compiler.get_stats();
    println!("\nCompilation Statistics:");
    println!("  Cache hits: {}", stats.cache_hits);
    println!("  Cache misses: {}", stats.cache_misses);
    println!("  Total compilations: {}", stats.total_compilations);
    println!("  Cache hit rate: {:.2}%", jit_compiler.cache_hit_rate() * 100.0);
    
    println!("\nJIT Compiler test completed successfully!");
    Ok(())
}