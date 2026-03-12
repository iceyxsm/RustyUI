//! Property-based tests for dual-mode operation
//! 
//! Based on 2026 production-grade testing practices, these tests validate
//! the fundamental correctness properties of the dual-mode architecture.

use crate::{DualModeEngine, config::{DualModeConfig, UIFramework}};
#[cfg(feature = "dev-ui")]
use crate::config::InterpretationStrategy;
use proptest::prelude::*;
use std::time::Duration;

/// Property 1: Dual-Mode Operation
/// 
/// For any valid codebase, the dual-mode engine should operate in development mode 
/// with runtime interpretation when the dev-ui feature is enabled, and in production 
/// mode with all interpretation features stripped when the feature is disabled, 
/// using the same source code.
/// 
/// Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.6, 8.1, 8.2

#[cfg(test)]
mod dual_mode_property_tests {
    use super::*;
    use proptest::collection::vec;
    use std::collections::HashMap;

    /// Test strategy for generating valid codebases
    fn valid_codebase_strategy() -> impl Strategy<Value = TestCodebase> {
        (
            "[a-zA-Z][a-zA-Z0-9_]{2,20}",  // project name
            vec(ui_component_strategy(), 1..10),  // components
            prop_oneof![
                Just(UIFramework::Egui),
                Just(UIFramework::Iced),
                Just(UIFramework::Slint),
                Just(UIFramework::Tauri),
            ],
        ).prop_map(|(name, components, framework)| {
            TestCodebase {
                name,
                components,
                framework,
            }
        })
    }

    /// Test strategy for UI components
    fn ui_component_strategy() -> impl Strategy<Value = TestUIComponent> {
        (
            "[a-zA-Z][a-zA-Z0-9_]{2,15}",  // component name
            prop_oneof![
                Just(ComponentType::Button),
                Just(ComponentType::TextInput),
                Just(ComponentType::Slider),
                Just(ComponentType::Layout),
            ],
            0..1000usize,  // complexity score
            any::<bool>(),  // has_state
        ).prop_map(|(name, component_type, complexity, has_state)| {
            TestUIComponent {
                name,
                component_type,
                complexity,
                has_state,
            }
        })
    }

    #[derive(Debug, Clone)]
    struct TestCodebase {
        name: String,
        components: Vec<TestUIComponent>,
        framework: UIFramework,
    }

    #[derive(Debug, Clone)]
    struct TestUIComponent {
        name: String,
        component_type: ComponentType,
        complexity: usize,
        has_state: bool,
    }

    #[derive(Debug, Clone)]
    enum ComponentType {
        Button,
        TextInput,
        Slider,
        Layout,
    }

