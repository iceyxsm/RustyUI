//! File system change monitoring for development mode using 2026 best practices

use crate::error::Result;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, Debouncer, DebouncedEventKind, DebounceEventResult};
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

/// File system change monitor for development mode using 2026 notify architecture
pub struct ChangeMonitor {
    /// Debounced file system watcher using notify 9.0 architecture
    debouncer: Option<Debouncer<RecommendedWatcher>>,
    
    /// Channel for receiving debounced file change events
    receiver: Receiver<DebounceEventResult>,
    
    /// Paths being watched
    watch_paths: Vec<std::path::PathBuf>,
    
    /// Debounce delay optimized for 2026 standards (50ms for near-real-time)
    debounce_delay: Duration,
    
    /// Statistics for monitoring performance
    stats: ChangeStats,
}

impl ChangeMonitor {
    /// Create a new change monitor for the given paths using 2026 best practices
    pub fn new(watch_paths: &[std::path::PathBuf]) -> Result<Self> {
        let (_sender, receiver) = mpsc::channel();
        
        // Use 50ms debounce as recommended by 2026 research for near-real-time processing
        let debounce_delay = Duration::from_millis(50);
        
        Ok(Self {
            debouncer: None,
            receiver,
            watch_paths: watch_paths.to_vec(),
            debounce_delay,
            stats: ChangeStats::new(),
        })
    }
    
    /// Start watching for file changes using the modular 2026 notify architecture
    pub fn start_watching(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        
        // Create debouncer using the new 2026 modular architecture
        let mut debouncer = new_debouncer(
            self.debounce_delay,
            move |result: DebounceEventResult| {
                if let Err(e) = tx.send(result) {
                    eprintln!("Failed to send debounced event: {}", e);
                }
            },
        )?;
        
        // Watch all configured paths with recursive monitoring
        for path in &self.watch_paths {
            if path.exists() {
                debouncer.watcher().watch(path, RecursiveMode::Recursive)?;
                println!("🔭 Watching path: {:?}", path);
            } else {
                eprintln!("⚠️  Path does not exist, skipping: {:?}", path);
            }
        }
        
        self.debouncer = Some(debouncer);
        self.receiver = rx;
        
        println!("✅ ChangeMonitor started with {}ms debounce delay", self.debounce_delay.as_millis());
        Ok(())
    }
    
    /// Check for pending file changes using 2026 optimized processing
    pub fn check_changes(&mut self) -> Vec<FileChange> {
        let mut changes = Vec::new();
        let start_time = Instant::now();
        
        // Process all pending debounced events (non-blocking)
        while let Ok(event_result) = self.receiver.try_recv() {
            match event_result {
                Ok(events) => {
                    for event in events {
                        if let Some(change) = self.process_debounced_event(event) {
                            changes.push(change);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Debounced file watcher error: {}", e);
                    self.stats.error_count += 1;
                }
            }
        }
        
        // Update performance statistics
        if !changes.is_empty() {
            let processing_time = start_time.elapsed();
            self.stats.total_events += changes.len();
            self.stats.total_processing_time += processing_time;
            
            // Log performance if it exceeds 2026 targets (should be <50ms)
            if processing_time > Duration::from_millis(50) {
                eprintln!("⚠️  Change processing took {:?} (target: <50ms)", processing_time);
            }
        }
        
        changes
    }
    
    /// Process a debounced event into a file change using 2026 filtering logic
    fn process_debounced_event(&self, event: notify_debouncer_mini::DebouncedEvent) -> Option<FileChange> {
        // Apply 2026 intelligent filtering - only process relevant file types
        if !self.is_relevant_file(&event.path) {
            return None;
        }
        
        // Convert debounced event to our internal representation
        let change_type = match event.kind {
            DebouncedEventKind::Any => ChangeType::Modified,
            DebouncedEventKind::AnyContinuous => ChangeType::Modified,
            _ => ChangeType::Modified, // Handle any future variants
        };
        
        Some(FileChange {
            path: event.path,
            change_type,
            timestamp: Instant::now(),
        })
    }
    
    /// Check if a file is relevant for hot reload using 2026 best practices
    fn is_relevant_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            match extension.to_str() {
                // Core Rust files
                Some("rs") => true,
                // Configuration files that affect UI
                Some("toml") | Some("json") | Some("yaml") | Some("yml") => {
                    // Only watch config files that might affect UI
                    let filename = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    
                    matches!(filename, "Cargo.toml" | "rustyui.toml" | "config.json" | "ui.json")
                }
                // Asset files that might be used in UI
                Some("css") | Some("scss") | Some("svg") | Some("png") | Some("jpg") | Some("jpeg") => true,
                _ => false,
            }
        } else {
            false
        }
    }
    
    /// Get performance statistics for monitoring
    pub fn get_stats(&self) -> &ChangeStats {
        &self.stats
    }
    
    /// Reset performance statistics
    pub fn reset_stats(&mut self) {
        self.stats = ChangeStats::new();
    }
    
    /// Check if the monitor is actively watching
    pub fn is_watching(&self) -> bool {
        self.debouncer.is_some()
    }
    
    /// Get the current debounce delay
    pub fn debounce_delay(&self) -> Duration {
        self.debounce_delay
    }
}

/// Information about a file change optimized for 2026 hot reload
#[derive(Debug, Clone)]
pub struct FileChange {
    /// Path to the changed file
    pub path: std::path::PathBuf,
    
    /// Type of change detected
    pub change_type: ChangeType,
    
    /// When the change was processed (after debouncing)
    pub timestamp: Instant,
}

/// Types of file changes relevant for hot reload
#[derive(Debug, Clone)]
pub enum ChangeType {
    Modified,
    Created,
    Deleted,
}

/// Performance statistics for monitoring 2026 targets
#[derive(Debug, Clone)]
pub struct ChangeStats {
    /// Total number of events processed
    pub total_events: usize,
    
    /// Total time spent processing events
    pub total_processing_time: Duration,
    
    /// Number of errors encountered
    pub error_count: usize,
    
    /// When statistics collection started
    pub start_time: Instant,
}

impl ChangeStats {
    fn new() -> Self {
        Self {
            total_events: 0,
            total_processing_time: Duration::from_nanos(0),
            error_count: 0,
            start_time: Instant::now(),
        }
    }
    
    /// Get average processing time per event
    pub fn average_processing_time(&self) -> Duration {
        if self.total_events > 0 {
            self.total_processing_time / self.total_events as u32
        } else {
            Duration::from_nanos(0)
        }
    }
    
    /// Check if performance meets 2026 targets (<50ms processing)
    pub fn meets_performance_targets(&self) -> bool {
        self.average_processing_time() < Duration::from_millis(50)
    }
}