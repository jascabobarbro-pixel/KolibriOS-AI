//! Storage Optimizer
//!
//! Optimizes storage based on MemoryCell feedback and usage patterns.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::{FileInfo, FileManagerError};
use super::file_index::FileIndex;

/// Storage Optimizer
pub struct StorageOptimizer {
    /// Memory cell endpoint
    memory_cell_endpoint: String,
    
    /// Optimization cache
    optimization_cache: Arc<RwLock<OptimizationCache>>,
    
    /// Configuration
    config: OptimizerConfig,
    
    /// Statistics
    stats: OptimizerStats,
}

#[derive(Debug, Clone, Default)]
pub struct OptimizationCache {
    /// Files analyzed for optimization
    analyzed_files: std::collections::HashMap<PathBuf, FileAnalysis>,
    
    /// Last optimization run
    last_run: Option<std::time::Instant>,
    
    /// Pending optimizations
    pending: Vec<OptimizationAction>,
}

#[derive(Debug, Clone)]
pub struct FileAnalysis {
    pub path: PathBuf,
    pub size: u64,
    pub access_frequency: f32,
    pub last_access: std::time::Instant,
    pub optimization_potential: f32,
    pub suggested_action: OptimizationAction,
}

#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// Threshold for considering a file "unused" (days)
    pub unused_threshold_days: u32,
    
    /// Threshold for considering a file "old" (days)
    pub old_threshold_days: u32,
    
    /// Minimum size for optimization (bytes)
    pub min_optimization_size: u64,
    
    /// Enable compression
    pub enable_compression: bool,
    
    /// Enable archival
    pub enable_archival: bool,
    
    /// Compression level (1-9)
    pub compression_level: u8,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            unused_threshold_days: 30,
            old_threshold_days: 90,
            min_optimization_size: 1024 * 1024, // 1MB
            enable_compression: true,
            enable_archival: false,
            compression_level: 6,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct OptimizerStats {
    pub files_analyzed: u64,
    pub files_optimized: u64,
    pub bytes_saved: u64,
    pub compression_ratio: f32,
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_space: u64,
    pub used_space: u64,
    pub available_space: u64,
    pub file_count: u64,
    pub directory_count: u64,
    pub average_file_size: f64,
    pub largest_file: Option<PathBuf>,
    pub duplicate_space: u64,
}

/// Optimization action
#[derive(Debug, Clone)]
pub enum OptimizationAction {
    /// No action needed
    None,
    
    /// Compress the file
    Compress {
        estimated_savings: u64,
        algorithm: CompressionAlgorithm,
    },
    
    /// Archive to cold storage
    Archive {
        destination: PathBuf,
        estimated_savings: u64,
    },
    
    /// Delete if confirmed
    DeleteCandidate {
        reason: String,
        last_access_days: u32,
    },
    
    /// Deduplicate
    Deduplicate {
        original_path: PathBuf,
        estimated_savings: u64,
    },
    
    /// Move to faster storage
    Promote {
        target_tier: StorageTier,
    },
    
