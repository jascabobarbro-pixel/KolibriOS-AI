//! File Suggestions
//!
//! Context-aware file suggestions based on user behavior and activity.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use super::{UserContext, FileInfo, FileManagerError};
use super::file_index::FileIndex;

/// File Suggester
pub struct FileSuggester {
    /// Recent file accesses
    recent_files: VecDeque<RecentFile>,
    
    /// File access counts
    access_counts: HashMap<PathBuf, u32>,
    
    /// File co-occurrence (files opened together)
    co_occurrences: HashMap<PathBuf, HashMap<PathBuf, u32>>,
    
    /// Time-based patterns
    time_patterns: HashMap<PathBuf, TimePattern>,
    
    /// Configuration
    config: SuggesterConfig,
}

#[derive(Debug, Clone)]
pub struct RecentFile {
    pub path: PathBuf,
    pub accessed_at: Instant,
    pub context: Option<String>,
    pub interaction_type: super::InteractionType,
}

#[derive(Debug, Clone)]
pub struct TimePattern {
    pub hour_accesses: [u32; 24],
    pub day_accesses: [u32; 7],
    pub last_access: Instant,
}

#[derive(Debug, Clone)]
pub struct SuggesterConfig {
    pub max_recent_files: usize,
    pub recent_weight: f32,
    pub frequency_weight: f32,
    pub context_weight: f32,
    pub time_weight: f32,
}

impl Default for SuggesterConfig {
    fn default() -> Self {
        Self {
            max_recent_files: 100,
            recent_weight: 0.35,
            frequency_weight: 0.25,
            context_weight: 0.25,
            time_weight: 0.15,
        }
    }
}

/// Suggested file with relevance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedFile {
    pub path: PathBuf,
    pub name: String,
    pub relevance: f32,
    pub reason: SuggestionReason,
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,
    pub preview: Option<String>,
}

/// Reason for suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionReason {
    RecentAccess,
    FrequentlyUsed,
    ContextuallyRelevant,
    TimeBasedPattern,
    CollaboratorFile,
    Predicted,
    SearchResult,
}

impl FileSuggester {
    pub fn new() -> Self {
        Self {
            recent_files: VecDeque::with_capacity(100),
            access_counts: HashMap::new(),
            co_occurrences: HashMap::new(),
            time_patterns: HashMap::new(),
            config: SuggesterConfig::default(),
        }
    }
    
    /// Record file access
    pub fn record_access(&mut self, path: &PathBuf) {
        // Update recent files
        self.recent_files.push_front(RecentFile {
            path: path.clone(),
            accessed_at: Instant::now(),
            context: None,
            interaction_type: super::InteractionType::Opened,
        });
        
        // Trim if needed
        while self.recent_files.len() > self.config.max_recent_files {
            self.recent_files.pop_back();
        }
        
        // Update access count
        *self.access_counts.entry(path.clone()).or_insert(0) += 1;
        
        // Update time pattern
        let now = chrono::Local::now();
        let pattern = self.time_patterns.entry(path.clone()).or_insert(TimePattern {
            hour_accesses: [0; 24],
            day_accesses: [0; 7],
            last_access: Instant::now(),
        });
        pattern.hour_accesses[now.hour() as usize] += 1;
        pattern.day_accesses[now.weekday().num_days_from_monday() as usize] += 1;
        pattern.last_access = Instant::now();
        
        // Update co-occurrences
        self.update_co_occurrences(path);
    }
    
    /// Update co-occurrence patterns
    fn update_co_occurrences(&mut self, path: &PathBuf) {
        // Look at recent files as co-occurring
        let recent_paths: Vec<PathBuf> = self.recent_files
            .iter()
            .take(5)
            .filter(|r| &r.path != path)
            .map(|r| r.path.clone())
            .collect();
        
        for co_path in recent_paths {
            *self.co_occurrences
                .entry(path.clone())
                .or_insert_with(HashMap::new)
                .entry(co_path)
                .or_insert(0) += 1;
        }
    }
    
