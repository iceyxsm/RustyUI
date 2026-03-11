//! Component lifecycle management for development mode

use crate::error::{Result, RustyUIError};
use std::collections::HashMap;
use std::time::{SystemTime, Duration};

#[cfg(feature = "dev-ui")]
use serde_json::Value;

/// Component lifecycle states
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentState {
    /// Component is being created
    Creating,
    /// Component is active and rendering
    Active,
    /// Component is being updated
    Updating,
    /// Component is being destroyed
    Destroying,
    /// Component has been destroyed
    Destroyed,
}

/// Component lifecycle information
#[derive(Debug, Clone)]
pub struct ComponentLifecycle {
    /// Component unique identifier
    pub id: String,
    /// Component type name
    pub type_name: String,
    /// Current state
    pub state: ComponentState,
    /// Creation timestamp
    pub created_at: SystemTime,
    /// Last update timestamp
    pub last_updated: SystemTime,
    /// Update count
    pub update_count: u64,
    /// Preserved state data
    #[cfg(feature = "dev-ui")]
    pub preserved_state: Option<Value>,
}

/// Component lifecycle manager for tracking and managing UI components
pub struct ComponentLifecycleManager {
    /// Registry of active components
    components: HashMap<String, ComponentLifecycle>,
    /// Component creation order
    creation_order: Vec<String>,
    /// Maximum number of components to track
    max_components: usize,
}

impl ComponentLifecycleManager {
    /// Create a new component lifecycle manager
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            creation_order: Vec::new(),
            max_components: 1000, // Reasonable default
        }
    }
    
    /// Create a new component lifecycle manager with custom limits
    pub fn with_limits(max_components: usize) -> Self {
        Self {
            components: HashMap::new(),
            creation_order: Vec::new(),
            max_components,
        }
    }
    
    /// Register a new component
    pub fn register_component(&mut self, id: String, type_name: String) -> Result<()> {
        // Check if we're at capacity
        if self.components.len() >= self.max_components {
            // Remove oldest component
            if let Some(oldest_id) = self.creation_order.first().cloned() {
                self.unregister_component(&oldest_id)?;
            }
        }
        
        let now = SystemTime::now();
        let lifecycle = ComponentLifecycle {
            id: id.clone(),
            type_name,
            state: ComponentState::Creating,
            created_at: now,
            last_updated: now,
            update_count: 0,
            #[cfg(feature = "dev-ui")]
            preserved_state: None,
        };
        
        self.components.insert(id.clone(), lifecycle);
        self.creation_order.push(id);
        
        Ok(())
    }
    
    /// Unregister a component
    pub fn unregister_component(&mut self, id: &str) -> Result<()> {
        if let Some(mut lifecycle) = self.components.remove(id) {
            lifecycle.state = ComponentState::Destroyed;
            self.creation_order.retain(|component_id| component_id != id);
        }
        Ok(())
    }
    
    /// Update component state
    pub fn update_component_state(&mut self, id: &str, state: ComponentState) -> Result<()> {
        if let Some(lifecycle) = self.components.get_mut(id) {
            lifecycle.state = state.clone();
            lifecycle.last_updated = SystemTime::now();
            if matches!(state, ComponentState::Updating) {
                lifecycle.update_count += 1;
            }
        } else {
            return Err(RustyUIError::component_not_found(format!("Component '{}' not found", id)));
        }
        Ok(())
    }
    
    /// Get component lifecycle information
    pub fn get_component(&self, id: &str) -> Option<&ComponentLifecycle> {
        self.components.get(id)
    }
    
    /// Get all active components
    pub fn get_active_components(&self) -> Vec<&ComponentLifecycle> {
        self.components.values()
            .filter(|lifecycle| lifecycle.state == ComponentState::Active)
            .collect()
    }
    
    /// Get components by type
    pub fn get_components_by_type(&self, type_name: &str) -> Vec<&ComponentLifecycle> {
        self.components.values()
            .filter(|lifecycle| lifecycle.type_name == type_name)
            .collect()
    }
    
    /// Preserve component state for hot reload
    #[cfg(feature = "dev-ui")]
    pub fn preserve_component_state(&mut self, id: &str, state: Value) -> Result<()> {
        if let Some(lifecycle) = self.components.get_mut(id) {
            lifecycle.preserved_state = Some(state);
            lifecycle.last_updated = SystemTime::now();
        } else {
            return Err(RustyUIError::component_not_found(format!("Component '{}' not found", id)));
        }
        Ok(())
    }
    
    /// Restore component state after hot reload
    #[cfg(feature = "dev-ui")]
    pub fn restore_component_state(&self, id: &str) -> Option<&Value> {
        self.components.get(id)
            .and_then(|lifecycle| lifecycle.preserved_state.as_ref())
    }
    
    /// Clear preserved state for a component
    #[cfg(feature = "dev-ui")]
    pub fn clear_preserved_state(&mut self, id: &str) -> Result<()> {
        if let Some(lifecycle) = self.components.get_mut(id) {
            lifecycle.preserved_state = None;
        }
        Ok(())
    }
    
    /// Get component statistics
    pub fn get_statistics(&self) -> ComponentStatistics {
        let total_components = self.components.len();
        let active_components = self.get_active_components().len();
        let total_updates = self.components.values()
            .map(|lifecycle| lifecycle.update_count)
            .sum();
        
        let avg_age = if !self.components.is_empty() {
            let now = SystemTime::now();
            let total_age: Duration = self.components.values()
                .filter_map(|lifecycle| now.duration_since(lifecycle.created_at).ok())
                .sum();
            Some(total_age / self.components.len() as u32)
        } else {
            None
        };
        
        ComponentStatistics {
            total_components,
            active_components,
            total_updates,
            average_age: avg_age,
        }
    }
    
    /// Cleanup destroyed components
    pub fn cleanup(&mut self) {
        let destroyed_ids: Vec<String> = self.components.iter()
            .filter(|(_, lifecycle)| lifecycle.state == ComponentState::Destroyed)
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in destroyed_ids {
            self.components.remove(&id);
            self.creation_order.retain(|component_id| component_id != &id);
        }
    }
    
    /// Get the number of registered components
    pub fn component_count(&self) -> usize {
        self.components.len()
    }
    
    /// Check if a component is registered
    pub fn has_component(&self, id: &str) -> bool {
        self.components.contains_key(id)
    }
}

