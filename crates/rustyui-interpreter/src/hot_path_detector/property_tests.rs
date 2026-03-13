//! Property-based tests for hot path detection
//! 
//! This module implements property-based testing for the hot path detection system,
//! validating hot function detection, priority scoring, and optimization candidate identification.

#[cfg(test)]
mod tests {
    use crate::profiling::ProfilingInfrastructure;
    use crate::hot_path_detector::{HotPathDetector, HotPathConfig};
    use crate::tiered_compilation::CompilationTier;
    use proptest::prelude::*;
    use proptest::test_runner::Config as ProptestConfig;
    use std::sync::Arc;
    use std::time::Duration;

    // Strategy generators for property testing
    
    /// Generate valid function IDs
    fn function_id_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z_][a-zA-Z0-9_]*"
    }
    
    /// Generate execution counts for testing hot function detection
    fn execution_count_strategy() -> impl Strategy<Value = u64> {
        0u64..=2000u64
    }
    
    /// Generate execution times in microseconds
    fn execution_time_strategy() -> impl Strategy<Value = Duration> {
        (1u64..=50000u64).prop_map(Duration::from_micros)
    }
    
    /// Generate hot path configuration
    fn hot_path_config_strategy() -> impl Strategy<Value = HotPathConfig> {
        (
            1u64..=100u64,  // min_execution_count
            1.0f64..=1000.0f64,  // min_priority_score
            1u64..=300u64,  // cache_ttl (seconds)
            1u64..=60u64,   // reanalysis_interval (seconds)
            1u64..=50u64,   // loop_hot_threshold
            1u64..=50u64,   // call_site_hot_threshold
            1.0f64..=100.0f64,  // min_inline_benefit_score
        ).prop_map(|(min_execution_count, min_priority_score, cache_ttl_secs, reanalysis_interval_secs, loop_hot_threshold, call_site_hot_threshold, min_inline_benefit_score)| {
            HotPathConfig {
                min_execution_count,
                min_priority_score,
                cache_ttl: Duration::from_secs(cache_ttl_secs),
                reanalysis_interval: Duration::from_secs(reanalysis_interval_secs),
                loop_hot_threshold,
                call_site_hot_threshold,
                min_inline_benefit_score,
            }
        })
    }
    
    /// Generate compilation tiers
    fn compilation_tier_strategy() -> impl Strategy<Value = CompilationTier> {
        prop_oneof![
            Just(CompilationTier::Interpreter),
            Just(CompilationTier::QuickJIT),
            Just(CompilationTier::OptimizedJIT),
            Just(CompilationTier::AggressiveJIT),
        ]
    }
    
    /// Generate function profile data for testing
    fn function_profile_strategy() -> impl Strategy<Value = (String, u64, Duration)> {
        (
            function_id_strategy(),
            execution_count_strategy(),
            execution_time_strategy()
        )
    }

    // Property Tests Implementation

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        /// **Property 9: Hot Function Detection**
        /// **Validates: Requirements US-2.2, US-2.3, FR-3.1**
        /// 
        /// For any function with execution count >= hot threshold and priority score >= minimum priority, 
        /// the hot path detector should identify it as a hot function.
        #[test]
        fn property_hot_function_detection(
            config in hot_path_config_strategy(),
            function_profiles in prop::collection::vec(function_profile_strategy(), 1..=20)
        ) {
            // Create profiling infrastructure and hot path detector
            let profiling = Arc::new(ProfilingInfrastructure::with_defaults());
            let detector = HotPathDetector::new(profiling.clone(), config.clone());
            
            // Populate profiling data with function executions
            let mut expected_hot_functions = Vec::new();
            
            for (function_id, execution_count, avg_execution_time) in function_profiles.iter() {
                // Record executions to build profile data
                for _ in 0..*execution_count {
                    profiling.record_execution(function_id, *avg_execution_time);
                }
                
                // Calculate priority score using the same formula as the detector
                let priority_score = detector.calculate_priority(
                    function_id, 
                    *execution_count, 
                    *avg_execution_time
                );
                
                // Determine if this function should be detected as hot
                if *execution_count >= config.min_execution_count && 
                   priority_score >= config.min_priority_score {
                    expected_hot_functions.push(function_id.clone());
                }
            }
            
            // Detect hot functions
            let detected_hot_functions = detector.detect_hot_functions();
            let detected_function_ids: std::collections::HashSet<String> = detected_hot_functions
                .iter()
                .map(|hf| hf.function_id.clone())
                .collect();
            
            // Verify that all expected hot functions are detected
            for expected_function_id in expected_hot_functions.iter() {
                prop_assert!(
                    detected_function_ids.contains(expected_function_id),
                    "Function '{}' should be detected as hot (execution_count >= {}, priority_score >= {})",
                    expected_function_id,
                    config.min_execution_count,
                    config.min_priority_score
                );
            }
            
            // Verify that detected hot functions meet the criteria
            for hot_function in detected_hot_functions.iter() {
                prop_assert!(
                    hot_function.execution_count >= config.min_execution_count,
                    "Detected hot function '{}' should have execution_count >= {} (actual: {})",
                    hot_function.function_id,
                    config.min_execution_count,
                    hot_function.execution_count
                );
                
                prop_assert!(
                    hot_function.priority_score >= config.min_priority_score,
                    "Detected hot function '{}' should have priority_score >= {} (actual: {})",
                    hot_function.function_id,
                    config.min_priority_score,
                    hot_function.priority_score
                );
            }
            
            // Verify that functions not meeting criteria are not detected as hot
            for (function_id, execution_count, avg_execution_time) in function_profiles.iter() {
                let priority_score = detector.calculate_priority(
                    function_id, 
                    *execution_count, 
                    *avg_execution_time
                );
                
                if *execution_count < config.min_execution_count || 
                   priority_score < config.min_priority_score {
                    prop_assert!(
                        !detected_function_ids.contains(function_id),
                        "Function '{}' should NOT be detected as hot (execution_count: {}, priority_score: {}, thresholds: {} and {})",
                        function_id,
                        execution_count,
                        priority_score,
                        config.min_execution_count,
                        config.min_priority_score
                    );
                }
            }
        }
    }
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        /// **Property 9 Extended: Hot Function Detection with Tier Consideration**
        /// **Validates: Requirements US-2.2, US-2.3, FR-3.1**
        /// 
        /// Verifies that hot function detection correctly considers current compilation tier
        /// and recommends appropriate target tiers based on execution patterns.
        #[test]
        fn property_hot_function_detection_with_tiers(
            config in hot_path_config_strategy(),
            function_data in prop::collection::vec(
                (function_profile_strategy(), compilation_tier_strategy()), 
                1..=15
            )
        ) {
            // Create profiling infrastructure and hot path detector
            let profiling = Arc::new(ProfilingInfrastructure::with_defaults());
            let detector = HotPathDetector::new(profiling.clone(), config.clone());
            
            // Track functions that should be optimization candidates
            let mut expected_candidates = Vec::new();
            
            for ((function_id, execution_count, avg_execution_time), current_tier) in function_data.iter() {
                // Record executions to build profile data
                for _ in 0..*execution_count {
                    profiling.record_execution(function_id, *avg_execution_time);
                }
                
                // Check if function is an optimization candidate
                let is_candidate = detector.is_optimization_candidate(function_id, *current_tier);
                
                // Calculate expected candidacy based on criteria
                let priority_score = detector.calculate_priority(
                    function_id, 
                    *execution_count, 
                    *avg_execution_time
                );
                
                let should_be_candidate = *execution_count >= config.min_execution_count && 
                                        priority_score >= config.min_priority_score &&
                                        *current_tier != CompilationTier::AggressiveJIT; // Can't optimize beyond highest tier
                
                prop_assert_eq!(
                    is_candidate, 
                    should_be_candidate,
                    "Function '{}' optimization candidacy mismatch: tier={:?}, execution_count={}, priority_score={:.2}",
                    function_id,
                    current_tier,
                    execution_count,
                    priority_score
                );
                
                if should_be_candidate {
                    expected_candidates.push(function_id.clone());
                }
            }
            
            // Detect hot functions and verify they align with optimization candidates
            let detected_hot_functions = detector.detect_hot_functions();
            
            for hot_function in detected_hot_functions.iter() {
                // Hot functions should have a recommended tier higher than interpreter
                prop_assert!(
                    hot_function.recommended_tier > CompilationTier::Interpreter,
                    "Hot function '{}' should have recommended tier higher than Interpreter (actual: {:?})",
                    hot_function.function_id,
                    hot_function.recommended_tier
                );
                
                // Priority score should be positive for hot functions
                prop_assert!(
                    hot_function.priority_score > 0.0,
                    "Hot function '{}' should have positive priority score (actual: {})",
                    hot_function.function_id,
                    hot_function.priority_score
                );
            }
        }
    }
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        /// **Property 10: Hot Path Compilation Priority**
        /// **Validates: Requirements US-2.3, FR-3.3**
        /// 
        /// For any two functions in the recompilation queue where one has a higher priority score, 
        /// the higher priority function should be compiled first (or within a small time window for batching).
        #[test]
        fn property_hot_path_compilation_priority(
            config in hot_path_config_strategy(),
            function_profiles in prop::collection::vec(function_profile_strategy(), 2..=10)
        ) {
            // Create profiling infrastructure and hot path detector
            let profiling = Arc::new(ProfilingInfrastructure::with_defaults());
            let detector = HotPathDetector::new(profiling.clone(), config.clone());
            
            // Populate profiling data and calculate priority scores
            let mut functions_with_priorities = Vec::new();
            
            for (function_id, execution_count, avg_execution_time) in function_profiles.iter() {
                // Record executions to build profile data
                for _ in 0..*execution_count {
                    profiling.record_execution(function_id, *avg_execution_time);
                }
                
                // Calculate priority score
                let priority_score = detector.calculate_priority(
                    function_id, 
                    *execution_count, 
                    *avg_execution_time
                );
                
                functions_with_priorities.push((function_id.clone(), priority_score));
            }
            
            // Sort functions by priority score (highest first) - this is the expected order
            functions_with_priorities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            // Detect hot functions using the detector
            let detected_hot_functions = detector.detect_hot_functions();
            
            // Verify that detected hot functions are ordered by priority score (highest first)
            for i in 1..detected_hot_functions.len() {
                let prev_priority = detected_hot_functions[i - 1].priority_score;
                let curr_priority = detected_hot_functions[i].priority_score;
                
                prop_assert!(
                    prev_priority >= curr_priority,
                    "Hot functions should be ordered by priority score (highest first). \
                     Function '{}' (priority: {:.2}) should come before function '{}' (priority: {:.2})",
                    detected_hot_functions[i - 1].function_id,
                    prev_priority,
                    detected_hot_functions[i].function_id,
                    curr_priority
                );
            }
            
            // Verify that functions with higher priority scores appear earlier in the results
            // (within the subset of functions that meet the hot function criteria)
            let hot_function_ids: std::collections::HashMap<String, f64> = detected_hot_functions
                .iter()
                .map(|hf| (hf.function_id.clone(), hf.priority_score))
                .collect();
            
            // For any two functions that are both detected as hot, verify priority ordering
            for i in 0..functions_with_priorities.len() {
                for j in (i + 1)..functions_with_priorities.len() {
                    let (func_a, priority_a) = &functions_with_priorities[i];
                    let (func_b, priority_b) = &functions_with_priorities[j];
                    
                    // If both functions are detected as hot
                    if let (Some(detected_priority_a), Some(detected_priority_b)) = 
                        (hot_function_ids.get(func_a), hot_function_ids.get(func_b)) {
                        
                        // Function A should have higher or equal priority than function B
                        // (since we sorted by priority descending)
                        prop_assert!(
                            detected_priority_a >= detected_priority_b,
                            "Priority ordering violation: function '{}' (priority: {:.2}) should have \
                             higher priority than function '{}' (priority: {:.2}) in hot function results",
                            func_a,
                            detected_priority_a,
                            func_b,
                            detected_priority_b
                        );
                        
                        // Verify the priorities match our calculated values
                        prop_assert!(
                            (detected_priority_a - priority_a).abs() < 0.001,
                            "Priority score mismatch for function '{}': expected {:.2}, got {:.2}",
                            func_a,
                            priority_a,
                            detected_priority_a
                        );
                        
                        prop_assert!(
                            (detected_priority_b - priority_b).abs() < 0.001,
                            "Priority score mismatch for function '{}': expected {:.2}, got {:.2}",
                            func_b,
                            priority_b,
                            detected_priority_b
                        );
                    }
                }
            }
            
            // Additional verification: ensure priority scores are consistent with execution patterns
            for hot_function in detected_hot_functions.iter() {
                // Priority score should be positive for hot functions
                prop_assert!(
                    hot_function.priority_score > 0.0,
                    "Hot function '{}' should have positive priority score (actual: {:.2})",
                    hot_function.function_id,
                    hot_function.priority_score
                );
                
                // Functions with higher execution counts should generally have higher priorities
                // (when execution times are similar)
                let execution_count = hot_function.execution_count;
                prop_assert!(
                    execution_count > 0,
                    "Hot function '{}' should have positive execution count (actual: {})",
                    hot_function.function_id,
                    execution_count
                );
            }
        }
    }
}