//! Virtual CPU Management for KolibriOS AI VMs.
//!
//! Provides CPU configuration, hotplug, and topology management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// CPU Topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuTopology {
    /// Number of sockets
    pub sockets: u32,
    /// Cores per socket
    pub cores: u32,
    /// Threads per core
    pub threads: u32,
}

impl Default for CpuTopology {
    fn default() -> Self {
        CpuTopology {
            sockets: 1,
            cores: 2,
            threads: 1,
        }
    }
}

impl CpuTopology {
    /// Create a new CPU topology
    pub fn new(sockets: u32, cores: u32, threads: u32) -> Self {
        CpuTopology { sockets, cores, threads }
    }

    /// Get total vCPU count
    pub fn total_cpus(&self) -> u32 {
        self.sockets * self.cores * self.threads
    }

    /// Convert to QEMU topology string
    pub fn to_qemu_string(&self) -> String {
        format!(
            "sockets={},cores={},threads={}",
            self.sockets, self.cores, self.threads
        )
    }
}

/// CPU Feature Flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuFeatures {
    /// Enabled features
    pub enabled: Vec<String>,
    /// Disabled features
    pub disabled: Vec<String>,
    /// Feature check mode
    pub check_mode: FeatureCheckMode,
}

impl Default for CpuFeatures {
    fn default() -> Self {
        CpuFeatures {
            enabled: Vec::new(),
            disabled: Vec::new(),
            check_mode: FeatureCheckMode::Enforce,
        }
    }
}

/// Feature Check Mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureCheckMode {
    /// Enforce features (fail if not available)
    Enforce,
    /// Require features
    Require,
    /// Optional features
    Optional,
}

/// Virtual CPU Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcpuConfig {
    /// CPU model name
    pub model: String,
    /// CPU topology
    pub topology: CpuTopology,
    /// CPU features
    pub features: CpuFeatures,
    /// CPU affinity (host CPU IDs)
    pub affinity: Option<Vec<u32>>,
    /// Enable hypervisor extensions
    pub hv_extensions: bool,
    /// Enable nested virtualization
    pub nested: bool,
    /// Maximum vCPUs (for hotplug)
    pub max_cpus: Option<u32>,
}

impl Default for VcpuConfig {
    fn default() -> Self {
        VcpuConfig {
            model: "host".to_string(),
            topology: CpuTopology::default(),
            features: CpuFeatures::default(),
            affinity: None,
            hv_extensions: false,
            nested: false,
            max_cpus: None,
        }
    }
}

impl VcpuConfig {
    /// Create a new vCPU configuration
    pub fn new(model: &str, count: u32) -> Self {
        VcpuConfig {
            model: model.to_string(),
            topology: CpuTopology::new(1, count, 1),
            ..Default::default()
        }
    }

    /// Set topology
    pub fn with_topology(mut self, sockets: u32, cores: u32, threads: u32) -> Self {
        self.topology = CpuTopology::new(sockets, cores, threads);
        self
    }

    /// Enable a CPU feature
    pub fn enable_feature(mut self, feature: &str) -> Self {
        self.features.enabled.push(feature.to_string());
        self
    }

    /// Disable a CPU feature
    pub fn disable_feature(mut self, feature: &str) -> Self {
        self.features.disabled.push(feature.to_string());
        self
    }

    /// Set CPU affinity
    pub fn with_affinity(mut self, cpus: Vec<u32>) -> Self {
        self.affinity = Some(cpus);
        self
    }

    /// Enable hypervisor extensions
    pub fn with_hv_extensions(mut self, enable: bool) -> Self {
        self.hv_extensions = enable;
        self
    }

    /// Enable nested virtualization
    pub fn with_nested(mut self, enable: bool) -> Self {
        self.nested = enable;
        self
    }

    /// Set max CPUs for hotplug
    pub fn with_max_cpus(mut self, max: u32) -> Self {
        self.max_cpus = Some(max);
        self
    }

    /// Convert to QEMU arguments
    pub fn to_qemu_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // CPU model with features
        let mut cpu_arg = self.model.clone();

        for feature in &self.features.enabled {
            cpu_arg.push_str(&format!(",+{}", feature));
        }

        for feature in &self.features.disabled {
            cpu_arg.push_str(&format!(",-{}", feature));
        }

        match self.features.check_mode {
            FeatureCheckMode::Enforce => cpu_arg.push_str(",enforce"),
            FeatureCheckMode::Require => cpu_arg.push_str(",require"),
            FeatureCheckMode::Optional => {}
        }

        args.push("-cpu".to_string());
        args.push(cpu_arg);

        // SMP configuration
        let mut smp_arg = format!("{}", self.topology.total_cpus());
        smp_arg.push_str(&format!(",{}", self.topology.to_qemu_string()));

        if let Some(max) = self.max_cpus {
            smp_arg.push_str(&format!(",maxcpus={}", max));
        }

