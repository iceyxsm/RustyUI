//! Benchmarks for runtime interpretation performance
//! 
//! These benchmarks measure the performance of different interpretation strategies
//! and validate that development mode interpretation meets performance targets.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use rustyui_interpreter::{RuntimeInterpreter, InterpretationStrategy, InterpretationResult};
use std::time::Duration;

/// Benchmark different interpretation strategies
fn benchmark_interpretation_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpretation_strategies");
    group.throughput(Throughput::Elements(1));
    
    let strategies = vec![
        InterpretationStrategy::RhaiOnly,
        InterpretationStrategy::ASTOnly,
        InterpretationStrategy::Hybrid { rhai_threshold: 10, jit_threshold: 100 },
        InterpretationStrategy::JITPreferred,
    ];
    
    let simple_code = "let x = 42; x";
    let complex_code = r#"
        fn calculate_fibonacci(n) {
            if n <= 1 {
                n
            } else {
                calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2)
            }
        }
        calculate_fibonacci(10)
    "#;
    
    for strategy in strategies {
        let mut interpreter = RuntimeInterpreter::with_strategy(strategy.clone()).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("simple_code", format!("{:?}", strategy)),
            &simple_code,
            |b, code| {
                b.iter(|| {
                    let result = interpreter.interpret(black_box(code));
                    black_box(result)
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("complex_code", format!("{:?}", strategy)),
            &complex_code,
            |b, code| {
                b.iter(|| {
                    let result = interpreter.interpret(black_box(code));
                    black_box(result)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark UI-specific interpretation
fn benchmark_ui_interpretation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ui_interpretation");
    group.throughput(Throughput::Elements(1));
    
    let mut interpreter = RuntimeInterpreter::new().unwrap();
    
    // Simple UI updates
    let simple_ui_codes = vec![
        "button.text = \"Click me\";",
        "label.color = \"red\";",
        "input.value = \"Hello\";",
        "checkbox.checked = true;",
        "slider.value = 50;",
    ];
    
    for (i, code) in simple_ui_codes.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("simple_ui", i),
            code,
            |b, code| {
                b.iter(|| {
                    let result = interpreter.interpret(black_box(code));
                    black_box(result)
                })
            },
        );
    }
    
    // Complex UI logic
    let complex_ui_code = r#"
        if state.user_count > 100 {
            status_label.text = "High traffic: " + state.user_count + " users";
            status_label.color = "red";
            warning_icon.visible = true;
        } else if state.user_count > 50 {
            status_label.text = "Medium traffic: " + state.user_count + " users";
            status_label.color = "orange";
            warning_icon.visible = false;
        } else {
            status_label.text = "Low traffic: " + state.user_count + " users";
            status_label.color = "green";
            warning_icon.visible = false;
        }
        
        for i in 0..state.notifications.len() {
            let notification = state.notifications[i];
            let notification_item = create_notification_item(notification);
            notification_list.add_child(notification_item);
        }
    "#;
    
    group.bench_function("complex_ui_logic", |b| {
        b.iter(|| {
            let result = interpreter.interpret(black_box(complex_ui_code));
            black_box(result)
        })
    });
    
    group.finish();
}

/// Benchmark interpretation caching
fn benchmark_interpretation_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpretation_caching");
    
    let mut interpreter = RuntimeInterpreter::new().unwrap();
    let code = "let result = 0; for i in 0..100 { result += i; } result";
    
    // First execution (no cache)
    group.bench_function("first_execution", |b| {
        b.iter(|| {
            // Create new interpreter for each iteration to avoid caching
            let mut fresh_interpreter = RuntimeInterpreter::new().unwrap();
            let result = fresh_interpreter.interpret(black_box(code));
            black_box(result)
        })
    });
    
    // Cached execution
    group.bench_function("cached_execution", |b| {
        // Pre-warm the cache
        interpreter.interpret(code).unwrap();
        
        b.iter(|| {
            let result = interpreter.interpret(black_box(code));
            black_box(result)
        })
    });
    
    group.finish();
}

/// Benchmark batch interpretation
fn benchmark_batch_interpretation(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_interpretation");
    
    let mut interpreter = RuntimeInterpreter::new().unwrap();
    
    // Generate batch of UI updates
    let batch_sizes = vec![1, 5, 10, 25, 50, 100];
    
    for batch_size in batch_sizes {
        group.throughput(Throughput::Elements(batch_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("ui_updates", batch_size),
            &batch_size,
            |b, &size| {
                b.iter(|| {
                    for i in 0..size {
                        let code = format!("component_{}.text = \"Update {}\";", i, i);
                        let result = interpreter.interpret(black_box(&code));
                        black_box(result).unwrap();
                    }
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark error handling performance
fn benchmark_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    
    let mut interpreter = RuntimeInterpreter::new().unwrap();
    
    // Valid code (baseline)
    group.bench_function("valid_code", |b| {
        b.iter(|| {
            let result = interpreter.interpret(black_box("let x = 42; x"));
            black_box(result)
        })
    });
    
    // Syntax errors
    group.bench_function("syntax_error", |b| {
        b.iter(|| {
            let result = interpreter.interpret(black_box("let x = 42 +;"));
            black_box(result)
        })
    });
    
    // Runtime errors
    group.bench_function("runtime_error", |b| {
        b.iter(|| {
            let result = interpreter.interpret(black_box("undefined_variable"));
            black_box(result)
        })
    });
    
    // Recovery after error
    group.bench_function("recovery_after_error", |b| {
        b.iter(|| {
            // Cause an error
            let _error_result = interpreter.interpret("invalid syntax");
            
            // Then execute valid code
            let result = interpreter.interpret(black_box("let y = 24; y"));
            black_box(result)
        })
    });
    
    group.finish();
}

/// Benchmark memory usage during interpretation
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    // Small script memory usage
    group.bench_function("small_script_memory", |b| {
        b.iter(|| {
            let mut interpreter = RuntimeInterpreter::new().unwrap();
            let result = interpreter.interpret(black_box("let x = 42; x"));
            black_box(result).unwrap();
            
            // Measure memory usage (simplified)
            let memory_usage = std::mem::size_of_val(&interpreter);
            black_box(memory_usage)
        })
    });
    
    // Large script memory usage
    let large_script = "let sum = 0; ".repeat(1000) + "sum";
    
    group.bench_function("large_script_memory", |b| {
        b.iter(|| {
            let mut interpreter = RuntimeInterpreter::new().unwrap();
            let result = interpreter.interpret(black_box(&large_script));
            black_box(result).unwrap();
            
            // Measure memory usage (simplified)
            let memory_usage = std::mem::size_of_val(&interpreter);
            black_box(memory_usage)
        })
    });
    
    group.finish();
}

/// Benchmark concurrent interpretation
fn benchmark_concurrent_interpretation(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_interpretation");
    
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let interpreter = Arc::new(Mutex::new(RuntimeInterpreter::new().unwrap()));
    
    // Sequential interpretation (baseline)
    group.bench_function("sequential", |b| {
        b.iter(|| {
            let mut interp = interpreter.lock().unwrap();
            for i in 0..10 {
                let code = format!("let x_{} = {}; x_{}", i, i, i);
                let result = interp.interpret(black_box(&code));
                black_box(result).unwrap();
            }
        })
    });
    
    // Simulated concurrent interpretation
    group.bench_function("simulated_concurrent", |b| {
        b.iter(|| {
            let mut handles = vec![];
            
            for i in 0..5 {
                let interpreter_clone = Arc::clone(&interpreter);
                let handle = thread::spawn(move || {
                    let mut interp = interpreter_clone.lock().unwrap();
                    let code = format!("let x_{} = {}; x_{}", i, i, i);
                    interp.interpret(&code)
                });
                handles.push(handle);
            }
            
            for handle in handles {
                let result = handle.join().unwrap();
                black_box(result).unwrap();
            }
        })
    });
    
    group.finish();
}

/// Comprehensive interpretation performance benchmark
fn benchmark_comprehensive_interpretation(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_interpretation");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);
    
    let mut interpreter = RuntimeInterpreter::new().unwrap();
    
    group.bench_function("comprehensive_workflow", |b| {
        b.iter(|| {
            // Simulate a complete interpretation workflow
            
            // 1. Initialize component state
            let init_code = r#"
                let state = #{
                    counter: 0,
                    message: "Hello",
                    items: []
                };
            "#;
            interpreter.interpret(black_box(init_code)).unwrap();
            
            // 2. Process user interactions
            for i in 0..5 {
                let interaction_code = format!(r#"
                    state.counter += 1;
                    state.items.push("Item {}");
                    if state.counter > 3 {{
                        state.message = "Counter is high: " + state.counter;
                    }}
                "#, i);
                interpreter.interpret(black_box(&interaction_code)).unwrap();
            }
            
            // 3. Update UI components
            let ui_update_code = r#"
                button.text = "Count: " + state.counter;
                label.text = state.message;
                list.items = state.items;
                
                if state.counter > 3 {
                    button.color = "red";
                } else {
                    button.color = "blue";
                }
            "#;
            let result = interpreter.interpret(black_box(ui_update_code));
            black_box(result).unwrap();
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_interpretation_strategies,
    benchmark_ui_interpretation,
    benchmark_interpretation_caching,
    benchmark_batch_interpretation,
    benchmark_error_handling,
    benchmark_memory_usage,
    benchmark_concurrent_interpretation,
    benchmark_comprehensive_interpretation,
);

criterion_main!(benches);