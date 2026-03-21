//! Context Analyzer
//!
//! Analyzes user context and learns patterns for better file suggestions.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use super::{FileInfo, InteractionType, FileManagerError};

/// Context Analyzer
pub struct ContextAnalyzer {
    /// User activity history
    activity_history: VecDeque<Activity>,
    
    /// Learned patterns
    patterns: HashMap<String, Pattern>,
    
    /// Session context
    current_session: SessionContext,
    
    /// Configuration
    config: AnalyzerConfig,
    
    /// Dirty flag for saving
    dirty: bool,
}

#[derive(Debug, Clone)]
pub struct Activity {
    pub timestamp: Instant,
    pub activity_type: ActivityType,
    pub file: Option<PathBuf>,
    pub context: Option<String>,
    pub duration: Option<Duration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    FileOpen,
    FileEdit,
    FileSave,
    FileClose,
    AppSwitch,
    Search,
    Command,
    Creative,
    Navigation,
    Idle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub occurrences: u32,
    pub last_occurrence: chrono::DateTime<chrono::Utc>,
    pub confidence: f32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// Files often opened together
    FileSequence { files: Vec<PathBuf> },
    
    /// Time-based file access
    TimeBased { hour: u8, files: Vec<PathBuf> },
    
    /// Context-based file access
    ContextBased { context: String, files: Vec<PathBuf> },
    
    /// Workflow pattern
    Workflow { name: String, steps: Vec<WorkflowStep> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub action: String,
    pub file: Option<PathBuf>,
    pub duration_seconds: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct SessionContext {
    pub started_at: Instant,
    pub current_file: Option<PathBuf>,
    pub recent_files: VecDeque<PathBuf>,
    pub active_app: Option<String>,
    pub total_activities: u32,
}

#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    pub max_history: usize,
    pub pattern_threshold: u32,
    pub learning_enabled: bool,
    pub save_interval: Duration,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            max_history: 10000,
            pattern_threshold: 3,
            learning_enabled: true,
            save_interval: Duration::from_secs(60),
        }
    }
}

/// User context for file suggestions
#[derive(Debug, Clone)]
pub struct UserContext {
    pub active_file: Option<PathBuf>,
    pub active_app: Option<String>,
    pub recent_files: Vec<PathBuf>,
    pub current_hour: u8,
    pub current_day: chrono::Weekday,
    pub search_query: Option<String>,
    pub activity_level: f32,
    pub focus_mode: bool,
}

impl UserContext {
    pub fn current() -> Self {
        let now = chrono::Local::now();
        
        Self {
            active_file: None,
            active_app: None,
            recent_files: Vec::new(),
            current_hour: now.hour(),
            current_day: now.weekday(),
            search_query: None,
            activity_level: 0.5,
            focus_mode: false,
        }
    }
}

impl ContextAnalyzer {
    pub fn new() -> Self {
        Self {
            activity_history: VecDeque::with_capacity(10000),
            patterns: HashMap::new(),
            current_session: SessionContext::default(),
            config: AnalyzerConfig::default(),
            dirty: false,
        }
    }
    
    /// Record an activity
    pub async fn record_activity(&mut self, activity: Activity) {
        self.activity_history.push_back(activity.clone());
        
        // Trim if needed
        while self.activity_history.len() > self.config.max_history {
            self.activity_history.pop_front();
        }
        
        // Update session
        self.current_session.total_activities += 1;
        
        // Learn patterns
        if self.config.learning_enabled {
            self.learn_from_activity(&activity);
        }
    }
    
    /// Record a file interaction
    pub async fn record_interaction(
        &mut self,
        path: &PathBuf,
        interaction_type: InteractionType,
    ) -> Result<(), FileManagerError> {
        let activity_type = match interaction_type {
            InteractionType::Opened => ActivityType::FileOpen,
            InteractionType::Edited => ActivityType::FileEdit,
            InteractionType::Saved => ActivityType::FileSave,
            InteractionType::Closed => ActivityType::FileClose,
            _ => ActivityType::FileOpen,
        };
        
        self.current_session.current_file = Some(path.clone());
        self.current_session.recent_files.push_front(path.clone());
        if self.current_session.recent_files.len() > 20 {
            self.current_session.recent_files.pop_back();
        }
        
        self.record_activity(Activity {
            timestamp: Instant::now(),
            activity_type,
            file: Some(path.clone()),
            context: None,
            duration: None,
        }).await;
        
        Ok(())
    }
    
    /// Record a search
    pub async fn record_search(
        &mut self,
        query: &str,
        results: &[FileInfo],
    ) -> Result<(), FileManagerError> {
        self.record_activity(Activity {
            timestamp: Instant::now(),
            activity_type: ActivityType::Search,
            file: results.first().map(|f| f.path.clone()),
            context: Some(query.to_string()),
            duration: None,
        }).await;
        
        Ok(())
    }
    
