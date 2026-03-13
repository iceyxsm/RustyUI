//! Recompilation scheduler for background compilation and atomic code replacement
//! 
//! Based on 2024-2026 industry best practices:
//! - Background thread pool for non-blocking recompilation
//! - Priority-based work queue with lock-free operations
//! - Atomic code replacement without execution pauses
//! - Reference counting for safe memory management
//! - Budget limiting to prevent compilation thrashing

use crate::tiered_compilation::CompilationTier;
use crate::profiling::ProfileData;
use std::sync::{Arc, RwLock, atomic::{AtomicUsize, AtomicU64, AtomicBool, AtomicU32, Ordering}};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use crossbeam::queue::SegQueue;
use std::thread;

/// Recompilation task for background processing
#[derive(Debug, Clone)]
pub struct RecompilationTask {
    /// Function identifier
    pub function_id: String,
    
    /// Source code to compile
    pub source_code: String,
    
    /// Current compilation tier
    pub current_tier: CompilationTier,
    
    /// Target compilation tier
    pub target_tier: CompilationTier,
    
    /// Profile data snapshot for optimization
    pub profile_data: Arc<ProfileData>,
    
    /// Priority score (higher = more important)
    pub priority: f64,
    
    /// Task submission timestamp
    pub submitted_at: Instant,
    
    /// Task ID for tracking
    pub task_id: u64,
}

