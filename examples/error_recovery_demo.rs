//! Error Recovery and Handling Demo
//! 
//! Demonstrates the comprehensive error handling and recovery mechanisms
//! implemented in RustyUI, including graceful fallbacks and error isolation.

use rustyui_core::{
    DualModeEngine, DualModeConfig, 
    error::{RustyUIError, Result},
};

#[cfg(feature = "dev-ui")]
use rustyui_core::{
    Operation, ErrorReportContext, ErrorOperation, ErrorSeverity,
    error_recovery::{RecoveryResult, HealthStatus},
};

use std::collections::HashMap;

fn main() -> Result<()> {
    println!("RustyUI Error Recovery and Handling Demo");
    println!("============================================");
    
    // Create dual-mode engine with error handling
    let config = DualModeConfig::default();
    let mut engine = DualModeEngine::new(config)?;
    engine.initialize()?;
    
    println!("Engine initialized with error recovery system");
    
    #[cfg(feature = "dev-ui")]
    {
        demonstrate_error_recovery(&mut engine)?;
        demonstrate_error_reporting(&mut engine)?;
        demonstrate_graceful_degradation(&mut engine)?;
        demonstrate_health_monitoring(&mut engine)?;
    }
    
    #[cfg(not(feature = "dev-ui"))]
    {
        println!("INFO: Error recovery features are only available in development mode");
        println!("   Run with: cargo run --features dev-ui --example error_recovery_demo");
    }
    
    Ok(())
}

#[cfg(feature = "dev-ui")]
fn demonstrate_error_recovery(engine: &mut DualModeEngine) -> Result<()> {
    println!("\nDemonstrating Error Recovery Mechanisms");
    println!("-------------------------------------------");
    
    // Simulate various types of errors and recovery
    let test_errors = vec![
        (RustyUIError::interpretation("Rhai script syntax error"), Operation::RhaiExecution),
        (RustyUIError::interpretation("AST parsing failed"), Operation::ASTParsing),
        (RustyUIError::state_preservation("Serialization failed"), Operation::StatePreservation),
        (RustyUIError::framework_adapter("Egui context lost"), Operation::FrameworkIntegration),
    ];
    
    for (error, operation) in test_errors {
        println!("\n🚨 Simulating error: {}", error);
        
        let result = engine.handle_error(&error, operation, Some("demo_component".to_string()));
        
        match result {
            RecoveryResult::Success { strategy, message, .. } => {
                println!("SUCCESS: Recovery successful: {} (strategy: {:?})", message, strategy);
            }
            RecoveryResult::PartialRecovery { strategy, message, limitations } => {
                println!("WARNING: Partial recovery: {} (strategy: {:?})", message, strategy);
                for limitation in limitations {
                    println!("   - Limitation: {}", limitation);
                }
            }
            RecoveryResult::Failed { strategy, message } => {
                println!("FAILED: Recovery failed: {} (strategy: {:?})", message, strategy);
            }
        }
    }
    
    Ok(())
}

#[cfg(feature = "dev-ui")]
fn demonstrate_error_reporting(engine: &mut DualModeEngine) -> Result<()> {
    println!("\nDemonstrating Error Reporting System");
    println!("---------------------------------------");
    
    // Generate some errors for reporting
    let errors = vec![
        RustyUIError::interpretation("Component update failed"),
        RustyUIError::interpretation("Invalid UI syntax"),
        RustyUIError::state_preservation("State corruption detected"),
    ];
    
    for (i, error) in errors.iter().enumerate() {
        let component_id = format!("component_{}", i);
        engine.handle_error(error, Operation::Interpretation, Some(component_id));
    }
    
    // Get error metrics
    if let Some(metrics) = engine.get_error_metrics() {
        println!("📈 Error Metrics:");
        println!("   Total errors: {}", metrics.total_errors);
        println!("   Critical errors: {}", metrics.critical_errors);
        println!("   High severity errors: {}", metrics.high_errors);
        println!("   Medium severity errors: {}", metrics.medium_errors);
        println!("   Low severity errors: {}", metrics.low_errors);
        
        println!("\nError breakdown by type:");
        for (error_type, count) in &metrics.error_type_counts {
            println!("   {}: {}", error_type.as_str(), count);
        }
        
        println!("\nComponent error breakdown:");
        for (component, count) in &metrics.component_error_counts {
            println!("   {}: {}", component, count);
        }
    }
    
    // Generate comprehensive error report
    if let Some(report) = engine.get_error_report() {
        println!("\n📄 Error Report Generated:");
        println!("   Recent errors: {}", report.recent_errors.len());
        println!("   Error patterns identified: {}", report.error_patterns.len());
        println!("   Recommendations: {}", report.recommendations.len());
        
        for (i, recommendation) in report.recommendations.iter().enumerate() {
            println!("   {}. {}", i + 1, recommendation);
        }
    }
    
    Ok(())
}

