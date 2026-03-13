//! Phase 2 Enhanced JIT PGO Integration Demo
//! 
//! This example demonstrates the complete PGO system integration,
//! showing tier progression, profiling, and performance improvements.

#[cfg(feature = "dev-ui")]
use rustyui_interpreter::{
    tiered_compilation::{TieredCompilationManager, TieredCompilationConfig, CompilationTier},
    profiling::{ProfilingInfrastructure, ProfilingConfig},
    hot_path_detector::{HotPathDetector, HotPathConfig},
    recompilation_scheduler::{RecompilationScheduler, RecompilationConfig},
    optimization_engine::OptimizationEngine,
    performance_tuning::PerformanceTuner,
    benchmarks::{PGOBenchmarkSuite, BenchmarkReport},
};

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
    thread,
};

#[cfg(feature = "dev-ui")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Phase 2 Enhanced JIT PGO Integration Demo ===\n");
    
    // 1. Setup PGO System
    println!("1. Setting up PGO system components...");
    
    let profiling_config = ProfilingConfig {
        enabled: true,
        sampling_rate: 10,
        max_memory: 50 * 1024 * 1024, // 50MB
        retention_period: Duration::from_secs(300),
        auto_export: false,
        export_interval: Duration::from_secs(60),
    };
    
    let profiling = Arc::new(ProfilingInfrastructure::new(profiling_config));
    println!("  ✓ Profiling infrastructure initialized");
    
    let recompilation_config = RecompilationConfig::default();
    let recompilation_scheduler = Arc::new(
        RecompilationScheduler::new(recompilation_config)?
    );
    println!("  ✓ Recompilation scheduler initialized");
    
    let optimization_engine = Arc::new(Mutex::new(
        OptimizationEngine::new()?
    ));
    println!("  ✓ Optimization engine initialized");
    
    let tiered_config = TieredCompilationConfig {
        tier1_threshold: 10,
        tier2_threshold: 50,
        tier3_threshold: 200,
        background_recompilation: true,
        max_concurrent_recompilations: 2,
        collect_statistics: true,
        profiling: profiling_config,
    };
    
    // Validate configuration
    tiered_config.validate()?;
    println!("  ✓ Configuration validated");
    
    let manager = TieredCompilationManager::with_pgo_integration(
        tiered_config,
        profiling.clone(),
        recompilation_scheduler,
        optimization_engine,
    );
    println!("  ✓ Tiered compilation manager initialized\n");
    
    // 2. Demonstrate Cold Start to Hot Path Workflow
    println!("2. Demonstrating cold start to hot path workflow...");
    
    let function_id = "fibonacci_demo";
    let fibonacci_code = "
        fn fibonacci(n) {
            if n <= 1 {
                return n;
            } else {
                return fibonacci(n - 1) + fibonacci(n - 2);
            }
        }
        fibonacci(10)
    ";
    
    // Track tier progression
    let mut execution_count = 0;
    let mut tier_changes = Vec::new();
    
    // Execute function multiple times to trigger tier promotions
    for batch in 0..5 {
        println!("  Batch {} (executions {}-{}):", batch + 1, execution_count + 1, execution_count + 50);
        
        let batch_start = Instant::now();
        
        for _ in 0..50 {
            execution_count += 1;
            
            let result = manager.execute_with_profiling(function_id, fibonacci_code);
            if let Err(e) = result {
                println!("    Execution {} failed: {:?}", execution_count, e);
                continue;
            }
            
            // Check for tier changes
            if let Some(metadata) = manager.get_metadata(function_id) {
                let current_tier = metadata.current_tier;
                if tier_changes.is_empty() || tier_changes.last().unwrap().1 != current_tier {
                    tier_changes.push((execution_count, current_tier));
                    println!("    → Tier promotion at execution {}: {:?}", execution_count, current_tier);
                }
            }
            
            // Small delay to allow background recompilation
            if execution_count % 10 == 0 {
                thread::sleep(Duration::from_millis(5));
            }
        }
        
        let batch_time = batch_start.elapsed();
        println!("    Batch completed in {:?} (avg: {:?} per execution)", 
                batch_time, batch_time / 50);
        
        // Show current statistics
        let stats = manager.get_statistics();
        println!("    Statistics: {:?}", stats.executions_per_tier);
        
        thread::sleep(Duration::from_millis(10)); // Allow background processing
    }
    
    println!("\n  Tier progression summary:");
    for (execution, tier) in &tier_changes {
        println!("    Execution {}: {:?}", execution, tier);
    }
    
    // 3. Show Profiling Data
    println!("\n3. Profiling data collected:");
    
    if let Some(profile_data) = profiling.get_profile(function_id) {
        println!("  Execution count: {}", profile_data.get_execution_count());
        
        let overhead = profiling.get_overhead_percentage();
        println!("  Profiling overhead: {:.2}%", overhead);
        
        if overhead <= 5.0 {
            println!("  ✓ Overhead within 5% target");
        } else {
            println!("  ⚠ Overhead exceeds 5% target");
        }
    }
    
    // 4. Show Hot Path Detection
    println!("\n4. Hot path detection results:");
    
    if let Some(detector) = manager.get_hot_path_detector() {
        let hot_functions = detector.detect_hot_functions();
        println!("  Hot functions detected: {}", hot_functions.len());
        
        for hot_function in &hot_functions {
            println!("    {} - Tier: {:?}, Priority: {:.2}", 
                    hot_function.function_id, 
                    hot_function.current_tier,
                    hot_function.priority_score);
        }
    }
    
    // 5. Show Optimization Recommendations
    println!("\n5. Optimization recommendations:");
    
    let recommendations = manager.get_optimization_recommendations();
    if recommendations.is_empty() {
        println!("  No recommendations (function already optimized)");
    } else {
        for rec in &recommendations {
            println!("    {} - Current: {:?}, Recommended: {:?}, Priority: {:.2}",
                    rec.function_id,
                    rec.current_tier,
                    rec.recommended_tier,
                    rec.priority_score);
        }
    }
    
    // 6. Performance Benchmarking
    println!("\n6. Running performance benchmarks...");
    
    let mut benchmark_suite = PGOBenchmarkSuite::new();
    let benchmark_report = benchmark_suite.run_all_benchmarks();
    
    benchmark_report.print_report();
    
    // 7. Performance Tuning Recommendations
    println!("\n7. Performance tuning analysis:");
    
    let mut tuner = PerformanceTuner::new();
    
    // Simulate some measurements for demonstration
    for _ in 0..20 {
        tuner.measure_baseline(|| thread::sleep(Duration::from_millis(1)));
        tuner.measure_pgo(|| thread::sleep(Duration::from_millis(1)));
    }
    
    let performance_report = tuner.generate_report();
    println!("  {}", performance_report.summary());
    
    if performance_report.meets_requirements() {
        println!("  ✓ Performance requirements met");
    } else {
        println!("  ⚠ Performance requirements not met");
        for rec in &performance_report.recommendations {
            println!("    - {}: {}", rec.description, rec.suggestion);
        }
    }
    
    // 8. Final Statistics
    println!("\n8. Final system statistics:");
    
    let final_stats = manager.get_statistics();
    println!("  Executions per tier: {:?}", final_stats.executions_per_tier);
    println!("  Functions per tier: {:?}", final_stats.functions_per_tier);
    println!("  Total recompilation time: {:?}", final_stats.total_recompilation_time);
    
    if let Some(final_metadata) = manager.get_metadata(function_id) {
        println!("  Final function state:");
        println!("    Tier: {:?}", final_metadata.current_tier);
        println!("    Execution count: {}", final_metadata.execution_count);
        println!("    Total execution time: {:?}", final_metadata.total_execution_time);
        println!("    Compilation time: {:?}", final_metadata.compilation_time);
        println!("    Recompilation count: {}", final_metadata.recompilation_count);
    }
    
    println!("\n=== Demo completed successfully! ===");
    
    Ok(())
}

#[cfg(not(feature = "dev-ui"))]
fn main() {
    println!("This demo requires the 'dev-ui' feature to be enabled.");
    println!("Run with: cargo run --example pgo_integration_demo --features dev-ui");
}