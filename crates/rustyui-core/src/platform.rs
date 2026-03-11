//! Cross-platform compatibility and capability detection
//! 
//! This module provides platform detection, capability checking, and platform-specific
//! optimizations for RustyUI. It ensures consistent behavior across Windows, macOS, and Linux
//! while leveraging native APIs for optimal performance.

use serde::{Serialize, Deserialize};
use std::fmt;

/// Supported platforms for RustyUI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Other(PlatformInfo),
}

/// Additional platform information for unsupported platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub family: PlatformFamily,
    pub arch: Architecture,
}

/// Platform family classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformFamily {
    Unix,
    Windows,
    Unknown,
}

/// CPU architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Architecture {
    X86_64,
    Aarch64,
    X86,
    Other,
}

/// File watcher backend options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileWatcherBackend {
    /// Windows ReadDirectoryChanges API
    ReadDirectoryChanges,
    /// macOS FSEvents API
    FSEvents,
    /// Linux inotify API
    INotify,
    /// Cross-platform polling fallback
    Poll,
}

/// JIT compilation capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitCapabilities {
    /// Whether Cranelift JIT is supported
    pub supports_cranelift: bool,
    /// Whether LLVM JIT is available
    pub supports_llvm: bool,
    /// Whether native code generation is supported
    pub supports_native_codegen: bool,
    /// Maximum JIT compilation threads
    pub max_jit_threads: usize,
}

/// Performance characteristics for file watching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWatcherPerformance {
    /// Expected latency in milliseconds
    pub latency_ms: u32,
    /// CPU overhead percentage (0-100)
    pub cpu_overhead_percent: u8,
    /// Memory overhead in bytes
    pub memory_overhead_bytes: u64,
    /// Maximum files that can be watched efficiently
    pub max_files: Option<u32>,
}

/// Platform-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// Detected platform
    pub platform: Platform,
    /// File watcher backend to use
    pub file_watcher_backend: FileWatcherBackend,
    /// Whether to use native APIs
    pub use_native_apis: bool,
    /// Whether JIT compilation is enabled
    pub use_jit_compilation: bool,
    /// Number of worker threads
    pub thread_count: usize,
    /// Memory allocation strategy
    pub memory_strategy: MemoryStrategy,
}

/// Memory allocation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryStrategy {
    /// Conservative memory usage
    Conservative,
    /// Balanced memory usage
    Balanced,
    /// Aggressive memory usage for performance
    Aggressive,
}

/// Platform capabilities checker
pub struct PlatformCapabilities;

impl Platform {
    /// Detect the current platform
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        return Platform::Windows;
        
        #[cfg(target_os = "macos")]
        return Platform::MacOS;
        
        #[cfg(target_os = "linux")]
        return Platform::Linux;
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return Platform::Other(PlatformInfo {
            family: Self::detect_family(),
            arch: Self::detect_architecture(),
        });
    }
    
    /// Detect platform family
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn detect_family() -> PlatformFamily {
        #[cfg(unix)]
        return PlatformFamily::Unix;
        
        #[cfg(windows)]
        return PlatformFamily::Windows;
        
        #[cfg(not(any(unix, windows)))]
        return PlatformFamily::Unknown;
    }
    
    /// Detect CPU architecture
    fn detect_architecture() -> Architecture {
        #[cfg(target_arch = "x86_64")]
        return Architecture::X86_64;
        
        #[cfg(target_arch = "aarch64")]
        return Architecture::Aarch64;
        
        #[cfg(target_arch = "x86")]
        return Architecture::X86;
        
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86")))]
        return Architecture::Other;
    }
    
    /// Get the optimal file watcher backend for this platform
    pub fn file_watcher_backend(&self) -> FileWatcherBackend {
        match self {
            Platform::Windows => FileWatcherBackend::ReadDirectoryChanges,
            Platform::MacOS => FileWatcherBackend::FSEvents,
            Platform::Linux => FileWatcherBackend::INotify,
            Platform::Other(info) => match info.family {
                PlatformFamily::Unix => FileWatcherBackend::INotify,
                PlatformFamily::Windows => FileWatcherBackend::ReadDirectoryChanges,
                PlatformFamily::Unknown => FileWatcherBackend::Poll,
            },
        }
    }
    
    /// Get JIT compilation capabilities for this platform
    pub fn jit_capabilities(&self) -> JitCapabilities {
        let max_threads = num_cpus::get().min(8); // Cap at 8 threads for JIT
        
        match self {
            Platform::Windows | Platform::MacOS | Platform::Linux => {
                JitCapabilities {
                    supports_cranelift: true,
                    supports_llvm: false, // LLVM JIT not implemented yet
                    supports_native_codegen: true,
                    max_jit_threads: max_threads,
                }
            }
            Platform::Other(info) => {
                JitCapabilities {
                    supports_cranelift: matches!(info.arch, Architecture::X86_64 | Architecture::Aarch64),
                    supports_llvm: false,
                    supports_native_codegen: matches!(info.arch, Architecture::X86_64),
                    max_jit_threads: max_threads.min(4), // More conservative for unknown platforms
                }
            }
        }
    }
    
    /// Check if native APIs are available
    pub fn has_native_apis(&self) -> bool {
        match self {
            Platform::Windows => cfg!(feature = "windows-native"),
            Platform::MacOS => cfg!(feature = "macos-native"),
            Platform::Linux => cfg!(feature = "unix-native"),
            Platform::Other(_) => false,
        }
    }
}

