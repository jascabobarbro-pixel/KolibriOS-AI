//! Virtual Machine Manager (VMM) for KolibriOS AI.
//!
//! Provides high-level management of multiple VMs, lifecycle control,
//! and resource management.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use super::qemu::{QemuVm, QemuConfig, VmState, QemuError};

/// Virtual Machine Manager
pub struct VirtualMachineManager {
    /// Managed VMs
    vms: Arc<RwLock<HashMap<String, Arc<RwLock<QemuVm>>>>>,
    /// VM configurations
    configs: Arc<RwLock<HashMap<String, QemuConfig>>>,
    /// Resource limits
    resource_limits: ResourceLimits,
    /// Event handlers
    event_handlers: Vec<Box<dyn VmEventHandler + Send + Sync>>,
}

/// Resource Limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum number of VMs
    pub max_vms: usize,
    /// Maximum total memory (MB)
    pub max_total_memory_mb: u64,
    /// Maximum total CPUs
    pub max_total_cpus: u32,
    /// Maximum disk space per VM (MB)
    pub max_disk_per_vm_mb: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        ResourceLimits {
            max_vms: 10,
            max_total_memory_mb: 32768, // 32GB
            max_total_cpus: 32,
            max_disk_per_vm_mb: 102400, // 100GB
        }
    }
}

/// VM Event Handler trait
pub trait VmEventHandler {
    fn on_vm_started(&self, vm_id: &str);
    fn on_vm_stopped(&self, vm_id: &str);
    fn on_vm_paused(&self, vm_id: &str);
    fn on_vm_resumed(&self, vm_id: &str);
    fn on_vm_error(&self, vm_id: &str, error: &str);
}

/// VM Manager Statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VmManagerStats {
    /// Total VMs created
    pub total_created: u64,
    /// Currently running VMs
    pub running_count: usize,
    /// Total memory allocated (MB)
    pub total_memory_mb: u64,
    /// Total CPUs allocated
    pub total_cpus: u32,
}

/// VM Manager Error
#[derive(Debug, thiserror::Error)]
pub enum VmManagerError {
    #[error("VM not found: {0}")]
    VmNotFound(String),
    
    #[error("VM already exists: {0}")]
    VmAlreadyExists(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("QEMU error: {0}")]
    QemuError(#[from] QemuError),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl VirtualMachineManager {
    /// Create a new VMM instance
    pub fn new() -> Self {
        VirtualMachineManager {
            vms: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            resource_limits: ResourceLimits::default(),
            event_handlers: Vec::new(),
        }
    }

    /// Create VMM with custom resource limits
    pub fn with_limits(limits: ResourceLimits) -> Self {
        VirtualMachineManager {
            vms: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            resource_limits: limits,
            event_handlers: Vec::new(),
        }
    }

    /// Add event handler
    pub fn add_event_handler<H: VmEventHandler + Send + Sync + 'static>(&mut self, handler: H) {
        self.event_handlers.push(Box::new(handler));
    }

    /// Create a new VM
    pub async fn create_vm(&self, config: QemuConfig) -> Result<String, VmManagerError> {
        let vm_id = config.name.clone();

        // Check resource limits
        self.check_resource_limits(&config).await?;

        // Check if VM already exists
        {
            let vms = self.vms.read().await;
            if vms.contains_key(&vm_id) {
                return Err(VmManagerError::VmAlreadyExists(vm_id));
            }
        }

        // Create VM
        let vm = QemuVm::new(config.clone());

        // Store VM and config
        {
            let mut vms = self.vms.write().await;
            vms.insert(vm_id.clone(), Arc::new(RwLock::new(vm)));
        }

        {
            let mut configs = self.configs.write().await;
            configs.insert(vm_id.clone(), config);
        }

        Ok(vm_id)
    }

    /// Start a VM
    pub async fn start_vm(&self, vm_id: &str) -> Result<(), VmManagerError> {
        let vm = self.get_vm(vm_id).await?;
        let mut vm = vm.write().await;

        vm.start().await?;

        // Notify handlers
        for handler in &self.event_handlers {
            handler.on_vm_started(vm_id);
        }

        Ok(())
    }

    /// Stop a VM
    pub async fn stop_vm(&self, vm_id: &str) -> Result<(), VmManagerError> {
        let vm = self.get_vm(vm_id).await?;
        let mut vm = vm.write().await;

        vm.stop().await?;

        // Notify handlers
        for handler in &self.event_handlers {
            handler.on_vm_stopped(vm_id);
        }

        Ok(())
    }

    /// Pause a VM
    pub async fn pause_vm(&self, vm_id: &str) -> Result<(), VmManagerError> {
        let vm = self.get_vm(vm_id).await?;
        let vm = vm.read().await;

        vm.pause().await?;

        // Notify handlers
        for handler in &self.event_handlers {
            handler.on_vm_paused(vm_id);
        }

        Ok(())
    }

    /// Resume a VM
    pub async fn resume_vm(&self, vm_id: &str) -> Result<(), VmManagerError> {
        let vm = self.get_vm(vm_id).await?;
        let vm = vm.read().await;

        vm.resume().await?;

        // Notify handlers
        for handler in &self.event_handlers {
            handler.on_vm_resumed(vm_id);
        }

        Ok(())
    }

    /// Delete a VM
    pub async fn delete_vm(&self, vm_id: &str) -> Result<(), VmManagerError> {
        // Stop VM first if running
        {
            let vm = self.get_vm(vm_id).await?;
            let mut vm = vm.write().await;

            if vm.is_running().await {
                vm.stop().await?;
            }
        }

        // Remove from maps
        {
            let mut vms = self.vms.write().await;
            vms.remove(vm_id);
        }

        {
            let mut configs = self.configs.write().await;
            configs.remove(vm_id);
        }

        Ok(())
    }

