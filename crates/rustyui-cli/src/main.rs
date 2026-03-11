//! RustyUI CLI tool for project initialization and development

mod commands;
mod config;
mod error;
mod project;
mod template;

use clap::{Parser, Subcommand};
use console::style;
use std::path::PathBuf;

use crate::commands::{dev::DevCommand, init::InitCommand, new::NewCommand};
use crate::error::CliResult;

#[derive(Parser)]
#[command(name = "rustyui")]
#[command(about = "Revolutionary dual-mode UI development system for Rust")]
#[command(version = "0.1.0")]
#[command(long_about = "
RustyUI provides instant UI feedback during development through runtime interpretation
while maintaining zero overhead in production builds. Inspired by Flutter's JIT/AOT
architecture and modern game engine live coding systems.

Features:
  • 0ms compilation time for UI changes during development
  • Zero runtime overhead in production builds
  • Framework-agnostic (egui, iced, slint, tauri)
  • State preservation across interpretation cycles
  • Seamless transition between development and production modes
")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize RustyUI in an existing project
    Init {
        /// UI framework to use
        #[arg(long, default_value = "egui")]
        #[arg(value_parser = ["egui", "iced", "slint", "tauri"])]
        framework: String,
        
        /// Project directory
        #[arg(long, default_value = ".")]
        path: PathBuf,
        
        /// Force initialization even if rustyui.toml exists
        #[arg(long)]
        force: bool,
        
        /// Skip interactive prompts and use defaults
        #[arg(long)]
        yes: bool,
    },
    
    /// Create a new RustyUI project
    New {
        /// Project name
        name: String,
        
        /// UI framework to use
        #[arg(long, default_value = "egui")]
        #[arg(value_parser = ["egui", "iced", "slint", "tauri"])]
        framework: String,
        
        /// Skip interactive prompts and use defaults
        #[arg(long)]
        yes: bool,
    },
    
    /// Start development mode with hot reload
    Dev {
        /// Project directory
        #[arg(long, default_value = ".")]
        path: PathBuf,
        
        /// Port for development server (if applicable)
        #[arg(long)]
        port: Option<u16>,
        
        /// Disable file watching
        #[arg(long)]
        no_watch: bool,
    },
    
    /// Build project for production
    Build {
        /// Project directory
        #[arg(long, default_value = ".")]
        path: PathBuf,
        
        /// Release build
        #[arg(long)]
        release: bool,
    },
    
    /// Show project configuration
    Config {
        /// Project directory
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

fn main() -> CliResult<()> {
    let cli = Cli::parse();
    
    // Set up logging based on verbosity
    setup_logging(cli.verbose, cli.quiet)?;
    
    // Check platform compatibility
    check_platform_compatibility(cli.verbose)?;
    
    match cli.command {
        Commands::Init { framework, path, force, yes } => {
            let mut cmd = InitCommand::new(framework, path, force, yes);
            cmd.execute()?;
        }
        Commands::New { name, framework, yes } => {
            let mut cmd = NewCommand::new(name, framework, yes);
            cmd.execute()?;
        }
        Commands::Dev { path, port, no_watch } => {
            let mut cmd = DevCommand::new(path, port, !no_watch);
            cmd.execute()?;
        }
        Commands::Build { path, release } => {
            build_project(&path, release)?;
        }
        Commands::Config { path } => {
            show_config(&path)?;
        }
    }
    
    Ok(())
}

/// Set up logging based on verbosity flags
fn setup_logging(verbose: bool, quiet: bool) -> CliResult<()> {
    if quiet {
        // Only show errors
        return Ok(());
    }
    
    if verbose {
        println!("{}", style("RustyUI CLI - Verbose mode enabled").dim());
    }
    
    Ok(())
}

/// Check platform compatibility and display information
fn check_platform_compatibility(verbose: bool) -> CliResult<()> {
    use rustyui_core::{Platform, PlatformCapabilities, PlatformConfig};
    
    let platform = Platform::current();
    
    if verbose {
        println!("{} Detected platform: {}", style("ℹ").blue(), style(platform.to_string()).cyan());
    }
    
    // Check minimum requirements
    if let Err(e) = PlatformCapabilities::check_minimum_requirements() {
        eprintln!("{} Platform compatibility issue: {}", style("⚠").yellow(), e);
        eprintln!("  RustyUI may not work optimally on this platform.");
    }
    
    // Show platform capabilities if verbose
    if verbose {
        let config = PlatformConfig::auto_detect();
        let jit_caps = platform.jit_capabilities();
        let watcher_backend = platform.file_watcher_backend();
        let watcher_perf = watcher_backend.performance_characteristics();
        
        println!("{} Platform capabilities:", style("🔧").green());
        println!("  File watcher: {} ({}ms latency)", 
            match watcher_backend {
                rustyui_core::FileWatcherBackend::ReadDirectoryChanges => "ReadDirectoryChanges",
                rustyui_core::FileWatcherBackend::FSEvents => "FSEvents",
                rustyui_core::FileWatcherBackend::INotify => "inotify", 
                rustyui_core::FileWatcherBackend::Poll => "polling",
            },
            watcher_perf.latency_ms
        );
        println!("  JIT compilation: {}", 
            if jit_caps.supports_cranelift { "supported" } else { "not supported" }
        );
        println!("  Native APIs: {}", 
            if config.use_native_apis { "enabled" } else { "disabled" }
        );
        println!("  Thread count: {}", config.thread_count);
    }
    
    Ok(())
}

/// Build project for production
fn build_project(path: &PathBuf, release: bool) -> CliResult<()> {
    use crate::project::ProjectManager;
    
    let project_manager = ProjectManager::new(path.clone())?;
    project_manager.build_production(release)?;
    
    println!("{} Production build completed", style("✓").green());
    
    Ok(())
}

/// Show current project configuration
fn show_config(path: &PathBuf) -> CliResult<()> {
    use crate::config::ConfigManager;
    use rustyui_core::{Platform, PlatformConfig};
    
    let config_manager = ConfigManager::new(path.clone())?;
    let config = config_manager.load_config()?;
    let platform = Platform::current();
    let platform_config = PlatformConfig::auto_detect();
    
    println!("{}", style("RustyUI Configuration:").bold());
    println!("  Framework: {}", style(&format!("{:?}", config.framework)).cyan());
    println!("  Watch paths: {:?}", config.watch_paths);
    
    println!("\n{}", style("Platform Information:").bold());
    println!("  Platform: {}", style(platform.to_string()).cyan());
    println!("  File watcher: {}", match platform_config.file_watcher_backend {
        rustyui_core::FileWatcherBackend::ReadDirectoryChanges => "ReadDirectoryChanges",
        rustyui_core::FileWatcherBackend::FSEvents => "FSEvents",
        rustyui_core::FileWatcherBackend::INotify => "inotify",
        rustyui_core::FileWatcherBackend::Poll => "polling",
    });
    println!("  JIT compilation: {}", 
        if platform_config.use_jit_compilation { "enabled" } else { "disabled" }
    );
    println!("  Native optimizations: {}", 
        if platform_config.use_native_apis { "enabled" } else { "disabled" }
    );
    println!("  Thread count: {}", platform_config.thread_count);
    
    #[cfg(feature = "dev-ui")]
    {
        println!("\n{}", style("Development Settings:").bold());
        println!("    Interpretation strategy: {:?}", config.development_settings.interpretation_strategy);
        println!("    JIT threshold: {}ms", config.development_settings.jit_compilation_threshold);
        println!("    State preservation: {}", config.development_settings.state_preservation);
        println!("    Performance monitoring: {}", config.development_settings.performance_monitoring);
        println!("    Change detection delay: {}ms", config.development_settings.change_detection_delay_ms);
    }
    
    println!("\n{}", style("Production Settings:").bold());
    println!("    Strip dev features: {}", config.production_settings.strip_dev_features);
    println!("    Optimization level: {:?}", config.production_settings.optimization_level);
    println!("    Binary size optimization: {}", config.production_settings.binary_size_optimization);
    println!("    Security hardening: {}", config.production_settings.security_hardening);
    
    Ok(())
}