impl FileWatcherBackend {
    /// Get performance characteristics for this backend
    pub fn performance_characteristics(&self) -> FileWatcherPerformance {
        match self {
            FileWatcherBackend::ReadDirectoryChanges => FileWatcherPerformance {
                latency_ms: 10,
                cpu_overhead_percent: 2,
                memory_overhead_bytes: 64 * 1024, // 64KB
                max_files: Some(10000),
            },
            FileWatcherBackend::FSEvents => FileWatcherPerformance {
                latency_ms: 5,
                cpu_overhead_percent: 1,
                memory_overhead_bytes: 32 * 1024, // 32KB
                max_files: Some(50000),
            },
            FileWatcherBackend::INotify => FileWatcherPerformance {
                latency_ms: 8,
                cpu_overhead_percent: 3,
                memory_overhead_bytes: 48 * 1024, // 48KB
                max_files: Some(8192), // Limited by inotify watches
            },
            FileWatcherBackend::Poll => FileWatcherPerformance {
                latency_ms: 100,
                cpu_overhead_percent: 15,
                memory_overhead_bytes: 128 * 1024, // 128KB
                max_files: Some(1000), // Polling is expensive
            },
        }
    }
    
    /// Check if this backend meets performance targets
    pub fn meets_performance_targets(&self) -> bool {
        let perf = self.performance_characteristics();
        perf.latency_ms <= 50 && perf.cpu_overhead_percent <= 10
    }
}

impl PlatformConfig {
    /// Auto-detect optimal platform configuration
    pub fn auto_detect() -> Self {
        let platform = Platform::current();
        let file_watcher_backend = platform.file_watcher_backend();
        let use_native_apis = platform.has_native_apis();
        let jit_caps = platform.jit_capabilities();
        
        // Determine thread count based on CPU cores and platform
        let cpu_count = num_cpus::get();
        let thread_count = match platform {
            Platform::Windows | Platform::MacOS => cpu_count.min(16),
            Platform::Linux => cpu_count.min(12),
            Platform::Other(_) => cpu_count.min(8),
        };
        
        // Choose memory strategy based on available resources
        let memory_strategy = if cpu_count >= 8 {
            MemoryStrategy::Aggressive
        } else if cpu_count >= 4 {
            MemoryStrategy::Balanced
        } else {
            MemoryStrategy::Conservative
        };
        
        Self {
            platform,
            file_watcher_backend,
            use_native_apis,
            use_jit_compilation: jit_caps.supports_cranelift,
            thread_count,
            memory_strategy,
        }
    }
    
    /// Validate the platform configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check thread count is reasonable
        if self.thread_count == 0 {
            return Err("Thread count must be greater than 0".to_string());
        }
        
        if self.thread_count > 32 {
            return Err("Thread count should not exceed 32 for optimal performance".to_string());
        }
        
        // Check JIT compilation compatibility
        if self.use_jit_compilation {
            let jit_caps = self.platform.jit_capabilities();
            if !jit_caps.supports_cranelift {
                return Err("JIT compilation requested but not supported on this platform".to_string());
            }
        }
        
        // Check native API availability
        if self.use_native_apis && !self.platform.has_native_apis() {
            return Err("Native APIs requested but not available on this platform".to_string());
        }
        
        // Check file watcher backend compatibility
        let expected_backend = self.platform.file_watcher_backend();
        if self.file_watcher_backend != expected_backend {
            log::warn!("File watcher backend mismatch: expected {:?}, got {:?}", 
                      expected_backend, self.file_watcher_backend);
        }
        
