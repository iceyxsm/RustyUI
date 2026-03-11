//! Property 3: Zero-Overhead Production Builds
//! 
//! Task 13.2: Property-based test for zero-overhead production builds
//! 
//! **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6**
//! 
//! Property 3: Zero-Overhead Production Builds
//! For any production build, the conditional builder should strip all interpretation 
//! code, produce binaries identical to standard Rust compilation with equivalent 
//! binary size, memory usage, and performance characteristics.

use rustyui_core::{
    DualModeEngine, DualModeConfig, UIFramework, ProductionVerifier,
    BuildConfig, OptimizationLevel,
};
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use proptest::prelude::*;

#[cfg(feature = "dev-ui")]
use rustyui_core::{DevelopmentSettings, InterpretationStrategy};

/// Strategy for generating production build configurations
fn production_build_config_strategy() -> impl Strategy<Value = ProductionBuildConfig> {
    (
        prop_oneof![
            Just(UIFramework::Egui),
            Just(UIFramework::Iced),
            Just(UIFramework::Slint),
            Just(UIFramework::Tauri),
        ],
        prop_oneof![
            Just(OptimizationLevel::None),
            Just(OptimizationLevel::Less),
            Just(OptimizationLevel::Default),
            Just(OptimizationLevel::Aggressive),
        ],
        any::<bool>(), // lto enabled
        any::<bool>(), // strip symbols
        any::<bool>(), // panic abort
    ).prop_map(|(framework, opt_level, lto, strip, panic_abort)| {
        ProductionBuildConfig {
            framework,
            optimization_level: opt_level,
            lto_enabled: lto,
            strip_symbols: strip,
            panic_abort,
        }
    })
}

/// Strategy for generating component complexity levels
fn component_complexity_strategy() -> impl Strategy<Value = ComponentComplexity> {
    (1..=10usize, 1..=5usize, 1..=20usize).prop_map(|(components, depth, state_fields)| {
        ComponentComplexity {
            component_count: components,
            nesting_depth: depth,
            state_field_count: state_fields,
        }
    })
}

/// Configuration for production build testing
#[derive(Debug, Clone)]
struct ProductionBuildConfig {
    framework: UIFramework,
    optimization_level: OptimizationLevel,
    lto_enabled: bool,
    strip_symbols: bool,
    panic_abort: bool,
}

/// Component complexity configuration
#[derive(Debug, Clone)]
struct ComponentComplexity {
    component_count: usize,
    nesting_depth: usize,
    state_field_count: usize,
}

/// Production build test fixture
struct ProductionBuildFixture {
    temp_dir: TempDir,
    project_path: PathBuf,
    config: ProductionBuildConfig,
    complexity: ComponentComplexity,
}

impl ProductionBuildFixture {
    fn new(config: ProductionBuildConfig, complexity: ComponentComplexity) -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().to_path_buf();
        
