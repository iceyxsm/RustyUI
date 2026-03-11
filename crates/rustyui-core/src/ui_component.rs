//! UI Component trait with hot reload state preservation support

use crate::error::Result;
use crate::RenderContext;
use serde::{Deserialize, Serialize};
use std::any::Any;

/// Trait for UI components that support hot reload with state preservation
/// 
/// This trait provides the interface for components to save and restore their state
/// during hot reload operations. All methods use conditional compilation to ensure
/// zero overhead in production builds.
pub trait UIComponent {
    /// The type of state this component maintains
    type State: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static;
    
    /// Render the component using the provided render context
    fn render(&mut self, ctx: &mut dyn RenderContext);
    
    /// Get the unique identifier for this component instance
    fn component_id(&self) -> &str;
    
    /// Get the component type name for validation
    fn component_type(&self) -> &str;
    
    /// Save the current state for hot reload (development only)
    #[cfg(feature = "dev-ui")]
    fn hot_reload_state(&self) -> Result<Self::State>;
    
    /// Restore state after hot reload (development only)
    #[cfg(feature = "dev-ui")]
    fn restore_state(&mut self, state: Self::State) -> Result<()>;
    
    /// Check if the component supports state preservation
    #[cfg(feature = "dev-ui")]
    fn supports_state_preservation(&self) -> bool {
        true
    }
    
    /// Get state preservation priority (higher = preserved first)
    #[cfg(feature = "dev-ui")]
    fn state_preservation_priority(&self) -> u32 {
        100 // Default priority
    }
    
    /// Validate that a state is compatible with this component
    #[cfg(feature = "dev-ui")]
    fn validate_state(&self, _state: &Self::State) -> Result<()> {
        // Default implementation - always valid
        Ok(())
    }
    
    /// Production builds have no-op implementations
    #[cfg(not(feature = "dev-ui"))]
    fn hot_reload_state(&self) -> Result<Self::State> {
        // This should never be called in production, but we need a default implementation
        // We'll create a default state - components should override this if needed
        unimplemented!("hot_reload_state should not be called in production builds")
    }
    
    #[cfg(not(feature = "dev-ui"))]
    fn restore_state(&mut self, _state: Self::State) -> Result<()> {
        // No-op in production
        Ok(())
    }
}

