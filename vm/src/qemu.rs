//! QEMU Integration for KolibriOS AI.
//!
//! Provides Rust bindings and control interface for QEMU virtualization.

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// QEMU Machine Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MachineType {
    /// Standard PC (i440FX + PIIX)
    Pc,
    /// Q35 Chipset
    Q35,
    /// Microvm (minimal)
    Microvm,
    /// Custom machine type
    Custom(String),
}

impl Default for MachineType {
    fn default() -> Self {
        MachineType::Q35
    }
}

/// CPU Types for QEMU
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CpuType {
    /// Host CPU passthrough
    Host,
    /// QEMU TCG (software emulation)
    Qemu64,
    /// kvm64 (for KVM)
    Kvm64,
    /// Custom CPU model
    Custom(String),
}

impl Default for CpuType {
    fn default() -> Self {
        CpuType::Host
    }
}

/// QEMU Acceleration Type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Acceleration {
    /// Kernel-based Virtual Machine (Linux)
    Kvm,
    /// Hardware-Assisted Virtualization (macOS)
    Hvf,
    /// Windows Hypervisor Platform
    Whpx,
    /// Software emulation (no acceleration)
    Tcg,
    /// Auto-detect best available
    Auto,
}

impl Default for Acceleration {
    fn default() -> Self {
        Acceleration::Auto
    }
}

/// Boot Device Options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BootDevice {
    /// Boot from hard disk
    Disk,
    /// Boot from CD-ROM
    Cdrom,
    /// Boot from network (PXE)
    Network,
    /// Boot from floppy
    Floppy,
    /// Boot order string (e.g., "cdn")
    Order(String),
}

impl Default for BootDevice {
    fn default() -> Self {
        BootDevice::Disk
    }
}

/// Network Device Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network backend type
    pub backend: NetworkBackend,
    /// Device model
    pub device_model: String,
    /// MAC address (auto-generated if None)
    pub mac_address: Option<String>,
    /// Network name/id
    pub netdev_id: Option<String>,
    /// Port forwarding rules
    pub port_forwards: Vec<PortForward>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            backend: NetworkBackend::User,
            device_model: "virtio-net-pci".to_string(),
            mac_address: None,
            netdev_id: Some("net0".to_string()),
            port_forwards: vec![],
        }
    }
}

/// Network Backend Type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkBackend {
    /// User-mode networking (SLIRP)
    User,
    /// TAP device
    Tap,
    /// Bridge
    Bridge,
    /// Socket
    Socket,
    /// VDE
    Vde,
    /// None
    None,
}

/// Port Forwarding Rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForward {
    /// Host port
    pub host_port: u16,
    /// Guest port
    pub guest_port: u16,
    /// Protocol (tcp/udp)
    pub protocol: String,
}

/// Storage Device Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Drive file path
    pub file_path: String,
    /// Drive format (qcow2, raw, vmdk, etc.)
    pub format: String,
    /// Interface type
    pub interface: StorageInterface,
    /// Read-only mode
    pub read_only: bool,
    /// Cache mode
    pub cache_mode: String,
    /// Drive ID
    pub drive_id: Option<String>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig {
            file_path: String::new(),
            format: "qcow2".to_string(),
            interface: StorageInterface::Virtio,
            read_only: false,
            cache_mode: "writeback".to_string(),
            drive_id: None,
        }
    }
}

/// Storage Interface Type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageInterface {
    Ide,
    Scsi,
    Virtio,
    Nvme,
    Sd,
    Mmc,
    Usb,
}

/// Memory Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// RAM size in MB
    pub size_mb: u64,
    /// Enable hugepages
    pub hugepages: bool,
    /// Memory balloon device
    pub balloon: bool,
    /// Memory slots (for hotplug)
    pub slots: u32,
    /// Maximum memory
    pub max_size_mb: Option<u64>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        MemoryConfig {
            size_mb: 1024,
            hugepages: false,
            balloon: true,
            slots: 4,
            max_size_mb: None,
        }
    }
}

