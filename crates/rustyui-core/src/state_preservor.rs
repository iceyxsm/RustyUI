//! State preservation system for hot reload

use crate::error::Result;
use serde_json::Value;
use std::collections::HashMap;

/// State preservation system for maintaining component state across hot reloads
pub struct StatePreservor {
    /// Stored component states
    component_states: HashMap<String, Value>,
    
    /// State preservation enabled
    enabled: bool,
}

impl StatePreservor {
    /// Create a new state preservor
    pub fn new() -> Self {
        Self {
            component_states: HashMap::new(),
            enabled: true,
        }
    }
    
    /// Save the state of a component
    pub fn save_component_state(&mut self, component_id: &str, state: Value) -> Result<()> {
        if self.enabled {
            self.component_states.insert(component_id.to_string(), state);
        }
        Ok(())
    }
    
    /// Restore the state of a component
    pub fn restore_component_state(&self, component_id: &str) -> Option<&Value> {
        if self.enabled {
            self.component_states.get(component_id)
        } else {
            None
        }
    }
    
    /// Clear all stored states
    pub fn clear_states(&mut self) {
        self.component_states.clear();
    }
    
    /// Get the number of stored component states
    pub fn state_count(&self) -> usize {
        self.component_states.len()
    }
    
    /// Enable or disable state preservation
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.clear_states();
        }
    }
    
    /// Check if state preservation is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Get memory usage estimate in bytes
    pub fn memory_usage(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();
        
        for (key, value) in &self.component_states {
            size += key.len();
            size += estimate_json_size(value);
        }
        
        size
    }
}

impl Default for StatePreservor {
    fn default() -> Self {
        Self::new()
    }
}

/// Estimate the memory size of a JSON value
fn estimate_json_size(value: &Value) -> usize {
    match value {
        Value::Null => 0,
        Value::Bool(_) => 1,
        Value::Number(_) => 8, // Approximate for f64
        Value::String(s) => s.len(),
        Value::Array(arr) => {
            arr.iter().map(estimate_json_size).sum::<usize>() + (arr.len() * 8)
        }
        Value::Object(obj) => {
            obj.iter()
                .map(|(k, v)| k.len() + estimate_json_size(v))
                .sum::<usize>()
                + (obj.len() * 16)
        }
    }
}