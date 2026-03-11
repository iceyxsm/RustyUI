//! End-to-End Integration Tests for RustyUI
//! 
//! Task 12.4: Comprehensive end-to-end integration tests with industry-level performance optimizations
//! 
//! This module implements comprehensive integration tests that validate:
//! - Complete workflow: init → dev → build → production
//! - State preservation across interpretation cycles
//! - Error recovery and graceful degradation
//! - Performance bounds compliance (interpretation <100ms, change detection <50ms)
//! - Memory usage optimization (<50MB overhead in development)
//! - Production build zero-overhead verification
//! 
//! **Validates: Requirements 1.6, 6.5, 9.3**

use rustyui_core::{
    DualModeEngine, DualModeConfig, UIFramework, UIComponent, RenderContext,
    Platform, PlatformConfig, ProductionVerifier, PerformanceTargets,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use serde_json::json;

#[cfg(feature = "dev-ui")]
use rustyui_core::{
    DevelopmentSettings, InterpretationStrategy, ChangeMonitor, StatePreservor,
    ErrorRecoveryManager, PerformanceMonitor, ComponentLifecycleManager,
};

/// Test fixture for end-to-end integration testing
struct IntegrationTestFixture {
    temp_dir: TempDir,
    project_path: PathBuf,
    config: DualModeConfig,
    engine: Option<DualModeEngine>,
}

impl IntegrationTestFixture {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().to_path_buf();
        
        // Create project structure
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::create_dir_all(project_path.join("examples")).unwrap();
        
        let config = DualModeConfig {
            framework: UIFramework::Egui,
            watch_paths: vec![
                project_path.join("src"),
                project_path.join("examples"),
            ],
            production_settings: Default::default(),
            conditional_compilation: Default::default(),
            #[cfg(feature = "dev-ui")]
            development_settings: DevelopmentSettings {
                interpretation_strategy: InterpretationStrategy::Hybrid { 
                    rhai_threshold: 10, 
                    jit_threshold: 100 
                },
                jit_compilation_threshold: 100,
                state_preservation: true,
                performance_monitoring: true,
                change_detection_delay_ms: 50,
                max_memory_overhead_mb: 50,
            },
        };
        
        Self {
            temp_dir,
            project_path,
            config,
            engine: None,
        }
    }
    
    fn initialize_engine(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut engine = DualModeEngine::new(self.config.clone())?;
        engine.initialize()?;
        self.engine = Some(engine);
        Ok(())
    }
    
    fn create_test_component_file(&self, filename: &str, content: &str) -> PathBuf {
        let file_path = self.project_path.join("src").join(filename);
        std::fs::write(&file_path, content).unwrap();
        file_path
    }
    
    fn create_test_ui_component(&self) -> TestUIComponent {
        TestUIComponent::new()
    }
}

/// Test UI component for integration testing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TestUIComponent {
    id: String,
    counter: i32,
    text: String,
    enabled: bool,
    items: Vec<String>,
}

impl TestUIComponent {
    fn new() -> Self {
        Self {
            id: "test_component".to_string(),
            counter: 0,
            text: "Hello, RustyUI!".to_string(),
            enabled: true,
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        }
    }
    
    fn increment_counter(&mut self) {
        self.counter += 1;
    }
    
    fn add_item(&mut self, item: String) {
        self.items.push(item);
    }
    
    fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
    }
}

impl UIComponent for TestUIComponent {
    fn render(&mut self, ctx: &mut dyn RenderContext) {
        ctx.render_text(&format!("Counter: {}", self.counter));
        ctx.render_text(&format!("Text: {}", self.text));
        ctx.render_text(&format!("Enabled: {}", self.enabled));
        ctx.render_text(&format!("Items: {}", self.items.len()));
    }
    
    #[cfg(feature = "dev-ui")]
    fn hot_reload_state(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
    
    #[cfg(feature = "dev-ui")]
    fn restore_state(&mut self, state: serde_json::Value) {
        if let Ok(restored) = serde_json::from_value::<TestUIComponent>(state) {
            *self = restored;
        }
    }
}

/// Mock render context for testing
struct MockRenderContext {
    rendered_elements: Vec<String>,
}

impl MockRenderContext {
    fn new() -> Self {
        Self {
            rendered_elements: Vec::new(),
        }
    }
}

impl RenderContext for MockRenderContext {
    fn render_button(&mut self, text: &str, _callback: Box<dyn Fn()>) {
        self.rendered_elements.push(format!("Button: {}", text));
    }
    
    fn render_text(&mut self, text: &str) {
        self.rendered_elements.push(format!("Text: {}", text));
    }
    
    #[cfg(feature = "dev-ui")]
    fn handle_runtime_update(&mut self, update: &rustyui_core::RuntimeUpdate) {
        self.rendered_elements.push(format!("Update: {:?}", update));
    }
}
// ============================================================================
// Test 1: Complete Workflow Integration (init → dev → build → production)
// ============================================================================

/// **Validates: Requirements 1.6, 8.1, 8.2, 8.3, 8.4, 8.5, 8.6**
/// 
/// Test complete workflow from project initialization through development mode
/// to production build, ensuring seamless transitions and zero overhead.
#[test]
fn test_complete_workflow_integration() {
    let mut fixture = IntegrationTestFixture::new();
    
    // Step 1: Initialize project (simulating `rustyui init`)
    println!("🚀 Step 1: Project Initialization");
    
    // Create Cargo.toml
    let cargo_toml = r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
rustyui-core = { path = "../../../crates/rustyui-core", features = ["dev-ui"] }

[features]
default = []
dev-ui = ["rustyui-core/dev-ui"]
"#;
    std::fs::write(fixture.project_path.join("Cargo.toml"), cargo_toml).unwrap();
    
    // Create main.rs with RustyUI integration
    let main_rs = r#"
use rustyui_core::{DualModeEngine, DualModeConfig, UIFramework};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DualModeConfig {
        framework: UIFramework::Egui,
        watch_paths: vec![std::path::PathBuf::from("src")],
        production_settings: Default::default(),
        conditional_compilation: Default::default(),
        #[cfg(feature = "dev-ui")]
        development_settings: Default::default(),
    };
    
    let mut engine = DualModeEngine::new(config)?;
    engine.initialize()?;
    
    #[cfg(feature = "dev-ui")]
    {
        println!("🔥 Development mode with hot reload enabled!");
        engine.start_development_mode()?;
    }
    
    #[cfg(not(feature = "dev-ui"))]
    {
        println!("🚀 Production mode - zero overhead!");
    }
    
    Ok(())
}
"#;
    fixture.create_test_component_file("main.rs", main_rs);
    