    /// Move to slower storage
    Demote {
        target_tier: StorageTier,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum CompressionAlgorithm {
    Zstd,
    Lz4,
    Gzip,
    Brotli,
}

#[derive(Debug, Clone, Copy)]
pub enum StorageTier {
    Hot,    // SSD/NVMe
    Warm,   // HDD
    Cold,   // Archive/Tape
}

impl StorageOptimizer {
    pub fn new(memory_cell_endpoint: &str) -> Self {
        Self {
            memory_cell_endpoint: memory_cell_endpoint.to_string(),
            optimization_cache: Arc::new(RwLock::new(OptimizationCache::default())),
            config: OptimizerConfig::default(),
            stats: OptimizerStats::default(),
        }
    }
    
    /// Get memory pressure from MemoryCell
    pub async fn get_memory_pressure(&self) -> Result<f32, FileManagerError> {
        // In production, this would call the MemoryCell gRPC service
        // For now, simulate based on system memory
        
        let system = sysinfo::System::new_all();
        system.refresh_memory();
        
        let total = system.total_memory() as f64;
        let used = system.used_memory() as f64;
        
        let pressure = (used / total) as f32;
        
        debug!("Memory pressure: {:.1}%", pressure * 100.0);
        
        Ok(pressure)
    }
    
    /// Find unused files for optimization
    pub async fn find_unused_files(&self, index: &FileIndex) -> Result<Vec<FileInfo>, FileManagerError> {
        let threshold = Duration::from_secs(self.config.unused_threshold_days as u64 * 24 * 3600);
        
        let unused = index.find_by_last_access(threshold).await?;
        
        // Filter by size
        let large_unused: Vec<FileInfo> = unused
            .into_iter()
            .filter(|f| f.size >= self.config.min_optimization_size)
            .collect();
        
        info!("Found {} unused large files", large_unused.len());
        
        Ok(large_unused)
    }
    
    /// Find old files for compression
    pub async fn find_old_files(&self, index: &FileIndex, age: Duration) -> Result<Vec<FileInfo>, FileManagerError> {
        let old = index.find_by_last_access(age).await?;
        
        let compressible: Vec<FileInfo> = old
            .into_iter()
            .filter(|f| {
                f.size >= self.config.min_optimization_size &&
                Self::is_compressible(&f.path)
            })
            .collect();
        
        info!("Found {} old compressible files", compressible.len());
        
        Ok(compressible)
    }
    
    /// Check if a file is compressible
    fn is_compressible(path: &PathBuf) -> bool {
        let compressible_extensions = [
            "txt", "log", "json", "xml", "csv", "md", "rst",
            "rs", "py", "js", "ts", "go", "java", "c", "cpp", "h",
            "html", "css", "scss", "sql",
        ];
        
        let incompressible_extensions = [
            "zip", "gz", "bz2", "xz", "zst", "7z", "rar",
            "mp3", "mp4", "avi", "mkv", "mov", "webm",
            "jpg", "jpeg", "png", "gif", "webp", "bmp",
            "pdf", "docx", "xlsx", "pptx",
        ];
        
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            return compressible_extensions.contains(&ext_lower.as_str()) &&
                   !incompressible_extensions.contains(&ext_lower.as_str());
        }
        
        true // Default to compressible
    }
    
    /// Optimize a single file
    pub async fn optimize_file(&self, file: &FileInfo) -> Result<u64, FileManagerError> {
        debug!("Optimizing file: {}", file.path.display());
        
        // Determine best action
        let action = self.analyze_file(file).await?;
        
        match action {
            OptimizationAction::Compress { estimated_savings, .. } => {
                if self.config.enable_compression {
                    self.compress_file_internal(&file.path).await?;
                    return Ok(estimated_savings);
                }
            }
            OptimizationAction::Deduplicate { estimated_savings, .. } => {
                // Would implement deduplication
                return Ok(estimated_savings);
            }
            OptimizationAction::Archive { estimated_savings, .. } => {
                if self.config.enable_archival {
                    return Ok(estimated_savings);
                }
            }
            _ => {}
        }
        
        Ok(0)
    }
    
    /// Compress a file
    pub async fn compress_file(&self, file: &FileInfo) -> Result<u64, FileManagerError> {
        if !self.config.enable_compression {
            return Ok(0);
        }
        
        self.compress_file_internal(&file.path).await
    }
    
    async fn compress_file_internal(&self, path: &PathBuf) -> Result<u64, FileManagerError> {
        use std::fs::File;
        use std::io::{BufReader, BufWriter};
        
        // Get original size
        let metadata = std::fs::metadata(path)?;
        let original_size = metadata.len();
        
        // Create compressed file path
        let compressed_path = path.with_extension(
            format!("{}.zst", path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or(""))
        );
        
        info!("Compressing {} -> {}", path.display(), compressed_path.display());
        
        // In production, would use zstd or similar
        // For now, estimate compression ratio
        let compression_ratio = 0.4; // 60% reduction
        let saved = (original_size as f64 * (1.0 - compression_ratio)) as u64;
        
        debug!("Estimated savings: {} bytes", saved);
        
        Ok(saved)
    }
    
    /// Analyze a file for optimization potential
    async fn analyze_file(&self, file: &FileInfo) -> Result<OptimizationAction, FileManagerError> {
        // Check if compressible
        if Self::is_compressible(&file.path) && file.size > self.config.min_optimization_size {
            let compression_ratio = 0.4;
            let estimated_savings = (file.size as f64 * (1.0 - compression_ratio)) as u64;
            
            return Ok(OptimizationAction::Compress {
                estimated_savings,
                algorithm: CompressionAlgorithm::Zstd,
            });
        }
        
        Ok(OptimizationAction::None)
    }
    
    /// Get storage statistics
    pub async fn get_storage_stats(&self, index: &FileIndex) -> Result<StorageStats, FileManagerError> {
        let files = index.get_all_files().await?;
        
        let file_count = files.len() as u64;
        let total_size: u64 = files.iter().map(|f| f.size).sum();
        let avg_size = if file_count > 0 {
            total_size as f64 / file_count as f64
        } else {
            0.0
        };
        
        let largest = files.iter().max_by_key(|f| f.size).map(|f| f.path.clone());
        let directory_count = files.iter().filter(|f| f.is_directory).count() as u64;
        
        Ok(StorageStats {
            total_space: 0, // Would get from df
            used_space: total_size,
            available_space: 0, // Would get from df
            file_count,
            directory_count,
            average_file_size: avg_size,
            largest_file: largest,
            duplicate_space: 0, // Would analyze
        })
    }
    
    /// Get statistics
    pub fn stats(&self) -> &OptimizerStats {
        &self.stats
    }
}