/// Display Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Display type
    pub display_type: DisplayType,
    /// VNC display number
    pub vnc_display: Option<i32>,
    /// SPICE port
    pub spice_port: Option<u16>,
    /// Enable GL acceleration
    pub gl: bool,
    /// Window size (width, height)
    pub window_size: Option<(u32, u32)>,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        DisplayConfig {
            display_type: DisplayType::Default,
            vnc_display: None,
            spice_port: None,
            gl: false,
            window_size: None,
        }
    }
}

/// Display Type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayType {
    /// Default SDL/GTK window
    Default,
    /// VNC server
    Vnc,
    /// SPICE server
    Spice,
    /// Headless (no display)
    None,
    /// GTK window
    Gtk,
    /// SDL window
    Sdl,
    /// Cocoa (macOS)
    Cocoa,
}

/// QEMU VM Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QemuConfig {
    /// VM name
    pub name: String,
    /// Machine type
    pub machine: MachineType,
    /// Number of CPU cores
    pub cpu_count: u32,
    /// CPU type
    pub cpu_type: CpuType,
    /// Acceleration type
    pub acceleration: Acceleration,
    /// Memory configuration
    pub memory: MemoryConfig,
    /// Boot device
    pub boot_device: BootDevice,
    /// Storage devices
    pub storage: Vec<StorageConfig>,
    /// Network devices
    pub network: Vec<NetworkConfig>,
    /// Display configuration
    pub display: DisplayConfig,
    /// Additional QEMU arguments
    pub extra_args: Vec<String>,
    /// BIOS/UEFI firmware path
    pub firmware: Option<String>,
    /// Enable QEMU Monitor
    pub monitor: MonitorConfig,
    /// Serial console configuration
    pub serial: SerialConfig,
    /// QEMU binary path
    pub qemu_binary: String,
}

impl Default for QemuConfig {
    fn default() -> Self {
        QemuConfig {
            name: "kolibrios-vm".to_string(),
            machine: MachineType::Q35,
            cpu_count: 2,
            cpu_type: CpuType::Host,
            acceleration: Acceleration::Auto,
            memory: MemoryConfig::default(),
            boot_device: BootDevice::Disk,
            storage: vec![],
            network: vec![NetworkConfig::default()],
            display: DisplayConfig::default(),
            extra_args: vec![],
            firmware: None,
            monitor: MonitorConfig::default(),
            serial: SerialConfig::default(),
            qemu_binary: "qemu-system-x86_64".to_string(),
        }
    }
}

/// QEMU Monitor Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Enable monitor
    pub enabled: bool,
    /// Monitor type
    pub monitor_type: MonitorType,
    /// Port for TCP monitor
    pub tcp_port: Option<u16>,
    /// Socket path for Unix socket
    pub socket_path: Option<String>,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        MonitorConfig {
            enabled: true,
            monitor_type: MonitorType::Stdio,
            tcp_port: None,
            socket_path: None,
        }
    }
}

/// Monitor Type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitorType {
    Stdio,
    Tcp,
    UnixSocket,
    None,
}

/// Serial Console Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialConfig {
    /// Serial type
    pub serial_type: SerialType,
    /// Port for TCP serial
    pub tcp_port: Option<u16>,
    /// File path for file output
    pub file_path: Option<String>,
}

impl Default for SerialConfig {
    fn default() -> Self {
        SerialConfig {
            serial_type: SerialType::Stdio,
            tcp_port: None,
            file_path: None,
        }
    }
}

/// Serial Type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerialType {
    Stdio,
    Tcp,
    File,
    UnixSocket,
    Pty,
    None,
}

/// QEMU VM Instance
pub struct QemuVm {
    /// VM configuration
    config: QemuConfig,
    /// VM process handle
    process: Option<Arc<RwLock<Option<tokio::process::Child>>>>,
    /// VM state
    state: Arc<RwLock<VmState>>,
    /// Monitor connection
    monitor: Option<QemuMonitor>,
}

/// VM State
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VmState {
    /// VM is not running
    Stopped,
    /// VM is starting
    Starting,
    /// VM is running
    Running,
    /// VM is paused
    Paused,
    /// VM is shutting down
    ShuttingDown,
    /// VM has crashed
    Crashed(String),
}

impl Default for VmState {
    fn default() -> Self {
        VmState::Stopped
    }
}

/// QEMU Monitor Interface
pub struct QemuMonitor {
    /// Connection to QEMU monitor
    connection: Option<Arc<RwLock<tokio::net::TcpStream>>>,
}

