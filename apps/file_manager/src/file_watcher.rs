//! File Watcher
//!
//! Real-time file system monitoring for adaptive suggestions.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use notify::{Event, EventKind, RecursiveMode, Watcher};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use super::{FileEvent, FileManagerError};

/// File system watcher
pub struct FileWatcher {
    /// Paths being watched
    watch_paths: Vec<PathBuf>,
    
    /// Event sender
    event_sender: broadcast::Sender<FileEvent>,
    
    /// Event receiver (for testing)
    #[allow(dead_code)]
    event_receiver: broadcast::Receiver<FileEvent>,
    
    /// Watcher handle
    watcher: Option<Arc<RwLock<Option<notify::RecommendedWatcher>>>>,
    
    /// Running state
    running: bool,
    
    /// Statistics
    stats: WatcherStats,
}

#[derive(Debug, Clone, Default)]
pub struct WatcherStats {
    pub events_processed: u64,
    pub files_created: u64,
    pub files_modified: u64,
    pub files_deleted: u64,
    pub files_renamed: u64,
}

impl FileWatcher {
    pub fn new(watch_paths: &[PathBuf]) -> Self {
        let (sender, receiver) = broadcast::channel(1000);
        
        Self {
            watch_paths: watch_paths.to_vec(),
            event_sender: sender,
            event_receiver: receiver,
            watcher: None,
            running: false,
            stats: WatcherStats::default(),
        }
    }
    
    /// Start watching
    pub async fn start(&mut self) -> Result<(), FileManagerError> {
        if self.running {
            return Ok(());
        }
        
        info!("Starting file watcher for {} paths", self.watch_paths.len());
        
        let sender = self.event_sender.clone();
        
        // Create watcher
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if let Some(file_event) = Self::convert_event(&event) {
                        let _ = sender.send(file_event);
                    }
                }
                Err(e) => {
                    error!("Watch error: {}", e);
                }
            }
        }).map_err(|e| FileManagerError::Watch(e.to_string()))?;
        
        // Watch each path
        for path in &self.watch_paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)
                    .map_err(|e| FileManagerError::Watch(e.to_string()))?;
                info!("Watching path: {}", path.display());
            } else {
                warn!("Path does not exist: {}", path.display());
            }
        }
        
        self.watcher = Some(Arc::new(RwLock::new(Some(watcher))));
        self.running = true;
        
        Ok(())
    }
    
    /// Stop watching
    pub async fn stop(&mut self) -> Result<(), FileManagerError> {
        if !self.running {
            return Ok(());
        }
        
        info!("Stopping file watcher");
        
        if let Some(watcher_arc) = &self.watcher {
            let mut watcher_guard = watcher_arc.write().await;
            if let Some(mut watcher) = watcher_guard.take() {
                // Unwatch all paths
                for path in &self.watch_paths {
                    watcher.unwatch(path).ok();
                }
            }
        }
        
        self.watcher = None;
        self.running = false;
        
        Ok(())
    }
    
    /// Subscribe to file events
    pub fn subscribe(&self) -> broadcast::Receiver<FileEvent> {
        self.event_sender.subscribe()
    }
    
    /// Convert notify event to our event type
    fn convert_event(event: &Event) -> Option<FileEvent> {
        let path = event.paths.first()?.clone();
        
        match &event.kind {
            EventKind::Create(_) => Some(FileEvent::Created(path)),
            EventKind::Modify(_) => Some(FileEvent::Modified(path)),
            EventKind::Remove(_) => Some(FileEvent::Deleted(path)),
            EventKind::Any => {
                // Could be rename
                if event.paths.len() >= 2 {
                    Some(FileEvent::Renamed {
                        old_path: event.paths[0].clone(),
                        new_path: event.paths[1].clone(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// Get statistics
    pub fn stats(&self) -> &WatcherStats {
        &self.stats
    }
    
    /// Check if running
    pub fn is_running(&self) -> bool {
        self.running
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        if self.running {
            // Try to stop synchronously
            warn!("FileWatcher dropped while running");
        }
    }
}
