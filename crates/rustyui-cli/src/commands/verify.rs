//! Production build verification command

use crate::error::{CliError, CliResult};
use console::style;
use rustyui_core::{ProductionVerifier, VerificationStatus};
use std::path::PathBuf;

/// Command to verify production build zero-overhead
pub struct VerifyCommand {
    path: PathBuf,
    detailed: bool,
    save_report: bool,
}

impl VerifyCommand {
    /// Create a new verify command
    pub fn new(path: PathBuf, detailed: bool, save_report: bool) -> Self {
        Self {
            path,
            detailed,
            save_report,
        }
    }
    
    /// Execute the verify command
    pub fn execute(&mut self) -> CliResult<()> {
        println!("{} Starting production build verification...", style("").blue());
        
        // Create production verifier
        let mut verifier = ProductionVerifier::new(&self.path);
        
        // Run verification
        let results = verifier.verify_production_build()
            .map_err(|e| CliError::dev_mode(format!("Verification failed: {}", e)))?;
        
        // Display results
        self.display_results(results)?;
        
        // Save detailed report if requested
        if self.save_report {
            self.save_detailed_report(results)?;
        }
        
        // Exit with appropriate code based on results
        match results.overall_status {
            VerificationStatus::ZeroOverheadConfirmed => {
                println!("\n{} Zero overhead confirmed! ", style("").green());
                Ok(())
            }
            VerificationStatus::MinorOverheadDetected => {
                println!("\n{} Minor overhead detected - see recommendations", style("").yellow());
                Ok(())
            }
            VerificationStatus::SignificantOverheadDetected => {
                println!("\n{} Significant overhead detected - investigation required", style("").red());
                Err(CliError::dev_mode("Significant overhead detected in production build"))
            }
            VerificationStatus::VerificationFailed => {
                println!("\n{} Verification failed - critical issues found", style("").red());
                Err(CliError::dev_mode("Production build verification failed"))
            }
        }
    }
    
    /// Display verification results
    fn display_results(&self, results: &rustyui_core::VerificationResults) -> CliResult<()> {
        println!("\n{}", style(" Production Build Verification Results").bold());
        println!("{}", "=".repeat(50));
        
        // Overall status
        let status_icon = match results.overall_status {
            VerificationStatus::ZeroOverheadConfirmed => style("").green(),
            VerificationStatus::MinorOverheadDetected => style("").yellow(),
            VerificationStatus::SignificantOverheadDetected => style("").red(),
            VerificationStatus::VerificationFailed => style("💥").red(),
        };
        
        println!("\n{} Overall Status: {}", 
            status_icon, 
            style(format!("{:?}", results.overall_status)).bold()
        );
        
        // Conditional compilation results
        println!("\n{}", style(" Conditional Compilation").bold());
        if results.conditional_compilation_ok {
            println!("{} Development features properly gated", style("").green());
        } else {
            println!("{} Issues with conditional compilation detected", style("").red());
        }
        
        // Binary size results
        println!("\n{}", style("📏 Binary Size Analysis").bold());
        let size_results = &results.binary_size_results;
        
        println!("RustyUI production: {} bytes", 
            style(format_bytes(size_results.rustyui_production_size)).cyan()
        );
        println!("Standard Rust: {} bytes", 
            style(format_bytes(size_results.standard_rust_size)).cyan()
        );
        
        let diff_color = if size_results.size_difference >= 0 { 
            style(format!("+{:.2}%", size_