impl QemuMonitor {
    /// Create a new monitor instance
    pub fn new() -> Self {
        QemuMonitor { connection: None }
    }

    /// Connect to QEMU monitor via TCP
    pub async fn connect_tcp(&mut self, port: u16) -> Result<(), QemuError> {
        let addr = format!("127.0.0.1:{}", port);
        let stream = tokio::net::TcpStream::connect(&addr).await
            .map_err(|e| QemuError::MonitorConnection(e.to_string()))?;
        
        self.connection = Some(Arc::new(RwLock::new(stream)));
        Ok(())
    }

    /// Send a command to the monitor
    pub async fn send_command(&self, command: &str) -> Result<String, QemuError> {
        if let Some(conn) = &self.connection {
            let mut stream = conn.write().await;
            let stream = stream.as_mut().ok_or(QemuError::MonitorNotConnected)?;
            
            use tokio::io::{AsyncWriteExt, AsyncReadExt};
            
            // Send command
            stream.write_all(format!("{}\n", command).as_bytes()).await
                .map_err(|e| QemuError::MonitorCommand(e.to_string()))?;
            
            // Read response
            let mut buffer = vec![0u8; 4096];
            let n = stream.read(&mut buffer).await
                .map_err(|e| QemuError::MonitorCommand(e.to_string()))?;
            
            Ok(String::from_utf8_lossy(&buffer[..n]).to_string())
        } else {
            Err(QemuError::MonitorNotConnected)
        }
    }

    /// Pause the VM
    pub async fn pause(&self) -> Result<(), QemuError> {
        self.send_command("stop").await?;
        Ok(())
    }

    /// Resume the VM
    pub async fn resume(&self) -> Result<(), QemuError> {
        self.send_command("cont").await?;
        Ok(())
    }

    /// Get VM status
    pub async fn status(&self) -> Result<String, QemuError> {
        self.send_command("info status").await
    }

    /// Save VM state
    pub async fn save_vm(&self, path: &str) -> Result<(), QemuError> {
        self.send_command(&format!("savevm {}", path)).await?;
        Ok(())
    }

    /// Load VM state
    pub async fn load_vm(&self, tag: &str) -> Result<(), QemuError> {
        self.send_command(&format!("loadvm {}", tag)).await?;
        Ok(())
    }
}

/// QEMU Error Types
#[derive(Debug, thiserror::Error)]
pub enum QemuError {
    #[error("QEMU binary not found: {0}")]
    BinaryNotFound(String),
    
    #[error("Failed to start QEMU: {0}")]
    StartFailed(String),
    
    #[error("VM not running")]
    VmNotRunning,
    
    #[error("Monitor connection failed: {0}")]
    MonitorConnection(String),
    
    #[error("Monitor command failed: {0}")]
    MonitorCommand(String),
    
    #[error("Monitor not connected")]
    MonitorNotConnected,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Timeout waiting for VM")]
    Timeout,
}

impl QemuVm {
    /// Create a new QEMU VM instance
    pub fn new(config: QemuConfig) -> Self {
        QemuVm {
            config,
            process: None,
            state: Arc::new(RwLock::new(VmState::Stopped)),
            monitor: Some(QemuMonitor::new()),
        }
    }

    /// Build QEMU command line arguments
    pub fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Machine type
        let machine = match &self.config.machine {
            MachineType::Pc => "pc",
            MachineType::Q35 => "q35",
            MachineType::Microvm => "microvm",
            MachineType::Custom(m) => m,
        };
        args.push("-machine".to_string());
        args.push(machine.to_string());

        // CPU configuration
        let cpu = match &self.config.cpu_type {
            CpuType::Host => "host",
            CpuType::Qemu64 => "qemu64",
            CpuType::Kvm64 => "kvm64",
            CpuType::Custom(c) => c,
        };
        args.push("-cpu".to_string());
        args.push(cpu.to_string());

        // CPU count
        args.push("-smp".to_string());
        args.push(self.config.cpu_count.to_string());

        // Memory
        args.push("-m".to_string());
        let mut mem_arg = format!("{}M", self.config.memory.size_mb);
        if let Some(max) = self.config.memory.max_size_mb {
            mem_arg = format!("{}M,slots={},maxmem={}M", 
                self.config.memory.size_mb,
                self.config.memory.slots,
                max
            );
        }
        args.push(mem_arg);

