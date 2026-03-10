//! Configuration management for dual-mode operation

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for dual-mode engine operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualModeConfig {
    /// UI framework being used
    pub framework: UIFramework,
    
    /// Development mode settings (only used when dev-ui feature is enabled)
    #[cfg(feature = "dev-ui")]
    pub development_settings: DevelopmentSettings,
    
    /// Production mode settings
    pub production_settings: ProductionSettings,
    
    /// Paths to watch for changes
    pub watch_paths: Vec<PathBuf>,
}

impl Default for DualModeConfig {
    fn default() -> Self {
        Self {
            framework: UIFramework::Egui,
            #[cfg(feature = "dev-ui")]
            development_settings: DevelopmentSettings::default(),
            production_settings: ProductionSettings::default(),
            watch_paths: vec![PathBuf::from("src")],
        }
    }
}

/// Supported UI frameworks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIFramework {
    Egui,
    Iced,
    Slint,
    Tauri,
    Custom { name: String, adapter_path: String },
}

/// Development mode configuration (only available with dev-ui feature)
#[cfg(feature = "dev-ui")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentSettings {
    /// Strategy for runtime interpretation
    pub interpretation_strategy: InterpretationStrategy,
    
    /// Threshold for JIT compilation (in milliseconds)
    pub jit_compilation_threshold: u32,
    
    /// State preservation configuration
    pub state_preservation: bool,
    
    /// Performance monitoring enabled
    pub performance_monitoring: bool,
    
    /// Delay before processing file changes (in milliseconds)
    pub change_detection_delay_ms: u64,
}

#[cfg(feature = "dev-ui")]
impl Default for DevelopmentSettings {
    fn default() -> Self {
        Self {
            interpretation_strategy: InterpretationStrategy::Hybrid { 
                rhai_threshold: 10, 
                jit_threshold: 100 
            },
            jit_compilation_threshold: 100,
            state_preservation: true,
            performance_monitoring: true,
            change_detection_delay_ms: 50,
        }
    }
}

/// Runtime interpretation strategies (development only)
#[cfg(feature = "dev-ui")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterpretationStrategy {
    /// Use only Rhai scripting
    RhaiOnly,
    /// Use only AST interpretation
    ASTOnly,
    /// Use hybrid approach with thresholds
    Hybrid { rhai_threshold: u32, jit_threshold: u32 },
    /// Prefer JIT compilation
    JITPreferred,
}

/// Production mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionSettings {
    /// Strip all development features
    pub strip_dev_features: bool,
    
    /// Optimization level
    pub optimization_level: OptimizationLevel,
    
    /// Enable binary size optimization
    pub binary_size_optimization: bool,
    
    /// Enable security hardening
    pub security_hardening: bool,
}

impl Default for ProductionSettings {
    fn default() -> Self {
        Self {
            strip_dev_features: true,
            optimization_level: OptimizationLevel::Release,
            binary_size_optimization: true,
            security_hardening: true,
        }
    }
}

/// Optimization levels for production builds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    Debug,
    Release,
    ReleaseLTO,
}