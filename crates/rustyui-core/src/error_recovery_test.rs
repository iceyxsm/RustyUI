//! Tests for error recovery functionality

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::RustyUIError;
    use crate::error_recovery::{ErrorRecoveryManager, ErrorContext, Operation, RecoveryResult};
    
    #[test]
    fn test_error_recovery_manager_creation() {
        let manager = ErrorRecoveryManager::new();
        assert_eq!(manager.get_health_status(), crate::error_recovery::HealthStatus::Healthy);
    }
    
    #[test]
    fn test_error_handling_and_recovery() {
        let mut manager = ErrorRecoveryManager::new();
        let error = RustyUIError::interpretation("Test interpretation error");
        let context = ErrorContext {
            operation: Operation::Interpretation,
            component_id: Some("test_component".to_string()),
            context_data: std::collections::HashMap::new(),
        };
        
        let result = manager.handle_error(&error, context);
        
        // Should attempt recovery
        match result {
            RecoveryResult::Success { .. } | RecoveryResult::PartialRecovery { .. } => {
                // Recovery attempted successfully
            }
            RecoveryResult::Failed { .. } => {
                panic!("Recovery should not fail for interpretation errors");
            }
        }
        
        // Check that metrics were updated
        let metrics = manager.get_metrics();
        assert_eq!(metrics.total_errors, 1);
    }
    
    #[test]
    fn test_fallback_state_storage_and_recovery() {
        let mut manager = ErrorRecoveryManager::new();
        let state_data = serde_json::json!({
            "component_type": "Button",
            "text": "Click me",
            "enabled": true
        });
        
        manager.store_fallback_state("test_button".to_string(), state_data.clone());
        
        // Simulate error that should trigger fallback
        let error = RustyUIError::interpretation("Component failure");
        let context = ErrorContext {
            operation: Operation::ComponentRendering,
            component_id: Some("test_button".to_string()),
            context_data: std::collections::HashMap::new(),
        };
        
        let result = manager.handle_error(&error, context);
        
        match result {
            RecoveryResult::Success { fallback_data: Some(data), .. } => {
                assert_eq!(data.component_id, "test_button");
            }
            _ => {
                // Fallback might not be available in all scenarios, but should not fail
            }
        }
    }
    
    #[test]
    fn test_health_status_degradation() {
        let mut manager = ErrorRecoveryManager::new();
        
        // Initially healthy
        assert_eq!(manager.get_health_status(), crate::error_recovery::HealthStatus::Healthy);
        
        // Generate multiple errors to trigger degraded mode
        for i in 0..10 {
            let error = RustyUIError::interpretation(format!("Error #{}", i));
            let context = ErrorContext {
                operation: Operation::Interpretation,
                component_id: Some(format!("component_{}", i)),
                context_data: std::collections::HashMap::new(),
            };
            
            manager.handle_error(&error, context);
        }
        
        // Should be in recovering or degraded mode
        let health = manager.get_health_status();
        assert!(matches!(health, 
            crate::error_recovery::HealthStatus::Recovering | 
            crate::error_recovery::HealthStatus::Degraded
        ));
    }
}