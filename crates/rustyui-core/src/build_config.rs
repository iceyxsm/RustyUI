//! Build configuration management for dual-mode compilation

/// Build configuration for RustyUI applications
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Whether development features are enabled
    pub development_mode: bool,
    
    /// Target optimization level
    pub optimization_level: OptimizationLevel,
    
    /// Whether to strip debug information
    pub strip_debug: bool,
    
    /// Whether to enable LTO (Link Time Optimization)
    pub enable_lto: bool,
}

impl BuildConfig {
    /// Create a development build configuration
    pub fn development() -> Self {
        Self {
            development_mode: true,
            optimization_level: OptimizationLevel::Debug,
            strip_debug: false,
            enable_lto: false,
        }
    }
    
    /// Create a production build configuration
    pub fn production() -> Self {
        Self {
            development_mode: false,
            optimization_level: OptimizationLevel::Release,
            strip_debug: true,
            enable_lto: true,
        }
    }
    
    /// Check if development features should be compiled
    pub fn has_dev_features(&self) -> bool {
        cfg!(feature = "dev-ui") && self.development_mode
    }
    
    /// Check if this is a zero-overhead production build
    pub fn is_zero_overhead(&self) -> bool {
        !self.development_mode && 
        !cfg!(feature = "dev-ui") &&
        matches!(self.optimization_level, OptimizationLevel::Release)
    }
    
    /// Get recommended Cargo build flags
    pub fn cargo_flags(&self) -> Vec<String> {
        let mut flags = Vec::new();
        
        if self.development_mode {
            flags.push("--features".to_string());
            flags.push("dev-ui".to_string());
        } else {
            flags.push("--release".to_string());
        }
        
        if self.strip_debug && !self.development_mode {
            // Strip flag is handled in Cargo.toml profile
        }
        
        flags
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        {
            Self::development()
        }
        #[cfg(not(debug_assertions))]
        {
            Self::production()
        }
    }
}

/// Optimization levels for builds
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationLevel {
    Debug,
    Release,
    Size,
}

/// Compile-time build information
pub struct BuildInfo;

impl BuildInfo {
    /// Check if development features are available at compile time
    pub const fn has_dev_features() -> bool {
        cfg!(feature = "dev-ui")
    }
    
    /// Check if this is a debug build
    pub const fn is_debug_build() -> bool {
        cfg!(debug_assertions)
    }
    
    /// Check if this is a release build
    pub const fn is_release_build() -> bool {
        !cfg!(debug_assertions)
    }
    
    /// Get build mode as string
    pub const fn build_mode() -> &'static str {
        if cfg!(feature = "dev-ui") {
            "development"
        } else if cfg!(debug_assertions) {
            "debug"
        } else {
            "production"
        }
    }
    
    /// Get estimated memory overhead for development features
    pub const fn dev_memory_overhead_bytes() -> usize {
        if cfg!(feature = "dev-ui") {
            // Estimated overhead for development features
            2 * 1024 * 1024 // ~2MB for interpreter, file watcher, state preservation
        } else {
            0 // Zero overhead in production
        }
    }
}