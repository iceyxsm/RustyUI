use crate::tiered_compilation::CompilationTier;
use crate::profiling::ProfileData;
use crate::hot_path_detector::HotCallSite;
use cranelift_codegen::ir::{Function as CraneliftFunction, InstBuilder};
use cranelift_codegen::settings::OptLevel;
use cranelift_codegen::{Context, CodegenError};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Module, FuncId};
use std::time::{Duration, Instant};

#[cfg(test)]
mod property_tests;

/// Simplified profile data snapshot for cloning
#[derive(Debug, Clone)]
pub struct ProfileDataSnapshot {
    pub function_id: String,
    pub execution_count: u64,
    pub branch_count: usize,
    pub loop_count: usize,
    pub call_site_count: usize,
    pub type_feedback_count: usize,
}

impl ProfileDataSnapshot {
    pub fn from_profile(profile: &ProfileData) -> Self {
        Self {
            function_id: profile.function_id.clone(),
            execution_count: profile.get_execution_count(),
            branch_count: profile.branch_stats.len(),
            loop_count: profile.loop_stats.len(),
            call_site_count: profile.call_site_stats.len(),
            type_feedback_count: profile.type_feedback.len(),
        }
    }
}

/// Optimization engine responsible for applying profile-guided optimizations
/// at each compilation tier using Cranelift IR generation and optimization.
pub struct OptimizationEngine {
    /// Cranelift code generation context
    codegen_context: Context,
    
    /// JIT module for code generation
    jit_module: JITModule,
    
    /// Optimization pass manager
    pass_manager: OptimizationPassManager,
    
    /// Profile-guided inliner
    inliner: ProfileGuidedInliner,
    
    /// Speculative optimization support
    speculation: SpeculativeOptimizer,
    
    /// Inlining configuration
    inlining_config: InliningConfig,
}

impl OptimizationEngine {
    /// Create a new optimization engine
    pub fn new() -> Result<Self, OptimizationError> {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())?;
        let jit_module = JITModule::new(builder);
        