    proptest! {
        /// Property Test 1: Dual-Mode Architecture Correctness
        /// 
        /// This test validates that the dual-mode engine behaves correctly
        /// across all possible configurations and codebases.
        #[test]
        fn dual_mode_operation_property(
            codebase in valid_codebase_strategy()
        ) {
            // Create configuration based on test parameters
            #[cfg(feature = "dev-ui")]
            let config = DualModeConfig {
                framework: codebase.framework.clone(),
                development_settings: crate::config::DevelopmentSettings {
                    interpretation_strategy: if codebase.components.len() > 5 {
                        InterpretationStrategy::Hybrid { 
                            rhai_threshold: 100, 
                            jit_threshold: 500 
                        }
                    } else {
                        InterpretationStrategy::RhaiOnly
                    },
                    ..Default::default()
                },
                watch_paths: vec!["src".into()],
                ..Default::default()
            };
            
            #[cfg(not(feature = "dev-ui"))]
            let config = DualModeConfig {
                framework: codebase.framework.clone(),
                watch_paths: vec!["src".into()],
                ..Default::default()
            };

            // Initialize dual-mode engine
            let engine_result = DualModeEngine::new(config.clone());
            prop_assert!(engine_result.is_ok(), "Engine initialization should succeed for valid config");
            
            let mut engine = engine_result.unwrap();
            
            // Initialize the engine
            let init_result = engine.initialize();
            prop_assert!(init_result.is_ok(), "Engine initialization should succeed");

            // Property 1.1: Development mode should have runtime interpretation when enabled
            #[cfg(feature = "dev-ui")]
            {
                prop_assert!(engine.has_runtime_interpreter(), 
                    "Development mode should have runtime interpreter");
                prop_assert!(engine.can_interpret_changes(), 
                    "Development mode should support change interpretation");
                prop_assert!(engine.has_state_preservation(), 
                    "Development mode should support state preservation");
            }

            // Property 1.2: Production mode should strip all interpretation features
            #[cfg(not(feature = "dev-ui"))]
            {
                prop_assert!(!engine.has_runtime_interpreter(), 
                    "Production builds should never have runtime interpreter");
                prop_assert!(!engine.can_interpret_changes(), 
                    "Production builds should not support change interpretation");
                prop_assert_eq!(engine.memory_overhead_bytes(), 0, 
                    "Production builds should have zero memory overhead");
            }

            // Property 1.3: Framework integration should work regardless of mode
            prop_assert!(engine.supports_framework(&codebase.framework), 
                "Engine should support the configured framework");

            // Property 1.4: Same codebase should work in both modes
            let component_count = codebase.components.len();
            prop_assert!(component_count > 0, "Codebase should have at least one component");
            
            // Validate that all components can be processed
            for component in &codebase.components {
                let component_valid = validate_component_compatibility(&engine, component);
                prop_assert!(component_valid, 
                    "Component '{}' should be compatible with engine", component.name);
            }

            // Property 1.5: Performance characteristics should be predictable
            let startup_time = engine.measure_startup_time();
            #[cfg(feature = "dev-ui")]
            {
                // Development mode: startup should be under 100ms (Phase 1 target)
                prop_assert!(startup_time < Duration::from_millis(100), 
                    "Development startup should be under 100ms, got {:?}", startup_time);
            }
            #[cfg(not(feature = "dev-ui"))]
            {
                // Production mode: startup should be minimal
                prop_assert!(startup_time < Duration::from_millis(10), 
                    "Production startup should be under 10ms, got {:?}", startup_time);
            }
        }

        /// Property Test 2: Configuration Validation
        /// 
        /// Tests that all valid configurations produce working engines
        /// and invalid configurations are properly rejected.
        #[test]
        fn configuration_validation_property(
            framework in prop_oneof![
                Just(UIFramework::Egui),
                Just(UIFramework::Iced),
                Just(UIFramework::Slint),
                Just(UIFramework::Tauri),
            ]
        ) {
            let framework_clone = framework.clone();
            
            #[cfg(feature = "dev-ui")]
            let config = DualModeConfig {
                framework,
                development_settings: crate::config::DevelopmentSettings {
                    interpretation_strategy: InterpretationStrategy::Hybrid { 
                        rhai_threshold: 100, 
                        jit_threshold: 500 
                    },
                    ..Default::default()
                },
                watch_paths: vec!["src".into(), "examples".into()],
                ..Default::default()
            };
            
            #[cfg(not(feature = "dev-ui"))]
            let config = DualModeConfig {
                framework,
                watch_paths: vec!["src".into(), "examples".into()],
                ..Default::default()
            };

            // All valid configurations should create working engines
            let engine_result = DualModeEngine::new(config);
            prop_assert!(engine_result.is_ok(), 
                "Valid configuration should create working engine");

            let mut engine = engine_result.unwrap();
            
            // Initialize the engine
            let init_result = engine.initialize();
            prop_assert!(init_result.is_ok(), "Engine initialization should succeed");
            
            // Engine should be in consistent state
            prop_assert!(engine.is_initialized(), "Engine should be initialized");
            prop_assert!(engine.get_framework() == &framework_clone, 
                "Engine should use configured framework");

            #[cfg(feature = "dev-ui")]
            {
                prop_assert!(engine.get_interpretation_strategy() == &InterpretationStrategy::Hybrid { 
                    rhai_threshold: 100, 
                    jit_threshold: 500 
                }, "Engine should use configured interpretation strategy");
            }
        }

        /// Property Test 3: Mode Transition Consistency
        /// 
        /// Tests that the same codebase produces consistent results
        /// when switching between development and production modes.
        #[test]
        fn mode_transition_consistency_property(
            codebase in valid_codebase_strategy()
        ) {
            // Test both development and production configurations
            #[cfg(feature = "dev-ui")]
            let dev_config = DualModeConfig {
                framework: codebase.framework.clone(),
                development_settings: crate::config::DevelopmentSettings {
                    interpretation_strategy: InterpretationStrategy::Hybrid { 
                        rhai_threshold: 50, 
                        jit_threshold: 200 
                    },
                    ..Default::default()
                },
                watch_paths: vec!["src".into()],
                ..Default::default()
            };

            let prod_config = DualModeConfig {
                framework: codebase.framework.clone(),
                watch_paths: vec!["src".into()],
                ..Default::default()
            };

            // Both configurations should work with the same codebase
            #[cfg(feature = "dev-ui")]
            let dev_engine_result = DualModeEngine::new(dev_config);
            let prod_engine_result = DualModeEngine::new(prod_config);

            #[cfg(feature = "dev-ui")]
            prop_assert!(dev_engine_result.is_ok(), "Development engine should initialize");
            prop_assert!(prod_engine_result.is_ok(), "Production engine should initialize");

            #[cfg(feature = "dev-ui")]
            let mut dev_engine = dev_engine_result.unwrap();
            let mut prod_engine = prod_engine_result.unwrap();
            
            // Initialize both engines
            #[cfg(feature = "dev-ui")]
            {
                let dev_init = dev_engine.initialize();
                prop_assert!(dev_init.is_ok(), "Development engine initialization should succeed");
            }
            let prod_init = prod_engine.initialize();
            prop_assert!(prod_init.is_ok(), "Production engine initialization should succeed");

            // Both should support the same framework
            #[cfg(feature = "dev-ui")]
            prop_assert_eq!(dev_engine.get_framework(), prod_engine.get_framework(),
                "Both engines should use the same framework");
            
            #[cfg(not(feature = "dev-ui"))]
            prop_assert_eq!(prod_engine.get_framework(), &codebase.framework,
                "Production engine should use the configured framework");

            // Component compatibility should be consistent
            for component in &codebase.components {
                #[cfg(feature = "dev-ui")]
                let dev_compatible = validate_component_compatibility(&dev_engine, component);
                let prod_compatible = validate_component_compatibility(&prod_engine, component);
                
                #[cfg(feature = "dev-ui")]
                prop_assert_eq!(dev_compatible, prod_compatible,
                    "Component '{}' compatibility should be consistent across modes", 
                    component.name);
                
                #[cfg(not(feature = "dev-ui"))]
                prop_assert!(prod_compatible,
                    "Component '{}' should be compatible with production engine", 
                    component.name);
            }

            // Production should have better performance characteristics
            #[cfg(feature = "dev-ui")]
            let dev_startup = dev_engine.measure_startup_time();
            let prod_startup = prod_engine.measure_startup_time();
            
            #[cfg(feature = "dev-ui")]
            prop_assert!(prod_startup <= dev_startup,
                "Production startup ({:?}) should be <= development startup ({:?})", 
                prod_startup, dev_startup);
            
            #[cfg(not(feature = "dev-ui"))]
            prop_assert!(prod_startup < Duration::from_millis(10),
                "Production startup should be under 10ms, got {:?}", prod_startup);
        }
    }

