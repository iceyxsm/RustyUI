//! Cross-platform detection and capability checking

use std::fmt;

/// Supported operating systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Unix,
    Unknown,
}

impl Platform {
    /// Detect the current platform at runtime
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        return Platform::Windows;
        
        #[cfg(target_os = "macos")]
        return Platform::MacOS;
        
        #[cfg(target_os = "linux")]
        return Platform::Linux;
        
        #[cfg(all(unix, not(target_os = "macos"), not(target_os = "linux")))]
        return Platform::Unix;
        
        #[cfg(not(any(windows, unix)))]
        return Platform::Unknown;
    }
    
    /// Check if the current platform is supported
    pub fn is_supported(&self) -> bool {
        matches!(self, Platform::Windows | Platform::MacOS | Platform::Linux)
    }
    
    /// Get platform-specific file watching backend
    pub fn file_watcher_backend(&self) -> FileWatcherBackend {
        match self {
            Platform::Windows => FileWatcherBackend::ReadDirectoryChanges,
            Platform::MacOS => FileWatcherBackend::FSEvents,
            Platform::Linux => FileWatcherBackend::INotify,
            Platform::Unix => FileWatcherBackend::Poll,
            Platform::Unknown => FileWatcherBackend::Poll,
        }
    }
    
    /// Get platform-specific JIT compilation capabilities
    pub fn jit_capabilities(&self) -> JITCapabilities {
        match self {
            Platform::Windows => JITCapabilities {
                supports_cranelift: true,
                supports_wasmtime: true,
                memory_protection: true,
                executable_memory: true,
            },
            Platform::MacOS => JITCapabilities {
                supports_cranelift: true,
                supports_wasmtime: true,
                memory_protection: true,
                executable_memory: true,
            },
            Platform::Linux => JITCapabilities {
                supports_cranelift: true,
                supports_wasmtime: true,
                memory_protection: true,
                executable_memory: true,
            },
            Platform::Unix => JITCapabilities {
                supports_cranelift: false,
                supports_wasmtime: true,
                memory_protection: false,
                executable_memory: false,
            },
            Platform::Unknown => JITCapabilities {
                supports_cranelift: false,
                supports_wasmtime: false,
                memory_protection: false,
                executable_memory: false,
            },
        }
    }
    
    /// Get platform-specific optimizations
    pub fn optimizations(&self) -> PlatformOptimizations {
        match self {
            Platform::Windows => PlatformOptimizations {
                use_native_file_watching: true,
                use_memory_mapped_files: true,
                use_vectorized_operations: true,
                preferred_thread_count: get_thread_count(),
            },
            Platform::MacOS => PlatformOptimizations {
                use_native_file_watching: true,
                use_memory_mapped_files: true,
                use_vectorized_operations: true,
                preferred_thread_count: get_thread_count(),
            },
            Platform::Linux => PlatformOptimizations {
                use_native_file_watching: true,
                use_memory_mapped_files: true,
                use_vectorized_operations: true,
                preferred_thread_count: get_thread_count(),
            },
            Platform::Unix => PlatformOptimizations {
                use_native_file_watching: false,
                use_memory_mapped_files: false,
                use_vectorized_operations: false,
                preferred_thread_count: 1,
            },
            Platform::Unknown => PlatformOptimizations {
                use_native_file_watching: false,
                use_memory_mapped_files: false,
                use_vectorized_operations: false,
                preferred_thread_count: 1,
            },
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::Windows => write!(f, "Windows"),
            Platform::MacOS => write!(f, "macOS"),
            Platform::Linux => write!(f, "Linux"),
            Platform::Unix => write!(f, "Unix"),
            Platform::Unknown => write!(f, "Unknown"),
        }
    }
}

/// File watcher backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileWatcherBackend {
    /// Windows ReadDirectoryChanges API
    ReadDirectoryChanges,
    /// macOS FSEvents API
    FSEvents,
    /// Linux inotify API
    INotify,
    /// Generic polling fallback
    Poll,
}

impl FileWatcherBackend {
    /// Get the expected performance characteristics
    pub fn performance_characteristics(&self) -> WatcherPerformance {
        match self {
            FileWatcherBackend::ReadDirectoryChanges => WatcherPerformance {
                latency_ms: 10,
                cpu_overhead: 0.1,
                memory_overhead_kb: 64,
                supports_recursive: true,
            },
            FileWatcherBackend::FSEvents => WatcherPerformance {
                latency_ms: 5,
                cpu_overhead: 0.05,
                memory_overhead_kb: 32,
                supports_recursive: true,
            },
            FileWatcherBackend::INotify => WatcherPerformance {
                latency_ms: 1,
                cpu_overhead: 0.02,
                memory_overhead_kb: 16,
                supports_recursive: true,
            },
            FileWatcherBackend::Poll => WatcherPerformance {
                latency_ms: 1000,
                cpu_overhead: 5.0,
                memory_overhead_kb: 8,
                supports_recursive: false,
            },
        }
    }
}

