//! State preservation system for hot reload with comprehensive serialization support

use crate::error::{Result, RustyUIError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// State preservation system for maintaining component state across hot reloads
pub struct StatePreservor {
    /// Stored component states with metadata
    component_states: HashMap<String, StateEntry>,
    
    /// Global application state
    global_state: HashMap<String, Value>,
    
    /// State preservation enabled
    enabled: bool,
    
    /// Maximum number of states to keep in memory
    max_states: usize,
    
    /// Statistics for monitoring
    stats: StateStats,
}

/// State entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEntry {
    /// The actual state data
    pub data: Value,
    
    /// Timestamp when state was saved
    pub timestamp: u64,
    
    /// Component type for validation
    pub component_type: String,
    
    /// State version for compatibility checking
    pub version: u32,
}

/// Statistics for state preservation monitoring
#[derive(Debug, Clone)]
pub struct StateStats {
    /// Total number of state saves performed
    pub total_saves: u64,
    
    /// Total number of state restores performed
    pub total_restores: u64,
    
    /// Number of failed serializations
    pub serialization_failures: u64,
    
    /// Number of failed deserializations
    pub deserialization_failures: u64,
    
    /// Total memory used by states
    pub memory_usage: usize,
}

impl StatePreservor {
    /// Create a new state preservor with default settings
    pub fn new() -> Self {
        Self {
            component_states: HashMap::new(),
            global_state: HashMap::new(),
            enabled: true,
            max_states: 1000, // Reasonable default
            stats: StateStats::new(),
        }
    }
    
    /// Create a new state preservor with custom settings
    pub fn with_capacity(max_states: usize) -> Self {
        Self {
            component_states: HashMap::new(),
            global_state: HashMap::new(),
            enabled: true,
            max_states,
            stats: StateStats::new(),
        }
    }
    
    /// Save the state of a component with type information
    pub fn save_component_state<T>(&mut self, component_id: &str, component_type: &str, state: &T) -> Result<()>
    where
        T: Serialize,
    {
        if !self.enabled {
            return Ok(());
        }
        
        // Serialize the state to JSON
        let serialized = serde_json::to_value(state)
            .map_err(|e| {
                self.stats.serialization_failures += 1;
                RustyUIError::state_preservation(format!("Failed to serialize state for {}: {}", component_id, e))
            })?;
        
        // Create state entry with metadata
        let entry = StateEntry {
            data: serialized,
            timestamp: current_timestamp(),
            component_type: component_type.to_string(),
            version: 1, // Phase 1 uses version 1
        };
        
        // Check if we need to evict old states
        if self.component_states.len() >= self.max_states {
            self.evict_oldest_state();
        }
        
        // Store the state
        self.component_states.insert(component_id.to_string(), entry);
        self.stats.total_saves += 1;
        self.update_memory_usage();
        
        Ok(())
    }
    
    /// Restore the state of a component with type checking
    pub fn restore_component_state<T>(&mut self, component_id: &str, expected_type: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if !self.enabled {
            return Ok(None);
        }
        
        if let Some(entry) = self.component_states.get(component_id) {
            // Validate component type
            if entry.component_type != expected_type {
                return Err(RustyUIError::state_preservation(format!(
                    "Component type mismatch for {}: expected {}, found {}",
                    component_id, expected_type, entry.component_type
                )));
            }
            
            // Deserialize the state
            match serde_json::from_value::<T>(entry.data.clone()) {
                Ok(state) => {
                    self.stats.total_restores += 1;
                    Ok(Some(state))
                }
                Err(e) => {
                    self.stats.deserialization_failures += 1;
                    Err(RustyUIError::state_preservation(format!(
                        "Failed to deserialize state for {}: {}", component_id, e
                    )))
                }
            }
        } else {
            Ok(None)
        }
    }
    
    /// Save global application state
    pub fn save_global_state<T>(&mut self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        if !self.enabled {
            return Ok(());
        }
        
        let serialized = serde_json::to_value(value)
            .map_err(|e| {
                self.stats.serialization_failures += 1;
                RustyUIError::state_preservation(format!("Failed to serialize global state {}: {}", key, e))
            })?;
        
        self.global_state.insert(key.to_string(), serialized);
        self.stats.total_saves += 1;
        self.update_memory_usage();
        
        Ok(())
    }
    