/// Extension trait for components that need custom state handling
pub trait UIComponentExt: UIComponent {
    /// Called before state is saved during hot reload
    #[cfg(feature = "dev-ui")]
    fn before_state_save(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Called after state is restored during hot reload
    #[cfg(feature = "dev-ui")]
    fn after_state_restore(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Get additional metadata to save with state
    #[cfg(feature = "dev-ui")]
    fn state_metadata(&self) -> Option<serde_json::Value> {
        None
    }
    
    /// Handle state migration between versions
    #[cfg(feature = "dev-ui")]
    fn migrate_state(&self, old_state: serde_json::Value, version: u32) -> Result<Self::State> {
        // Default: try to deserialize directly
        serde_json::from_value(old_state)
            .map_err(|e| crate::error::RustyUIError::state_preservation(
                format!("Failed to migrate state from version {}: {}", version, e)
            ))
    }
}

/// Automatically implement UIComponentExt for all UIComponent types
impl<T: UIComponent> UIComponentExt for T {}

/// Component state wrapper with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStateWrapper<T> {
    /// The actual component state
    pub state: T,
    
    /// Component ID for validation
    pub component_id: String,
    
    /// Component type for validation
    pub component_type: String,
    
    /// State version for migration
    pub version: u32,
    
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
    
    /// Timestamp when state was saved
    pub timestamp: u64,
}

/// Manager for handling UI component state preservation
pub struct ComponentStateManager {
    /// State preservor instance
    #[cfg(feature = "dev-ui")]
    state_preservor: crate::StatePreservor,
}

impl ComponentStateManager {
    /// Create a new component state manager
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "dev-ui")]
            state_preservor: crate::StatePreservor::new(),
        }
    }
    
    /// Save state for a UI component (development only)
    #[cfg(feature = "dev-ui")]
    pub fn save_component_state<T: UIComponent>(&mut self, component: &mut T) -> Result<()> {
        // Call pre-save hook
        component.before_state_save()?;
        
        // Get the component state
        let state = component.hot_reload_state()?;
        
        // Validate the state
        component.validate_state(&state)?;
        
        // Create state wrapper with metadata
        let wrapper = ComponentStateWrapper {
            state,
            component_id: component.component_id().to_string(),
            component_type: component.component_type().to_string(),
            version: 1, // Phase 1 uses version 1
            metadata: component.state_metadata(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        // Save to state preservor
        self.state_preservor.save_component_state(
            component.component_id(),
            component.component_type(),
            &wrapper,
        )?;
        
        Ok(())
    }
    
    /// Restore state for a UI component (development only)
    #[cfg(feature = "dev-ui")]
    pub fn restore_component_state<T: UIComponent>(&mut self, component: &mut T) -> Result<bool> {
        // Try to restore state from preservor
        if let Some(wrapper) = self.state_preservor.restore_component_state::<ComponentStateWrapper<T::State>>(
            component.component_id(),
            component.component_type(),
        )? {
            // Validate component ID and type
            if wrapper.component_id != component.component_id() {
                return Err(crate::error::RustyUIError::state_preservation(
                    format!("Component ID mismatch: expected {}, found {}", 
                        component.component_id(), wrapper.component_id)
                ));
            }
            
            if wrapper.component_type != component.component_type() {
                return Err(crate::error::RustyUIError::state_preservation(
                    format!("Component type mismatch: expected {}, found {}", 
                        component.component_type(), wrapper.component_type)
                ));
            }
            
            // Handle version migration if needed
            let state = if wrapper.version != 1 {
                // Convert to JSON for migration
                let json_state = serde_json::to_value(&wrapper.state)
                    .map_err(|e| crate::error::RustyUIError::state_preservation(
                        format!("Failed to convert state for migration: {}", e)
                    ))?;
                
                component.migrate_state(json_state, wrapper.version)?
            } else {
                wrapper.state
            };
            
            // Validate the restored state
            component.validate_state(&state)?;
            
            // Restore the state
            component.restore_state(state)?;
            
            // Call post-restore hook
            component.after_state_restore()?;
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Save state for multiple components in priority order (development only)
    #[cfg(feature = "dev-ui")]
    pub fn save_multiple_components(&mut self, components: &mut [&mut dyn UIComponentDyn]) -> Result<()> {
        // Sort by priority (higher priority first)
        components.sort_by(|a, b| b.state_preservation_priority().cmp(&a.state_preservation_priority()));
        
        for component in components {
            if component.supports_state_preservation() {
                component.save_state_dyn(self)?;
            }
        }
        
        Ok(())
    }
    
    /// Restore state for multiple components (development only)
    #[cfg(feature = "dev-ui")]
    pub fn restore_multiple_components(&mut self, components: &mut [&mut dyn UIComponentDyn]) -> Result<Vec<bool>> {
        let mut results = Vec::new();
        
        for component in components {
            if component.supports_state_preservation() {
                let restored = component.restore_state_dyn(self)?;
                results.push(restored);
            } else {
                results.push(false);
            }
        }
        
        Ok(results)
    }
    
    /// Get state preservor statistics (development only)
    #[cfg(feature = "dev-ui")]
    pub fn get_stats(&self) -> &crate::state_preservor::StateStats {
        self.state_preservor.get_stats()
    }
    
    /// Clear all component states (development only)
    #[cfg(feature = "dev-ui")]
    pub fn clear_all_states(&mut self) {
        self.state_preservor.clear_states();
    }
    
    /// Production builds have no-op implementations
    #[cfg(not(feature = "dev-ui"))]
    pub fn save_component_state<T: UIComponent>(&mut self, _component: &mut T) -> Result<()> {
        Ok(())
    }
    
    #[cfg(not(feature = "dev-ui"))]
    pub fn restore_component_state<T: UIComponent>(&mut self, _component: &mut T) -> Result<bool> {
        Ok(false)
    }
}

impl Default for ComponentStateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Dynamic trait object for UI components (needed for collections)
pub trait UIComponentDyn {
    /// Get component ID
    fn component_id(&self) -> &str;
    
    /// Get component type
    fn component_type(&self) -> &str;
    
    /// Check if supports state preservation
    #[cfg(feature = "dev-ui")]
    fn supports_state_preservation(&self) -> bool;
    
    /// Get state preservation priority
    #[cfg(feature = "dev-ui")]
    fn state_preservation_priority(&self) -> u32;
    
    /// Save state using dynamic dispatch
    #[cfg(feature = "dev-ui")]
    fn save_state_dyn(&mut self, manager: &mut ComponentStateManager) -> Result<()>;
    
    /// Restore state using dynamic dispatch
    #[cfg(feature = "dev-ui")]
    fn restore_state_dyn(&mut self, manager: &mut ComponentStateManager) -> Result<bool>;
    
    /// Get as Any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// Get as mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Blanket implementation for all UIComponent types
impl<T: UIComponent + 'static> UIComponentDyn for T {
    fn component_id(&self) -> &str {
        UIComponent::component_id(self)
    }
    
    fn component_type(&self) -> &str {
        UIComponent::component_type(self)
    }
    
    #[cfg(feature = "dev-ui")]
    fn supports_state_preservation(&self) -> bool {
        UIComponent::supports_state_preservation(self)
    }
    
    #[cfg(feature = "dev-ui")]
    fn state_preservation_priority(&self) -> u32 {
        UIComponent::state_preservation_priority(self)
    }
    
    #[cfg(feature = "dev-ui")]
    fn save_state_dyn(&mut self, manager: &mut ComponentStateManager) -> Result<()> {
        manager.save_component_state(self)
    }
    
    #[cfg(feature = "dev-ui")]
    fn restore_state_dyn(&mut self, manager: &mut ComponentStateManager) -> Result<bool> {
        manager.restore_component_state(self)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Example button component implementation
#[derive(Debug)]
pub struct ButtonComponent {
    id: String,
    label: String,
    enabled: bool,
    click_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonState {
    label: String,
    enabled: bool,
    click_count: u32,
}

impl ButtonComponent {
    pub fn new(id: String, label: String) -> Self {
        Self {
            id,
            label,
            enabled: true,
            click_count: 0,
        }
    }
    
    pub fn click(&mut self) {
        if self.enabled {
            self.click_count += 1;
        }
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    pub fn set_label(&mut self, label: String) {
        self.label = label;
    }
    
    pub fn get_click_count(&self) -> u32 {
        self.click_count
    }
}

impl UIComponent for ButtonComponent {
    type State = ButtonState;
    
    fn render(&mut self, ctx: &mut dyn RenderContext) {
        ctx.render_button(&self.label, Box::new(|| {
            // This would normally trigger the click handler
            // For now, we'll just increment the count
        }));
    }
    
    fn component_id(&self) -> &str {
        &self.id
    }
    
    fn component_type(&self) -> &str {
        "Button"
    }
    
    #[cfg(feature = "dev-ui")]
    fn hot_reload_state(&self) -> Result<Self::State> {
        Ok(ButtonState {
            label: self.label.clone(),
            enabled: self.enabled,
            click_count: self.click_count,
        })
    }
    
    #[cfg(feature = "dev-ui")]
    fn restore_state(&mut self, state: Self::State) -> Result<()> {
        self.label = state.label;
        self.enabled = state.enabled;
        self.click_count = state.click_count;
        Ok(())
    }
    
    #[cfg(feature = "dev-ui")]
    fn state_preservation_priority(&self) -> u32 {
        200 // Higher priority for interactive components
    }
}

/// Example input component implementation
#[derive(Debug)]
pub struct InputComponent {
    id: String,
    value: String,
    placeholder: String,
    focused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputState {
    value: String,
    placeholder: String,
    focused: bool,
}

impl InputComponent {
    pub fn new(id: String, placeholder: String) -> Self {
        Self {
            id,
            value: String::new(),
            placeholder,
            focused: false,
        }
    }
    
    pub fn set_value(&mut self, value: String) {
        self.value = value;
    }
    
    pub fn get_value(&self) -> &str {
        &self.value
    }
    
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
    
    pub fn is_focused(&self) -> bool {
        self.focused
    }
}

impl UIComponent for InputComponent {
    type State = InputState;
    
    fn render(&mut self, ctx: &mut dyn RenderContext) {
        ctx.render_text(&format!("Input: {} (placeholder: {})", self.value, self.placeholder));
    }
    
    fn component_id(&self) -> &str {
        &self.id
    }
    
    fn component_type(&self) -> &str {
        "Input"
    }
    
    #[cfg(feature = "dev-ui")]
    fn hot_reload_state(&self) -> Result<Self::State> {
        Ok(InputState {
            value: self.value.clone(),
            placeholder: self.placeholder.clone(),
            focused: self.focused,
        })
    }
    
    #[cfg(feature = "dev-ui")]
    fn restore_state(&mut self, state: Self::State) -> Result<()> {
        self.value = state.value;
        self.placeholder = state.placeholder;
        self.focused = state.focused;
        Ok(())
    }
    
    #[cfg(feature = "dev-ui")]
    fn state_preservation_priority(&self) -> u32 {
        300 // Highest priority for input components (preserve user data)
    }
    
    #[cfg(feature = "dev-ui")]
    fn validate_state(&self, state: &Self::State) -> Result<()> {
        // Validate that the value isn't too long
        if state.value.len() > 10000 {
            return Err(crate::error::RustyUIError::state_preservation(
                "Input value too long for state preservation".to_string()
            ));
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_button_component_creation() {
        let button = ButtonComponent::new("btn1".to_string(), "Click Me".to_string());
        assert_eq!(UIComponent::component_id(&button), "btn1");
        assert_eq!(UIComponent::component_type(&button), "Button");
        assert_eq!(button.get_click_count(), 0);
    }
    
    #[test]
    fn test_button_component_interaction() {
        let mut button = ButtonComponent::new("btn1".to_string(), "Click Me".to_string());
        
        // Test clicking
        button.click();
        assert_eq!(button.get_click_count(), 1);
        
        button.click();
        assert_eq!(button.get_click_count(), 2);
        
        // Test disabling
        button.set_enabled(false);
        button.click(); // Should not increment when disabled
        assert_eq!(button.get_click_count(), 2);
        
        // Re-enable and click
        button.set_enabled(true);
        button.click();
        assert_eq!(button.get_click_count(), 3);
    }
    
    #[cfg(feature = "dev-ui")]
    #[test]
    fn test_button_state_preservation() {
        let mut button = ButtonComponent::new("btn1".to_string(), "Click Me".to_string());
        
        // Modify state
        button.click();
        button.click();
        button.set_label("New Label".to_string());
        button.set_enabled(false);
        
        // Save state
        let state = button.hot_reload_state().unwrap();
        assert_eq!(state.click_count, 2);
        assert_eq!(state.label, "New Label");
        assert!(!state.enabled);
        
        // Create new button and restore state
        let mut new_button = ButtonComponent::new("btn1".to_string(), "Original".to_string());
        new_button.restore_state(state).unwrap();
        
        assert_eq!(new_button.get_click_count(), 2);
        assert_eq!(new_button.label, "New Label");
        assert!(!new_button.enabled);
    }
    
    #[test]
    fn test_input_component_creation() {
        let input = InputComponent::new("input1".to_string(), "Enter text...".to_string());
        assert_eq!(UIComponent::component_id(&input), "input1");
        assert_eq!(UIComponent::component_type(&input), "Input");
        assert_eq!(input.get_value(), "");
    }
    
    #[test]
    fn test_input_component_interaction() {
        let mut input = InputComponent::new("input1".to_string(), "Enter text...".to_string());
        
        // Test setting value
        input.set_value("Hello, World!".to_string());
        assert_eq!(input.get_value(), "Hello, World!");
        
        // Test focus
        input.set_focused(true);
        assert!(input.focused);
        
        input.set_focused(false);
        assert!(!input.focused);
    }
    
    #[cfg(feature = "dev-ui")]
    #[test]
    fn test_input_state_preservation() {
        let mut input = InputComponent::new("input1".to_string(), "Enter text...".to_string());
        
        // Modify state
        input.set_value("User input text".to_string());
        input.set_focused(true);
        
        // Save state
        let state = input.hot_reload_state().unwrap();
        assert_eq!(state.value, "User input text");
        assert!(state.focused);
        
        // Create new input and restore state
        let mut new_input = InputComponent::new("input1".to_string(), "Enter text...".to_string());
        new_input.restore_state(state).unwrap();
        
        assert_eq!(new_input.get_value(), "User input text");
        assert!(new_input.focused);
    }
    
    #[cfg(feature = "dev-ui")]
    #[test]
    fn test_input_state_validation() {
        let input = InputComponent::new("input1".to_string(), "Enter text...".to_string());
        
        // Valid state
        let valid_state = InputState {
            value: "Short text".to_string(),
            placeholder: "Placeholder".to_string(),
            focused: false,
        };
        assert!(input.validate_state(&valid_state).is_ok());
        
        // Invalid state (too long)
        let invalid_state = InputState {
            value: "x".repeat(20000), // Too long
            placeholder: "Placeholder".to_string(),
            focused: false,
        };
        assert!(input.validate_state(&invalid_state).is_err());
    }
    
    #[cfg(feature = "dev-ui")]
    #[test]
    fn test_component_state_manager() {
        let mut manager = ComponentStateManager::new();
        let mut button = ButtonComponent::new("btn1".to_string(), "Test".to_string());
        
        // Modify button state
        button.click();
        button.click();
        button.set_label("Modified".to_string());
        
        // Save state
        manager.save_component_state(&mut button).unwrap();
        
        // Create new button and restore
        let mut new_button = ButtonComponent::new("btn1".to_string(), "Original".to_string());
        let restored = manager.restore_component_state(&mut new_button).unwrap();
        
        assert!(restored);
        assert_eq!(new_button.get_click_count(), 2);
        assert_eq!(new_button.label, "Modified");
    }
    
    #[cfg(feature = "dev-ui")]
    #[test]
    fn test_component_state_manager_type_validation() {
        let mut manager = ComponentStateManager::new();
        let mut button = ButtonComponent::new("comp1".to_string(), "Test".to_string());
        
        // Save button state
        button.click();
        manager.save_component_state(&mut button).unwrap();
        
        // Try to restore as input (should fail)
        let mut input = InputComponent::new("comp1".to_string(), "Test".to_string());
        let result = manager.restore_component_state(&mut input);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Component type mismatch"));
    }
    
    #[cfg(feature = "dev-ui")]
    #[test]
    fn test_component_state_manager_id_validation() {
        let mut manager = ComponentStateManager::new();
        let mut button1 = ButtonComponent::new("btn1".to_string(), "Test".to_string());
        
        // Save button1 state
        button1.click();
        manager.save_component_state(&mut button1).unwrap();
        
        // Try to restore with different ID (should not find state)
        let mut button2 = ButtonComponent::new("btn2".to_string(), "Test".to_string());
        let restored = manager.restore_component_state(&mut button2).unwrap();
        
        assert!(!restored); // Should not restore anything
        assert_eq!(button2.get_click_count(), 0); // Should remain unchanged
    }
    
    #[cfg(feature = "dev-ui")]
    #[test]
    fn test_component_priority_ordering() {
        let button = ButtonComponent::new("btn1".to_string(), "Test".to_string());
        let input = InputComponent::new("input1".to_string(), "Test".to_string());
        
        // Input should have higher priority than button
        assert!(UIComponent::state_preservation_priority(&input) > UIComponent::state_preservation_priority(&button));
    }
    
    #[cfg(feature = "dev-ui")]
    #[test]
    fn test_component_state_wrapper() {
        let state = ButtonState {
            label: "Test".to_string(),
            enabled: true,
            click_count: 5,
        };
        
        let wrapper = ComponentStateWrapper {
            state: state.clone(),
            component_id: "btn1".to_string(),
            component_type: "Button".to_string(),
            version: 1,
            metadata: None,
            timestamp: 1234567890,
        };
        
        assert_eq!(wrapper.state.click_count, 5);
        assert_eq!(wrapper.component_id, "btn1");
        assert_eq!(wrapper.component_type, "Button");
        assert_eq!(wrapper.version, 1);
    }
    
    #[cfg(feature = "dev-ui")]
    #[test]
    fn test_component_state_manager_statistics() {
        let mut manager = ComponentStateManager::new();
        let mut button = ButtonComponent::new("btn1".to_string(), "Test".to_string());
        
        // Initial stats
        let stats = manager.get_stats();
        assert_eq!(stats.total_saves, 0);
        assert_eq!(stats.total_restores, 0);
        
        // Save state
        manager.save_component_state(&mut button).unwrap();
        
        let stats = manager.get_stats();
        assert_eq!(stats.total_saves, 1);
        
        // Restore state
        manager.restore_component_state(&mut button).unwrap();
        
        let stats = manager.get_stats();
        assert_eq!(stats.total_restores, 1);
    }
    
    #[cfg(feature = "dev-ui")]
    #[test]
    fn test_component_state_manager_clear() {
        let mut manager = ComponentStateManager::new();
        let mut button = ButtonComponent::new("btn1".to_string(), "Test".to_string());
        
        // Save state
        button.click();
        manager.save_component_state(&mut button).unwrap();
        
        // Verify state exists
        let mut new_button = ButtonComponent::new("btn1".to_string(), "Test".to_string());
        let restored = manager.restore_component_state(&mut new_button).unwrap();
        assert!(restored);
        
        // Clear all states
        manager.clear_all_states();
        
        // Verify state no longer exists
        let mut another_button = ButtonComponent::new("btn1".to_string(), "Test".to_string());
        let restored = manager.restore_component_state(&mut another_button).unwrap();
        assert!(!restored);
    }
    
    #[test]
    fn test_ui_component_dyn_trait() {
        let button = ButtonComponent::new("btn1".to_string(), "Test".to_string());
        let input = InputComponent::new("input1".to_string(), "Test".to_string());
        
        // Test dynamic trait methods
        assert_eq!(UIComponent::component_id(&button), "btn1");
        assert_eq!(UIComponent::component_type(&button), "Button");
        assert_eq!(UIComponent::component_id(&input), "input1");
        assert_eq!(UIComponent::component_type(&input), "Input");
        
        // Test Any trait methods
        assert!(button.as_any().is::<ButtonComponent>());
        assert!(input.as_any().is::<InputComponent>());
        assert!(!button.as_any().is::<InputComponent>());
        assert!(!input.as_any().is::<ButtonComponent>());
    }
}