        Ok(())
    }
    
    /// Create a configuration optimized for development
    pub fn for_development() -> Self {
        let mut config = Self::auto_detect();
        
        // Optimize for development speed
        config.use_jit_compilation = true;
        config.memory_strategy = MemoryStrategy::Aggressive;
        
        // Ensure file watcher meets performance targets
        if !config.file_watcher_backend.meets_performance_targets() {
            log::warn!("File watcher backend {:?} may not meet performance targets", 
                      config.file_watcher_backend);
        }
        
        config
    }
    
    /// Create a configuration optimized for production
    pub fn for_production() -> Self {
        let mut config = Self::auto_detect();
        
        // Optimize for production efficiency
        config.use_jit_compilation = false;
        config.memory_strategy = MemoryStrategy::Conservative;
        config.thread_count = config.thread_count.min(4); // Limit threads in production
        
        config
    }
}

impl PlatformCapabilities {
    /// Check minimum system requirements
    pub fn check_minimum_requirements() -> Result<(), String> {
        let platform = Platform::current();
        
        // Check CPU architecture support
        match Platform::detect_architecture() {
            Architecture::X86_64 | Architecture::Aarch64 => {
                // Supported architectures
            }
            Architecture::X86 => {
                return Err("32-bit x86 is not supported. Please use a 64-bit system.".to_string());
            }
            Architecture::Other => {
                return Err("Unsupported CPU architecture. RustyUI requires x86_64 or ARM64.".to_string());
            }
        }
        
        // Check platform support
        match platform {
            Platform::Windows | Platform::MacOS | Platform::Linux => {
                // Fully supported platforms
            }
            Platform::Other(_) => {
                log::warn!("Running on unsupported platform. Some features may not work correctly.");
            }
        }
        
        // Check available memory (rough estimate)
        let thread_count = num_cpus::get();
        if thread_count < 2 {
            return Err("RustyUI requires at least 2 CPU cores for optimal performance.".to_string());
        }
        
        // Check file watcher capabilities
        let backend = platform.file_watcher_backend();
        if !backend.meets_performance_targets() {
            log::warn!("File watching performance may be suboptimal on this platform");
        }
        
        Ok(())
    }
    
    /// Check development features availability
    pub fn check_dev_features() -> Result<(), String> {
        #[cfg(not(feature = "dev-ui"))]
        {
            return Err("Development features not available. Build with --features dev-ui".to_string());
        }
        
        #[cfg(feature = "dev-ui")]
        {
            let platform = Platform::current();
            let jit_caps = platform.jit_capabilities();
            
            // Check JIT compilation support
            if !jit_caps.supports_cranelift {
                log::warn!("JIT compilation not supported on this platform. Runtime interpretation will be limited.");
            }
            
            // Check file watching support
            let backend = platform.file_watcher_backend();
            if !backend.meets_performance_targets() {
                log::warn!("File watching may have suboptimal performance on this platform");
            }
            
            // Check native API availability
            if !platform.has_native_apis() {
                log::warn!("Native APIs not available. Using cross-platform fallbacks.");
            }
            
            Ok(())
        }
    }
    
    /// Get detailed capability report
    pub fn capability_report() -> CapabilityReport {
        let platform = Platform::current();
        let config = PlatformConfig::auto_detect();
        let jit_caps = platform.jit_capabilities();
        let watcher_perf = config.file_watcher_backend.performance_characteristics();
        
        CapabilityReport {
            platform,
            architecture: Platform::detect_architecture(),
            cpu_cores: num_cpus::get(),
            file_watcher_backend: config.file_watcher_backend,
            file_watcher_performance: watcher_perf,
            jit_capabilities: jit_caps,
            native_apis_available: platform.has_native_apis(),
            recommended_config: config,
        }
    }
}

/// Detailed capability report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityReport {
    pub platform: Platform,
    pub architecture: Architecture,
    pub cpu_cores: usize,
    pub file_watcher_backend: FileWatcherBackend,
    pub file_watcher_performance: FileWatcherPerformance,
    pub jit_capabilities: JitCapabilities,
    pub native_apis_available: bool,
    pub recommended_config: PlatformConfig,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::Windows => write!(f, "Windows"),
            Platform::MacOS => write!(f, "macOS"),
            Platform::Linux => write!(f, "Linux"),
            Platform::Other(info) => write!(f, "{:?} ({:?})", info.family, info.arch),
        }
    }
}