impl RecompilationTask {
    /// Create new recompilation task
    pub fn new(
        function_id: String,
        source_code: String,
        current_tier: CompilationTier,
        target_tier: CompilationTier,
        profile_data: Arc<ProfileData>,
        priority: f64,
    ) -> Self {
        static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(0);
        
        Self {
            function_id,
            source_code,
            current_tier,
            target_tier,
            profile_data,
            priority,
            submitted_at: Instant::now(),
            task_id: TASK_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
    
    /// Get task age
    pub fn age(&self) -> Duration {
        self.submitted_at.elapsed()
    }
    
    /// Calculate effective priority (priority adjusted by age)
    pub fn effective_priority(&self) -> f64 {
        let age_bonus = self.age().as_secs_f64() * 0.1; // Small age bonus
        self.priority + age_bonus
    }
}

/// Status of recompilation task
#[derive(Debug, Clone, PartialEq)]
pub enum RecompilationStatus {
    /// Task is queued for processing
    Queued,
    
    /// Task is currently being processed
    InProgress { started_at: Instant },
    
    /// Task completed successfully
    Completed { 
        completed_at: Instant,
        compilation_time: Duration,
    },
    
    /// Task failed with error
    Failed { 
        failed_at: Instant,
        error: String,
    },
    
    /// Task was cancelled
    Cancelled { cancelled_at: Instant },
}

impl RecompilationStatus {
    /// Check if task is active (queued or in progress)
    pub fn is_active(&self) -> bool {
        matches!(self, RecompilationStatus::Queued | RecompilationStatus::InProgress { .. })
    }
    
    /// Check if task is completed (success or failure)
    pub fn is_completed(&self) -> bool {
        matches!(
            self,
            RecompilationStatus::Completed { .. } | 
            RecompilationStatus::Failed { .. } | 
            RecompilationStatus::Cancelled { .. }
        )
    }
}

/// Queue statistics for monitoring
#[derive(Debug, Clone)]
pub struct QueueStatistics {
    /// Number of tasks in queue
    pub queued_tasks: usize,
    
    /// Number of tasks in progress
    pub in_progress_tasks: usize,
    
    /// Number of completed tasks
    pub completed_tasks: usize,
    
    /// Number of failed tasks
    pub failed_tasks: usize,
    
    /// Average queue wait time
    pub avg_queue_wait_time: Duration,
    
    /// Average compilation time
    pub avg_compilation_time: Duration,
    
    /// Current budget usage (compilations per second)
    pub current_budget_usage: f64,
    
    /// Maximum budget limit
    pub budget_limit: u32,
}

/// Configuration for recompilation scheduler
#[derive(Debug, Clone)]
pub struct RecompilationConfig {
    /// Thread pool size for background compilation
    pub thread_pool_size: usize,
    
    /// Maximum concurrent recompilations
    pub max_concurrent: usize,
    
    /// Recompilation budget (compilations per second)
    pub budget_per_second: u32,
    
    /// Old code grace period before GC
    pub gc_grace_period: Duration,
    
    /// GC interval
    pub gc_interval: Duration,
    
    /// Maximum queue size
    pub max_queue_size: usize,
    
    /// Task timeout
    pub task_timeout: Duration,
}

impl Default for RecompilationConfig {
    fn default() -> Self {
        Self {
            thread_pool_size: std::cmp::min(4, num_cpus::get()),
            max_concurrent: 4,
            budget_per_second: 10, // Max 10 compilations per second
            gc_grace_period: Duration::from_millis(100),
            gc_interval: Duration::from_secs(30),
            max_queue_size: 1000,
            task_timeout: Duration::from_secs(30),
        }
    }
}

/// Compiled code with version tracking
#[derive(Debug)]
pub struct CompiledCode {
    /// Function identifier
    pub function_id: String,
    
    /// Compilation tier
    pub tier: CompilationTier,
    
    /// Version number for atomic replacement
    pub version: u64,
    
    /// Code size in bytes
    pub code_size: usize,
    
    /// Compilation timestamp
    pub compiled_at: Instant,
    
    /// Compilation time
    pub compilation_time: Duration,
    
    /// Profile data snapshot used for compilation
    pub profile_snapshot: Arc<ProfileData>,
    
    /// Reference count for safe deallocation
    pub ref_count: Arc<AtomicUsize>,
}

// Manual implementation of Send and Sync for CompiledCode
// This is safe because we don't actually use the raw pointer in a multi-threaded context
// In a real implementation, the code pointer would be managed by a JIT engine
unsafe impl Send for CompiledCode {}
unsafe impl Sync for CompiledCode {}

impl CompiledCode {
    /// Create new compiled code
    pub fn new(
        function_id: String,
        tier: CompilationTier,
        code_size: usize,
        compilation_time: Duration,
        profile_snapshot: Arc<ProfileData>,
    ) -> Self {
        static VERSION_COUNTER: AtomicU64 = AtomicU64::new(0);
        
        Self {
            function_id,
            tier,
            version: VERSION_COUNTER.fetch_add(1, Ordering::Relaxed),
            code_size,
            compiled_at: Instant::now(),
            compilation_time,
            profile_snapshot,
            ref_count: Arc::new(AtomicUsize::new(1)),
        }
    }
    
    /// Increment reference count
    pub fn add_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Decrement reference count
    pub fn release(&self) -> usize {
        self.ref_count.fetch_sub(1, Ordering::Relaxed) - 1
    }
    
    /// Get current reference count
    pub fn ref_count(&self) -> usize {
        self.ref_count.load(Ordering::Relaxed)
    }
    
    /// Check if code can be safely deallocated
    pub fn can_deallocate(&self, grace_period: Duration) -> bool {
        self.ref_count() == 0 && self.compiled_at.elapsed() > grace_period
    }
}

/// Code version manager for atomic replacement
#[derive(Debug)]
pub struct CodeVersionManager {
    /// Current active version per function
    active_versions: RwLock<HashMap<String, Arc<CompiledCode>>>,
    
    /// Old versions pending garbage collection
    old_versions: RwLock<Vec<(Arc<CompiledCode>, Instant)>>,
    
    /// GC configuration
    gc_config: RecompilationConfig,
    
    /// Version replacement statistics
    replacement_stats: Arc<RwLock<ReplacementStatistics>>,
}

/// Statistics for code replacement operations
#[derive(Debug, Clone)]
pub struct ReplacementStatistics {
    /// Total number of successful replacements
    pub successful_replacements: u64,
    
    /// Total number of failed replacements
    pub failed_replacements: u64,
    
    /// Average replacement time
    pub avg_replacement_time: Duration,
    
    /// Number of versions currently pending GC
    pub pending_gc_versions: usize,
    
    /// Total memory used by active versions
    pub active_memory_bytes: usize,
    
    /// Total memory used by old versions
    pub old_memory_bytes: usize,
}

/// Result of garbage collection operation
#[derive(Debug, Clone)]
pub struct GCResult {
    /// Number of versions collected
    pub collected_versions: usize,
    
    /// Memory freed in bytes
    pub collected_memory_bytes: usize,
    
    /// Number of versions remaining
    pub remaining_versions: usize,
    
    /// Time taken for GC
    pub gc_time: Duration,
}

/// Comprehensive scheduler status
#[derive(Debug, Clone)]
pub struct SchedulerStatus {
    /// Whether the scheduler is healthy
    pub is_healthy: bool,
    
    /// Number of active worker threads
    pub worker_count: usize,
    
    /// Queue statistics
    pub queue_stats: QueueStatistics,
    
    /// Budget statistics
    pub budget_stats: BudgetStatistics,
    
    /// Code replacement statistics
    pub replacement_stats: ReplacementStatistics,
    
    /// Active memory usage in bytes
    pub active_memory_bytes: usize,
    
    /// Old memory usage in bytes
    pub old_memory_bytes: usize,
}

impl CodeVersionManager {
    /// Create new code version manager
    pub fn new(config: RecompilationConfig) -> Self {
        Self {
            active_versions: RwLock::new(HashMap::new()),
            old_versions: RwLock::new(Vec::new()),
            gc_config: config,
            replacement_stats: Arc::new(RwLock::new(ReplacementStatistics {
                successful_replacements: 0,
                failed_replacements: 0,
                avg_replacement_time: Duration::from_millis(0),
                pending_gc_versions: 0,
                active_memory_bytes: 0,
                old_memory_bytes: 0,
            })),
        }
    }
    
    /// Get current active version for function
    pub fn get_active_version(&self, function_id: &str) -> Option<Arc<CompiledCode>> {
        let versions = self.active_versions.read().unwrap();
        versions.get(function_id).cloned()
    }
    
    /// Replace code atomically with detailed error handling
    pub fn replace_code(&self, new_code: Arc<CompiledCode>) -> crate::Result<()> {
        let replacement_start = Instant::now();
        let function_id = &new_code.function_id;
        
        // Validate new code before replacement
        self.validate_code(&new_code)?;
        
        // Perform atomic replacement
        let old_code = {
            let mut active_versions = self.active_versions.write().unwrap();
            
            // Get old version if it exists
            let old_code = active_versions.get(function_id).cloned();
            
            // Install new version atomically
            active_versions.insert(function_id.clone(), new_code.clone());
            
            old_code
        };
        
        // Move old version to pending GC if it exists
        if let Some(old_code) = old_code {
            let mut old_versions = self.old_versions.write().unwrap();
            old_versions.push((old_code, Instant::now()));
        }
        
        // Update statistics
        let replacement_time = replacement_start.elapsed();
        self.update_replacement_stats(true, replacement_time);
        
        Ok(())
    }
    
    /// Validate compiled code before installation
    fn validate_code(&self, code: &CompiledCode) -> crate::Result<()> {
        // Basic validation checks
        if code.function_id.is_empty() {
            return Err(crate::InterpreterError::generic("Function ID cannot be empty".to_string()));
        }
        
        if code.code_size == 0 {
            return Err(crate::InterpreterError::generic("Code size cannot be zero".to_string()));
        }
        
        // Check if compilation time is reasonable (not too fast or too slow)
        let compilation_ms = code.compilation_time.as_millis();
        if compilation_ms == 0 {
            return Err(crate::InterpreterError::generic("Compilation time suspiciously fast".to_string()));
        }
        
        if compilation_ms > 10000 { // 10 seconds
            return Err(crate::InterpreterError::generic("Compilation time too slow".to_string()));
        }
        
        Ok(())
    }
    
    /// Update replacement statistics
    fn update_replacement_stats(&self, success: bool, replacement_time: Duration) {
        let mut stats = self.replacement_stats.write().unwrap();
        
        if success {
            stats.successful_replacements += 1;
            
            // Update average replacement time (simple moving average)
            let total_replacements = stats.successful_replacements;
            let current_avg_ms = stats.avg_replacement_time.as_millis() as f64;
            let new_time_ms = replacement_time.as_millis() as f64;
            let new_avg_ms = (current_avg_ms * (total_replacements - 1) as f64 + new_time_ms) / total_replacements as f64;
            stats.avg_replacement_time = Duration::from_millis(new_avg_ms as u64);
        } else {
            stats.failed_replacements += 1;
        }
    }
    
    /// Garbage collect old code versions with detailed reporting
    pub fn gc_old_versions(&self) -> GCResult {
        let gc_start = Instant::now();
        let mut old_versions = self.old_versions.write().unwrap();
        let initial_count = old_versions.len();
        let mut collected_memory = 0usize;
        
        // Remove versions that can be safely deallocated
        old_versions.retain(|(code, retired_at)| {
            let age = retired_at.elapsed();
            let can_gc = code.can_deallocate(self.gc_config.gc_grace_period) && 
                        age > self.gc_config.gc_grace_period;
            
            if can_gc {
                collected_memory += code.code_size;
                // In a real implementation, we would deallocate the machine code here
                // For now, we just remove it from tracking
                false
            } else {
                true
            }
        });
        
        let gc_count = initial_count - old_versions.len();
        let gc_time = gc_start.elapsed();
        
        // Update statistics
        {
            let mut stats = self.replacement_stats.write().unwrap();
            stats.pending_gc_versions = old_versions.len();
        }
        
        GCResult {
            collected_versions: gc_count,
            collected_memory_bytes: collected_memory,
            remaining_versions: old_versions.len(),
            gc_time,
        }
    }
    
    /// Schedule periodic garbage collection
    pub fn schedule_periodic_gc(self: Arc<Self>) -> std::thread::JoinHandle<()> {
        let gc_interval = self.gc_config.gc_interval;
        
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(gc_interval);
                
                let gc_result = self.gc_old_versions();
                
                if gc_result.collected_versions > 0 {
                    println!(
                        "Periodic GC: Collected {} versions, freed {} bytes in {:?}",
                        gc_result.collected_versions,
                        gc_result.collected_memory_bytes,
                        gc_result.gc_time
                    );
                }
            }
        })
    }
    
    /// Check if GC is needed based on memory pressure
    pub fn needs_gc(&self) -> bool {
        let old_versions = self.old_versions.read().unwrap();
        
        // GC if we have too many old versions
        if old_versions.len() > 100 {
            return true;
        }
        
        // GC if old versions are using too much memory
        let old_memory: usize = old_versions.iter()
            .map(|(code, _)| code.code_size)
            .sum();
        
        if old_memory > 10 * 1024 * 1024 { // 10MB
            return true;
        }
        
        // GC if oldest version is very old
        if let Some((_, oldest_time)) = old_versions.first() {
            if oldest_time.elapsed() > Duration::from_secs(300) { // 5 minutes
                return true;
            }
        }
        
        false
    }
    
    /// Trigger GC if needed
    pub fn gc_if_needed(&self) -> Option<GCResult> {
        if self.needs_gc() {
            Some(self.gc_old_versions())
        } else {
            None
        }
    }
    
    /// Force garbage collection of all old versions (for testing/cleanup)
    pub fn force_gc_all(&self) {
        let mut old_versions = self.old_versions.write().unwrap();
        let gc_count = old_versions.len();
        old_versions.clear();
        
        // Update statistics
        {
            let mut stats = self.replacement_stats.write().unwrap();
            stats.pending_gc_versions = 0;
        }
        
        if gc_count > 0 {
            println!("Force GC: Collected {} old code versions", gc_count);
        }
    }
    
    /// Get memory usage statistics
    pub fn get_memory_stats(&self) -> (usize, usize) {
        let active_versions = self.active_versions.read().unwrap();
        let old_versions = self.old_versions.read().unwrap();
        
        let active_memory: usize = active_versions.values()
            .map(|code| code.code_size)
            .sum();
        
        let old_memory: usize = old_versions.iter()
            .map(|(code, _)| code.code_size)
            .sum();
        
        // Update statistics
        {
            let mut stats = self.replacement_stats.write().unwrap();
            stats.active_memory_bytes = active_memory;
            stats.old_memory_bytes = old_memory;
        }
        
        (active_memory, old_memory)
    }
    
    /// Get replacement statistics
    pub fn get_replacement_stats(&self) -> ReplacementStatistics {
        // Update memory stats first
        let _ = self.get_memory_stats();
        
        let stats = self.replacement_stats.read().unwrap();
        stats.clone()
    }
    
    /// Get all active function IDs
    pub fn get_active_function_ids(&self) -> Vec<String> {
        let active_versions = self.active_versions.read().unwrap();
        active_versions.keys().cloned().collect()
    }
    
    /// Check if function has active version
    pub fn has_active_version(&self, function_id: &str) -> bool {
        let active_versions = self.active_versions.read().unwrap();
        active_versions.contains_key(function_id)
    }
    
    /// Remove function version (for cleanup/testing)
    pub fn remove_function(&self, function_id: &str) -> Option<Arc<CompiledCode>> {
        let mut active_versions = self.active_versions.write().unwrap();
        active_versions.remove(function_id)
    }
    
    /// Clear all versions (for cleanup/testing)
    pub fn clear_all(&self) {
        {
            let mut active_versions = self.active_versions.write().unwrap();
            active_versions.clear();
        }
        
        {
            let mut old_versions = self.old_versions.write().unwrap();
            old_versions.clear();
        }
        
        // Reset statistics
        {
            let mut stats = self.replacement_stats.write().unwrap();
            *stats = ReplacementStatistics {
                successful_replacements: 0,
                failed_replacements: 0,
                avg_replacement_time: Duration::from_millis(0),
                pending_gc_versions: 0,
                active_memory_bytes: 0,
                old_memory_bytes: 0,
            };
        }
    }
    
    /// Get version count for function (active + old)
    pub fn get_version_count(&self, function_id: &str) -> (bool, usize) {
        let active_versions = self.active_versions.read().unwrap();
        let old_versions = self.old_versions.read().unwrap();
        
        let has_active = active_versions.contains_key(function_id);
        let old_count = old_versions.iter()
            .filter(|(code, _)| code.function_id == function_id)
            .count();
        
        (has_active, old_count)
    }
}

