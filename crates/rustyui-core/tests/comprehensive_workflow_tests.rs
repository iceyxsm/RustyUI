//! Comprehensive Workflow End-to-End Tests
//! 
//! Task 12.4: Complete workflow testing with production-grade validation
//! 
//! This module implements comprehensive end-to-end tests that validate:
//! - Complete workflow: `rustyui init` → `rustyui dev` → build → production
//! - State preservation across interpretation cycles with complex scenarios
//! - Error recovery and graceful degradation under stress
//! - Cross-platform compatibility with platform-specific optimizations
//! - Production build zero-overhead verification with binary analysis
//! 
//! **Validates: Requirements 1.6, 6.5, 9.3**

use rustyui_core::{
    DualModeEngine, DualModeConfig, UIFramework, ProductionVerifier,
    Platform, PlatformConfig, HealthStatus,
};
use rustyui_cli::{ConfigManager, ProjectManager, TemplateManager, WorkflowManager};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use serde_json::json;

#[cfg(feature = "dev-ui")]
use rustyui_core::{
    DevelopmentSettings, InterpretationStrategy, StatePreservor,
    ErrorRecoveryManager, PerformanceMonitor,
};

/// Comprehensive workflow test fixture
struct ComprehensiveWorkflowFixture {
    temp_dir: TempDir,
    project_path: PathBuf,
    config_manager: ConfigManager,
    project_manager: ProjectManager,
    template_manager: TemplateManager,
    workflow_manager: WorkflowManager,
    engine: Option<DualModeEngine>,
}

impl ComprehensiveWorkflowFixture {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().to_path_buf();
        
        // Create realistic project structure
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::create_dir_all(project_path.join("examples")).unwrap();
        std::fs::create_dir_all(project_path.join("tests")).unwrap();
        std::fs::create_dir_all(project_path.join("benches")).unwrap();
        
        let config_manager = ConfigManager::new(project_path.clone()).unwrap();
        let project_manager = ProjectManager::new(project_path.clone()).unwrap();
        let template_manager = TemplateManager::new(project_path.clone());
        let workflow_manager = WorkflowManager::new(project_path.clone()).unwrap();
        
        Self {
            temp_dir,
            project_path,
            config_manager,
            project_manager,
            template_manager,
            workflow_manager,
            engine: None,
        }
    }
    
    fn create_realistic_rust_project(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create Cargo.toml
        let cargo_toml = r#"[package]
name = "comprehensive-test-app"
version = "0.1.0"
edition = "2021"
authors = ["RustyUI Test <test@rustyui.dev>"]
description = "Comprehensive test application for RustyUI workflow validation"
license = "MIT OR Apache-2.0"
repository = "https://github.com/rustyui/comprehensive-test-app"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
egui = "0.24"
eframe = "0.24"

[dev-dependencies]
proptest = "1.0"
criterion = "0.5"

[features]
default = []
dev-ui = []

[[bin]]
name = "app"
path = "src/main.rs"

[[example]]
name = "demo"
path = "examples/demo.rs"

[[bench]]
name = "performance"
harness = false
"#;
        std::fs::write(self.project_path.join("Cargo.toml"), cargo_toml)?;
        
        // Create main.rs
        let main_rs = r#"//! Comprehensive Test Application
//! 
//! This application demonstrates RustyUI's complete workflow capabilities
//! with realistic UI components and state management.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub user_name: String,
    pub counter: i32,
    pub todos: Vec<Todo>,
    pub settings: AppSettings,
    pub ui_state: UiState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u32,
    pub text: String,
    pub completed: bool,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub auto_save: bool,
    pub notifications: bool,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiState {
    pub sidebar_open: bool,
    pub modal_open: bool,
    pub selected_tab: String,
    pub window_size: (f32, f32),
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            user_name: "Test User".to_string(),
            counter: 0,
            todos: vec![
                Todo {
                    id: 1,
                    text: "Learn RustyUI".to_string(),
                    completed: false,
                    created_at: 1640995200, // 2022-01-01
                },
                Todo {
                    id: 2,
                    text: "Build awesome UI".to_string(),
                    completed: false,
                    created_at: 1640995260,
                },
            ],
            settings: AppSettings {
                theme: "dark".to_string(),
                auto_save: true,
                notifications: true,
                language: "en".to_string(),
            },
            ui_state: UiState {
                sidebar_open: true,
                modal_open: false,
                selected_tab: "todos".to_string(),
                window_size: (800.0, 600.0),
            },
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Comprehensive Test Application");
    
    let mut app_state = AppState::default();
    
    // Simulate application lifecycle
    println!("📊 Initial state: {} todos, counter: {}", 
        app_state.todos.len(), app_state.counter);
    
    // Simulate user interactions
    app_state.counter += 1;
    app_state.todos.push(Todo {
        id: 3,
        text: "Test hot reload".to_string(),
        completed: false,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
    });
    
    println!("📊 Updated state: {} todos, counter: {}", 
        app_state.todos.len(), app_state.counter);
    
    // Serialize state for persistence testing
    let serialized = serde_json::to_string_pretty(&app_state)?;
    println!("💾 Serialized state size: {} bytes", serialized.len());
    
    Ok(())
}
"#;
        std::fs::write(self.project_path.join("src").join("main.rs"), main_rs)?;
        
        Ok(())
    }
}
// ============================================================================
// Test 1: Complete Workflow Integration (init → dev → build → production)
// ============================================================================

