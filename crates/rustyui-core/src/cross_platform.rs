//! Cross-platform build and runtime configuration
//! 
//! This module provides cross-platform build configuration, conditional compilation
//! helpers, and runtime platform adaptation for RustyUI.

use crate::platform::{Platform, PlatformConfig, FileWatcherBackend};
use serde::{Serialize, Deserialize};

/// Cross-platform build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPlatformConfig {
    /// Target platform configurations
    pub platforms: Vec<PlatformTargetConfig>,
    /// Default configuration for unknown platforms
    pub default_config: PlatformConfig,
    /// Feature flags for conditional compilation
    pub feature_flags: FeatureFlags,
}

/// Configuration for a specific platform target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformTargetConfig {
    /// Target platform
    pub platform: Platform,
    /// Platform-specific configuration
    pub config: PlatformConfig,
    /// Platform-specific feature flags
    pub features: Vec<String>,
    /// Platform-specific dependencies
    pub dependencies: Vec<PlatformDependency>,
}

/// Platform-specific dependency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformDependency {
    /// Dependency name
    pub name: String,
    /// Version requirement
    pub version: String,
    /// Optional features to enable
    pub features: Vec<String>,
    /// Whether this dependency is optional
    pub optional: bool,
}

/// Feature flags for conditional compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable Windows-specific features
    pub windows_native: bool,
    /// Enable macOS-specific features
    pub macos_native: bool,
    /// Enable Unix/Linux-specific features
    pub unix_native: bool,
    /// Enable development-only features
    pub dev_ui: bool,
    /// Enable JIT compilation
    pub jit_compilation: bool,
    /// Enable performance monitoring
    pub performance_monitoring: bool,
}

/// Cross-platform compatibility layer
pub struct CrossPlatformLayer {
    _config: CrossPlatformConfig,
    current_platform: Platform,
    runtime_config: PlatformConfig,
}

impl CrossPlatformConfig {
    /// Create default cross-platform configuration
    pub fn default() -> Self {
        let platforms = vec![
            PlatformTargetConfig {
                platform: Platform::Windows,
                config: PlatformConfig::auto_detect(),
                features: vec![
                    "windows-native".to_string(),
                    "dev-ui".to_string(),
                ],
                dependencies: vec![
                    PlatformDependency {
                        name: "winapi".to_string(),
                        version: "0.3".to_string(),
                        features: vec![
                            "winuser".to_string(),
                            "processthreadsapi".to_string(),
                            "handleapi".to_string(),
                            "fileapi".to_string(),
                            "winnt".to_string(),
                        ],
                        optional: true,
                    },
                ],
            },
            PlatformTargetConfig {
                platform: Platform::MacOS,
                config: PlatformConfig::auto_detect(),
                features: vec![
                    "macos-native".to_string(),
                    "dev-ui".to_string(),
                ],
                dependencies: vec![
                    PlatformDependency {
                        name: "core-foundation".to_string(),
                        version: "0.9".to_string(),
                        features: vec![],
                        optional: true,
                    },
                    PlatformDependency {
                        name: "core-services".to_string(),
                        version: "0.2".to_string(),
                        features: vec![],
                        optional: true,
                    },
                ],
            },
            PlatformTargetConfig {
                platform: Platform::Linux,
                config: PlatformConfig::auto_detect(),
                features: vec![
                    "unix-native".to_string(),
                    "dev-ui".to_string(),
                ],
                dependencies: vec![
                    PlatformDependency {
                        name: "libc".to_string(),
                        version: "0.2".to_string(),
                        features: vec![],
                        optional: true,
                    },
                ],
            },
        ];

        Self {
            platforms,
            default_config: PlatformConfig::auto_detect(),
            feature_flags: FeatureFlags::default(),
        }
    }

    /// Get configuration for a specific platform
    pub fn get_platform_config(&self, platform: Platform) -> Option<&PlatformTargetConfig> {
        self.platforms.iter().find(|p| p.platform == platform)
    }

    /// Generate Cargo.toml features section
    pub fn generate_cargo_features(&self) -> String {
        let mut features = Vec::new();
        
        features.push("[features]".to_string());
        features.push("default = []".to_string());
        
        if self.feature_flags.dev_ui {
            features.push("dev-ui = [".to_string());
            features.push("    \"notify\",".to_string());
            features.push("    \"notify-debouncer-mini\",".to_string());
            features.push("    \"rhai\",".to_string());
            features.push("]".to_string());
        }
        
        if self.feature_flags.windows_native {
            features.push("windows-native = [\"winapi\"]".to_string());
        }
        
        if self.feature_flags.macos_native {
            features.push("macos-native = [\"core-foundation\", \"core-services\"]".to_string());
        }
        
        if self.feature_flags.unix_native {
            features.push("unix-native = [\"libc\"]".to_string());
        }
        
        features.push("cross-platform = [\"windows-native\", \"macos-native\", \"unix-native\"]".to_string());
        
        features.join("\n")
    }

