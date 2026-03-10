//! Rhai scripting engine integration for runtime interpretation

use crate::{error::Result, InterpretationResult};
use rhai::{Engine, Scope};
use std::time::Instant;

/// Rhai-based runtime interpreter for simple UI logic
pub struct RhaiInterpreter {
    /// Rhai scripting engine
    engine: Engine,
    
    /// Global scope for persistent variables
    scope: Scope<'static>,
}

impl RhaiInterpreter {
    /// Create a new Rhai interpreter
    pub fn new() -> Result<Self> {
        let mut engine = Engine::new();
        
        // Configure engine for UI scripting
        engine.set_max_operations(10_000); // Limit operations for safety
        engine.set_max_string_size(1024); // Limit string size
        engine.set_max_array_size(100); // Limit array size
        
        // Register UI-specific functions
        Self::register_ui_functions(&mut engine);
        
        Ok(Self {
            engine,
            scope: Scope::new(),
        })
    }
    
    /// Interpret a piece of UI code using Rhai
    pub fn interpret(&mut self, code: &str) -> Result<InterpretationResult> {
        let start_time = Instant::now();
        
        match self.engine.eval_with_scope::<rhai::Dynamic>(&mut self.scope, code) {
            Ok(_result) => {
                let execution_time = start_time.elapsed();
                Ok(InterpretationResult {
                    execution_time,
                    success: true,
                    error_message: None,
                })
            }
            Err(err) => {
                let execution_time = start_time.elapsed();
                Ok(InterpretationResult {
                    execution_time,
                    success: false,
                    error_message: Some(err.to_string()),
                })
            }
        }
    }
    
    /// Register UI-specific functions in the Rhai engine
    fn register_ui_functions(engine: &mut Engine) {
        // Register basic UI functions
        engine.register_fn("button", |text: &str| {
            format!("Button({})", text)
        });
        
        engine.register_fn("text", |content: &str| {
            format!("Text({})", content)
        });
        
        engine.register_fn("layout_vertical", || {
            "VerticalLayout".to_string()
        });
        
        engine.register_fn("layout_horizontal", || {
            "HorizontalLayout".to_string()
        });
        
        // Register state management functions
        engine.register_fn("get_state", |key: &str| {
            format!("get_state({})", key)
        });
        
        engine.register_fn("set_state", |key: &str, value: rhai::Dynamic| {
            format!("set_state({}, {:?})", key, value)
        });
    }
    
    /// Clear the interpreter scope
    pub fn clear_scope(&mut self) {
        self.scope.clear();
    }
    
    /// Get the number of variables in scope
    pub fn scope_size(&self) -> usize {
        self.scope.len()
    }
}

impl Default for RhaiInterpreter {
    fn default() -> Self {
        Self::new().expect("Failed to create default Rhai interpreter")
    }
}