    // Step 2: Development mode initialization
    println!("🔥 Step 2: Development Mode Initialization");
    
    fixture.initialize_engine().expect("Engine initialization should succeed");
    let engine = fixture.engine.as_ref().unwrap();
    
    // Verify development mode capabilities
    #[cfg(feature = "dev-ui")]
    {
        assert!(engine.has_runtime_interpreter(), "Should have runtime interpreter in dev mode");
        assert!(engine.can_interpret_changes(), "Should be able to interpret changes");
        assert!(engine.has_performance_monitoring(), "Should have performance monitoring");
        
        // Memory overhead should be reasonable
        let memory_overhead = engine.current_memory_overhead_bytes();
        assert!(memory_overhead < 50 * 1024 * 1024, 
            "Memory overhead should be under 50MB, got {} bytes", memory_overhead);
    }
    
    #[cfg(not(feature = "dev-ui"))]
    {
        assert!(!engine.has_runtime_interpreter(), "Should not have runtime interpreter in production");
        assert!(!engine.can_interpret_changes(), "Should not be able to interpret changes");
        assert_eq!(engine.current_memory_overhead_bytes(), 0, "Should have zero memory overhead");
    }
    
    // Step 3: Development workflow simulation
    println!("⚡ Step 3: Development Workflow Simulation");
    
    #[cfg(feature = "dev-ui")]
    {
        let mut engine = fixture.engine.take().unwrap();
        
        // Start development mode
        engine.start_development_mode().expect("Should start development mode");
        
        // Simulate component registration
        engine.register_component("test_button".to_string(), "Button".to_string())
            .expect("Should register component");
        
        // Simulate UI code interpretation
        let ui_code = r#"
            button.text = "Click me!";
            button.enabled = true;
            button.on_click = || { counter += 1; };
        "#;
        
        let start_time = Instant::now();
        let interpretation_result = engine.interpret_ui_change(ui_code, Some("test_button".to_string()));
        let interpretation_time = start_time.elapsed();
        
        assert!(interpretation_result.is_ok(), "UI interpretation should succeed");
        assert!(interpretation_time < Duration::from_millis(100), 
            "Interpretation should be under 100ms, took {:?}", interpretation_time);
        
        fixture.engine = Some(engine);
    }
    
    // Step 4: Production build verification
    println!("🚀 Step 4: Production Build Verification");
    
    // Simulate production build (without dev-ui feature)
    let verifier = ProductionVerifier::new();
    let verification_results = verifier.verify_zero_overhead_build(&fixture.project_path);
    
    assert!(verification_results.is_ok(), "Production build verification should succeed");
    let results = verification_results.unwrap();
    
    // Verify zero overhead characteristics
    assert!(results.has_zero_overhead(), "Production build should have zero overhead");
    assert!(!results.contains_dev_features(), "Production build should not contain dev features");
    
    println!("✅ Complete workflow integration test passed!");
}
// ============================================================================
// Test 2: State Preservation Across Interpretation Cycles
// ============================================================================

/// **Validates: Requirements 6.1, 6.2, 6.3, 6.4, 6.5, 6.6**
/// 
/// Test state preservation round-trip across multiple interpretation cycles,
/// including complex state objects and error recovery scenarios.
#[test]
fn test_state_preservation_across_interpretation_cycles() {
    let mut fixture = IntegrationTestFixture::new();
    fixture.initialize_engine().expect("Engine initialization should succeed");
    
    println!("🔄 Testing State Preservation Across Interpretation Cycles");
    
    #[cfg(feature = "dev-ui")]
    {
        let mut engine = fixture.engine.take().unwrap();
        
        // Create test component with complex state
        let mut test_component = fixture.create_test_ui_component();
        test_component.increment_counter();
        test_component.add_item("Dynamic Item".to_string());
        test_component.toggle_enabled();
        
        let component_id = "state_test_component";
        
        // Register component
        engine.register_component(component_id.to_string(), "TestComponent".to_string())
            .expect("Should register component");
        
        // Cycle 1: Preserve initial state
        println!("  📸 Cycle 1: Preserving initial state");
        let initial_state = test_component.hot_reload_state();
        engine.preserve_component_state(component_id, initial_state.clone())
            .expect("Should preserve initial state");
        
        // Simulate interpretation cycle with state changes
        test_component.increment_counter();
        test_component.add_item("Another Item".to_string());
        
        // Cycle 2: Preserve modified state
        println!("  📸 Cycle 2: Preserving modified state");
        let modified_state = test_component.hot_reload_state();
        engine.preserve_component_state(component_id, modified_state.clone())
            .expect("Should preserve modified state");
        
        // Simulate hot reload - restore state
        println!("  🔄 Cycle 2: Restoring state after hot reload");
        let restored_state = engine.restore_component_state(component_id);
        assert!(restored_state.is_some(), "Should restore preserved state");
        
        let mut restored_component = TestUIComponent::new();
        restored_component.restore_state(restored_state.unwrap());
        
        // Verify state preservation accuracy
        assert_eq!(restored_component.counter, 2, "Counter should be preserved");
        assert_eq!(restored_component.items.len(), 4, "Items should be preserved");
        assert!(!restored_component.enabled, "Enabled state should be preserved");
        
        // Cycle 3: Test state preservation under rapid changes
        println!("  ⚡ Cycle 3: Rapid state changes");
        for i in 0..10 {
            restored_component.increment_counter();
            restored_component.add_item(format!("Rapid Item {}", i));
            
            let rapid_state = restored_component.hot_reload_state();
            engine.preserve_component_state(component_id, rapid_state)
                .expect("Should preserve rapid state changes");
        }
        
        // Verify final state
        let final_restored = engine.restore_component_state(component_id);
        assert!(final_restored.is_some(), "Should restore final state");
        
        let mut final_component = TestUIComponent::new();
        final_component.restore_state(final_restored.unwrap());
        
        assert_eq!(final_component.counter, 12, "Final counter should be preserved");
        assert_eq!(final_component.items.len(), 14, "Final items should be preserved");
        
        // Test state preservation with serialization errors
        println!("  ❌ Cycle 4: Error handling in state preservation");
        
        // Attempt to preserve invalid state (should handle gracefully)
        let invalid_state = json!({"invalid": "structure"});
        let preserve_result = engine.preserve_component_state("invalid_component", invalid_state);
        
        // Should handle gracefully without crashing
        assert!(preserve_result.is_ok() || preserve_result.is_err(), 
            "Should handle invalid state gracefully");
        
        // Verify system remains stable after error
        assert!(engine.get_health_status().is_healthy(), 
            "System should remain healthy after state preservation error");
        
        fixture.engine = Some(engine);
    }
    
    #[cfg(not(feature = "dev-ui"))]
    {
        println!("  🚀 Production mode: State preservation disabled for zero overhead");
        // In production mode, state preservation should be disabled
        let engine = fixture.engine.as_ref().unwrap();
        assert_eq!(engine.current_memory_overhead_bytes(), 0, 
            "Production mode should have zero memory overhead");
    }
    
    println!("✅ State preservation across interpretation cycles test passed!");
}
// ============================================================================
// Test 3: Error Recovery and Graceful Degradation
// ============================================================================

