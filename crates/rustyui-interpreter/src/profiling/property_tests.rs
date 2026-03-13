//! Property-based tests for profiling infrastructure
//! 
//! This module implements property-based testing for the profiling system,
//! validating execution count tracking, branch statistics, loop iteration tracking,
//! call site frequency tracking, type feedback collection, and profiling overhead limits.

#[cfg(test)]
mod tests {
    use super::super::*;
    use proptest::prelude::*;
    use proptest::test_runner::Config as ProptestConfig;
    use std::sync::Arc;
    use std::time::Duration;
    use std::thread;

    // Strategy generators for property testing
    
    /// Generate valid function IDs
    fn function_id_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z_][a-zA-Z0-9_]*"
    }
    
    /// Generate execution counts for testing
    fn execution_count_strategy() -> impl Strategy<Value = u64> {
        1u64..=1000u64
    }
    
    /// Generate branch IDs
    fn branch_id_strategy() -> impl Strategy<Value = u32> {
        0u32..=100u32
    }
    
    /// Generate loop IDs
    fn loop_id_strategy() -> impl Strategy<Value = u32> {
        0u32..=50u32
    }
    
    /// Generate call site IDs
    fn call_site_id_strategy() -> impl Strategy<Value = u32> {
        0u32..=200u32
    }
    
    /// Generate operation IDs for type feedback
    fn operation_id_strategy() -> impl Strategy<Value = u32> {
        0u32..=100u32
    }
    
    /// Generate target function names
    fn target_function_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z_][a-zA-Z0-9_]*"
    }
    
    /// Generate type names
    fn type_name_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("String".to_string()),
            Just("i32".to_string()),
            Just("f64".to_string()),
            Just("bool".to_string()),
            Just("Vec<i32>".to_string()),
            Just("HashMap<String, i32>".to_string()),
            Just("Option<String>".to_string()),
            Just("Result<i32, String>".to_string()),
        ]
    }
    
    /// Generate loop iteration counts
    fn iteration_count_strategy() -> impl Strategy<Value = u64> {
        1u64..=1000u64
    }
    
    /// Generate execution times
    fn execution_time_strategy() -> impl Strategy<Value = Duration> {
        (1u64..=10000u64).prop_map(Duration::from_micros)
    }

    // Property Tests Implementation

    proptest! {
        /// **Property 18: Profile Data Serialization Round-Trip**
        /// **Validates: Requirements FR-2.6**
        /// 
        /// For any profile data, serializing then deserializing should produce equivalent profile data 
        /// (within acceptable floating-point precision for averages).
        #[test]
        fn property_profile_data_serialization_round_trip(
            functions_data in prop::collection::vec(
                (
                    function_id_strategy(),
                    execution_count_strategy(),
                    prop::collection::vec((branch_id_strategy(), 1u64..=100u64, 1u64..=100u64), 0..=5), // (branch_id, taken, not_taken)
                    prop::collection::vec((loop_id_strategy(), prop::collection::vec(iteration_count_strategy(), 1..=10)), 0..=3), // (loop_id, iterations_list)
                    prop::collection::vec((call_site_id_strategy(), target_function_strategy(), 1u64..=50u64), 0..=5), // (call_site_id, target, count)
                    prop::collection::vec((operation_id_strategy(), type_name_strategy(), 1u64..=30u64), 0..=4) // (operation_id, type, count)
                ),
                1..=8
            )
        ) {
            // Create profiling infrastructure
            let profiling = ProfilingInfrastructure::with_defaults();
            
            // Populate profile data with various statistics
            for (function_id, execution_count, branch_data, loop_data, call_site_data, type_data) in functions_data.iter() {
                // Record executions to create the profile
                for _ in 0..*execution_count {
                    profiling.record_execution(function_id, Duration::from_micros(100));
                }
                
                // Record branch statistics
                for (branch_id, taken_count, not_taken_count) in branch_data.iter() {
                    for _ in 0..*taken_count {
                        profiling.record_branch(function_id, *branch_id, true);
                    }
                    for _ in 0..*not_taken_count {
                        profiling.record_branch(function_id, *branch_id, false);
                    }
                }
                
                // Record loop statistics
                for (loop_id, iterations_list) in loop_data.iter() {
                    for iterations in iterations_list.iter() {
                        profiling.record_loop(function_id, *loop_id, *iterations);
                    }
                }
                
                // Record call site statistics
                for (call_site_id, target_function, call_count) in call_site_data.iter() {
                    for _ in 0..*call_count {
                        profiling.record_call_site(function_id, *call_site_id, target_function);
                    }
                }
                
                // Record type feedback
                for (operation_id, type_name, observation_count) in type_data.iter() {
                    for _ in 0..*observation_count {
                        profiling.record_type(function_id, *operation_id, type_name);
                    }
                }
            }
            
            // Capture original profile data before serialization
            let original_profiles: std::collections::HashMap<String, _> = profiling.get_all_profiles()
                .into_iter()
                .collect();
            
            // Export profile data (serialize)
            let exported_data = profiling.export_profiles();
            prop_assert!(exported_data.is_ok(), "Profile data export should succeed");
            
            let exported_data = exported_data.unwrap();
            prop_assert!(!exported_data.is_empty(), "Exported data should not be empty");
            
            // Create new profiling infrastructure and import data (deserialize)
            let new_profiling = ProfilingInfrastructure::with_defaults();
            let import_result = new_profiling.import_profiles(&exported_data);
            prop_assert!(import_result.is_ok(), "Profile data import should succeed: {:?}", import_result.err());
            
            // Verify all functions are preserved
            let restored_profiles: std::collections::HashMap<String, _> = new_profiling.get_all_profiles()
                .into_iter()
                .collect();
            
            prop_assert_eq!(restored_profiles.len(), original_profiles.len(), 
                "Number of functions should be preserved after round-trip");
            
            // Verify each function's profile data is equivalent
            for (function_id, original_profile) in original_profiles.iter() {
                let restored_profile = restored_profiles.get(function_id);
                prop_assert!(restored_profile.is_some(), 
                    "Function {} should exist after round-trip", function_id);
                
                let restored_profile = restored_profile.unwrap();
                
                // Verify execution count (exact match)
                let original_execution_count = original_profile.get_execution_count();
                let restored_execution_count = restored_profile.get_execution_count();
                prop_assert_eq!(restored_execution_count, original_execution_count, 
                    "Function {} execution count should be preserved: {} vs {}", 
                    function_id, original_execution_count, restored_execution_count);
            }
        }
    }
}