    /// Generate platform-specific dependencies section
    pub fn generate_platform_dependencies(&self) -> String {
        let mut deps = Vec::new();
        
        // Windows dependencies
        deps.push("# Windows-specific dependencies".to_string());
        deps.push("[target.'cfg(windows)'.dependencies]".to_string());
        for platform_config in &self.platforms {
            if platform_config.platform == Platform::Windows {
                for dep in &platform_config.dependencies {
                    let features_str = if dep.features.is_empty() {
                        String::new()
                    } else {
                        format!(", features = {:?}", dep.features)
                    };
                    
                    let optional_str = if dep.optional { ", optional = true" } else { "" };
                    
                    deps.push(format!(
                        "{} = {{ version = \"{}\"{}{} }}",
                        dep.name, dep.version, features_str, optional_str
                    ));
                }
            }
        }
        
        deps.push("".to_string());
        
        // Unix dependencies
        deps.push("# Unix/Linux-specific dependencies".to_string());
        deps.push("[target.'cfg(unix)'.dependencies]".to_string());
        for platform_config in &self.platforms {
            if matches!(platform_config.platform, Platform::Linux) {
                for dep in &platform_config.dependencies {
                    let optional_str = if dep.optional { ", optional = true" } else { "" };
                    deps.push(format!(
                        "{} = {{ version = \"{}\"{} }}",
                        dep.name, dep.version, optional_str
                    ));
                }
            }
        }
        
        deps.push("".to_string());
        
        // macOS dependencies
        deps.push("# macOS-specific dependencies".to_string());
        deps.push("[target.'cfg(target_os = \"macos\")'.dependencies]".to_string());
        for platform_config in &self.platforms {
            if platform_config.platform == Platform::MacOS {
                for dep in &platform_config.dependencies {
                    let optional_str = if dep.optional { ", optional = true" } else { "" };
                    deps.push(format!(
                        "{} = {{ version = \"{}\"{} }}",
                        dep.name, dep.version, optional_str
                    ));
                }
            }
        }
        
        deps.join("\n")
    }
}

impl FeatureFlags {
    /// Create default feature flags
    pub fn default() -> Self {
        Self {
            windows_native: cfg!(target_os = "windows"),
            macos_native: cfg!(target_os = "macos"),
            unix_native: cfg!(unix),
            dev_ui: cfg!(feature = "dev-ui"),
            jit_compilation: true,
            performance_monitoring: cfg!(feature = "dev-ui"),
        }
    }

    /// Check if native APIs should be used
    pub fn use_native_apis(&self) -> bool {
        self.windows_native || self.macos_native || self.unix_native
    }

    /// Get active feature list
    pub fn active_features(&self) -> Vec<&'static str> {
        let mut features = Vec::new();
        
        if self.windows_native {
            features.push("windows-native");
        }
        if self.macos_native {
            features.push("macos-native");
        }
        if self.unix_native {
            features.push("unix-native");
        }
        if self.dev_ui {
            features.push("dev-ui");
        }
        if self.jit_compilation {
            features.push("jit-compilation");
        }
        if self.performance_monitoring {
            features.push("performance-monitoring");
        }
        
        features
    }
}

impl CrossPlatformLayer {
    /// Create a new cross-platform layer
    pub fn new() -> Self {
        let config = CrossPlatformConfig::default();
        let current_platform = Platform::current();
        let runtime_config = PlatformConfig::auto_detect();
        
        Self {
            _config: config,
            current_platform,
            runtime_config,
        }
    }

    /// Get the current platform
    pub fn current_platform(&self) -> Platform {
        self.current_platform
    }

    /// Get the runtime configuration
    pub fn runtime_config(&self) -> &PlatformConfig {
        &self.runtime_config
    }

    /// Check if a feature is available on the current platform
    pub fn is_feature_available(&self, feature: &str) -> bool {
        match feature {
            "file-watching" => true, // Available on all platforms
            "jit-compilation" => self.runtime_config.use_jit_compilation,
            "native-apis" => self.runtime_config.use_native_apis,
            "performance-monitoring" => cfg!(feature = "dev-ui"),
            _ => false,
        }
    }

    /// Get optimal file watcher configuration
    pub fn file_watcher_config(&self) -> FileWatcherConfig {
        let backend = self.runtime_config.file_watcher_backend;
        let perf = backend.performance_characteristics();
        
        FileWatcherConfig {
            backend,
            debounce_timeout: std::time::Duration::from_millis(perf.latency_ms.max(10) as u64),
            max_files: perf.max_files.unwrap_or(1000),
            use_native_apis: self.runtime_config.use_native_apis,
        }
    }