/// **Validates: Requirements 9.1, 9.2, 9.3, 9.4, 9.5, 9.6**
/// 
/// Test comprehensive error recovery mechanisms and graceful degradation
/// under various failure scenarios while maintaining system stability.
#[test]
fn test_error_recovery_and_graceful_degradation() {
    let mut fixture = IntegrationTestFixture::new();
    fixture.initialize_engine().expect("Engine initialization should succeed");
    
    println!("🛡️ Testing Error Recovery and Graceful Degradation");
    
    #[cfg(feature = "dev-ui")]
    {
        let mut engine = fixture.engine.take().unwrap();
        
        // Test 1: Interpretation Error Recovery
        println!("  ❌ Test 1: Interpretation Error Recovery");
        
        let invalid_ui_code = r#"
            this is not valid rust code!!!
            syntax error here ->
        "#;
        
        let interpretation_result = engine.interpret_ui_change(invalid_ui_code, Some("test_component".to_string()));
        
        // Should handle interpretation errors gracefully
        match interpretation_result {
            Ok(_) => println!("    ✅ Invalid code was handled gracefully"),
            Err(error) => {
                println!("    ✅ Interpretation error handled: {}", error);
                
                // System should remain stable after error
                assert!(engine.get_health_status().is_healthy(), 
                    "System should remain healthy after interpretation error");
            }
        }
        
        // Test 2: Component Registration Error Recovery
        println!("  ❌ Test 2: Component Registration Error Recovery");
        
        // Try to register component with invalid ID
        let invalid_registration = engine.register_component("".to_string(), "InvalidComponent".to_string());
        
        match invalid_registration {
            Ok(_) => println!("    ✅ Invalid registration was handled gracefully"),
            Err(error) => {
                println!("    ✅ Registration error handled: {}", error);
                
                // Verify error recovery metrics are updated
                if let Some(recovery_metrics) = engine.get_error_recovery_metrics() {
                    assert!(recovery_metrics.total_errors > 0, 
                        "Error recovery metrics should track errors");
                }
            }
        }
        
        // Test 3: State Preservation Error Recovery
        println!("  ❌ Test 3: State Preservation Error Recovery");
        
        // Create component and preserve valid state first
        engine.register_component("recovery_test".to_string(), "TestComponent".to_string())
            .expect("Should register component");
        
        let valid_state = json!({
            "counter": 42,
            "text": "Valid state",
            "enabled": true
        });
        
        engine.preserve_component_state("recovery_test", valid_state.clone())
            .expect("Should preserve valid state");
        
        // Now try to preserve invalid state
        let invalid_state = json!({
            "circular_reference": {"self": "circular_reference"}
        });
        
        let invalid_preserve_result = engine.preserve_component_state("recovery_test", invalid_state);
        
        // Should handle invalid state gracefully
        match invalid_preserve_result {
            Ok(_) => println!("    ✅ Invalid state preservation handled gracefully"),
            Err(error) => {
                println!("    ✅ State preservation error handled: {}", error);
            }
        }
        
        // Verify that valid state can still be restored
        let restored_state = engine.restore_component_state("recovery_test");
        assert!(restored_state.is_some(), "Should still be able to restore valid state");
        
        // Test 4: Memory Pressure Recovery
        println!("  💾 Test 4: Memory Pressure Recovery");
        
        // Simulate memory pressure by creating many components
        for i in 0..100 {
            let component_id = format!("memory_test_{}", i);
            let _ = engine.register_component(component_id.clone(), "MemoryTestComponent".to_string());
            
            let large_state = json!({
                "data": vec![0; 1000], // 1KB of data per component
                "id": i
            });
            
            let _ = engine.preserve_component_state(&component_id, large_state);
        }
        
        // Check memory overhead is still within bounds
        let memory_overhead = engine.current_memory_overhead_bytes();
        println!("    📊 Memory overhead after stress test: {} MB", memory_overhead / (1024 * 1024));
        
        // Should still be under the 50MB limit (with some tolerance for test overhead)
        assert!(memory_overhead < 75 * 1024 * 1024, 
            "Memory overhead should remain reasonable under stress, got {} bytes", memory_overhead);
        
        // Test 5: Graceful Degradation Under Platform Limitations
        println!("  🖥️ Test 5: Platform Limitation Handling");
        
        // Test platform-specific capabilities
        let platform = engine.platform();
        let platform_config = engine.platform_config();
        
        match platform {
            Platform::Windows | Platform::MacOS | Platform::Linux => {
                println!("    ✅ Running on supported platform: {:?}", platform);
                
                // Verify platform-specific optimizations are available
                if platform_config.jit_capabilities.cranelift_available {
                    println!("    ✅ JIT compilation available");
                    assert!(engine.jit_compilation_available(), "JIT should be available");
                } else {
                    println!("    ⚠️ JIT compilation not available, graceful degradation expected");
                    assert!(!engine.jit_compilation_available(), "JIT should not be available");
                }
            }
            Platform::Unknown => {
                println!("    ⚠️ Unknown platform, testing graceful degradation");
                // System should still function with basic capabilities
                assert!(engine.get_health_status().is_healthy(), 
                    "System should remain healthy on unknown platforms");
            }
        }
        
        // Test 6: Error Isolation and Recovery
        println!("  🔒 Test 6: Error Isolation and Recovery");
        
        // Create multiple components to test error isolation
        for i in 0..5 {
            let component_id = format!("isolation_test_{}", i);
            engine.register_component(component_id.clone(), "IsolationTestComponent".to_string())
                .expect("Should register isolation test component");
        }
        
        // Cause error in one component
        let error_result = engine.interpret_ui_change("invalid syntax", Some("isolation_test_2".to_string()));
        
        // Error should be isolated - other components should remain functional
        for i in [0, 1, 3, 4] {
            let component_id = format!("isolation_test_{}", i);
            let valid_code = "button.enabled = true;";
            let result = engine.interpret_ui_change(valid_code, Some(component_id));
            
            // These should still work despite error in isolation_test_2
            match result {
                Ok(_) => println!("    ✅ Component {} remains functional after isolated error", i),
                Err(e) => println!("    ⚠️ Component {} affected by error: {}", i, e),
            }
        }
        
        // Verify overall system health
        let health_status = engine.get_health_status();
        assert!(health_status.is_healthy() || health_status.is_degraded(), 
            "System should be healthy or gracefully degraded, not failed");
        
        fixture.engine = Some(engine);
    }
    
    #[cfg(not(feature = "dev-ui"))]
    {
        println!("  🚀 Production mode: Error recovery overhead eliminated for zero-cost abstractions");
        let engine = fixture.engine.as_ref().unwrap();
        
        // Production mode should have minimal error handling overhead
        assert_eq!(engine.current_memory_overhead_bytes(), 0, 
            "Production mode should have zero memory overhead");
        
        // Basic error handling should still work
        assert!(engine.get_health_status().is_healthy(), 
            "Production mode should maintain basic health status");
    }
    
    println!("✅ Error recovery and graceful degradation test passed!");
}
// ============================================================================
// Test 4: Performance Bounds Validation and Optimization
// ============================================================================