    /// Restore global application state
    pub fn restore_global_state<T>(&mut self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if !self.enabled {
            return Ok(None);
        }
        
        if let Some(value) = self.global_state.get(key) {
            match serde_json::from_value::<T>(value.clone()) {
                Ok(state) => {
                    self.stats.total_restores += 1;
                    Ok(Some(state))
                }
                Err(e) => {
                    self.stats.deserialization_failures += 1;
                    Err(RustyUIError::state_preservation(format!(
                        "Failed to deserialize global state {}: {}", key, e
                    )))
                }
            }
        } else {
            Ok(None)
        }
    }
    
    /// Create a snapshot of all current states
    pub fn create_snapshot(&self) -> Result<StateSnapshot> {
        if !self.enabled {
            return Ok(StateSnapshot::empty());
        }
        
        let snapshot = StateSnapshot {
            component_states: self.component_states.clone(),
            global_state: self.global_state.clone(),
            timestamp: current_timestamp(),
            version: 1,
        };
        
        Ok(snapshot)
    }
    
    /// Restore from a state snapshot
    pub fn restore_from_snapshot(&mut self, snapshot: StateSnapshot) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        // Validate snapshot version
        if snapshot.version != 1 {
            return Err(RustyUIError::state_preservation(format!(
                "Unsupported snapshot version: {}", snapshot.version
            )));
        }
        
        // Restore states
        self.component_states = snapshot.component_states;
        self.global_state = snapshot.global_state;
        self.update_memory_usage();
        
        Ok(())
    }
    
    /// Save state snapshot to JSON string
    pub fn serialize_snapshot(&self) -> Result<String> {
        let snapshot = self.create_snapshot()?;
        serde_json::to_string(&snapshot)
            .map_err(|e| RustyUIError::state_preservation(format!("Failed to serialize snapshot: {}", e)))
    }
    
    /// Load state snapshot from JSON string
    pub fn deserialize_snapshot(&mut self, json: &str) -> Result<()> {
        let snapshot: StateSnapshot = serde_json::from_str(json)
            .map_err(|e| RustyUIError::state_preservation(format!("Failed to deserialize snapshot: {}", e)))?;
        
        self.restore_from_snapshot(snapshot)
    }
    
    /// Clear all stored states
    pub fn clear_states(&mut self) {
        self.component_states.clear();
        self.global_state.clear();
        self.update_memory_usage();
    }
    
    /// Clear states older than the specified duration (in seconds)
    pub fn clear_old_states(&mut self, max_age_seconds: u64) {
        let current_time = current_timestamp();
        let cutoff_time = current_time.saturating_sub(max_age_seconds);
        
        self.component_states.retain(|_, entry| entry.timestamp >= cutoff_time);
        self.update_memory_usage();
    }
    
    /// Get the number of stored component states
    pub fn state_count(&self) -> usize {
        self.component_states.len()
    }
    
    /// Get the number of stored global states
    pub fn global_state_count(&self) -> usize {
        self.global_state.len()
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
    
    /// Set maximum number of states to keep
    pub fn set_max_states(&mut self, max_states: usize) {
        self.max_states = max_states;
        
        // Evict excess states if necessary
        while self.component_states.len() > max_states {
            self.evict_oldest_state();
        }
    }
    
    /// Get current statistics
    pub fn get_stats(&self) -> &StateStats {
        &self.stats
    }
    
    /// Get memory usage estimate in bytes
    pub fn memory_usage(&self) -> usize {
        self.stats.memory_usage
    }
    
    /// Check if state preservation is working correctly
    pub fn health_check(&self) -> StateHealthReport {
        let success_rate = if self.stats.total_saves > 0 {
            1.0 - (self.stats.serialization_failures as f64 / self.stats.total_saves as f64)
        } else {
            1.0
        };
        
        let restore_success_rate = if self.stats.total_restores > 0 {
            1.0 - (self.stats.deserialization_failures as f64 / self.stats.total_restores as f64)
        } else {
            1.0
        };
        
        StateHealthReport {
            is_healthy: success_rate > 0.95 && restore_success_rate > 0.95,
            save_success_rate: success_rate,
            restore_success_rate,
            memory_usage: self.stats.memory_usage,
            state_count: self.component_states.len(),
        }
    }
    
    /// Evict the oldest state entry
    fn evict_oldest_state(&mut self) {
        if let Some((oldest_key, _)) = self.component_states
            .iter()
            .min_by_key(|(_, entry)| entry.timestamp)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.component_states.remove(&oldest_key);
        }
    }
    
    /// Update memory usage statistics
    fn update_memory_usage(&mut self) {
        let mut size = std::mem::size_of::<Self>();
        
        // Component states
        for (key, entry) in &self.component_states {
            size += key.len();
            size += entry.component_type.len();
            size += estimate_json_size(&entry.data);
            size += std::mem::size_of::<StateEntry>();
        }
        
        // Global states
        for (key, value) in &self.global_state {
            size += key.len();
            size += estimate_json_size(value);
        }
        
        self.stats.memory_usage = size;
    }
}

