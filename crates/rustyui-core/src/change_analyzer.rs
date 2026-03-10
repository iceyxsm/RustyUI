//! Advanced change type analysis and intelligent filtering for 2026

use crate::change_monitor::{FileChange, ChangeType, ChangePriority};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Advanced change analyzer with 2026 AI-inspired filtering
pub struct ChangeAnalyzer {
    /// File pattern rules for intelligent categorization
    pattern_rules: HashMap<String, ChangeCategory>,
    
    /// Content-based analysis cache
    content_cache: HashMap<PathBuf, FileMetadata>,
    
    /// Project structure understanding
    project_structure: ProjectStructure,
    
    /// Performance metrics
    analysis_count: u64,
}

impl ChangeAnalyzer {
    /// Create a new change analyzer with 2026 intelligence
    pub fn new() -> Self {
        let mut pattern_rules = HashMap::new();
        
        // UI Component patterns (highest priority)
        pattern_rules.insert("*.rs".to_string(), ChangeCategory::UIComponent);
        pattern_rules.insert("**/components/**/*.rs".to_string(), ChangeCategory::UIComponent);
        pattern_rules.insert("**/ui/**/*.rs".to_string(), ChangeCategory::UIComponent);
        pattern_rules.insert("**/widgets/**/*.rs".to_string(), ChangeCategory::UIComponent);
        
        // Configuration patterns (high prio