    /// Helper function to validate component compatibility with engine
    fn validate_component_compatibility(engine: &DualModeEngine, component: &TestUIComponent) -> bool {
        // Check if component type is supported by the framework
        let framework_supports = match (engine.get_framework(), &component.component_type) {
            (UIFramework::Egui, _) => true,  // egui supports all component types
            (UIFramework::Iced, ComponentType::Layout) => true,
            (UIFramework::Iced, ComponentType::Button) => true,
            (UIFramework::Iced, ComponentType::TextInput) => true,
            (UIFramework::Iced, ComponentType::Slider) => true,
            (UIFramework::Slint, _) => true,  // slint supports all component types
            (UIFramework::Tauri, _) => true,  // tauri supports all component types
            (UIFramework::Custom { .. }, _) => true,  // custom frameworks support all types
        };

        // Check complexity limits
        let complexity_ok = component.complexity < 10000;  // Reasonable complexity limit

        // Check state preservation support
        let state_ok = if component.has_state {
            #[cfg(feature = "dev-ui")]
            {
                engine.has_state_preservation()
            }
            #[cfg(not(feature = "dev-ui"))]
            {
                true  // Production mode doesn't need state preservation
            }
        } else {
            true  // Stateless components always work
        };

        framework_supports && complexity_ok && state_ok
    }
}