    /// Get recent files
    pub fn get_recent_files(&self, context: &UserContext, max: usize) -> Vec<SuggestedFile> {
        self.recent_files
            .iter()
            .take(max)
            .map(|rf| SuggestedFile {
                path: rf.path.clone(),
                name: rf.path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                relevance: self.calculate_recent_relevance(rf),
                reason: SuggestionReason::RecentAccess,
                last_accessed: Some(chrono::DateTime::from(rf.accessed_at)),
                preview: None,
            })
            .collect()
    }
    
    /// Get contextual files based on current context
    pub async fn get_contextual_files(
        &self,
        context: &UserContext,
        index: &FileIndex,
        max: usize,
    ) -> Result<Vec<SuggestedFile>, FileManagerError> {
        let mut suggestions = Vec::new();
        
        // Get files related to current activity
        if let Some(current_file) = &context.active_file {
            // Find co-occurring files
            if let Some(co_occur) = self.co_occurrences.get(current_file) {
                for (co_path, count) in co_occur {
                    if suggestions.len() >= max {
                        break;
                    }
                    
                    let relevance = (*count as f32 / 10.0).min(1.0) * self.config.context_weight;
                    
                    suggestions.push(SuggestedFile {
                        path: co_path.clone(),
                        name: co_path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        relevance,
                        reason: SuggestionReason::ContextuallyRelevant,
                        last_accessed: None,
                        preview: None,
                    });
                }
            }
        }
        
        // Sort by relevance
        suggestions.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        suggestions.truncate(max);
        
        Ok(suggestions)
    }
    
    /// Get files based on time patterns
    pub fn get_time_based_files(&self, max: usize) -> Vec<SuggestedFile> {
        let now = chrono::Local::now();
        let current_hour = now.hour() as usize;
        let current_day = now.weekday().num_days_from_monday() as usize;
        
        let mut suggestions: Vec<SuggestedFile> = self.time_patterns
            .iter()
            .filter_map(|(path, pattern)| {
                // Calculate time-based relevance
                let hour_score = pattern.hour_accesses[current_hour] as f32
                    / pattern.hour_accesses.iter().sum::<u32>() as f32;
                let day_score = pattern.day_accesses[current_day] as f32
                    / pattern.day_accesses.iter().sum::<u32>() as f32;
                
                // Boost if accessed recently
                let recency_boost = if pattern.last_access.elapsed() < Duration::from_hours(1) {
                    1.5
                } else if pattern.last_access.elapsed() < Duration::from_hours(24) {
                    1.2
                } else {
                    1.0
                };
                
                let relevance = ((hour_score + day_score) / 2.0 * self.config.time_weight * recency_boost)
                    .min(1.0);
                
                if relevance > 0.1 {
                    Some(SuggestedFile {
                        path: path.clone(),
                        name: path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        relevance,
                        reason: SuggestionReason::TimeBasedPattern,
                        last_accessed: Some(chrono::DateTime::from(pattern.last_access)),
                        preview: None,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        suggestions.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        suggestions.truncate(max);
        
        suggestions
    }
    
    /// Calculate relevance for recent file
    fn calculate_recent_relevance(&self, rf: &RecentFile) -> f32 {
        let age = rf.accessed_at.elapsed();
        let age_hours = age.as_secs_f32() / 3600.0;
        
        // Exponential decay based on age
        let recency = (-age_hours / 8.0).exp() * self.config.recent_weight;
        
        // Boost by frequency
        let frequency = self.access_counts.get(&rf.path).copied().unwrap_or(1) as f32;
        let frequency_boost = (frequency.log2() / 10.0).min(0.3);
        
        (recency + frequency_boost).min(1.0)
    }
    
    /// Get frequently used files
    pub fn get_frequent_files(&self, max: usize) -> Vec<SuggestedFile> {
        let mut frequent: Vec<_> = self.access_counts
            .iter()
            .map(|(path, count)| {
                let relevance = (*count as f32 / 100.0).min(1.0) * self.config.frequency_weight;
                SuggestedFile {
                    path: path.clone(),
                    name: path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    relevance,
                    reason: SuggestionReason::FrequentlyUsed,
                    last_accessed: self.time_patterns.get(path)
                        .map(|p| chrono::DateTime::from(p.last_access)),
                    preview: None,
                }
            })
            .collect();
        
        frequent.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        frequent.truncate(max);
        
        frequent
    }
}

impl Default for FileSuggester {
    fn default() -> Self {
        Self::new()
    }
}
