//! Adaptive File Manager
//!
//! A living application that:
//! - Suggests relevant files based on user context and activity
//! - Optimizes storage based on MemoryCell feedback
//! - Learns from user behavior to improve suggestions

pub mod file_watcher;
pub mod suggestions;
pub mod storage_optimizer;
pub mod context_analyzer;
pub mod grpc_client;
pub mod file_index;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub use file_watcher::FileWatcher;
pub use suggestions::{FileSuggester, SuggestedFile, SuggestionReason};
pub use storage_optimizer::{StorageOptimizer, OptimizationAction, StorageStats};
pub use context_analyzer::{ContextAnalyzer, UserContext, ActivityType};
pub use file_index::FileIndex;

/// Main Adaptive File Manager
pub struct AdaptiveFileManager {
    /// File watcher for real-time changes
    watcher: FileWatcher,
    
    /// File suggester for context-aware suggestions
    suggester: FileSuggester,
    
    /// Storage optimizer for memory-based optimization
    optimizer: StorageOptimizer,
    
    /// Context analyzer for learning user behavior
    context_analyzer: ContextAnalyzer,
    
    /// File index for fast searching
    index: FileIndex,
    
    /// Configuration
    config: FileManagerConfig,
    
    /// Statistics
    stats: FileManagerStats,
}

/// File Manager Configuration
#[derive(Debug, Clone)]
pub struct FileManagerConfig {
    /// Root directories to watch
    pub watch_paths: Vec<PathBuf>,
    
    /// Enable file suggestions
    pub suggestions_enabled: bool,
    
    /// Enable storage optimization
    pub optimization_enabled: bool,
    
    /// Maximum suggestions to show
    pub max_suggestions: usize,
    
    /// Index update interval
    pub index_update_interval: Duration,
    
    /// Memory cell endpoint
    pub memory_cell_endpoint: String,
    
    /// Unified Mind endpoint
    pub mind_endpoint: String,
    
    /// Enable learning from user behavior
    pub learning_enabled: bool,
}

impl Default for FileManagerConfig {
    fn default() -> Self {
        Self {
            watch_paths: vec![PathBuf::from("/home")],
            suggestions_enabled: true,
            optimization_enabled: true,
            max_suggestions: 10,
            index_update_interval: Duration::from_secs(60),
            memory_cell_endpoint: "http://localhost:50051".to_string(),
            mind_endpoint: "http://localhost:50052".to_string(),
            learning_enabled: true,
        }
    }
}

/// File Manager Statistics
#[derive(Debug, Clone, Default)]
pub struct FileManagerStats {
    pub files_indexed: u64,
    pub suggestions_made: u64,
    pub suggestions_accepted: u64,
    pub optimizations_performed: u64,
    pub space_saved_bytes: u64,
    pub avg_suggestion_relevance: f32,
    pub last_index_update: Option<Instant>,
    pub total_searches: u64,
}

impl AdaptiveFileManager {
    /// Create a new Adaptive File Manager
    pub fn new(config: FileManagerConfig) -> Self {
        Self {
            watcher: FileWatcher::new(&config.watch_paths),
            suggester: FileSuggester::new(),
            optimizer: StorageOptimizer::new(&config.memory_cell_endpoint),
            context_analyzer: ContextAnalyzer::new(),
            index: FileIndex::new(),
            config,
            stats: FileManagerStats::default(),
        }
    }
    
    /// Initialize the file manager
    pub async fn initialize(&mut self) -> Result<(), FileManagerError> {
        info!("Initializing Adaptive File Manager");
        
        // Build initial file index
        self.build_index().await?;
        
        // Start file watcher
        self.watcher.start().await?;
        
        // Load learned patterns
        if self.config.learning_enabled {
            self.context_analyzer.load_patterns().await?;
        }
        
        info!("Adaptive File Manager initialized successfully");
        Ok(())
    }
    
    /// Build or rebuild the file index
    async fn build_index(&mut self) -> Result<(), FileManagerError> {
        info!("Building file index for {:?} paths", self.config.watch_paths);
        
        let mut total_files = 0;
        
        for path in &self.config.watch_paths {
            if path.exists() {
                let count = self.index.index_directory(path).await?;
                total_files += count;
            }
        }
        
        self.stats.files_indexed = total_files;
        self.stats.last_index_update = Some(Instant::now());
        
        info!("Indexed {} files", total_files);
        Ok(())
    }
    
    /// Get file suggestions based on current context
    pub async fn get_suggestions(&mut self, context: &UserContext) -> Result<Vec<SuggestedFile>, FileManagerError> {
        if !self.config.suggestions_enabled {
            return Ok(Vec::new());
        }
        
        // Analyze context
        let context_features = self.context_analyzer.extract_features(context);
        
        // Get suggestions from multiple sources
        let mut suggestions = Vec::new();
        
        // 1. Recent files
        let recent = self.suggester.get_recent_files(context, self.config.max_suggestions / 2);
        suggestions.extend(recent);
        
        // 2. Contextually relevant files
        let contextual = self.suggester.get_contextual_files(
            context,
            &self.index,
            self.config.max_suggestions / 2,
        ).await?;
        suggestions.extend(contextual);
        
        // 3. Predicted files based on patterns
        if self.config.learning_enabled {
            let predicted = self.context_analyzer.predict_next_files(context, 3);
            suggestions.extend(predicted);
        }
        
        // Remove duplicates and sort by relevance
        suggestions.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        suggestions.dedup_by(|a, b| a.path == b.path);
        suggestions.truncate(self.config.max_suggestions);
        
        self.stats.suggestions_made += suggestions.len() as u64;
        
        Ok(suggestions)
    }
    
