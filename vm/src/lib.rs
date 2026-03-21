//! KolibriOS AI Virtual Machine Module
//!
//! Provides comprehensive virtualization support using QEMU:
//! - VM lifecycle management
//! - Device configuration and hotplug
//! - Memory management with hugepage support
//! - CPU topology and hotplug
//! - Network and storage configuration
//!
//! # Example
//!
//! ```rust,no_run
//! use kolibrios_vm::{QemuVm, QemuConfig, create_kolibrios_vm_config};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create VM configuration
//!     let config = create_kolibrios_vm_config(
//!         "my-kolibrios-vm",
//!         "/var/lib/kolibrios/disk.qcow2",
//!         4096,  // 4GB RAM
//!         4,     // 4 CPUs
//!     );
//!
//!     // Create and start VM
//!     let mut vm = QemuVm::new(config);
//!     vm.start().await.expect("Failed to start VM");
//!     
//!     // VM is now running...
//!     
//!     // Stop VM
//!     vm.stop().await.expect("Failed to stop VM");
//! }
//! ```

pub mod qemu;
pub mod vmm;
pub mod device;
pub mod memory;
pub mod cpu;

// Re-exports
pub use qemu::{
    QemuVm, QemuConfig, QemuError, VmState,
    QemuMonitor, QemuImage,
    MachineType, CpuType, Acceleration, BootDevice,
    NetworkConfig, NetworkBackend, PortForward,
    StorageConfig, StorageInterface,
    MemoryConfig, DisplayConfig, DisplayType,
    MonitorConfig, MonitorType, SerialConfig, SerialType,
    create_kolibrios_vm_config,
};

pub use vmm::{
    VirtualMachineManager, VmManagerError, VmManagerStats,
    ResourceLimits, VmEventHandler, global_vmm,
};

pub use device::{
    DeviceType, DeviceState, DeviceConfig, DeviceManager,
    NetworkDeviceBuilder, StorageDeviceBuilder, UsbDeviceBuilder, GpuDeviceBuilder,
    NetdevConfig, NetdevType, DriveConfig,
};

pub use memory::{
    MemoryBackend, MemoryRegion, MemoryRegionState,
    MemoryBalloon, MemoryStats, MemoryError,
    VirtualMemoryManager, HugepageSize,
    hugepages_available, get_hugepages_count,
};

pub use cpu::{
    CpuTopology, VcpuConfig, VcpuState, Vcpu, CpuStats, CpuError,
    CpuFeatures, FeatureCheckMode, VirtualCpuManager,
    HostCpuInfo, kvm_available, nested_virtualization_supported,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::qemu::{QemuVm, QemuConfig, VmState, create_kolibrios_vm_config};
    pub use crate::vmm::VirtualMachineManager;
    pub use crate::device::{DeviceConfig, DeviceManager, DeviceType};
    pub use crate::memory::VirtualMemoryManager;
    pub use crate::cpu::{VcpuConfig, CpuTopology};
}

/// Version information
pub const VM_MODULE_VERSION: &str = env!("CARGO_PKG_VERSION");