    /// Get VM state
    pub async fn get_vm_state(&self, vm_id: &str) -> Result<VmState, VmManagerError> {
        let vm = self.get_vm(vm_id).await?;
        let vm = vm.read().await;
        Ok(vm.state().await)
    }

    /// Get VM configuration
    pub async fn get_vm_config(&self, vm_id: &str) -> Result<QemuConfig, VmManagerError> {
        let configs = self.configs.read().await;
        configs.get(vm_id)
            .cloned()
            .ok_or_else(|| VmManagerError::VmNotFound(vm_id.to_string()))
    }

    /// List all VMs
    pub async fn list_vms(&self) -> Vec<String> {
        let vms = self.vms.read().await;
        vms.keys().cloned().collect()
    }

    /// Get statistics
    pub async fn get_stats(&self) -> VmManagerStats {
        let vms = self.vms.read().await;
        let configs = self.configs.read().await;

        let mut stats = VmManagerStats::default();
        stats.total_created = vms.len() as u64;

        for (vm_id, vm) in vms.iter() {
            let vm = vm.read().await;
            if vm.is_running().await {
                stats.running_count += 1;
            }

            if let Some(config) = configs.get(vm_id) {
                stats.total_memory_mb += config.memory.size_mb;
                stats.total_cpus += config.cpu_count;
            }
        }

        stats
    }

    /// Check resource limits
    async fn check_resource_limits(&self, config: &QemuConfig) -> Result<(), VmManagerError> {
        let stats = self.get_stats().await;

        // Check VM count
        if stats.total_created as usize >= self.resource_limits.max_vms {
            return Err(VmManagerError::ResourceLimitExceeded(
                format!("Maximum VM count ({}) reached", self.resource_limits.max_vms)
            ));
        }

        // Check total memory
        let new_total_memory = stats.total_memory_mb + config.memory.size_mb;
        if new_total_memory > self.resource_limits.max_total_memory_mb {
            return Err(VmManagerError::ResourceLimitExceeded(
                format!("Memory limit exceeded: {}MB > {}MB",
                    new_total_memory, self.resource_limits.max_total_memory_mb)
            ));
        }

        // Check total CPUs
        let new_total_cpus = stats.total_cpus + config.cpu_count;
        if new_total_cpus > self.resource_limits.max_total_cpus {
            return Err(VmManagerError::ResourceLimitExceeded(
                format!("CPU limit exceeded: {} > {}",
                    new_total_cpus, self.resource_limits.max_total_cpus)
            ));
        }

        Ok(())
    }

    /// Get VM by ID
    async fn get_vm(&self, vm_id: &str) -> Result<Arc<RwLock<QemuVm>>, VmManagerError> {
        let vms = self.vms.read().await;
        vms.get(vm_id)
            .cloned()
            .ok_or_else(|| VmManagerError::VmNotFound(vm_id.to_string()))
    }

    /// Snapshot VM state
    pub async fn snapshot_vm(&self, vm_id: &str, snapshot_name: &str) -> Result<(), VmManagerError> {
        let vm = self.get_vm(vm_id).await?;
        let vm = vm.read().await;

        if let Some(monitor) = vm.monitor() {
            monitor.save_vm(snapshot_name).await?;
        } else {
            return Err(VmManagerError::InvalidOperation("Monitor not available".to_string()));
        }

        Ok(())
    }

    /// Restore VM from snapshot
    pub async fn restore_snapshot(&self, vm_id: &str, snapshot_name: &str) -> Result<(), VmManagerError> {
        let vm = self.get_vm(vm_id).await?;
        let vm = vm.read().await;

        if let Some(monitor) = vm.monitor() {
            monitor.load_vm(snapshot_name).await?;
        } else {
            return Err(VmManagerError::InvalidOperation("Monitor not available".to_string()));
        }

        Ok(())
    }

    /// Clone a VM
    pub async fn clone_vm(&self, source_id: &str, new_id: &str) -> Result<String, VmManagerError> {
        let config = self.get_vm_config(source_id).await?;

        let mut new_config = config.clone();
        new_config.name = new_id.to_string();

        // Create new VM with cloned config
        self.create_vm(new_config).await
    }

    /// Migrate VM to another host
    pub async fn migrate_vm(&self, vm_id: &str, target_host: &str) -> Result<(), VmManagerError> {
        // This would implement live migration
        // For now, return not implemented
        Err(VmManagerError::InvalidOperation(
            "Live migration not yet implemented".to_string()
        ))
    }
}

impl Default for VirtualMachineManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global VMM instance
static VMM_INSTANCE: once_cell::sync::Lazy<Arc<RwLock<VirtualMachineManager>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(VirtualMachineManager::new())));

/// Get global VMM instance
pub fn global_vmm() -> Arc<RwLock<VirtualMachineManager>> {
    VMM_INSTANCE.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vmm_create_vm() {
        let vmm = VirtualMachineManager::new();
        let config = QemuConfig {
            name: "test-vm".to_string(),
            ..Default::default()
        };

        let result = vmm.create_vm(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_vmm_list_vms() {
        let vmm = VirtualMachineManager::new();
        let config = QemuConfig {
            name: "test-vm-2".to_string(),
            ..Default::default()
        };

        vmm.create_vm(config).await.unwrap();
        let vms = vmm.list_vms().await;

        assert!(!vms.is_empty());
    }

    #[tokio::test]
    async fn test_vmm_stats() {
        let vmm = VirtualMachineManager::new();
        let stats = vmm.get_stats().await;

        assert_eq!(stats.running_count, 0);
    }
}