/// State snapshot for backup and restore operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Component states at snapshot time
    pub component_states: HashMap<String, StateEntry>,
    
    /// Global states at snapshot time
    pub global_state: HashMap<String, Value>,
    
    /// Snapshot timestamp
    pub timestamp: u64,
    
    /// Snapshot format version
    pub version: u32,
}

impl StateSnapshot {
    /// Create an empty snapshot
    pub fn empty() -> Self {
        Self {
            component_states: HashMap::new(),
            global_state: HashMap::new(),
            timestamp: current_timestamp(),
            version: 1,
        }
    }
}

/// Health report for state preservation system
#[derive(Debug, Clone)]
pub struct StateHealthReport {
    /// Overall health status
    pub is_healthy: bool,
    
    /// Success rate for state saves (0.0 to 1.0)
    pub save_success_rate: f64,
    
    /// Success rate for state restores (0.0 to 1.0)
    pub restore_success_rate: f64,
    
    /// Current memory usage in bytes
    pub memory_usage: usize,
    
    /// Number of stored states
    pub state_count: usize,
}

impl StateStats {
    fn new() -> Self {
        Self {
            total_saves: 0,
            total_restores: 0,
            serialization_failures: 0,
            deserialization_failures: 0,
            memory_usage: 0,
        }
    }
}