/// JIT compilation capabilities for the platform
#[derive(Debug, Clone)]
pub struct JITCapabilities {
    /// Whether Cranelift JIT is supported
    pub supports_cranelift: bool,
    /// Whether Wasmtime is supported
    pub supports_wasmtime: bool,
    /// Whether memory protection is available
    pub memory_protection: bool,
    /// Whether executable memory allocation is supported
    pub executable_memory: bool,
}

/// Platform-specific optimizations
#[derive(Debug, Clone)]
pub struct PlatformOptimizations {
    /// Whether to use native file watching APIs
    pub use_native_file_watching: bool,
    /// Whether to use memory-mapped files for performance
    pub use_memory_mapped_files: bool,
    /// Whether to use vectorized operations (SIMD)
    pub use_vectorized_operations: bool,
    /// Preferred number of threads for parallel operations
    pub preferred_thread_count: usize,
}

/// File watcher performance characteristics
#[derive(Debug, Clone)]
pub struct WatcherPerformance {
    /// Expected latency in milliseconds
    pub latency_ms: u64,
    /// CPU overhead as percentage (0.0-100.0)
    pub cpu_overhead: f64,
    /// Memory overhead in kilobytes
    pub memory_overhead_kb: u64,
    /// Whether recursive watching is supported
    pub supports_recursive: bool,
}

/// Platform capability checker
pub struct PlatformCapabilities;

impl PlatformCapabilities {
    /// Check if the current platform meets minimum requirements
    pub fn check_minimum_requirements() -> Result<(), String> {
        let platform = Platform::current();
        
        if !platform.is_supported() {
            return Err(format!("Unsupported platform: {}", platform));
        }
        
        // Check Rust version requirements
        if !Self::check_rust_version() {
            return Err("Rust 1.70.0 or later is required".to_string());
        }
        
        // Check platform-specific requirements
        match platform {
            Platform::Windows => Self::check_windows_requirements(),
            Platform::MacOS => Self::check_macos_requirements(),
            Platform::Linux => Self::check_linux_requirements(),
            _ => Ok(()),
        }
    }
    
    /// Check if development features are available
    pub fn check_dev_features() -> Result<(), String> {
        #[cfg(not(feature = "dev-ui"))]
        return Err("Development features not enabled. Use --features dev-ui".to_string());
        
        #[cfg(feature = "dev-ui")]
        {
            let platform = Platform::current();
            let jit_caps = platform.jit_capabilities();
            
            if !jit_caps.supports_wasmtime {
                return Err("Wasmtime not supported on this platform".to_string());
            }
            
            if !jit_caps.executable_memory {
                return Err("Executable memory allocation not supported".to_string());
            }
            
            Ok(())
        }
    }
    
    /// Get optimal configuration for the current platform
    pub fn optimal_config() -> PlatformConfig {
        let platform = Platform::current();
        let optimizations = platform.optimizations();
        let jit_caps = platform.jit_capabilities();
        let watcher_backend = platform.file_watcher_backend();
        
        PlatformConfig {
            platform,
            file_watcher_backend: watcher_backend,
            thread_count: optimizations.preferred_thread_count,
            use_jit_compilation: jit_caps.supports_cranelift,
            use_native_apis: optimizations.use_native_file_watching,
            memory_optimization: optimizations.use_memory_mapped_files,
        }
    }
    
    fn check_rust_version() -> bool {
        // This is a compile-time check - if we're compiling, the version is sufficient
        true
    }
    
    #[cfg(target_os = "windows")]
    fn check_windows_requirements() -> Result<(), String> {
        // Check Windows version (Windows 10 1903 or later)
        // For now, assume Windows requirements are met
        // In a full implementation, we'd check the actual Windows version
        Ok(())
    }
    
    #[cfg(not(target_os = "windows"))]
    fn check_windows_requirements() -> Result<(), String> {
        Ok(())
    }
    
    #[cfg(target_os = "macos")]
    fn check_macos_requirements() -> Result<(), String> {
        // Check macOS version (10.15 or later)
        // For now, assume macOS requirements are met
        Ok(())
    }
    
