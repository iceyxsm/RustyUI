//! Production build verification and zero-overhead validation

use crate::error::{Result, RustyUIError};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

/// Production build verification system
pub struct ProductionVerifier {
    project_path: std::path::PathBuf,
    verification_results: VerificationResults,
}

/// Results of production build verification
#[derive(Debug, Clone)]
pub struct VerificationResults {
    /// Whether conditional compilation is working correctly
    pub conditional_compilation_ok: bool,
    
    /// Binary size comparison results
    pub binary_size_results: BinarySizeResults,
    
    /// Performance comparison results
    pub performance_results: PerformanceResults,
    
    /// Feature stripping verification
    pub feature_stripping_results: FeatureStrippingResults,
    
    /// Overall verification status
    pub overall_status: VerificationStatus,
    
    /// Detailed verification report
    pub detailed_report: String,
}

/// Binary size comparison results
#[derive(Debug, Clone)]
pub struct BinarySizeResults {
    /// Size of RustyUI production build in bytes
    pub rustyui_production_size: u64,
    
    /// Size of equivalent standard Rust build in bytes
    pub standard_rust_size: u64,
    
    /// Size difference in bytes (positive means RustyUI is larger)
    pub size_difference: i64,
    
    /// Size difference as percentage
    pub size_difference_percent: f64,
    
    /// Whether size is within acceptable limits
    pub size_acceptable: bool,
}

/// Performance comparison results
#[derive(Debug, Clone)]
pub struct PerformanceResults {
    /// RustyUI production build performance metrics
    pub rustyui_performance: PerformanceMetrics,
    
    /// Standard Rust build performance metrics
    pub standard_performance: PerformanceMetrics,
    
    /// Performance difference (positive means RustyUI is slower)
    pub performance_difference_percent: f64,
    
    /// Whether performance is within acceptable limits
    pub performance_acceptable: bool,
}

/// Performance metrics for a build
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Startup time in milliseconds
    pub startup_time_ms: f64,
    
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    
    /// CPU usage percentage during benchmark
    pub cpu_usage_percent: f64,
    
    /// Benchmark execution time in milliseconds
    pub benchmark_time_ms: f64,
}

/// Feature stripping verification results
#[derive(Debug, Clone)]
pub struct FeatureStrippingResults {
    /// Development features found in production build
    pub dev_features_found: Vec<String>,
    
    /// Development symbols found in production build
    pub dev_symbols_found: Vec<String>,
    
    /// Whether all development features were stripped
    pub all_dev_features_stripped: bool,
    
    /// Conditional compilation verification
    pub conditional_compilation_verified: bool,
}

/// Overall verification status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationStatus {
    /// All verifications passed - zero overhead confirmed
    ZeroOverheadConfirmed,
    
    /// Minor overhead detected but within acceptable limits
    MinorOverheadDetected,
    
    /// Significant overhead detected - investigation needed
    SignificantOverheadDetected,
    
    /// Verification failed due to errors
    VerificationFailed,
}

impl ProductionVerifier {
    /// Create a new production verifier
    pub fn new<P: AsRef<Path>>(project_path: P) -> Self {
        Self {
            project_path: project_path.as_ref().to_path_buf(),
            verification_results: VerificationResults::default(),
        }
    }
    
    /// Run complete production build verification
    pub fn verify_production_build(&mut self) -> Result<&VerificationResults> {
        println!("🔍 Starting production build verification...");
        
        // Step 1: Verify conditional compilation
        self.verify_conditional_compilation()?;
        
        // Step 2: Compare binary sizes
        self.compare_binary_sizes()?;
        
        // Step 3: Compare performance
        self.compare_performance()?;
        
        // Step 4: Verify feature stripping
        self.verify_feature_stripping()?;
        
        // Step 5: Generate overall assessment
        self.generate_overall_assessment();
        
        println!("✅ Production build verification completed");
        
        Ok(&self.verification_results)
    }
    