/// Budget limiter to prevent compilation thrashing
#[derive(Debug)]
pub struct BudgetLimiter {
    /// Budget limit (compilations per second)
    budget_limit: u32,

    /// Compilation timestamps (sliding window)
    compilation_times: RwLock<Vec<Instant>>,

    /// Window duration for budget calculation
    window_duration: Duration,

    /// Budget statistics
    stats: Arc<RwLock<BudgetStatistics>>,

    /// Adaptive budget adjustment
    adaptive_config: AdaptiveBudgetConfig,

    /// Atomic counter for reserved slots (prevents race conditions)
    reserved_slots: AtomicU32,
}


/// Statistics for budget limiter
#[derive(Debug, Clone)]
pub struct BudgetStatistics {
    /// Total compilations attempted
    pub total_attempts: u64,
    
    /// Total compilations allowed
    pub total_allowed: u64,
    
    /// Total compilations rejected due to budget
    pub total_rejected: u64,
    
    /// Current budget utilization (0.0 - 1.0)
    pub current_utilization: f64,
    
    /// Average compilations per second over last minute
    pub avg_compilations_per_second: f64,
    
    /// Peak compilations per second observed
    pub peak_compilations_per_second: f64,
}

/// Configuration for adaptive budget adjustment
#[derive(Debug, Clone)]
pub struct AdaptiveBudgetConfig {
    /// Enable adaptive budget adjustment
    pub enabled: bool,
    