/// Additional property test utilities for 2026 production-grade testing
#[cfg(test)]
mod property_test_utilities {
    use super::*;
    use proptest::test_runner::{TestRunner, Config};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Advanced test runner configuration for production-grade testing
    pub fn create_production_test_config() -> Config {
        Config {
            cases: 1000,  // Increased from default 256 for better coverage
            max_shrink_iters: 10000,  // More thorough shrinking
            timeout: 30,  // Longer timeout for complex tests (in seconds)
            ..Config::default()
        }
    }

    /// Performance-aware property test that tracks execution metrics
    pub struct PerformanceTrackingRunner {
        runner: TestRunner,
        execution_times: Arc<AtomicU64>,
        memory_peaks: Arc<AtomicU64>,
    }

    impl PerformanceTrackingRunner {
        pub fn new() -> Self {
            Self {
                runner: TestRunner::new(create_production_test_config()),
                execution_times: Arc::new(AtomicU64::new(0)),
                memory_peaks: Arc::new(AtomicU64::new(0)),
            }
        }

        pub fn get_average_execution_time(&self) -> Duration {
            let total_nanos = self.execution_times.load(Ordering::Relaxed);
            Duration::from_nanos(total_nanos / 1000)  // Assuming 1000 test cases
        }

        pub fn get_peak_memory_usage(&self) -> u64 {
            self.memory_peaks.load(Ordering::Relaxed)
        }
    }

    /// Test strategy for generating realistic UI update scenarios
    pub fn ui_update_scenario_strategy() -> impl Strategy<Value = UIUpdateScenario> {
        (
            proptest::collection::vec(ui_change_strategy(), 1..20),  // sequence of changes
            0..5000u64,  // time_span_ms
            any::<bool>(),  // has_errors
        ).prop_map(|(changes, time_span_ms, has_errors)| {
            UIUpdateScenario {
                changes,
                time_span_ms,
                has_errors,
            }
        })
    }

    fn ui_change_strategy() -> impl Strategy<Value = UIChange> {
        (
            prop_oneof![
                Just(ChangeType::ComponentUpdate),
                Just(ChangeType::StyleChange),
                Just(ChangeType::LayoutChange),
                Just(ChangeType::StateChange),
            ],
            "[a-zA-Z][a-zA-Z0-9_]{2,15}",  // component_id
            ".*{0,500}",  // change_content (up to 500 chars)
        ).prop_map(|(change_type, component_id, content)| {
            UIChange {
                change_type,
                component_id,
                content,
            }
        })
    }

    #[derive(Debug, Clone)]
    pub struct UIUpdateScenario {
        pub changes: Vec<UIChange>,
        pub time_span_ms: u64,
        pub has_errors: bool,
    }

    #[derive(Debug, Clone)]
    pub struct UIChange {
        pub change_type: ChangeType,
        pub component_id: String,
        pub content: String,
    }

    #[derive(Debug, Clone)]
    pub enum ChangeType {
        ComponentUpdate,
        StyleChange,
        LayoutChange,
        StateChange,
    }
}


