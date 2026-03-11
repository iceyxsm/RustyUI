//! Optimized AST interpreter with lazy parsing and caching

//! - Lazy regex compilation for syntax parsing
//! - AST caching for repeated code patterns
//! - Memory-efficient parsing strategies

use crate::{InterpreterError, Result};
use syn::{File, Item, Expr, Stmt};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

/// Lazy-initialized regex patterns for AST parsing
static PARSING_PATTERNS: LazyLock<ParsingPatterns> = LazyLock::new(|| ParsingPatterns::new());

/// Optimized AST interpreter with caching and performance monitoring
pub struct ASTInterpreter {
    /// Parsing and execution statistics
    stats: ASTStats,
    
    /// Local AST cache (not global to avoid thread safety issues)
    ast_cache: HashMap<String, CachedAST>,
    
    /// Cache cleanup threshold
    cache_cleanup_threshold: usize,
    
    /// Maximum AST depth to prevent stack overflow
    max_ast_depth: usize,
}

impl ASTInterpreter {
    /// Create a new AST interpreter
    pub fn new() -> Result<Self> {
        Ok(Self {
            stats: ASTStats::new(),
            ast_cache: HashMap::new(),
            cache_cleanup_threshold: 50, // Clean cache when it exceeds 50 entries
            max_ast_depth: 100,          // Prevent deeply nested ASTs
        })
    }
    
    /// Interpret Rust code by parsing and executing AST
    pub fn interpret(&mut self, code: &str) -> Result<crate::InterpretationResult> {
        let start_time = Instant::now();
        
        // Calculate code hash for caching
        let code_hash = self.calculate_code_hash(code);
        
        // Check local AST cache first
        if let Some(cached_ast) = self.ast_cache.get(&code_hash) {
            // Execute cached AST
            let execution_result = self.execute_ast(&cached_ast.ast);
            
            // Update cache statistics (need to get mutable reference after immutable borrow ends)
            if let Some(cached_ast) = self.ast_cache.get_mut(&code_hash) {
                cached_ast.hit_count += 1;
                cached_ast.last_used = Instant::now();
            }
            self.stats.cache_hits += 1;
            
            return Ok(crate::InterpretationResult {
                execution_time: start_time.elapsed(),
                success: execution_result.is_ok(),
                error_message: execution_result.err().map(|e| e.to_string()),
            });
        }
        
        // AST not cached, parse and execute
        self.stats.cache_misses += 1;
        
        // Parse code to AST
        let parsing_start = Instant::now();
        let ast = self.parse_code(code)?;
        let parsing_time = parsing_start.elapsed();
        
        // Validate AST depth
        let ast_depth = self.calculate_ast_depth(&ast);
        if ast_depth > self.max_ast_depth {
            return Err(InterpreterError::compilation(
                format!("AST depth {} exceeds maximum {}", ast_depth, self.max_ast_depth)
            ));
        }
        
        // Execute the AST
        let execution_result = self.execute_ast(&ast);
        
        // Cache the parsed AST for future use
        self.ast_cache.insert(code_hash, CachedAST {
            ast,
            parsing_time,
            hit_count: 1,
            last_used: Instant::now(),
            ast_depth,
        });
        
        // Cleanup cache if it gets too large
        if self.ast_cache.len() > self.cache_cleanup_threshold {
            self.cleanup_cache();
        }
        
        self.stats.total_interpretations += 1;
        self.stats.total_interpretation_time += start_time.elapsed();
        
        Ok(crate::InterpretationResult {
            execution_time: start_time.elapsed(),
            success: execution_result.is_ok(),
            error_message: execution_result.err().map(|e| e.to_string()),
        })
    }
    
    /// Parse Rust code to AST using optimized parsing
    fn parse_code(&self, code: &str) -> Result<File> {
        // Use lazy-initialized parsing patterns for better performance
        let patterns = &*PARSING_PATTERNS;
        
        // Pre-validate code structure using regex patterns
        if !patterns.is_valid_rust_structure(code) {
            return Err(InterpreterError::compilation("Invalid Rust code structure"));
        }
        
        // Parse using syn
        syn::parse_file(code)
            .map_err(|e| InterpreterError::compilation(format!("AST parsing failed: {}", e)))
    }
    
    /// Execute an AST by interpreting its items
    fn execute_ast(&self, ast: &File) -> Result<()> {
        for item in &ast.items {
            self.execute_item(item)?;
        }
        Ok(())
    }
    