/// Component statistics
#[derive(Debug, Clone)]
pub struct ComponentStatistics {
    /// Total number of components
    pub total_components: usize,
    /// Number of active components
    pub active_components: usize,
    /// Total number of updates across all components
    pub total_updates: u64,
    /// Average age of components
    pub average_age: Option<Duration>,
}

impl Default for ComponentLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_component_lifecycle_manager_creation() {
        let manager = ComponentLifecycleManager::new();
        assert_eq!(manager.component_count(), 0);
        assert_eq!(manager.max_components, 1000);
    }
    
    #[test]
    fn test_component_registration() {
        let mut manager = ComponentLifecycleManager::new();
        
        assert!(manager.register_component("btn1".to_string(), "Button".to_string()).is_ok());
        assert_eq!(manager.component_count(), 1);
        assert!(manager.has_component("btn1"));
        
        let component = manager.get_component("btn1").unwrap();
        assert_eq!(component.id, "btn1");
        assert_eq!(component.type_name, "Button");
        assert_eq!(component.state, ComponentState::Creating);
    }
    
    #[test]
    fn test_component_state_updates() {
        let mut manager = ComponentLifecycleManager::new();
        manager.register_component("btn1".to_string(), "Button".to_string()).unwrap();
        
        assert!(manager.update_component_state("btn1", ComponentState::Active).is_ok());
        let component = manager.get_component("btn1").unwrap();
        assert_eq!(component.state, ComponentState::Active);
        
        assert!(manager.update_component_state("btn1", ComponentState::Updating).is_ok());
        let component = manager.get_component("btn1").unwrap();
        assert_eq!(component.update_count, 1);
    }
    
    #[test]
    fn test_component_unregistration() {
        let mut manager = ComponentLifecycleManager::new();
        manager.register_component("btn1".to_string(), "Button".to_string()).unwrap();
        
        assert!(manager.unregister_component("btn1").is_ok());
        assert!(!manager.has_component("btn1"));
        assert_eq!(manager.component_count(), 0);
    }
    
    #[cfg(feature = "dev-ui")]
    #[test]
    fn test_state_preservation() {
        let mut manager = ComponentLifecycleManager::new();
        manager.register_component("btn1".to_string(), "Button".to_string()).unwrap();
        
        let state = serde_json::json!({"text": "Hello", "enabled": true});
        assert!(manager.preserve_component_state("btn1", state.clone()).is_ok());
        
        let restored_state = manager.restore_component_state("btn1").unwrap();
        assert_eq!(*restored_state, state);
        
        assert!(manager.clear_preserved_state("btn1").is_ok());
        assert!(manager.restore_component_state("btn1").is_none());
    }
}