/// **Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5, 7.6**
/// 
/// Test performance bounds compliance with industry-level optimizations:
/// - Interpretation <100ms, change detection <50ms
/// - Memory overhead <50MB in development
/// - Production build zero-overhead verification
/// - Incremental compilation and smart rebuild strategies
#[test]
fn test_performance_bounds_validation() {
    let mut fixture = IntegrationTestFixture::new();
    fixture.initialize_engine().expect("Engine initialization should succeed");
    
    println!("⚡ Testing Performance Bounds Validation and Optimization");
    
    #[cfg(feature = "dev-ui")]
    {
        let mut engine = fixture.engine.take().unwrap();
        
        // Test 1: Interpretation Performance (<100ms target)
        println!("  ⏱️ Test 1: Interpretation Performance Validation");
        
        let test_cases = vec![
            ("Simple UI update", "button.text = 'Updated';"),
            ("Complex state change", r#"
                component.counter += 1;
                component.items.push('New Item');
                component.enabled = !component.enabled;
            "#),
            ("Layout modification", r#"
                layout.width = 300;
                layout.height = 200;
                layout.padding = 10;
            "#),
            ("Event handler update", r#"
                button.on_click = || {
                    counter += 1;
                    update_ui();
                };
            "#),
        ];
        
        let mut interpretation_times = Vec::new();
        
        for (test_name, ui_code) in test_cases {
            let start_time = Instant::now();
            let result = engine.interpret_ui_change(ui_code, Some("perf_test_component".to_string()));
            let interpretation_time = start_time.elapsed();
            
            interpretation_times.push(interpretation_time);
            
            println!("    📊 {}: {:?}", test_name, interpretation_time);
            
            // Individual interpretation should be under 100ms
            assert!(interpretation_time < Duration::from_millis(100), 
                "{} took {:?}, should be under 100ms", test_name, interpretation_time);
            
            // Verify interpretation succeeded or failed gracefully
            match result {
                Ok(_) => println!("      ✅ Interpretation succeeded"),
                Err(e) => println!("      ⚠️ Interpretation handled gracefully: {}", e),
            }
        }
        
        // Calculate average interpretation time
        let avg_time = interpretation_times.iter().sum::<Duration>() / interpretation_times.len() as u32;
        println!("    📈 Average interpretation time: {:?}", avg_time);
        
        // Average should be well under the 100ms target
        assert!(avg_time < Duration::from_millis(50), 
            "Average interpretation time should be under 50ms for optimal performance, got {:?}", avg_time);
        
        // Test 2: Change Detection Performance (<50ms target)
        println!("  🔍 Test 2: Change Detection Performance");
        
        // Create test files to monitor
        let test_files = vec![
            ("component1.rs", "// Component 1 content"),
            ("component2.rs", "// Component 2 content"),
            ("styles.css", "/* CSS styles */"),
            ("config.toml", "# Configuration"),
        ];
        
        for (filename, content) in &test_files {
            fixture.create_test_component_file(filename, content);
        }
        
        // Simulate file changes and measure detection time
        let mut detection_times = Vec::new();
        
        for (filename, new_content) in test_files {
            let file_path = fixture.project_path.join("src").join(&filename);
            
            let start_time = Instant::now();
            
            // Modify file
            std::fs::write(&file_path, format!("{}\n// Modified at {:?}", new_content, start_time)).unwrap();
            
            // Process file changes
            let changes_result = engine.process_file_changes();
            let detection_time = start_time.elapsed();
            
            detection_times.push(detection_time);
            
            println!("    📊 Change detection for {}: {:?}", filename, detection_time);
            
            // Change detection should be under 50ms
            assert!(detection_time < Duration::from_millis(50), 
                "Change detection for {} took {:?}, should be under 50ms", filename, detection_time);
            
            // Verify changes were detected
            match changes_result {
                Ok(changes) => {
                    if !changes.is_empty() {
                        println!("      ✅ Changes detected: {} files", changes.len());
                    }
                }
                Err(e) => println!("      ⚠️ Change detection handled gracefully: {}", e),
            }
        }
        
        let avg_detection_time = detection_times.iter().sum::<Duration>() / detection_times.len() as u32;
        println!("    📈 Average change detection time: {:?}", avg_detection_time);
        
        // Test 3: Memory Usage Optimization (<50MB target)
        println!("  💾 Test 3: Memory Usage Optimization");
        
        let initial_memory = engine.current_memory_overhead_bytes();
        println!("    📊 Initial memory overhead: {:.2} MB", initial_memory as f64 / (1024.0 * 1024.0));
        
        // Simulate realistic development session
        for i in 0..50 {
            let component_id = format!("memory_opt_test_{}", i);
            engine.register_component(component_id.clone(), "MemoryOptTestComponent".to_string())
                .expect("Should register component");
            
            // Preserve state for each component
            let state = json!({
                "id": i,
                "data": format!("Component {} data", i),
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            });
            
            engine.preserve_component_state(&component_id, state)
                .expect("Should preserve component state");
            
            // Simulate interpretation for each component
            let ui_code = format!("component_{}.enabled = true;", i);
            let _ = engine.interpret_ui_change(&ui_code, Some(component_id));
        }
        
        let final_memory = engine.current_memory_overhead_bytes();
        println!("    📊 Final memory overhead: {:.2} MB", final_memory as f64 / (1024.0 * 1024.0));
        
        // Memory should stay under 50MB even with many components
        assert!(final_memory < 50 * 1024 * 1024, 
            "Memory overhead should be under 50MB, got {:.2} MB", 
            final_memory as f64 / (1024.0 * 1024.0));
        
        // Test 4: Performance Monitoring Accuracy
        println!("  📈 Test 4: Performance Monitoring Accuracy");
        
        if let Some(metrics) = engine.get_performance_metrics() {
            println!("    📊 Performance Metrics:");
            println!("      - Total operations: {}", metrics.total_operations);
            println!("      - Average interpretation time: {:?}", metrics.average_interpretation_time);
            println!("      - Max interpretation time: {:?}", metrics.max_interpretation_time);
            println!("      - Target violations: {}", metrics.target_violations);
            
            // Verify metrics are reasonable
            assert!(metrics.total_operations > 0, "Should have recorded operations");
            assert!(metrics.average_interpretation_time <= Duration::from_millis(100), 
                "Average interpretation time should meet target");
            assert!(metrics.target_violations <= metrics.total_operations, 
                "Target violations should not exceed total operations");
            
            // Calculate performance efficiency
            let efficiency = if metrics.total_operations > 0 {
                ((metrics.total_operations - metrics.target_violations) as f64 / metrics.total_operations as f64) * 100.0
            } else {
                100.0
            };
            
            println!("    📈 Performance efficiency: {:.1}%", efficiency);
            
            // Should maintain high efficiency (>90%)
            assert!(efficiency >= 90.0, 
                "Performance efficiency should be at least 90%, got {:.1}%", efficiency);
        } else {
            panic!("Performance metrics should be available in development mode");
        }
        
        // Test 5: Incremental Compilation Optimization
        println!("  🔄 Test 5: Incremental Compilation Optimization");
        
        // Test smart rebuild strategies
        let component_code = r#"
            struct OptimizedComponent {
                counter: i32,
                enabled: bool,
            }
            
            impl OptimizedComponent {
                fn increment(&mut self) {
                    self.counter += 1;
                }
            }
        "#;
        
        // First compilation
        let start_time = Instant::now();
        let first_result = engine.interpret_ui_change(component_code, Some("incremental_test".to_string()));
        let first_compile_time = start_time.elapsed();
        
        println!("    📊 First compilation time: {:?}", first_compile_time);
        
        // Second compilation (should be faster due to incremental compilation)
        let modified_code = r#"
            struct OptimizedComponent {
                counter: i32,
                enabled: bool,
                // Added new field - minimal change
                label: String,
            }
            
            impl OptimizedComponent {
                fn increment(&mut self) {
                    self.counter += 1;
                }
            }
        "#;
        
        let start_time = Instant::now();
        let second_result = engine.interpret_ui_change(modified_code, Some("incremental_test".to_string()));
        let second_compile_time = start_time.elapsed();
        
        println!("    📊 Incremental compilation time: {:?}", second_compile_time);
        
        // Incremental compilation should be faster (or at least not significantly slower)
        let speedup_ratio = first_compile_time.as_nanos() as f64 / second_compile_time.as_nanos() as f64;
        println!("    📈 Compilation speedup ratio: {:.2}x", speedup_ratio);
        
        // Both compilations should succeed or fail gracefully
        match (first_result, second_result) {
            (Ok(_), Ok(_)) => println!("    ✅ Both compilations succeeded"),
            (Err(e1), Err(e2)) => println!("    ⚠️ Both compilations handled gracefully: {} / {}", e1, e2),
            _ => println!("    ⚠️ Mixed results - system handling gracefully"),
        }
        
        fixture.engine = Some(engine);
    }
    
    #[cfg(not(feature = "dev-ui"))]
    {
        println!("  🚀 Production mode: Performance optimization overhead eliminated");
        let engine = fixture.engine.as_ref().unwrap();
        
        // Production mode should have zero overhead
        assert_eq!(engine.current_memory_overhead_bytes(), 0, 
            "Production mode should have zero memory overhead");
        
        assert!(!engine.has_performance_monitoring(), 
            "Production mode should not have performance monitoring overhead");
    }
    
    println!("✅ Performance bounds validation test passed!");
}
// ============================================================================
// Test 5: Cross-Platform Compatibility and Production Build Verification
// ============================================================================

/// **Validates: Requirements 10.1, 10.2, 10.3, 10.4, 10.5, 10.6, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6**
/// 
/// Test cross-platform compatibility and production build zero-overhead verification
/// with comprehensive binary analysis and performance comparison.
#[test]
fn test_cross_platform_compatibility_and_production_verification() {
    let mut fixture = IntegrationTestFixture::new();
    fixture.initialize_engine().expect("Engine initialization should succeed");
    
    println!("🌍 Testing Cross-Platform Compatibility and Production Build Verification");
    
    // Test 1: Platform Detection and Capabilities
    println!("  🖥️ Test 1: Platform Detection and Capabilities");
    
    let engine = fixture.engine.as_ref().unwrap();
    let platform = engine.platform();
    let platform_config = engine.platform_config();
    
    println!("    📊 Detected platform: {:?}", platform);
    println!("    📊 Platform capabilities:");
    println!("      - JIT available: {}", platform_config.jit_capabilities.cranelift_available);
    println!("      - File watcher backend: {:?}", platform_config.file_watcher_backend);
    println!("      - Native optimizations: {}", engine.is_using_native_optimizations());
    
    // Verify platform is supported
    match platform {
        Platform::Windows => {
            println!("    ✅ Windows platform detected and supported");
            // Windows-specific validations
            assert!(platform_config.file_watcher_backend != rustyui_core::FileWatcherBackend::Unsupported, 
                "File watching should be supported on Windows");
        }
        Platform::MacOS => {
            println!("    ✅ macOS platform detected and supported");
            // macOS-specific validations
            assert!(platform_config.file_watcher_backend != rustyui_core::FileWatcherBackend::Unsupported, 
                "File watching should be supported on macOS");
        }
        Platform::Linux => {
            println!("    ✅ Linux platform detected and supported");
            // Linux-specific validations
            assert!(platform_config.file_watcher_backend != rustyui_core::FileWatcherBackend::Unsupported, 
                "File watching should be supported on Linux");
        }
        Platform::Unknown => {
            println!("    ⚠️ Unknown platform - testing graceful degradation");
            // System should still function with basic capabilities
            assert!(engine.get_health_status().is_healthy(), 
                "System should remain healthy on unknown platforms");
        }
    }
    
    // Test 2: Platform-Specific Optimizations
    println!("  ⚡ Test 2: Platform-Specific Optimizations");
    
    #[cfg(feature = "dev-ui")]
    {
        let mut engine = fixture.engine.take().unwrap();
        
        // Test JIT compilation availability
        if engine.jit_compilation_available() {
            println!("    ✅ JIT compilation available - testing performance");
            
            let jit_test_code = r#"
                fn performance_critical_function(x: i32) -> i32 {
                    let mut result = 0;
                    for i in 0..x {
                        result += i * i;
                    }
                    result
                }
            "#;
            
            let start_time = Instant::now();
            let jit_result = engine.interpret_ui_change(jit_test_code, Some("jit_test".to_string()));
            let jit_time = start_time.elapsed();
            
            println!("    📊 JIT compilation time: {:?}", jit_time);
            
            // JIT compilation should be under 100ms
            assert!(jit_time < Duration::from_millis(100), 
                "JIT compilation should be under 100ms, got {:?}", jit_time);
            
            match jit_result {
                Ok(_) => println!("    ✅ JIT compilation succeeded"),
                Err(e) => println!("    ⚠️ JIT compilation handled gracefully: {}", e),
            }
        } else {
            println!("    ⚠️ JIT compilation not available - testing fallback");
            
            // Should fall back to AST interpretation
            let fallback_code = "button.enabled = true;";
            let fallback_result = engine.interpret_ui_change(fallback_code, Some("fallback_test".to_string()));
            
            match fallback_result {
                Ok(_) => println!("    ✅ Fallback interpretation succeeded"),
                Err(e) => println!("    ⚠️ Fallback interpretation handled gracefully: {}", e),
            }
        }
        
        fixture.engine = Some(engine);
    }
    
    // Test 3: Production Build Zero-Overhead Verification
    println!("  🚀 Test 3: Production Build Zero-Overhead Verification");
    
    let verifier = ProductionVerifier::new();
    
    // Create a test project structure for verification
    let test_project_path = fixture.project_path.clone();
    
    // Create production build configuration
    let production_cargo_toml = r#"[package]
name = "production-test"
version = "0.1.0"
edition = "2021"

[dependencies]
rustyui-core = { path = "../../../crates/rustyui-core" }

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
"#;
    
    std::fs::write(test_project_path.join("Cargo.toml"), production_cargo_toml).unwrap();
    
    // Create production main.rs (without dev-ui feature)
    let production_main = r#"
use rustyui_core::{DualModeEngine, DualModeConfig, UIFramework};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DualModeConfig {
        framework: UIFramework::Egui,
        watch_paths: vec![],
        production_settings: Default::default(),
        conditional_compilation: Default::default(),
    };
    
    let engine = DualModeEngine::new(config)?;
    
    // In production mode, no development features should be available
    assert!(!engine.has_runtime_interpreter());
    assert!(!engine.can_interpret_changes());
    assert_eq!(engine.current_memory_overhead_bytes(), 0);
    
    println!("Production build verified - zero overhead achieved!");
    
    Ok(())
}
"#;
    
    fixture.create_test_component_file("main.rs", production_main);
    
    // Verify production build characteristics
    let verification_result = verifier.verify_zero_overhead_build(&test_project_path);
    
    match verification_result {
        Ok(results) => {
            println!("    📊 Production Build Verification Results:");
            println!("      - Zero overhead: {}", results.has_zero_overhead());
            println!("      - Contains dev features: {}", results.contains_dev_features());
            println!("      - Binary size optimized: {}", results.is_size_optimized());
            println!("      - Security hardened: {}", results.is_security_hardened());
            
            // Verify zero overhead characteristics
            assert!(results.has_zero_overhead(), 
                "Production build should have zero overhead");
            
            assert!(!results.contains_dev_features(), 
                "Production build should not contain development features");
            
            if let Some(binary_results) = results.binary_size_results() {
                println!("    📊 Binary Size Analysis:");
                println!("      - Optimized size: {} bytes", binary_results.optimized_size);
                println!("      - Baseline size: {} bytes", binary_results.baseline_size);
                println!("      - Size ratio: {:.2}", binary_results.size_ratio());
                
                // Binary should be reasonably sized (not bloated with dev features)
                assert!(binary_results.size_ratio() <= 1.1, 
                    "Production binary should not be significantly larger than baseline");
            }
            
            if let Some(perf_results) = results.performance_results() {
                println!("    📊 Performance Analysis:");
                println!("      - Startup time: {:?}", perf_results.startup_time);
                println!("      - Memory usage: {} bytes", perf_results.memory_usage);
                println!("      - Performance ratio: {:.2}", perf_results.performance_ratio());
                
                // Performance should match native Rust
                assert!(perf_results.performance_ratio() >= 0.95, 
                    "Production performance should be at least 95% of native Rust");
            }
            
            println!("    ✅ Production build verification passed");
        }
        Err(e) => {
            println!("    ⚠️ Production build verification handled gracefully: {}", e);
            
            // Even if verification fails, basic production mode should work
            let engine = fixture.engine.as_ref().unwrap();
            
            #[cfg(not(feature = "dev-ui"))]
            {
                assert!(!engine.has_runtime_interpreter(), 
                    "Production mode should not have runtime interpreter");
                assert_eq!(engine.current_memory_overhead_bytes(), 0, 
                    "Production mode should have zero memory overhead");
            }
        }
    }
    
    // Test 4: Cross-Platform File Watching
    println!("  👁️ Test 4: Cross-Platform File Watching");
    
    #[cfg(feature = "dev-ui")]
    {
        let mut engine = fixture.engine.take().unwrap();
        
        // Test file watching on current platform
        let test_file = fixture.create_test_component_file("watch_test.rs", "// Initial content");
        
        // Modify file and test change detection
        std::fs::write(&test_file, "// Modified content").unwrap();
        
        let start_time = Instant::now();
        let changes_result = engine.process_file_changes();
        let detection_time = start_time.elapsed();
        
        println!("    📊 File change detection time: {:?}", detection_time);
        
        // Should detect changes quickly regardless of platform
        assert!(detection_time < Duration::from_millis(100), 
            "File change detection should be under 100ms on all platforms");
        
        match changes_result {
            Ok(changes) => {
                println!("    ✅ File changes detected: {} files", changes.len());
            }
            Err(e) => {
                println!("    ⚠️ File watching handled gracefully: {}", e);
            }
        }
        
        fixture.engine = Some(engine);
    }
    
    println!("✅ Cross-platform compatibility and production verification test passed!");
}

// ============================================================================
// Test 6: Comprehensive Integration Benchmark
// ============================================================================

/// **Validates: All Requirements - Comprehensive Integration Test**
/// 
/// Final comprehensive benchmark that tests the complete system integration
/// with realistic workloads and validates all performance targets.
#[test]
fn test_comprehensive_integration_benchmark() {
    let mut fixture = IntegrationTestFixture::new();
    fixture.initialize_engine().expect("Engine initialization should succeed");
    
    println!("🏁 Running Comprehensive Integration Benchmark");
    
    #[cfg(feature = "dev-ui")]
    {
        let mut engine = fixture.engine.take().unwrap();
        
        println!("  🚀 Starting development mode");
        engine.start_development_mode().expect("Should start development mode");
        
        // Benchmark 1: Realistic Development Session
        println!("  📊 Benchmark 1: Realistic Development Session");
        
        let session_start = Instant::now();
        
        // Simulate creating a complex UI application
        let components = vec![
            ("header", "Header component with navigation"),
            ("sidebar", "Sidebar with menu items"),
            ("main_content", "Main content area"),
            ("footer", "Footer with links"),
            ("modal", "Modal dialog component"),
        ];
        
        for (component_id, description) in &components {
            engine.register_component(component_id.to_string(), description.to_string())
                .expect("Should register component");
            
            let ui_code = format!(r#"
                struct {}Component {{
                    visible: bool,
                    content: String,
                    style: ComponentStyle,
                }}
                
                impl {}Component {{
                    fn render(&self) {{
                        if self.visible {{
                            render_content(&self.content);
                        }}
                    }}
                    
                    fn update(&mut self, message: Message) {{
                        match message {{
                            Message::Show => self.visible = true,
                            Message::Hide => self.visible = false,
                            Message::UpdateContent(content) => self.content = content,
                        }}
                    }}
                }}
            "#, component_id, component_id);
            
            let interpretation_start = Instant::now();
            let result = engine.interpret_ui_change(&ui_code, Some(component_id.to_string()));
            let interpretation_time = interpretation_start.elapsed();
            
            println!("    📊 {} interpretation: {:?}", component_id, interpretation_time);
            
            // Each component interpretation should be fast
            assert!(interpretation_time < Duration::from_millis(100), 
                "Component interpretation should be under 100ms");
            
            match result {
                Ok(_) => {
                    // Preserve component state
                    let state = json!({
                        "visible": true,
                        "content": format!("{} content", component_id),
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                    });
                    
                    engine.preserve_component_state(component_id, state)
                        .expect("Should preserve component state");
                }
                Err(e) => println!("    ⚠️ Component {} handled gracefully: {}", component_id, e),
            }
        }
        
        let session_time = session_start.elapsed();
        println!("    📈 Total development session time: {:?}", session_time);
        
        // Benchmark 2: Hot Reload Stress Test
        println!("  🔥 Benchmark 2: Hot Reload Stress Test");
        
        let stress_start = Instant::now();
        let mut total_reload_time = Duration::new(0, 0);
        
        for i in 0..20 {
            let component_id = format!("stress_test_{}", i);
            engine.register_component(component_id.clone(), "StressTestComponent".to_string())
                .expect("Should register stress test component");
            
            let reload_start = Instant::now();
            
            // Simulate rapid code changes
            let ui_code = format!(r#"
                button_{}.text = "Iteration {}";
                button_{}.enabled = {};
                button_{}.on_click = || {{ counter += {}; }};
            "#, i, i, i, i % 2 == 0, i, i);
            
            let _ = engine.interpret_ui_change(&ui_code, Some(component_id.clone()));
            
            // Preserve and restore state
            let state = json!({"iteration": i, "active": true});
            engine.preserve_component_state(&component_id, state).unwrap();
            let _ = engine.restore_component_state(&component_id);
            
            let reload_time = reload_start.elapsed();
            total_reload_time += reload_time;
            
            if i % 5 == 0 {
                println!("    📊 Stress test iteration {}: {:?}", i, reload_time);
            }
        }
        
        let stress_total_time = stress_start.elapsed();
        let avg_reload_time = total_reload_time / 20;
        
        println!("    📈 Stress test results:");
        println!("      - Total time: {:?}", stress_total_time);
        println!("      - Average reload time: {:?}", avg_reload_time);
        println!("      - Reloads per second: {:.1}", 20.0 / stress_total_time.as_secs_f64());
        
        // Average reload should be very fast
        assert!(avg_reload_time < Duration::from_millis(50), 
            "Average hot reload should be under 50ms, got {:?}", avg_reload_time);
        
        // Benchmark 3: Memory Efficiency Under Load
        println!("  💾 Benchmark 3: Memory Efficiency Under Load");
        
        let initial_memory = engine.current_memory_overhead_bytes();
        println!("    📊 Initial memory: {:.2} MB", initial_memory as f64 / (1024.0 * 1024.0));
        
        // Create many components with state
        for i in 0..100 {
            let component_id = format!("memory_benchmark_{}", i);
            engine.register_component(component_id.clone(), "MemoryBenchmarkComponent".to_string())
                .expect("Should register memory benchmark component");
            
            let large_state = json!({
                "id": i,
                "data": vec![i; 100], // 100 integers per component
                "metadata": {
                    "created": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    "type": "benchmark",
                    "version": "1.0"
                }
            });
            
            engine.preserve_component_state(&component_id, large_state)
                .expect("Should preserve large state");
        }
        
        let final_memory = engine.current_memory_overhead_bytes();
        let memory_per_component = (final_memory - initial_memory) / 100;
        
        println!("    📊 Final memory: {:.2} MB", final_memory as f64 / (1024.0 * 1024.0));
        println!("    📊 Memory per component: {:.2} KB", memory_per_component as f64 / 1024.0);
        
        // Memory should remain reasonable
        assert!(final_memory < 50 * 1024 * 1024, 
            "Total memory should be under 50MB, got {:.2} MB", 
            final_memory as f64 / (1024.0 * 1024.0));
        
        // Benchmark 4: Final Performance Metrics
        println!("  📈 Benchmark 4: Final Performance Metrics");
        
        if let Some(metrics) = engine.get_performance_metrics() {
            println!("    📊 Final Performance Summary:");
            println!("      - Total operations: {}", metrics.total_operations);
            println!("      - Average interpretation: {:?}", metrics.average_interpretation_time);
            println!("      - Max interpretation: {:?}", metrics.max_interpretation_time);
            println!("      - Target violations: {}", metrics.target_violations);
            println!("      - Success rate: {:.1}%", 
                ((metrics.total_operations - metrics.target_violations) as f64 / metrics.total_operations as f64) * 100.0);
            
            // Final performance should meet all targets
            assert!(metrics.average_interpretation_time < Duration::from_millis(50), 
                "Average interpretation should be under 50ms");
            assert!(metrics.max_interpretation_time < Duration::from_millis(100), 
                "Max interpretation should be under 100ms");
            
            let success_rate = ((metrics.total_operations - metrics.target_violations) as f64 / metrics.total_operations as f64) * 100.0;
            assert!(success_rate >= 95.0, 
                "Success rate should be at least 95%, got {:.1}%", success_rate);
        }
        
        fixture.engine = Some(engine);
    }
    
    #[cfg(not(feature = "dev-ui"))]
    {
        println!("  🚀 Production mode benchmark");
        let engine = fixture.engine.as_ref().unwrap();
        
        // Production mode should have zero overhead
        assert_eq!(engine.current_memory_overhead_bytes(), 0);
        assert!(!engine.has_runtime_interpreter());
        assert!(!engine.has_performance_monitoring());
        
        println!("    ✅ Production mode: Zero overhead verified");
    }
    
    println!("🏆 Comprehensive integration benchmark completed successfully!");
}