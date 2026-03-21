//! Virtual Memory Management for KolibriOS AI VMs.
//!
//! Provides memory allocation, balloon driver, and hugepage support.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Memory Backend Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryBackend {
    /// Normal RAM
    Ram,
    /// File-backed memory
    File {
        path: String,
        share: bool,
    },
    /// Hugepages
    Hugepages {
        path: Option<String>,
        size: HugepageSize,
    },
    /// Memory-mapped memory
    Mmap {
        path: String,
        offset: u64,
        size: u64,
    },
}

/// Hugepage Size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HugepageSize {
    /// 2MB hugepages
    Huge2M,
    /// 1GB hugepages
    Huge1G,
    /// Custom size
    Custom(u64),
}

/// Memory Region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRegion {
    /// Region ID
    pub id: String,
    /// Size in bytes
    pub size: u64,
    /// Memory backend
    pub backend: MemoryBackend,
    /// Is region removable
    pub removable: bool,
    /// Current state
    pub state: MemoryRegionState,
}

/// Memory Region State
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryRegionState {
    /// Region is active
    Active,
    /// Region is inactive
    Inactive,
    /// Region is being hotplugged
    Hotplugging,
    /// Region is being removed
    Removing,
}

impl MemoryRegion {
    /// Create a new RAM region
    pub fn new_ram(id: &str, size: u64) -> Self {
        MemoryRegion {
            id: id.to_string(),
            size,
            backend: MemoryBackend::Ram,
            removable: false,
            state: MemoryRegionState::Active,
        }
    }

    /// Create a file-backed region
    pub fn new_file(id: &str, size: u64, path: &str, share: bool) -> Self {
        MemoryRegion {
            id: id.to_string(),
            size,
            backend: MemoryBackend::File {
                path: path.to_string(),
                share,
            },
            removable: true,
            state: MemoryRegionState::Active,
        }
    }

    /// Create a hugepage region
    pub fn new_hugepages(id: &str, size: u64, page_size: HugepageSize) -> Self {
        MemoryRegion {
            id: id.to_string(),
            size,
            backend: MemoryBackend::Hugepages {
                path: None,
                size: page_size,
            },
            removable: true,
            state: MemoryRegionState::Active,
        }
    }

    /// Convert to QEMU memory-backend arguments
    pub fn to_qemu_backend_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Create memory-backend object
        let backend_type = match &self.backend {
            MemoryBackend::Ram => "memory-backend-ram",
            MemoryBackend::File { path, share } => {
                args.push("-object".to_string());
                args.push(format!(
                    "memory-backend-file,id={},size={},mem-path={},share={}",
                    self.id,
                    self.size,
                    path,
                    if *share { "on" } else { "off" }
                ));
                return args;
            }
            MemoryBackend::Hugepages { path, size } => {
                let size_str = match size {
                    HugepageSize::Huge2M => "2M",
                    HugepageSize::Huge1G => "1G",
                    HugepageSize::Custom(s) => &format!("{}", s),
                };
                args.push("-object".to_string());
                let mut obj_arg = format!(
                    "memory-backend-file,id={},size={},mem-path=/dev/hugepages",
                    self.id,
                    self.size,
                );
                if let Some(p) = path {
                    obj_arg.push_str(&format!(",mem-path={}", p));
                }
                args.push(obj_arg);
                return args;
            }
            MemoryBackend::Mmap { path, offset, size } => {
                args.push("-object".to_string());
                args.push(format!(
                    "memory-backend-file,id={},size={},mem-path={},offset={}",
                    self.id, size, path, offset
                ));
                return args;
            }
        };

        args.push("-object".to_string());
        args.push(format!(
            "{},id={},size={}",
            backend_type,
            self.id,
            self.size
        ));

        args
    }
}

/// Memory Balloon Device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBalloon {
    /// Balloon device ID
    pub id: String,
    /// Initial size in bytes (0 = fully deflated)
    pub initial_size: u64,
    /// Maximum size
    pub max_size: u64,
    /// Current actual size (reported by guest)
    pub actual_size: u64,
}