        // Acceleration
        let accel = match &self.config.acceleration {
            Acceleration::Kvm => "kvm",
            Acceleration::Hvf => "hvf",
            Acceleration::Whpx => "whpx",
            Acceleration::Tcg => "tcg",
            Acceleration::Auto => "kvm:tcg", // Try KVM, fallback to TCG
        };
        args.push("-accel".to_string());
        args.push(accel.to_string());

        // Name
        args.push("-name".to_string());
        args.push(self.config.name.clone());

        // Boot device
        let boot = match &self.config.boot_device {
            BootDevice::Disk => "c",
            BootDevice::Cdrom => "d",
            BootDevice::Network => "n",
            BootDevice::Floppy => "a",
            BootDevice::Order(o) => o,
        };
        args.push("-boot".to_string());
        args.push(format!("order={}", boot));

        // Storage devices
        for (i, storage) in self.config.storage.iter().enumerate() {
            let drive_id = storage.drive_id.clone()
                .unwrap_or(format!("drive{}", i));
            
            args.push("-drive".to_string());
            let mut drive_arg = format!(
                "file={},format={},if={},id={}",
                storage.file_path,
                storage.format,
                match storage.interface {
                    StorageInterface::Ide => "ide",
                    StorageInterface::Scsi => "scsi",
                    StorageInterface::Virtio => "virtio",
                    StorageInterface::Nvme => "nvme",
                    StorageInterface::Sd => "sd",
                    StorageInterface::Mmc => "mmc",
                    StorageInterface::Usb => "usb",
                },
                drive_id
            );

            if storage.read_only {
                drive_arg.push_str(",readonly=on");
            }
            drive_arg.push_str(&format!(",cache={}", storage.cache_mode));
            
            args.push(drive_arg);
        }

        // Network devices
        for (i, net) in self.config.network.iter().enumerate() {
            let netdev_id = net.netdev_id.clone()
                .unwrap_or(format!("net{}", i));
            
            // Network backend
            let backend_arg = match &net.backend {
                NetworkBackend::User => {
                    let mut user_arg = "user".to_string();
                    for fwd in &net.port_forwards {
                        user_arg.push_str(&format!(
                            ",hostfwd={}::{}-:{}",
                            fwd.protocol,
                            fwd.host_port,
                            fwd.guest_port
                        ));
                    }
                    user_arg
                }
                NetworkBackend::Tap => "tap".to_string(),
                NetworkBackend::Bridge => "bridge".to_string(),
                NetworkBackend::Socket => "socket".to_string(),
                NetworkBackend::Vde => "vde".to_string(),
                NetworkBackend::None => "none".to_string(),
            };

            args.push("-netdev".to_string());
            args.push(format!("{},id={}", backend_arg, netdev_id));

            // Network device
            let mut device_arg = format!("{},netdev={}", net.device_model, netdev_id);
            if let Some(mac) = &net.mac_address {
                device_arg.push_str(&format!(",mac={}", mac));
            }
            args.push("-device".to_string());
            args.push(device_arg);
        }

        // Display
        match &self.config.display.display_type {
            DisplayType::None => {
                args.push("-display".to_string());
                args.push("none".to_string());
            }
            DisplayType::Vnc => {
                let display = self.config.display.vnc_display.unwrap_or(0);
                args.push("-vnc".to_string());
                args.push(format!(":{}", display));
            }
            DisplayType::Spice => {
                args.push("-spice".to_string());
                let port = self.config.display.spice_port.unwrap_or(5900);
                args.push(format!("port={}", port));
            }
            DisplayType::Default | DisplayType::Gtk | DisplayType::Sdl | DisplayType::Cocoa => {
                // Use default display
            }
        }

        // Firmware
        if let Some(fw) = &self.config.firmware {
            args.push("-bios".to_string());
            args.push(fw.clone());
        }