        Ok(Self {
            codegen_context: Context::new(),
            jit_module,
            pass_manager: OptimizationPassManager::new(),
            inliner: ProfileGuidedInliner::new(),
            speculation: SpeculativeOptimizer::new(),
            inlining_config: InliningConfig::default(),
        })
    }
    
    /// Compile function at specified tier with profile data
    pub fn compile_with_profile(
        &mut self,
        source: &str,
        tier: CompilationTier,
        profile: &ProfileData,
    ) -> Result<CompiledCode, OptimizationError> {
        let start_time = Instant::now();
        
        // Parse source code to Cranelift IR
        let mut func = self.parse_to_cranelift_ir(source)?;
        
        // Apply tier-specific optimizations based on tier parameter
        self.apply_tier_optimizations(&mut func, tier, profile)?;
        
        // Apply profile-guided optimizations using profile data
        self.apply_profile_guided_optimizations(&mut func, profile)?;
        
        // Generate machine code
        let compiled_code = self.generate_machine_code(func, tier, profile)?;
        
        let compilation_time = start_time.elapsed();
        
        // Verify compilation time budget
        let budget = tier.compilation_time_budget();
        if compilation_time > budget {
            return Err(OptimizationError::CompilationTimeBudgetExceeded {
                tier,
                actual: compilation_time,
                budget,
            });
        }
        
        // Create compiled code with metadata
        Ok(CompiledCode {
            function_id: profile.function_id.clone(),
            tier,
            version: self.generate_version_number(&profile.function_id),
            code_ptr: compiled_code.code_ptr,
            code_size: compiled_code.code_size,
            cranelift_func_id: compiled_code.cranelift_func_id,
            deopt_info: compiled_code.deopt_info,
            compiled_at: Instant::now(),
            compilation_time,
            profile_snapshot: ProfileDataSnapshot::from_profile(profile),
        })
    }
    
    /// Generate version number for compiled code
    fn generate_version_number(&self, function_id: &str) -> u64 {
        // TODO: Implement proper version tracking
        // For now, use timestamp as version
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    
    /// Parse source code to Cranelift IR
    fn parse_to_cranelift_ir(&mut self, source: &str) -> Result<CraneliftFunction, OptimizationError> {
        // TODO: Implement proper parsing from source to Cranelift IR
        // For now, create a minimal function structure with proper entry block
        let mut func = CraneliftFunction::new();
        
        // Add basic function signature
        let signature = self.jit_module.make_signature();
        func.signature = signature;
        
        // Create a function builder to add basic blocks
        let mut builder_context = cranelift_frontend::FunctionBuilderContext::new();
        let mut builder = cranelift_frontend::FunctionBuilder::new(&mut func, &mut builder_context);
        
        // Create entry block
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);
        
        // Add a simple return instruction
        let return_val = builder.ins().iconst(cranelift_codegen::ir::types::I32, 42);
        builder.ins().return_(&[return_val]);
        
        // Finalize the function
        builder.finalize();
        
        Ok(func)
    }
    
    /// Apply tier-specific optimizations
    fn apply_tier_optimizations(
        &mut self,
        func: &mut CraneliftFunction,
        tier: CompilationTier,
        profile: &ProfileData,
    ) -> Result<(), OptimizationError> {
        match tier {
            CompilationTier::Interpreter => {
                // No optimizations for interpreter tier
                Ok(())
            }
            CompilationTier::QuickJIT => {
                // Tier 1: Minimal optimizations, target <5ms
                // Note: Optimization level is set during code generation
                self.pass_manager.apply_quick_optimizations(func)?;
                Ok(())
            }
            CompilationTier::OptimizedJIT => {
                // Tier 2: Moderate optimizations with limited inlining, target <20ms
                // Note: Optimization level is set during code generation
                self.pass_manager.apply_moderate_optimizations(func)?;
                
                // Apply limited profile-guided inlining
                let hot_call_sites = self.extract_hot_call_sites(profile, 50); // Max 50 IR instructions
                self.inliner.inline_hot_calls(func, &hot_call_sites, &self.inlining_config)?;
                
                Ok(())
            }
            CompilationTier::AggressiveJIT => {
                // Tier 3: Aggressive optimizations with full inlining, target <100ms
                // Note: Optimization level is set during code generation
                self.pass_manager.apply_aggressive_optimizations(func)?;
                
                // Apply aggressive profile-guided inlining
                let hot_call_sites = self.extract_hot_call_sites(profile, 200); // Max 200 IR instructions
                self.inliner.inline_hot_calls(func, &hot_call_sites, &self.inlining_config)?;
                
                // Apply speculative optimizations with guards
                self.speculation.apply_speculative_opts(func, profile)?;
                
                Ok(())
            }
        }
    }
    
    /// Apply profile-guided optimizations
    fn apply_profile_guided_optimizations(
        &mut self,
        func: &mut CraneliftFunction,
        profile: &ProfileData,
    ) -> Result<(), OptimizationError> {
        // Apply loop unrolling for hot loops
        self.pass_manager.apply_loop_unrolling(func, profile)?;
        
        // Apply dead code elimination based on profile data
        self.pass_manager.apply_dead_code_elimination(func, profile)?;
        
        // Apply branch prediction optimization
        self.pass_manager.apply_branch_prediction_optimization(func, profile)?;
        
        // Apply constant propagation with runtime values
        self.pass_manager.apply_constant_propagation(func, profile)?;
        
        Ok(())
    }
    
    /// Extract hot call sites from profile data
    fn extract_hot_call_sites(&self, profile: &ProfileData, max_inline_size: usize) -> Vec<HotCallSite> {
        let mut hot_call_sites = Vec::new();
        
        for call_site_ref in &profile.call_site_stats {
            let call_site_id = *call_site_ref.key();
            let call_stats = call_site_ref.value();
            let call_count = call_stats.call_count.load(std::sync::atomic::Ordering::Relaxed);
            
            // Check if call site is hot enough for inlining
            if call_count >= self.inlining_config.min_call_frequency {
                // Check if target function is small enough for inlining
                if let Some(hot_target) = call_stats.hot_target() {
                    // TODO: Get actual function size from somewhere
                    let estimated_size = 30; // Placeholder
                    
                    if estimated_size <= max_inline_size {
                        hot_call_sites.push(HotCallSite {
                            call_site_id,
                            call_count,
                            target_function: hot_target.clone(),
                            is_monomorphic: call_stats.target_frequencies.len() == 1,
                            inline_benefit_score: call_count as f64 * (1.0 / estimated_size as f64),
                        });
                    }
                }
            }
        }
        
        // Sort by benefit score (highest first)
        hot_call_sites.sort_by(|a, b| b.inline_benefit_score.partial_cmp(&a.inline_benefit_score).unwrap());
        
        hot_call_sites
    }
    
    /// Generate machine code from Cranelift IR
    fn generate_machine_code(
        &mut self,
        func: CraneliftFunction,
        tier: CompilationTier,
        profile: &ProfileData,
    ) -> Result<GeneratedCode, OptimizationError> {
        // Compile the function
        self.codegen_context.func = func;
        let func_id = self.jit_module.declare_function(
            &format!("func_{}", profile.function_id),
            cranelift_module::Linkage::Local,
            &self.codegen_context.func.signature,
        )?;
        
        self.jit_module.define_function(func_id, &mut self.codegen_context)?;
        self.jit_module.finalize_definitions()?;
        
        let code_ptr = self.jit_module.get_finalized_function(func_id);
        
        Ok(GeneratedCode {
            code_ptr,
            code_size: 0, // TODO: Get actual code size
            cranelift_func_id: func_id,
            deopt_info: None, // TODO: Generate deoptimization info for speculative optimizations
        })
    }
}