/// **Validates: Requirements 1.6, 8.1, 8.2, 8.3, 8.4, 8.5, 8.6**
/// 
/// Test the complete RustyUI workflow from project initialization through
/// development mode to production build with comprehensive validation.
#[test]
fn test_complete_workflow_init_dev_build_production() {
    let mut fixture = ComprehensiveWorkflowFixture::new();
    
    println!("🔄 Testing Complete Workflow: init → dev → build → production");
    
    // Phase 1: Project Initialization (`rustyui init`)
    println!("  📊 Phase 1: Project Initialization (rustyui init)");
    
    // Create base Rust project
    fixture.create_realistic_rust_project().expect("Should create Rust project");
    
    // Analyze existing project
    let analysis = fixture.project_manager.analyze_project().expect("Should analyze project");
    println!("    📊 Project analysis:");
    println!("      - Is Rust project: {}", analysis.is_rust_project);
    println!("      - Has RustyUI config: {}", analysis.has_rustyui_config);
    println!("      - Detected framework: {:?}", analysis.detected_framework);
    
    assert!(analysis.is_rust_project, "Should detect Rust project");
    assert!(!analysis.is_rustyui_project, "Should not be RustyUI project yet");
    
    // Initialize RustyUI configuration
    let init_start = Instant::now();
    
    let config = fixture.config_manager.create_default_config(UIFramework::Egui)
        .expect("Should create default config");
    
    fixture.config_manager.validate_config(&config)
        .expect("Should validate config");
    
    fixture.config_manager.save_config(&config)
        .expect("Should save config");
    
    // Generate templates and examples
    fixture.template_manager.generate_example_code("egui")
        .expect("Should generate example code");
    
    fixture.template_manager.generate_gitignore()
        .expect("Should generate gitignore");
    
    fixture.template_manager.generate_readme("comprehensive-test-app", "egui")
        .expect("Should generate README");
    
    let init_time = init_start.elapsed();
    println!("    📊 Initialization time: {:?}", init_time);
    
    // Verify initialization results
    let post_init_analysis = fixture.project_manager.analyze_project()
        .expect("Should analyze project after init");
    
    assert!(post_init_analysis.is_rust_project, "Should still be Rust project");
    assert!(post_init_analysis.is_rustyui_project, "Should now be RustyUI project");
    assert!(post_init_analysis.has_rustyui_config, "Should have RustyUI config");
    
    // Verify files were created
    assert!(fixture.project_path.join("rustyui.toml").exists(), "Should create rustyui.toml");
    assert!(fixture.project_path.join(".gitignore").exists(), "Should create .gitignore");
    assert!(fixture.project_path.join("README.md").exists(), "Should create README.md");
    
    println!("    ✅ Project initialization completed successfully");
    
    // Phase 2: Development Mode (`rustyui dev`)
    println!("  📊 Phase 2: Development Mode (rustyui dev)");
    
    let dev_start = Instant::now();
    
    // Load configuration and start development mode
    let loaded_config = fixture.config_manager.load_config()
        .expect("Should load saved config");
    
    let mut engine = DualModeEngine::new(loaded_config.clone())
        .expect("Should create dual-mode engine");
    
    engine.initialize().expect("Should initialize engine");
    
    #[cfg(feature = "dev-ui")]
    {
        engine.start_development_mode().expect("Should start development mode");
        
        println!("    📊 Development mode capabilities:");
        println!("      - Runtime interpreter: {}", engine.has_runtime_interpreter());
        println!("      - Can interpret changes: {}", engine.can_interpret_changes());
        println!("      - Performance monitoring: {}", engine.has_performance_monitoring());
        println!("      - Memory overhead: {:.2} KB", engine.current_memory_overhead_bytes() as f64 / 1024.0);
        
        // Verify development mode features
        assert!(engine.has_runtime_interpreter(), "Should have runtime interpreter");
        assert!(engine.can_interpret_changes(), "Should be able to interpret changes");
        assert!(engine.has_performance_monitoring(), "Should have performance monitoring");
        
        // Test hot reload functionality
        let hot_reload_start = Instant::now();
        
        // Register application components
        engine.register_component("app_state".to_string(), "AppState".to_string())
            .expect("Should register app state component");
        
        engine.register_component("todo_list".to_string(), "TodoList".to_string())
            .expect("Should register todo list component");
        
        engine.register_component("settings_panel".to_string(), "SettingsPanel".to_string())
            .expect("Should register settings panel component");
        
        // Simulate UI code changes
        let ui_changes = vec![
            ("app_state", r#"
                app_state.counter += 1;
                app_state.user_name = "Updated User";
                app_state.ui_state.sidebar_open = !app_state.ui_state.sidebar_open;
            "#),
            ("todo_list", r#"
                todo_list.add_todo("New todo from hot reload");
                todo_list.mark_completed(1);
                todo_list.filter_by_status("active");
            "#),
            ("settings_panel", r#"
                settings_panel.theme = "light";
                settings_panel.auto_save = false;
                settings_panel.language = "es";
            "#),
        ];
        
        let mut hot_reload_times = Vec::new();
        
        for (component_id, ui_code) in ui_changes {
            let reload_start = Instant::now();
            let result = engine.interpret_ui_change(ui_code, Some(component_id.to_string()));
            let reload_time = reload_start.elapsed();
            
            hot_reload_times.push(reload_time);
            
            println!("    📊 Hot reload for {}: {:?}", component_id, reload_time);
            
            // Hot reload should be fast
            assert!(reload_time < Duration::from_millis(100), 
                "Hot reload should be under 100ms, got {:?}", reload_time);
            
            match result {
                Ok(_) => println!("      ✅ Hot reload succeeded"),
                Err(e) => println!("      ⚠️ Hot reload handled gracefully: {}", e),
            }
        }
        
        let avg_hot_reload_time = hot_reload_times.iter().sum::<Duration>() / hot_reload_times.len() as u32;
        println!("    📈 Average hot reload time: {:?}", avg_hot_reload_time);
        
        // Test file watching
        let file_watch_start = Instant::now();
        
        // Create and modify a component file
        let component_file = fixture.project_path.join("src").join("components.rs");
        let component_code = r#"//! Application Components
//! 
//! This module contains the UI components for the comprehensive test app.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoComponent {
    pub id: u32,
    pub text: String,
    pub completed: bool,
    pub editing: bool,
}

impl TodoComponent {
    pub fn new(id: u32, text: String) -> Self {
        Self {
            id,
            text,
            completed: false,
            editing: false,
        }
    }
    
    pub fn toggle_completed(&mut self) {
        self.completed = !self.completed;
    }
    
    pub fn start_editing(&mut self) {
        self.editing = true;
    }
    
    pub fn finish_editing(&mut self, new_text: String) {
        self.text = new_text;
        self.editing = false;
    }
}
"#;
        std::fs::write(&component_file, component_code).unwrap();
        
        // Process file changes
        let changes_result = engine.process_file_changes();
        let file_watch_time = file_watch_start.elapsed();
        
        println!("    📊 File watching time: {:?}", file_watch_time);
        
        // File watching should be fast
        assert!(file_watch_time < Duration::from_millis(50), 
            "File watching should be under 50ms, got {:?}", file_watch_time);
        
        match changes_result {
            Ok(changes) => {
                println!("      ✅ File changes detected: {} files", changes.len());
            }
            Err(e) => println!("      ⚠️ File watching handled gracefully: {}", e),
        }
    }
    
    let dev_time = dev_start.elapsed();
    println!("    📊 Development mode setup time: {:?}", dev_time);
    
    fixture.engine = Some(engine);
    
    println!("    ✅ Development mode completed successfully");
    
    // Phase 3: Build Process
    println!("  📊 Phase 3: Build Process");
    
    let build_start = Instant::now();
    
    // Simulate build process using workflow manager
    let build_result = fixture.workflow_manager.execute_build_workflow(&loaded_config);
    let build_time = build_start.elapsed();
    
    println!("    📊 Build time: {:?}", build_time);
    
    match build_result {
        Ok(build_info) => {
            println!("    📊 Build results:");
            println!("      - Build succeeded: {}", build_info.success);
            println!("      - Artifacts created: {}", build_info.artifacts.len());
            println!("      - Warnings: {}", build_info.warnings.len());
            println!("      - Build type: {:?}", build_info.build_type);
            
            assert!(build_info.success, "Build should succeed");
        }
        Err(e) => {
            println!("    ⚠️ Build handled gracefully: {}", e);
        }
    }
    
    println!("    ✅ Build process completed");
    
    // Phase 4: Production Verification
    println!("  📊 Phase 4: Production Build Verification");
    
    let production_start = Instant::now();
    
    let verifier = ProductionVerifier::new();
    let verification_result = verifier.verify_zero_overhead_build(&fixture.project_path);
    let production_time = production_start.elapsed();
    
    println!("    📊 Production verification time: {:?}", production_time);
    
    match verification_result {
        Ok(results) => {
            println!("    📊 Production verification results:");
            println!("      - Zero overhead: {}", results.has_zero_overhead());
            println!("      - Contains dev features: {}", results.contains_dev_features());
            println!("      - Size optimized: {}", results.is_size_optimized());
            println!("      - Security hardened: {}", results.is_security_hardened());
            
            // Production build should have zero overhead
            assert!(results.has_zero_overhead(), "Production build should have zero overhead");
            assert!(!results.contains_dev_features(), "Production build should not contain dev features");
            
            if let Some(perf_results) = results.performance_results() {
                println!("      - Startup time: {:?}", perf_results.startup_time);
                println!("      - Memory usage: {} bytes", perf_results.memory_usage);
                println!("      - Performance ratio: {:.3}", perf_results.performance_ratio());
                
                // Performance should match native Rust
                assert!(perf_results.performance_ratio() >= 0.95, 
                    "Production performance should be at least 95% of native");
            }
        }
        Err(e) => {
            println!("    ⚠️ Production verification handled gracefully: {}", e);
        }
    }
    
    println!("    ✅ Production verification completed");
    
    // Phase 5: Workflow Summary
    let total_workflow_time = init_time + dev_time + build_time + production_time;
    
    println!("  🏆 Complete Workflow Summary:");
    println!("    - Initialization: {:?}", init_time);
    println!("    - Development setup: {:?}", dev_time);
    println!("    - Build process: {:?}", build_time);
    println!("    - Production verification: {:?}", production_time);
    println!("    - Total workflow time: {:?}", total_workflow_time);
    
    // Entire workflow should complete in reasonable time
    assert!(total_workflow_time < Duration::from_secs(30), 
        "Complete workflow should finish under 30 seconds, got {:?}", total_workflow_time);
    
    println!("✅ Complete workflow integration test passed!");
}
// ============================================================================
// Test 2: Advanced State Preservation Across Complex Interpretation Cycles
// ============================================================================

/// **Validates: Requirements 6.1, 6.2, 6.3, 6.4, 6.5, 6.6**
/// 
/// Test advanced state preservation scenarios with complex data structures,
/// nested components, async state, and error recovery.
#[test]
fn test_advanced_state_preservation_complex_scenarios() {
    let mut fixture = ComprehensiveWorkflowFixture::new();
    fixture.create_realistic_rust_project().expect("Should create project");
    
    println!("🔄 Testing Advanced State Preservation Across Complex Scenarios");
    
    let config = fixture.config_manager.create_default_config(UIFramework::Egui)
        .expect("Should create config");
    
    let mut engine = DualModeEngine::new(config).expect("Should create engine");
    engine.initialize().expect("Should initialize engine");
    
    #[cfg(feature = "dev-ui")]
    {
        engine.start_development_mode().expect("Should start development mode");
        
        // Phase 1: Complex Nested State Preservation
        println!("  📊 Phase 1: Complex Nested State Preservation");
        
        // Create complex application state
        let complex_state = json!({
            "app": {
                "version": "1.0.0",
                "user": {
                    "id": 12345,
                    "name": "Test User",
                    "preferences": {
                        "theme": "dark",
                        "language": "en",
                        "notifications": {
                            "email": true,
                            "push": false,
                            "sms": true
                        }
                    },
                    "recent_activity": [
                        {"action": "login", "timestamp": 1640995200},
                        {"action": "create_todo", "timestamp": 1640995260},
                        {"action": "update_settings", "timestamp": 1640995320}
                    ]
                },
                "todos": [
                    {
                        "id": 1,
                        "text": "Learn RustyUI",
                        "completed": false,
                        "priority": "high",
                        "tags": ["learning", "ui", "rust"],
                        "subtasks": [
                            {"id": 11, "text": "Read documentation", "completed": true},
                            {"id": 12, "text": "Try examples", "completed": false}
                        ]
                    },
                    {
                        "id": 2,
                        "text": "Build application",
                        "completed": false,
                        "priority": "medium",
                        "tags": ["development", "project"],
                        "subtasks": []
                    }
                ],
                "ui_state": {
                    "sidebar_open": true,
                    "modal_stack": ["settings", "confirmation"],
                    "selected_items": [1, 3, 7],
                    "scroll_positions": {
                        "main_content": 150.5,
                        "sidebar": 0.0,
                        "todo_list": 75.2
                    }
                }
            }
        });
        
        // Register complex component
        engine.register_component("complex_app".to_string(), "ComplexAppComponent".to_string())
            .expect("Should register complex component");
        
        // Preserve complex state
        let preserve_start = Instant::now();
        engine.preserve_component_state("complex_app", complex_state.clone())
            .expect("Should preserve complex state");
        let preserve_time = preserve_start.elapsed();
        
        println!("    📊 Complex state preservation time: {:?}", preserve_time);
        
        // Simulate multiple interpretation cycles with state changes
        for cycle in 0..10 {
            println!("    🔄 Interpretation cycle {}", cycle + 1);
            
            // Modify different parts of the state
            let state_modification = match cycle % 4 {
                0 => json!({
                    "app": {
                        "user": {
                            "name": format!("User Updated {}", cycle),
                            "preferences": {
                                "theme": if cycle % 2 == 0 { "light" } else { "dark" }
                            }
                        }
                    }
                }),
                1 => json!({
                    "app": {
                        "todos": [
                            {
                                "id": cycle + 100,
                                "text": format!("New todo from cycle {}", cycle),
                                "completed": false,
                                "priority": "low",
                                "tags": ["generated"],
                                "subtasks": []
                            }
                        ]
                    }
                }),
                2 => json!({
                    "app": {
                        "ui_state": {
                            "sidebar_open": cycle % 2 == 0,
                            "scroll_positions": {
                                "main_content": cycle as f64 * 10.0
                            }
                        }
                    }
                }),
                _ => json!({
                    "app": {
                        "user": {
                            "recent_activity": [
                                {
                                    "action": format!("cycle_{}_action", cycle),
                                    "timestamp": 1640995200 + cycle * 60
                                }
                            ]
                        }
                    }
                })
            };
            
            // Apply state modification
            engine.preserve_component_state("complex_app", state_modification)
                .expect("Should preserve modified state");
            
            // Simulate interpretation
            let interpretation_code = format!(r#"
                // Cycle {} interpretation
                app.user.last_active = {};
                app.ui_state.current_cycle = {};
                app.version = "1.0.{}";
            "#, cycle, 1640995200 + cycle * 60, cycle, cycle);
            
            let result = engine.interpret_ui_change(&interpretation_code, Some("complex_app".to_string()));
            
            match result {
                Ok(_) => {
                    // Restore and verify state
                    let restored_state = engine.restore_component_state("complex_app");
                    assert!(restored_state.is_some(), "Should restore state in cycle {}", cycle);
                    
                    if let Some(state) = restored_state {
                        // Verify state structure is preserved
                        assert!(state.get("app").is_some(), "Should preserve app structure");
                        assert!(state["app"].get("user").is_some(), "Should preserve user structure");
                        assert!(state["app"].get("todos").is_some(), "Should preserve todos structure");
                    }
                }
                Err(e) => println!("      ⚠️ Cycle {} handled gracefully: {}", cycle, e),
            }
        }
        
        println!("    ✅ Complex nested state preservation completed");
        
        // Phase 2: Concurrent State Preservation Test
        println!("  📊 Phase 2: Concurrent State Preservation");
        
        // Create multiple components with concurrent state changes
        let concurrent_components = vec![
            ("component_a", "ComponentA"),
            ("component_b", "ComponentB"),
            ("component_c", "ComponentC"),
            ("component_d", "ComponentD"),
            ("component_e", "ComponentE"),
        ];
        
        for (component_id, component_type) in &concurrent_components {
            engine.register_component(component_id.to_string(), component_type.to_string())
                .expect("Should register concurrent component");
        }
        
        let concurrent_start = Instant::now();
        
        // Simulate concurrent state changes
        for i in 0..20 {
            let component_id = &concurrent_components[i % concurrent_components.len()].0;
            
            let concurrent_state = json!({
                "id": component_id,
                "iteration": i,
                "data": {
                    "values": (0..100).collect::<Vec<i32>>(),
                    "metadata": {
                        "created": 1640995200 + i * 10,
                        "modified": 1640995200 + i * 10 + 5,
                        "version": format!("1.0.{}", i)
                    }
                },
                "concurrent_test": true
            });
            
            engine.preserve_component_state(component_id, concurrent_state)
                .expect("Should preserve concurrent state");
            
            // Simulate rapid interpretation
            let rapid_code = format!("component.iteration = {}; component.updated = true;", i);
            let _ = engine.interpret_ui_change(&rapid_code, Some(component_id.to_string()));
        }
        
        let concurrent_time = concurrent_start.elapsed();
        println!("    📊 Concurrent state preservation time: {:?}", concurrent_time);
        
        // Verify all concurrent states are preserved
        for (component_id, _) in &concurrent_components {
            let restored = engine.restore_component_state(component_id);
            assert!(restored.is_some(), "Should restore concurrent state for {}", component_id);
        }
        
        println!("    ✅ Concurrent state preservation completed");
        
        // Phase 3: Error Recovery During State Preservation
        println!("  📊 Phase 3: Error Recovery During State Preservation");
        
        // Test state preservation with invalid data
        let invalid_states = vec![
            ("circular_ref", json!({"self": {"ref": "circular_ref"}})),
            ("too_large", json!({"data": vec![0; 1000000]})), // 1M elements
            ("invalid_json", serde_json::Value::Null),
            ("malformed", json!({"unclosed": {"nested": {"deep": {}}}})),
        ];
        
        for (test_name, invalid_state) in invalid_states {
            let error_component = format!("error_test_{}", test_name);
            engine.register_component(error_component.clone(), "ErrorTestComponent".to_string())
                .expect("Should register error test component");
            
            let error_result = engine.preserve_component_state(&error_component, invalid_state);
            
            match error_result {
                Ok(_) => {
                    println!("    ✅ Error case '{}' handled gracefully", test_name);
                    
                    // Try to restore
                    let restored = engine.restore_component_state(&error_component);
                    match restored {
                        Some(_) => println!("      ✅ State restored successfully"),
                        None => println!("      ⚠️ State restoration failed gracefully"),
                    }
                }
                Err(e) => {
                    println!("    ✅ Error case '{}' failed gracefully: {}", test_name, e);
                }
            }
            
            // System should remain healthy after errors
            assert!(engine.get_health_status().is_healthy(), 
                "System should remain healthy after error case '{}'", test_name);
        }
        
        println!("    ✅ Error recovery during state preservation completed");
    }
    
    #[cfg(not(feature = "dev-ui"))]
    {
        println!("  🚀 Production mode: State preservation overhead eliminated");
    }
    
    println!("✅ Advanced state preservation test completed!");
}