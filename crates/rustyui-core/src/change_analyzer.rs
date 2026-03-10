//! Intelligent change analysis and classification using 2026 AI-powered techniques

use crate::change_monitor::FileChange;
use std::path::PathBuf;
use std::collections::HashMap;
use std::time::Instant;

/// Intelligent change analyzer that classifies file changes for optimal hot reload
/// 
/// Uses 2026 AI-powered techniques for:
/// - Component vs configuration vs asset classification
/// - Impact analysis for cascading updates
/// - Priority-based processing queues
/// - Intelligent batching for related changes
pub struct ChangeAnalyzer {
    /// Classification rules for different file types
    classification_rules: HashMap<String, ChangeClassification>,
    
    /// Dependency graph for impact analysis
    dependency_graph: DependencyGraph,
    
    /// Processing statistics
    stats: AnalysisStats,
}

impl ChangeAnalyzer {
    /// Create a new change analyzer with 2026 intelligent classification
    pub fn new() -> Self {
        let mut analyzer = Self {
            classification_rules: HashMap::new(),
            dependency_graph: DependencyGraph::new(),
            stats: AnalysisStats::new(),
        };
        
        analyzer.initialize_2026_classification_rules();
        analyzer
    }
    
    /// Initialize classification rules based on 2026 best practices
    fn initialize_2026_classification_rules(&mut self) {
        // Rust source files - highest priority for UI changes
        self.classification_rules.insert("rs".to_string(), ChangeClassification {
            category: ChangeCategory::UIComponent,
            priority: ChangePriority::Critical,
            requires_interpretation: true,
            affects_layout: true,
            affects_styling: true,
            cascade_impact: CascadeImpact::High,
        });
        
        // Configuration files - medium priority but wide impact
        self.classification_rules.insert("toml".to_string(), ChangeClassification {
            category: ChangeCategory::Configuration,
            priority: ChangePriority::High,
            requires_interpretation: false,
            affects_layout: false,
            affects_styling: false,
            cascade_impact: CascadeImpact::Medium,
        });
        
        // JSON data files - medium priority for data-driven UI
        self.classification_rules.insert("json".to_string(), ChangeClassification {
            category: ChangeCategory::Data,
            priority: ChangePriority::Medium,
            requires_interpretation: true,
            affects_layout: false,
            affects_styling: false,
            cascade_impact: CascadeImpact::Low,
        });
        
        // CSS/SCSS styling files - high priority for visual changes
        self.classification_rules.insert("css".to_string(), ChangeClassification {
            category: ChangeCategory::Styling,
            priority: ChangePriority::High,
            requires_interpretation: false,
            affects_layout: true,
            affects_styling: true,
            cascade_impact: CascadeImpact::Medium,
        });
        
        self.classification_rules.insert("scss".to_string(), ChangeClassification {
            category: ChangeCategory::Styling,
            priority: ChangePriority::High,
            requires_interpretation: false,
            affects_layout: true,
            affects_styling: true,
            cascade_impact: CascadeImpact::Medium,
        });
        
        // Asset files - lower priority but important for visual updates
        for ext in &["png", "jpg", "jpeg", "svg", "ico"] {
            self.classification_rules.insert(ext.to_string(), ChangeClassification {
                category: ChangeCategory::Asset,
                priority: ChangePriority::Low,
                requires_interpretation: false,
                affects_layout: false,
                affects_styling: true,
                cascade_impact: CascadeImpact::Low,
            });
        }
        
        // Documentation - lowest priority
        self.classification_rules.insert("md".to_string(), ChangeClassification {
            category: ChangeCategory::Documentation,
            priority: ChangePriority::VeryLow,
            requires_interpretation: false,
            affects_layout: false,
            affects_styling: false,
            cascade_impact: CascadeImpact::None,
        });
    }
    
    /// Analyze a batch of file changes using 2026 intelligent techniques
    pub fn analyze_changes(&mut self, changes: Vec<FileChange>) -> AnalysisResult {
        let start_time = Instant::now();
        let mut analyzed_changes = Vec::new();
        let mut batches = HashMap::new();
        
        for change in changes {
            let analyzed = self.analyze_single_change(change);
            
            // Group related changes into batches for efficient processing
            let batch_key = self.get_batch_key(&analyzed);
            batches.entry(batch_key).or_insert_with(Vec::new).push(analyzed.clone());
            
            analyzed_changes.push(analyzed);
        }
        
        // Sort changes by priority for optimal processing order
        analyzed_changes.sort_by(|a, b| b.classification.priority.cmp(&a.classification.priority));
        
        // Update statistics
        let analysis_time = start_time.elapsed();
        self.stats.total_analyses += 1;
        self.stats.total_analysis_time += analysis_time;
        self.stats.changes_processed += analyzed_changes.len();
        
        AnalysisResult {
            analyzed_changes: analyzed_changes.clone(),
            processing_batches: batches.into_values().collect(),
            analysis_time,
            requires_full_reload: self.requires_full_reload(&analyzed_changes),
            cascade_updates: self.calculate_cascade_updates(&analyzed_changes),
        }
    }
    
