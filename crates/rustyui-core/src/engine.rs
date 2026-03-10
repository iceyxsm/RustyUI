//! Dual-mode engine implementation

use crate::{config::DualModeConfig, error::{Result, RustyUIError}};

#[cfg(feature = "dev-ui")]
use crate::{ChangeMonitor, StatePreservor};

#[cfg(feature = "dev-ui")]
use rustyui_interpreter::InterpretationResult;

// Production-compatible InterpretationResult
#[cfg(not(feature = "dev-ui"))]
#[derive(Debug)]
pub struct InterpretationResult {
    pub execution_time: std::time::Duration,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Core dual-mode engine that manages runtime interpretation and production compilation
pub struct DualModeEngine {
    /// Configuration for dual-mode operation
    config: DualModeConfig,
    
    /// Change monitor for file system watching (development only)
    #[cfg(feature = "dev-ui")]
    change_monitor: Option<ChangeMonitor>,
    
    /// State preservation system (development only)
    #[cfg(feature = "dev-ui")]
    state_preservor: Option<StatePreservor>,
    
    /// Engine initialization state
    initialized: bool,
}

impl DualModeEngine {
    /// Create a new dual-mode engine with the given configuration
    pub fn new(config: DualModeConfig) -> Result<Self> {
        Ok(Self {
            config,
            #[cfg(feature = "dev-ui")]
            change_monitor: None,
            #[cfg(feature = "dev-ui")]
            state_preservor: None,
            initialized: false,
        })
    }
    
    /// Initialize the dual-mode engine
    pub fn initialize(&mut self) -> Result<()> {
        #[cfg(feature = "dev-ui")]
        {
            // Initialize development-only components
            self.change_monitor = Some(ChangeMonitor::new(&self.config.watch_paths)?);
            self.state_preservor = Some(StatePreservor::new());
        }
        
        self.initialized = true;
        Ok(())
    }
    
    /// Check if the engine is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Start development mode with runtime interpretation (development only)
    #[cfg(feature = "dev-ui")]
    pub fn start_development_mode(&mut self) -> Result<()> {
        if !self.initialized {
            self.initialize()?;
        }
        
        if let Some(ref mut monitor) = self.change_monitor {
            monitor.start_watching()?;
        }
        
        Ok(())
    }
    
    /// Start development mode (no-op in production builds)
    #[cfg(not(feature = "dev-ui"))]
    pub fn start_development_mode(&mut self) -> Result<()> {
        // Development features are stripped in production
        Ok(())
    }
    
    /// Check if runtime interpreter is available (development only)
    #[cfg(feature = "dev-ui")]
    pub fn has_runtime_interpreter(&self) -> bool {
        self.change_monitor.is_some()
    }
    
    /// Check if runtime interpreter is available (always false in production)
    #[cfg(not(feature = "dev-ui"))]
    pub fn has_runtime_interpreter(&self) -> bool {
        false
    }
    
    /// Check if the engine can interpret changes (development only)
    #[cfg(feature = "dev-ui")]
    pub fn can_interpret_changes(&self) -> bool {
        self.initialized && self.change_monitor.is_some()
    }
    
    /// Check if the engine can interpret changes (always false in production)
    #[cfg(not(feature = "dev-ui"))]
    pub fn can_interpret_changes(&self) -> bool {
        false
    }
    
    /// Get memory overhead in bytes (development only)
    #[cfg(feature = "dev-ui")]
    pub fn memory_overhead(&self) -> usize {
        // Estimate memory overhead from development components
        let mut overhead = 0;
        
        if self.change_monitor.is_some() {
            overhead += 1024 * 1024; // ~1MB for file watching
        }
        
        if self.state_preservor.is_some() {
            overhead += 512 * 1024; // ~512KB for state preservation
        }
        
        overhead
    }
    
    /// Get memory overhead in bytes (always 0 in production)
    #[cfg(not(feature = "dev-ui"))]
    pub fn memory_overhead(&self) -> usize {
        0
    }
    
    /// Interpret a UI change during development (development only)
    #[cfg(feature = "dev-ui")]
    pub fn interpret_ui_change(&mut self, code: &str, component_id: Option<String>) -> Result<InterpretationResult> {
        use rustyui_interpreter::{RuntimeInterpreter, UIChange, ChangeType};
        
        if !self.can_interpret_changes() {
            return Err(RustyUIError::interpretation("Engine not initialized for interpretation"));
        }
        
        // Create a runtime interpreter (for Phase 1, create new instance each time)
        let mut interpreter = RuntimeInterpreter::new()
            .map_err(|e| RustyUIError::interpretation(format!("Failed to create interpreter: {}", e)))?;
        
        // Create UI change object
        let change = UIChange {
            content: code.to_string(),
            interpretation_strategy: None, // Auto-select strategy
            component_id,
            change_type: ChangeType::ComponentUpdate,
        };
        
        // Interpret the change
        interpreter.interpret_change(&change)
            .map_err(|e| RustyUIError::interpretation(format!("Interpretation failed: {}", e)))
    }
    
    /// Interpret a UI change (no-op in production builds)
    #[cfg(not(feature = "dev-ui"))]
    pub fn interpret_ui_change(&mut self, _code: &str, _component_id: Option<String>) -> Result<InterpretationResult> {
        // Return a dummy success result in production
        Ok(InterpretationResult {
            execution_time: std::time::Duration::from_nanos(0),
            success: true,
            error_message: None,
        })
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &DualModeConfig {
        &self.config
    }
}