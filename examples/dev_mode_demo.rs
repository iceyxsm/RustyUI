//! Development mode demonstration
//! 
//! This example shows how to use the enhanced rustyui dev command
//! with full dual-mode engine integration and hot reload capabilities.

use rustyui_core::{DualModeEngine, DualModeConfig};
use rustyui_core::config::{UIFramework, DevelopmentSettings, InterpretationStrategy};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 RustyUI Development Mode Demo");
    
    // Create a sample configuration for development mode
    let config = DualModeConfig {
        framework: UIFramework::Egui,
        watch_paths: vec![
            PathBuf::from("src"),
            PathBuf::from("examples"),
        ],
        development_settings: DevelopmentSettings {
            interpretation_strategy: InterpretationStrategy::Hybrid { 
                rhai_threshold: 1000, 
                jit_threshold: 5000 
            },
            jit_compilation_threshold: 100,
            state_preservation: true,
            performance_monitoring: true,
            change_detection_delay_ms: 50,
        },
        ..Default::default()
    };
    
    println!("📋 Configuration:");
    println!("  Framework: {:?}", config.framework);
    println!("  Watch paths: {:?}", config.watch_paths);
    println!("  State preservation: {}", config.development_settings.state_preservation);
    println!("  Performance monitoring: {}", config.development_settings.performance_monitoring);
    
    #[cfg(feature = "dev-ui")]
    {
        println!("\n🔧 Creating dual-mode engine...");
        
        // Create and initialize the dual-mode engine
        let mut engine = DualModeEngine::new(config)?;
        engine.initialize()?;
        
        println!("✅ Engine initialized successfully!");
        println!("  Runtime interpreter available: {}", engine.has_runtime_interpreter());
        println!("  Can interpret changes: {}", engine.can_interpret_changes());
        println!("  Memory overhead: {:.1} MB", engine.memory_overhead() as f64 / (1024.0 * 1024.0));
        
        // Demonstrate file change processing
        println!("\n🔍 Testing file change processing...");
        let changes = engine.process_file_changes()?;
        println!("  Found {} pending changes", changes.len());
        
        // Test interpretation with sample code
        println!("\n🎯 Testing code interpretation...");
        let sample_code = r#"
            // Sample UI component code
            pub struct Button {
                text: String,
                clicked: bool,
            }
            
            impl Button {
                pub fn new(text: &str) -> Self {
                    Self {
                        text: text.to_string(),
                        clicked: false,
                    }
                }
            }
        "#;
        
        match engine.interpret_ui_change(sample_code, Some("Button".to_string())) {
            Ok(result) => {
                println!("  ✅ Interpretation successful!");
                println!("    Execution time: {:?}", result.execution_time);
                println!("    Success: {}", result.success);
            }
            Err(e) => {
                println!("  ❌ Interpretation failed: {}", e);
            }
        }
        
        println!("\n🎉 Development mode demo completed successfully!");
        println!("  Use 'rustyui dev' in your project to start the enhanced development server");
    }
    
    #[cfg(not(feature = "dev-ui"))]
    {
        println!("\n⚠️  Development features not available in this build");
        println!("  Run with: cargo run --example dev_mode_demo --features dev-ui");
    }
    
    Ok(())
}