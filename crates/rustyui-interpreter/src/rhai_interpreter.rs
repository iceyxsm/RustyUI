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
        // Register basic UI component functions
        engine.register_fn("button", |text: &str| {
            format!("Button(text: \"{}\")", text)
        });
        
        engine.register_fn("text", |content: &str| {
            format!("Text(content: \"{}\")", content)
        });
        
        engine.register_fn("input", |placeholder: &str| {
            format!("Input(placeholder: \"{}\")", placeholder)
        });
        
        engine.register_fn("checkbox", |label: &str, checked: bool| {
            format!("Checkbox(label: \"{}\", checked: {})", label, checked)
        });
        
        // Register layout functions
        engine.register_fn("vertical_layout", || {
            "VerticalLayout".to_string()
        });
        
        engine.register_fn("horizontal_layout", || {
            "HorizontalLayout".to_string()
        });
        
        engine.register_fn("grid_layout", |rows: i64, cols: i64| {
            format!("GridLayout(rows: {}, cols: {})", rows, cols)
        });
        
        // Register styling functions
        engine.register_fn("style", |property: &str, value: &str| {
            format!("Style({}: {})", property, value)
        });
        
        engine.register_fn("color", |r: i64, g: i64, b: i64| {
            format!("Color(r: {}, g: {}, b: {})", r, g, b)
        });
        
        engine.register_fn("padding", |value: i64| {
            format!("Padding({})", value)
        });
        
        engine.register_fn("margin", |value: i64| {
            format!("Margin({})", value)
        });
        
        // Register state management functions
        engine.register_fn("get_state", |key: &str| {
            format!("get_state(\"{}\")", key)
        });
        
        engine.register_fn("set_state", |key: &str, value: rhai::Dynamic| {
            format!("set_state(\"{}\", {:?})", key, value)
        });
        
        engine.register_fn("update_component", |id: &str, property: &str, value: rhai::Dynamic| {
            format!("update_component(\"{}\", \"{}\", {:?})", id, property, value)
        });
        
        // Register event handling functions
        engine.register_fn("on_click", |handler: &str| {
            format!("on_click({})", handler)
        });
        
        engine.register_fn("on_change", |handler: &str| {
            format!("on_change({})", handler)
        });
        
        engine.register_fn("on_hover", |handler: &str| {
            format!("on_hover({})", handler)
        });
        
        // Register utility functions
        engine.register_fn("log", |message: &str| {
            println!("UI Log: {}", message);
            format!("log(\"{}\")", message)
        });
        
        engine.register_fn("debug", |value: rhai::Dynamic| {
            println!("UI Debug: {:?}", value);
            format!("debug({:?})", value)
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