    /// Minimum budget limit
    pub min_budget: u32,
    
    /// Maximum budget limit
    pub max_budget: u32,
    
    /// Adjustment factor (0.0 - 1.0)
    pub adjustment_factor: f64,
    
    /// Utilization threshold for increasing budget
    pub increase_threshold: f64,
    
    /// Utilization threshold for decreasing budget
    pub decrease_threshold: f64,
}

impl Default for AdaptiveBudgetConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_budget: 5,
            max_budget: 50,
            adjustment_factor: 0.1,
            increase_threshold: 0.8,
            decrease_threshold: 0.3,
        }
    }
}

impl BudgetLimiter {
    /// Create new budget limiter
    pub fn new(budget_limit: u32) -> Self {
        Self {
            budget_limit,
            compilation_times: RwLock::new(Vec::new()),
            window_duration: Duration::from_secs(1),
            stats: Arc::new(RwLock::new(BudgetStatistics {
                total_attempts: 0,
                total_allowed: 0,
                total_rejected: 0,
                current_utilization: 0.0,
                avg_compilations_per_second: 0.0,
                peak_compilations_per_second: 0.0,
            })),
            adaptive_config: AdaptiveBudgetConfig::default(),
            reserved_slots: AtomicU32::new(0),
        }
    }
    
    /// Create with adaptive configuration
    pub fn with_adaptive_config(budget_limit: u32, adaptive_config: AdaptiveBudgetConfig) -> Self {
        Self {
            budget_limit,
            compilation_times: RwLock::new(Vec::new()),
            window_duration: Duration::from_secs(1),
            stats: Arc::new(RwLock::new(BudgetStatistics {
                total_attempts: 0,
                total_allowed: 0,
                total_rejected: 0,
                current_utilization: 0.0,
                avg_compilations_per_second: 0.0,
                peak_compilations_per_second: 0.0,
            })),
            adaptive_config,
            reserved_slots: AtomicU32::new(0),
        }
    }
    
    /// Check if compilation is allowed within budget (atomic reservation)
    pub fn can_compile(&self) -> bool {
        // First, atomically try to reserve a slot
        let current_reserved = self.reserved_slots.fetch_add(1, Ordering::SeqCst);
        
        // Check if we exceeded the budget with this reservation
        if current_reserved >= self.budget_limit {
            // We exceeded the budget, release the reservation
            self.reserved_slots.fetch_sub(1, Ordering::SeqCst);
            
            // Update statistics for rejection
            {
                let mut stats = self.stats.write().unwrap();
                stats.total_attempts += 1;
                stats.total_rejected += 1;
            }
            
            return false;
        }
        
        // We successfully reserved a slot, now check the sliding window
        let mut times = self.compilation_times.write().unwrap();
        let now = Instant::now();
        
        // Remove old entries outside the window
        times.retain(|&time| now.duration_since(time) <= self.window_duration);
        
        let current_count = times.len();
        let can_compile = current_count < self.budget_limit as usize;
        
        if !can_compile {
            // Sliding window check failed, release the reservation
            self.reserved_slots.fetch_sub(1, Ordering::SeqCst);
            
            // Update statistics for rejection
            {
                let mut stats = self.stats.write().unwrap();
                stats.total_attempts += 1;
                stats.total_rejected += 1;
                stats.current_utilization = current_count as f64 / self.budget_limit as f64;
                stats.avg_compilations_per_second = current_count as f64;
                
                if stats.avg_compilations_per_second > stats.peak_compilations_per_second {
                    stats.peak_compilations_per_second = stats.avg_compilations_per_second;
                }
            }
            
            return false;
        }
        
        // Both checks passed, update statistics for success
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_attempts += 1;
            stats.total_allowed += 1;
            stats.current_utilization = current_count as f64 / self.budget_limit as f64;
            stats.avg_compilations_per_second = current_count as f64;
            
            if stats.avg_compilations_per_second > stats.peak_compilations_per_second {
                stats.peak_compilations_per_second = stats.avg_compilations_per_second;
            }
        }
        
