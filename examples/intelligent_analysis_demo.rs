//! Demonstration of advanced change analysis and classification

use rustyui_core::{DualModeEngine, DualModeConfig};
use std::time::Duration;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RustyUI Advanced Change Analysis Demo");
    println!("Demonstrating file change analysis with priority-based processing\n");
    
    // Create dual-mode configuration
    let config = DualModeConfig::default();
    
    // Create and initialize the engine
    let mut engine = DualModeEngine::new(config)?;
    engine.initialize()?;
    
    println!("Engine initialized with change analyzer");
    
    #[cfg(feature = "dev-ui")]
    {
        // Start development mode with file watching and analysis
        engine.start_development_mode()?;
        
        println!("Development mode started with change analysis");
        println!("Watching paths: {:?}", engine.config().watch_paths);
        println!("\nChange Classification System:");
        println!("Critical: Cargo.toml, core configuration files");
        println!("High: .rs files, rustyui.toml, CSS/SCSS");
        println!("Medium: JSON data files");
        println!("Low: Assets (PNG, SVG, etc.)");
        println!("Very Low: Documentation (.md files)");
        println!("\nTry editing different file types to see classification!");
        println!("Press Ctrl+C to exit\n");
        
        // Monitor and analyze file changes for 30 seconds
        let start_time = std::time::Instant::now();
        let mut total_changes = 0;
        let mut critical_changes = 0;
        let mut high_priority_changes = 0;
        
        while start_time.elapsed() < Duration::from_secs(30) {
            // Process and analyze changes with classification
            if let Ok(Some(analysis)) = engine.process_and_analyze_changes() {
                total_changes += analysis.analyzed_changes.len();
                
                // Demonstrate classification
                for (i, analyzed_change) in analysis.analyzed_changes.iter().enumerate() {
                    let priority_marker = match analyzed_change.classification.priority {
                        rustyui_core::change_analyzer::ChangePriority::Critical => "[CRITICAL]",
                        rustyui_core::change_analyzer::ChangePriority::High => "[HIGH]",
                        rustyui_core::change_analyzer::ChangePriority::Medium => "[MEDIUM]",
                        rustyui_core::change_analyzer::ChangePriority::Low => "[LOW]",
                        rustyui_core::change_analyzer::ChangePriority::VeryLow => "[VERY_LOW]",
                    };
                    
                    let category_name = match analyzed_change.classification.category {
                        rustyui_core::change_analyzer::ChangeCategory::UIComponent => "UI Component",
                        rustyui_core::change_analyzer::ChangeCategory::Configuration => "Configuration",
                        rustyui_core::change_analyzer::ChangeCategory::Styling => "Styling",
                        rustyui_core::change_analyzer::ChangeCategory::Asset => "Asset",
                        rustyui_core::change_analyzer::ChangeCategory::Data => "Data",
                        rustyui_core::change_analyzer::ChangeCategory::Documentation => "Documentation",
                        rustyui_core::change_analyzer::ChangeCategory::Unknown => "Unknown",
                    };
                    
                    println!("{} Change #{}: {} - {}", 
                        priority_marker,
                        total_changes - analysis.analyzed_changes.len() + i + 1,
                        category_name,
                        analyzed_change.original_change.path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                    );
                    
                    // Show analysis details
                    println!("Analysis:");
                    println!("Priority: {:?}", analyzed_change.classification.priority);
                    println!("Requires interpretation: {}", analyzed_change.classification.requires_interpretation);
                    println!("Affects layout: {}", analyzed_change.classification.affects_layout);
                    println!("Affects styling: {}", analyzed_change.classification.affects_styling);
                    println!("Impact scope: {:?}", analyzed_change.impact.scope);
                    println!("Estimated update time: {:?}", analyzed_change.impact.estimated_update_time);
                    println!("Processing order: {}", analyzed_change.processing_order);
                    
                    // Count priority changes
                    match analyzed_change.classification.priority {
                        rustyui_core::change_analyzer::ChangePriority::Critical => critical_changes += 1,
                        rustyui_core::change_analyzer::ChangePriority::High => high_priority_changes += 1,
                        _ => {}
                    }
                    
                    // Demonstrate UI interpretation for relevant changes
                    if analyzed_change.classification.requires_interpretation {
                        let ui_code = format!(r#"
                            // Auto-generated UI update from change analysis
                            {}Component {{ 
                                file: "{}", 
                                priority: "{:?}",
                                category: "{}",
                                timestamp: "{:?}" 
                            }}
                        "#, 
                            category_name.replace(" ", ""),
                            analyzed_change.original_change.path.file_name()
                                .unwrap_or_default().to_string_lossy(),
                            analyzed_change.classification.priority,
                            category_name,
                            analyzed_change.original_change.timestamp
                        );
                        
                        match engine.interpret_ui_change(&ui_code, Some("change_analysis".to_string())) {
                            Ok(result) => {
                                println!("UI interpreted in {:?}", result.execution_time);
                            }
                            Err(e) => {
                                println!("UI interpretation failed: {}", e);
                            }
                        }
                    }
                    
                    println!();
                }
                
                // Show batch processing information
                if analysis.processing_batches.len() > 1 {
                    println!("Batch Processing: {} batches created for optimal processing", 
                        analysis.processing_batches.len());
                }
                
                // Show cascade updates
                if !analysis.cascade_updates.is_empty() {
                    println!("Cascade Updates: {} files will be updated automatically", 
                        analysis.cascade_updates.len());
                    for cascade in &analysis.cascade_updates {
                        println!("{} -> {} ({:?})", 
                            cascade.source_file.file_name().unwrap_or_default().to_string_lossy(),
                            cascade.affected_file.file_name().unwrap_or_default().to_string_lossy(),
                            cascade.update_type
                        );
                    }
                }
                
                // Show reload requirement
                if analysis.requires_full_reload {
                    println!("Full application reload required due to critical configuration changes");
                }
                
                println!("-------------------------------------------------------------");
            }
            
            // Show performance statistics every 5 seconds
            if start_time.elapsed().as_secs() % 5 == 0 && start_time.elapsed().as_millis() % 5000 < 100 {
                if let Some(analysis_stats) = engine.get_analysis_stats() {
                    println!("\nAnalysis Performance Stats:");
                    println!("Total analyses: {}", analysis_stats.total_analyses);
                    println!("Average analysis time: {:?}", analysis_stats.average_analysis_time());
                    println!("Meets targets (<10ms): {}", analysis_stats.meets_performance_targets());
                    println!("Changes processed: {}\n", analysis_stats.changes_processed);
                }
            }
            
            // Sleep for a short time to avoid busy waiting
            thread::sleep(Duration::from_millis(100));
        }
        
        println!("\nChange Analysis Demo Completed!");
        println!("Final Statistics:");
        println!("Total changes analyzed: {}", total_changes);
        println!("Critical priority changes: {}", critical_changes);
        println!("High priority changes: {}", high_priority_changes);
        
        if let Some(analysis_stats) = engine.get_analysis_stats() {
            println!("\nAnalysis Performance Report:");
            println!("Total analyses: {}", analysis_stats.total_analyses);
            println!("Average analysis time: {:?}", analysis_stats.average_analysis_time());
            println!("Performance target met: {}", analysis_stats.meets_performance_targets());
            println!("Total changes processed: {}", analysis_stats.changes_processed);
            
            if analysis_stats.meets_performance_targets() {
                println!("Performance targets met successfully");
            } else {
                println!("Analysis performance could be improved");
            }
        }
        
        if let Some(watch_stats) = engine.get_file_watching_stats() {
            println!("\nFile Watching Performance:");
            println!("Events processed: {}", watch_stats.total_events);
            println!("Average response time: {:?}", watch_stats.average_processing_time());
            println!("Meets response targets: {}", watch_stats.meets_performance_targets());
        }
    }
    
    #[cfg(not(feature = "dev-ui"))]
    {
        println!("Production mode - change analysis features disabled");
        println!("This demonstrates zero-overhead production builds");
        println!("Memory overhead: {} bytes", engine.memory_overhead());
    }
    
    Ok(())
}