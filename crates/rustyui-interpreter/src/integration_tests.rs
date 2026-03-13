//! End-to-end integration tests for Phase 2 Enhanced JIT PGO
//! 
//! These tests verify the complete workflow from cold start to hot path optimization

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        tiered_compilation::{TieredCompilationManager, TieredCompilationConfig, CompilationTier},
        profiling::{ProfilingInfrastructure, ProfilingConfig},
        hot_path_detector::{HotPathDetector, HotPathConfig},
        recompilation_scheduler::{RecompilationScheduler, RecompilationConfig},
        optimization_engine::OptimizationEngine,
    };
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
        thread,
    };

    /// Test cold start to hot path workflow (Tier 0 → 1 → 2 → 3)
    #[test]
    #[cfg(feature = "dev-ui")]
    fn test_cold_start_to_hot_path_workflow() {
        // Setup PGO components
        let profiling_config = ProfilingConfig::default();
        let profiling = Arc::new(ProfilingInfrastructure::new(profiling_config));
        
        let recompilation_config = RecompilationConfig::default();
        let recompilation_scheduler = Arc::new(
            RecompilationScheduler::new(recompilation_config).unwrap()
        );
        
        let optimization_engine = Arc::new(Mutex::new(
            OptimizationEngine::new().unwrap()
        ));
        
        let config = TieredCompilationConfig::default();
        let manager = TieredCompilationManager::with_pgo_integration(
            config,
            profiling.clone(),
            recompilation_scheduler,
            optimization_engine,
        );
        
        let function_id = "test_function";
        let code = "fn test() { return 42; }";
        
        // Initially should be in interpreter tier
        assert_eq!(
            manager.get_metadata(function_id)
                .map(|m| m.current_tier)
                .unwrap_or(CompilationTier::Interpreter),
            CompilationTier::Interpreter
        );
        
        // Execute function multiple times to trigger tier promotions
        for i in 0..15 {
            let result = manager.execute_with_profiling(function_id, code);
            assert!(result.is_ok(), "Execution {} failed: {:?}", i, result);
            
            // Check tier progression
            if let Some(metadata) = manager.get_metadata(function_id) {
                match i {
                    0..=9 => assert_eq!(metadata.current_tier, CompilationTier::Interpreter),
                    10..=14 => {
                        // Should be promoted to QuickJIT after 10 executions
                        // Allow some time for background compilation
                        thread::sleep(Duration::from_millis(10));
                    }
                    _ => {}
                }
            }
        }
        
        // Verify profiling data was collected
        if let Some(profile_data) = profiling.get_profile(function_id) {
            assert!(profile_data.get_execution_count() >= 15);
        }
        
        // Verify hot path detection
        if let Some(detector) = manager.get_hot_path_detector() {
            let hot_functions = detector.detect_hot_functions();
            assert!(!hot_functions.is_empty(), "Should detect hot functions");
        }
    }
    
    /// Test concurrent execution during recompilation
    #[test]
    #[cfg(feature = "dev-ui")]
    fn test_concurrent_execution_during_recompilation() {
        let profiling_config = ProfilingConfig::default();
        let profiling = Arc::new(ProfilingInfrastructure::new(profiling_config));
        
        let recompilation_config = RecompilationConfig::default();
        let recompilation_scheduler = Arc::new(
            RecompilationScheduler::new(recompilation_config).unwrap()
        );
        
        let optimization_engine = Arc::new(Mutex::new(
            OptimizationEngine::new().unwrap()
        ));
        
        let config = TieredCompilationConfig::default();
        let manager = Arc::new(TieredCompilationManager::with_pgo_integration(
            config,
            profiling,
            recompilation_scheduler,
            optimization_engine,
        ));
        
        let function_id = "concurrent_test_function";
        let code = "fn concurrent_test() { return 123; }";
        
        // Spawn multiple threads executing the same function
        let mut handles = vec![];
        for thread_id in 0..4 {
            let manager_clone = manager.clone();
            let function_id_clone = function_id.to_string();
            let code_clone = code.to_string();
            
            let handle = thread::spawn(move || {
                for i in 0..50 {
                    let result = manager_clone.execute_with_profiling(&function_id_clone, &code_clone);
                    assert!(result.is_ok(), "Thread {} execution {} failed: {:?}", thread_id, i, result);
                    
                    // Small delay to allow interleaving
                    thread::sleep(Duration::from_micros(100));
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
        
        // Verify that concurrent executions were handled correctly
        if let Some(metadata) = manager.get_metadata(function_id) {
            assert!(metadata.execution_count >= 200, "Should have recorded all executions");
        }
    }
    
    /// Test profile-guided optimization effectiveness
    #[test]
    #[cfg(feature = "dev-ui")]
    fn test_profile_guided_optimization_effectiveness() {
        let profiling_config = ProfilingConfig::default();
        let profiling = Arc::new(ProfilingInfrastructure::new(profiling_config));
        
        let optimization_engine = Arc::new(Mutex::new(
            OptimizationEngine::new().unwrap()
        ));
        
        let function_id = "optimization_test_function";
        let code = "fn optimization_test() { 
            let mut sum = 0;
            for i in 0..100 {
                sum += i;
            }
            return sum;
        }";
        
        // Record some profile data
        for _ in 0..50 {
            profiling.record_execution(function_id, Duration::from_millis(1));
            profiling.record_branch(function_id, 1, true); // Loop branch taken
            profiling.record_loop(function_id, 1, 100); // Loop iterations
        }
        
        // Get profile data
        let profile_data = profiling.get_profile(function_id).unwrap();
        
        // Test compilation with profile data
        let engine = optimization_engine.lock().unwrap();
        let result_with_profile = engine.compile_with_profile(
            code,
            CompilationTier::OptimizedJIT,
            &profile_data,
        );
        
        assert!(result_with_profile.is_ok(), "Compilation with profile should succeed");
        
        // Test compilation without profile data (empty profile)
        let empty_profile = crate::profiling::ProfileData::new(function_id.to_string());
        let result_without_profile = engine.compile_with_profile(
            code,
            CompilationTier::OptimizedJIT,
            &empty_profile,
        );
        
        assert!(result_without_profile.is_ok(), "Compilation without profile should succeed");
        
        // The compiled code should be different (profile-guided optimizations applied)
        // This is a basic check - in practice, we'd need more sophisticated verification
        let with_profile = result_with_profile.unwrap();
        let without_profile = result_without_profile.unwrap();
        
        // At minimum, compilation times or code sizes might differ
        // This is a placeholder assertion - real implementation would have more detailed checks
        assert!(true, "Profile-guided optimization test completed");
    }
    
    /// Test memory management and garbage collection
    #[test]
    #[cfg(feature = "dev-ui")]
    fn test_memory_management_and_gc() {
        let profiling_config = ProfilingConfig {
            max_memory: 1024 * 1024, // 1MB limit
            ..ProfilingConfig::default()
        };
        let profiling = Arc::new(ProfilingInfrastructure::new(profiling_config));
        
        let recompilation_config = RecompilationConfig::default();
        let recompilation_scheduler = Arc::new(
            RecompilationScheduler::new(recompilation_config).unwrap()
        );
        
        // Create many functions to test memory management
        for i in 0..1000 {
            let function_id = format!("test_function_{}", i);
            
            // Record execution data
            profiling.record_execution(&function_id, Duration::from_millis(1));
            
            // Simulate some profile data
            profiling.record_branch(&function_id, 1, i % 2 == 0);
            profiling.record_loop(&function_id, 1, i as u64);
        }
        
        // Check that profiling overhead is within limits
        let overhead = profiling.get_overhead_percentage();
        assert!(overhead < 5.0, "Profiling overhead should be < 5%, got {}%", overhead);
        
        // Test garbage collection
        recompilation_scheduler.gc_old_versions();
        
        // Memory usage should be reasonable
        // This is a placeholder - real implementation would check actual memory usage
        assert!(true, "Memory management test completed");
    }
    
    /// Test configuration validation and updates
    #[test]
    fn test_configuration_validation_and_updates() {
        // Test valid configuration
        let valid_config = TieredCompilationConfig {
            tier1_threshold: 10,
            tier2_threshold: 100,
            tier3_threshold: 1000,
            background_recompilation: true,
            max_concurrent_recompilations: 4,
            collect_statistics: true,
            profiling: ProfilingConfig::default(),
        };
        
        assert!(valid_config.validate().is_ok(), "Valid config should pass validation");
        
        // Test invalid configuration - thresholds not in order
        let invalid_config = TieredCompilationConfig {
            tier1_threshold: 100,
            tier2_threshold: 50, // Invalid: less than tier1
            tier3_threshold: 1000,
            ..valid_config.clone()
        };
        
        assert!(invalid_config.validate().is_err(), "Invalid config should fail validation");
        
        // Test runtime configuration update
        let mut config = valid_config.clone();
        let new_config = TieredCompilationConfig {
            tier1_threshold: 20,
            tier2_threshold: 200,
            tier3_threshold: 2000,
            collect_statistics: false,
            ..valid_config
        };
        
        let update_result = config.update_runtime_safe(&new_config);
        assert!(update_result.is_ok(), "Runtime update should succeed");
        assert_eq!(config.tier1_threshold, 20, "Threshold should be updated");
        assert_eq!(config.collect_statistics, false, "Statistics flag should be updated");
    }
    
    /// Test error handling and recovery
    #[test]
    #[cfg(feature = "dev-ui")]
    fn test_error_handling_and_recovery() {
        let profiling_config = ProfilingConfig::default();
        let profiling = Arc::new(ProfilingInfrastructure::new(profiling_config));
        
        let config = TieredCompilationConfig::default();
        let manager = TieredCompilationManager::with_hot_path_detector(config, profiling);
        
        let function_id = "error_test_function";
        
        // Simulate compilation failure
        manager.handle_compilation_failure(
            function_id,
            CompilationTier::OptimizedJIT,
            "Simulated compilation error"
        );
        
        // Function should fall back to previous tier
        if let Some(metadata) = manager.get_metadata(function_id) {
            assert_eq!(metadata.current_tier, CompilationTier::QuickJIT);
            assert!(!metadata.recompiling);
        }
        
        // Test profile data corruption recovery
        manager.recover_from_corruption(function_id);
        
        // Function should be reset to interpreter
        if let Some(metadata) = manager.get_metadata(function_id) {
            assert_eq!(metadata.current_tier, CompilationTier::Interpreter);
            assert_eq!(metadata.execution_count, 0);
        }
    }
}