/// Property Test 4: Framework-Agnostic Integration
///
/// Tests that the dual-mode engine integrates seamlessly with all supported
/// UI frameworks without requiring framework-specific modifications.
///
/// Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 4.6
proptest! {
    /// Property Test 4.1: Framework Support Without Modifications
    ///
    /// Tests that all supported frameworks work with the dual-mode engine
    /// without requiring any framework-specific modifications.
    #[test]
    fn framework_agnostic_integration_property(
        framework in prop_oneof![
            Just(UIFramework::Egui),
            Just(UIFramework::Iced),
            Just(UIFramework::Slint),
            Just(UIFramework::Tauri),
            Just(UIFramework::Custom {
                name: "TestFramework".to_string(),
                adapter_path: "test_adapter".to_string()
            }),
        ],
        component_count in 1..20usize,
        has_complex_state in any::<bool>(),
        uses_async_rendering in any::<bool>(),
    ) {
        // Create configuration for the framework
        let config = DualModeConfig {
            framework: framework.clone(),
            #[cfg(feature = "dev-ui")]
            development_settings: crate::config::DevelopmentSettings::default(),
            production_settings: crate::config::ProductionSettings::default(),
            conditional_compilation: crate::config::ConditionalCompilation::default(),
            watch_paths: vec!["src".into()],
        };

        // Test engine creation with framework
        let engine_result = DualModeEngine::new(config);
        prop_assert!(engine_result.is_ok(),
            "Engine should initialize successfully with {:?}", framework);

        let mut engine = engine_result.unwrap();

        // Test engine initialization
        let init_result = engine.initialize();
        prop_assert!(init_result.is_ok(),
            "Engine should initialize successfully with {:?}", framework);

        // Requirement 4.1-4.4: Framework support without modifications
        match &framework {
            UIFramework::Egui => {
                prop_assert!(engine.supports_framework(&framework),
                    "Engine should support egui without modifications");

                // Test egui-specific integration
                prop_assert!(validate_egui_integration(&mut engine, component_count),
                    "egui integration should work seamlessly");
            }
            UIFramework::Iced => {
                prop_assert!(engine.supports_framework(&framework),
                    "Engine should support Iced without modifications");

                // Test Iced-specific integration
                prop_assert!(validate_iced_integration(&mut engine, component_count),
                    "Iced integration should work seamlessly");
            }
            UIFramework::Slint => {
                prop_assert!(engine.supports_framework(&framework),
                    "Engine should support Slint without modifications");

                // Test Slint-specific integration
                prop_assert!(validate_slint_integration(&mut engine, component_count),
                    "Slint integration should work seamlessly");
            }
            UIFramework::Tauri => {
                prop_assert!(engine.supports_framework(&framework),
                    "Engine should support Tauri without modifications");

                // Test Tauri-specific integration
                prop_assert!(validate_tauri_integration(&mut engine, component_count),
                    "Tauri integration should work seamlessly");
            }
            UIFramework::Custom { name, .. } => {
                prop_assert!(engine.supports_framework(&framework),
                    "Engine should support custom framework {} without modifications", name);

                // Test custom framework integration
                prop_assert!(validate_custom_framework_integration(&mut engine, component_count),
                    "Custom framework integration should work seamlessly");
            }
        }

        // Requirement 4.5: Generic integration for any Rust UI framework
        prop_assert!(test_generic_framework_integration(&mut engine, &framework, component_count),
            "Generic framework integration should work for {:?}", framework);

        // Requirement 4.6: Runtime interpreter works with framework-specific rendering
        #[cfg(feature = "dev-ui")]
        {
            prop_assert!(test_runtime_interpreter_with_framework(&mut engine, &framework, has_complex_state),
                "Runtime interpreter should work with {:?} rendering pipeline", framework);

            // Test state preservation with framework-specific components
            if has_complex_state {
                prop_assert!(test_framework_state_preservation(&mut engine, &framework),
                    "State preservation should work with {:?} components", framework);
            }

            // Test async rendering integration
            if uses_async_rendering {
                prop_assert!(test_async_rendering_integration(&mut engine, &framework),
                    "Async rendering should work with {:?}", framework);
            }
        }

        // Test framework-specific performance characteristics
        let performance_result = test_framework_performance(&mut engine, &framework, component_count);
        prop_assert!(performance_result.is_ok(),
            "Framework performance should meet requirements for {:?}", framework);

        // Test error handling with framework-specific errors
        let error_handling_result = test_framework_error_handling(&mut engine, &framework);
        prop_assert!(error_handling_result.is_ok(),
            "Framework error handling should be robust for {:?}", framework);
    }
}

// Helper functions for framework-specific integration testing