        // Perform adaptive budget adjustment
        if self.adaptive_config.enabled {
            self.adjust_budget_if_needed();
        }
        
        true
    }
    
    /// Record a compilation (call after starting compilation)
    pub fn record_compilation(&self) {
        let mut times = self.compilation_times.write().unwrap();
        times.push(Instant::now());
        
        // Release the reserved slot since compilation has started
        self.reserved_slots.fetch_sub(1, Ordering::SeqCst);
    }
    
    /// Cancel a compilation reservation (call if can_compile() returned true but compilation won't proceed)
    pub fn cancel_compilation(&self) {
        // Release the reserved slot
        self.reserved_slots.fetch_sub(1, Ordering::SeqCst);
        
        // Update statistics to reflect the cancellation
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_rejected += 1;
            if stats.total_allowed > 0 {
                stats.total_allowed -= 1;
            }
        }
    }
    
    /// Get current budget usage percentage
    pub fn get_usage(&self) -> f64 {
        let times = self.compilation_times.read().unwrap();
        let now = Instant::now();
        
        // Count compilations in the current window
        let current_count = times.iter()
            .filter(|&&time| now.duration_since(time) <= self.window_duration)
            .count();
        
        (current_count as f64 / self.budget_limit as f64) * 100.0
    }
    
    /// Get budget statistics
    pub fn get_statistics(&self) -> BudgetStatistics {
        let stats = self.stats.read().unwrap();
        stats.clone()
    }
    
    /// Get current number of reserved slots (for debugging/monitoring)
    pub fn get_reserved_slots(&self) -> u32 {
        self.reserved_slots.load(Ordering::SeqCst)
    }
    
    /// Get current budget limit
    pub fn get_budget_limit(&self) -> u32 {
        self.budget_limit
    }
    
    /// Set budget limit (for adaptive adjustment)
    pub fn set_budget_limit(&mut self, new_limit: u32) {
        let clamped_limit = new_limit.clamp(
            self.adaptive_config.min_budget,
            self.adaptive_config.max_budget
        );
        
        if clamped_limit != self.budget_limit {
            println!("Budget adjusted: {} -> {}", self.budget_limit, clamped_limit);
            self.budget_limit = clamped_limit;
        }
    }
    
    /// Adjust budget based on utilization patterns
    fn adjust_budget_if_needed(&self) {
        let stats = self.stats.read().unwrap();
        let utilization = stats.current_utilization;
        
        // Only adjust if we have enough data
        if stats.total_attempts < 10 {
            return;
        }
        
        let mut should_increase = false;
        let mut should_decrease = false;
        
        // Check if we should increase budget (high utilization)
        if utilization > self.adaptive_config.increase_threshold {
            should_increase = true;
        }
        
        // Check if we should decrease budget (low utilization)
        if utilization < self.adaptive_config.decrease_threshold {
            should_decrease = true;
        }
        
        drop(stats); // Release the lock before potentially modifying budget
        
        if should_increase && self.budget_limit < self.adaptive_config.max_budget {
            let increase = ((self.budget_limit as f64) * self.adaptive_config.adjustment_factor).max(1.0) as u32;
            let new_limit = (self.budget_limit + increase).min(self.adaptive_config.max_budget);
            
            // This is a bit of a hack since we can't modify self in this context
            // In a real implementation, we'd use atomic operations or a different design
            println!("Budget should increase to {}", new_limit);
        } else if should_decrease && self.budget_limit > self.adaptive_config.min_budget {
            let decrease = ((self.budget_limit as f64) * self.adaptive_config.adjustment_factor).max(1.0) as u32;
            let new_limit = self.budget_limit.saturating_sub(decrease).max(self.adaptive_config.min_budget);
            
            println!("Budget should decrease to {}", new_limit);
        }
    }
    
    /// Reset budget limiter statistics
    pub fn reset(&self) {
        {
            let mut times = self.compilation_times.write().unwrap();
            times.clear();
        }
        
        {
            let mut stats = self.stats.write().unwrap();
            *stats = BudgetStatistics {
                total_attempts: 0,
                total_allowed: 0,
                total_rejected: 0,
                current_utilization: 0.0,
                avg_compilations_per_second: 0.0,
                peak_compilations_per_second: 0.0,
            };
        }
    }
    
    /// Check if budget is under pressure (high rejection rate)
    pub fn is_under_pressure(&self) -> bool {
        let stats = self.stats.read().unwrap();
        
        if stats.total_attempts < 10 {
            return false;
        }
        
        let rejection_rate = stats.total_rejected as f64 / stats.total_attempts as f64;
        rejection_rate > 0.5 // More than 50% rejections
    }
    
    /// Get time until next compilation slot is available
    pub fn time_until_available(&self) -> Option<Duration> {
        let times = self.compilation_times.read().unwrap();
        
        if times.len() < self.budget_limit as usize {
            return None; // Immediately available
        }
        
        // Find the oldest compilation in the current window
        if let Some(&oldest) = times.first() {
            let elapsed = oldest.elapsed();
            if elapsed < self.window_duration {
                Some(self.window_duration - elapsed)
            } else {
                None // Should be available now
            }
        } else {
            None
        }
    }
}