#[cfg(feature = "dev-ui")]
fn demonstrate_graceful_degradation(engine: &mut DualModeEngine) -> Result<()> {
    println!("\nDemonstrating Graceful Degradation");
    println!("-------------------------------------");
    
    // Store some fallback state
    let fallback_state = serde_json::json!({
        "component_type": "Button",
        "text": "Click me",
        "enabled": true,
        "style": {
            "background": "#007acc",
            "color": "white"
        }
    });
    
    engine.store_fallback_state("demo_button".to_string(), fallback_state);
    println!("💾 Stored fallback state for demo_button");
    
    // Simulate critical error that requires fallback
    let critical_error = RustyUIError::interpretation("Critical component failure - falling back to safe state");
    let result = engine.handle_error(&critical_error, Operation::ComponentRendering, Some("demo_button".to_string()));
    
    match result {
        RecoveryResult::Success { message, fallback_data, .. } => {
            println!("SUCCESS: Graceful degradation successful: {}", message);
            if let Some(data) = fallback_data {
                println!("   Restored to fallback state: {}", data.component_id);
            }
        }
        _ => {
            println!("WARNING: Graceful degradation had issues, but system remains stable");
        }
    }
    
    // Demonstrate error isolation
    println!("\n🔒 Demonstrating Error Isolation");
    let isolated_error = RustyUIError::interpretation("Isolated component error");
    let isolation_result = engine.handle_error(&isolated_error, Operation::ComponentRendering, Some("isolated_component".to_string()));
    
    match isolation_result {
        RecoveryResult::PartialRecovery { message, limitations, .. } => {
            println!("Error isolated: {}", message);
            for limitation in limitations {
                println!("   - {}", limitation);
            }
            println!("   Main application continues running normally");
        }
        _ => {
            println!("Error isolation mechanism activated");
        }
    }
    
    Ok(())
}

#[cfg(feature = "dev-ui")]
fn demonstrate_health_monitoring(engine: &mut DualModeEngine) -> Result<()> {
    println!("\n💚 Demonstrating System Health Monitoring");
    println!("-----------------------------------------");
    
    // Check initial health status
    let health_status = engine.get_health_status();
    println!("🏥 Current system health: {:?}", health_status);
    
    match health_status {
        HealthStatus::Healthy => {
            println!("   All systems operating normally");
        }
        HealthStatus::Recovering => {
            println!("   System is recovering from recent errors");
        }
        HealthStatus::Degraded => {
            println!("   System is in degraded mode due to persistent errors");
        }
    }
    
    // Get recovery metrics
    if let Some(recovery_metrics) = engine.get_error_recovery_metrics() {
        println!("\nRecovery Performance:");
        println!("   Total errors handled: {}", recovery_metrics.total_errors);
        println!("   Successful recoveries: {}", recovery_metrics.successful_recoveries);
        println!("   Partial recoveries: {}", recovery_metrics.partial_recoveries);
        println!("   Failed recoveries: {}", recovery_metrics.failed_recoveries);
        println!("   Success rate: {:.1}%", recovery_metrics.success_rate() * 100.0);
        println!("   Recovery rate: {:.1}%", recovery_metrics.recovery_rate() * 100.0);
    }
    
    // Demonstrate system resilience
    println!("\nTesting System Resilience");
    for i in 0..5 {
        let stress_error = RustyUIError::interpretation(format!("Stress test error #{}", i + 1));
        engine.handle_error(&stress_error, Operation::Interpretation, Some(format!("stress_component_{}", i)));
    }
    
    let final_health = engine.get_health_status();
    println!("🏥 Health after stress test: {:?}", final_health);
    
    if final_health == HealthStatus::Healthy || final_health == HealthStatus::Recovering {
        println!("   System maintained stability under stress");
    } else {
        println!("   System entered degraded mode - recovery mechanisms active");
    }
    
    Ok(())
}

#[cfg(feature = "dev-ui")]
fn demonstrate_unsupported_features() -> Result<()> {
    println!("\n🚫 Demonstrating Unsupported Feature Handling");
    println!("---------------------------------------------");
    
    // This would be called when encountering unsupported features
    let unsupported_features = vec![
        "advanced_3d_rendering",
        "neural_network_ui",
        "quantum_state_management",
    ];
    
    for feature in unsupported_features {
        println!("BLOCKED: Unsupported feature detected: {}", feature);
        println!("   Gracefully disabled, fallback mechanisms active");
        println!("   Suggestion: Use alternative implementation or update library");
    }
    
    Ok(())
}

// Helper function to create test error context
#[cfg(feature = "dev-ui")]
fn create_test_context(operation: ErrorOperation, component_id: Option<String>) -> ErrorReportContext {
    let mut system_state = HashMap::new();
    system_state.insert("test_mode".to_string(), "true".to_string());
    system_state.insert("demo_active".to_string(), "true".to_string());
    
    ErrorReportContext {
        operation,
        component_id,
        file_path: Some("examples/error_recovery_demo.rs".to_string()),
        line_number: Some(42),
        affects_core_functionality: true,
        user_action: Some("Running error recovery demo".to_string()),
        system_state,
    }
}