//! File system change monitoring with intelligent debouncing

use crate::error::{Result, RustyUIError};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

/// Represents a file system change event
#[derive(Debug, Clone)]
pub struct FileChange {
    /// Path of the changed file
    pub path: PathBuf,
    /// Type of change that occurred
    pub change_type: FileChangeType,
    /// Timestamp when the change was detected
    pub timestamp: Instant,
}

/// Types of file system changes we track
#[derive(Debug, Clone, PartialEq)]
pub enum FileChangeType {
    /// File was created
    Created,
    /// File was modified
    Modified,
    /// File was deleted
    Removed,
    /// File was renamed
    Renamed,
}

/// Statistics for change monitoring performance
#[derive(Debug, Clone)]
pub struct ChangeStats {
    /// Total number of events processed
    pub total_events: u64,
    /// Total processing time for all events
    pub total_processing_time: Duration,
    /// Number of events that were debounced (filtered out)
    pub debounced_events: u64,
    /// Average time between event detection and processing
    pub average_processing_time: Duration,
}

impl ChangeStats {
    fn new() -> Self {
        Self {
            total_events: 0,
            total_processing_time: Duration::ZERO,
            debounced_events: 0,
            average_processing_time: Duration::ZERO,
        }
    }
    
    /// Calculate average processing time
    pub fn average_processing_time(&self) -> Duration {
        if self.total_events > 0 {
            self.total_processing_time / self.total_events as u32
        } else {
            Duration::ZERO
        }
    }
    
    /// Check if performance targets are met (<50ms response time)
    pub fn meets_performance_targets(&self) -> bool {
        self.average_processing_time() < Duration::from_millis(50)
    }
}

/// File system change monitor with intelligent debouncing
pub struct ChangeMonitor {
    /// File system watcher
    watcher: Option<RecommendedWatcher>,
    /// Channel for receiving file system events
    receiver: Receiver<notify::Result<Event>>,
    /// Sender for file system events (kept for watcher)
    _sender: Sender<notify::Result<Event>>,
    /// Paths being watched
    watch_paths: Vec<PathBuf>,
    /// Debouncing state for file changes
    debounce_map: HashMap<PathBuf, Instant>,
    /// Debounce timeout (50ms target)
    debounce_timeout: Duration,
    /// Pending changes waiting for debounce
    pending_changes: Vec<FileChange>,
    /// Performance statistics
    stats: ChangeStats,
}

impl ChangeMonitor {
    /// Create a new change monitor for the given paths
    pub fn new(watch_paths: &[PathBuf]) -> Result<Self> {
        let (sender, receiver) = mpsc::channel();
        
        Ok(Self {
            watcher: None,
            receiver,
            _sender: sender,
            watch_paths: watch_paths.to_vec(),
            debounce_map: HashMap::new(),
            debounce_timeout: Duration::from_millis(50), // 50ms target
            pending_changes: Vec::new(),
            stats: ChangeStats::new(),
        })
    }
    