        // Monitor
        if self.config.monitor.enabled {
            match self.config.monitor.monitor_type {
                MonitorType::Tcp => {
                    let port = self.config.monitor.tcp_port.unwrap_or(4444);
                    args.push("-monitor".to_string());
                    args.push(format!("tcp:127.0.0.1:{},server,nowait", port));
                }
                MonitorType::UnixSocket => {
                    if let Some(path) = &self.config.monitor.socket_path {
                        args.push("-monitor".to_string());
                        args.push(format!("unix:{},server,nowait", path));
                    }
                }
                MonitorType::Stdio => {
                    args.push("-monitor".to_string());
                    args.push("stdio".to_string());
                }
                MonitorType::None => {}
            }
        }

        // Serial console
        match self.config.serial.serial_type {
            SerialType::Tcp => {
                let port = self.config.serial.tcp_port.unwrap_or(4445);
                args.push("-serial".to_string());
                args.push(format!("tcp:127.0.0.1:{},server,nowait", port));
            }
            SerialType::File => {
                if let Some(path) = &self.config.serial.file_path {
                    args.push("-serial".to_string());
                    args.push(format!("file:{}", path));
                }
            }
            SerialType::Stdio => {
                args.push("-serial".to_string());
                args.push("stdio".to_string());
            }
            SerialType::Pty => {
                args.push("-serial".to_string());
                args.push("pty".to_string());
            }
            SerialType::UnixSocket => {}
            SerialType::None => {
                args.push("-serial".to_string());
                args.push("none".to_string());
            }
        }

        // Extra arguments
        args.extend(self.config.extra_args.clone());

        args
    }

    /// Start the VM
    pub async fn start(&mut self) -> Result<(), QemuError> {
        // Check if QEMU binary exists
        let qemu_path = which::which(&self.config.qemu_binary)
            .map_err(|_| QemuError::BinaryNotFound(self.config.qemu_binary.clone()))?;

        // Update state
        {
            let mut state = self.state.write().await;
            *state = VmState::Starting;
        }

        // Build arguments
        let args = self.build_args();
        
        // Start QEMU process
        let mut cmd = tokio::process::Command::new(qemu_path);
        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn()
            .map_err(|e| QemuError::StartFailed(e.to_string()))?;

        // Store process handle
        self.process = Some(Arc::new(RwLock::new(Some(child))));

        // Update state
        {
            let mut state = self.state.write().await;
            *state = VmState::Running;
        }

        Ok(())
    }

    /// Stop the VM
    pub async fn stop(&mut self) -> Result<(), QemuError> {
        if let Some(process) = &self.process {
            let mut child_guard = process.write().await;
            if let Some(child) = child_guard.as_mut() {
                // Try graceful shutdown first
                child.kill().await
                    .map_err(|e| QemuError::StartFailed(e.to_string()))?;
            }
            *child_guard = None;
        }

        // Update state
        {
            let mut state = self.state.write().await;
            *state = VmState::Stopped;
        }

        Ok(())
    }

    /// Pause the VM
    pub async fn pause(&self) -> Result<(), QemuError> {
        if let Some(monitor) = &self.monitor {
            monitor.pause().await?;
            let mut state = self.state.write().await;
            *state = VmState::Paused;
        }
        Ok(())
    }

    /// Resume the VM
    pub async fn resume(&self) -> Result<(), QemuError> {
        if let Some(monitor) = &self.monitor {
            monitor.resume().await?;
            let mut state = self.state.write().await;
            *state = VmState::Running;
        }
        Ok(())
    }

    /// Get current VM state
    pub async fn state(&self) -> VmState {
        self.state.read().await.clone()
    }

    /// Check if VM is running
    pub async fn is_running(&self) -> bool {
        matches!(self.state.read().await.as_ref(), VmState::Running)
    }

    /// Get monitor interface
    pub fn monitor(&self) -> Option<&QemuMonitor> {
        self.monitor.as_ref()
    }

    /// Wait for VM to stop
    pub async fn wait(&mut self) -> Result<i32, QemuError> {
        if let Some(process) = &self.process {
            let mut child_guard = process.write().await;
            if let Some(child) = child_guard.as_mut() {
                let status = child.wait().await
                    .map_err(QemuError::IoError)?;
                
                // Update state
                {
                    let mut state = self.state.write().await;
                    *state = VmState::Stopped;
                }
                
                return Ok(status.code().unwrap_or(-1));
            }
        }
        Err(QemuError::VmNotRunning)
    }
}

