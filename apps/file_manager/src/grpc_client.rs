//! gRPC Client for MemoryCell and Unified Mind integration

use std::sync::Arc;
use std::time::Duration;

use tonic::transport::Channel;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::FileManagerError;

/// gRPC client for MemoryCell
pub struct MemoryCellClient {
    channel: Option<Channel>,
    endpoint: String,
}

impl MemoryCellClient {
    pub fn new(endpoint: &str) -> Self {
        Self {
            channel: None,
            endpoint: endpoint.to_string(),
        }
    }
    
    pub async fn connect(&mut self) -> Result<(), FileManagerError> {
        let channel = Channel::from_shared(self.endpoint.clone())
            .map_err(|e| FileManagerError::Grpc(e.to_string()))?
            .timeout(Duration::from_secs(5))
            .connect()
            .await
            .map_err(|e| FileManagerError::Grpc(e.to_string()))?;
        
        self.channel = Some(channel);
        info!("Connected to MemoryCell at {}", self.endpoint);
        Ok(())
    }
    
    /// Get memory pressure (0.0 - 1.0)
    pub async fn get_memory_pressure(&self) -> Result<f32, FileManagerError> {
        // In production, would call actual gRPC method
        // For now, use system memory info
        let system = sysinfo::System::new_all();
        system.refresh_memory();
        
        let total = system.total_memory() as f64;
        let used = system.used_memory() as f64;
        
        Ok((used / total) as f32)
    }
    
    /// Request memory allocation for file cache
    pub async fn request_cache_allocation(&self, size: u64) -> Result<bool, FileManagerError> {
        let pressure = self.get_memory_pressure().await?;
        
        // Only allow allocation if memory pressure is not critical
        if pressure > 0.9 {
            warn!("Memory pressure too high ({:.1}%) for cache allocation", pressure * 100.0);
            return Ok(false);
        }
        
        Ok(true)
    }
}

/// gRPC client for Unified Mind
pub struct UnifiedMindClient {
    channel: Option<Channel>,
    endpoint: String,
}

impl UnifiedMindClient {
    pub fn new(endpoint: &str) -> Self {
        Self {
            channel: None,
            endpoint: endpoint.to_string(),
        }
    }
    
    pub async fn connect(&mut self) -> Result<(), FileManagerError> {
        let channel = Channel::from_shared(self.endpoint.clone())
            .map_err(|e| FileManagerError::Grpc(e.to_string()))?
            .timeout(Duration::from_secs(5))
            .connect()
            .await
            .map_err(|e| FileManagerError::Grpc(e.to_string()))?;
        
        self.channel = Some(channel);
        info!("Connected to Unified Mind at {}", self.endpoint);
        Ok(())
    }
    
    /// Get context-aware file predictions
    pub async fn get_file_predictions(
        &self,
        current_context: &str,
        recent_files: &[std::path::PathBuf],
    ) -> Result<Vec<String>, FileManagerError> {
        // In production, would call LLM through Unified Mind
        // For now, return based on simple heuristics
        
        let mut predictions = Vec::new();
        
        // Predict based on file extensions
        let extensions: Vec<_> = recent_files
            .iter()
            .filter_map(|p| p.extension().and_then(|e| e.to_str()))
            .collect();
        
        // If editing Rust files, predict Cargo.toml or tests
        if extensions.iter().any(|e| *e == "rs") {
            predictions.push("Cargo.toml".to_string());
            predictions.push("src/lib.rs".to_string());
        }
        
        // If editing markdown, predict related docs
        if extensions.iter().any(|e| *e == "md") {
            predictions.push("README.md".to_string());
            predictions.push("docs/".to_string());
        }
        
        Ok(predictions)
    }
    
    /// Analyze file content for tagging
    pub async fn analyze_content(&self, content: &str) -> Result<Vec<String>, FileManagerError> {
        // Simple tag extraction based on keywords
        let mut tags = Vec::new();
        
        let keywords = [
            ("TODO", "todo"),
            ("FIXME", "fixme"),
            ("import", "import"),
            ("function", "function"),
            ("class", "class"),
            ("struct", "struct"),
            ("test", "test"),
            ("config", "config"),
            ("api", "api"),
            ("database", "database"),
        ];
        
        let content_lower = content.to_lowercase();
        
        for (keyword, tag) in keywords {
            if content_lower.contains(keyword) {
                tags.push(tag.to_string());
            }
        }
        
        Ok(tags)
    }
}