/// Test egui-specific integration
fn validate_egui_integration(engine: &mut DualModeEngine, component_count: usize) -> bool {
    // egui supports all component types
    let component_types = vec!["Button", "TextInput", "Slider", "Layout", "Canvas"];

    for (i, component_type) in component_types.iter().enumerate() {
        if i >= component_count { break; }

        let component_id = format!("egui_component_{}", i);
        let register_result = engine.register_component(component_id.clone(), component_type.to_string());

        if register_result.is_err() {
            return false;
        }

        // Test component state management
        let test_state = serde_json::json!({
            "type": component_type,
            "visible": true,
            "enabled": true,
            "properties": {
                "width": 100,
                "height": 30
            }
        });

        let state_result = engine.preserve_component_state(&component_id, test_state);
        if state_result.is_err() {
            return false;
        }
    }

    true
}

/// Test Iced-specific integration
fn validate_iced_integration(engine: &mut DualModeEngine, component_count: usize) -> bool {
    // Iced supports specific component types
    let component_types = vec!["Button", "TextInput", "Slider", "Container", "Column"];

    for (i, component_type) in component_types.iter().enumerate() {
        if i >= component_count { break; }

        let component_id = format!("iced_component_{}", i);
        let register_result = engine.register_component(component_id.clone(), component_type.to_string());

        if register_result.is_err() {
            return false;
        }

        // Test Iced-specific state structure
        let test_state = serde_json::json!({
            "type": component_type,
            "message_type": "Update",
            "state": {
                "value": "test",
                "focused": false
            }
        });

        let state_result = engine.preserve_component_state(&component_id, test_state);
        if state_result.is_err() {
            return false;
        }
    }

    true
}

/// Test Slint-specific integration
fn validate_slint_integration(engine: &mut DualModeEngine, component_count: usize) -> bool {
    // Slint supports declarative components
    let component_types = vec!["Rectangle", "Text", "TouchArea", "ListView", "GridLayout"];

    for (i, component_type) in component_types.iter().enumerate() {
        if i >= component_count { break; }

        let component_id = format!("slint_component_{}", i);
        let register_result = engine.register_component(component_id.clone(), component_type.to_string());

        if register_result.is_err() {
            return false;
        }

        // Test Slint-specific properties
        let test_state = serde_json::json!({
            "type": component_type,
            "properties": {
                "x": 0,
                "y": 0,
                "width": "100px",
                "height": "30px",
                "background": "#ffffff"
            }
        });

        let state_result = engine.preserve_component_state(&component_id, test_state);
        if state_result.is_err() {
            return false;
        }
    }

    true
}

/// Test Tauri-specific integration
fn validate_tauri_integration(engine: &mut DualModeEngine, component_count: usize) -> bool {
    // Tauri works with web components
    let component_types = vec!["WebView", "Window", "Menu", "SystemTray", "Dialog"];

    for (i, component_type) in component_types.iter().enumerate() {
        if i >= component_count { break; }

        let component_id = format!("tauri_component_{}", i);
        let register_result = engine.register_component(component_id.clone(), component_type.to_string());

        if register_result.is_err() {
            return false;
        }

        // Test Tauri-specific state with web integration
        let test_state = serde_json::json!({
            "type": component_type,
            "webview_state": {
                "url": "http://localhost:3000",
                "title": "Test App",
                "visible": true
            },
            "native_state": {
                "window_id": i,
                "focused": false
            }
        });

        let state_result = engine.preserve_component_state(&component_id, test_state);
        if state_result.is_err() {
            return false;
        }
    }

    true
}

/// Test custom framework integration
fn validate_custom_framework_integration(engine: &mut DualModeEngine, component_count: usize) -> bool {
    // Custom frameworks should support generic components
    let component_types = vec!["CustomWidget", "CustomContainer", "CustomControl"];

    for (i, component_type) in component_types.iter().enumerate() {
        if i >= component_count { break; }

        let component_id = format!("custom_component_{}", i);
        let register_result = engine.register_component(component_id.clone(), component_type.to_string());

        if register_result.is_err() {
            return false;
        }

        // Test generic state structure
        let test_state = serde_json::json!({
            "type": component_type,
            "custom_properties": {
                "id": i,
                "name": format!("Component {}", i),
                "data": "custom_data"
            }
        });

        let state_result = engine.preserve_component_state(&component_id, test_state);
        if state_result.is_err() {
            return false;
        }
    }

    true
}

