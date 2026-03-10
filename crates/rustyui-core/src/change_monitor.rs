//! File system change monitoring for development mode with 2026 optimizations

use crate::error::Result;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

/// File system change monitor optimized for 2026 hot reload performance
/// 
/// Features:
/// - Sub-50ms change detection using native inotify
/// - Intelligent debouncing to prevent event spam
/// - Glob pattern filtering for relevant files only
/// - Event type classification for optimized handling
pub struct ChangeMonitor {
    /// File system watcher using native OS APIs
    watcher: Option<RecommendedWatcher>,
    
    /// Channel for receiving file change events
    receiver: Receiver<notify::Result<Event>>,
    
    /// Sender for file change events
    _sender: Sender<notify::Result<Event>>,
    
    /// Paths being watched
    watch_paths: Vec<std::path::PathBuf>,
    
    /// Last change timestamp for debouncing
    last_change: Option<Instant>,
    
    /// Debounce delay optimized for 2026 standards (sub-50ms)
    debounce_delay: Duration,
    
    /// Event filtering patterns
    file_patterns: Vec<String>,
    
    /// Performance metrics
    events_processed: u64,
    average_response_time: Duration,
}

impl ChangeMonitor {
    /// Create a new change monitor with 2026 performance optimizations
    pub fn new(watch_paths: &[std::path::PathBuf]) -> Result<Self> {
        let (sender, receiver) = mpsc::channel();
        
        Ok(Self {
            watcher: None,
            receiver,
            _sender: sender,
            watch_paths: watch_paths.to_vec(),
            last_change: None,
            debounce_delay: Duration::from_millis(25), // Optimized for 2026: 25ms for instant feedback
            file_patterns: vec![
                "*.rs".to_string(),
                "*.toml".to_string(), 
                "*.json".to_string(),
                "*.md".to_string(),
            ],
            events_processed: 0,
            average_response_time: Duration::from_millis(0),
        })
    }
    