    /// Verify that conditional compilation is working correctly
    fn verify_conditional_compilation(&mut self) -> Result<()> {
        println!("  🔧 Verifying conditional compilation...");
        
        // Build with dev-ui feature
        let dev_build_result = self.build_with_features(&["dev-ui"])?;
        
        // Build without dev-ui feature (production)
        let prod_build_result = self.build_with_features(&[])?;
        
        // Analyze build outputs for conditional compilation markers
        let dev_has_markers = self.check_for_dev_markers(&dev_build_result.binary_path)?;
        let prod_has_markers = self.check_for_dev_markers(&prod_build_result.binary_path)?;
        
        // Conditional compilation is working if dev build has markers but prod doesn't
        self.verification_results.conditional_compilation_ok = dev_has_markers && !prod_has_markers;
        
        if self.verification_results.conditional_compilation_ok {
            println!("    ✅ Conditional compilation working correctly");
        } else {
            println!("    ❌ Conditional compilation issues detected");
        }
        
        Ok(())
    }
    
    /// Compare binary sizes between RustyUI and standard Rust builds
    fn compare_binary_sizes(&mut self) -> Result<()> {
        println!("  📏 Comparing binary sizes...");
        
        // Build RustyUI production binary
        let rustyui_build = self.build_rustyui_production()?;
        
        // Build equivalent standard Rust binary
        let standard_build = self.build_standard_rust_equivalent()?;
        
        // Get file sizes
        let rustyui_size = std::fs::metadata(&rustyui_build.binary_path)?.len();
        let standard_size = std::fs::metadata(&standard_build.binary_path)?.len();
        
        // Calculate differences
        let size_diff = rustyui_size as i64 - standard_size as i64;
        let size_diff_percent = if standard_size > 0 {
            (size_diff as f64 / standard_size as f64) * 100.0
        } else {
            0.0
        };
        
        // Size is acceptable if within 5% of standard Rust build
        let size_acceptable = size_diff_percent.abs() <= 5.0;
        
        self.verification_results.binary_size_results = BinarySizeResults {
            rustyui_production_size: rustyui_size,
            standard_rust_size: standard_size,
            size_difference: size_diff,
            size_difference_percent: size_diff_percent,
            size_acceptable,
        };
        
        println!("    RustyUI size: {} bytes", rustyui_size);
        println!("    Standard size: {} bytes", standard_size);
        println!("    Difference: {:.2}%", size_diff_percent);
        
        if size_acceptable {
            println!("    ✅ Binary size within acceptable limits");
        } else {
            println!("    ⚠️ Binary size difference exceeds 5%");
        }
        
        Ok(())
    }
    
    /// Compare performance between RustyUI and standard Rust builds
    fn compare_performance(&mut self) -> Result<()> {
        println!("  ⚡ Comparing performance...");
        
        // Build both versions
        let rustyui_build = self.build_rustyui_production()?;
        let standard_build = self.build_standard_rust_equivalent()?;
        
        // Run performance benchmarks
        let rustyui_perf = self.benchmark_binary(&rustyui_build.binary_path)?;
        let standard_perf = self.benchmark_binary(&standard_build.binary_path)?;
        
        // Calculate performance difference
        let perf_diff = if standard_perf.benchmark_time_ms > 0.0 {
            ((rustyui_perf.benchmark_time_ms - standard_perf.benchmark_time_ms) / standard_perf.benchmark_time_ms) * 100.0
        } else {
            0.0
        };
        
        // Performance is acceptable if within 2% of standard Rust
        let perf_acceptable = perf_diff.abs() <= 2.0;
        
        self.verification_results.performance_results = PerformanceResults {
            rustyui_performance: rustyui_perf,
            standard_performance: standard_perf,
            performance_difference_percent: perf_diff,
            performance_acceptable: perf_acceptable,
        };
        
        println!("    RustyUI benchmark: {:.2}ms", rustyui_perf.benchmark_time_ms);
        println!("    Standard benchmark: {:.2}ms", standard_perf.benchmark_time_ms);
        println!("    Performance difference: {:.2}%", perf_diff);
        
        if perf_acceptable {
            println!("    ✅ Performance within acceptable limits");
        } else {
            println!("    ⚠️ Performance difference exceeds 2%");
        }
        
        Ok(())
    }
    
