//! Project management utilities for RustyUI CLI

use crate::error::{CliError, CliResult};
use console::style;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Project manager for RustyUI operations
pub struct ProjectManager {
    project_path: PathBuf,
}

impl ProjectManager {
    /// Create a new project manager
    pub fn new(project_path: PathBuf) -> CliResult<Self> {
        Ok(Self { project_path })
    }
    
    /// Check if the directory is a valid Rust project
    pub fn is_rust_project(&self) -> bool {
        self.project_path.join("Cargo.toml").exists()
    }
    
    /// Check if the directory is a RustyUI project
    pub fn is_rustyui_project(&self) -> bool {
        self.project_path.join("rustyui.toml").exists()
    }
    
    /// Check if this project is a workspace member
    fn is_workspace_member(&self) -> CliResult<bool> {
        // Check if there's a parent Cargo.toml with workspace configuration
        let mut current_path = self.project_path.clone();
        
        // Go up directories looking for a workspace root
        while let Some(parent) = current_path.parent() {
            let parent_cargo = parent.join("Cargo.toml");
            if parent_cargo.exists() {
                let cargo_content = std::fs::read_to_string(&parent_cargo)?;
                if cargo_content.contains("[workspace]") {
                    // Check if our project is listed as a member
                    let relative_path = self.project_path.strip_prefix(parent)
                        .map_err(|_| CliError::project("Failed to determine workspace membership"))?;
                    
                    if cargo_content.contains(&format!("\"{}\"", relative_path.display())) {
                        return Ok(true);
                    }
                }
            }
            current_path = parent.to_path_buf();
        }
        
        Ok(false)
    }
    
    /// Create a new Rust project using cargo
    pub fn create_rust_project(&self, name: &str) -> CliResult<()> {
        if self.project_path.exists() {
            return Err(CliError::directory_exists(
                format!("Directory '{}' already exists", self.project_path.display())
            ));
        }
        
        println!("{} Creating new Rust project '{}'...", style("").blue(), name);
        
        let status = Command::new("cargo")
            .args(&["init", "--name", name])
            .current_dir(&self.project_path.parent().unwrap_or(Path::new(".")))
            .status()?;
        
        if !status.success() {
            return Err(CliError::command("Failed to create Rust project with cargo init"));
        }
        
        println!("{} Created Rust project structure", style("✓").green());
        
        Ok(())
    }
    
    /// Modify Cargo.toml to add RustyUI dependencies with dual-mode support
    pub fn add_rustyui_dependencies(&self, framework: &str) -> CliResult<()> {
        let cargo_toml_path = self.project_path.join("Cargo.toml");
        
        if !cargo_toml_path.exists() {
            return Err(CliError::file_not_found("Cargo.toml not found"));
        }
        
        let mut cargo_content = std::fs::read_to_string(&cargo_toml_path)?;
        
        // Parse existing Cargo.toml to understand structure
        let analysis = self.analyze_cargo_toml(&cargo_content)?;
        
        // Add RustyUI dependencies with dual-mode support
        let rustyui_deps = self.generate_dual_mode_dependencies(framework, &analysis)?;
        
        // Insert dependencies in the appropriate section
        cargo_content = self.insert_dependencies(&cargo_content, &rustyui_deps)?;
        
        // Add or update features section for dual-mode support
        cargo_content = self.add_dual_mode_features(&cargo_content)?;
        
        // Add conditional compilation configuration
        cargo_content = self.add_conditional_compilation_config(&cargo_content)?;
        
        std::fs::write(&cargo_toml_path, cargo_content)?;
        
        println!("{} Updated Cargo.toml with dual-mode RustyUI dependencies", style("✓").green());
        
        Ok(())
    }
    