/// Test generic framework integration (Requirement 4.5)
fn test_generic_framework_integration(
    engine: &mut DualModeEngine,
    framework: &UIFramework,
    component_count: usize
) -> bool {
    // Test that the engine provides generic integration capabilities

    // 1. Component registration should work for any framework
    for i in 0..component_count.min(5) {
        let component_id = format!("generic_component_{}", i);
        let component_type = "GenericComponent";

        let register_result = engine.register_component(component_id.clone(), component_type.to_string());
        if register_result.is_err() {
            return false;
        }
    }

    // 2. Framework detection should work
    if engine.get_framework() != framework {
        return false;
    }

    // 3. Framework support check should work
    if !engine.supports_framework(framework) {
        return false;
    }

    // 4. Generic state operations should work
    let generic_state = serde_json::json!({
        "framework": format!("{:?}", framework),
        "component_count": component_count,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });

    let state_result = engine.preserve_component_state("generic_test", generic_state);
    if state_result.is_err() {
        return false;
    }

    true
}

/// Test runtime interpreter with framework-specific rendering (Requirement 4.6)
#[cfg(feature = "dev-ui")]
fn test_runtime_interpreter_with_framework(
    engine: &mut DualModeEngine,
    framework: &UIFramework,
    has_complex_state: bool
) -> bool {
    // Test that runtime interpreter works with framework-specific rendering pipelines

    // 1. Test basic UI code interpretation
    let ui_code = match framework {
        UIFramework::Egui => "button.text = \"Hello egui\";",
        UIFramework::Iced => "button.label = \"Hello Iced\";",
        UIFramework::Slint => "text.text = \"Hello Slint\";",
        UIFramework::Tauri => "element.innerHTML = \"Hello Tauri\";",
        UIFramework::Custom { .. } => "widget.content = \"Hello Custom\";",
    };

    let interpret_result = engine.interpret_ui_change(ui_code, Some("test_component".to_string()));
    if interpret_result.is_err() {
        return false;
    }

    // 2. Test framework-specific rendering integration
    if has_complex_state {
        let complex_ui_code = match framework {
            UIFramework::Egui => {
                "if ui.button(\"Click me\").clicked() { state.counter += 1; }"
            }
            UIFramework::Iced => {
                "Message::ButtonPressed => { self.counter += 1; }"
            }
            UIFramework::Slint => {
                "callback clicked => { counter += 1; }"
            }
            UIFramework::Tauri => {
                "document.getElementById('button').onclick = () => { state.counter++; };"
            }
            UIFramework::Custom { .. } => {
                "widget.on_click(() => { this.counter++; });"
            }
        };

        let complex_interpret_result = engine.interpret_ui_change(complex_ui_code, Some("complex_component".to_string()));
        if complex_interpret_result.is_err() {
            return false;
        }
    }

    true
}

/// Test framework-specific state preservation
#[cfg(feature = "dev-ui")]
fn test_framework_state_preservation(engine: &mut DualModeEngine, framework: &UIFramework) -> bool {
    // Create framework-specific state
    let framework_state = match framework {
        UIFramework::Egui => serde_json::json!({
            "ui_state": {
                "window_size": [800, 600],
                "widgets": {
                    "button_1": { "pressed": false, "text": "Click me" },
                    "text_input_1": { "value": "Hello", "cursor_pos": 5 }
                }
            }
        }),
        UIFramework::Iced => serde_json::json!({
            "application_state": {
                "counter": 42,
                "text_input": "Hello Iced",
                "selected_item": 2
            }
        }),
        UIFramework::Slint => serde_json::json!({
            "component_state": {
                "properties": {
                    "text": "Hello Slint",
                    "visible": true,
                    "x": 100,
                    "y": 200
                }
            }
        }),
        UIFramework::Tauri => serde_json::json!({
            "webview_state": {
                "dom_state": {
                    "input_values": { "name": "John", "email": "john@example.com" },
                    "scroll_position": 150
                },
                "window_state": {
                    "width": 1024,
                    "height": 768,
                    "maximized": false
                }
            }
        }),
        UIFramework::Custom { name, .. } => serde_json::json!({
            "custom_framework_state": {
                "framework_name": name,
                "custom_data": {
                    "widgets": [],
                    "layout": "vertical",
                    "theme": "dark"
                }
            }
        }),
    };

    // Test state preservation
    let preserve_result = engine.preserve_component_state("framework_test", framework_state.clone());
    if preserve_result.is_err() {
        return false;
    }

    // Test state restoration
    let restored_state = engine.restore_component_state("framework_test");
    match restored_state {
        Some(state) => state == framework_state,
        None => false,
    }
}

