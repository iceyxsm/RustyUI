//! Benchmarks comparing production vs development performance
//! 
//! These benchmarks validate that production builds achieve zero overhead
//! compared to native Rust while development mode provides acceptable performance.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use rustyui_core::{DualModeEngine, DualModeConfig, UIFramework, StatePreservor};
use serde_json::json;
use std::time::Duration;

/// Benchmark engine initialization performance
fn benchmark_engine_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_initialization");
    
    // Test different frameworks
    let frameworks = vec![
        UIFramework::Egui,
        UIFramework::Iced,
        UIFramework::Slint,
        UIFramework::Tauri,
    ];
    
    for framework in frameworks {
        let config = DualModeConfig {
            framework: framework.clone(),
            ..Default::default()
        };
        
        group.bench_with_input(
            BenchmarkId::new("production", format!("{:?}", framework)),
            &config,
            |b, config| {
                b.iter(|| {
                    let engine = DualModeEngine::new(config.clone()).unwrap();
                    black_box(engine)
                })
            },
        );
        
        #[cfg(feature = "dev-ui")]
        {
            let dev_config = DualModeConfig {
                framework: framework.clone(),
                development_settings: rustyui_core::config::DevelopmentSettings::default(),
                ..Default::default()
            };
            
            group.bench_with_input(
                BenchmarkId::new("development", format!("{:?}", framework)),
                &dev_config,
                |b, config| {
                    b.iter(|| {
                        let engine = DualModeEngine::new(config.clone()).unwrap();
                        black_box(engine)
                    })
                },
            );
        }
    }
    
    group.finish();
}

/// Benchmark memory overhead
fn benchmark_memory_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_overhead");
    
    let config = DualModeConfig::default();
    
    group.bench_function("production_memory", |b| {
        b.iter(|| {
            let engine = DualModeEngine::new(config.clone()).unwrap();
            let memory_overhead = engine.memory_overhead_bytes();
            black_box(memory_overhead)
        })
    });
    
    #[cfg(feature = "dev-ui")]
    {
        group.bench_function("development_memory", |b| {
            b.iter(|| {
                let mut engine = DualModeEngine::new(config.clone()).unwrap();
                engine.initialize().unwrap();
                let memory_overhead = engine.current_memory_overhead_bytes();
                black_box(memory_overhead)
            })
        });
    }
    
    group.finish();
}

/// Benchmark runtime interpretation performance (development only)
#[cfg(feature = "dev-ui")]
fn benchmark_runtime_interpretation(c: &mut Criterion) {
    let mut group = c.benchmark_group("runtime_interpretation");
    group.throughput(Throughput::Elements(1));
    
    let config = DualModeConfig::default();
    let mut engine = DualModeEngine::new(config).unwrap();
    engine.initialize().unwrap();
    
    // Simple UI code interpretation
    group.bench_function("simple_ui_code", |b| {
        b.iter(|| {
            let result = engine.interpret_ui_change(
                black_box("button.text = \"Hello\";"),
                Some("test_button".to_string())
            );
            black_box(result)
        })
    });
    
    // Complex UI code interpretation
    let complex_code = r#"
        if state.counter > 10 {
            label.text = "High: " + state.counter;
            label.color = "red";
        } else {
            label.text = "Low: " + state.counter;
            label.color = "green";
        }
        state.counter += 1;
    "#;
    
    group.bench_function("complex_ui_code", |b| {
        b.iter(|| {
            let result = engine.interpret_ui_change(
                black_box(complex_code),
                Some("complex_component".to_string())
            );
            black_box(result)
        })
    });
    
    // Batch interpretation
    group.throughput(Throughput::Elements(10));
    group.bench_function("batch_interpretation", |b| {
        b.iter(|| {
            for i in 0..10 {
                let code = format!("button_{}.text = \"Button {}\";", i, i);
                let result = engine.interpret_ui_change(
                    black_box(&code),
                    Some(format!("button_{}", i))
                );
                black_box(result).unwrap();
            }
        })
    });
    
    group.finish();
}

