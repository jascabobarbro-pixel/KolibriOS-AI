//! File Index
//!
//! Fast file indexing and search for the Adaptive File Manager.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::{FileInfo, FilePermissions, FileManagerError};

/// File Index for fast searching
pub struct FileIndex {
    /// Files by path
    files: Arc<RwLock<HashMap<PathBuf, FileInfo>>>,
    
    /// Name index for fast name search
    name_index: Arc<RwLock<HashMap<String, HashSet<PathBuf>>>>,
    
    /// Extension index
    ext_index: Arc<RwLock<HashMap<String, HashSet<PathBuf>>>>,
    
    /// Tag index
    tag_index: Arc<RwLock<HashMap<String, HashSet<PathBuf>>>>,
    
    /// Content hash index (for deduplication)
    hash_index: Arc<RwLock<HashMap<String, PathBuf>>>,
    
    /// Configuration
    config: IndexConfig,
    
    /// Statistics
    stats: IndexStats,
}

#[derive(Debug, Clone)]
pub struct IndexConfig {
    pub max_files: usize,
    pub index_content: bool,
    pub content_preview_length: usize,
    pub follow_symlinks: bool,
    pub ignore_patterns: Vec<String>,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            max_files: 1000000,
            index_content: false,
            content_preview_length: 256,
            follow_symlinks: false,
            ignore_patterns: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "target".to_string(),
                "__pycache__".to_string(),
                ".venv".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    pub total_files: usize,
    pub total_directories: usize,
    pub total_size: u64,
    pub last_update: Option<Instant>,
    pub index_duration_ms: u64,
}

/// Search result with relevance
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file: FileInfo,
    pub relevance: f32,
    pub match_type: MatchType,
}

#[derive(Debug, Clone)]
pub enum MatchType {
    ExactName,
    PartialName,
    Extension,
    Path,
    Tag,
    Content,
}

/// Suggested file
#[derive(Debug, Clone)]
pub struct SuggestedFile {
    pub path: PathBuf,
    pub name: String,
    pub relevance: f32,
    pub reason: SuggestionReason,
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,
    pub preview: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum SuggestionReason {
    RecentAccess,
    FrequentlyUsed,
    ContextuallyRelevant,
    TimeBasedPattern,
    CollaboratorFile,
    Predicted,
    SearchResult,
}

impl FileIndex {
    pub fn new() -> Self {
        Self {
            files: Arc::new(RwLock::new(HashMap::new())),
            name_index: Arc::new(RwLock::new(HashMap::new())),
            ext_index: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
            hash_index: Arc::new(RwLock::new(HashMap::new())),
            config: IndexConfig::default(),
            stats: IndexStats::default(),
        }
    }
    
    /// Index a directory
    pub async fn index_directory(&mut self, path: &PathBuf) -> Result<u64, FileManagerError> {
        let start = Instant::now();
        let mut count = 0u64;
        
        info!("Indexing directory: {}", path.display());
        
        use walkdir::WalkDir;
        
        let walker = WalkDir::new(path)
            .follow_links(self.config.follow_symlinks)
            .into_iter()
            .filter_entry(|e| !self.should_ignore(e.path()));
        
        for entry in walker {
            match entry {
                Ok(entry) => {
                    let path = entry.path().to_path_buf();
                    
                    if entry.file_type().is_file() {
                        if let Ok(info) = self.create_file_info(&path).await {
                            self.add_file_internal(info).await;
                            count += 1;
                        }
                    } else if entry.file_type().is_dir() {
                        self.stats.total_directories += 1;
                    }
                    
                    // Progress logging
                    if count % 1000 == 0 {
                        debug!("Indexed {} files...", count);
                    }
                }
                Err(e) => {
                    warn!("Error walking directory: {}", e);
                }
            }
        }
        
        self.stats.last_update = Some(Instant::now());
        self.stats.index_duration_ms = start.elapsed().as_millis() as u64;
        
        info!("Indexed {} files in {}ms", count, self.stats.index_duration_ms);
        
        Ok(count)
    }
    
    /// Check if path should be ignored
    fn should_ignore(&self, path: &std::path::Path) -> bool {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            self.config.ignore_patterns.iter().any(|p| name == p || name.starts_with('.'))
        } else {
            false
        }
    }
    
    /// Create file info from path
    async fn create_file_info(&self, path: &PathBuf) -> Result<FileInfo, FileManagerError> {
        let metadata = std::fs::metadata(path)?;
        
        let created = metadata.created()
            .map(|t| chrono::DateTime::from(t))
            .unwrap_or_else(|_| chrono::Utc::now());
        
        let modified = metadata.modified()
            .map(|t| chrono::DateTime::from(t))
            .unwrap_or_else(|_| chrono::Utc::now());
        
        let accessed = metadata.accessed()
            .map(|t| chrono::DateTime::from(t))
            .unwrap_or_else(|_| chrono::Utc::now());
        
        let permissions = FilePermissions {
            readable: true, // Would check actual permissions
            writable: !metadata.permissions().readonly(),
            executable: false,
        };
        
        let mime_type = mime_guess::from_path(path)
            .first()
            .map(|m| m.to_string());
        
        Ok(FileInfo {
            path: path.clone(),
            name: path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
            extension: path.extension()
                .map(|e| e.to_string_lossy().to_lowercase()),
            size: metadata.len(),
            created,
            modified,
            accessed,
            is_directory: metadata.is_dir(),
            permissions,
            mime_type,
            hash: None,
            tags: Vec::new(),
            access_count: 0,
        })
    }
    