impl MemoryBalloon {
    /// Create a new memory balloon
    pub fn new(id: &str, max_size: u64) -> Self {
        MemoryBalloon {
            id: id.to_string(),
            initial_size: 0,
            max_size,
            actual_size: max_size,
        }
    }

    /// Inflate balloon by specified amount
    pub fn inflate(&mut self, amount: u64) -> Result<(), MemoryError> {
        let new_size = self.initial_size + amount;
        if new_size > self.max_size {
            return Err(MemoryError::BalloonLimitExceeded);
        }
        self.initial_size = new_size;
        Ok(())
    }

    /// Deflate balloon by specified amount
    pub fn deflate(&mut self, amount: u64) -> Result<(), MemoryError> {
        if amount > self.initial_size {
            self.initial_size = 0;
        } else {
            self.initial_size -= amount;
        }
        Ok(())
    }

    /// Get available memory (actual - balloon size)
    pub fn available_memory(&self) -> u64 {
        self.actual_size.saturating_sub(self.initial_size)
    }

    /// Convert to QEMU arguments
    pub fn to_qemu_args(&self) -> Vec<String> {
        vec![
            "-device".to_string(),
            format!("virtio-balloon-pci,id={}", self.id),
        ]
    }
}

/// Memory Statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total memory allocated (bytes)
    pub total_memory: u64,
    /// Memory used by VMs (bytes)
    pub used_memory: u64,
    /// Available memory for new VMs (bytes)
    pub available_memory: u64,
    /// Memory ballooned (bytes)
    pub ballooned_memory: u64,
    /// Memory regions count
    pub regions_count: usize,
}

