//! File system change monitoring for development mode

use crate::error::Result;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

/// File system change monitor for development mode
pub struct ChangeMonitor {
    /// File system watcher
    watcher: Option<RecommendedWatcher>,
    
    /// Channel for receiving file change events
    receiver: Receiver<notify::Result<Event>>,
    
    /// Sender for file change events
    _sender: Sender<notify::Result<Event>>,
    
    /// Paths being watched
    watch_paths: Vec<std::path::PathBuf>,
    
    /// Last change timestamp for debouncing
    last_change: Option<Instant>,
    
    /// Debounce delay in milliseconds
    debounce_delay: Duration,
}

impl ChangeMonitor {
    /// Create a new change monitor for the given paths
    pub fn new(watch_paths: &[std::path::PathBuf]) -> Result<Self> {
        let (sender, receiver) = mpsc::channel();
        
        Ok(Self {
            watcher: None,
            receiver,
            _sender: sender,
            watch_paths: watch_paths.to_vec(),
            last_change: None,
            debounce_delay: Duration::from_millis(50),
        })
    }
    
    /// Start watching for file changes
    pub fn start_watching(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Err(e) = tx.send(res) {
                    eprintln!("Failed to send file change event: {}", e);
                }
            },
            Config::default(),
        )?;
        
        // Watch all configured paths
        for path in &self.watch_paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)?;
            }
        }
        
        self.watcher = Some(watcher);
        
        // Replace the receiver with the new one
        self.receiver = rx;
        
        Ok(())
    }
    
    /// Check for pending file changes (non-blocking)
    pub fn check_changes(&mut self) -> Vec<FileChange> {
        let mut changes = Vec::new();
        
        // Process all pending events
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
        
        // Apply debouncing
        if !changes.is_empty() {
            let now = Instant::now();
            if let Some(last_change) = self.last_change {
                if now.duration_since(last_change) < self.debounce_delay {
                    // Too soon, ignore these changes
                    return Vec::new();
                }
            }
            self.last_change = Some(now);
        }
        
        changes
    }
    
    /// Process a file system event into a file change
    fn process_event(&self, event: Event) -> Option<FileChange> {
        use notify::EventKind;
        
        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                if let Some(path) = event.paths.first() {
                    if self.is_relevant_file(path) {
                        return Some(FileChange {
                            path: path.clone(),
                            change_type: ChangeType::Modified,
                            timestamp: Instant::now(),
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
                        });
                    }
                }
            }
            _ => {}
        }
        
        None
    }
    
    /// Check if a file is relevant for hot reload
    fn is_relevant_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            match extension.to_str() {
                Some("rs") | Some("toml") | Some("json") => true,
                _ => false,
            }
        } else {
            false
        }
    }
}

/// Information about a file change
#[derive(Debug, Clone)]
pub struct FileChange {
    /// Path to the changed file
    pub path: std::path::PathBuf,
    
    /// Type of change
    pub change_type: ChangeType,
    
    /// When the change occurred
    pub timestamp: Instant,
}

/// Types of file changes
#[derive(Debug, Clone)]
pub enum ChangeType {
    Modified,
    Created,
    Deleted,
}