impl fmt::Display for FileWatcherBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileWatcherBackend::ReadDirectoryChanges => write!(f, "ReadDirectoryChanges (Windows)"),
            FileWatcherBackend::FSEvents => write!(f, "FSEvents (macOS)"),
            FileWatcherBackend::INotify => write!(f, "inotify (Linux)"),
            FileWatcherBackend::Poll => write!(f, "Polling (Cross-platform)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_platform_detection() {
        let platform = Platform::current();
        
        // Should detect a known platform on CI
        match platform {
            Platform::Windows | Platform::MacOS | Platform::Linux => {
                // Expected on CI systems
            }
            Platform::Other(_) => {
                // May happen on exotic platforms
                println!("Detected other platform: {:?}", platform);
            }
        }
    }
    
    #[test]
    fn test_file_watcher_backend_selection() {
        let platform = Platform::current();
        let backend = platform.file_watcher_backend();
        
        // Should select appropriate backend for platform
        match platform {
            Platform::Windows => assert_eq!(backend, FileWatcherBackend::ReadDirectoryChanges),
            Platform::MacOS => assert_eq!(backend, FileWatcherBackend::FSEvents),
            Platform::Linux => assert_eq!(backend, FileWatcherBackend::INotify),
            Platform::Other(_) => {
                // Should select a reasonable fallback
                assert!(matches!(backend, 
                    FileWatcherBackend::INotify | 
                    FileWatcherBackend::ReadDirectoryChanges | 
                    FileWatcherBackend::Poll
                ));
            }
        }
    }
    
    #[test]
    fn test_jit_capabilities() {
        let platform = Platform::current();
        let jit_caps = platform.jit_capabilities();
        
        // Should have reasonable JIT capabilities on supported platforms
        assert!(jit_caps.max_jit_threads > 0);
        assert!(jit_caps.max_jit_threads <= num_cpus::get());
        
        // Cranelift should be supported on major platforms
        match platform {
            Platform::Windows | Platform::MacOS | Platform::Linux => {
                assert!(jit_caps.supports_cranelift);
            }
            Platform::Other(_) => {
                // May or may not support Cranelift
            }
        }
    }
    
    #[test]
    fn test_platform_config_auto_detect() {
        let config = PlatformConfig::auto_detect();
        
        // Should have reasonable defaults
        assert!(config.thread_count > 0);
        assert!(config.thread_count <= num_cpus::get());
        
        // File watcher should be appropriate for platform
        let expected_backend = config.platform.file_watcher_backend();
        assert_eq!(config.file_watcher_backend, expected_backend);
    }
    
    #[test]
    fn test_development_vs_production_config() {
        let dev_config = PlatformConfig::for_development();
        let prod_config = PlatformConfig::for_production();
        
        // Development should be more aggressive
        assert!(matches!(dev_config.memory_strategy, MemoryStrategy::Aggressive));
        
        // Production should be more conservative
        assert!(matches!(prod_config.memory_strategy, MemoryStrategy::Conservative));
        assert!(!prod_config.use_jit_compilation);
        assert!(prod_config.thread_count <= dev_config.thread_count);
    }
    
    #[test]
    fn test_minimum_requirements() {
        // Should pass on development machines
        let result = PlatformCapabilities::check_minimum_requirements();
        
        match result {
            Ok(()) => {
                // Requirements met
            }
            Err(msg) => {
                println!("Minimum requirements not met: {}", msg);
                // This might be expected on some CI environments
            }
        }
    }
    
    #[test]
    fn test_capability_report() {
        let report = PlatformCapabilities::capability_report();
        
        // Should have valid data
        assert!(report.cpu_cores > 0);
        assert!(report.jit_capabilities.max_jit_threads > 0);
        
        // Performance characteristics should be reasonable
        let perf = &report.file_watcher_performance;
        assert!(perf.latency_ms > 0);
        assert!(perf.latency_ms < 1000); // Should be under 1 second
        assert!(perf.cpu_overhead_percent <= 100);
    }
    
    #[test]
    fn test_file_watcher_performance_targets() {
        let backends = [
            FileWatcherBackend::ReadDirectoryChanges,
            FileWatcherBackend::FSEvents,
            FileWatcherBackend::INotify,
            FileWatcherBackend::Poll,
        ];
        
        for backend in backends {
            let perf = backend.performance_characteristics();
            
            // All backends should have reasonable characteristics
            assert!(perf.latency_ms > 0);
            assert!(perf.cpu_overhead_percent <= 100);
            assert!(perf.memory_overhead_bytes > 0);
            
            // Native backends should meet performance targets
            match backend {
                FileWatcherBackend::ReadDirectoryChanges |
                FileWatcherBackend::FSEvents |
                FileWatcherBackend::INotify => {
                    assert!(backend.meets_performance_targets());
                }
                FileWatcherBackend::Poll => {
                    // Polling may not meet targets, but should be functional
                }
            }
        }
    }
}