    /// Learn patterns from activity
    fn learn_from_activity(&mut self, activity: &Activity) {
        // Check for file sequences
        if let Some(file) = &activity.file {
            // Look for patterns in recent activities
            let recent: Vec<_> = self.activity_history
                .iter()
                .rev()
                .take(5)
                .filter_map(|a| a.file.clone())
                .collect();
            
            if recent.len() >= 2 {
                // Check if this file often follows previous files
                let pattern_key = format!("seq:{:?}", recent.first());
                
                self.patterns
                    .entry(pattern_key)
                    .and_modify(|p| {
                        p.occurrences += 1;
                        p.confidence = (p.occurrences as f32 / 10.0).min(1.0);
                    })
                    .or_insert(Pattern {
                        pattern_type: PatternType::FileSequence {
                            files: recent.into_iter().take(3).collect(),
                        },
                        occurrences: 1,
                        last_occurrence: chrono::Utc::now(),
                        confidence: 0.1,
                        metadata: HashMap::new(),
                    });
            }
            
            // Time-based patterns
            let now = chrono::Local::now();
            let hour = now.hour();
            let time_key = format!("time:{}:{:?}", hour, file);
            
            self.patterns
                .entry(time_key)
                .and_modify(|p| {
                    p.occurrences += 1;
                    p.confidence = (p.occurrences as f32 / 5.0).min(1.0);
                })
                .or_insert(Pattern {
                    pattern_type: PatternType::TimeBased {
                        hour,
                        files: vec![file.clone()],
                    },
                    occurrences: 1,
                    last_occurrence: chrono::Utc::now(),
                    confidence: 0.2,
                    metadata: HashMap::new(),
                });
        }
        
        self.dirty = true;
    }
    
    /// Predict next files based on context
    pub fn predict_next_files(&self, context: &UserContext, max: usize) -> Vec<super::SuggestedFile> {
        let mut predictions = Vec::new();
        
        // Time-based predictions
        for (_, pattern) in &self.patterns {
            if let PatternType::TimeBased { hour, files } = &pattern.pattern_type {
                if *hour == context.current_hour && pattern.confidence > 0.3 {
                    for file in files.iter().take(max - predictions.len()) {
                        predictions.push(super::SuggestedFile {
                            path: file.clone(),
                            name: file.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default(),
                            relevance: pattern.confidence,
                            reason: super::SuggestionReason::TimeBasedPattern,
                            last_accessed: None,
                            preview: None,
                        });
                    }
                }
            }
            
            if predictions.len() >= max {
                break;
            }
        }
        
        // Sequence-based predictions
        if let Some(current) = &context.active_file {
            let seq_key = format!("seq:{:?}", current);
            
            if let Some(pattern) = self.patterns.get(&seq_key) {
                if let PatternType::FileSequence { files } = &pattern.pattern_type {
                    for file in files.iter().take(max - predictions.len()) {
                        if !predictions.iter().any(|p| &p.path == file) {
                            predictions.push(super::SuggestedFile {
                                path: file.clone(),
                                name: file.file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_default(),
                                relevance: pattern.confidence * 0.9,
                                reason: super::SuggestionReason::Predicted,
                                last_accessed: None,
                                preview: None,
                            });
                        }
                    }
                }
            }
        }
        
        predictions.truncate(max);
        predictions
    }
    
    /// Extract features from context
    pub fn extract_features(&self, context: &UserContext) -> ContextFeatures {
        let mut features = ContextFeatures::default();
        
        // Time features
        features.hour_of_day = context.current_hour as f32 / 24.0;
        features.is_work_hours = (9..=17).contains(&context.current_hour);
        
        // Activity features
        features.recent_file_count = context.recent_files.len() as f32;
        features.has_active_file = context.active_file.is_some();
        
        // Pattern features
        features.matching_patterns = self.patterns.values()
            .filter(|p| p.confidence > 0.5)
            .count() as f32;
        
        features
    }
    
    /// Load patterns from disk
    pub async fn load_patterns(&mut self) -> Result<(), FileManagerError> {
        let patterns_path = std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".kolibri/file_patterns.json"))
            .unwrap_or_else(|_| PathBuf::from("/tmp/file_patterns.json"));
        
        if patterns_path.exists() {
            let content = std::fs::read_to_string(&patterns_path)?;
            self.patterns = serde_json::from_str(&content)?;
            info!("Loaded {} patterns from disk", self.patterns.len());
        }
        
        Ok(())
    }
    
    /// Save patterns to disk
    pub async fn save_patterns(&self) -> Result<(), FileManagerError> {
        let patterns_path = std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".kolibri/file_patterns.json"))
            .unwrap_or_else(|_| PathBuf::from("/tmp/file_patterns.json"));
        
        // Ensure directory exists
        if let Some(parent) = patterns_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(&self.patterns)?;
        std::fs::write(&patterns_path, content)?;
        
        info!("Saved {} patterns to disk", self.patterns.len());
        
        Ok(())
    }
    
    /// Get current session
    pub fn session(&self) -> &SessionContext {
        &self.current_session
    }
}

impl Default for ContextAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SessionContext {
    fn default() -> Self {
        Self {
            started_at: Instant::now(),
            current_file: None,
            recent_files: VecDeque::with_capacity(20),
            active_app: None,
            total_activities: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ContextFeatures {
    pub hour_of_day: f32,
    pub is_work_hours: bool,
    pub recent_file_count: f32,
    pub has_active_file: bool,
    pub matching_patterns: f32,
}