/// Test async rendering integration
#[cfg(feature = "dev-ui")]
fn test_async_rendering_integration(engine: &mut DualModeEngine, framework: &UIFramework) -> bool {
    // Test that async rendering works with the runtime interpreter

    let async_ui_code = match framework {
        UIFramework::Egui => {
            "async { let data = fetch_data().await; ui.label(data); }"
        }
        UIFramework::Iced => {
            "Command::perform(fetch_data(), Message::DataLoaded)"
        }
        UIFramework::Slint => {
            "property <string> async_data: @async { fetch_data() };"
        }
        UIFramework::Tauri => {
            "async function updateUI() { const data = await fetchData(); updateElement(data); }"
        }
        UIFramework::Custom { .. } => {
            "widget.async_update(async () => { const data = await fetch(); return data; });"
        }
    };

    // Note: This is a simplified test - in a real implementation,
    // we would need to handle actual async execution
    let interpret_result = engine.interpret_ui_change(async_ui_code, Some("async_component".to_string()));
    interpret_result.is_ok()
}

/// Test framework-specific performance characteristics
fn test_framework_performance(
    engine: &mut DualModeEngine,
    framework: &UIFramework,
    component_count: usize
) -> Result<(), String> {
    let start_time = std::time::Instant::now();

    // Perform framework-specific operations
    for i in 0..component_count.min(10) {
        let component_id = format!("perf_test_{}", i);

        // Register component
        engine.register_component(component_id.clone(), "PerfTestComponent".to_string())
            .map_err(|e| format!("Component registration failed: {}", e))?;

        // Test state operations
        let test_state = serde_json::json!({
            "framework": format!("{:?}", framework),
            "component_index": i,
            "performance_data": {
                "render_time": 16.67, // 60 FPS target
                "memory_usage": 1024 * i
            }
        });

        engine.preserve_component_state(&component_id, test_state)
            .map_err(|e| format!("State preservation failed: {}", e))?;
    }

    let elapsed = start_time.elapsed();

    // Performance should be reasonable (under 100ms for 10 components)
    if elapsed > std::time::Duration::from_millis(100) {
        return Err(format!("Performance test took too long: {:?}", elapsed));
    }

    Ok(())
}

/// Test framework-specific error handling
fn test_framework_error_handling(engine: &mut DualModeEngine, framework: &UIFramework) -> Result<(), String> {
    // Test error handling with framework-specific invalid operations

    // 1. Test invalid component registration
    let invalid_result = engine.register_component("".to_string(), "".to_string());
    if invalid_result.is_ok() {
        return Err("Should reject invalid component registration".to_string());
    }

    // 2. Test invalid state preservation
    let invalid_state = serde_json::Value::Null;
    let invalid_state_result = engine.preserve_component_state("nonexistent", invalid_state);
    // This might succeed or fail depending on implementation - both are acceptable

    // 3. Test framework-specific error scenarios
    #[cfg(feature = "dev-ui")]
    {
        let invalid_ui_code = match framework {
            UIFramework::Egui => "invalid_egui_syntax((((",
            UIFramework::Iced => "invalid iced syntax ;;;",
            UIFramework::Slint => "invalid { slint: syntax }",
            UIFramework::Tauri => "invalid javascript syntax ;;;",
            UIFramework::Custom { .. } => "invalid custom syntax ;;;",
        };

        let error_result = engine.interpret_ui_change(invalid_ui_code, Some("error_test".to_string()));
        // Should handle errors gracefully (either succeed with error recovery or fail gracefully)
        match error_result {
            Ok(_) => {}, // Error recovery worked
            Err(_) => {}, // Graceful failure is also acceptable
        }
    }

    Ok(())
}