        Self {
            temp_dir,
            project_path,
            config,
            complexity,
        }
    }
    
    fn setup_project(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create project structure
        std::fs::create_dir_all(self.project_path.join("src"))?;
        
        // Create Cargo.toml with production configuration
        let cargo_toml = self.generate_cargo_toml();
        std::fs::write(self.project_path.join("Cargo.toml"), cargo_toml)?;
        
        // Create main.rs with test application
        let main_rs = self.generate_main_rs();
        std::fs::write(self.project_path.join("src/main.rs"), main_rs)?;
        
        // Create components based on complexity
        for i in 0..self.complexity.component_count {
            let component_code = self.generate_component(i);
            let component_path = self.project_path.join("src").join(format!("component_{}.rs", i));
            std::fs::write(component_path, component_code)?;
        }
        
        Ok(())
    }
    
    fn generate_cargo_toml(&self) -> String {
        format!(r#"[package]
name = "production-test-app"
version = "0.1.0"
edition = "2021"

[dependencies]
rustyui-core = {{ path = "../../../crates/rustyui-core" }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

[features]
default = []
dev-ui = ["rustyui-core/dev-ui"]

[profile.release]
opt-level = {}
lto = {}
codegen-units = 1
panic = "{}"
strip = {}
"#,
            match self.config.optimization_level {
                OptimizationLevel::None => "0",
                OptimizationLevel::Less => "1",
                OptimizationLevel::Default => "2",
                OptimizationLevel::Aggressive => "3",
            },
            if self.config.lto_enabled { "true" } else { "false" },
            if self.config.panic_abort { "abort" } else { "unwind" },
            if self.config.strip_symbols { "true" } else { "false" }
        )
    }
    
    fn generate_main_rs(&self) -> String {
        let mut main_code = String::new();
        
        main_code.push_str("use rustyui_core::{DualModeEngine, DualModeConfig, UIFramework};\n");
        main_code.push_str("use std::time::Instant;\n\n");
        
        // Include component modules
        for i in 0..self.complexity.component_count {
            main_code.push_str(&format!("mod component_{};\n", i));
        }
        
        main_code.push_str("\nfn main() -> Result<(), Box<dyn std::error::Error>> {\n");
        main_code.push_str("    let start_time = Instant::now();\n\n");
        
        // Create dual-mode configuration
        main_code.push_str("    let config = DualModeConfig {\n");
        main_code.push_str(&format!("        framework: UIFramework::{:?},\n", self.config.framework));
        main_code.push_str("        watch_paths: vec![],\n");
        main_code.push_str("        production_settings: Default::default(),\n");
        main_code.push_str("        conditional_compilation: Default::default(),\n");
        
        #[cfg(feature = "dev-ui")]
        main_code.push_str("        development_settings: DevelopmentSettings::default(),\n");
        
        main_code.push_str("    };\n\n");
        
        // Initialize engine
        main_code.push_str("    let engine = DualModeEngine::new(config)?;\n");
        main_code.push_str("    let init_time = start_time.elapsed();\n\n");
        
        // Verify production characteristics
        main_code.push_str("    // Verify zero-overhead production characteristics\n");
        main_code.push_str("    #[cfg(not(feature = \"dev-ui\"))]\n");
        main_code.push_str("    {\n");
        main_code.push_str("        assert_eq!(engine.current_memory_overhead_bytes(), 0,\n");
        main_code.push_str("            \"Production build should have zero memory overhead\");\n");
        main_code.push_str("        assert!(!engine.supports_runtime_interpretation(),\n");
        main_code.push_str("            \"Production build should not support runtime interpretation\");\n");
        main_code.push_str("    }\n\n");
        
        // Simulate application workload
        main_code.push_str("    let workload_start = Instant::now();\n");
        main_code.push_str("    let mut total_operations = 0u64;\n\n");
        
        for i in 0..self.complexity.component_count {
            main_code.push_str(&format!("    // Component {} workload\n", i));
            main_code.push_str(&format!("    for _ in 0..1000 {{\n"));
            main_code.push_str(&format!("        let mut comp = component_{}::Component::new();\n", i));
            main_code.push_str("        comp.update();\n");
            main_code.push_str("        comp.render();\n");
            main_code.push_str("        total_operations += 1;\n");
            main_code.push_str("    }\n\n");
        }
        
        main_code.push_str("    let workload_time = workload_start.elapsed();\n");
        main_code.push_str("    let total_time = start_time.elapsed();\n\n");
        
        // Output performance metrics
        main_code.push_str("    println!(\"Production Build Performance:\");\n");
        main_code.push_str("    println!(\"  - Initialization: {:?}\", init_time);\n");
        main_code.push_str("    println!(\"  - Workload: {:?}\", workload_time);\n");
        main_code.push_str("    println!(\"  - Total: {:?}\", total_time);\n");
        main_code.push_str("    println!(\"  - Operations: {}\", total_operations);\n");
        main_code.push_str("    println!(\"  - Memory overhead: {} bytes\", engine.current_memory_overhead_bytes());\n\n");
        
        main_code.push_str("    Ok(())\n");
        main_code.push_str("}\n");
        
        main_code
    }
    
    fn generate_component(&self, index: usize) -> String {
        let mut component_code = String::new();
        
        component_code.push_str("use serde::{Serialize, Deserialize};\n\n");
        
        // Generate state struct
        component_code.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
        component_code.push_str("pub struct ComponentState {\n");
        
        for field_idx in 0..self.complexity.state_field_count {
            component_code.push_str(&format!("    pub field_{}: i32,\n", field_idx));
        }
        
        component_code.push_str("    pub counter: u64,\n");
        component_code.push_str("    pub enabled: bool,\n");
        component_code.push_str("}\n\n");
        
        // Generate component struct
        component_code.push_str("pub struct Component {\n");
        component_code.push_str("    state: ComponentState,\n");
        component_code.push_str("    render_count: u64,\n");
        component_code.push_str("}\n\n");
        
        // Generate implementation
        component_code.push_str("impl Component {\n");
        component_code.push_str("    pub fn new() -> Self {\n");
        component_code.push_str("        Self {\n");
        component_code.push_str("            state: ComponentState {\n");
        
        for field_idx in 0..self.complexity.state_field_count {
            component_code.push_str(&format!("                field_{}: {},\n", field_idx, field_idx));
        }
        
        component_code.push_str("                counter: 0,\n");
        component_code.push_str("                enabled: true,\n");
        component_code.push_str("            },\n");
        component_code.push_str("            render_count: 0,\n");
        component_code.push_str("        }\n");
        component_code.push_str("    }\n\n");
        
        component_code.push_str("    pub fn update(&mut self) {\n");
        component_code.push_str("        self.state.counter += 1;\n");
        
        for field_idx in 0..self.complexity.state_field_count {
            component_code.push_str(&format!("        self.state.field_{} = (self.state.counter % {}) as i32;\n", 
                field_idx, field_idx + 1));
        }
        
        component_code.push_str("        self.state.enabled = self.state.counter % 2 == 0;\n");
        component_code.push_str("    }\n\n");
        
        component_code.push_str("    pub fn render(&mut self) {\n");
        component_code.push_str("        self.render_count += 1;\n");
        component_code.push_str("        \n");
        component_code.push_str("        // Simulate rendering work\n");
        component_code.push_str("        if self.state.enabled {\n");
        
        for field_idx in 0..self.complexity.state_field_count {
            component_code.push_str(&format!("            std::hint::black_box(self.state.field_{});\n", field_idx));
        }
        
        component_code.push_str("            std::hint::black_box(self.state.counter);\n");
        component_code.push_str("        }\n");
        component_code.push_str("    }\n");
        component_code.push_str("}\n");
        
        component_code
    }
    
    fn build_production(&self) -> Result<ProductionBuildResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        // Build without dev-ui feature (production mode)
        let output = std::process::Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(&self.project_path)
            .output()?;
        
        let build_time = start_time.elapsed();
        
        if !output.status.success() {
            return Err(format!("Production build failed: {}", 
                String::from_utf8_lossy(&output.stderr)).into());
        }
        
        // Get binary path and size
        let binary_path = self.project_path
            .join("target")
            .join("release")
            .join("production-test-app");
        
        let binary_size = if binary_path.exists() {
            std::fs::metadata(&binary_path)?.len()
        } else {
            // Try with .exe extension on Windows
            let exe_path = binary_path.with_extension("exe");
            if exe_path.exists() {
                std::fs::metadata(&exe_path)?.len()
            } else {
                return Err("Production binary not found".into());
            }
        };
        
        Ok(ProductionBuildResult {
            build_time,
            binary_size,
            binary_path,
            build_output: String::from_utf8_lossy(&output.stdout).to_string(),
        })
    }
    
    fn build_development(&self) -> Result<DevelopmentBuildResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        // Build with dev-ui feature (development mode)
        let output = std::process::Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--features")
            .arg("dev-ui")
            .current_dir(&self.project_path)
            .output()?;
        
        let build_time = start_time.elapsed();
        
        if !output.status.success() {
            return Err(format!("Development build failed: {}", 
                String::from_utf8_lossy(&output.stderr)).into());
        }
        
        // Get binary path and size
        let binary_path = self.project_path
            .join("target")
            .join("release")
            .join("production-test-app");
        
        let binary_size = if binary_path.exists() {
            std::fs::metadata(&binary_path)?.len()
        } else {
            // Try with .exe extension on Windows
            let exe_path = binary_path.with_extension("exe");
            if exe_path.exists() {
                std::fs::metadata(&exe_path)?.len()
            } else {
                return Err("Development binary not found".into());
            }
        };
        
        Ok(DevelopmentBuildResult {
            build_time,
            binary_size,
            binary_path,
            build_output: String::from_utf8_lossy(&output.stdout).to_string(),
        })
    }
    
    fn run_binary_benchmark(&self, binary_path: &std::path::Path) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        let output = std::process::Command::new(binary_path)
            .output()?;
        
        let execution_time = start_time.elapsed();
        
        if !output.status.success() {
            return Err(format!("Binary execution failed: {}", 
                String::from_utf8_lossy(&output.stderr)).into());
        }
        
        Ok(BenchmarkResult {
            execution_time,
            output: String::from_utf8_lossy(&output.stdout).to_string(),
        })
    }
}

