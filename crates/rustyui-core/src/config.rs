//! Configuration management for dual-mode operation

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

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
    
    /// Conditional compilation configuration
    pub conditional_compilation: ConditionalCompilation,
    
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
            conditional_compilation: ConditionalCompilation::default(),
            watch_paths: vec![PathBuf::from("src")],
        }
    }
}

impl DualModeConfig {
    /// Create a new configuration with development mode enabled
    #[cfg(feature = "dev-ui")]
    pub fn development(framework: UIFramework) -> Self {
        Self {
            framework,
            development_settings: DevelopmentSettings::default(),
            production_settings: ProductionSettings::default(),
            conditional_compilation: ConditionalCompilation::default(),
            watch_paths: vec![PathBuf::from("src")],
        }
    }
    
    /// Create a new configuration with production mode
    pub fn production(framework: UIFramework) -> Self {
        Self {
            framework,
            #[cfg(feature = "dev-ui")]
            development_settings: DevelopmentSettings::default(),
            production_settings: ProductionSettings::default(),
            conditional_compilation: ConditionalCompilation::default(),
            watch_paths: vec![PathBuf::from("src")],
        }
    }
    
    /// Check if development mode is enabled
    #[cfg(feature = "dev-ui")]
    pub fn development_mode(&self) -> bool {
        true // Always true when dev-ui feature is enabled
    }
    
    #[cfg(not(feature = "dev-ui"))]
    pub fn development_mode(&self) -> bool {
        false // Always false when dev-ui feature is disabled
    }
}

/// Supported UI frameworks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    
    /// Maximum memory overhead in MB (optional)
    pub max_memory_overhead_mb: Option<u64>,
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
            max_memory_overhead_mb: Some(50),
        }
    }
}

/// Runtime interpretation strategies (development only)
#[cfg(feature = "dev-ui")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Conditional compilation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalCompilation {
    /// Development feature flag name
    pub dev_feature_flag: String,
    
    /// Conditional compilation attributes
    pub cfg_attributes: Vec<String>,
    
    /// Feature gates configuration
    pub feature_gates: HashMap<String, bool>,
}

impl Default for ConditionalCompilation {
    fn default() -> Self {
        Self {
            dev_feature_flag: "dev-ui".to_string(),
            cfg_attributes: vec!["feature = \"dev-ui\"".to_string()],
            feature_gates: HashMap::new(),
        }
    }
}