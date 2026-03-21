# KolibriOS AI Virtual Machine Module

## Overview

The KolibriOS AI VM module provides comprehensive virtualization support using QEMU. It enables running KolibriOS AI in isolated virtual machines with hardware acceleration.

## Features

- **VM Lifecycle Management**: Create, start, stop, pause, and resume VMs
- **Hardware Acceleration**: Support for KVM (Linux), HVF (macOS), and WHPX (Windows)
- **Device Hotplug**: Add/remove devices at runtime
- **Memory Management**: Hugepage support, memory ballooning
- **CPU Hotplug**: Dynamic CPU allocation
- **Network Configuration**: User-mode, TAP, bridge networking
- **Storage Management**: Multiple disk formats (qcow2, raw, vmdk)
- **QEMU Monitor**: Full control interface via TCP or Unix socket

## Installation

### Prerequisites

1. **QEMU** - Install QEMU for your platform:
   ```bash
   # Ubuntu/Debian
   sudo apt install qemu-system-x86 qemu-utils

   # Fedora
   sudo dnf install qemu qemu-img

   # macOS
   brew install qemu

   # Windows
   # Download from https://www.qemu.org/download/
   ```

2. **KVM** (Linux only) - For hardware acceleration:
   ```bash
   sudo apt install qemu-kvm
   sudo usermod -aG kvm $USER
   ```

3. **Rust** - Install Rust toolchain:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

### Building

```bash
cd kolibrios-ai
cargo build --release -p kolibrios-vm
```

## Quick Start

### Creating a VM

```rust
use kolibrios_vm::{QemuVm, QemuConfig, create_kolibrios_vm_config};

#[tokio::main]
async fn main() {
    // Create VM configuration
    let config = create_kolibrios_vm_config(
        "my-kolibrios-vm",
        "/var/lib/kolibrios/disk.qcow2",
        4096,  // 4GB RAM
        4,     // 4 CPUs
    );

    // Create and start VM
    let mut vm = QemuVm::new(config);
    vm.start().await.expect("Failed to start VM");
    
    println!("VM is running!");
    
    // Stop VM when done
    vm.stop().await.expect("Failed to stop VM");
}
```

### Using the Virtual Machine Manager

```rust
use kolibrios_vm::vmm::VirtualMachineManager;
use kolibrios_vm::QemuConfig;

#[tokio::main]
async fn main() {
    let vmm = VirtualMachineManager::new();
    
    // Create a VM
    let config = QemuConfig {
        name: "test-vm".to_string(),
        cpu_count: 2,
        memory: kolibrios_vm::MemoryConfig {
            size_mb: 2048,
            ..Default::default()
        },
        ..Default::default()
    };
    
    let vm_id = vmm.create_vm(config).await.unwrap();
    
    // Start the VM
    vmm.start_vm(&vm_id).await.unwrap();
    
    // List all VMs
    let vms = vmm.list_vms().await;
    println!("Running VMs: {:?}", vms);
    
    // Get statistics
    let stats = vmm.get_stats().await;
    println!("Running count: {}", stats.running_count);
}
```

## Configuration

### VM Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `name` | VM name | "kolibrios-vm" |
| `machine` | Machine type (Q35, PC, Microvm) | Q35 |
| `cpu_count` | Number of vCPUs | 2 |
| `cpu_type` | CPU model (Host, Qemu64, etc.) | Host |
| `acceleration` | Acceleration type (KVM, HVF, TCG) | Auto |
| `memory.size_mb` | RAM size in MB | 1024 |
| `boot_device` | Boot device (Disk, Cdrom, Network) | Disk |

### Network Configuration

```rust
use kolibrios_vm::{NetworkConfig, NetworkBackend, PortForward};

let network = NetworkConfig {
    backend: NetworkBackend::User,
    device_model: "virtio-net-pci".to_string(),
    port_forwards: vec![
        PortForward {
            host_port: 8080,
            guest_port: 80,
            protocol: "tcp".to_string(),
        },
    ],
    ..Default::default()
};
```

### Storage Configuration

```rust
use kolibrios_vm::{StorageConfig, StorageInterface};

let storage = StorageConfig {
    file_path: "/var/lib/vm/disk.qcow2".to_string(),
    format: "qcow2".to_string(),
    interface: StorageInterface::Virtio,
    cache_mode: "writeback".to_string(),
    ..Default::default()
};
```