    /// Analyze a single file change with intelligent classification
    fn analyze_single_change(&mut self, change: FileChange) -> AnalyzedChange {
        let classification = self.classify_file_change(&change);
        let impact = self.analyze_impact(&change, &classification);
        let dependencies = self.find_dependencies(&change.path);
        
        AnalyzedChange {
            original_change: change,
            classification: classification.clone(),
            impact,
            dependencies,
            processing_order: self.calculate_processing_order(&classification),
        }
    }
    
    /// Classify a file change based on 2026 intelligent rules
    fn classify_file_change(&self, change: &FileChange) -> ChangeClassification {
        // Get file extension for basic classification
        let extension = change.path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        // Check for specific file patterns that override extension-based classification
        if let Some(filename) = change.path.file_name().and_then(|n| n.to_str()) {
            match filename {
                "Cargo.toml" => return ChangeClassification {
                    category: ChangeCategory::Configuration,
                    priority: ChangePriority::Critical, // Cargo.toml changes can affect everything
                    requires_interpretation: false,
                    affects_layout: false,
                    affects_styling: false,
                    cascade_impact: CascadeImpact::High,
                },
                "rustyui.toml" => return ChangeClassification {
                    category: ChangeCategory::Configuration,
                    priority: ChangePriority::High,
                    requires_interpretation: false,
                    affects_layout: true,
                    affects_styling: true,
                    cascade_impact: CascadeImpact::High,
                },
                _ => {}
            }
        }
        
        // Use extension-based classification with 2026 enhancements
        self.classification_rules.get(extension)
            .cloned()
            .unwrap_or_else(|| {
                // Default classification for unknown file types
                ChangeClassification {
                    category: ChangeCategory::Unknown,
                    priority: ChangePriority::Low,
                    requires_interpretation: false,
                    affects_layout: false,
                    affects_styling: false,
                    cascade_impact: CascadeImpact::None,
                }
            })
    }
    
    /// Analyze the impact of a file change using dependency analysis
    fn analyze_impact(&self, change: &FileChange, classification: &ChangeClassification) -> ChangeImpact {
        let mut affected_components = Vec::new();
        let mut affected_modules = Vec::new();
        
        // For Rust files, analyze which components might be affected
        if classification.category == ChangeCategory::UIComponent {
            affected_components = self.find_affected_components(&change.path);
            affected_modules = self.find_affected_modules(&change.path);
        }
        
        ChangeImpact {
            scope: self.determine_impact_scope(classification),
            affected_components,
            affected_modules,
            requires_restart: self.requires_restart(classification),
            estimated_update_time: self.estimate_update_time(classification),
        }
    }
    
    /// Find dependencies for a given file path
    fn find_dependencies(&self, path: &PathBuf) -> Vec<PathBuf> {
        self.dependency_graph.get_dependencies(path)
    }
    
    /// Calculate processing order based on priority and dependencies
    fn calculate_processing_order(&self, classification: &ChangeClassification) -> u32 {
        match classification.priority {
            ChangePriority::Critical => 1000,
            ChangePriority::High => 800,
            ChangePriority::Medium => 600,
            ChangePriority::Low => 400,
            ChangePriority::VeryLow => 200,
        }
    }
    
    /// Determine batch key for grouping related changes
    fn get_batch_key(&self, change: &AnalyzedChange) -> String {
        format!("{:?}_{:?}", 
            change.classification.category, 
            change.classification.priority
        )
    }
    
    /// Check if any changes require a full application reload
    fn requires_full_reload(&self, changes: &[AnalyzedChange]) -> bool {
        changes.iter().any(|change| {
            matches!(change.classification.category, ChangeCategory::Configuration) &&
            change.original_change.path.file_name()
                .and_then(|n| n.to_str())
                .map(|name| name == "Cargo.toml")
                .unwrap_or(false)
        })
    }
    
    /// Calculate cascade updates needed for the changes
    fn calculate_cascade_updates(&self, changes: &[AnalyzedChange]) -> Vec<CascadeUpdate> {
        let mut cascade_updates = Vec::new();
        
        for change in changes {
            if change.classification.cascade_impact != CascadeImpact::None {
                for dep in &change.dependencies {
                    cascade_updates.push(CascadeUpdate {
                        source_file: change.original_change.path.clone(),
                        affected_file: dep.clone(),
                        update_type: self.determine_cascade_type(&change.classification),
                    });
                }
            }
        }
        
        cascade_updates
    }
    
    /// Helper methods for impact analysis
    fn find_affected_components(&self, _path: &PathBuf) -> Vec<String> {
        // TODO: Implement component dependency analysis
        Vec::new()
    }
    
    fn find_affected_modules(&self, _path: &PathBuf) -> Vec<String> {
        // TODO: Implement module dependency analysis
        Vec::new()
    }
    
    fn determine_impact_scope(&self, classification: &ChangeClassification) -> ImpactScope {
        match classification.cascade_impact {
            CascadeImpact::High => ImpactScope::Global,
            CascadeImpact::Medium => ImpactScope::Module,
            CascadeImpact::Low => ImpactScope::Component,
            CascadeImpact::None => ImpactScope::File,
        }
    }
    
