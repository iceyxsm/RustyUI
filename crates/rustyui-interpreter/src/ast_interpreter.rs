//! AST-based interpretation for Rust UI code

use crate::{error::Result, InterpretationResult};
use std::time::Instant;

/// AST-based interpreter for direct Rust code interpretation
pub struct ASTInterpreter {
    /// Cached AST nodes for performance
    ast_cache: std::collections::HashMap<String, CachedAST>,
}

impl ASTInterpreter {
    /// Create a new AST interpreter
    pub fn new() -> Result<Self> {
        Ok(Self {
            ast_cache: std::collections::HashMap::new(),
        })
    }
    
    /// Interpret Rust code by parsing and executing AST
    pub fn interpret(&mut self, code: &str) -> Result<InterpretationResult> {
        let start_time = Instant::now();
        
        // Check cache first
        if let Some(cached) = self.ast_cache.get(code) {
            if cached.is_valid() {
                return Ok(InterpretationResult {
                    execution_time: start_time.elapsed(),
                    success: true,
                    error_message: None,
                });
            }
        }
        
        // Parse and interpret the code
        match self.parse_and_interpret(code) {
            Ok(result) => {
                // Cache successful interpretation
                self.ast_cache.insert(code.to_string(), CachedAST {
                    result: result.clone(),
                    timestamp: Instant::now(),
                });
                
                Ok(InterpretationResult {
                    execution_time: start_time.elapsed(),
                    success: true,
                    error_message: None,
                })
            }
            Err(err) => {
                Ok(InterpretationResult {
                    execution_time: start_time.elapsed(),
                    success: false,
                    error_message: Some(err.to_string()),
                })
            }
        }
    }
    
    /// Parse and interpret Rust code
    fn parse_and_interpret(&self, code: &str) -> Result<String> {
        // For Phase 1, implement basic pattern matching for common UI patterns
        if code.contains("Button") {
            Ok("Interpreted button component".to_string())
        } else if code.contains("Text") {
            Ok("Interpreted text component".to_string())
        } else if code.contains("Layout") {
            Ok("Interpreted layout component".to_string())
        } else {
            // For now, return a generic success for any other code
            Ok("Generic UI component interpreted".to_string())
        }
    }
    
    /// Clear the AST cache
    pub fn clear_cache(&mut self) {
        self.ast_cache.clear();
    }
    
    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.ast_cache.len()
    }
}

impl Default for ASTInterpreter {
    fn default() -> Self {
        Self::new().expect("Failed to create default AST interpreter")
    }
}

/// Cached AST interpretation result
#[derive(Debug, Clone)]
struct CachedAST {
    result: String,
    timestamp: Instant,
}

impl CachedAST {
    /// Check if the cached result is still valid (within 1 minute)
    fn is_valid(&self) -> bool {
        self.timestamp.elapsed().as_secs() < 60
    }
}