    /// Analyze existing Cargo.toml structure
    fn analyze_cargo_toml(&self, content: &str) -> CliResult<CargoTomlAnalysis> {
        let mut analysis = CargoTomlAnalysis::default();
        
        analysis.has_dependencies = content.contains("[dependencies]");
        analysis.has_dev_dependencies = content.contains("[dev-dependencies]");
        analysis.has_build_dependencies = content.contains("[build-dependencies]");
        analysis.has_features = content.contains("[features]");
        analysis.has_profiles = content.contains("[profile.");
        analysis.has_workspace = content.contains("[workspace]");
        
        // Extract existing features
        if let Some(features_start) = content.find("[features]") {
            let features_section = &content[features_start..];
            if let Some(features_end) = features_section.find("\n[") {
                let features_content = &features_section[..features_end];
                for line in features_content.lines() {
                    if let Some(feature_name) = line.split('=').next() {
                        let feature_name = feature_name.trim();
                        if !feature_name.is_empty() && feature_name != "[features]" {
                            analysis.existing_features.push(feature_name.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(analysis)
    }
    
    /// Generate dual-mode dependencies for the specified framework
    fn generate_dual_mode_dependencies(&self, framework: &str, analysis: &CargoTomlAnalysis) -> CliResult<String> {
        let mut deps = String::new();
        
        // Check if this is a workspace member
        let is_workspace_member = self.is_workspace_member()?;
        
        // Core RustyUI dependencies with dual-mode support
        deps.push_str("\n# RustyUI Core Dependencies (Dual-Mode Support)\n");
        if is_workspace_member {
            deps.push_str("rustyui-core = { path = \"../crates/rustyui-core\", features = [] }\n");
            deps.push_str("rustyui-adapters = { path = \"../crates/rustyui-adapters\", optional = true }\n");
            deps.push_str("rustyui-macros = { path = \"../crates/rustyui-macros\", optional = true }\n");
        } else {
            deps.push_str("rustyui-core = { version = \"0.1.0\", features = [] }\n");
            deps.push_str("rustyui-adapters = { version = \"0.1.0\", optional = true }\n");
            deps.push_str("rustyui-macros = { version = \"0.1.0\", optional = true }\n");
        }
        deps.push_str("\n");
        
        // Development-only dependencies (conditionally compiled)
        deps.push_str("# Development-Only Dependencies (Stripped in Production)\n");
        deps.push_str("rhai = { version = \"1.16\", optional = true }\n");
        deps.push_str("syn = { version = \"2.0\", features = [\"full\"], optional = true }\n");
        deps.push_str("cranelift-jit = { version = \"0.101\", optional = true }\n");
        deps.push_str("wasmtime = { version = \"13.0\", optional = true }\n");
        deps.push_str("notify = { version = \"6.0\", optional = true }\n");
        deps.push_str("\n");
        
        // Framework-specific dependencies
        match framework {
            "egui" => {
                deps.push_str("# egui Framework Dependencies\n");
                deps.push_str("egui = \"0.24\"\n");
                deps.push_str("eframe = { version = \"0.24\", features = [\"default_fonts\"] }\n");
                deps.push_str("winapi = { version = \"0.3\", features = [\"winuser\", \"windef\"] }\n");
            }
            "iced" => {
                deps.push_str("# iced Framework Dependencies\n");
                deps.push_str("iced = { version = \"0.10\", features = [\"debug\"] }\n");
            }
            "slint" => {
                deps.push_str("# slint Framework Dependencies\n");
                deps.push_str("slint = \"1.3\"\n");
            }
            "tauri" => {
                deps.push_str("# tauri Framework Dependencies\n");
                deps.push_str("tauri = { version = \"1.5\", features = [\"api-all\"] }\n");
                deps.push_str("serde = { version = \"1.0\", features = [\"derive\"] }\n");
                deps.push_str("serde_json = \"1.0\"\n");
                deps.push_str("tokio = { version = \"1.0\", features = [\"full\"] }\n");
            }
            _ => {
                return Err(CliError::unsupported_framework(framework));
            }
        }
        
        deps.push_str("\n");
        
        Ok(deps)
    }
    
    /// Insert dependencies into Cargo.toml content
    fn insert_dependencies(&self, content: &str, dependencies: &str) -> CliResult<String> {
        let mut result = content.to_string();
        
        if result.contains("[dependencies]") {
            // Find the [dependencies] section and insert after it
            if let Some(deps_start) = result.find("[dependencies]") {
                let deps_line_end = result[deps_start..].find('\n').unwrap_or(0) + deps_start + 1;
                result.insert_str(deps_line_end, dependencies);
            }
        } else {
            // Add [dependencies] section at the end
            result.push_str("\n[dependencies]\n");
            result.push_str(dependencies);
        }
        
        Ok(result)
    }
    
    /// Add dual-mode features configuration
    fn add_dual_mode_features(&self, content: &str) -> CliResult<String> {
        let mut result = content.to_string();
        
        let features_section = r#"
[features]
default = []

# Development mode with runtime interpretation
dev-ui = [
    "rustyui-core/dev-ui",
    "rustyui-adapters",
    "rustyui-macros",
    "rhai",
    "syn",
    "cranelift-jit",
    "wasmtime",
    "notify",
]

# JIT compilation support (development only)
jit-support = ["dev-ui", "cranelift-jit"]

# WASM runtime support (development only)
wasm-runtime = ["dev-ui", "wasmtime"]
"#;
        
        if result.contains("[features]") {
            // Replace existing features section
            if let Some(features_start) = result.find("[features]") {
                if let Some(features_end) = result[features_start..].find("\n[") {
                    let end_pos = features_start + features_end;
                    result.replace_range(features_start..end_pos, features_section.trim());
                } else {
                    // Features section is at the end
                    result.replace_range(features_start.., features_section.trim());
                }
            }
        } else {
            // Add features section
            result.push_str(features_section);
        }
        
        Ok(result)
    }
    
    /// Add dual-mode profile configurations
    fn add_dual_mode_profiles(&self, content: &str) -> CliResult<String> {
        let mut result = content.to_string();
        
        let _profiles_section = r#"
# Development profile optimized for fast compilation and hot reload
[profile.dev]
opt-level = 0
debug = true
incremental = true
codegen-units = 256

# Release profile with maximum optimization and zero development overhead
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true

# Development release profile for testing production performance
[profile.dev-release]
inherits = "release"
debug = true
strip = false

# Profile for benchmarking dual-mode performance
[profile.bench]
opt-level = 3
debug = false
lto = true
codegen-units = 1
"#;
        
        // Check if profiles already exist and add only missing ones
        if !result.contains("[profile.dev]") {
            result.push_str("\n[profile.dev]\nopt-level = 0\ndebug = true\nincremental = true\ncodegen-units = 256\n");
        }
        
        if !result.contains("[profile.release]") {
            result.push_str("\n[profile.release]\nopt-level = 3\nlto = true\ncodegen-units = 1\npanic = \"abort\"\nstrip = true\n");
        }
        
        if !result.contains("[profile.dev-release]") {
            result.push_str("\n[profile.dev-release]\ninherits = \"release\"\ndebug = true\nstrip = false\n");
        }
        
        if !result.contains("[profile.bench]") {
            result.push_str("\n[profile.bench]\nopt-level = 3\ndebug = false\nlto = true\ncodegen-units = 1\n");
        }
        
        Ok(result)
    }
    
    /// Add conditional compilation configuration
    fn add_conditional_compilation_config(&self, content: &str) -> CliResult<String> {
        let mut result = content.to_string();
        
        // Add metadata section for dual-mode configuration
        let metadata_section = r#"
[package.metadata.rustyui]
# Dual-mode configuration metadata
dual-mode = true
development-features = ["dev-ui"]
conditional-compilation = true

# Hot reload configuration
hot-reload = { enabled = true, watch-paths = ["src/"] }
state-preservation = { enabled = true, strategy = "hybrid" }
interpretation-strategy = "hybrid"
"#;
        
        if !result.contains("[package.metadata.rustyui]") {
            result.push_str(metadata_section);
        }
        
        Ok(result)
    }
    
    /// Generate RustyUI dependencies for Cargo.toml (legacy method for compatibility)
    fn generate_rustyui_dependencies(&self, framework: &str) -> CliResult<String> {
        let analysis = CargoTomlAnalysis::default();
        self.generate_dual_mode_dependencies(framework, &analysis)
    }
    
    /// Generate features section for Cargo.toml (legacy method for compatibility)
    fn generate_features_section(&self) -> CliResult<String> {
        Ok(r#"
[features]
default = []
dev-ui = [
    "rustyui-core/dev-ui",
    "rustyui-adapters/dev-ui",
]
"#.to_string())
    }
    
    /// Build project for production
    pub fn build_production(&self, release: bool) -> CliResult<()> {
        if !self.is_rust_project() {
            return Err(CliError::project("Not a Rust project"));
        }
        
        println!("{} Building for production...", style("").blue());
        
        let mut args = vec!["build"];
        if release {
            args.push("--release");
        }
        
        let status = Command::new("cargo")
            .args(&args)
            .current_dir(&self.project_path)
            .status()?;
        
        if !status.success() {
            return Err(CliError::build("Production build failed"));
        }
        
        Ok(())
    }
    
    /// Run project in development mode
    pub fn run_development(&self, _watch: bool) -> CliResult<()> {
        if !self.is_rust_project() {
            return Err(CliError::project("Not a Rust project"));
        }
        
        if !self.is_rustyui_project() {
            return Err(CliError::project("Not a RustyUI project. Run 'rustyui init' first."));
        }
        
        println!("{} Starting development mode...", style("").blue());
        
        let args = vec!["run", "--features", "dev-ui"];
        
        let status = Command::new("cargo")
            .args(&args)
            .current_dir(&self.project_path)
            .status()?;
        
        if !status.success() {
            return Err(CliError::dev_mode("Failed to start development mode"));
        }
        
        Ok(())
    }
    
    /// Analyze project structure
    pub fn analyze_project(&self) -> CliResult<ProjectAnalysis> {
        let mut analysis = ProjectAnalysis::default();
        
        // Check if it's a Rust project
        analysis.is_rust_project = self.is_rust_project();
        
        // Check if it's a RustyUI project
        analysis.is_rustyui_project = self.is_rustyui_project();
        
        // Analyze build system
        analysis.build_system = self.analyze_build_system()?;
        
        // Analyze project type and workspace info
        analysis.project_type = self.detect_project_type()?;
        analysis.workspace_info = self.analyze_workspace()?;
        
        // Detect potential UI framework and existing dependencies
        if let Ok(cargo_content) = std::fs::read_to_string(self.project_path.join("Cargo.toml")) {
            analysis.existing_dependencies = self.extract_dependencies(&cargo_content)?;
            
            if cargo_content.contains("egui") {
                analysis.detected_framework = Some("egui".to_string());
            } else if cargo_content.contains("iced") {
                analysis.detected_framework = Some("iced".to_string());
            } else if cargo_content.contains("slint") {
                analysis.detected_framework = Some("slint".to_string());
            } else if cargo_content.contains("tauri") {
                analysis.detected_framework = Some("tauri".to_string());
            }
        }
        
        // Check for common UI directories
        let ui_dirs = ["src", "ui", "components", "widgets", "views"];
        for dir in &ui_dirs {
            let dir_path = self.project_path.join(dir);
            if dir_path.exists() {
                analysis.ui_directories.push(dir.to_string());
            }
        }
        
        // Analyze source files
        analysis.source_files = self.analyze_source_files()?;
        
        // Check for existing configuration files
        analysis.configuration_files = self.find_configuration_files()?;
        if self.project_path.join("rustyui.toml").exists() {
            analysis.has_rustyui_config = true;
        }
        
        Ok(analysis)
    }
    
    /// Analyze build system configuration
    fn analyze_build_system(&self) -> CliResult<BuildSystem> {
        let mut build_system = BuildSystem::default();
        
        // Check for Cargo.toml
        let cargo_toml_path = self.project_path.join("Cargo.toml");
        build_system.has_cargo_toml = cargo_toml_path.exists();
        
        if build_system.has_cargo_toml {
            let cargo_content = std::fs::read_to_string(&cargo_toml_path)?;
            
            // Extract features
            if let Some(features_start) = cargo_content.find("[features]") {
                let features_section = &cargo_content[features_start..];
                if let Some(features_end) = features_section.find("\n[") {
                    let features_content = &features_section[..features_end];
                    for line in features_content.lines() {
                        if let Some(feature_name) = line.split('=').next() {
                            let feature_name = feature_name.trim();
                            if !feature_name.is_empty() && feature_name != "[features]" {
                                build_system.cargo_features.push(feature_name.to_string());
                            }
                        }
                    }
                }
            }
            
            // Extract profile configurations
            for profile in &["dev", "release", "test", "bench"] {
                if cargo_content.contains(&format!("[profile.{}]", profile)) {
                    build_system.profile_configurations.push(profile.to_string());
                }
            }
        }
        
        // Check for build.rs
        build_system.has_build_rs = self.project_path.join("build.rs").exists();
        
        // Check for Makefile
        build_system.has_makefile = self.project_path.join("Makefile").exists() 
            || self.project_path.join("makefile").exists();
        
        Ok(build_system)
    }
    
    /// Detect project type (binary, library, workspace, etc.)
    fn detect_project_type(&self) -> CliResult<ProjectType> {
        let cargo_toml_path = self.project_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(ProjectType::Binary); // Default fallback
        }
        
        let cargo_content = std::fs::read_to_string(&cargo_toml_path)?;
        
        // Check for workspace
        if cargo_content.contains("[workspace]") {
            return Ok(ProjectType::Workspace);
        }
        
        // Check for library vs binary
        let has_lib_rs = self.project_path.join("src").join("lib.rs").exists();
        let has_main_rs = self.project_path.join("src").join("main.rs").exists();
        
        match (has_lib_rs, has_main_rs) {
            (true, true) => Ok(ProjectType::Mixed),
            (true, false) => Ok(ProjectType::Library),
            (false, true) => Ok(ProjectType::Binary),
            (false, false) => Ok(ProjectType::Binary), // Default
        }
    }
    
    /// Analyze workspace configuration
    fn analyze_workspace(&self) -> CliResult<Option<WorkspaceInfo>> {
        let cargo_toml_path = self.project_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(None);
        }
        
        let cargo_content = std::fs::read_to_string(&cargo_toml_path)?;
        
        if !cargo_content.contains("[workspace]") {
            return Ok(None);
        }
        
        let mut workspace_info = WorkspaceInfo {
            is_workspace_root: true,
            workspace_members: Vec::new(),
            workspace_dependencies: Vec::new(),
        };
        
        // Extract workspace members
        if let Some(members_start) = cargo_content.find("members = [") {
            let members_section = &cargo_content[members_start..];
            if let Some(members_end) = members_section.find(']') {
                let members_content = &members_section[11..members_end]; // Skip "members = ["
                for member in members_content.split(',') {
                    let member = member.trim().trim_matches('"');
                    if !member.is_empty() {
                        workspace_info.workspace_members.push(member.to_string());
                    }
                }
            }
        }
        
        // Extract workspace dependencies
        if let Some(deps_start) = cargo_content.find("[workspace.dependencies]") {
            let deps_section = &cargo_content[deps_start..];
            if let Some(deps_end) = deps_section.find("\n[") {
                let deps_content = &deps_section[..deps_end];
                for line in deps_content.lines() {
                    if let Some(dep_name) = line.split('=').next() {
                        let dep_name = dep_name.trim();
                        if !dep_name.is_empty() && dep_name != "[workspace.dependencies]" {
                            workspace_info.workspace_dependencies.push(dep_name.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(Some(workspace_info))
    }
    
    /// Extract dependencies from Cargo.toml content
    fn extract_dependencies(&self, cargo_content: &str) -> CliResult<Vec<String>> {
        let mut dependencies = Vec::new();
        
        // Find [dependencies] section
        if let Some(deps_start) = cargo_content.find("[dependencies]") {
            let deps_section = &cargo_content[deps_start..];
            if let Some(deps_end) = deps_section.find("\n[") {
                let deps_content = &deps_section[..deps_end];
                for line in deps_content.lines() {
                    if let Some(dep_name) = line.split('=').next() {
                        let dep_name = dep_name.trim();
                        if !dep_name.is_empty() && dep_name != "[dependencies]" && !dep_name.starts_with('#') {
                            dependencies.push(dep_name.to_string());
                        }
                    }
                }
            }
        }
        
        // Also check [dev-dependencies] and [build-dependencies]
        for section in &["[dev-dependencies]", "[build-dependencies]"] {
            if let Some(deps_start) = cargo_content.find(section) {
                let deps_section = &cargo_content[deps_start..];
                if let Some(deps_end) = deps_section.find("\n[") {
                    let deps_content = &deps_section[..deps_end];
                    for line in deps_content.lines() {
                        if let Some(dep_name) = line.split('=').next() {
                            let dep_name = dep_name.trim();
                            if !dep_name.is_empty() && dep_name != *section && !dep_name.starts_with('#') {
                                dependencies.push(format!("{} ({})", dep_name, section));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(dependencies)
    }
    
    /// Analyze source files in the project
    fn analyze_source_files(&self) -> CliResult<Vec<SourceFileInfo>> {
        let mut source_files = Vec::new();
        
        // Analyze src directory
        if let Ok(entries) = std::fs::read_dir(self.project_path.join("src")) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let path = entry.path();
                        if let Some(_extension) = path.extension() {
                            let file_info = self.analyze_source_file(&path)?;
                            source_files.push(file_info);
                        }
                    }
                }
            }
        }
        
        // Analyze other common directories
        for dir in &["ui", "components", "widgets", "views"] {
            let dir_path = self.project_path.join(dir);
            if dir_path.exists() {
                if let Ok(entries) = std::fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        if let Ok(file_type) = entry.file_type() {
                            if file_type.is_file() {
                                let path = entry.path();
                                let file_info = self.analyze_source_file(&path)?;
                                source_files.push(file_info);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(source_files)
    }
    
    /// Analyze a single source file
    fn analyze_source_file(&self, path: &Path) -> CliResult<SourceFileInfo> {
        let file_type = match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => SourceFileType::Rust,
            Some("toml") => SourceFileType::Toml,
            Some("json") => SourceFileType::Json,
            Some("yaml") | Some("yml") => SourceFileType::Yaml,
            Some(ext) => SourceFileType::Other(ext.to_string()),
            None => SourceFileType::Other("unknown".to_string()),
        };
        
        let mut has_ui_code = false;
        let mut framework_usage = None;
        
        // Analyze Rust files for UI code
        if matches!(file_type, SourceFileType::Rust) {
            if let Ok(content) = std::fs::read_to_string(path) {
                // Check for UI-related imports and code
                let ui_indicators = [
                    "use egui", "egui::", "eframe::",
                    "use iced", "iced::",
                    "use slint", "slint::",
                    "use tauri", "tauri::",
                    "fn ui(", "fn render(", "fn view(",
                    "Widget", "Component", "Element",
                ];
                
                for indicator in &ui_indicators {
                    if content.contains(indicator) {
                        has_ui_code = true;
                        
                        // Detect specific framework usage
                        if indicator.contains("egui") {
                            framework_usage = Some("egui".to_string());
                        } else if indicator.contains("iced") {
                            framework_usage = Some("iced".to_string());
                        } else if indicator.contains("slint") {
                            framework_usage = Some("slint".to_string());
                        } else if indicator.contains("tauri") {
                            framework_usage = Some("tauri".to_string());
                        }
                        
                        break;
                    }
                }
            }
        }
        
        Ok(SourceFileInfo {
            path: path.to_path_buf(),
            file_type,
            has_ui_code,
            framework_usage,
        })
    }
    
    /// Find configuration files in the project
    fn find_configuration_files(&self) -> CliResult<Vec<String>> {
        let mut config_files = Vec::new();
        
        let config_file_names = [
            "Cargo.toml", "Cargo.lock",
            "rustyui.toml", "rustyui.yaml", "rustyui.json",
            "tauri.conf.json", "src-tauri/tauri.conf.json",
            ".gitignore", ".rustfmt.toml", "rust-toolchain.toml",
            "build.rs", "Makefile", "makefile",
            "README.md", "LICENSE", "CHANGELOG.md",
        ];
        
        for file_name in &config_file_names {
            let file_path = self.project_path.join(file_name);
            if file_path.exists() {
                config_files.push(file_name.to_string());
            }
        }
        
        Ok(config_files)
    }
    
    /// Get project path
    pub fn project_path(&self) -> &Path {
        &self.project_path
    }
}

/// Project analysis results
#[derive(Debug, Default)]
pub struct ProjectAnalysis {
    pub is_rust_project: bool,
    pub is_rustyui_project: bool,
    pub detected_framework: Option<String>,
    pub ui_directories: Vec<String>,
    pub has_rustyui_config: bool,
    pub project_type: ProjectType,
    pub existing_dependencies: Vec<String>,
    pub build_system: BuildSystem,
    pub workspace_info: Option<WorkspaceInfo>,
    pub source_files: Vec<SourceFileInfo>,
    pub configuration_files: Vec<String>,
}

/// Type of Rust project detected
#[derive(Debug, Clone, Default)]
pub enum ProjectType {
    #[default]
    Binary,
    Library,
    Workspace,
    Mixed,
}

/// Build system information
#[derive(Debug, Clone, Default)]
pub struct BuildSystem {
    pub has_cargo_toml: bool,
    pub has_build_rs: bool,
    pub has_makefile: bool,
    pub cargo_features: Vec<String>,
    pub profile_configurations: Vec<String>,
}

/// Workspace information for multi-crate projects
#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub is_workspace_root: bool,
    pub workspace_members: Vec<String>,
    pub workspace_dependencies: Vec<String>,
}

/// Information about source files in the project
#[derive(Debug, Clone)]
pub struct SourceFileInfo {
    pub path: PathBuf,
    pub file_type: SourceFileType,
    pub has_ui_code: bool,
    pub framework_usage: Option<String>,
}

/// Analysis of existing Cargo.toml structure
#[derive(Debug, Default)]
struct CargoTomlAnalysis {
    pub has_dependencies: bool,
    pub has_dev_dependencies: bool,
    pub has_build_dependencies: bool,
    pub has_features: bool,
    pub has_profiles: bool,
    pub has_workspace: bool,
    pub existing_features: Vec<String>,
}

/// Type of source file
#[derive(Debug, Clone)]
pub enum SourceFileType {
    Rust,
    Toml,
    Json,
    Yaml,
    Other(String),
}