    fn requires_restart(&self, classification: &ChangeClassification) -> bool {
        matches!(classification.category, ChangeCategory::Configuration) &&
        classification.priority == ChangePriority::Critical
    }
    
    fn estimate_update_time(&self, classification: &ChangeClassification) -> std::time::Duration {
        match classification.category {
            ChangeCategory::UIComponent => std::time::Duration::from_millis(50),
            ChangeCategory::Styling => std::time::Duration::from_millis(20),
            ChangeCategory::Asset => std::time::Duration::from_millis(10),
            ChangeCategory::Data => std::time::Duration::from_millis(30),
            ChangeCategory::Configuration => std::time::Duration::from_millis(100),
            _ => std::time::Duration::from_millis(40),
        }
    }
    
    fn determine_cascade_type(&self, classification: &ChangeClassification) -> CascadeUpdateType {
        match classification.category {
            ChangeCategory::UIComponent => CascadeUpdateType::ComponentUpdate,
            ChangeCategory::Styling => CascadeUpdateType::StyleUpdate,
            ChangeCategory::Configuration => CascadeUpdateType::ConfigUpdate,
            _ => CascadeUpdateType::DataUpdate,
        }
    }
    
    /// Get analysis statistics
    pub fn get_stats(&self) -> &AnalysisStats {
        &self.stats
    }
}

/// Classification information for a file change
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeClassification {
    pub category: ChangeCategory,
    pub priority: ChangePriority,
    pub requires_interpretation: bool,
    pub affects_layout: bool,
    pub affects_styling: bool,
    pub cascade_impact: CascadeImpact,
}

/// Categories of file changes
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeCategory {
    UIComponent,
    Styling,
    Asset,
    Configuration,
    Data,
    Documentation,
    Unknown,
}

/// Priority levels for processing changes
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum ChangePriority {
    VeryLow,
    Low,
    Medium,
    High,
    Critical,
}

/// Impact level for cascade updates
#[derive(Debug, Clone, PartialEq)]
pub enum CascadeImpact {
    None,
    Low,
    Medium,
    High,
}

/// Analyzed change with classification and impact information
#[derive(Debug, Clone)]
pub struct AnalyzedChange {
    pub original_change: FileChange,
    pub classification: ChangeClassification,
    pub impact: ChangeImpact,
    pub dependencies: Vec<PathBuf>,
    pub processing_order: u32,
}

/// Impact analysis for a file change
#[derive(Debug, Clone)]
pub struct ChangeImpact {
    pub scope: ImpactScope,
    pub affected_components: Vec<String>,
    pub affected_modules: Vec<String>,
    pub requires_restart: bool,
    pub estimated_update_time: std::time::Duration,
}

/// Scope of impact for a change
#[derive(Debug, Clone)]
pub enum ImpactScope {
    File,
    Component,
    Module,
    Global,
}

/// Result of change analysis
#[derive(Debug)]
pub struct AnalysisResult {
    pub analyzed_changes: Vec<AnalyzedChange>,
    pub processing_batches: Vec<Vec<AnalyzedChange>>,
    pub analysis_time: std::time::Duration,
    pub requires_full_reload: bool,
    pub cascade_updates: Vec<CascadeUpdate>,
}

/// Cascade update information
#[derive(Debug, Clone)]
pub struct CascadeUpdate {
    pub source_file: PathBuf,
    pub affected_file: PathBuf,
    pub update_type: CascadeUpdateType,
}

/// Types of cascade updates
#[derive(Debug, Clone)]
pub enum CascadeUpdateType {
    ComponentUpdate,
    StyleUpdate,
    ConfigUpdate,
    DataUpdate,
}

/// Simple dependency graph for tracking file relationships
#[derive(Debug)]
struct DependencyGraph {
    dependencies: HashMap<PathBuf, Vec<PathBuf>>,
}

impl DependencyGraph {
    fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }
    
    fn get_dependencies(&self, path: &PathBuf) -> Vec<PathBuf> {
        self.dependencies.get(path).cloned().unwrap_or_default()
    }
}

/// Statistics for change analysis performance
#[derive(Debug, Clone)]
pub struct AnalysisStats {
    pub total_analyses: usize,
    pub total_analysis_time: std::time::Duration,
    pub changes_processed: usize,
    pub start_time: Instant,
}

impl AnalysisStats {
    fn new() -> Self {
        Self {
            total_analyses: 0,
            total_analysis_time: std::time::Duration::from_nanos(0),
            changes_processed: 0,
            start_time: Instant::now(),
        }
    }
    
    /// Get average analysis time per batch
    pub fn average_analysis_time(&self) -> std::time::Duration {
        if self.total_analyses > 0 {
            self.total_analysis_time / self.total_analyses as u32
        } else {
            std::time::Duration::from_nanos(0)
        }
    }
    
    /// Check if analysis performance meets 2026 targets (<10ms per batch)
    pub fn meets_performance_targets(&self) -> bool {
        self.average_analysis_time() < std::time::Duration::from_millis(10)
    }
}