//! File watching demonstration with 2026 performance optimizations

use rustyui_core::{DualModeEngine, DualModeConfig};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 RustyUI File Watching Demo (2026 Optimizations)");
    
    // Create dual-mode configuration
    let config = DualModeConfig::default();
    
    // Create and initialize the engine
    let mut engine = DualModeEngine::new(config)?;
    engine.initialize()?;
    
    println!("✅ Engine initialized successfully");
    
    #[cfg(feature = "dev-ui")]
    {
        println!("\n🔍 Starting file watching with 2026 optimizations...");
        
        // Start development mode with file watching
        engine.start_development_mode()?;
        
        println!("📁 Watching for file changes in:");
        println!("   - src/ (Rust files)");
        println!("   - Cargo.toml (Configuration)");
        println!("   - *.json (Data files)");
        
        println!("\n⚡ Performance targets:");
        println!("   - Change detection: <50ms");
        println!("   - Debounce delay: 25ms (2026 optimized)");
        println!("   - Memory overhead: <100MB");
        
        println!("\n🎯 Try editing files in the project to see instant detection!");
        println!("   Edit this file (examples/file_watching_demo.rs) to test");
        
        // Monitor for file changes for 30 seconds
        let start_time = std::time::Instant::now();
        let mut total_changes = 0;
        
        while start_time.elapsed() < Duration::from_secs(30) {
            // Process file changes
            match engine.process_file_changes() {
                Ok(changes) => {
                    for change in changes {
                        total_changes += 1;
                        println!("📝 File change detected:");
                        println!("   Path: {:?}", change.path);
                        println!("   Type: {:?}", change.change_type);
                        println!("   Priority: {:?}", change.priority);
                        println!("   Timestamp: {:?}", change.timestamp);
                        
                        // Demonstrate UI interpretation for Rust files
                        if change.path.extension().map_or(false, |ext| ext == "rs") {
                            println!("   🔄 Triggering UI interpretation...");
                            
                            let sample_ui_code = r#"
                                Button {
                                    text: "File Changed!",
                                    on_click: || println!("Detected change in {:?}", change.path)
                                }
                            "#;
                            
                            match engine.interpret_ui_change(sample_ui_code, Some("file_change_button".to_string())) {
                                Ok(result) => {
                                    println!("   ✅ UI interpretation successful in {:?}", result.execution_time);
                                }
                                Err(e) => {
                                    println!("   ❌ UI interpretation failed: {}", e);
                                }
                            }
                        }
                        
                        println!();
                    }
                }
                Err(e) => {
                    println!("❌ Error processing file changes: {}", e);
                }
            }
            
            // Show performance stats every 10 seconds
            if start_time.elapsed().as_secs() % 10 == 0 && start_time.elapsed().as_secs() > 0 {
                if let Some(stats) = engine.get_file_watching_stats() {
                    println!("📊 Performance Stats:");
                    println!("   Events processed: {}", stats.events_processed);
                    println!("   Average response time: {:?}", stats.average_response_time);
                    println!("   Debounce delay: {:?}", stats.debounce_delay);
                    println!("   Performance optimal: {}", stats.is_optimal);
                    println!();
                }
            }
            
            // Small delay to prevent busy waiting
            std::thread::sleep(Duration::from_millis(100));
        }
        
        println!("📈 Demo completed!");
        println!("   Total changes detected: {}", total_changes);
        
        if let Some(stats) = engine.get_file_watching_stats() {
            println!("   Final performance stats:");
            println!("     Events processed: {}", stats.events_processed);
            println!("     Average response time: {:?}", stats.average_response_time);
            println!("     Performance optimal: {}", if stats.is_optimal { "✅ Yes" } else { "⚠️  No" });
        }
    }
    
    #[cfg(not(feature = "dev-ui"))]
    {
        println!("\n🏭 Production mode - file watching disabled");
        println!("This is expected behavior for zero-overhead production builds");
        println!("Memory overhead: {} bytes", engine.memory_overhead());
    }
    
    println!("\n🎉 File watching demo completed successfully!");
    Ok(())
}