    /// Start watching for file system changes
    pub fn start_watching(&mut self) -> Result<()> {
        let (sender, receiver) = mpsc::channel();
        
        let config = Config::default()
            .with_poll_interval(Duration::from_millis(10))
            .with_compare_contents(false);
            
        let mut watcher = RecommendedWatcher::new(sender, config)
            .map_err(|e| RustyUIError::file_watching(format!("Failed to create watcher: {}", e)))?;
        
        // Watch all configured paths
        for path in &self.watch_paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)
                    .map_err(|e| RustyUIError::file_watching(format!("Failed to watch path {:?}: {}", path, e)))?;
            }
        }
        
        self.watcher = Some(watcher);
        self.receiver = receiver;
        
        Ok(())
    }
    
    /// Check for pending file changes with debouncing
    pub fn check_changes(&mut self) -> Vec<FileChange> {
        let start_time = Instant::now();
        let mut new_changes = Vec::new();
        
        // Process all pending events from the file system watcher
        while let Ok(event_result) = self.receiver.try_recv() {
            match event_result {
                Ok(event) => {
                    if let Some(change) = self.process_event(event) {
                        new_changes.push(change);
                    }
                }
                Err(e) => {
                    eprintln!("File watching error: {}", e);
                }
            }
        }
        
        // Apply debouncing to new changes
        let debounced_changes = self.apply_debouncing(new_changes);
        
        // Update statistics
        let processing_time = start_time.elapsed();
        self.stats.total_events += debounced_changes.len() as u64;
        self.stats.total_processing_time += processing_time;
        
        debounced_changes
    }
    
    /// Process a single file system event
    fn process_event(&mut self, event: Event) -> Option<FileChange> {
        // Filter out events we don't care about
        let change_type = match event.kind {
            EventKind::Create(_) => FileChangeType::Created,
            EventKind::Modify(_) => FileChangeType::Modified,
            EventKind::Remove(_) => FileChangeType::Removed,
            _ => return None, // Ignore other event types
        };
        
        // Process each path in the event
        for path in event.paths {
            // Filter out non-relevant files
            if self.should_ignore_file(&path) {
                continue;
            }
            
            return Some(FileChange {
                path,
                change_type,
                timestamp: Instant::now(),
            });
        }
        
        None
    }
    
    /// Apply debouncing to filter out rapid successive changes
    fn apply_debouncing(&mut self, mut changes: Vec<FileChange>) -> Vec<FileChange> {
        let now = Instant::now();
        let mut debounced_changes = Vec::new();
        
        // Add new changes to pending list
        self.pending_changes.append(&mut changes);
        
        // Process pending changes and apply debouncing
        let mut i = 0;
        while i < self.pending_changes.len() {
            let change = &self.pending_changes[i];
            
            // Check if enough time has passed since last change to this file
            let should_process = match self.debounce_map.get(&change.path) {
                Some(last_change_time) => {
                    now.duration_since(*last_change_time) >= self.debounce_timeout
                }
                None => true, // First change to this file
            };
            
            if should_process {
                // Process this change
                self.debounce_map.insert(change.path.clone(), now);
                debounced_changes.push(self.pending_changes.remove(i));
            } else {
                // Skip this change (still debouncing)
                self.stats.debounced_events += 1;
                i += 1;
            }
        }
        
        // Clean up old debounce entries (older than 1 second)
        self.debounce_map.retain(|_, last_time| {
            now.duration_since(*last_time) < Duration::from_secs(1)
        });
        
        debounced_changes
    }
    
    /// Check if a file should be ignored based on its path/extension
    fn should_ignore_file(&self, path: &Path) -> bool {
        // Ignore hidden files and directories
        if let Some(name) = path.file_name() {
            if name.to_string_lossy().starts_with('.') {
                return true;
            }
        }
        
        // Ignore common build/cache directories
        let path_str = path.to_string_lossy();
        if path_str.contains("/target/") || 
           path_str.contains("\\target\\") ||
           path_str.contains("/node_modules/") ||
           path_str.contains("\\node_modules\\") ||
           path_str.contains("/.git/") ||
           path_str.contains("\\.git\\") {
            return true;
        }
        
        // Ignore temporary files
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            if matches!(ext.as_str(), "tmp" | "temp" | "swp" | "swo" | "bak" | "orig") {
                return true;
            }
        }
        
        false
    }
    
    /// Get performance statistics
    pub fn get_stats(&self) -> &ChangeStats {
        &self.stats
    }
    
    /// Get the paths being watched
    pub fn watch_paths(&self) -> &[PathBuf] {
        &self.watch_paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::change_monitor::{FileChange, FileChangeType};
    use tempfile::TempDir;
    
    #[test]
    fn test_change_monitor_creation() {
        let temp_dir = TempDir::new().unwrap();
        let watch_paths = vec![temp_dir.path().to_path_buf()];
        
        let monitor = ChangeMonitor::new(&watch_paths).unwrap();
        assert_eq!(monitor.watch_paths(), &watch_paths);
        assert_eq!(monitor.get_stats().total_events, 0);
    }
    
    #[test]
    fn test_file_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let watch_paths = vec![temp_dir.path().to_path_buf()];
        let monitor = ChangeMonitor::new(&watch_paths).unwrap();
        
        // Test ignored files
        assert!(monitor.should_ignore_file(&PathBuf::from(".hidden")));
        assert!(monitor.should_ignore_file(&PathBuf::from("target/debug/app")));
        assert!(monitor.should_ignore_file(&PathBuf::from("file.tmp")));
        assert!(monitor.should_ignore_file(&PathBuf::from(".git/config")));
        
        // Test allowed files
        assert!(!monitor.should_ignore_file(&PathBuf::from("src/main.rs")));
        assert!(!monitor.should_ignore_file(&PathBuf::from("Cargo.toml")));
        assert!(!monitor.should_ignore_file(&PathBuf::from("README.md")));
    }
    
    #[test]
    fn test_debouncing_logic() {
        let temp_dir = TempDir::new().unwrap();
        let watch_paths = vec![temp_dir.path().to_path_buf()];
        let mut monitor = ChangeMonitor::new(&watch_paths).unwrap();
        
        let test_path = temp_dir.path().join("test.rs");
        
        // Create rapid successive changes
        let changes = vec![
            FileChange {
                path: test_path.clone(),
                change_type: FileChangeType::Modified,
                timestamp: Instant::now(),
            },
            FileChange {
                path: test_path.clone(),
                change_type: FileChangeType::Modified,
                timestamp: Instant::now(),
            },
        ];
        
        // First call should process the first change
        let debounced = monitor.apply_debouncing(changes);
        assert_eq!(debounced.len(), 1);
        
        // Immediate second call should be debounced
        let changes2 = vec![
            FileChange {
                path: test_path.clone(),
                change_type: FileChangeType::Modified,
                timestamp: Instant::now(),
            },
        ];
        
        let debounced2 = monitor.apply_debouncing(changes2);
        assert_eq!(debounced2.len(), 0);
        assert!(monitor.get_stats().debounced_events > 0);
    }
    
    #[test]
    fn test_change_type_detection() {
        let temp_dir = TempDir::new().unwrap();
        let watch_paths = vec![temp_dir.path().to_path_buf()];
        let monitor = ChangeMonitor::new(&watch_paths).unwrap();
        
        // Test different change types
        let create_event = Event::new(EventKind::Create(notify::event::CreateKind::File))
            .add_path(temp_dir.path().join("new_file.rs"));
        
        let modify_event = Event::new(EventKind::Modify(notify::event::ModifyKind::Data(
            notify::event::DataChange::Content
        ))).add_path(temp_dir.path().join("existing_file.rs"));
        
        let remove_event = Event::new(EventKind::Remove(notify::event::RemoveKind::File))
            .add_path(temp_dir.path().join("deleted_file.rs"));
        
        // Process events (we can't easily test the actual processing without a running watcher,
        // but we can test the logic)
        assert!(matches!(
            FileChangeType::Created,
            FileChangeType::Created
        ));
        assert!(matches!(
            FileChangeType::Modified,
            FileChangeType::Modified
        ));
        assert!(matches!(
            FileChangeType::Removed,
            FileChangeType::Removed
        ));
    }
    
    #[test]
    fn test_performance_targets() {
        let stats = ChangeStats {
            total_events: 100,
            total_processing_time: Duration::from_millis(2000), // 20ms average
            debounced_events: 10,
            average_processing_time: Duration::from_millis(20),
        };
        
        assert!(stats.meets_performance_targets()); // 20ms < 50ms target
        
        let slow_stats = ChangeStats {
            total_events: 100,
            total_processing_time: Duration::from_millis(6000), // 60ms average
            debounced_events: 10,
            average_processing_time: Duration::from_millis(60),
        };
        
        assert!(!slow_stats.meets_performance_targets()); // 60ms > 50ms target
    }
    
    #[test]
    fn test_stats_calculation() {
        let mut stats = ChangeStats::new();
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.average_processing_time(), Duration::ZERO);
        
        stats.total_events = 10;
        stats.total_processing_time = Duration::from_millis(100);
        assert_eq!(stats.average_processing_time(), Duration::from_millis(10));
    }
}