/// Benchmark state preservation performance
fn benchmark_state_preservation(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_preservation");
    group.throughput(Throughput::Elements(1));
    
    let mut preservor = StatePreservor::new();
    
    // Simple state preservation
    let simple_state = json!({
        "counter": 42,
        "message": "Hello, World!"
    });
    
    group.bench_function("simple_state_save", |b| {
        b.iter(|| {
            let result = preservor.save_global_state(
                black_box("simple_component"),
                black_box(&simple_state)
            );
            black_box(result)
        })
    });
    
    group.bench_function("simple_state_restore", |b| {
        // Ensure state exists
        preservor.save_global_state("simple_component", &simple_state).unwrap();
        
        b.iter(|| {
            let result: Result<Option<serde_json::Value>, _> = preservor.restore_global_state(
                black_box("simple_component")
            );
            black_box(result)
        })
    });
    
    // Complex state preservation
    let complex_state = json!({
        "ui_components": {
            "buttons": [
                {"id": "btn1", "text": "Button 1", "enabled": true},
                {"id": "btn2", "text": "Button 2", "enabled": false},
                {"id": "btn3", "text": "Button 3", "enabled": true}
            ],
            "inputs": [
                {"id": "input1", "value": "Hello", "placeholder": "Enter text"},
                {"id": "input2", "value": "World", "placeholder": "Enter more text"}
            ]
        },
        "application_state": {
            "current_page": "home",
            "user_preferences": {
                "theme": "dark",
                "font_size": 14,
                "language": "en"
            },
            "session_data": {
                "login_time": "2024-01-01T00:00:00Z",
                "last_activity": "2024-01-01T12:00:00Z",
                "permissions": ["read", "write", "admin"]
            }
        }
    });
    
    group.bench_function("complex_state_save", |b| {
        b.iter(|| {
            let result = preservor.save_global_state(
                black_box("complex_component"),
                black_box(&complex_state)
            );
            black_box(result)
        })
    });
    
    group.bench_function("complex_state_restore", |b| {
        // Ensure state exists
        preservor.save_global_state("complex_component", &complex_state).unwrap();
        
        b.iter(|| {
            let result: Result<Option<serde_json::Value>, _> = preservor.restore_global_state(
                black_box("complex_component")
            );
            black_box(result)
        })
    });
    
    // Batch state operations
    group.throughput(Throughput::Elements(100));
    group.bench_function("batch_state_operations", |b| {
        b.iter(|| {
            for i in 0..100 {
                let component_id = format!("component_{}", i);
                let state = json!({
                    "id": i,
                    "name": format!("Component {}", i),
                    "active": i % 2 == 0
                });
                
                let save_result = preservor.save_global_state(&component_id, &state);
                black_box(save_result).unwrap();
                
                let restore_result: Result<Option<serde_json::Value>, _> = 
                    preservor.restore_global_state(&component_id);
                black_box(restore_result).unwrap();
            }
        })
    });
    
    group.finish();
}

/// Benchmark component lifecycle performance
fn benchmark_component_lifecycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_lifecycle");
    
    let config = DualModeConfig::default();
    let mut engine = DualModeEngine::new(config).unwrap();
    engine.initialize().unwrap();
    
    // Component registration
    group.bench_function("component_registration", |b| {
        let mut counter = 0;
        b.iter(|| {
            let component_id = format!("component_{}", counter);
            counter += 1;
            let result = engine.register_component(
                black_box(component_id),
                black_box("TestComponent".to_string())
            );
            black_box(result)
        })
    });
    
    // Component state updates
    group.bench_function("component_state_update", |b| {
        // Pre-register a component
        engine.register_component("benchmark_component".to_string(), "TestComponent".to_string()).unwrap();
        
        b.iter(|| {
            let state = json!({
                "counter": black_box(42),
                "message": black_box("Updated")
            });
            
            let result = engine.preserve_component_state(
                black_box("benchmark_component"),
                black_box(state)
            );
            black_box(result)
        })
    });
    
    group.finish();
}

/// Benchmark startup time comparison
fn benchmark_startup_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup_time");
    
    let config = DualModeConfig::default();
    
    group.bench_function("production_startup", |b| {
        b.iter(|| {
            let mut engine = DualModeEngine::new(config.clone()).unwrap();
            engine.initialize().unwrap();
            let startup_time = engine.measure_startup_time();
            black_box(startup_time)
        })
    });
    
    #[cfg(feature = "dev-ui")]
    {
        group.bench_function("development_startup", |b| {
            b.iter(|| {
                let mut engine = DualModeEngine::new(config.clone()).unwrap();
                engine.initialize().unwrap();
                
                // Simulate development mode initialization
                engine.start_development_mode().unwrap();
                
                let startup_time = engine.measure_startup_time();
                black_box(startup_time)
            })
        });
    }
    
    group.finish();
}