impl Default for StatePreservor {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current timestamp in seconds since UNIX epoch
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
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
#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestComponentState {
        counter: i32,
        text: String,
        enabled: bool,
    }
    
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct ComplexState {
        numbers: Vec<i32>,
        mapping: HashMap<String, String>,
        nested: TestComponentState,
    }
    
    #[test]
    fn test_state_preservor_creation() {
        let preservor = StatePreservor::new();
        assert!(preservor.is_enabled());
        assert_eq!(preservor.state_count(), 0);
        assert_eq!(preservor.global_state_count(), 0);
    }
    
    #[test]
    fn test_component_state_save_and_restore() {
        let mut preservor = StatePreservor::new();
        
        let state = TestComponentState {
            counter: 42,
            text: "Hello, World!".to_string(),
            enabled: true,
        };
        
        // Save state
        preservor.save_component_state("button1", "Button", &state).unwrap();
        assert_eq!(preservor.state_count(), 1);
        
        // Restore state
        let restored: Option<TestComponentState> = preservor
            .restore_component_state("button1", "Button")
            .unwrap();
        
        assert_eq!(restored, Some(state));
        assert_eq!(preservor.get_stats().total_saves, 1);
        assert_eq!(preservor.get_stats().total_restores, 1);
    }
    
    #[test]
    fn test_component_type_validation() {
        let mut preservor = StatePreservor::new();
        
        let state = TestComponentState {
            counter: 10,
            text: "Test".to_string(),
            enabled: false,
        };
        
        // Save with one type
        preservor.save_component_state("comp1", "Button", &state).unwrap();
        
        // Try to restore with different type
        let result: Result<Option<TestComponentState>> = preservor
            .restore_component_state("comp1", "Input");
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Component type mismatch"));
    }
    
    #[test]
    fn test_global_state_operations() {
        let mut preservor = StatePreservor::new();
        
        // Save global state
        preservor.save_global_state("app_theme", &"dark".to_string()).unwrap();
        preservor.save_global_state("user_id", &12345u32).unwrap();
        
        assert_eq!(preservor.global_state_count(), 2);
        
        // Restore global state
        let theme: Option<String> = preservor.restore_global_state("app_theme").unwrap();
        let user_id: Option<u32> = preservor.restore_global_state("user_id").unwrap();
        
        assert_eq!(theme, Some("dark".to_string()));
        assert_eq!(user_id, Some(12345u32));
    }
    
    #[test]
    fn test_complex_state_serialization() {
        let mut preservor = StatePreservor::new();
        
        let mut mapping = HashMap::new();
        mapping.insert("key1".to_string(), "value1".to_string());
        mapping.insert("key2".to_string(), "value2".to_string());
        
        let complex_state = ComplexState {
            numbers: vec![1, 2, 3, 4, 5],
            mapping,
            nested: TestComponentState {
                counter: 100,
                text: "Nested".to_string(),
                enabled: true,
            },
        };
        
        // Save and restore complex state
        preservor.save_component_state("complex1", "ComplexComponent", &complex_state).unwrap();
        
        let restored: Option<ComplexState> = preservor
            .restore_component_state("complex1", "ComplexComponent")
            .unwrap();
        
        assert_eq!(restored, Some(complex_state));
    }
    
    #[test]
    fn test_state_snapshot_operations() {
        let mut preservor = StatePreservor::new();
        
        // Add some states
        let state1 = TestComponentState {
            counter: 1,
            text: "State1".to_string(),
            enabled: true,
        };
        
        let state2 = TestComponentState {
            counter: 2,
            text: "State2".to_string(),
            enabled: false,
        };
        
        preservor.save_component_state("comp1", "Button", &state1).unwrap();
        preservor.save_component_state("comp2", "Input", &state2).unwrap();
        preservor.save_global_state("theme", &"light".to_string()).unwrap();
        
        // Create snapshot
        let snapshot = preservor.create_snapshot().unwrap();
        assert_eq!(snapshot.component_states.len(), 2);
        assert_eq!(snapshot.global_state.len(), 1);
        
        // Clear states and restore from snapshot
        preservor.clear_states();
        assert_eq!(preservor.state_count(), 0);
        
        preservor.restore_from_snapshot(snapshot).unwrap();
        assert_eq!(preservor.state_count(), 2);
        assert_eq!(preservor.global_state_count(), 1);
        
        // Verify restored states
        let restored1: Option<TestComponentState> = preservor
            .restore_component_state("comp1", "Button")
            .unwrap();
        assert_eq!(restored1, Some(state1));
    }
    
    #[test]
    fn test_snapshot_serialization() {
        let mut preservor = StatePreservor::new();
        
        let state = TestComponentState {
            counter: 42,
            text: "Serialization Test".to_string(),
            enabled: true,
        };
        
        preservor.save_component_state("test_comp", "TestComponent", &state).unwrap();
        preservor.save_global_state("setting", &"value".to_string()).unwrap();
        
        // Serialize to JSON
        let json = preservor.serialize_snapshot().unwrap();
        assert!(!json.is_empty());
        
        // Clear and deserialize
        preservor.clear_states();
        assert_eq!(preservor.state_count(), 0);
        
        preservor.deserialize_snapshot(&json).unwrap();
        assert_eq!(preservor.state_count(), 1);
        assert_eq!(preservor.global_state_count(), 1);
        
        // Verify restored state
        let restored: Option<TestComponentState> = preservor
            .restore_component_state("test_comp", "TestComponent")
            .unwrap();
        assert_eq!(restored, Some(state));
    }
    
    #[test]
    fn test_state_eviction() {
        let mut preservor = StatePreservor::with_capacity(2);
        
        let state1 = TestComponentState { counter: 1, text: "1".to_string(), enabled: true };
        let state2 = TestComponentState { counter: 2, text: "2".to_string(), enabled: true };
        let state3 = TestComponentState { counter: 3, text: "3".to_string(), enabled: true };
        
        // Add states up to capacity
        preservor.save_component_state("comp1", "Button", &state1).unwrap();
        preservor.save_component_state("comp2", "Button", &state2).unwrap();
        assert_eq!(preservor.state_count(), 2);
        
        // Add one more - should evict oldest
        preservor.save_component_state("comp3", "Button", &state3).unwrap();
        assert_eq!(preservor.state_count(), 2);
        
        // comp1 should be evicted, comp2 and comp3 should remain
        let restored1: Option<TestComponentState> = preservor
            .restore_component_state("comp1", "Button")
            .unwrap();
        assert_eq!(restored1, None);
        
        let restored3: Option<TestComponentState> = preservor
            .restore_component_state("comp3", "Button")
            .unwrap();
        assert_eq!(restored3, Some(state3));
    }
    
    #[test]
    fn test_state_clearing_operations() {
        let mut preservor = StatePreservor::new();
        
        let state = TestComponentState {
            counter: 1,
            text: "Test".to_string(),
            enabled: true,
        };
        
        preservor.save_component_state("comp1", "Button", &state).unwrap();
        preservor.save_global_state("setting", &"value".to_string()).unwrap();
        
        assert_eq!(preservor.state_count(), 1);
        assert_eq!(preservor.global_state_count(), 1);
        
        // Clear all states
        preservor.clear_states();
        assert_eq!(preservor.state_count(), 0);
        assert_eq!(preservor.global_state_count(), 0);
    }
    
    #[test]
    fn test_disabled_state_preservation() {
        let mut preservor = StatePreservor::new();
        preservor.set_enabled(false);
        
        let state = TestComponentState {
            counter: 42,
            text: "Test".to_string(),
            enabled: true,
        };
        
        // Operations should succeed but not store anything
        preservor.save_component_state("comp1", "Button", &state).unwrap();
        assert_eq!(preservor.state_count(), 0);
        
        let restored: Option<TestComponentState> = preservor
            .restore_component_state("comp1", "Button")
            .unwrap();
        assert_eq!(restored, None);
    }
    
    #[test]
    fn test_health_check() {
        let mut preservor = StatePreservor::new();
        
        let state = TestComponentState {
            counter: 1,
            text: "Health".to_string(),
            enabled: true,
        };
        
        // Perform some operations
        preservor.save_component_state("comp1", "Button", &state).unwrap();
        let _: Option<TestComponentState> = preservor
            .restore_component_state("comp1", "Button")
            .unwrap();
        
        let health = preservor.health_check();
        assert!(health.is_healthy);
        assert_eq!(health.save_success_rate, 1.0);
        assert_eq!(health.restore_success_rate, 1.0);
        assert_eq!(health.state_count, 1);
        assert!(health.memory_usage > 0);
    }
    
    #[test]
    fn test_memory_usage_tracking() {
        let mut preservor = StatePreservor::new();
        
        let initial_usage = preservor.memory_usage();
        
        let state = TestComponentState {
            counter: 42,
            text: "Memory Test".to_string(),
            enabled: true,
        };
        
        preservor.save_component_state("comp1", "Button", &state).unwrap();
        
        let after_save_usage = preservor.memory_usage();
        assert!(after_save_usage > initial_usage);
        
        preservor.clear_states();
        let after_clear_usage = preservor.memory_usage();
        assert!(after_clear_usage <= after_save_usage);
    }
    
    #[test]
    fn test_statistics_tracking() {
        let mut preservor = StatePreservor::new();
        
        let state = TestComponentState {
            counter: 1,
            text: "Stats".to_string(),
            enabled: true,
        };
        
        // Initial stats
        let stats = preservor.get_stats();
        assert_eq!(stats.total_saves, 0);
        assert_eq!(stats.total_restores, 0);
        
        // Perform operations
        preservor.save_component_state("comp1", "Button", &state).unwrap();
        preservor.save_component_state("comp2", "Input", &state).unwrap();
        
        let _: Option<TestComponentState> = preservor
            .restore_component_state("comp1", "Button")
            .unwrap();
        
        // Check updated stats
        let stats = preservor.get_stats();
        assert_eq!(stats.total_saves, 2);
        assert_eq!(stats.total_restores, 1);
        assert_eq!(stats.serialization_failures, 0);
        assert_eq!(stats.deserialization_failures, 0);
    }
}