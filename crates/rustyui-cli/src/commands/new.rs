//! Create a new RustyUI project

use crate::commands::init::InitCommand;
use crate::error::{CliError, CliResult};
use crate::project::ProjectManager;
use console::style;
use std::path::PathBuf;

/// Command to create a new RustyUI project
pub struct NewCommand {
    name: String,
    framework: String,
    yes: bool,
}

impl NewCommand {
    /// Create a new command instance
    pub fn new(name: String, framework: String, yes: bool) -> Self {
        Self {
            name,
            framework,
            yes,
        }
    }
    
    /// Execute the new command
    pub fn execute(&mut self) -> CliResult<()> {
        println!("{} Creating new RustyUI project '{}'...", style("").blue(), self.name);
        
        // Validate project name
        self.validate_project_name()?;
        
        let project_path = PathBuf::from(&self.name);
        
        // Check if directory already exists
        if project_path.exists() {
            return Err(CliError::directory_exists(
                format!("Directory '{}' already exists", self.name)
            ));
        }
        
        // Create the directory
        std::fs::create_dir_all(&project_path)?;
        
        // Create Rust project structure
        let project_manager = ProjectManager::new(project_path.clone())?;
        project_manager.create_rust_project(&self.name)?;
        
        // Initialize RustyUI in the new project
        let mut init_command = InitCommand::new(
            self.framework.clone(),
            project_path.clone(),
            false, // force
            self.yes,
        );
        
        init_command.execute()?;
        
        // Show final success message
        self.show_success_message(&project_path)?;
        
        Ok(())
    }
    
    /// Validate project name
    fn validate_project_name(&self) -> CliResult<()> {
        // Check if name is empty
        if self.name.is_empty() {
            return Err(CliError::project("Project name cannot be empty"));
        }
        
        // Check for invalid characters
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(CliError::project(
                "Project name can only contain alphanumeric characters, hyphens, and underscores"
            ));
        }
        
        // Check if name starts with a number
        if self.name.chars().next().unwrap().is_numeric() {
            return Err(CliError::project("Project name cannot start with a number"));
        }
        
        // Check for reserved names
        let reserved_names = ["test", "src", "target", "cargo", "rust", "main"];
        if reserved_names.contains(&self.name.as_str()) {
            return Err(CliError::project(
                format!("'{}' is a reserved name and cannot be used", self.name)
            ));
        }
        
        Ok(())
    }
    
    /// Show final success message
    fn show_success_message(&self, _project_path: &PathBuf) -> CliResult<()> {
        println!("\n{} Successfully created RustyUI project '{}'!", style("").green(), self.name);
        println!("\n{}", style("Get started:").bold());
        println!("{}", style(format!("cd {}", self.name)).cyan());
        println!("{}", style("rustyui dev").cyan());
        println!("\n{}", style("Project structure:").bold());
        println!("{}/", self.name);
        println!("├── src/");
        println!("│   └── main.rs      # Main application with hot reload");
        println!("├── rustyui.toml     # RustyUI configuration");
        println!("├── Cargo.toml       # Rust dependencies");
        println!("└── README.md        # Project documentation");
        
        println!("\n{}", style("Features enabled:").bold());
        println!("Instant hot reload for {} UI", self.framework);
        println!("💾 State preservation across changes");
        println!("Zero overhead production builds");
        println!("📝 Example code ready to run");
        
        Ok(())
    }
}