        args.push("-smp".to_string());
        args.push(smp_arg);

        // Hypervisor extensions
        if self.hv_extensions {
            args.push("-cpu".to_string());
            args.push("host,hv_time,hv_relaxed,hv_vapic,hv_spinlocks=0x1fff".to_string());
        }

        args
    }
}

/// vCPU State
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VcpuState {
    /// CPU is running
    Running,
    /// CPU is stopped
    Stopped,
    /// CPU is paused
    Paused,
    /// CPU is being hotplugged
    Hotplugging,
    /// CPU is offline
    Offline,
}

/// Individual vCPU
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vcpu {
    /// CPU ID
    pub id: u32,
    /// CPU state
    pub state: VcpuState,
    /// CPU thread ID (host)
    pub thread_id: Option<u32>,
    /// Current frequency (MHz)
    pub frequency: Option<u32>,
}

impl Vcpu {
    /// Create a new vCPU
    pub fn new(id: u32) -> Self {
        Vcpu {
            id,
            state: VcpuState::Stopped,
            thread_id: None,
            frequency: None,
        }
    }
}

/// CPU Statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuStats {
    /// Total vCPUs
    pub total_vcpus: u32,
    /// Running vCPUs
    pub running_vcpus: u32,
    /// Total host CPUs
    pub host_cpus: u32,
    /// CPU usage percentage
    pub cpu_usage: f32,
    /// CPU time (nanoseconds)
    pub cpu_time: u64,
}

/// CPU Error
#[derive(Debug, thiserror::Error)]
pub enum CpuError {
    #[error("CPU not found: {0}")]
    CpuNotFound(u32),
    
    #[error("Hotplug failed: {0}")]
    HotplugFailed(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("CPU limit exceeded")]
    CpuLimitExceeded,
}

/// Virtual CPU Manager
pub struct VirtualCpuManager {
    /// vCPUs
    vcpus: Arc<RwLock<Vec<Vcpu>>>,
    /// CPU configuration
    config: Arc<RwLock<VcpuConfig>>,
}