    /// Adapt configuration for current platform
    pub fn adapt_config<T>(&self, base_config: T, adapter: impl Fn(T, Platform) -> T) -> T {
        adapter(base_config, self.current_platform)
    }
}

/// File watcher configuration optimized for current platform
#[derive(Debug, Clone)]
pub struct FileWatcherConfig {
    pub backend: FileWatcherBackend,
    pub debounce_timeout: std::time::Duration,
    pub max_files: u32,
    pub use_native_apis: bool,
}

/// Conditional compilation helpers
pub mod conditional {
    /// Execute code only on Windows
    #[macro_export]
    macro_rules! windows_only {
        ($($code:tt)*) => {
            #[cfg(target_os = "windows")]
            {
                $($code)*
            }
        };
    }

    /// Execute code only on macOS
    #[macro_export]
    macro_rules! macos_only {
        ($($code:tt)*) => {
            #[cfg(target_os = "macos")]
            {
                $($code)*
            }
        };
    }

    /// Execute code only on Linux
    #[macro_export]
    macro_rules! linux_only {
        ($($code:tt)*) => {
            #[cfg(target_os = "linux")]
            {
                $($code)*
            }
        };
    }

    /// Execute code only on Unix-like systems
    #[macro_export]
    macro_rules! unix_only {
        ($($code:tt)*) => {
            #[cfg(unix)]
            {
                $($code)*
            }
        };
    }

    /// Execute code only in development mode
    #[macro_export]
    macro_rules! dev_only {
        ($($code:tt)*) => {
            #[cfg(feature = "dev-ui")]
            {
                $($code)*
            }
        };
    }

    /// Execute code only in production mode
    #[macro_export]
    macro_rules! production_only {
        ($($code:tt)*) => {
            #[cfg(not(feature = "dev-ui"))]
            {
                $($code)*
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_platform_config_creation() {
        let config = CrossPlatformConfig::default();
        
        // Should have configurations for major platforms
        assert!(config.get_platform_config(Platform::Windows).is_some());
        assert!(config.get_platform_config(Platform::MacOS).is_some());
        assert!(config.get_platform_config(Platform::Linux).is_some());
    }

    #[test]
    fn test_feature_flags() {
        let flags = FeatureFlags::default();
        
        // Should have reasonable defaults
        let active = flags.active_features();
        assert!(!active.is_empty());
        
        // Should detect current platform features
        #[cfg(target_os = "windows")]
        assert!(flags.windows_native);
        
        #[cfg(target_os = "macos")]
        assert!(flags.macos_native);
        
        #[cfg(unix)]
        assert!(flags.unix_native);
    }

    #[test]
    fn test_cross_platform_layer() {
        let layer = CrossPlatformLayer::new();
        
        // Should detect current platform
        let platform = layer.current_platform();
        assert!(matches!(platform, 
            Platform::Windows | Platform::MacOS | Platform::Linux | Platform::Other(_)
        ));
        
        // Should have valid runtime config
        let config = layer.runtime_config();
        assert!(config.thread_count > 0);
    }

    #[test]
    fn test_file_watcher_config() {
        let layer = CrossPlatformLayer::new();
        let watcher_config = layer.file_watcher_config();
        
        // Should have reasonable configuration
        assert!(watcher_config.max_files > 0);
        assert!(watcher_config.debounce_timeout.as_millis() > 0);
        assert!(watcher_config.debounce_timeout.as_millis() < 1000);
    }

    #[test]
    fn test_cargo_features_generation() {
        let config = CrossPlatformConfig::default();
        let features = config.generate_cargo_features();
        
        // Should contain basic feature structure
        assert!(features.contains("[features]"));
        assert!(features.contains("default = []"));
    }

    #[test]
    fn test_platform_dependencies_generation() {
        let config = CrossPlatformConfig::default();
        let deps = config.generate_platform_dependencies();
        
        // Should contain platform-specific sections
        assert!(deps.contains("cfg(windows)"));
        assert!(deps.contains("cfg(unix)"));
        assert!(deps.contains("cfg(target_os = \"macos\")"));
    }

    #[test]
    fn test_conditional_compilation_macros() {
        // Test that macros compile (actual execution depends on platform)
        crate::windows_only! {
            let _windows_specific = "This only runs on Windows";
        }
        
        crate::macos_only! {
            let _macos_specific = "This only runs on macOS";
        }
        
        crate::linux_only! {
            let _linux_specific = "This only runs on Linux";
        }
        
        crate::unix_only! {
            let _unix_specific = "This only runs on Unix-like systems";
        }
        
        crate::dev_only! {
            let _dev_specific = "This only runs in development mode";
        }
        
        crate::production_only! {
            let _prod_specific = "This only runs in production mode";
        }
    }
}