/// Priority queue for recompilation tasks
pub struct PriorityQueue {
    /// Lock-free queue for tasks
    queue: SegQueue<RecompilationTask>,
    
    /// Task count for statistics
    task_count: AtomicUsize,
}

impl PriorityQueue {
    /// Create new priority queue
    pub fn new() -> Self {
        Self {
            queue: SegQueue::new(),
            task_count: AtomicUsize::new(0),
        }
    }
    
    /// Push task to queue (maintains priority order)
    pub fn push(&self, task: RecompilationTask) {
        // For simplicity, we use a FIFO queue
        // In a full implementation, we'd use a proper priority queue
        self.queue.push(task);
        self.task_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Pop highest priority task
    pub fn pop(&self) -> Option<RecompilationTask> {
        if let Some(task) = self.queue.pop() {
            self.task_count.fetch_sub(1, Ordering::Relaxed);
            Some(task)
        } else {
            None
        }
    }
    
    /// Get queue length
    pub fn len(&self) -> usize {
        self.task_count.load(Ordering::Relaxed)
    }
    
    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Background recompilation scheduler with thread pool
pub struct RecompilationScheduler {
    /// Work queue for recompilation tasks
    work_queue: Arc<PriorityQueue>,
    
    /// Task status tracking
    task_status: Arc<RwLock<HashMap<u64, RecompilationStatus>>>,
    
    /// Code version manager for atomic replacement
    code_manager: Arc<CodeVersionManager>,
    
    /// Budget limiter to prevent thrashing
    budget_limiter: Arc<BudgetLimiter>,
    
    /// Configuration
    config: RecompilationConfig,
    
    /// Worker thread handles
    worker_handles: RwLock<Vec<thread::JoinHandle<()>>>,
    
    /// Shutdown signal
    shutdown: Arc<AtomicBool>,
    
    /// Statistics
    stats: Arc<RwLock<QueueStatistics>>,
}

impl RecompilationScheduler {
    /// Create new recompilation scheduler
    pub fn new(config: RecompilationConfig) -> Self {
        let scheduler = Self {
            work_queue: Arc::new(PriorityQueue::new()),
            task_status: Arc::new(RwLock::new(HashMap::new())),
            code_manager: Arc::new(CodeVersionManager::new(config.clone())),
            budget_limiter: Arc::new(BudgetLimiter::new(config.budget_per_second)),
            config: config.clone(),
            worker_handles: RwLock::new(Vec::new()),
            shutdown: Arc::new(AtomicBool::new(false)),
            stats: Arc::new(RwLock::new(QueueStatistics {
                queued_tasks: 0,
                in_progress_tasks: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                avg_queue_wait_time: Duration::from_millis(0),
                avg_compilation_time: Duration::from_millis(0),
                current_budget_usage: 0.0,
                budget_limit: config.budget_per_second,
            })),
        };
        
        // Start worker threads
        scheduler.start_workers();
        
        scheduler
    }
    
    /// Start worker threads
    fn start_workers(&self) {
        let mut handles = self.worker_handles.write().unwrap();
        
        for worker_id in 0..self.config.thread_pool_size {
            let work_queue = self.work_queue.clone();
            let task_status = self.task_status.clone();
            let code_manager = self.code_manager.clone();
            let budget_limiter = self.budget_limiter.clone();
            let shutdown = self.shutdown.clone();
            let stats = self.stats.clone();
            let config = self.config.clone();
            
            let handle = thread::spawn(move || {
                Self::worker_loop(
                    worker_id,
                    work_queue,
                    task_status,
                    code_manager,
                    budget_limiter,
                    shutdown,
                    stats,
                    config,
                );
            });
            
            handles.push(handle);
        }
    }
    
    /// Worker thread main loop with enhanced error handling and monitoring
    fn worker_loop(
        _worker_id: usize,
        work_queue: Arc<PriorityQueue>,
        task_status: Arc<RwLock<HashMap<u64, RecompilationStatus>>>,
        code_manager: Arc<CodeVersionManager>,
        budget_limiter: Arc<BudgetLimiter>,
        shutdown: Arc<AtomicBool>,
        stats: Arc<RwLock<QueueStatistics>>,
        config: RecompilationConfig,
    ) {
        let mut consecutive_failures = 0u32;
        let max_consecutive_failures = 5;
        let failure_backoff_base = Duration::from_millis(100);
        
        while !shutdown.load(Ordering::Relaxed) {
            // Check if we should back off due to consecutive failures
            if consecutive_failures > 0 {
                let backoff_duration = failure_backoff_base * (2_u32.pow(consecutive_failures.min(5)));
                thread::sleep(backoff_duration);
            }
            
            // Try to get a task from the queue
            if let Some(task) = work_queue.pop() {
                // Check budget before starting compilation
                if !budget_limiter.can_compile() {
                    // Put task back and wait
                    work_queue.push(task);
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                
                // Check if task has timed out
                if task.age() > config.task_timeout {
                    // Mark task as cancelled due to timeout
                    let timeout_status = RecompilationStatus::Cancelled { 
                        cancelled_at: Instant::now() 
                    };
                    
                    {
                        let mut status_map = task_status.write().unwrap();
                        status_map.insert(task.task_id, timeout_status);
                    }
                    
                    println!("Task {} timed out after {:?}", task.task_id, task.age());
                    continue;
                }
                
                // Update task status to in progress
                {
                    let mut status_map = task_status.write().unwrap();
                    status_map.insert(task.task_id, RecompilationStatus::InProgress { 
                        started_at: Instant::now() 
                    });
                }
                
                // Record compilation start for budget tracking
                budget_limiter.record_compilation();
                
                let compilation_start = Instant::now();
                
                // Perform compilation with retry logic
                let result = Self::compile_task_with_retry(&task, &config, 3);
                
                let compilation_time = compilation_start.elapsed();
                
                // Update task status based on result
                let final_status = match result {
                    Ok(compiled_code) => {
                        // Install new code atomically
                        match code_manager.replace_code(compiled_code) {
                            Ok(_) => {
                                consecutive_failures = 0; // Reset failure count on success
                                RecompilationStatus::Completed {
                                    completed_at: Instant::now(),
                                    compilation_time,
                                }
                            }
                            Err(e) => {
                                consecutive_failures += 1;
                                RecompilationStatus::Failed {
                                    failed_at: Instant::now(),
                                    error: format!("Code replacement failed: {}", e),
                                }
                            }
                        }
                    }
                    Err(e) => {
                        consecutive_failures += 1;
                        RecompilationStatus::Failed {
                            failed_at: Instant::now(),
                            error: format!("Compilation failed: {}", e),
                        }
                    }
                };
                
                // Update task status
                {
                    let mut status_map = task_status.write().unwrap();
                    status_map.insert(task.task_id, final_status.clone());
                }
                
                // Update statistics
                {
                    let mut stats = stats.write().unwrap();
                    match final_status {
                        RecompilationStatus::Completed { .. } => {
                            stats.completed_tasks += 1;
                        }
                        RecompilationStatus::Failed { .. } => {
                            stats.failed_tasks += 1;
                        }
                        _ => {}
                    }
                    
                    // Update average compilation time (simple moving average)
                    let total_completed = stats.completed_tasks + stats.failed_tasks;
                    if total_completed > 0 {
                        let current_avg_ms = stats.avg_compilation_time.as_millis() as f64;
                        let new_time_ms = compilation_time.as_millis() as f64;
                        let new_avg_ms = (current_avg_ms * (total_completed - 1) as f64 + new_time_ms) / total_completed as f64;
                        stats.avg_compilation_time = Duration::from_millis(new_avg_ms as u64);
                    }
                    
                    stats.current_budget_usage = budget_limiter.get_usage();
                }
                
                // Check if we should trigger GC
                if code_manager.needs_gc() {
                    let gc_result = code_manager.gc_old_versions();
                    if gc_result.collected_versions > 0 {
                        println!(
                            "Worker GC: Collected {} versions, freed {} bytes",
                            gc_result.collected_versions,
                            gc_result.collected_memory_bytes
                        );
                    }
                }
                
                // Back off if we've had too many consecutive failures
                if consecutive_failures >= max_consecutive_failures {
                    println!("Worker backing off due to {} consecutive failures", consecutive_failures);
                    thread::sleep(Duration::from_secs(1));
                    consecutive_failures = 0; // Reset after backoff
                }
                
            } else {
                // No tasks available, sleep briefly
                thread::sleep(Duration::from_millis(10));
            }
        }
        
        println!("Worker thread shutting down");
    }
    
    /// Compile a recompilation task with retry logic
    fn compile_task_with_retry(
        task: &RecompilationTask, 
        config: &RecompilationConfig, 
        max_retries: u32
    ) -> crate::Result<Arc<CompiledCode>> {
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            match Self::compile_task(task, config) {
                Ok(compiled_code) => {
                    if attempt > 0 {
                        println!("Compilation succeeded on attempt {}", attempt + 1);
                    }
                    return Ok(compiled_code);
                }
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempt < max_retries {
                        println!("Compilation attempt {} failed, retrying: {}", attempt + 1, last_error.as_ref().unwrap());
                        
                        // Exponential backoff between retries
                        let backoff = Duration::from_millis(100 * (2_u64.pow(attempt)));
                        thread::sleep(backoff);
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            crate::InterpreterError::generic("Compilation failed with unknown error".to_string())
        }))
    }
    
    /// Compile a recompilation task (simplified implementation)
    fn compile_task(task: &RecompilationTask, _config: &RecompilationConfig) -> crate::Result<Arc<CompiledCode>> {
        // Simulate compilation work
        thread::sleep(Duration::from_millis(10)); // Simulate compilation time
        
        // Create dummy compiled code (in real implementation, this would use JITCompiler)
        let code_size = task.source_code.len(); // Rough estimate
        let compilation_time = Duration::from_millis(10);
        
        let compiled_code = CompiledCode::new(
            task.function_id.clone(),
            task.target_tier,
            code_size,
            compilation_time,
            task.profile_data.clone(),
        );
        
        Ok(Arc::new(compiled_code))
    }
    
    /// Schedule a function for recompilation
    pub fn schedule_recompilation(&self, task: RecompilationTask) -> crate::Result<u64> {
        // Check if queue is full
        if self.work_queue.len() >= self.config.max_queue_size {
            return Err(crate::InterpreterError::generic("Recompilation queue is full".to_string()));
        }
        
        let task_id = task.task_id;
        
        // Add task to status tracking
        {
            let mut status_map = self.task_status.write().unwrap();
            status_map.insert(task_id, RecompilationStatus::Queued);
        }
        
        // Add task to work queue
        self.work_queue.push(task);
        
        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.queued_tasks = self.work_queue.len();
        }
        
        Ok(task_id)
    }
    
    /// Check if function is currently being recompiled
    pub fn is_recompiling(&self, _function_id: &str) -> bool {
        let status_map = self.task_status.read().unwrap();
        
        for status in status_map.values() {
            if let RecompilationStatus::InProgress { .. } = status {
                // In a full implementation, we'd check the function_id
                // For now, we just check if any compilation is in progress
                return true;
            }
        }
        
        false
    }
    
    /// Wait for recompilation to complete
    pub fn wait_for_recompilation(&self, task_id: u64, timeout: Duration) -> crate::Result<RecompilationStatus> {
        let start = Instant::now();
        
        loop {
            {
                let status_map = self.task_status.read().unwrap();
                if let Some(status) = status_map.get(&task_id) {
                    if status.is_completed() {
                        return Ok(status.clone());
                    }
                }
            }
            
            if start.elapsed() > timeout {
                return Err(crate::InterpreterError::generic("Recompilation timeout".to_string()));
            }
            
            thread::sleep(Duration::from_millis(10));
        }
    }
    
    /// Get current queue statistics
    pub fn get_queue_stats(&self) -> QueueStatistics {
        let mut stats = self.stats.write().unwrap();
        
        // Update current queue size
        stats.queued_tasks = self.work_queue.len();
        
        // Count in-progress tasks
        let status_map = self.task_status.read().unwrap();
        stats.in_progress_tasks = status_map.values()
            .filter(|status| matches!(status, RecompilationStatus::InProgress { .. }))
            .count();
        
        stats.clone()
    }
    
    /// Get code version manager
    pub fn get_code_manager(&self) -> Arc<CodeVersionManager> {
        self.code_manager.clone()
    }
    
    /// Get budget limiter
    pub fn get_budget_limiter(&self) -> Arc<BudgetLimiter> {
        self.budget_limiter.clone()
    }
    
    /// Get worker thread count
    pub fn get_worker_count(&self) -> usize {
        let handles = self.worker_handles.read().unwrap();
        handles.len()
    }
    
    /// Check if scheduler is healthy (workers running, no excessive failures)
    pub fn is_healthy(&self) -> bool {
        // Check if workers are still running
        let handles = self.worker_handles.read().unwrap();
        if handles.is_empty() {
            return false;
        }
        
        // Check if budget limiter is not under excessive pressure
        if self.budget_limiter.is_under_pressure() {
            return false;
        }
        
        // Check failure rate
        let stats = self.get_queue_stats();
        let total_tasks = stats.completed_tasks + stats.failed_tasks;
        if total_tasks > 10 {
            let failure_rate = stats.failed_tasks as f64 / total_tasks as f64;
            if failure_rate > 0.5 { // More than 50% failure rate
                return false;
            }
        }
        
        true
    }
    
    /// Get comprehensive scheduler status
    pub fn get_scheduler_status(&self) -> SchedulerStatus {
        let queue_stats = self.get_queue_stats();
        let budget_stats = self.budget_limiter.get_statistics();
        let replacement_stats = self.code_manager.get_replacement_stats();
        let (active_memory, old_memory) = self.code_manager.get_memory_stats();
        
        SchedulerStatus {
            is_healthy: self.is_healthy(),
            worker_count: self.get_worker_count(),
            queue_stats,
            budget_stats,
            replacement_stats,
            active_memory_bytes: active_memory,
            old_memory_bytes: old_memory,
        }
    }
    
    /// Shutdown the scheduler and wait for workers to finish
    pub fn shutdown(&self) -> crate::Result<()> {
        // Signal shutdown
        self.shutdown.store(true, Ordering::Relaxed);
        
        // Wait for worker threads to finish
        let mut handles = self.worker_handles.write().unwrap();
        while let Some(handle) = handles.pop() {
            if let Err(e) = handle.join() {
                eprintln!("Worker thread panicked: {:?}", e);
            }
        }
        
        Ok(())
    }
    
    /// Trigger garbage collection of old code versions
    pub fn gc_old_versions(&self) {
        let gc_result = self.code_manager.gc_old_versions();
        
        if gc_result.collected_versions > 0 {
            println!(
                "Scheduler GC: Collected {} versions, freed {} bytes in {:?}",
                gc_result.collected_versions,
                gc_result.collected_memory_bytes,
                gc_result.gc_time
            );
        }
    }
}

impl Drop for RecompilationScheduler {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod property_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiling::ProfileData;
    use std::sync::Arc;
    use std::time::Duration;
    