    /// Search for files
    pub async fn search(&mut self, query: &str) -> Result<Vec<FileInfo>, FileManagerError> {
        self.stats.total_searches += 1;
        
        let results = self.index.search(query).await?;
        
        // Learn from search
        if self.config.learning_enabled {
            self.context_analyzer.record_search(query, &results).await?;
        }
        
        Ok(results)
    }
    
    /// Record user interaction with a file
    pub async fn record_file_interaction(
        &mut self,
        path: &PathBuf,
        interaction_type: InteractionType,
    ) -> Result<(), FileManagerError> {
        // Update file access time
        self.index.update_access_time(path).await?;
        
        // Learn from interaction
        if self.config.learning_enabled {
            self.context_analyzer.record_interaction(path, interaction_type).await?;
        }
        
        // Update suggestions based on this
        self.suggester.record_access(path);
        
        Ok(())
    }
    
    /// Optimize storage based on MemoryCell feedback
    pub async fn optimize_storage(&mut self) -> Result<OptimizationReport, FileManagerError> {
        if !self.config.optimization_enabled {
            return Ok(OptimizationReport::default());
        }
        
        info!("Starting storage optimization");
        
        // Get memory pressure from MemoryCell
        let memory_pressure = self.optimizer.get_memory_pressure().await?;
        
        let mut report = OptimizationReport::default();
        
        if memory_pressure > 0.8 {
            // High memory pressure - aggressive optimization
            warn!("High memory pressure: {:.1}% - performing aggressive optimization", memory_pressure * 100.0);
            
            // Find large unused files
            let unused = self.optimizer.find_unused_files(&self.index).await?;
            
            for file in unused.iter().take(10) {
                match self.optimizer.optimize_file(file).await {
                    Ok(saved) => {
                        report.files_optimized += 1;
                        report.space_saved += saved;
                        self.stats.optimizations_performed += 1;
                        self.stats.space_saved_bytes += saved;
                    }
                    Err(e) => {
                        warn!("Failed to optimize file {:?}: {}", file.path, e);
                        report.errors.push(format!("{}: {}", file.path.display(), e));
                    }
                }
            }
        } else if memory_pressure > 0.6 {
            // Medium memory pressure - moderate optimization
            debug!("Medium memory pressure: {:.1}% - performing moderate optimization", memory_pressure * 100.0);
            
            // Compress old files
            let old_files = self.optimizer.find_old_files(&self.index, Duration::from_secs(30 * 24 * 3600)).await?;
            
            for file in old_files.iter().take(5) {
                match self.optimizer.compress_file(file).await {
                    Ok(saved) => {
                        report.files_optimized += 1;
                        report.space_saved += saved;
                    }
                    Err(e) => {
                        report.errors.push(format!("{}: {}", file.path.display(), e));
                    }
                }
            }
        }
        
        report.memory_pressure_before = memory_pressure;
        report.memory_pressure_after = self.optimizer.get_memory_pressure().await?;
        
        info!("Storage optimization complete: {} files optimized, {} bytes saved", 
              report.files_optimized, report.space_saved);
        
        Ok(report)
    }
    
    /// Handle file system event
    pub async fn handle_fs_event(&mut self, event: FileEvent) -> Result<(), FileManagerError> {
        match event {
            FileEvent::Created(path) => {
                self.index.add_file(&path).await?;
            }
            FileEvent::Modified(path) => {
                self.index.update_file(&path).await?;
            }
            FileEvent::Deleted(path) => {
                self.index.remove_file(&path).await?;
            }
            FileEvent::Renamed { old_path, new_path } => {
                self.index.remove_file(&old_path).await?;
                self.index.add_file(&new_path).await?;
            }
        }
        
        Ok(())
    }
    
    /// Get current statistics
    pub fn stats(&self) -> &FileManagerStats {
        &self.stats
    }
    
    /// Shutdown the file manager
    pub async fn shutdown(&mut self) -> Result<(), FileManagerError> {
        info!("Shutting down Adaptive File Manager");
        
        // Stop file watcher
        self.watcher.stop().await?;
        
        // Save learned patterns
        if self.config.learning_enabled {
            self.context_analyzer.save_patterns().await?;
        }
        
        info!("Adaptive File Manager shutdown complete");
        Ok(())
    }
}

/// File event types
#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
    Renamed { old_path: PathBuf, new_path: PathBuf },
}

/// File interaction types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionType {
    Opened,
    Edited,
    Saved,
    Closed,
    Deleted,
    Moved,
    Copied,
    Renamed,
}

/// File information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub accessed: chrono::DateTime<chrono::Utc>,
    pub is_directory: bool,
    pub permissions: FilePermissions,
    pub mime_type: Option<String>,
    pub hash: Option<String>,
    pub tags: Vec<String>,
    pub access_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePermissions {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

/// Optimization report
#[derive(Debug, Clone, Default)]
pub struct OptimizationReport {
    pub files_optimized: u64,
    pub space_saved: u64,
    pub memory_pressure_before: f32,
    pub memory_pressure_after: f32,
    pub errors: Vec<String>,
}

/// File Manager Error
#[derive(Debug, thiserror::Error)]
pub enum FileManagerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Index error: {0}")]
    Index(String),
    
    #[error("Watch error: {0}")]
    Watch(String),
    
    #[error("gRPC error: {0}")]
    Grpc(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

impl Default for AdaptiveFileManager {
    fn default() -> Self {
        Self::new(FileManagerConfig::default())
    }
}