/// Optimization pass manager for applying different optimization passes
pub struct OptimizationPassManager {
    /// Configuration for optimization passes
    config: OptimizationPassConfig,
}

impl OptimizationPassManager {
    pub fn new() -> Self {
        Self {
            config: OptimizationPassConfig::default(),
        }
    }
    
    /// Apply quick optimizations for Tier 1
    pub fn apply_quick_optimizations(&self, func: &mut CraneliftFunction) -> Result<(), OptimizationError> {
        // Basic constant folding
        // Simple register allocation
        // No inlining
        Ok(())
    }
    
    /// Apply moderate optimizations for Tier 2
    pub fn apply_moderate_optimizations(&self, func: &mut CraneliftFunction) -> Result<(), OptimizationError> {
        // All quick optimizations plus:
        self.apply_quick_optimizations(func)?;
        
        // Branch prediction hints
        // Limited loop optimizations
        Ok(())
    }
    
    /// Apply aggressive optimizations for Tier 3
    pub fn apply_aggressive_optimizations(&self, func: &mut CraneliftFunction) -> Result<(), OptimizationError> {
        // All moderate optimizations plus:
        self.apply_moderate_optimizations(func)?;
        
        // Escape analysis
        // Advanced loop optimizations
        // Vectorization candidates
        Ok(())
    }
    
    /// Apply loop unrolling for hot loops with predictable iteration counts
    pub fn apply_loop_unrolling(&self, func: &mut CraneliftFunction, profile: &ProfileData) -> Result<(), OptimizationError> {
        for loop_ref in &profile.loop_stats {
            let loop_id = *loop_ref.key();
            let loop_stats = loop_ref.value();
            let execution_count = loop_stats.execution_count.load(std::sync::atomic::Ordering::Relaxed);
            let avg_iterations = loop_stats.average_iterations();
            
            // Check if loop is hot and has predictable iteration count
            if execution_count >= self.config.hot_loop_threshold {
                let variance = (loop_stats.max_iterations() as f64 - loop_stats.min_iterations() as f64) / avg_iterations;
                if variance < 0.1 { // Less than 10% variance
                    // Apply loop unrolling
                    // TODO: Implement actual loop unrolling in Cranelift IR
                }
            }
        }
        Ok(())
    }
    
    /// Apply dead code elimination for never-executed paths
    pub fn apply_dead_code_elimination(&self, func: &mut CraneliftFunction, profile: &ProfileData) -> Result<(), OptimizationError> {
        // TODO: Implement dead code elimination based on branch statistics
        // Remove code paths that have execution count = 0
        Ok(())
    }
    
    /// Apply branch prediction optimization for biased branches
    pub fn apply_branch_prediction_optimization(&self, func: &mut CraneliftFunction, profile: &ProfileData) -> Result<(), OptimizationError> {
        for branch_ref in &profile.branch_stats {
            let branch_id = *branch_ref.key();
            let branch_stats = branch_ref.value();
            let taken = branch_stats.taken_count.load(std::sync::atomic::Ordering::Relaxed);
            let not_taken = branch_stats.not_taken_count.load(std::sync::atomic::Ordering::Relaxed);
            let total = taken + not_taken;
            
            if total > 0 {
                let taken_ratio = taken as f64 / total as f64;
                
                // Apply optimization for highly biased branches (>90%)
                if taken_ratio > 0.9 || taken_ratio < 0.1 {
                    // TODO: Implement branch prediction hints in Cranelift IR
                    // This could involve code layout optimization or branch hints
                }
            }
        }
        Ok(())
    }
    