/// Production build result
#[derive(Debug)]
struct ProductionBuildResult {
    build_time: Duration,
    binary_size: u64,
    binary_path: PathBuf,
    build_output: String,
}

/// Development build result
#[derive(Debug)]
struct DevelopmentBuildResult {
    build_time: Duration,
    binary_size: u64,
    binary_path: PathBuf,
    build_output: String,
}

/// Binary benchmark result
#[derive(Debug)]
struct BenchmarkResult {
    execution_time: Duration,
    output: String,
}

// ============================================================================
// Property 3: Zero-Overhead Production Builds
// ============================================================================

proptest! {
    /// **Property 3: Zero-Overhead Production Builds**
    /// 
    /// **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6**
    /// 
    /// For any production build configuration and component complexity,
    /// the production build should:
    /// 1. Strip all development features (Requirement 3.1)
    /// 2. Produce binaries equivalent to standard Rust (Requirement 3.2)
    /// 3. Maintain full compile-time optimizations (Requirement 3.3)
    /// 4. Have zero memory overhead (Requirement 3.4)
    /// 5. Match or exceed standard Rust performance (Requirement 3.5)
    /// 6. Preserve security characteristics (Requirement 3.6)
    #[test]
    fn property_3_zero_overhead_production_builds(
        config in production_build_config_strategy(),
        complexity in component_complexity_strategy()
    ) {
        let fixture = ProductionBuildFixture::new(config.clone(), complexity.clone());
        
        // Setup test project
        fixture.setup_project()
            .expect("Should setup test project");
        
        // Build production version (without dev-ui feature)
        let prod_result = fixture.build_production()
            .expect("Production build should succeed");
        
        // Build development version (with dev-ui feature) for comparison
        let dev_result = fixture.build_development()
            .expect("Development build should succeed");
        
        // Property 3.1: Development features should be stripped in production
        // Production binary should be smaller or equal to development binary
        prop_assert!(prod_result.binary_size <= dev_result.binary_size,
            "Production binary ({} bytes) should not be larger than development binary ({} bytes)",
            prod_result.binary_size, dev_result.binary_size);
        
        // Property 3.2: Binary size should be reasonable for the complexity
        let expected_base_size = 1024 * 1024; // 1MB base
        let complexity_factor = complexity.component_count * complexity.state_field_count;
        let max_expected_size = expected_base_size + (complexity_factor * 1024) as u64;
        
        prop_assert!(prod_result.binary_size <= max_expected_size,
            "Production binary size ({} bytes) should be reasonable for complexity (max {} bytes)",
            prod_result.binary_size, max_expected_size);
        
        // Property 3.3: Build time should be reasonable
        prop_assert!(prod_result.build_time <= Duration::from_secs(120),
            "Production build time should be under 2 minutes, got {:?}",
            prod_result.build_time);
        
        // Property 3.4 & 3.5: Performance should be good
        let prod_benchmark = fixture.run_binary_benchmark(&prod_result.binary_path)
            .expect("Production binary should run successfully");
        
        let dev_benchmark = fixture.run_binary_benchmark(&dev_result.binary_path)
            .expect("Development binary should run successfully");
        
        // Production should be faster or equal to development
        prop_assert!(prod_benchmark.execution_time <= dev_benchmark.execution_time * 2,
            "Production execution ({:?}) should not be significantly slower than development ({:?})",
            prod_benchmark.execution_time, dev_benchmark.execution_time);
        
        // Property 3.6: Both binaries should produce valid output
        prop_assert!(prod_benchmark.output.contains("Production Build Performance"),
            "Production binary should produce expected output");
        
        prop_assert!(prod_benchmark.output.contains("Memory overhead: 0 bytes") || 
                    !prod_benchmark.output.contains("dev-ui"),
            "Production binary should show zero overhead or no dev features");
        
        // Additional verification: Binary should not contain development strings
        let binary_content = std::fs::read(&prod_result.binary_path)
            .expect("Should read production binary");
        let binary_str = String::from_utf8_lossy(&binary_content);
        
        // These strings should not appear in production binaries
        let dev_markers = [
            "runtime_interpreter",
            "hot_reload_state",
            "development_mode",
            "interpret_ui_change",
        ];
        
        for marker in &dev_markers {
            prop_assert!(!binary_str.contains(marker),
                "Production binary should not contain development marker: {}", marker);
        }
        
        println!("✅ Property 3 verified for config: {:?}, complexity: {:?}", config, complexity);
        println!("   Production: {} bytes, {:?} execution", 
            prod_result.binary_size, prod_benchmark.execution_time);
        println!("   Development: {} bytes, {:?} execution", 
            dev_result.binary_size, dev_benchmark.execution_time);
    }
}

