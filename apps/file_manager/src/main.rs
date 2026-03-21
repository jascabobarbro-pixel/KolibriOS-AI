//! Adaptive File Manager - Main Entry Point

use std::path::PathBuf;
use std::time::Duration;

use tracing::{info, warn, error};
use tracing_subscriber::fmt;

mod lib;

use adaptive_file_manager::{
    AdaptiveFileManager, FileManagerConfig, UserContext,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    fmt::init();
    
    info!("Starting Adaptive File Manager");
    
    // Configuration
    let home = std::env::var("HOME")
        .map(|h| PathBuf::from(h))
        .unwrap_or_else(|_| PathBuf::from("/home/user"));
    
    let config = FileManagerConfig {
        watch_paths: vec![
            home.join("Documents"),
            home.join("Projects"),
            home.join("Downloads"),
        ],
        suggestions_enabled: true,
        optimization_enabled: true,
        max_suggestions: 10,
        index_update_interval: Duration::from_secs(60),
        memory_cell_endpoint: "http://localhost:50051".to_string(),
        mind_endpoint: "http://localhost:50052".to_string(),
        learning_enabled: true,
    };
    
    // Create file manager
    let mut file_manager = AdaptiveFileManager::new(config);
    
    // Initialize
    match file_manager.initialize().await {
        Ok(_) => info!("File Manager initialized successfully"),
        Err(e) => {
            error!("Failed to initialize file manager: {}", e);
            return Err(e.into());
        }
    }
    
    // Example: Get suggestions
    let context = UserContext::current();
    match file_manager.get_suggestions(&context).await {
        Ok(suggestions) => {
            info!("Got {} suggestions:", suggestions.len());
            for s in suggestions.iter().take(5) {
                info!("  - {} (relevance: {:.2})", s.name, s.relevance);
            }
        }
        Err(e) => {
            warn!("Failed to get suggestions: {}", e);
        }
    }
    
    // Example: Search
    match file_manager.search("readme").await {
        Ok(results) => {
            info!("Found {} files matching 'readme'", results.len());
        }
        Err(e) => {
            warn!("Search failed: {}", e);
        }
    }
    
    // Example: Optimize storage
    match file_manager.optimize_storage().await {
        Ok(report) => {
            info!(
                "Storage optimization complete: {} files optimized, {} bytes saved",
                report.files_optimized, report.space_saved
            );
        }
        Err(e) => {
            warn!("Storage optimization failed: {}", e);
        }
    }
    
    // Print statistics
    let stats = file_manager.stats();
    info!("Statistics:");
    info!("  Files indexed: {}", stats.files_indexed);
    info!("  Suggestions made: {}", stats.suggestions_made);
    info!("  Optimizations performed: {}", stats.optimizations_performed);
    info!("  Space saved: {} bytes", stats.space_saved_bytes);
    
    // Shutdown
    file_manager.shutdown().await?;
    
    info!("File Manager shutdown complete");
    
    Ok(())
}