    /// Execute a single AST item
    fn execute_item(&self, item: &Item) -> Result<()> {
        match item {
            Item::Fn(func) => {
                println!("Executing function: {}", func.sig.ident);
                // In a real implementation, this would execute the function
                Ok(())
            }
            Item::Struct(struct_item) => {
                println!("Processing struct: {}", struct_item.ident);
                // In a real implementation, this would register the struct
                Ok(())
            }
            Item::Impl(_impl_item) => {
                println!("Processing impl block");
                // In a real implementation, this would process the implementation
                Ok(())
            }
            Item::Use(use_item) => {
                println!("Processing use statement: {}", quote::quote!(#use_item));
                // In a real implementation, this would handle imports
                Ok(())
            }
            _ => {
                // Handle other item types as needed
                Ok(())
            }
        }
    }
    
    /// Calculate AST depth to prevent stack overflow
    fn calculate_ast_depth(&self, ast: &File) -> usize {
        let mut max_depth = 0;
        
        for item in &ast.items {
            let item_depth = self.calculate_item_depth(item, 1);
            max_depth = max_depth.max(item_depth);
        }
        
        max_depth
    }
    
    /// Calculate depth of a single item
    fn calculate_item_depth(&self, item: &Item, current_depth: usize) -> usize {
        match item {
            Item::Fn(func) => {
                let mut max_depth = current_depth;
                
                // Function blocks are always present in ItemFn
                for stmt in &func.block.stmts {
                    let stmt_depth = self.calculate_stmt_depth(stmt, current_depth + 1);
                    max_depth = max_depth.max(stmt_depth);
                }
                
                max_depth
            }
            Item::Impl(impl_item) => {
                let mut max_depth = current_depth;
                
                for impl_item in &impl_item.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        // Calculate depth for method blocks
                        for stmt in &method.block.stmts {
                            let stmt_depth = self.calculate_stmt_depth(stmt, current_depth + 1);
                            max_depth = max_depth.max(stmt_depth);
                        }
                    }
                }
                
                max_depth
            }
            _ => current_depth,
        }
    }
    
    /// Calculate depth of a statement
    fn calculate_stmt_depth(&self, stmt: &Stmt, current_depth: usize) -> usize {
        match stmt {
            Stmt::Expr(expr, _) => {
                self.calculate_expr_depth(expr, current_depth)
            }
            Stmt::Local(local) => {
                if let Some(ref init_expr) = local.init.as_ref().map(|init| &init.expr) {
                    self.calculate_expr_depth(init_expr, current_depth)
                } else {
                    current_depth
                }
            }
            _ => current_depth,
        }
    }
    
    /// Calculate depth of an expression
    fn calculate_expr_depth(&self, expr: &Expr, current_depth: usize) -> usize {
        match expr {
            Expr::Block(block_expr) => {
                let mut max_depth = current_depth;
                
                for stmt in &block_expr.block.stmts {
                    let stmt_depth = self.calculate_stmt_depth(stmt, current_depth + 1);
                    max_depth = max_depth.max(stmt_depth);
                }
                
                max_depth
            }
            Expr::If(if_expr) => {
                let mut max_depth = self.calculate_expr_depth(&if_expr.cond, current_depth + 1);
                
                // Handle the then branch - it's a Block, not an ExprBlock
                for stmt in &if_expr.then_branch.stmts {
                    let stmt_depth = self.calculate_stmt_depth(stmt, current_depth + 1);
                    max_depth = max_depth.max(stmt_depth);
                }
                
                if let Some((_, ref else_branch)) = if_expr.else_branch {
                    max_depth = max_depth.max(self.calculate_expr_depth(else_branch, current_depth + 1));
                }
                
                max_depth
            }
            Expr::Match(match_expr) => {
                let mut max_depth = self.calculate_expr_depth(&match_expr.expr, current_depth + 1);
                
                for arm in &match_expr.arms {
                    max_depth = max_depth.max(self.calculate_expr_depth(&arm.body, current_depth + 1));
                }
                
                max_depth
            }
            _ => current_depth,
        }
    }
    
    /// Calculate hash for code caching
    fn calculate_code_hash(&self, code: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    /// Clean up old cached ASTs to manage memory
    fn cleanup_cache(&mut self) {
        let now = Instant::now();
        let max_age = Duration::from_secs(600); // 10 minutes
        
        // Remove ASTs that haven't been used recently
        self.ast_cache.retain(|_, ast| {
            now.duration_since(ast.last_used) < max_age || ast.hit_count > 3
        });
        
        // If still too large, remove least frequently used ASTs
        if self.ast_cache.len() > self.cache_cleanup_threshold {
            let mut asts: Vec<_> = self.ast_cache.iter().map(|(k, v)| (k.clone(), v.hit_count)).collect();
            asts.sort_by_key(|(_, hit_count)| *hit_count);
            
            let remove_count = self.ast_cache.len() - (self.cache_cleanup_threshold * 3 / 4);
            for (hash, _) in asts.iter().take(remove_count) {
                self.ast_cache.remove(hash);
            }
        }
    }
    
    /// Get interpretation statistics
    pub fn get_stats(&self) -> &ASTStats {
        &self.stats
    }
    
    /// Get cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total_requests = self.stats.cache_hits + self.stats.cache_misses;
        if total_requests == 0 {
            0.0
        } else {
            self.stats.cache_hits as f64 / total_requests as f64
        }
    }
    