## Device Management

### Adding Network Device

```rust
use kolibrios_vm::device::NetworkDeviceBuilder;

let (device, backend) = NetworkDeviceBuilder::new("net0")
    .driver("virtio-net-pci")
    .auto_mac()
    .user_mode()
    .port_forward(8080, 80, "tcp")
    .build();
```

### Adding Storage Device

```rust
use kolibrios_vm::device::StorageDeviceBuilder;

let (device, drive) = StorageDeviceBuilder::new("disk0")
    .file("/var/lib/vm/disk.qcow2")
    .format("qcow2")
    .cache("writeback")
    .build();
```

### Adding USB Device

```rust
use kolibrios_vm::device::UsbDeviceBuilder;

let tablet = UsbDeviceBuilder::new("usb0")
    .tablet()
    .build();
```

## Memory Management

### Using Hugepages

```rust
use kolibrios_vm::memory::{MemoryRegion, HugepageSize};

let region = MemoryRegion::new_hugepages(
    "hugepage-mem",
    1024 * 1024 * 1024,  // 1GB
    HugepageSize::Huge2M,
);
```

### Memory Ballooning

```rust
use kolibrios_vm::memory::MemoryBalloon;

let mut balloon = MemoryBalloon::new("balloon0", 1024 * 1024 * 1024);

// Inflate to reclaim memory
balloon.inflate(512 * 1024 * 1024).unwrap();

// Deflate to give memory back
balloon.deflate(256 * 1024 * 1024).unwrap();
```

## CPU Management

### CPU Topology

```rust
use kolibrios_vm::cpu::{VcpuConfig, CpuTopology};

let config = VcpuConfig::new("host", 8)
    .with_topology(2, 4, 1);  // 2 sockets, 4 cores, 1 thread
```

### CPU Hotplug

```rust
use kolibrios_vm::cpu::VirtualCpuManager;

let manager = VirtualCpuManager::new(config);

// Hotplug a new CPU
let new_cpu_id = manager.hotplug_cpu().await.unwrap();

// Hotunplug a CPU
manager.hotunplug_cpu(new_cpu_id).await.unwrap();
```

## QEMU Monitor

The QEMU monitor provides full control over the VM:

```rust
use kolibrios_vm::QemuMonitor;

let mut monitor = QemuMonitor::new();

// Connect to monitor
monitor.connect_tcp(4444).await.unwrap();

// Pause VM
monitor.pause().await.unwrap();

// Resume VM
monitor.resume().await.unwrap();

// Get status
let status = monitor.status().await.unwrap();
println!("VM Status: {}", status);

// Save VM state
monitor.save_vm("snapshot1").await.unwrap();

// Load VM state
monitor.load_vm("snapshot1").await.unwrap();
```

## Disk Image Management

```rust
use kolibrios_vm::QemuImage;

// Create a new disk image
QemuImage::create(
    "/var/lib/vm/disk.qcow2",
    "10G",
    "qcow2"
).await.unwrap();

// Convert disk image format
QemuImage::convert(
    "/var/lib/vm/disk.raw",
    "/var/lib/vm/disk.qcow2",
    "qcow2"
).await.unwrap();

// Get disk info
let info = QemuImage::info("/var/lib/vm/disk.qcow2").await.unwrap();
println!("Disk info: {:?}", info);

// Resize disk
QemuImage::resize("/var/lib/vm/disk.qcow2", "20G").await.unwrap();
```

## Security Considerations

1. **Sandboxing**: Run VMs with minimal privileges
2. **Network Isolation**: Use bridge or TAP networking for isolation
3. **Resource Limits**: Set memory and CPU limits
4. **Secure Boot**: Use UEFI with secure boot

## Troubleshooting

### KVM Permission Denied

```bash
sudo usermod -aG kvm $USER
# Log out and log back in
```

### Hugepages Not Available

```bash
# Allocate hugepages
echo 1024 | sudo tee /proc/sys/vm/nr_hugepages

# Make persistent
echo "vm.nr_hugepages = 1024" | sudo tee -a /etc/sysctl.conf
```

### VM Fails to Start

1. Check QEMU is installed: `qemu-system-x86_64 --version`
2. Check KVM is available: `ls -la /dev/kvm`
3. Check logs: `journalctl -xe`

## License

MIT License - See LICENSE file for details.