    /// Start watching for file changes with native OS optimizations
    pub fn start_watching(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        
        // Configure watcher for maximum performance
        let config = Config::default()
            .with_poll_interval(Duration::from_millis(10)) // 10ms polling for instant detection
            .with_compare_contents(false); // Skip content comparison for speed
        
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Err(e) = tx.send(res) {
                    eprintln!("Failed to send file change event: {}", e);
                }
            },
            config,
        )?;
        
        // Watch all configured paths with recursive monitoring
        for path in &self.watch_paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)?;
                println!("📁 Watching: {:?}", path);
            }
        }
        
        self.watcher = Some(watcher);
        self.receiver = rx;
        
        println!("🚀 File watcher started with 2026 optimizations (25ms debounce)");
        Ok(())
    }
    
    /// Check for pending file changes with intelligent filtering
    pub fn check_changes(&mut self) -> Vec<FileChange> {
        let start_time = Instant::now();
        let mut changes = Vec::new();
        
        // Process all pending events in batch for efficiency
        while let Ok(event_result) = self.receiver.try_recv() {
            match event_result {
                Ok(event) => {
                    if let Some(change) = self.process_event(event) {
                        changes.push(change);
                    }
                }
                Err(e) => {
                    eprintln!("File watcher error: {}", e);
                }
            }
        }
        
        // Apply intelligent debouncing
        if !changes.is_empty() {
            let now = Instant::now();
            if let Some(last_change) = self.last_change {
                if now.duration_since(last_change) < self.debounce_delay {
                    // Too soon, ignore these changes for debouncing
                    return Vec::new();
                }
            }
            self.last_change = Some(now);
            
            // Update performance metrics
            self.events_processed += changes.len() as u64;
            let response_time = start_time.elapsed();
            
            // Calculate average response time with proper type handling
            let current_avg_nanos = self.average_response_time.as_nanos() as u64;
            let new_response_nanos = response_time.as_nanos().min(u64::MAX as u128) as u64;
            let new_avg_nanos = (current_avg_nanos + new_response_nanos) / 2;
            
            self.average_response_time = Duration::from_nanos(new_avg_nanos);
            
            if response_time > Duration::from_millis(50) {
                println!("⚠️  Slow file change detection: {:?} (target: <50ms)", response_time);
            }
        }
        
        changes
    }
    
    /// Process a file system event with 2026 event classification
    fn process_event(&self, event: Event) -> Option<FileChange> {
        use notify::EventKind;
        
        match event.kind {
            EventKind::Modify(_) => {
                if let Some(path) = event.paths.first() {
                    if self.is_relevant_file(path) {
                        return Some(FileChange {
                            path: path.clone(),
                            change_type: ChangeType::Modified,
                            timestamp: Instant::now(),
                            priority: self.get_change_priority(path),
                        });
                    }
                }
            }
            EventKind::Create(_) => {
                if let Some(path) = event.paths.first() {
                    if self.is_relevant_file(path) {
                        return Some(FileChange {
                            path: path.clone(),
                            change_type: ChangeType::Created,
                            timestamp: Instant::now(),
                            priority: ChangePriority::High, // New files are high priority
                        });
                    }
                }
            }
            EventKind::Remove(_) => {
                if let Some(path) = event.paths.first() {
                    if self.is_relevant_file(path) {
                        return Some(FileChange {
                            path: path.clone(),
                            change_type: ChangeType::Deleted,
                            timestamp: Instant::now(),
                            priority: ChangePriority::High, // Deletions are high priority
                        });
                    }
                }
            }
            _ => {}
        }
        
        None
    }
    
    /// Check if a file is relevant for hot reload with pattern matching
    fn is_relevant_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy();
            
            // Check against configured patterns
            for pattern in &self.file_patterns {
                if pattern.ends_with(&format!("*.{}", ext_str)) {
                    return true;
                }
            }
        }
        
        // Additional checks for specific files
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();
            match filename_str.as_ref() {
                "Cargo.toml" | "rustyui.toml" | "package.json" => true,
                _ => false,
            }
        } else {
            false
        }
    }
    
    /// Determine change priority for processing optimization
    fn get_change_priority(&self, path: &Path) -> ChangePriority {
        if let Some(extension) = path.extension() {
            match extension.to_str() {
                Some("rs") => ChangePriority::Critical, // Rust files are critical
                Some("toml") => ChangePriority::High,    // Config files are high priority
                Some("json") => ChangePriority::Medium,  // Data files are medium
                Some("md") => ChangePriority::Low,       // Documentation is low priority
                _ => ChangePriority::Medium,
            }
        } else {
            ChangePriority::Medium
        }
    }
    
    /// Get performance statistics
    pub fn get_performance_stats(&self) -> PerformanceStats {
        PerformanceStats {
            events_processed: self.events_processed,
            average_response_time: self.average_response_time,
            debounce_delay: self.debounce_delay,
            is_optimal: self.average_response_time < Duration::from_millis(50),
        }
    }
    
    /// Update file patterns for filtering
    pub fn set_file_patterns(&mut self, patterns: Vec<String>) {
        self.file_patterns = patterns;
    }
    
    /// Adjust debounce delay for different performance requirements
    pub fn set_debounce_delay(&mut self, delay: Duration) {
        self.debounce_delay = delay;
    }
}

/// Information about a file change with 2026 enhancements
#[derive(Debug, Clone)]
pub struct FileChange {
    /// Path to the changed file
    pub path: std::path::PathBuf,
    
    /// Type of change
    pub change_type: ChangeType,
    
    /// When the change occurred
    pub timestamp: Instant,
    
    /// Processing priority
    pub priority: ChangePriority,
}

/// Types of file changes
#[derive(Debug, Clone)]
pub enum ChangeType {
    Modified,
    Created,
    Deleted,
}

/// Change processing priority for optimization
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChangePriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Performance statistics for monitoring
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub events_processed: u64,
    pub average_response_time: Duration,
    pub debounce_delay: Duration,
    pub is_optimal: bool,
}