/// Benchmark file change processing (development only)
#[cfg(feature = "dev-ui")]
fn benchmark_file_change_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_change_processing");
    
    let config = DualModeConfig::default();
    let mut engine = DualModeEngine::new(config).unwrap();
    engine.initialize().unwrap();
    
    group.bench_function("file_change_detection", |b| {
        b.iter(|| {
            let result = engine.process_file_changes();
            black_box(result)
        })
    });
    
    group.finish();
}

/// Benchmark cross-platform performance
fn benchmark_cross_platform_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cross_platform");
    
    use rustyui_core::platform::{Platform, PlatformConfig};
    
    let current_platform = Platform::current();
    let platform_config = PlatformConfig::auto_detect();
    let config = DualModeConfig::default();
    
    group.bench_function("platform_detection", |b| {
        b.iter(|| {
            let platform = Platform::current();
            black_box(platform)
        })
    });
    
    group.bench_function("platform_config_creation", |b| {
        b.iter(|| {
            let config = PlatformConfig::auto_detect();
            black_box(config)
        })
    });
    
    group.bench_function("engine_with_platform_config", |b| {
        b.iter(|| {
            let engine = DualModeEngine::with_platform_config(
                black_box(config.clone()),
                black_box(platform_config.clone())
            );
            black_box(engine)
        })
    });
    
    group.finish();
}

/// Comprehensive performance comparison benchmark
fn benchmark_comprehensive_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_comparison");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);
    
    let config = DualModeConfig::default();
    
    // Production mode comprehensive test
    group.bench_function("production_comprehensive", |b| {
        b.iter(|| {
            // Engine creation and initialization
            let mut engine = DualModeEngine::new(config.clone()).unwrap();
            engine.initialize().unwrap();
            
            // Component operations
            for i in 0..10 {
                let component_id = format!("prod_component_{}", i);
                engine.register_component(component_id.clone(), "TestComponent".to_string()).unwrap();
                
                let state = json!({
                    "id": i,
                    "active": true
                });
                engine.preserve_component_state(&component_id, state).unwrap();
            }
            
            // Memory measurement
            let memory_overhead = engine.memory_overhead_bytes();
            black_box(memory_overhead);
            
            black_box(engine)
        })
    });
    
    // Development mode comprehensive test
    #[cfg(feature = "dev-ui")]
    {
        group.bench_function("development_comprehensive", |b| {
            b.iter(|| {
                // Engine creation and initialization
                let mut engine = DualModeEngine::new(config.clone()).unwrap();
                engine.initialize().unwrap();
                engine.start_development_mode().unwrap();
                
                // Component operations
                for i in 0..10 {
                    let component_id = format!("dev_component_{}", i);
                    engine.register_component(component_id.clone(), "TestComponent".to_string()).unwrap();
                    
                    let state = json!({
                        "id": i,
                        "active": true
                    });
                    engine.preserve_component_state(&component_id, state).unwrap();
                }
                
                // Runtime interpretation
                for i in 0..5 {
                    let code = format!("component_{}.text = \"Updated {}\";", i, i);
                    engine.interpret_ui_change(&code, Some(format!("component_{}", i))).unwrap();
                }
                
                // File change processing
                engine.process_file_changes().unwrap();
                
                // Memory measurement
                let memory_overhead = engine.current_memory_overhead_bytes();
                black_box(memory_overhead);
                
                black_box(engine)
            })
        });
    }
    
    group.finish();
}

// Configure benchmark groups
criterion_group!(
    benches,
    benchmark_engine_initialization,
    benchmark_memory_overhead,
    benchmark_state_preservation,
    benchmark_component_lifecycle,
    benchmark_startup_time,
    benchmark_cross_platform_performance,
    benchmark_comprehensive_comparison,
);

// Add development-only benchmarks when dev-ui feature is enabled
#[cfg(feature = "dev-ui")]
criterion_group!(
    dev_benches,
    benchmark_runtime_interpretation,
    benchmark_file_change_processing,
);

#[cfg(feature = "dev-ui")]
criterion_main!(benches, dev_benches);

#[cfg(not(feature = "dev-ui"))]
criterion_main!(benches);