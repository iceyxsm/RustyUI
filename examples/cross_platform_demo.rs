//! Cross-platform capabilities demonstration
//! 
//! This example shows how RustyUI detects platform capabilities and configures
//! itself for optimal performance on Windows, macOS, and Linux.

use rustyui_core::{
    Platform, PlatformConfig, DualModeConfig, DualModeEngine,
    FileWatcherBackend
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RustyUI Cross-Platform Capabilities Demo");
    println!("============================================\n");
    
    // Detect current platform
    let platform = Platform::current();
    println!("Detected Platform: {:?}", platform);
    
    // Get platform configuration
    let platform_config = PlatformConfig::auto_detect();
    println!("\n Platform Configuration:");
    println!("File Watcher Backend: {:?}", platform_config.file_watcher_backend);
    println!("Thread Count: {}", platform_config.thread_count);
    println!("JIT Compilation: {}", platform_config.use_jit_compilation);
    println!("Native APIs: {}", platform_config.use_native_apis);
    println!("Memory Strategy: {:?}", platform_config.memory_strategy);
    
    // Show file watcher performance characteristics
    let watcher_perf = platform_config.file_watcher_backend.performance_characteristics();
    println!("\n File Watcher Performance:");
    println!("Expected Latency: {}ms", watcher_perf.latency_ms);
    println!("CPU Overhead: {:.2}%", watcher_perf.cpu_overhead_percent);
    println!("Memory Overhead: {} bytes", watcher_perf.memory_overhead_bytes);
    println!("Max Files: {:?}", watcher_perf.max_files);
    
    // Show JIT capabilities
    let jit_caps = platform.jit_capabilities();
    println!("\n JIT Compilation Capabilities:");
    println!("Cranelift Support: {}", jit_caps.supports_cranelift);
    println!("LLVM Support: {}", jit_caps.supports_llvm);
    println!("Native Codegen: {}", jit_caps.supports_native_codegen);
    println!("Max JIT Threads: {}", jit_caps.max_jit_threads);
    
    // Demonstrate dual-mode engine with platform optimization
    println!("\n Initializing Dual-Mode Engine...");
    
    let config = DualModeConfig::default();
    let mut engine = DualModeEngine::new(config)?;
    
    println!("Engine Platform: {:?}", engine.platform());
    println!("JIT Available: {}", engine.jit_compilation_available());
    
    // Initialize the engine
    engine.initialize()?;
    println!("Engine initialized successfully with platform optimizations");
    
    // Show platform-specific recommendations
    println!("\n Platform-Specific Recommendations:");
    match platform {
        Platform::Windows => {
            println!("• Use ReadDirectoryChanges for optimal file watching");
            println!("• Enable memory-mapped files for large projects");
            println!("• Consider using {} threads for parallel operations", platform_config.thread_count);
        }
        Platform::MacOS => {
            println!("• Use FSEvents for ultra-low latency file watching");
            println!("• Enable vectorized operations for UI rendering");
            println!("• Leverage {} CPU cores for optimal performance", platform_config.thread_count);
        }
        Platform::Linux => {
            println!("• Use inotify for efficient file system monitoring");
            println!("• Enable all native optimizations for best performance");
            println!("• Configure {} worker threads", platform_config.thread_count);
        }
        Platform::Other(_) => {
            println!("• Use fallback implementations only");
            println!("• Expect reduced performance and functionality");
            println!("• Consider using a supported platform for development");
        }
    }
    
    #[cfg(feature = "dev-ui")]
    {
        // Check development features
        println!("\n  Development Features:");
        println!("All development features available");
        println!("• Runtime interpretation enabled");
        println!("• File watching active");
        println!("• State preservation ready");
        if jit_caps.supports_cranelift {
            println!("• JIT compilation available");
        }
    }
    
    #[cfg(not(feature = "dev-ui"))]
    {
        println!("\n Production Mode:");
        println!("All development features stripped for zero overhead");
        println!("• Binary size optimized");
        println!("• Runtime performance maximized");
        println!("• Memory usage minimized");
    }
    
    // Demonstrate file watcher backends
    demonstrate_file_watcher_backends();
    
    // Show compatibility matrix
    show_compatibility_matrix();
    
    println!("\n Cross-platform demo completed successfully!");
    
    Ok(())
}

/// Demonstrate platform-specific file watcher backends
fn demonstrate_file_watcher_backends() {
    println!("\n File Watcher Backend Comparison:");
    
    let backends = [
        FileWatcherBackend::ReadDirectoryChanges,
        FileWatcherBackend::FSEvents,
        FileWatcherBackend::INotify,
        FileWatcherBackend::Poll,
    ];
    
    for backend in &backends {
        let perf = backend.performance_characteristics();
        println!("{:?}:", backend);
        println!("Latency: {}ms", perf.latency_ms);
        println!("CPU: {:.2}%", perf.cpu_overhead_percent);
        println!("Memory: {} bytes", perf.memory_overhead_bytes);
        println!("Max Files: {:?}", perf.max_files);
    }
}

/// Show cross-platform compatibility matrix
fn show_compatibility_matrix() {
    println!("\n Cross-Platform Compatibility Matrix:");
    println!("┌─────────────┬─────────┬─────────┬─────────┬─────────┐");
    println!("│ Feature     │ Windows │ macOS   │ Linux   │ Other   │");
    println!("├─────────────┼─────────┼─────────┼─────────┼─────────┤");
    println!("│ File Watch  │ Native  │ Native  │ Native  │ Poll    │");
    println!("│ JIT Compile │ Yes     │ Yes     │ Yes     │ Limited │");
    println!("│ Memory Map  │ Yes     │ Yes     │ Yes     │ No      │");
    println!("│ Vectorized  │ Yes     │ Yes     │ Yes     │ No      │");
    println!("│ Multi-thread│ Yes     │ Yes     │ Yes     │ Limited │");
    println!("└─────────────┴─────────┴─────────┴─────────┴─────────┘");
}