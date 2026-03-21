//! Virtual Device Management for KolibriOS AI VMs.
//!
//! Provides device configuration and hotplug support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Virtual Device Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    /// Network interface
    Network,
    /// Storage device
    Storage,
    /// USB device
    Usb,
    /// PCI device
    Pci,
    /// GPU device
    Gpu,
    /// Audio device
    Audio,
    /// Serial port
    Serial,
    /// Input device (keyboard, mouse)
    Input,
    /// TPM device
    Tpm,
    /// RNG device
    Rng,
    /// Custom device
    Custom(String),
}

/// Device State
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceState {
    /// Device is attached and active
    Attached,
    /// Device is detached
    Detached,
    /// Device is in error state
    Error(String),
    /// Device is being hotplugged
    Hotplugging,
}

/// Virtual Device Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    /// Device ID
    pub id: String,
    /// Device type
    pub device_type: DeviceType,
    /// Device driver/model
    pub driver: String,
    /// Device properties
    pub properties: HashMap<String, String>,
    /// Is device hotpluggable
    pub hotpluggable: bool,
    /// Current state
    pub state: DeviceState,
}

impl DeviceConfig {
    /// Create a new device configuration
    pub fn new(id: &str, device_type: DeviceType, driver: &str) -> Self {
        DeviceConfig {
            id: id.to_string(),
            device_type,
            driver: driver.to_string(),
            properties: HashMap::new(),
            hotpluggable: false,
            state: DeviceState::Detached,
        }
    }

    /// Add a property
    pub fn with_property(mut self, key: &str, value: &str) -> Self {
        self.properties.insert(key.to_string(), value.to_string());
        self
    }

    /// Set hotpluggable
    pub fn with_hotplug(mut self, hotpluggable: bool) -> Self {
        self.hotpluggable = hotpluggable;
        self
    }

    /// Convert to QEMU arguments
    pub fn to_qemu_args(&self) -> Vec<String> {
        let mut args = vec!["-device".to_string()];

        let mut device_str = self.driver.clone();

        for (key, value) in &self.properties {
            device_str.push_str(&format!(",{}={}", key, value));
        }

        device_str.push_str(&format!(",id={}", self.id));

        args.push(device_str);
        args
    }
}

/// Network Device Builder
pub struct NetworkDeviceBuilder {
    config: DeviceConfig,
    backend_config: Option<NetdevConfig>,
}

/// Network Backend Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetdevConfig {
    /// Backend type
    pub backend_type: NetdevType,
    /// Backend ID
    pub id: String,
    /// Additional options
    pub options: HashMap<String, String>,
}

/// Network Backend Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetdevType {
    User,
    Tap,
    Bridge,
    Socket,
    Vde,
}

impl NetworkDeviceBuilder {
    /// Create a new network device builder
    pub fn new(id: &str) -> Self {
        NetworkDeviceBuilder {
            config: DeviceConfig::new(id, DeviceType::Network, "virtio-net-pci"),
            backend_config: None,
        }
    }

    /// Set driver
    pub fn driver(mut self, driver: &str) -> Self {
        self.config.driver = driver.to_string();
        self
    }

    /// Set MAC address
    pub fn mac_address(mut self, mac: &str) -> Self {
        self.config.properties.insert("mac".to_string(), mac.to_string());
        self
    }

    /// Auto-generate MAC address
    pub fn auto_mac(mut self) -> Self {
        // Generate a random locally administered MAC
        let mac = format!(
            "52:54:00:{:02x}:{:02x}:{:02x}",
            rand::random::<u8>(),
            rand::random::<u8>(),
            rand::random::<u8>()
        );
        self.config.properties.insert("mac".to_string(), mac);
        self
    }

    /// Use user-mode networking
    pub fn user_mode(mut self) -> Self {
        let netdev_id = format!("netdev-{}", self.config.id);
        self.config.properties.insert("netdev".to_string(), netdev_id.clone());
        self.backend_config = Some(NetdevConfig {
            backend_type: NetdevType::User,
            id: netdev_id,
            options: HashMap::new(),
        });
        self
    }

    /// Add port forwarding
    pub fn port_forward(mut self, host_port: u16, guest_port: u16, protocol: &str) -> Self {
        if let Some(ref mut backend) = self.backend_config {
            if matches!(backend.backend_type, NetdevType::User) {
                let key = format!("hostfwd_{}_{}", host_port, guest_port);
                let value = format!("{}::{}-:{}", protocol, host_port, guest_port);
                backend.options.insert(key, value);
            }
        }
        self
    }

    /// Build the device configuration
    pub fn build(self) -> (DeviceConfig, Option<NetdevConfig>) {
        (self.config, self.backend_config)
    }
}

/// Storage Device Builder
pub struct StorageDeviceBuilder {
    config: DeviceConfig,
    drive_config: Option<DriveConfig>,
}

/// Drive Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveConfig {
    /// Drive ID
    pub id: String,
    /// File path
    pub file: String,
    /// Format (qcow2, raw, etc.)
    pub format: String,
    /// Interface (virtio, ide, scsi)
    pub interface: String,
    /// Cache mode
    pub cache: String,
    /// Is read-only
    pub readonly: bool,
}

impl StorageDeviceBuilder {
    /// Create a new storage device builder
    pub fn new(id: &str) -> Self {
        StorageDeviceBuilder {
            config: DeviceConfig::new(id, DeviceType::Storage, "virtio-blk-pci"),
            drive_config: None,
        }
    }