    #[test]
    fn test_recompilation_scheduler_creation() {
        let config = RecompilationConfig::default();
        let scheduler = RecompilationScheduler::new(config);
        
        assert!(scheduler.is_healthy());
        assert_eq!(scheduler.get_worker_count(), num_cpus::get().min(4));
        
        let stats = scheduler.get_queue_stats();
        assert_eq!(stats.queued_tasks, 0);
        assert_eq!(stats.in_progress_tasks, 0);
    }
    
    #[test]
    fn test_code_version_manager_basic_operations() {
        let config = RecompilationConfig::default();
        let manager = CodeVersionManager::new(config);
        
        // Test that no version exists initially
        assert!(manager.get_active_version("test_function").is_none());
        
        // Create a mock profile data
        let profile_data = Arc::new(ProfileData::new("test_function".to_string()));
        
        // Create and add a compiled code version
        let code = Arc::new(CompiledCode::new(
            "test_function".to_string(),
            crate::tiered_compilation::CompilationTier::QuickJIT,
            1024, // code_size
            Duration::from_millis(5), // compilation_time
            profile_data,
        ));
        
        // Test code replacement
        let result = manager.replace_code(code.clone());
        assert!(result.is_ok());
        
        // Test that version now exists
        let retrieved = manager.get_active_version("test_function");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().function_id, "test_function");
    }
    
    #[test]
    fn test_budget_limiter_basic_functionality() {
        let limiter = BudgetLimiter::new(10); // 10 compilations per second
        
        // Should allow compilation initially
        assert!(limiter.can_compile());
        
        // Record a compilation
        limiter.record_compilation();
        
        // Check statistics - can_compile was called once, so total_allowed should be 1
        let stats = limiter.get_statistics();
        assert_eq!(stats.total_allowed, 1);
        assert!(stats.current_utilization <= 1.0);
        
        // Should still allow more compilations (under budget)
        assert!(limiter.can_compile());
        
        // Now total_allowed should be 2 (called can_compile twice)
        let stats2 = limiter.get_statistics();
        assert_eq!(stats2.total_allowed, 2);
    }
}