/// QEMU Disk Image Utilities
pub struct QemuImage;

impl QemuImage {
    /// Create a new disk image
    pub async fn create(
        file_path: &str,
        size: &str,
        format: &str,
    ) -> Result<(), QemuError> {
        let output = Command::new("qemu-img")
            .args(["create", "-f", format, file_path, size])
            .output()
            .map_err(QemuError::IoError)?;

        if !output.status.success() {
            return Err(QemuError::StartFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(())
    }

    /// Convert disk image format
    pub async fn convert(
        input_path: &str,
        output_path: &str,
        output_format: &str,
    ) -> Result<(), QemuError> {
        let output = Command::new("qemu-img")
            .args([
                "convert",
                "-O", output_format,
                input_path,
                output_path,
            ])
            .output()
            .map_err(QemuError::IoError)?;

        if !output.status.success() {
            return Err(QemuError::StartFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(())
    }

    /// Get disk image info
    pub async fn info(file_path: &str) -> Result<HashMap<String, String>, QemuError> {
        let output = Command::new("qemu-img")
            .args(["info", file_path])
            .output()
            .map_err(QemuError::IoError)?;

        if !output.status.success() {
            return Err(QemuError::StartFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        // Parse output
        let mut info = HashMap::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if let Some((key, value)) = line.split_once(':') {
                info.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Ok(info)
    }

    /// Resize disk image
    pub async fn resize(file_path: &str, new_size: &str) -> Result<(), QemuError> {
        let output = Command::new("qemu-img")
            .args(["resize", file_path, new_size])
            .output()
            .map_err(QemuError::IoError)?;

        if !output.status.success() {
            return Err(QemuError::StartFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(())
    }
}

/// Create a KolibriOS AI VM configuration
pub fn create_kolibrios_vm_config(
    name: &str,
    disk_path: &str,
    memory_mb: u64,
    cpu_count: u32,
) -> QemuConfig {
    QemuConfig {
        name: name.to_string(),
        machine: MachineType::Q35,
        cpu_count,
        cpu_type: CpuType::Host,
        acceleration: Acceleration::Auto,
        memory: MemoryConfig {
            size_mb: memory_mb,
            ..Default::default()
        },
        boot_device: BootDevice::Disk,
        storage: vec![StorageConfig {
            file_path: disk_path.to_string(),
            format: "qcow2".to_string(),
            interface: StorageInterface::Virtio,
            ..Default::default()
        }],
        network: vec![NetworkConfig {
            backend: NetworkBackend::User,
            device_model: "virtio-net-pci".to_string(),
            port_forwards: vec![
                PortForward {
                    host_port: 2222,
                    guest_port: 22,
                    protocol: "tcp".to_string(),
                },
            ],
            ..Default::default()
        }],
        display: DisplayConfig {
            display_type: DisplayType::Vnc,
            vnc_display: Some(0),
            ..Default::default()
        },
        monitor: MonitorConfig {
            enabled: true,
            monitor_type: MonitorType::Tcp,
            tcp_port: Some(4444),
            ..Default::default()
        },
        serial: SerialConfig {
            serial_type: SerialType::Tcp,
            tcp_port: Some(4445),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_args() {
        let config = QemuConfig {
            name: "test-vm".to_string(),
            cpu_count: 4,
            memory: MemoryConfig {
                size_mb: 2048,
                ..Default::default()
            },
            ..Default::default()
        };

        let vm = QemuVm::new(config);
        let args = vm.build_args();

        assert!(args.contains(&"-machine".to_string()));
        assert!(args.contains(&"q35".to_string()));
        assert!(args.contains(&"-smp".to_string()));
        assert!(args.contains(&"4".to_string()));
        assert!(args.contains(&"-m".to_string()));
    }

    #[test]
    fn test_create_kolibrios_config() {
        let config = create_kolibrios_vm_config(
            "kolibrios-test",
            "/var/lib/kolibrios/disk.qcow2",
            4096,
            4,
        );

        assert_eq!(config.name, "kolibrios-test");
        assert_eq!(config.memory.size_mb, 4096);
        assert_eq!(config.cpu_count, 4);
        assert!(!config.storage.is_empty());
    }
}