    /// Get average interpretation time
    pub fn average_interpretation_time(&self) -> Duration {
        if self.stats.total_interpretations == 0 {
            Duration::from_secs(0)
        } else {
            self.stats.total_interpretation_time / self.stats.total_interpretations
        }
    }
    
    /// Clear all cached ASTs
    pub fn clear_cache(&mut self) -> Result<()> {
        self.ast_cache.clear();
        Ok(())
    }
    
    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.ast_cache.len()
    }
}

/// Cached AST with metadata
struct CachedAST {
    /// Parsed AST
    ast: File,
    
    /// Time taken to parse this AST
    parsing_time: Duration,
    
    /// Number of times this AST has been used
    hit_count: u64,
    
    /// Last time this AST was used
    last_used: Instant,
    
    /// Depth of the AST
    ast_depth: usize,
}

/// AST interpretation statistics
#[derive(Debug, Clone)]
pub struct ASTStats {
    /// Number of cache hits
    pub cache_hits: u64,
    
    /// Number of cache misses
    pub cache_misses: u64,
    
    /// Total number of interpretations
    pub total_interpretations: u32,
    
    /// Total time spent on interpretation
    pub total_interpretation_time: Duration,
}

impl ASTStats {
    fn new() -> Self {
        Self {
            cache_hits: 0,
            cache_misses: 0,
            total_interpretations: 0,
            total_interpretation_time: Duration::from_secs(0),
        }
    }
}

/// Parsing patterns for optimized code validation
struct ParsingPatterns {
    /// Pattern for function definitions
    function_pattern: regex::Regex,
    
    /// Pattern for struct definitions
    struct_pattern: regex::Regex,
    
    /// Pattern for impl blocks
    impl_pattern: regex::Regex,
    
    /// Pattern for use statements
    use_pattern: regex::Regex,
}

impl ParsingPatterns {
    fn new() -> Self {
        Self {
            function_pattern: regex::Regex::new(r"(?m)^fn\s+\w+\s*\(").unwrap(),
            struct_pattern: regex::Regex::new(r"(?m)^struct\s+\w+\s*\{").unwrap(),
            impl_pattern: regex::Regex::new(r"(?m)^impl\s+").unwrap(),
            use_pattern: regex::Regex::new(r"(?m)^use\s+").unwrap(),
        }
    }
    
    /// Check if code has valid Rust structure using regex patterns
    fn is_valid_rust_structure(&self, code: &str) -> bool {
        // Basic validation - code should have at least one recognizable Rust construct
        self.function_pattern.is_match(code) ||
        self.struct_pattern.is_match(code) ||
        self.impl_pattern.is_match(code) ||
        self.use_pattern.is_match(code) ||
        code.trim().is_empty() // Allow empty code
    }
}

/// AST optimization strategies
#[derive(Debug, Clone)]
pub enum ASTOptimizationStrategy {
    /// Fast parsing with minimal validation
    Fast,
    
    /// Balanced parsing with moderate validation
    Balanced,
    
    /// Thorough parsing with extensive validation
    Thorough,
}

impl ASTOptimizationStrategy {
    /// Get maximum AST depth for this strategy
    pub fn max_ast_depth(&self) -> usize {
        match self {
            ASTOptimizationStrategy::Fast => 50,
            ASTOptimizationStrategy::Balanced => 100,
            ASTOptimizationStrategy::Thorough => 200,
        }
    }
    
    /// Get cache retention time for this strategy
    pub fn cache_retention_time(&self) -> Duration {
        match self {
            ASTOptimizationStrategy::Fast => Duration::from_secs(300),      // 5 minutes
            ASTOptimizationStrategy::Balanced => Duration::from_secs(600),  // 10 minutes
            ASTOptimizationStrategy::Thorough => Duration::from_secs(1800), // 30 minutes
        }
    }
}

/// Advanced AST interpretation features
pub struct AdvancedASTFeatures {
    /// Optimization strategy
    optimization_strategy: ASTOptimizationStrategy,
    
    /// Whether to perform semantic analysis
    semantic_analysis: bool,
    
    /// Whether to validate type information
    type_validation: bool,
}

impl AdvancedASTFeatures {
    pub fn new() -> Self {
        Self {
            optimization_strategy: ASTOptimizationStrategy::Balanced,
            semantic_analysis: false, // Disabled for performance
            type_validation: false,   // Disabled for performance
        }
    }
    
    /// Set optimization strategy
    pub fn set_optimization_strategy(&mut self, strategy: ASTOptimizationStrategy) {
        self.optimization_strategy = strategy;
    }
    
    /// Enable or disable semantic analysis
    pub fn set_semantic_analysis(&mut self, enabled: bool) {
        self.semantic_analysis = enabled;
    }
    
    /// Enable or disable type validation
    pub fn set_type_validation(&mut self, enabled: bool) {
        self.type_validation = enabled;
    }
}