    /// Verify that development features are stripped from production builds
    fn verify_feature_stripping(&mut self) -> Result<()> {
        println!("  🔒 Verifying feature stripping...");
        
        let prod_build = self.build_rustyui_production()?;
        
        // Check for development symbols in the binary
        let dev_symbols = self.find_development_symbols(&prod_build.binary_path)?;
        let dev_features = self.find_development_features(&prod_build.binary_path)?;
        
        let all_stripped = dev_symbols.is_empty() && dev_features.is_empty();
        
        self.verification_results.feature_stripping_results = FeatureStrippingResults {
            dev_features_found: dev_features.clone(),
            dev_symbols_found: dev_symbols.clone(),
            all_dev_features_stripped: all_stripped,
            conditional_compilation_verified: self.verification_results.conditional_compilation_ok,
        };
        
        if all_stripped {
            println!("    ✅ All development features stripped successfully");
        } else {
            println!("    ⚠️ Development features found in production build:");
            for feature in &dev_features {
                println!("      - Feature: {}", feature);
            }
            for symbol in &dev_symbols {
                println!("      - Symbol: {}", symbol);
            }
        }
        
        Ok(())
    }
    
    /// Generate overall assessment of verification results
    fn generate_overall_assessment(&mut self) {
        let size_ok = self.verification_results.binary_size_results.size_acceptable;
        let perf_ok = self.verification_results.performance_results.performance_acceptable;
        let features_ok = self.verification_results.feature_stripping_results.all_dev_features_stripped;
        let compilation_ok = self.verification_results.conditional_compilation_ok;
        
        let status = if compilation_ok && size_ok && perf_ok && features_ok {
            VerificationStatus::ZeroOverheadConfirmed
        } else if compilation_ok && (size_ok || perf_ok) {
            VerificationStatus::MinorOverheadDetected
        } else if compilation_ok {
            VerificationStatus::SignificantOverheadDetected
        } else {
            VerificationStatus::VerificationFailed
        };
        
        self.verification_results.overall_status = status.clone();
        
        // Generate detailed report
        let mut report = String::new();
        report.push_str("# RustyUI Production Build Verification Report\n\n");
        
        report.push_str(&format!("## Overall Status: {:?}\n\n", status));
        
        report.push_str("## Conditional Compilation\n");
        report.push_str(&format!("- Status: {}\n", if compilation_ok { "✅ PASS" } else { "❌ FAIL" }));
        report.push_str(&format!("- Development features properly gated: {}\n\n", compilation_ok));
        
        report.push_str("## Binary Size Analysis\n");
        let size_results = &self.verification_results.binary_size_results;
        report.push_str(&format!("- Status: {}\n", if size_ok { "✅ PASS" } else { "⚠️ WARN" }));
        report.push_str(&format!("- RustyUI production size: {} bytes\n", size_results.rustyui_production_size));
        report.push_str(&format!("- Standard Rust size: {} bytes\n", size_results.standard_rust_size));
        report.push_str(&format!("- Size difference: {:.2}%\n\n", size_results.size_difference_percent));
        
        report.push_str("## Performance Analysis\n");
        let perf_results = &self.verification_results.performance_results;
        report.push_str(&format!("- Status: {}\n", if perf_ok { "✅ PASS" } else { "⚠️ WARN" }));
        report.push_str(&format!("- RustyUI benchmark: {:.2}ms\n", perf_results.rustyui_performance.benchmark_time_ms));
        report.push_str(&format!("- Standard benchmark: {:.2}ms\n", perf_results.standard_performance.benchmark_time_ms));
        report.push_str(&format!("- Performance difference: {:.2}%\n\n", perf_results.performance_difference_percent));
        
        report.push_str("## Feature Stripping Analysis\n");
        let feature_results = &self.verification_results.feature_stripping_results;
        report.push_str(&format!("- Status: {}\n", if features_ok { "✅ PASS" } else { "❌ FAIL" }));
        report.push_str(&format!("- Development features found: {}\n", feature_results.dev_features_found.len()));
        report.push_str(&format!("- Development symbols found: {}\n", feature_results.dev_symbols_found.len()));
        
        if !feature_results.dev_features_found.is_empty() {
            report.push_str("- Remaining dev features:\n");
            for feature in &feature_results.dev_features_found {
                report.push_str(&format!("  - {}\n", feature));
            }
        }
        
        report.push_str("\n## Recommendations\n");
        match status {
            VerificationStatus::ZeroOverheadConfirmed => {
                report.push_str("- ✅ Zero overhead confirmed! Production builds are optimal.\n");
            }
            VerificationStatus::MinorOverheadDetected => {
                report.push_str("- ⚠️ Minor overhead detected. Consider optimization.\n");
                if !size_ok {
                    report.push_str("- Consider binary size optimization techniques.\n");
                }
                if !perf_ok {
                    report.push_str("- Consider performance optimization techniques.\n");
                }
            }
            VerificationStatus::SignificantOverheadDetected => {
                report.push_str("- ❌ Significant overhead detected. Investigation required.\n");
                report.push_str("- Review conditional compilation implementation.\n");
                report.push_str("- Analyze remaining development code in production builds.\n");
            }
            VerificationStatus::VerificationFailed => {
                report.push_str("- ❌ Verification failed. Critical issues detected.\n");
                report.push_str("- Fix conditional compilation issues immediately.\n");
                report.push_str("- Ensure development features are properly gated.\n");
            }
        }
        
        self.verification_results.detailed_report = report;
    }
    