/// Memory Error
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Insufficient memory: {0}")]
    InsufficientMemory(String),
    
    #[error("Balloon limit exceeded")]
    BalloonLimitExceeded,
    
    #[error("Region not found: {0}")]
    RegionNotFound(String),
    
    #[error("Hotplug failed: {0}")]
    HotplugFailed(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Virtual Memory Manager
pub struct VirtualMemoryManager {
    /// Memory regions
    regions: Arc<RwLock<Vec<MemoryRegion>>>,
    /// Balloon devices
    balloons: Arc<RwLock<Vec<MemoryBalloon>>>,
    /// Total physical memory on host
    total_host_memory: u64,
}

impl VirtualMemoryManager {
    /// Create a new memory manager
    pub fn new() -> Self {
        VirtualMemoryManager {
            regions: Arc::new(RwLock::new(Vec::new())),
            balloons: Arc::new(RwLock::new(Vec::new())),
            total_host_memory: get_host_memory(),
        }
    }

    /// Add a memory region
    pub async fn add_region(&self, region: MemoryRegion) -> Result<(), MemoryError> {
        let mut regions = self.regions.write().await;
        regions.push(region);
        Ok(())
    }

    /// Remove a memory region
    pub async fn remove_region(&self, region_id: &str) -> Result<MemoryRegion, MemoryError> {
        let mut regions = self.regions.write().await;
        let pos = regions.iter().position(|r| r.id == region_id)
            .ok_or_else(|| MemoryError::RegionNotFound(region_id.to_string()))?;
        
        let region = regions.remove(pos);
        Ok(region)
    }

    /// Hotplug memory
    pub async fn hotplug_memory(&self, size: u64, region_id: &str) -> Result<(), MemoryError> {
        let region = MemoryRegion::new_ram(region_id, size);
        let mut regions = self.regions.write().await;
        
        // In real implementation, this would use QEMU monitor commands
        // qom-set /machine/peripheral-anon/device[0]/size size
        
        regions.push(region);
        Ok(())
    }

    /// Add a balloon device
    pub async fn add_balloon(&self, balloon: MemoryBalloon) {
        let mut balloons = self.balloons.write().await;
        balloons.push(balloon);
    }

    /// Get balloon by ID
    pub async fn get_balloon(&self, balloon_id: &str) -> Option<MemoryBalloon> {
        let balloons = self.balloons.read().await;
        balloons.iter().find(|b| b.id == balloon_id).cloned()
    }

    /// Inflate balloon
    pub async fn inflate_balloon(&self, balloon_id: &str, amount: u64) -> Result<(), MemoryError> {
        let mut balloons = self.balloons.write().await;
        let balloon = balloons.iter_mut()
            .find(|b| b.id == balloon_id)
            .ok_or_else(|| MemoryError::RegionNotFound(balloon_id.to_string()))?;
        
        balloon.inflate(amount)
    }

    /// Deflate balloon
    pub async fn deflate_balloon(&self, balloon_id: &str, amount: u64) -> Result<(), MemoryError> {
        let mut balloons = self.balloons.write().await;
        let balloon = balloons.iter_mut()
            .find(|b| b.id == balloon_id)
            .ok_or_else(|| MemoryError::RegionNotFound(balloon_id.to_string()))?;
        
        balloon.deflate(amount)
    }

    /// Get memory statistics
    pub async fn get_stats(&self) -> MemoryStats {
        let regions = self.regions.read().await;
        let balloons = self.balloons.read().await;

        let total_memory: u64 = regions.iter().map(|r| r.size).sum();
        let ballooned_memory: u64 = balloons.iter().map(|b| b.initial_size).sum();
        let actual_size: u64 = balloons.iter().map(|b| b.actual_size).sum();

        MemoryStats {
            total_memory,
            used_memory: actual_size,
            available_memory: total_memory.saturating_sub(actual_size),
            ballooned_memory,
            regions_count: regions.len(),
        }
    }

    /// Get all QEMU memory arguments
    pub async fn to_qemu_args(&self) -> Vec<String> {
        let regions = self.regions.read().await;
        let balloons = self.balloons.read().await;

        let mut args = Vec::new();

        // Memory regions
        for region in regions.iter() {
            args.extend(region.to_qemu_backend_args());
        }

        // Balloon devices
        for balloon in balloons.iter() {
            args.extend(balloon.to_qemu_args());
        }

        args
    }
}

impl Default for VirtualMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Get host system memory
fn get_host_memory() -> u64 {
    // Read from /proc/meminfo on Linux
    if let Ok(contents) = std::fs::read_to_string("/proc/meminfo") {
        for line in contents.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        return kb * 1024; // Convert to bytes
                    }
                }
            }
        }
    }

    // Default fallback
    16 * 1024 * 1024 * 1024 // 16GB
}

/// Check if hugepages are available
pub fn hugepages_available() -> bool {
    // Check for hugepage support
    std::path::Path::new("/dev/hugepages").exists() ||
    std::fs::read_to_string("/proc/sys/vm/nr_hugepages")
        .map(|s| s.trim().parse::<u64>().unwrap_or(0) > 0)
        .unwrap_or(false)
}

/// Get available hugepages
pub fn get_hugepages_count() -> u64 {
    std::fs::read_to_string("/proc/sys/vm/nr_hugepages")
        .map(|s| s.trim().parse::<u64>().unwrap_or(0))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_region() {
        let region = MemoryRegion::new_ram("mem0", 1024 * 1024 * 1024);
        assert_eq!(region.id, "mem0");
        assert_eq!(region.size, 1024 * 1024 * 1024);
    }

    #[test]
    fn test_memory_balloon() {
        let mut balloon = MemoryBalloon::new("balloon0", 1024 * 1024 * 1024);
        assert_eq!(balloon.initial_size, 0);
        
        balloon.inflate(512 * 1024 * 1024).unwrap();
        assert_eq!(balloon.initial_size, 512 * 1024 * 1024);
        
        balloon.deflate(256 * 1024 * 1024).unwrap();
        assert_eq!(balloon.initial_size, 256 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_memory_manager() {
        let manager = VirtualMemoryManager::new();
        let region = MemoryRegion::new_ram("mem0", 1024 * 1024 * 1024);
        
        manager.add_region(region).await.unwrap();
        let stats = manager.get_stats().await;
        
        assert_eq!(stats.regions_count, 1);
    }
}