impl VirtualCpuManager {
    /// Create a new CPU manager
    pub fn new(config: VcpuConfig) -> Self {
        let total_cpus = config.topology.total_cpus();
        let vcpus: Vec<Vcpu> = (0..total_cpus).map(Vcpu::new).collect();

        VirtualCpuManager {
            vcpus: Arc::new(RwLock::new(vcpus)),
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Get CPU count
    pub async fn cpu_count(&self) -> u32 {
        let vcpus = self.vcpus.read().await;
        vcpus.len() as u32
    }

    /// Get running CPU count
    pub async fn running_count(&self) -> u32 {
        let vcpus = self.vcpus.read().await;
        vcpus.iter().filter(|v| v.state == VcpuState::Running).count() as u32
    }

    /// Hotplug a CPU
    pub async fn hotplug_cpu(&self) -> Result<u32, CpuError> {
        let mut vcpus = self.vcpus.write().await;
        let config = self.config.read().await;

        // Check max CPUs limit
        if let Some(max) = config.max_cpus {
            if vcpus.len() as u32 >= max {
                return Err(CpuError::CpuLimitExceeded);
            }
        }

        let new_id = vcpus.len() as u32;
        let mut new_cpu = Vcpu::new(new_id);
        new_cpu.state = VcpuState::Hotplugging;

        // In real implementation, would use QEMU monitor:
        // device_add driver=host-x86_64-cpu,id=cpuN,socket-id=X,core-id=Y,thread-id=Z

        vcpus.push(new_cpu);

        Ok(new_id)
    }

    /// Hotunplug a CPU
    pub async fn hotunplug_cpu(&self, cpu_id: u32) -> Result<(), CpuError> {
        let mut vcpus = self.vcpus.write().await;

        let cpu = vcpus.iter_mut()
            .find(|v| v.id == cpu_id)
            .ok_or(CpuError::CpuNotFound(cpu_id))?;

        if cpu.state != VcpuState::Offline {
            cpu.state = VcpuState::Offline;
        }

        Ok(())
    }

    /// Get CPU by ID
    pub async fn get_cpu(&self, cpu_id: u32) -> Option<Vcpu> {
        let vcpus = self.vcpus.read().await;
        vcpus.iter().find(|v| v.id == cpu_id).cloned()
    }

    /// Get all CPUs
    pub async fn get_all_cpus(&self) -> Vec<Vcpu> {
        let vcpus = self.vcpus.read().await;
        vcpus.clone()
    }

    /// Get CPU statistics
    pub async fn get_stats(&self) -> CpuStats {
        let vcpus = self.vcpus.read().await;

        CpuStats {
            total_vcpus: vcpus.len() as u32,
            running_vcpus: vcpus.iter().filter(|v| v.state == VcpuState::Running).count() as u32,
            host_cpus: num_cpus::get() as u32,
            cpu_usage: 0.0, // Would calculate from actual metrics
            cpu_time: 0,    // Would get from cgroups or /proc
        }
    }

    /// Set CPU affinity
    pub async fn set_affinity(&self, cpu_id: u32, host_cpus: Vec<u32>) -> Result<(), CpuError> {
        let mut vcpus = self.vcpus.write().await;

        let cpu = vcpus.iter_mut()
            .find(|v| v.id == cpu_id)
            .ok_or(CpuError::CpuNotFound(cpu_id))?;

        // In real implementation, would use taskset or sched_setaffinity
        // via QEMU monitor or directly via pthread_setaffinity_np

        Ok(())
    }

    /// Get QEMU arguments
    pub async fn to_qemu_args(&self) -> Vec<String> {
        let config = self.config.read().await;
        config.to_qemu_args()
    }
}

/// Host CPU information
pub struct HostCpuInfo {
    /// CPU model name
    pub model: String,
    /// Number of cores
    pub cores: u32,
    /// Number of threads
    pub threads: u32,
    /// Number of sockets
    pub sockets: u32,
    /// Available features
    pub features: Vec<String>,
    /// CPU frequency (MHz)
    pub frequency: u32,
}

impl HostCpuInfo {
    /// Get host CPU information
    pub fn get() -> Self {
        let mut info = HostCpuInfo {
            model: "Unknown".to_string(),
            cores: 1,
            threads: 1,
            sockets: 1,
            features: Vec::new(),
            frequency: 0,
        };

        // Read from /proc/cpuinfo on Linux
        if let Ok(contents) = std::fs::read_to_string("/proc/cpuinfo") {
            let mut processors = 0u32;
            let mut physical_ids = std::collections::HashSet::new();
            let mut core_ids = std::collections::HashSet::new();

            for line in contents.lines() {
                if line.starts_with("model name") {
                    if let Some(value) = line.split(':').nth(1) {
                        info.model = value.trim().to_string();
                    }
                } else if line.starts_with("flags") {
                    if let Some(value) = line.split(':').nth(1) {
                        info.features = value.split_whitespace()
                            .map(String::from)
                            .collect();
                    }
                } else if line.starts_with("cpu MHz") {
                    if let Some(value) = line.split(':').nth(1) {
                        info.frequency = value.trim().parse().unwrap_or(0);
                    }
                } else if line.starts_with("processor") {
                    processors += 1;
                } else if line.starts_with("physical id") {
                    if let Some(value) = line.split(':').nth(1) {
                        if let Ok(id) = value.trim().parse::<u32>() {
                            physical_ids.insert(id);
                        }
                    }
                } else if line.starts_with("core id") {
                    if let Some(value) = line.split(':').nth(1) {
                        if let Ok(id) = value.trim().parse::<u32>() {
                            core_ids.insert(id);
                        }
                    }
                }
            }

            info.sockets = physical_ids.len() as u32;
            info.cores = if info.sockets > 0 {
                core_ids.len() as u32 / info.sockets
            } else {
                1
            };
            info.threads = if info.cores > 0 {
                processors / (info.sockets * info.cores)
            } else {
                processors
            };
        }

        info
    }
}

/// Check if KVM is available
pub fn kvm_available() -> bool {
    std::path::Path::new("/dev/kvm").exists() &&
    std::fs::metadata("/dev/kvm")
        .map(|m| !m.permissions().readonly())
        .unwrap_or(false)
}

/// Check if nested virtualization is supported
pub fn nested_virtualization_supported() -> bool {
    // Check for nested virtualization support
    if let Ok(contents) = std::fs::read_to_string(
        "/sys/module/kvm_intel/parameters/nested"
    ) {
        contents.trim() == "Y" || contents.trim() == "1"
    } else if let Ok(contents) = std::fs::read_to_string(
        "/sys/module/kvm_amd/parameters/nested"
    ) {
        contents.trim() == "Y" || contents.trim() == "1"
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_topology() {
        let topo = CpuTopology::new(2, 4, 2);
        assert_eq!(topo.total_cpus(), 16);
        assert_eq!(topo.to_qemu_string(), "sockets=2,cores=4,threads=2");
    }

    #[test]
    fn test_vcpu_config() {
        let config = VcpuConfig::new("host", 4)
            .enable_feature("vmx")
            .disable_feature("smx")
            .with_nested(true);

        assert_eq!(config.model, "host");
        assert_eq!(config.topology.total_cpus(), 4);
        assert!(config.nested);
    }

    #[tokio::test]
    async fn test_cpu_manager() {
        let config = VcpuConfig::new("host", 2);
        let manager = VirtualCpuManager::new(config);

        assert_eq!(manager.cpu_count().await, 2);
    }

    #[test]
    fn test_host_cpu_info() {
        let info = HostCpuInfo::get();
        assert!(!info.model.is_empty());
    }
}