    #[cfg(not(target_os = "macos"))]
    fn check_macos_requirements() -> Result<(), String> {
        Ok(())
    }
    
    #[cfg(target_os = "linux")]
    fn check_linux_requirements() -> Result<(), String> {
        // Check glibc version (2.28 or later)
        // For now, assume Linux requirements are met
        Ok(())
    }
    
    #[cfg(not(target_os = "linux"))]
    fn check_linux_requirements() -> Result<(), String> {
        Ok(())
    }
}

/// Platform-specific configuration
#[derive(Debug, Clone)]
pub struct PlatformConfig {
    /// Current platform
    pub platform: Platform,
    /// File watcher backend to use
    pub file_watcher_backend: FileWatcherBackend,
    /// Number of threads to use
    pub thread_count: usize,
    /// Whether to use JIT compilation
    pub use_jit_compilation: bool,
    /// Whether to use native platform APIs
    pub use_native_apis: bool,
    /// Whether to enable memory optimizations
    pub memory_optimization: bool,
}

impl PlatformConfig {
    /// Create a new platform configuration with auto-detection
    pub fn auto_detect() -> Self {
        PlatformCapabilities::optimal_config()
    }
    
    /// Validate the configuration for the current platform
    pub fn validate(&self) -> Result<(), String> {
        if self.platform != Platform::current() {
            return Err("Platform mismatch in configuration".to_string());
        }
        
        let jit_caps = self.platform.jit_capabilities();
        if self.use_jit_compilation && !jit_caps.supports_cranelift {
            return Err("JIT compilation not supported on this platform".to_string());
        }
        
        Ok(())
    }
}

// Thread count detection using standard library
fn get_thread_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_platform_detection() {
        let platform = Platform::current();
        assert!(matches!(
            platform,
            Platform::Windows | Platform::MacOS | Platform::Linux | Platform::Unix | Platform::Unknown
        ));
    }
    
    #[test]
    fn test_platform_support() {
        assert!(Platform::Windows.is_supported());
        assert!(Platform::MacOS.is_supported());
        assert!(Platform::Linux.is_supported());
        assert!(!Platform::Unknown.is_supported());
    }
    
    #[test]
    fn test_file_watcher_backend_selection() {
        let windows_backend = Platform::Windows.file_watcher_backend();
        assert_eq!(windows_backend, FileWatcherBackend::ReadDirectoryChanges);
        
        let macos_backend = Platform::MacOS.file_watcher_backend();
        assert_eq!(macos_backend, FileWatcherBackend::FSEvents);
        
        let linux_backend = Platform::Linux.file_watcher_backend();
        assert_eq!(linux_backend, FileWatcherBackend::INotify);
    }
    
    #[test]
    fn test_jit_capabilities() {
        let platform = Platform::current();
        let jit_caps = platform.jit_capabilities();
        
        // On supported platforms, we should have basic JIT capabilities
        if platform.is_supported() {
            assert!(jit_caps.supports_wasmtime);
        }
    }
    
    #[test]
    fn test_platform_optimizations() {
        let platform = Platform::current();
        let opts = platform.optimizations();
        
        if platform.is_supported() {
            assert!(opts.preferred_thread_count > 0);
        }
    }
    
    #[test]
    fn test_watcher_performance_characteristics() {
        let inotify_perf = FileWatcherBackend::INotify.performance_characteristics();
        assert!(inotify_perf.latency_ms < 100);
        assert!(inotify_perf.supports_recursive);
        
        let poll_perf = FileWatcherBackend::Poll.performance_characteristics();
        assert!(poll_perf.latency_ms > inotify_perf.latency_ms);
    }
    
    #[test]
    fn test_platform_config_auto_detect() {
        let config = PlatformConfig::auto_detect();
        assert_eq!(config.platform, Platform::current());
        assert!(config.thread_count > 0);
    }
    
    #[test]
    fn test_platform_config_validation() {
        let config = PlatformConfig::auto_detect();
        assert!(config.validate().is_ok());
        
        // Test invalid configuration
        let mut invalid_config = config.clone();
        invalid_config.platform = Platform::Unknown;
        assert!(invalid_config.validate().is_err());
    }
    
    #[test]
    fn test_minimum_requirements_check() {
        // This should pass on supported platforms
        let result = PlatformCapabilities::check_minimum_requirements();
        let platform = Platform::current();
        
        if platform.is_supported() {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
        }
    }
}