    /// Set file path
    pub fn file(mut self, path: &str) -> Self {
        if let Some(ref mut drive) = self.drive_config {
            drive.file = path.to_string();
        } else {
            self.drive_config = Some(DriveConfig {
                id: format!("drive-{}", self.config.id),
                file: path.to_string(),
                format: "qcow2".to_string(),
                interface: "virtio".to_string(),
                cache: "writeback".to_string(),
                readonly: false,
            });
            self.config.properties.insert(
                "drive".to_string(),
                format!("drive-{}", self.config.id),
            );
        }
        self
    }

    /// Set format
    pub fn format(mut self, format: &str) -> Self {
        if let Some(ref mut drive) = self.drive_config {
            drive.format = format.to_string();
        }
        self
    }

    /// Set cache mode
    pub fn cache(mut self, cache: &str) -> Self {
        if let Some(ref mut drive) = self.drive_config {
            drive.cache = cache.to_string();
        }
        self
    }

    /// Set read-only
    pub fn readonly(mut self, readonly: bool) -> Self {
        if let Some(ref mut drive) = self.drive_config {
            drive.readonly = readonly;
        }
        self
    }

    /// Build the device configuration
    pub fn build(self) -> (DeviceConfig, Option<DriveConfig>) {
        (self.config, self.drive_config)
    }
}

/// USB Device Builder
pub struct UsbDeviceBuilder {
    config: DeviceConfig,
}

impl UsbDeviceBuilder {
    /// Create a new USB device builder
    pub fn new(id: &str) -> Self {
        UsbDeviceBuilder {
            config: DeviceConfig::new(id, DeviceType::Usb, "usb-tablet"),
        }
    }

    /// Set device type to tablet
    pub fn tablet(mut self) -> Self {
        self.config.driver = "usb-tablet".to_string();
        self
    }

    /// Set device type to keyboard
    pub fn keyboard(mut self) -> Self {
        self.config.driver = "usb-kbd".to_string();
        self
    }

    /// Set device type to mouse
    pub fn mouse(mut self) -> Self {
        self.config.driver = "usb-mouse".to_string();
        self
    }

    /// Pass through host USB device
    pub fn host_device(mut self, vendor_id: u16, product_id: u16) -> Self {
        self.config.driver = "usb-host".to_string();
        self.config.properties.insert(
            "vendorid".to_string(),
            format!("0x{:04x}", vendor_id),
        );
        self.config.properties.insert(
            "productid".to_string(),
            format!("0x{:04x}", product_id),
        );
        self
    }

    /// Build the device configuration
    pub fn build(self) -> DeviceConfig {
        self.config
    }
}

/// GPU Device Builder
pub struct GpuDeviceBuilder {
    config: DeviceConfig,
}

impl GpuDeviceBuilder {
    /// Create a new GPU device builder
    pub fn new(id: &str) -> Self {
        GpuDeviceBuilder {
            config: DeviceConfig::new(id, DeviceType::Gpu, "virtio-vga"),
        }
    }

    /// Use virtio-gpu
    pub fn virtio(mut self) -> Self {
        self.config.driver = "virtio-gpu-pci".to_string();
        self
    }

    /// Use QXL
    pub fn qxl(mut self) -> Self {
        self.config.driver = "qxl-vga".to_string();
        self
    }

    /// Enable OpenGL
    pub fn with_gl(mut self) -> Self {
        self.config.properties.insert("gl".to_string(), "on".to_string());
        self
    }

    /// Build the device configuration
    pub fn build(self) -> DeviceConfig {
        self.config
    }
}

/// Device Manager for managing attached devices
pub struct DeviceManager {
    devices: HashMap<String, DeviceConfig>,
}

impl DeviceManager {
    /// Create a new device manager
    pub fn new() -> Self {
        DeviceManager {
            devices: HashMap::new(),
        }
    }

    /// Attach a device
    pub fn attach(&mut self, config: DeviceConfig) {
        self.devices.insert(config.id.clone(), config);
    }

    /// Detach a device
    pub fn detach(&mut self, device_id: &str) -> Option<DeviceConfig> {
        self.devices.remove(device_id)
    }

    /// Get device by ID
    pub fn get(&self, device_id: &str) -> Option<&DeviceConfig> {
        self.devices.get(device_id)
    }

    /// List all devices
    pub fn list(&self) -> Vec<&DeviceConfig> {
        self.devices.values().collect()
    }

    /// List devices by type
    pub fn list_by_type(&self, device_type: DeviceType) -> Vec<&DeviceConfig> {
        self.devices
            .values()
            .filter(|d| d.device_type == device_type)
            .collect()
    }

    /// Get all QEMU arguments
    pub fn to_qemu_args(&self) -> Vec<String> {
        self.devices.values().flat_map(|d| d.to_qemu_args()).collect()
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_device_builder() {
        let (device, backend) = NetworkDeviceBuilder::new("net0")
            .driver("virtio-net-pci")
            .auto_mac()
            .user_mode()
            .port_forward(8080, 80, "tcp")
            .build();

        assert_eq!(device.device_type, DeviceType::Network);
        assert!(backend.is_some());
    }

    #[test]
    fn test_storage_device_builder() {
        let (device, drive) = StorageDeviceBuilder::new("disk0")
            .file("/var/lib/vm/disk.qcow2")
            .format("qcow2")
            .cache("writeback")
            .build();

        assert_eq!(device.device_type, DeviceType::Storage);
        assert!(drive.is_some());
    }

    #[test]
    fn test_device_manager() {
        let mut manager = DeviceManager::new();

        let net_device = NetworkDeviceBuilder::new("net0")
            .driver("virtio-net-pci")
            .build().0;

        manager.attach(net_device);

        assert_eq!(manager.list().len(), 1);
    }
}