    /// Apply constant propagation with runtime values
    pub fn apply_constant_propagation(&self, func: &mut CraneliftFunction, profile: &ProfileData) -> Result<(), OptimizationError> {
        // Use type feedback to identify frequently used constant values
        for type_ref in &profile.type_feedback {
            let operation_id = *type_ref.key();
            let type_feedback = type_ref.value();
            if let Some(hot_type) = type_feedback.hot_type() {
                // Check if this operation consistently uses the same value
                let total_observations: u64 = type_feedback.type_frequencies.iter().map(|e| *e.value()).sum();
                if let Some(hot_type_count) = type_feedback.type_frequencies.get(&hot_type) {
                    let consistency_ratio = *hot_type_count.value() as f64 / total_observations as f64;
                    
                    if consistency_ratio > 0.95 { // 95% consistency
                        // TODO: Implement constant propagation in Cranelift IR
                        // This involves:
                        // 1. Identifying the operation by operation_id
                        // 2. Replacing variable loads with constant values
                        // 3. Propagating constants through the IR
                        // 4. Eliminating dead code that becomes unreachable
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Get optimization pass statistics
    pub fn get_stats(&self) -> OptimizationPassStats {
        OptimizationPassStats {
            loops_unrolled: 0, // TODO: Track actual statistics
            dead_code_eliminated: 0,
            branches_optimized: 0,
            constants_propagated: 0,
        }
    }
}

/// Statistics for optimization passes
#[derive(Debug, Default)]
pub struct OptimizationPassStats {
    pub loops_unrolled: u64,
    pub dead_code_eliminated: u64,
    pub branches_optimized: u64,
    pub constants_propagated: u64,
}

/// Profile-guided inliner for hot call sites
pub struct ProfileGuidedInliner {
    /// Inlining statistics
    stats: InliningStats,
}

impl ProfileGuidedInliner {
    pub fn new() -> Self {
        Self {
            stats: InliningStats::new(),
        }
    }
    
    /// Inline hot calls based on call frequency and target function size
    pub fn inline_hot_calls(
        &mut self,
        func: &mut CraneliftFunction,
        hot_call_sites: &[HotCallSite],
        config: &InliningConfig,
    ) -> Result<(), OptimizationError> {
        let mut total_inlined_size = 0;
        let original_size = self.estimate_function_size(func);
        let size_budget = (original_size as f64 * config.size_budget_multiplier) as usize;
        
        for call_site in hot_call_sites {
            // Check call frequency threshold
            if call_site.call_count < config.min_call_frequency {
                self.stats.rejected_frequency += 1;
                continue;
            }
            
            // Estimate target function size
            let target_size = self.estimate_target_function_size(&call_site.target_function)?;
            
            // Check size constraints
            if target_size > config.max_inline_size {
                self.stats.rejected_size += 1;
                continue;
            }
            
            if total_inlined_size + target_size > size_budget {
                self.stats.rejected_size += 1;
                break; // Size budget exceeded
            }
            
            // Check inlining depth to prevent excessive nesting
            if self.get_current_inline_depth(func) >= config.max_inline_depth {
                continue;
            }
            
            // Perform inlining
            self.inline_call_site(func, call_site)?;
            total_inlined_size += target_size;
            self.stats.inlined_calls += 1;
        }
        
        Ok(())
    }
    
    /// Estimate function size in IR instructions
    fn estimate_function_size(&self, func: &CraneliftFunction) -> usize {
        func.dfg.num_insts()
    }
    
    /// Estimate target function size
    fn estimate_target_function_size(&self, target_function: &str) -> Result<usize, OptimizationError> {
        // TODO: Look up actual function size from compilation cache or analyze source
        // For now, use a heuristic based on function name length and complexity
        let base_size = target_function.len() * 2; // Rough estimate
        Ok(base_size.min(200).max(10)) // Clamp between 10-200 instructions
    }
    
    /// Get current inlining depth in the function
    fn get_current_inline_depth(&self, func: &CraneliftFunction) -> u32 {
        // TODO: Track inlining depth through metadata or analysis
        // For now, return a conservative estimate
        0
    }
    
    /// Inline a specific call site
    fn inline_call_site(&mut self, func: &mut CraneliftFunction, call_site: &HotCallSite) -> Result<(), OptimizationError> {
        // TODO: Implement actual inlining in Cranelift IR
        // This involves:
        // 1. Finding the call instruction by call_site_id
        // 2. Loading the target function's IR
        // 3. Replacing the call with the target function's body
        // 4. Handling parameter passing and return values
        // 5. Updating control flow and phi nodes
        // 6. Renaming variables to avoid conflicts
        
        // For now, just mark as inlined for testing
        Ok(())
    }
    
    /// Get inlining statistics
    pub fn get_stats(&self) -> &InliningStats {
        &self.stats
    }
    
    /// Reset inlining statistics
    pub fn reset_stats(&mut self) {
        self.stats = InliningStats::new();
    }
}

/// Speculative optimizer for type-based optimizations with guards
pub struct SpeculativeOptimizer {
    /// Speculation statistics
    stats: SpeculationStats,
}

impl SpeculativeOptimizer {
    pub fn new() -> Self {
        Self {
            stats: SpeculationStats::new(),
        }
    }
    
    /// Apply speculative optimizations with guards
    pub fn apply_speculative_opts(
        &mut self,
        func: &mut CraneliftFunction,
        profile: &ProfileData,
    ) -> Result<(), OptimizationError> {
        // Apply type guards for monomorphic call sites
        self.apply_type_guards(func, profile)?;
        
        // Apply branch prediction hints
        self.apply_branch_hints(func, profile)?;
        
        // Generate deoptimization trampolines for guard failures
        self.generate_deoptimization_trampolines(func)?;
        
        Ok(())
    }
    
    /// Apply type guards for monomorphic call sites (>80% one type)
    fn apply_type_guards(&mut self, func: &mut CraneliftFunction, profile: &ProfileData) -> Result<(), OptimizationError> {
        for type_ref in &profile.type_feedback {
            let operation_id = *type_ref.key();
            let type_feedback = type_ref.value();
            if type_feedback.is_monomorphic() {
                if let Some(hot_type) = type_feedback.hot_type() {
                    // Calculate type frequency
                    let total_observations: u64 = type_feedback.type_frequencies.iter().map(|e| *e.value()).sum();
                    if let Some(hot_type_count) = type_feedback.type_frequencies.get(&hot_type) {
                        let hot_type_ratio = *hot_type_count.value() as f64 / total_observations as f64;
                        
                        if hot_type_ratio >= 0.8 {
                            // Generate type guard and speculative optimization
                            self.generate_type_guard(func, operation_id, &hot_type)?;
                            self.stats.type_guards_generated += 1;
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Generate type guard for speculative optimization
    fn generate_type_guard(&mut self, func: &mut CraneliftFunction, operation_id: u32, expected_type: &str) -> Result<(), OptimizationError> {
        // TODO: Implement type guard generation in Cranelift IR
        // This involves:
        // 1. Inserting type check instruction at the operation site
        // 2. Generating fast path for expected type (optimized code)
        // 3. Generating slow path for unexpected types (deoptimization)
        // 4. Linking the guard to the deoptimization trampoline
        
        // For now, just record that we would generate a guard
        Ok(())
    }
    
    /// Apply branch prediction hints based on profile data
    fn apply_branch_hints(&mut self, func: &mut CraneliftFunction, profile: &ProfileData) -> Result<(), OptimizationError> {
        for branch_ref in &profile.branch_stats {
            let branch_id = *branch_ref.key();
            let branch_stats = branch_ref.value();
            let taken = branch_stats.taken_count.load(std::sync::atomic::Ordering::Relaxed);
            let not_taken = branch_stats.not_taken_count.load(std::sync::atomic::Ordering::Relaxed);
            let total = taken + not_taken;
            
            if total > 0 {
                let taken_ratio = taken as f64 / total as f64;
                
                // Apply hints for highly biased branches (>90% or <10%)
                if taken_ratio > 0.9 {
                    self.generate_branch_hint(func, branch_id, true)?;
                    self.stats.branch_hints_applied += 1;
                } else if taken_ratio < 0.1 {
                    self.generate_branch_hint(func, branch_id, false)?;
                    self.stats.branch_hints_applied += 1;
                }
            }
        }
        Ok(())
    }
    
    /// Generate branch prediction hint
    fn generate_branch_hint(&mut self, func: &mut CraneliftFunction, branch_id: u32, likely_taken: bool) -> Result<(), OptimizationError> {
        // TODO: Implement branch hint generation in Cranelift IR
        // This could involve:
        // 1. Adding branch probability metadata
        // 2. Reordering basic blocks for better cache locality
        // 3. Using target-specific branch hint instructions
        Ok(())
    }
    
    /// Generate deoptimization trampolines for guard failures
    fn generate_deoptimization_trampolines(&mut self, func: &mut CraneliftFunction) -> Result<(), OptimizationError> {
        // TODO: Implement deoptimization trampoline generation
        // This involves:
        // 1. Creating a separate basic block for deoptimization
        // 2. Saving the current execution state
        // 3. Calling back to the interpreter or lower tier
        // 4. Handling state reconstruction
        Ok(())
    }
    
    /// Get speculation statistics
    pub fn get_stats(&self) -> &SpeculationStats {
        &self.stats
    }
    
    /// Reset speculation statistics
    pub fn reset_stats(&mut self) {
        self.stats = SpeculationStats::new();
    }
}

/// Configuration for inlining decisions
#[derive(Debug, Clone)]
pub struct InliningConfig {
    /// Maximum function size for inlining (IR instructions)
    pub max_inline_size: usize,
    
    /// Minimum call frequency for inlining
    pub min_call_frequency: u64,
    
    /// Maximum inline depth
    pub max_inline_depth: u32,
    
    /// Code size budget multiplier
    pub size_budget_multiplier: f64,
}

impl Default for InliningConfig {
    fn default() -> Self {
        Self {
            max_inline_size: 50,
            min_call_frequency: 10,
            max_inline_depth: 3,
            size_budget_multiplier: 2.0,
        }
    }
}

/// Configuration for optimization passes
#[derive(Debug, Clone)]
pub struct OptimizationPassConfig {
    /// Minimum execution count to consider loop hot
    pub hot_loop_threshold: u64,
    
    /// Minimum branch bias for prediction optimization
    pub branch_bias_threshold: f64,
}

impl Default for OptimizationPassConfig {
    fn default() -> Self {
        Self {
            hot_loop_threshold: 100,
            branch_bias_threshold: 0.9,
        }
    }
}

/// Compiled code with metadata
#[derive(Debug)]
pub struct CompiledCode {
    /// Function identifier
    pub function_id: String,
    
    /// Compilation tier
    pub tier: CompilationTier,
    
    /// Version number
    pub version: u64,
    
    /// Pointer to executable code
    pub code_ptr: *const u8,
    
    /// Code size in bytes
    pub code_size: usize,
    
    /// Cranelift function ID
    pub cranelift_func_id: FuncId,
    
    /// Deoptimization metadata (for speculative optimizations)
    pub deopt_info: Option<DeoptimizationInfo>,
    
    /// Compilation timestamp
    pub compiled_at: Instant,
    
    /// Compilation time
    pub compilation_time: Duration,
    
    /// Profile data snapshot used for compilation (simplified for cloning)
    pub profile_snapshot: ProfileDataSnapshot,
}

/// Deoptimization information for speculative optimizations
#[derive(Debug)]
pub struct DeoptimizationInfo {
    /// Deoptimization points in the code
    pub deopt_points: Vec<DeoptPoint>,
    
    /// Fallback tier for deoptimization
    pub fallback_tier: CompilationTier,
    
    /// Fallback code pointer
    pub fallback_code: *const u8,
    
    /// Deoptimization frequency counter
    pub deopt_count: std::sync::atomic::AtomicU64,
    
    /// Last deoptimization timestamp
    pub last_deopt: std::sync::atomic::AtomicU64,
}

impl DeoptimizationInfo {
    /// Create new deoptimization info
    pub fn new(fallback_tier: CompilationTier, fallback_code: *const u8) -> Self {
        Self {
            deopt_points: Vec::new(),
            fallback_tier,
            fallback_code,
            deopt_count: std::sync::atomic::AtomicU64::new(0),
            last_deopt: std::sync::atomic::AtomicU64::new(0),
        }
    }
    
    /// Add a deoptimization point
    pub fn add_deopt_point(&mut self, point: DeoptPoint) {
        self.deopt_points.push(point);
    }
    
    /// Record a deoptimization event
    pub fn record_deoptimization(&self) {
        self.deopt_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.last_deopt.store(now, std::sync::atomic::Ordering::Relaxed);
    }
    
    /// Get deoptimization frequency
    pub fn get_deopt_count(&self) -> u64 {
        self.deopt_count.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    /// Check if function should be prevented from re-speculation
    pub fn should_prevent_respeculation(&self) -> bool {
        // Prevent re-speculation if deoptimized more than 5 times
        self.get_deopt_count() > 5
    }
}

/// Deoptimization point in compiled code
#[derive(Debug, Clone)]
pub struct DeoptPoint {
    /// Offset in compiled code
    pub code_offset: usize,
    
    /// Speculation type
    pub speculation_type: SpeculationType,
    
    /// Guard condition
    pub guard: Guard,
    
    /// State reconstruction information
    pub state_info: StateReconstructionInfo,
}

impl DeoptPoint {
    /// Create a new deoptimization point
    pub fn new(
        code_offset: usize,
        speculation_type: SpeculationType,
        guard: Guard,
        state_info: StateReconstructionInfo,
    ) -> Self {
        Self {
            code_offset,
            speculation_type,
            guard,
            state_info,
        }
    }
    
    /// Check if this deoptimization point matches a guard failure
    pub fn matches_guard_failure(&self, failed_guard: &Guard) -> bool {
        match (&self.guard, failed_guard) {
            (Guard::TypeCheck { operation_id: a, .. }, Guard::TypeCheck { operation_id: b, .. }) => a == b,
            (Guard::BranchCondition { branch_id: a }, Guard::BranchCondition { branch_id: b }) => a == b,
            _ => false, // Different guard types don't match
        }
    }
}

/// State reconstruction information for deoptimization
#[derive(Debug, Clone)]
pub struct StateReconstructionInfo {
    /// Variable mappings from compiled to interpreter state
    pub variable_mappings: Vec<VariableMapping>,
    
    /// Stack frame information
    pub stack_frame_info: StackFrameInfo,
    
    /// Program counter mapping
    pub pc_mapping: ProgramCounterMapping,
}

/// Variable mapping for state reconstruction
#[derive(Debug, Clone)]
pub struct VariableMapping {
    /// Variable name in interpreter
    pub interpreter_name: String,
    
    /// Register or stack location in compiled code
    pub compiled_location: CompiledLocation,
    
    /// Variable type
    pub variable_type: String,
}

/// Location of a variable in compiled code
#[derive(Debug, Clone)]
pub enum CompiledLocation {
    /// CPU register
    Register { reg_id: u8 },
    
    /// Stack offset
    Stack { offset: i32 },
    
    /// Constant value
    Constant { value: i64 },
}

/// Stack frame information
#[derive(Debug, Clone)]
pub struct StackFrameInfo {
    /// Frame size in bytes
    pub frame_size: usize,
    
    /// Return address offset
    pub return_address_offset: usize,
    
    /// Saved registers
    pub saved_registers: Vec<u8>,
}

/// Program counter mapping
#[derive(Debug, Clone)]
pub struct ProgramCounterMapping {
    /// Compiled code offset
    pub compiled_offset: usize,
    
    /// Corresponding interpreter instruction index
    pub interpreter_pc: usize,
}

/// Type of speculation
#[derive(Debug, Clone)]
pub enum SpeculationType {
    /// Type-based speculation
    TypeSpeculation { expected_type: String },
    
    /// Branch prediction speculation
    BranchPrediction { expected_taken: bool },
    
    /// Inlining speculation
    InliningSpeculation { target_function: String },
}

/// Guard condition for speculation
#[derive(Debug, Clone)]
pub enum Guard {
    /// Type check guard
    TypeCheck { operation_id: u32, expected_type: String },
    
    /// Branch condition guard
    BranchCondition { branch_id: u32 },
}

/// Deoptimization manager for handling guard failures and state reconstruction
pub struct DeoptimizationManager {
    /// Deoptimization statistics
    stats: DeoptimizationStats,
}

impl DeoptimizationManager {
    pub fn new() -> Self {
        Self {
            stats: DeoptimizationStats::new(),
        }
    }
    
    /// Handle a guard failure and perform deoptimization
    pub fn handle_guard_failure(
        &mut self,
        deopt_info: &DeoptimizationInfo,
        failed_guard: &Guard,
        execution_context: &ExecutionContext,
    ) -> Result<InterpreterState, OptimizationError> {
        // Find the matching deoptimization point
        let deopt_point = deopt_info.deopt_points
            .iter()
            .find(|point| point.matches_guard_failure(failed_guard))
            .ok_or_else(|| OptimizationError::SpeculationFailed {
                reason: "No matching deoptimization point found".to_string(),
            })?;
        
        // Record deoptimization event
        deopt_info.record_deoptimization();
        self.stats.total_deoptimizations += 1;
        
        // Reconstruct interpreter state
        let interpreter_state = self.reconstruct_interpreter_state(deopt_point, execution_context)?;
        
        // Update statistics based on speculation type
        match &deopt_point.speculation_type {
            SpeculationType::TypeSpeculation { .. } => {
                self.stats.type_speculation_failures += 1;
            }
            SpeculationType::BranchPrediction { .. } => {
                self.stats.branch_prediction_failures += 1;
            }
            SpeculationType::InliningSpeculation { .. } => {
                self.stats.inlining_speculation_failures += 1;
            }
        }
        
        Ok(interpreter_state)
    }
    
    /// Reconstruct interpreter state from compiled frame
    fn reconstruct_interpreter_state(
        &self,
        deopt_point: &DeoptPoint,
        execution_context: &ExecutionContext,
    ) -> Result<InterpreterState, OptimizationError> {
        let mut interpreter_state = InterpreterState::new();
        
        // Reconstruct variables from compiled locations
        for mapping in &deopt_point.state_info.variable_mappings {
            let value = self.extract_value_from_compiled_location(
                &mapping.compiled_location,
                execution_context,
            )?;
            
            interpreter_state.set_variable(mapping.interpreter_name.clone(), value);
        }
        
        // Set program counter
        interpreter_state.set_pc(deopt_point.state_info.pc_mapping.interpreter_pc);
        
        Ok(interpreter_state)
    }
    
    /// Extract value from compiled code location
    fn extract_value_from_compiled_location(
        &self,
        location: &CompiledLocation,
        execution_context: &ExecutionContext,
    ) -> Result<InterpreterValue, OptimizationError> {
        match location {
            CompiledLocation::Register { reg_id } => {
                // TODO: Extract value from CPU register
                // This requires platform-specific code to read register values
                Ok(InterpreterValue::Integer(0)) // Placeholder
            }
            CompiledLocation::Stack { offset } => {
                // TODO: Extract value from stack location
                // This requires reading from the current stack frame
                Ok(InterpreterValue::Integer(0)) // Placeholder
            }
            CompiledLocation::Constant { value } => {
                Ok(InterpreterValue::Integer(*value))
            }
        }
    }
    
    /// Check if function should be prevented from re-speculation
    pub fn should_prevent_respeculation(&self, deopt_info: &DeoptimizationInfo) -> bool {
        deopt_info.should_prevent_respeculation()
    }
    
    /// Get deoptimization statistics
    pub fn get_stats(&self) -> &DeoptimizationStats {
        &self.stats
    }
    
    /// Reset deoptimization statistics
    pub fn reset_stats(&mut self) {
        self.stats = DeoptimizationStats::new();
    }
}

/// Execution context for deoptimization
#[derive(Debug)]
pub struct ExecutionContext {
    /// Current stack pointer
    pub stack_pointer: *const u8,
    
    /// Current frame pointer
    pub frame_pointer: *const u8,
    
    /// CPU register values (platform-specific)
    pub registers: Vec<u64>,
}

/// Interpreter state for fallback execution
#[derive(Debug)]
pub struct InterpreterState {
    /// Variable values
    variables: std::collections::HashMap<String, InterpreterValue>,
    
    /// Program counter
    pc: usize,
}

impl InterpreterState {
    pub fn new() -> Self {
        Self {
            variables: std::collections::HashMap::new(),
            pc: 0,
        }
    }
    
    pub fn set_variable(&mut self, name: String, value: InterpreterValue) {
        self.variables.insert(name, value);
    }
    
    pub fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }
    
    pub fn get_pc(&self) -> usize {
        self.pc
    }
}

/// Interpreter value types
#[derive(Debug, Clone)]
pub enum InterpreterValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

/// Deoptimization statistics
#[derive(Debug, Default)]
pub struct DeoptimizationStats {
    pub total_deoptimizations: u64,
    pub type_speculation_failures: u64,
    pub branch_prediction_failures: u64,
    pub inlining_speculation_failures: u64,
}

impl DeoptimizationStats {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Generated machine code
struct GeneratedCode {
    code_ptr: *const u8,
    code_size: usize,
    cranelift_func_id: FuncId,
    deopt_info: Option<DeoptimizationInfo>,
}

/// Inlining statistics
#[derive(Debug, Default)]
struct InliningStats {
    inlined_calls: u64,
    rejected_size: u64,
    rejected_frequency: u64,
}

impl InliningStats {
    fn new() -> Self {
        Self::default()
    }
}

/// Speculation statistics
#[derive(Debug, Default)]
struct SpeculationStats {
    type_guards_generated: u64,
    branch_hints_applied: u64,
    deoptimizations: u64,
}

impl SpeculationStats {
    fn new() -> Self {
        Self::default()
    }
}

/// Optimization engine errors
#[derive(Debug, thiserror::Error)]
pub enum OptimizationError {
    #[error("Compilation time budget exceeded for tier {tier:?}: {actual:?} > {budget:?}")]
    CompilationTimeBudgetExceeded {
        tier: CompilationTier,
        actual: Duration,
        budget: Duration,
    },
    
    #[error("Cranelift module error: {0}")]
    CraneliftModule(#[from] cranelift_module::ModuleError),
    
    #[error("Cranelift codegen error: {0}")]
    CraneliftCodegen(#[from] CodegenError),
    
    #[error("Function not found: {function_name}")]
    FunctionNotFound { function_name: String },
    
    #[error("Inlining failed: {reason}")]
    InliningFailed { reason: String },
    
    #[error("Speculation failed: {reason}")]
    SpeculationFailed { reason: String },
}

// Ensure CompiledCode is Send + Sync for thread safety
unsafe impl Send for CompiledCode {}
unsafe impl Sync for CompiledCode {}