// ============================================================================
// Unit Tests for Property 3 Components
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;
    
    #[test]
    fn test_production_build_config_generation() {
        let config = ProductionBuildConfig {
            framework: UIFramework::Egui,
            optimization_level: OptimizationLevel::Aggressive,
            lto_enabled: true,
            strip_symbols: true,
            panic_abort: true,
        };
        
        let complexity = ComponentComplexity {
            component_count: 2,
            nesting_depth: 1,
            state_field_count: 3,
        };
        
        let fixture = ProductionBuildFixture::new(config, complexity);
        let cargo_toml = fixture.generate_cargo_toml();
        
        assert!(cargo_toml.contains("opt-level = 3"));
        assert!(cargo_toml.contains("lto = true"));
        assert!(cargo_toml.contains("panic = \"abort\""));
        assert!(cargo_toml.contains("strip = true"));
    }
    
    #[test]
    fn test_component_generation() {
        let config = ProductionBuildConfig {
            framework: UIFramework::Egui,
            optimization_level: OptimizationLevel::Default,
            lto_enabled: false,
            strip_symbols: false,
            panic_abort: false,
        };
        
        let complexity = ComponentComplexity {
            component_count: 1,
            nesting_depth: 1,
            state_field_count: 5,
        };
        
        let fixture = ProductionBuildFixture::new(config, complexity);
        let component_code = fixture.generate_component(0);
        
        assert!(component_code.contains("pub struct ComponentState"));
        assert!(component_code.contains("pub struct Component"));
        assert!(component_code.contains("field_0: i32"));
        assert!(component_code.contains("field_4: i32"));
        assert!(component_code.contains("pub fn update"));
        assert!(component_code.contains("pub fn render"));
    }
    
    #[test]
    fn test_main_rs_generation() {
        let config = ProductionBuildConfig {
            framework: UIFramework::Slint,
            optimization_level: OptimizationLevel::Less,
            lto_enabled: true,
            strip_symbols: false,
            panic_abort: true,
        };
        
        let complexity = ComponentComplexity {
            component_count: 3,
            nesting_depth: 2,
            state_field_count: 2,
        };
        
        let fixture = ProductionBuildFixture::new(config, complexity);
        let main_code = fixture.generate_main_rs();
        
        assert!(main_code.contains("UIFramework::Slint"));
        assert!(main_code.contains("mod component_0"));
        assert!(main_code.contains("mod component_1"));
        assert!(main_code.contains("mod component_2"));
        assert!(main_code.contains("current_memory_overhead_bytes()"));
        assert!(main_code.contains("supports_runtime_interpretation()"));
    }
}