    /// Add a file to the index
    pub async fn add_file(&mut self, path: &PathBuf) -> Result<(), FileManagerError> {
        let info = self.create_file_info(path).await?;
        self.add_file_internal(info).await;
        Ok(())
    }
    
    async fn add_file_internal(&self, info: FileInfo) {
        let mut files = self.files.write().await;
        let mut name_index = self.name_index.write().await;
        let mut ext_index = self.ext_index.write().await;
        
        let path = info.path.clone();
        let name = info.name.to_lowercase();
        
        // Add to main index
        files.insert(path.clone(), info.clone());
        
        // Add to name index
        name_index
            .entry(name.clone())
            .or_insert_with(HashSet::new)
            .insert(path.clone());
        
        // Add to extension index
        if let Some(ext) = &info.extension {
            ext_index
                .entry(ext.clone())
                .or_insert_with(HashSet::new)
                .insert(path.clone());
        }
        
        // Update stats
        self.stats.total_files = files.len();
        self.stats.total_size += info.size;
    }
    
    /// Update a file in the index
    pub async fn update_file(&mut self, path: &PathBuf) -> Result<(), FileManagerError> {
        // Remove old entry
        self.remove_file_internal(path).await;
        
        // Add new entry
        self.add_file(path).await
    }
    
    /// Update access time for a file
    pub async fn update_access_time(&mut self, path: &PathBuf) -> Result<(), FileManagerError> {
        let mut files = self.files.write().await;
        
        if let Some(info) = files.get_mut(path) {
            info.accessed = chrono::Utc::now();
            info.access_count += 1;
        }
        
        Ok(())
    }
    
    /// Remove a file from the index
    pub async fn remove_file(&mut self, path: &PathBuf) -> Result<(), FileManagerError> {
        self.remove_file_internal(path).await;
        Ok(())
    }
    
    async fn remove_file_internal(&self, path: &PathBuf) {
        let mut files = self.files.write().await;
        let mut name_index = self.name_index.write().await;
        let mut ext_index = self.ext_index.write().await;
        
        if let Some(info) = files.remove(path) {
            // Remove from name index
            if let Some(paths) = name_index.get_mut(&info.name.to_lowercase()) {
                paths.remove(path);
            }
            
            // Remove from extension index
            if let Some(ext) = &info.extension {
                if let Some(paths) = ext_index.get_mut(ext) {
                    paths.remove(path);
                }
            }
            
            self.stats.total_size -= info.size;
        }
        
        self.stats.total_files = files.len();
    }
    
    /// Search for files
    pub async fn search(&self, query: &str) -> Result<Vec<FileInfo>, FileManagerError> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        
        let files = self.files.read().await;
        let name_index = self.name_index.read().await;
        let ext_index = self.ext_index.read().await;
        
        // Exact name match
        if let Some(paths) = name_index.get(&query_lower) {
            for path in paths {
                if let Some(info) = files.get(path) {
                    results.push(info.clone());
                }
            }
        }
        
        // Partial name match
        for (name, paths) in name_index.iter() {
            if name.contains(&query_lower) && results.len() < 50 {
                for path in paths {
                    if let Some(info) = files.get(path) {
                        if !results.iter().any(|r: &FileInfo| r.path == *path) {
                            results.push(info.clone());
                        }
                    }
                }
            }
        }
        
        // Extension match
        if results.len() < 50 {
            if let Some(paths) = ext_index.get(&query_lower) {
                for path in paths {
                    if let Some(info) = files.get(path) {
                        if !results.iter().any(|r: &FileInfo| r.path == *path) {
                            results.push(info.clone());
                        }
                    }
                }
            }
        }
        
        // Path match
        if results.len() < 50 {
            for info in files.values() {
                if info.path.to_string_lossy().to_lowercase().contains(&query_lower) {
                    if !results.iter().any(|r: &FileInfo| r.path == info.path) {
                        results.push(info.clone());
                    }
                }
            }
        }
        
        results.truncate(100);
        Ok(results)
    }
    
    /// Find files by last access time
    pub async fn find_by_last_access(&self, threshold: Duration) -> Result<Vec<FileInfo>, FileManagerError> {
        let files = self.files.read().await;
        let now = chrono::Utc::now();
        
        let old_files: Vec<FileInfo> = files
            .values()
            .filter(|f| {
                let age = now.signed_duration_since(f.accessed);
                age.num_seconds() as f64 > threshold.as_secs_f64()
            })
            .cloned()
            .collect();
        
        Ok(old_files)
    }
    
    /// Get all files
    pub async fn get_all_files(&self) -> Result<Vec<FileInfo>, FileManagerError> {
        let files = self.files.read().await;
        Ok(files.values().cloned().collect())
    }
    
    /// Get file by path
    pub async fn get_file(&self, path: &PathBuf) -> Option<FileInfo> {
        let files = self.files.read().await;
        files.get(path).cloned()
    }
    
    /// Get statistics
    pub fn stats(&self) -> &IndexStats {
        &self.stats
    }
    
    /// Get total file count
    pub fn file_count(&self) -> usize {
        self.stats.total_files
    }
}

impl Default for FileIndex {
    fn default() -> Self {
        Self::new()
    }
}