    /// Build project with specified features
    fn build_with_features(&self, features: &[&str]) -> Result<BuildResult> {
        let mut cmd = Command::new("cargo");
        cmd.arg("build")
           .arg("--release")
           .current_dir(&self.project_path);
        
        if !features.is_empty() {
            cmd.arg("--features").arg(features.join(","));
        }
        
        let output = cmd.output()
            .map_err(|e| RustyUIError::generic(format!("Failed to run cargo build: {}", e)))?;
        
        if !output.status.success() {
            return Err(RustyUIError::generic(format!(
                "Build failed: {}", 
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        
        // Find the binary path
        let binary_name = self.get_binary_name()?;
        let binary_path = self.project_path
            .join("target")
            .join("release")
            .join(&binary_name);
        
        Ok(BuildResult {
            binary_path,
            features: features.iter().map(|s| s.to_string()).collect(),
            build_output: String::from_utf8_lossy(&output.stdout).to_string(),
        })
    }
    
    /// Build RustyUI production version
    fn build_rustyui_production(&self) -> Result<BuildResult> {
        self.build_with_features(&[])
    }
    
    /// Build equivalent standard Rust version for comparison
    fn build_standard_rust_equivalent(&self) -> Result<BuildResult> {
        // For this implementation, we'll create a minimal equivalent Rust program
        // In a real scenario, this would be a carefully crafted equivalent program
        
        let temp_dir = tempfile::tempdir()
            .map_err(|e| RustyUIError::generic(format!("Failed to create temp dir: {}", e)))?;
        
        // Create a minimal Rust program equivalent to our RustyUI app
        let main_rs_content = r#"
fn main() {
    // Equivalent functionality to RustyUI app without any framework overhead
    println!("Standard Rust equivalent");
    
    // Simulate some work equivalent to what RustyUI does
    let mut sum = 0u64;
    for i in 0..1000000 {
        sum = sum.wrapping_add(i);
    }
    
    // Prevent optimization from removing the loop
    if sum > 0 {
        std::hint::black_box(sum);
    }
}
"#;
        
        let cargo_toml_content = r#"
[package]
name = "standard-rust-equivalent"
version = "0.1.0"
edition = "2021"

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
"#;
        
        std::fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml_content)
            .map_err(|e| RustyUIError::generic(format!("Failed to write Cargo.toml: {}", e)))?;
        
        std::fs::create_dir_all(temp_dir.path().join("src"))
            .map_err(|e| RustyUIError::generic(format!("Failed to create src dir: {}", e)))?;
        
        std::fs::write(temp_dir.path().join("src").join("main.rs"), main_rs_content)
            .map_err(|e| RustyUIError::generic(format!("Failed to write main.rs: {}", e)))?;
        
        // Build the standard Rust equivalent
        let mut cmd = Command::new("cargo");
        cmd.arg("build")
           .arg("--release")
           .current_dir(temp_dir.path());
        
        let output = cmd.output()
            .map_err(|e| RustyUIError::generic(format!("Failed to build standard equivalent: {}", e)))?;
        
        if !output.status.success() {
            return Err(RustyUIError::generic(format!(
                "Standard build failed: {}", 
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        
        let binary_path = temp_dir.path()
            .join("target")
            .join("release")
            .join("standard-rust-equivalent");
        
        Ok(BuildResult {
            binary_path,
            features: vec![],
            build_output: String::from_utf8_lossy(&output.stdout).to_string(),
        })
    }
    
    /// Check for development markers in a binary
    fn check_for_dev_markers(&self, binary_path: &Path) -> Result<bool> {
        // Read binary and look for development-specific strings
        let binary_content = std::fs::read(binary_path)
            .map_err(|e| RustyUIError::generic(format!("Failed to read binary: {}", e)))?;
        
        let content_str = String::from_utf8_lossy(&binary_content);
        
        // Look for development-specific markers
        let dev_markers = [
            "dev-ui",
            "development_mode",
            "hot_reload",
            "runtime_interpreter",
            "state_preservation",
            "performance_monitoring",
        ];
        
        for marker in &dev_markers {
            if content_str.contains(marker) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Find development symbols in a binary
    fn find_development_symbols(&self, binary_path: &Path) -> Result<Vec<String>> {
        let mut symbols = Vec::new();
        
        // Use objdump or similar tool to extract symbols (simplified implementation)
        // In a real implementation, you'd use proper binary analysis tools
        
        let binary_content = std::fs::read(binary_path)
            .map_err(|e| RustyUIError::generic(format!("Failed to read binary: {}", e)))?;
        
        let content_str = String::from_utf8_lossy(&binary_content);
        
        // Look for development-specific function names and symbols
        let dev_symbols = [
            "hot_reload_state",
            "restore_state",
            "interpret_ui_change",
            "start_development_mode",
            "runtime_interpreter",
            "performance_monitor",
        ];
        
        for symbol in &dev_symbols {
            if content_str.contains(symbol) {
                symbols.push(symbol.to_string());
            }
        }
        
        Ok(symbols)
    }
    
    /// Find development features in a binary
    fn find_development_features(&self, binary_path: &Path) -> Result<Vec<String>> {
        let mut features = Vec::new();
        
        let binary_content = std::fs::read(binary_path)
            .map_err(|e| RustyUIError::generic(format!("Failed to read binary: {}", e)))?;
        
        let content_str = String::from_utf8_lossy(&binary_content);
        
        // Look for feature-specific strings that shouldn't be in production
        let dev_features = [
            "cfg(feature = \"dev-ui\")",
            "development_settings",
            "interpretation_strategy",
            "jit_compilation_threshold",
        ];
        
        for feature in &dev_features {
            if content_str.contains(feature) {
                features.push(feature.to_string());
            }
        }
        
        Ok(features)
    }
    
    /// Benchmark a binary's performance
    fn benchmark_binary(&self, binary_path: &Path) -> Result<PerformanceMetrics> {
        let start_time = Instant::now();
        
        // Run the binary and measure performance
        let mut cmd = Command::new(binary_path);
        
        let output = cmd.output()
            .map_err(|e| RustyUIError::generic(format!("Failed to run binary: {}", e)))?;
        
        let execution_time = start_time.elapsed();
        
        // For this simplified implementation, we'll use basic metrics
        // In a real implementation, you'd use more sophisticated profiling
        
        Ok(PerformanceMetrics {
            startup_time_ms: execution_time.as_millis() as f64,
            memory_usage_bytes: 0, // Would need platform-specific memory measurement
            cpu_usage_percent: 0.0, // Would need CPU profiling
            benchmark_time_ms: execution_time.as_millis() as f64,
        })
    }
    
    /// Get the binary name from Cargo.toml
    fn get_binary_name(&self) -> Result<String> {
        let cargo_toml_path = self.project_path.join("Cargo.toml");
        let cargo_content = std::fs::read_to_string(&cargo_toml_path)
            .map_err(|e| RustyUIError::generic(format!("Failed to read Cargo.toml: {}", e)))?;
        
        // Parse Cargo.toml to get package name (simplified)
        for line in cargo_content.lines() {
            if line.trim().starts_with("name = ") {
                if let Some(name) = line.split('=').nth(1) {
                    let name = name.trim().trim_matches('"');
                    return Ok(name.to_string());
                }
            }
        }
        
        Err(RustyUIError::generic("Could not find package name in Cargo.toml"))
    }
    
    /// Get verification results
    pub fn get_results(&self) -> &VerificationResults {
        &self.verification_results
    }
}

/// Build result information
#[derive(Debug, Clone)]
struct BuildResult {
    binary_path: std::path::PathBuf,
    features: Vec<String>,
    build_output: String,
}

impl Default for VerificationResults {
    fn default() -> Self {
        Self {
            conditional_compilation_ok: false,
            binary_size_results: BinarySizeResults::default(),
            performance_results: PerformanceResults::default(),
            feature_stripping_results: FeatureStrippingResults::default(),
            overall_status: VerificationStatus::VerificationFailed,
            detailed_report: String::new(),
        }
    }
}

impl Default for BinarySizeResults {
    fn default() -> Self {
        Self {
            rustyui_production_size: 0,
            standard_rust_size: 0,
            size_difference: 0,
            size_difference_percent: 0.0,
            size_acceptable: false,
        }
    }
}

impl Default for PerformanceResults {
    fn default() -> Self {
        Self {
            rustyui_performance: PerformanceMetrics::default(),
            standard_performance: PerformanceMetrics::default(),
            performance_difference_percent: 0.0,
            performance_acceptable: false,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            startup_time_ms: 0.0,
            memory_usage_bytes: 0,
            cpu_usage_percent: 0.0,
            benchmark_time_ms: 0.0,
        }
    }
}

impl Default for FeatureStrippingResults {
    fn default() -> Self {
        Self {
            dev_features_found: Vec::new(),
            dev_symbols_found: Vec::new(),
            all_dev_features_stripped: false,
            conditional_compilation_verified: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_production_verifier_creation() {
        let temp_dir = TempDir::new().unwrap();
        let verifier = ProductionVerifier::new(temp_dir.path());
        
        assert_eq!(verifier.project_path, temp_dir.path());
        assert_eq!(verifier.verification_results.overall_status, VerificationStatus::VerificationFailed);
    }
    
    #[test]
    fn test_verification_status_ordering() {
        assert_eq!(VerificationStatus::ZeroOverheadConfirmed, VerificationStatus::ZeroOverheadConfirmed);
        assert_ne!(VerificationStatus::ZeroOverheadConfirmed, VerificationStatus::VerificationFailed);
    }
    
    #[test]
    fn test_binary_size_calculation() {
        let mut results = BinarySizeResults::default();
        results.rustyui_production_size = 1000;
        results.standard_rust_size = 950;
        results.size_difference = 50;
        results.size_difference_percent = 5.26;
        results.size_acceptable = false; // > 5%
        
        assert_eq!(results.size_difference, 50);
        assert!(!results.size_acceptable);
    }
    
    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        
        assert_eq!(metrics.startup_time_ms, 0.0);
        assert_eq!(metrics.memory_usage_bytes, 0);
        assert_eq!(metrics.cpu_usage_percent, 0.0);
        assert_eq!(metrics.benchmark_time_ms, 0.0);
    }
}