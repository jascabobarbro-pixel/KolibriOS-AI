//! Sandbox System for KolibriOS AI
//!
//! This module implements a comprehensive sandboxing system for process isolation
//! and resource containment. It provides:
//! - Resource limits (memory, CPU, I/O)
//! - System call filtering and interception
//! - Filesystem isolation
//! - Network isolation
//! - Namespace isolation

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::RwLock;

extern crate alloc;

use super::capability::{Capability, CapabilityError, CapabilityId, Permission, ResourceDescriptor, SubjectId, Timestamp};

/// Unique identifier for a sandbox
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SandboxId(pub u64);

impl fmt::Display for SandboxId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SBX-{:016X}", self.0)
    }
}

/// Unique identifier for a sandboxed process
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SandboxedProcessId(pub u64);

impl fmt::Display for SandboxedProcessId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PROC-{:016X}", self.0)
    }
}

/// Resource limits for a sandbox
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceLimits {
    /// Maximum memory in bytes
    pub max_memory: u64,
    /// Current memory usage
    pub current_memory: u64,
    /// Maximum CPU time in milliseconds
    pub max_cpu_time: u64,
    /// Current CPU time used
    pub current_cpu_time: u64,
    /// Maximum number of processes/threads
    pub max_processes: u32,
    /// Current number of processes
    pub current_processes: u32,
    /// Maximum number of open files
    pub max_open_files: u32,
    /// Current number of open files
    pub current_open_files: u32,
    /// Maximum number of network connections
    pub max_network_connections: u32,
    /// Current number of network connections
    pub current_network_connections: u32,
    /// Maximum I/O operations per second
    pub max_io_ops_per_sec: u32,
    /// Maximum disk I/O bandwidth in bytes/sec
    pub max_disk_bandwidth: u64,
    /// Maximum network bandwidth in bytes/sec
    pub max_network_bandwidth: u64,
    /// Maximum file size in bytes
    pub max_file_size: u64,
    /// Maximum stack size in bytes
    pub max_stack_size: u64,
    /// Maximum data segment size in bytes
    pub max_data_size: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 256 * 1024 * 1024, // 256 MB
            current_memory: 0,
            max_cpu_time: 60_000, // 60 seconds
            current_cpu_time: 0,
            max_processes: 10,
            current_processes: 0,
            max_open_files: 100,
            current_open_files: 0,
            max_network_connections: 10,
            current_network_connections: 0,
            max_io_ops_per_sec: 1000,
            max_disk_bandwidth: 10 * 1024 * 1024, // 10 MB/s
            max_network_bandwidth: 5 * 1024 * 1024, // 5 MB/s
            max_file_size: 100 * 1024 * 1024, // 100 MB
            max_stack_size: 8 * 1024 * 1024, // 8 MB
            max_data_size: 64 * 1024 * 1024, // 64 MB
        }
    }
}

impl ResourceLimits {
    /// Create new resource limits with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create resource limits for a restricted sandbox
    pub fn restricted() -> Self {
        Self {
            max_memory: 64 * 1024 * 1024, // 64 MB
            current_memory: 0,
            max_cpu_time: 10_000, // 10 seconds
            current_cpu_time: 0,
            max_processes: 3,
            current_processes: 0,
            max_open_files: 20,
            current_open_files: 0,
            max_network_connections: 3,
            current_network_connections: 0,
            max_io_ops_per_sec: 100,
            max_disk_bandwidth: 1024 * 1024, // 1 MB/s
            max_network_bandwidth: 512 * 1024, // 512 KB/s
            max_file_size: 10 * 1024 * 1024, // 10 MB
            max_stack_size: 2 * 1024 * 1024, // 2 MB
            max_data_size: 16 * 1024 * 1024, // 16 MB
        }
    }

    /// Create resource limits for an unrestricted sandbox
    pub fn unrestricted() -> Self {
        Self {
            max_memory: u64::MAX,
            current_memory: 0,
            max_cpu_time: u64::MAX,
            current_cpu_time: 0,
            max_processes: 1024,
            current_processes: 0,
            max_open_files: 4096,
            current_open_files: 0,
            max_network_connections: 1024,
            current_network_connections: 0,
            max_io_ops_per_sec: u32::MAX,
            max_disk_bandwidth: u64::MAX,
            max_network_bandwidth: u64::MAX,
            max_file_size: u64::MAX,
            max_stack_size: u64::MAX,
            max_data_size: u64::MAX,
        }
    }

    /// Check if memory limit is exceeded
    pub const fn memory_exceeded(&self) -> bool {
        self.current_memory > self.max_memory
    }

    /// Check if CPU time limit is exceeded
    pub const fn cpu_time_exceeded(&self) -> bool {
        self.current_cpu_time > self.max_cpu_time
    }

    /// Check if process limit is exceeded
    pub const fn process_limit_exceeded(&self) -> bool {
        self.current_processes >= self.max_processes
    }

    /// Check if file descriptor limit is exceeded
    pub const fn file_limit_exceeded(&self) -> bool {
        self.current_open_files >= self.max_open_files
    }

    /// Check if network connection limit is exceeded
    pub const fn network_limit_exceeded(&self) -> bool {
        self.current_network_connections >= self.max_network_connections
    }

    /// Allocate memory (returns error if limit exceeded)
    pub fn allocate_memory(&mut self, size: u64) -> Result<(), SandboxError> {
        let new_total = self.current_memory.saturating_add(size);
        if new_total > self.max_memory {
            return Err(SandboxError::ResourceLimitExceeded {
                resource: String::from("memory"),
                limit: self.max_memory,
                requested: size,
            });
        }
        self.current_memory = new_total;
        Ok(())
    }

    /// Free memory
    pub fn free_memory(&mut self, size: u64) {
        self.current_memory = self.current_memory.saturating_sub(size);
    }

    /// Add CPU time
    pub fn add_cpu_time(&mut self, ms: u64) -> Result<(), SandboxError> {
        let new_total = self.current_cpu_time.saturating_add(ms);
        if new_total > self.max_cpu_time {
            return Err(SandboxError::ResourceLimitExceeded {
                resource: String::from("cpu_time"),
                limit: self.max_cpu_time,
                requested: ms,
            });
        }
        self.current_cpu_time = new_total;
        Ok(())
    }

    /// Increment process count
    pub fn increment_processes(&mut self) -> Result<(), SandboxError> {
        if self.process_limit_exceeded() {
            return Err(SandboxError::ResourceLimitExceeded {
                resource: String::from("processes"),
                limit: self.max_processes as u64,
                requested: 1,
            });
        }
        self.current_processes += 1;
        Ok(())
    }

    /// Decrement process count
    pub fn decrement_processes(&mut self) {
        self.current_processes = self.current_processes.saturating_sub(1);
    }

    /// Increment open files
    pub fn increment_open_files(&mut self) -> Result<(), SandboxError> {
        if self.file_limit_exceeded() {
            return Err(SandboxError::ResourceLimitExceeded {
                resource: String::from("open_files"),
                limit: self.max_open_files as u64,
                requested: 1,
            });
        }
        self.current_open_files += 1;
        Ok(())
    }

    /// Decrement open files
    pub fn decrement_open_files(&mut self) {
        self.current_open_files = self.current_open_files.saturating_sub(1);
    }

    /// Increment network connections
    pub fn increment_network_connections(&mut self) -> Result<(), SandboxError> {
        if self.network_limit_exceeded() {
            return Err(SandboxError::ResourceLimitExceeded {
                resource: String::from("network_connections"),
                limit: self.max_network_connections as u64,
                requested: 1,
            });
        }
        self.current_network_connections += 1;
        Ok(())
    }

    /// Decrement network connections
    pub fn decrement_network_connections(&mut self) {
        self.current_network_connections = self.current_network_connections.saturating_sub(1);
    }

    /// Get memory usage percentage
    pub fn memory_usage_percent(&self) -> f32 {
        if self.max_memory == 0 || self.max_memory == u64::MAX {
            return 0.0;
        }
        (self.current_memory as f32 / self.max_memory as f32) * 100.0
    }

    /// Get CPU time usage percentage
    pub fn cpu_usage_percent(&self) -> f32 {
        if self.max_cpu_time == 0 || self.max_cpu_time == u64::MAX {
            return 0.0;
        }
        (self.current_cpu_time as f32 / self.max_cpu_time as f32) * 100.0
    }
}

/// System call filter rule
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyscallFilterRule {
    /// System call number
    pub syscall_number: u32,
    /// Action to take
    pub action: SyscallAction,
    /// Optional error code to return
    pub error_code: Option<i32>,
    /// Optional log message
    pub log_message: Option<String>,
}

/// Action to take for a system call
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallAction {
    /// Allow the system call
    Allow,
    /// Deny the system call
    Deny,
    /// Allow with logging
    Log,
    /// Notify and wait for decision
    Notify,
    /// Redirect to a different handler
    Redirect(u32),
}

impl SyscallFilterRule {
    /// Create a new syscall filter rule
    pub const fn new(syscall_number: u32, action: SyscallAction) -> Self {
        Self {
            syscall_number,
            action,
            error_code: None,
            log_message: None,
        }
    }

    /// Create an allow rule
    pub const fn allow(syscall_number: u32) -> Self {
        Self::new(syscall_number, SyscallAction::Allow)
    }

    /// Create a deny rule
    pub const fn deny(syscall_number: u32) -> Self {
        Self::new(syscall_number, SyscallAction::Deny)
    }

    /// Create a log rule
    pub const fn log(syscall_number: u32) -> Self {
        Self::new(syscall_number, SyscallAction::Log)
    }

    /// Create a notify rule
    pub const fn notify(syscall_number: u32) -> Self {
        Self::new(syscall_number, SyscallAction::Notify)
    }

    /// Add error code
    pub fn with_error(mut self, code: i32) -> Self {
        self.error_code = Some(code);
        self
    }

    /// Add log message
    pub fn with_log_message(mut self, msg: &str) -> Self {
        self.log_message = Some(String::from(msg));
        self
    }
}

/// System call filter
#[derive(Debug, Clone)]
pub struct SyscallFilter {
    /// Default action for unknown syscalls
    pub default_action: SyscallAction,
    /// Specific syscall rules
    pub rules: BTreeMap<u32, SyscallFilterRule>,
    /// Whether to log all syscalls
    pub log_all: bool,
}

impl Default for SyscallFilter {
    fn default() -> Self {
        Self {
            default_action: SyscallAction::Deny,
            rules: BTreeMap::new(),
            log_all: false,
        }
    }
}

impl SyscallFilter {
    /// Create a new syscall filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a permissive filter (allow all by default)
    pub fn permissive() -> Self {
        Self {
            default_action: SyscallAction::Allow,
            rules: BTreeMap::new(),
            log_all: false,
        }
    }

    /// Create a restrictive filter (deny all by default)
    pub fn restrictive() -> Self {
        Self {
            default_action: SyscallAction::Deny,
            rules: BTreeMap::new(),
            log_all: true,
        }
    }

    /// Add a rule to the filter
    pub fn add_rule(&mut self, rule: SyscallFilterRule) {
        self.rules.insert(rule.syscall_number, rule);
    }

    /// Remove a rule from the filter
    pub fn remove_rule(&mut self, syscall_number: u32) -> Option<SyscallFilterRule> {
        self.rules.remove(&syscall_number)
    }

    /// Check if a syscall is allowed
    pub fn check(&self, syscall_number: u32) -> SyscallAction {
        self.rules
            .get(&syscall_number)
            .map(|r| r.action)
            .unwrap_or(self.default_action)
    }

    /// Get the rule for a syscall
    pub fn get_rule(&self, syscall_number: u32) -> Option<&SyscallFilterRule> {
        self.rules.get(&syscall_number)
    }

    /// Create a basic filter for general processes
    pub fn basic() -> Self {
        let mut filter = Self::new();
        
        // Allow basic I/O
        filter.add_rule(SyscallFilterRule::allow(0));   // read
        filter.add_rule(SyscallFilterRule::allow(1));   // write
        filter.add_rule(SyscallFilterRule::allow(2));   // open
        filter.add_rule(SyscallFilterRule::allow(3));   // close
        filter.add_rule(SyscallFilterRule::allow(9));   // mmap
        filter.add_rule(SyscallFilterRule::allow(11));  // munmap
        filter.add_rule(SyscallFilterRule::allow(60));  // exit
        
        // Log dangerous syscalls
        filter.add_rule(SyscallFilterRule::log(39));    // getpid
        filter.add_rule(SyscallFilterRule::log(57));    // fork
        filter.add_rule(SyscallFilterRule::log(59));    // execve
        
        filter
    }

    /// Create a filter for network processes
    pub fn network() -> Self {
        let mut filter = Self::basic();
        
        // Allow network syscalls
        filter.add_rule(SyscallFilterRule::allow(41));  // socket
        filter.add_rule(SyscallFilterRule::allow(42));  // connect
        filter.add_rule(SyscallFilterRule::allow(43));  // accept
        filter.add_rule(SyscallFilterRule::allow(44));  // sendto
        filter.add_rule(SyscallFilterRule::allow(45));  // recvfrom
        filter.add_rule(SyscallFilterRule::allow(46));  // sendmsg
        filter.add_rule(SyscallFilterRule::allow(47));  // recvmsg
        filter.add_rule(SyscallFilterRule::allow(48));  // shutdown
        filter.add_rule(SyscallFilterRule::allow(49));  // bind
        filter.add_rule(SyscallFilterRule::allow(50));  // listen
        filter.add_rule(SyscallFilterRule::allow(51));  // getsockname
        filter.add_rule(SyscallFilterRule::allow(52));  // getpeername
        filter.add_rule(SyscallFilterRule::allow(53));  // socketpair
        filter.add_rule(SyscallFilterRule::allow(54));  // setsockopt
        filter.add_rule(SyscallFilterRule::allow(55));  // getsockopt
        
        filter
    }
}

/// Filesystem isolation configuration
#[derive(Debug, Clone)]
pub struct FilesystemIsolation {
    /// Root directory for the sandbox
    pub root: String,
    /// Whether to use chroot isolation
    pub use_chroot: bool,
    /// Mounted paths (source -> target)
    pub mounts: BTreeMap<String, String>,
    /// Read-only paths
    pub read_only: Vec<String>,
    /// Hidden paths (not accessible)
    pub hidden: Vec<String>,
    /// Whether to allow device access
    pub allow_devices: bool,
    /// Whether to allow proc filesystem
    pub allow_proc: bool,
    /// Whether to allow sys filesystem
    pub allow_sys: bool,
}

impl Default for FilesystemIsolation {
    fn default() -> Self {
        Self {
            root: String::from("/"),
            use_chroot: false,
            mounts: BTreeMap::new(),
            read_only: Vec::new(),
            hidden: Vec::new(),
            allow_devices: false,
            allow_proc: false,
            allow_sys: false,
        }
    }
}

impl FilesystemIsolation {
    /// Create a new filesystem isolation config
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a specific root
    pub fn with_root(root: &str) -> Self {
        Self {
            root: String::from(root),
            use_chroot: true,
            ..Self::default()
        }
    }

    /// Add a mount point
    pub fn add_mount(&mut self, source: &str, target: &str) {
        self.mounts.insert(String::from(source), String::from(target));
    }

    /// Add a read-only path
    pub fn add_read_only(&mut self, path: &str) {
        self.read_only.push(String::from(path));
    }

    /// Add a hidden path
    pub fn add_hidden(&mut self, path: &str) {
        self.hidden.push(String::from(path));
    }

    /// Check if a path is accessible
    pub fn is_path_accessible(&self, path: &str) -> bool {
        // Check hidden paths
        for hidden in &self.hidden {
            if path.starts_with(hidden) {
                return false;
            }
        }
        true
    }

    /// Check if a path is read-only
    pub fn is_path_read_only(&self, path: &str) -> bool {
        for ro in &self.read_only {
            if path.starts_with(ro) {
                return true;
            }
        }
        false
    }

    /// Translate a path to sandbox root
    pub fn translate_path(&self, path: &str) -> String {
        if self.use_chroot {
            let mut translated = self.root.clone();
            if !path.starts_with('/') {
                translated.push('/');
            }
            translated.push_str(path);
            translated
        } else {
            String::from(path)
        }
    }
}

/// Network isolation configuration
#[derive(Debug, Clone)]
pub struct NetworkIsolation {
    /// Whether networking is allowed
    pub allowed: bool,
    /// Allowed network namespaces
    pub namespaces: Vec<String>,
    /// Allowed IP addresses
    pub allowed_ips: Vec<String>,
    /// Denied IP addresses
    pub denied_ips: Vec<String>,
    /// Allowed ports
    pub allowed_ports: Vec<u16>,
    /// Denied ports
    pub denied_ports: Vec<u16>,
    /// Maximum bandwidth
    pub max_bandwidth: u64,
    /// Whether to allow IPv4
    pub allow_ipv4: bool,
    /// Whether to allow IPv6
    pub allow_ipv6: bool,
    /// Whether to allow TCP
    pub allow_tcp: bool,
    /// Whether to allow UDP
    pub allow_udp: bool,
    /// Whether to allow ICMP
    pub allow_icmp: bool,
}

impl Default for NetworkIsolation {
    fn default() -> Self {
        Self {
            allowed: true,
            namespaces: Vec::new(),
            allowed_ips: Vec::new(),
            denied_ips: Vec::new(),
            allowed_ports: Vec::new(),
            denied_ports: Vec::new(),
            max_bandwidth: u64::MAX,
            allow_ipv4: true,
            allow_ipv6: true,
            allow_tcp: true,
            allow_udp: true,
            allow_icmp: false,
        }
    }
}

impl NetworkIsolation {
    /// Create new network isolation config
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a no-network isolation
    pub fn no_network() -> Self {
        Self {
            allowed: false,
            ..Self::default()
        }
    }

    /// Add an allowed IP
    pub fn add_allowed_ip(&mut self, ip: &str) {
        self.allowed_ips.push(String::from(ip));
    }

    /// Add a denied IP
    pub fn add_denied_ip(&mut self, ip: &str) {
        self.denied_ips.push(String::from(ip));
    }

    /// Add an allowed port
    pub fn add_allowed_port(&mut self, port: u16) {
        self.allowed_ports.push(port);
    }

    /// Add a denied port
    pub fn add_denied_port(&mut self, port: u16) {
        self.denied_ports.push(port);
    }

    /// Check if IP is allowed
    pub fn is_ip_allowed(&self, ip: &str) -> bool {
        if !self.allowed {
            return false;
        }

        // Check denied list first
        for denied in &self.denied_ips {
            if ip.starts_with(denied) {
                return false;
            }
        }

        // If allowed list is empty, allow all non-denied
        if self.allowed_ips.is_empty() {
            return true;
        }

        // Check allowed list
        for allowed in &self.allowed_ips {
            if ip.starts_with(allowed) {
                return true;
            }
        }

        false
    }

    /// Check if port is allowed
    pub fn is_port_allowed(&self, port: u16) -> bool {
        if !self.allowed {
            return false;
        }

        // Check denied list first
        if self.denied_ports.contains(&port) {
            return false;
        }

        // If allowed list is empty, allow all non-denied
        if self.allowed_ports.is_empty() {
            return true;
        }

        self.allowed_ports.contains(&port)
    }
}

/// Sandbox isolation configuration
#[derive(Debug, Clone)]
pub struct IsolationConfig {
    /// Process isolation (PID namespace)
    pub process_isolation: bool,
    /// Network isolation
    pub network_isolation: bool,
    /// Filesystem isolation
    pub filesystem_isolation: bool,
    /// IPC isolation
    pub ipc_isolation: bool,
    /// User isolation (UID namespace)
    pub user_isolation: bool,
    /// UTS isolation (hostname)
    pub uts_isolation: bool,
    /// Cgroup isolation
    pub cgroup_isolation: bool,
}

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            process_isolation: true,
            network_isolation: false,
            filesystem_isolation: false,
            ipc_isolation: true,
            user_isolation: false,
            uts_isolation: false,
            cgroup_isolation: true,
        }
    }
}

impl IsolationConfig {
    /// Create new isolation config
    pub fn new() -> Self {
        Self::default()
    }

    /// Create maximum isolation
    pub fn maximum() -> Self {
        Self {
            process_isolation: true,
            network_isolation: true,
            filesystem_isolation: true,
            ipc_isolation: true,
            user_isolation: true,
            uts_isolation: true,
            cgroup_isolation: true,
        }
    }

    /// Create minimal isolation
    pub fn minimal() -> Self {
        Self {
            process_isolation: false,
            network_isolation: false,
            filesystem_isolation: false,
            ipc_isolation: false,
            user_isolation: false,
            uts_isolation: false,
            cgroup_isolation: false,
        }
    }
}

/// Sandbox state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxState {
    /// Sandbox is being created
    Creating,
    /// Sandbox is running
    Running,
    /// Sandbox is paused
    Paused,
    /// Sandbox is stopping
    Stopping,
    /// Sandbox has stopped
    Stopped,
    /// Sandbox has crashed
    Crashed,
    /// Sandbox has been killed
    Killed,
}

impl SandboxState {
    /// Check if sandbox is active
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Creating | Self::Running | Self::Paused)
    }

    /// Check if sandbox is terminated
    pub const fn is_terminated(&self) -> bool {
        matches!(self, Self::Stopped | Self::Crashed | Self::Killed)
    }
}

/// A sandboxed process
#[derive(Debug, Clone)]
pub struct SandboxedProcess {
    /// Process ID
    pub id: SandboxedProcessId,
    /// Parent sandbox ID
    pub sandbox_id: SandboxId,
    /// Subject ID (owner)
    pub subject_id: SubjectId,
    /// Process name
    pub name: String,
    /// Whether the process is running
    pub running: bool,
    /// Exit code (if terminated)
    pub exit_code: Option<i32>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Capabilities granted to this process
    pub capabilities: Vec<CapabilityId>,
}

impl SandboxedProcess {
    /// Create a new sandboxed process
    pub fn new(sandbox_id: SandboxId, subject_id: SubjectId, name: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        let id = SandboxedProcessId(COUNTER.fetch_add(1, Ordering::SeqCst));

        Self {
            id,
            sandbox_id,
            subject_id,
            name: String::from(name),
            running: false,
            exit_code: None,
            created_at: Timestamp::now(),
            capabilities: Vec::new(),
        }
    }

    /// Add a capability
    pub fn add_capability(&mut self, cap_id: CapabilityId) {
        if !self.capabilities.contains(&cap_id) {
            self.capabilities.push(cap_id);
        }
    }

    /// Remove a capability
    pub fn remove_capability(&mut self, cap_id: CapabilityId) {
        self.capabilities.retain(|&id| id != cap_id);
    }
}

/// A sandbox container
#[derive(Debug, Clone)]
pub struct Sandbox {
    /// Unique identifier
    pub id: SandboxId,
    /// Sandbox name
    pub name: String,
    /// Owner subject
    pub owner: SubjectId,
    /// Current state
    pub state: SandboxState,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Syscall filter
    pub syscall_filter: SyscallFilter,
    /// Filesystem isolation
    pub filesystem: FilesystemIsolation,
    /// Network isolation
    pub network: NetworkIsolation,
    /// Isolation configuration
    pub isolation: IsolationConfig,
    /// Processes in this sandbox
    pub processes: Vec<SandboxedProcessId>,
    /// Capabilities granted to this sandbox
    pub capabilities: Vec<CapabilityId>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last activity timestamp
    pub last_activity: Timestamp,
    /// Metadata
    pub metadata: BTreeMap<String, String>,
}

impl Sandbox {
    /// Create a new sandbox
    pub fn new(name: &str, owner: SubjectId) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        let id = SandboxId(COUNTER.fetch_add(1, Ordering::SeqCst));
        let now = Timestamp::now();

        Self {
            id,
            name: String::from(name),
            owner,
            state: SandboxState::Creating,
            resource_limits: ResourceLimits::new(),
            syscall_filter: SyscallFilter::basic(),
            filesystem: FilesystemIsolation::new(),
            network: NetworkIsolation::new(),
            isolation: IsolationConfig::new(),
            processes: Vec::new(),
            capabilities: Vec::new(),
            created_at: now,
            last_activity: now,
            metadata: BTreeMap::new(),
        }
    }

    /// Create a restricted sandbox
    pub fn restricted(name: &str, owner: SubjectId) -> Self {
        Self {
            resource_limits: ResourceLimits::restricted(),
            syscall_filter: SyscallFilter::restrictive(),
            isolation: IsolationConfig::maximum(),
            network: NetworkIsolation::no_network(),
            ..Self::new(name, owner)
        }
    }

    /// Create an unrestricted sandbox
    pub fn unrestricted(name: &str, owner: SubjectId) -> Self {
        Self {
            resource_limits: ResourceLimits::unrestricted(),
            syscall_filter: SyscallFilter::permissive(),
            isolation: IsolationConfig::minimal(),
            ..Self::new(name, owner)
        }
    }

    /// Set resource limits
    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }

    /// Set syscall filter
    pub fn with_syscall_filter(mut self, filter: SyscallFilter) -> Self {
        self.syscall_filter = filter;
        self
    }

    /// Set filesystem isolation
    pub fn with_filesystem(mut self, fs: FilesystemIsolation) -> Self {
        self.filesystem = fs;
        self
    }

    /// Set network isolation
    pub fn with_network(mut self, net: NetworkIsolation) -> Self {
        self.network = net;
        self
    }

    /// Set isolation config
    pub fn with_isolation(mut self, iso: IsolationConfig) -> Self {
        self.isolation = iso;
        self
    }

    /// Start the sandbox
    pub fn start(&mut self) -> Result<(), SandboxError> {
        if self.state.is_terminated() {
            return Err(SandboxError::InvalidState {
                current: self.state,
                expected: SandboxState::Stopped,
            });
        }
        self.state = SandboxState::Running;
        self.touch();
        Ok(())
    }

    /// Pause the sandbox
    pub fn pause(&mut self) -> Result<(), SandboxError> {
        if self.state != SandboxState::Running {
            return Err(SandboxError::InvalidState {
                current: self.state,
                expected: SandboxState::Running,
            });
        }
        self.state = SandboxState::Paused;
        self.touch();
        Ok(())
    }

    /// Resume the sandbox
    pub fn resume(&mut self) -> Result<(), SandboxError> {
        if self.state != SandboxState::Paused {
            return Err(SandboxError::InvalidState {
                current: self.state,
                expected: SandboxState::Paused,
            });
        }
        self.state = SandboxState::Running;
        self.touch();
        Ok(())
    }

    /// Stop the sandbox
    pub fn stop(&mut self) -> Result<(), SandboxError> {
        if !self.state.is_active() {
            return Err(SandboxError::InvalidState {
                current: self.state,
                expected: SandboxState::Running,
            });
        }
        self.state = SandboxState::Stopping;
        // In a real implementation, would signal all processes to stop
        self.state = SandboxState::Stopped;
        self.touch();
        Ok(())
    }

    /// Kill the sandbox
    pub fn kill(&mut self) -> Result<(), SandboxError> {
        self.state = SandboxState::Killed;
        self.touch();
        Ok(())
    }

    /// Add a process to the sandbox
    pub fn add_process(&mut self, process_id: SandboxedProcessId) -> Result<(), SandboxError> {
        if !self.state.is_active() {
            return Err(SandboxError::InvalidState {
                current: self.state,
                expected: SandboxState::Running,
            });
        }

        self.resource_limits.increment_processes()?;
        self.processes.push(process_id);
        self.touch();
        Ok(())
    }

    /// Remove a process from the sandbox
    pub fn remove_process(&mut self, process_id: SandboxedProcessId) {
        self.processes.retain(|&id| id != process_id);
        self.resource_limits.decrement_processes();
        self.touch();
    }

    /// Add a capability to the sandbox
    pub fn add_capability(&mut self, cap_id: CapabilityId) {
        if !self.capabilities.contains(&cap_id) {
            self.capabilities.push(cap_id);
        }
        self.touch();
    }

    /// Remove a capability from the sandbox
    pub fn remove_capability(&mut self, cap_id: CapabilityId) {
        self.capabilities.retain(|&id| id != cap_id);
        self.touch();
    }

    /// Check if a syscall is allowed
    pub fn check_syscall(&self, syscall_number: u32) -> SyscallAction {
        self.syscall_filter.check(syscall_number)
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Timestamp::now();
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(String::from(key), String::from(value));
        self.touch();
    }

    /// Check if the sandbox can allocate more memory
    pub fn can_allocate_memory(&self, size: u64) -> bool {
        self.resource_limits.current_memory.saturating_add(size) <= self.resource_limits.max_memory
    }

    /// Check if the sandbox can create more processes
    pub fn can_create_process(&self) -> bool {
        !self.resource_limits.process_limit_exceeded()
    }

    /// Check if the sandbox can open more files
    pub fn can_open_file(&self) -> bool {
        !self.resource_limits.file_limit_exceeded()
    }

    /// Check if the sandbox can create network connections
    pub fn can_create_connection(&self) -> bool {
        self.network.allowed && !self.resource_limits.network_limit_exceeded()
    }

    /// Get sandbox statistics
    pub fn stats(&self) -> SandboxStats {
        SandboxStats {
            id: self.id,
            state: self.state,
            process_count: self.processes.len() as u32,
            memory_usage: self.resource_limits.current_memory,
            memory_limit: self.resource_limits.max_memory,
            cpu_time_used: self.resource_limits.current_cpu_time,
            cpu_time_limit: self.resource_limits.max_cpu_time,
            open_files: self.resource_limits.current_open_files,
            network_connections: self.resource_limits.current_network_connections,
        }
    }
}

/// Sandbox statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SandboxStats {
    /// Sandbox ID
    pub id: SandboxId,
    /// Current state
    pub state: SandboxState,
    /// Number of processes
    pub process_count: u32,
    /// Current memory usage
    pub memory_usage: u64,
    /// Memory limit
    pub memory_limit: u64,
    /// CPU time used
    pub cpu_time_used: u64,
    /// CPU time limit
    pub cpu_time_limit: u64,
    /// Open file descriptors
    pub open_files: u32,
    /// Network connections
    pub network_connections: u32,
}

/// Errors that can occur during sandbox operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxError {
    /// Sandbox not found
    NotFound(SandboxId),
    /// Process not found
    ProcessNotFound(SandboxedProcessId),
    /// Resource limit exceeded
    ResourceLimitExceeded {
        resource: String,
        limit: u64,
        requested: u64,
    },
    /// Invalid state for operation
    InvalidState {
        current: SandboxState,
        expected: SandboxState,
    },
    /// Capability error
    CapabilityError(CapabilityError),
    /// Permission denied
    PermissionDenied(String),
    /// Syscall denied
    SyscallDenied { syscall_number: u32, process_id: SandboxedProcessId },
    /// Path not accessible
    PathNotAccessible(String),
    /// Network not allowed
    NetworkNotAllowed { address: String, port: u16 },
    /// Internal error
    Internal(String),
}

impl fmt::Display for SandboxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "Sandbox not found: {}", id),
            Self::ProcessNotFound(id) => write!(f, "Process not found: {}", id),
            Self::ResourceLimitExceeded { resource, limit, requested } => {
                write!(f, "Resource limit exceeded: {} (limit: {}, requested: {})", resource, limit, requested)
            }
            Self::InvalidState { current, expected } => {
                write!(f, "Invalid state: current {:?}, expected {:?}", current, expected)
            }
            Self::CapabilityError(e) => write!(f, "Capability error: {}", e),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Self::SyscallDenied { syscall_number, process_id } => {
                write!(f, "Syscall {} denied for process {}", syscall_number, process_id)
            }
            Self::PathNotAccessible(path) => write!(f, "Path not accessible: {}", path),
            Self::NetworkNotAllowed { address, port } => {
                write!(f, "Network access not allowed: {}:{}", address, port)
            }
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl From<CapabilityError> for SandboxError {
    fn from(e: CapabilityError) -> Self {
        Self::CapabilityError(e)
    }
}

impl core::error::Error for SandboxError {}

/// Sandbox manager
pub struct SandboxManager {
    /// All sandboxes
    sandboxes: RwLock<BTreeMap<SandboxId, Sandbox>>,
    /// All sandboxed processes
    processes: RwLock<BTreeMap<SandboxedProcessId, SandboxedProcess>>,
    /// Process to sandbox mapping
    process_sandbox_map: RwLock<BTreeMap<SandboxedProcessId, SandboxId>>,
    /// Maximum number of sandboxes
    max_sandboxes: usize,
}

impl SandboxManager {
    /// Create a new sandbox manager
    pub fn new() -> Self {
        Self {
            sandboxes: RwLock::new(BTreeMap::new()),
            processes: RwLock::new(BTreeMap::new()),
            process_sandbox_map: RwLock::new(BTreeMap::new()),
            max_sandboxes: 1024,
        }
    }

    /// Create with custom limits
    pub fn with_limits(max_sandboxes: usize) -> Self {
        Self {
            sandboxes: RwLock::new(BTreeMap::new()),
            processes: RwLock::new(BTreeMap::new()),
            process_sandbox_map: RwLock::new(BTreeMap::new()),
            max_sandboxes,
        }
    }

    /// Create a new sandbox
    pub fn create_sandbox(&self, name: &str, owner: SubjectId) -> Result<SandboxId, SandboxError> {
        {
            let sandboxes = self.sandboxes.read();
            if sandboxes.len() >= self.max_sandboxes {
                return Err(SandboxError::Internal(String::from("Maximum sandbox limit reached")));
            }
        }

        let sandbox = Sandbox::new(name, owner);
        let id = sandbox.id;

        {
            let mut sandboxes = self.sandboxes.write();
            sandboxes.insert(id, sandbox);
        }

        Ok(id)
    }

    /// Create a restricted sandbox
    pub fn create_restricted(&self, name: &str, owner: SubjectId) -> Result<SandboxId, SandboxError> {
        let sandbox = Sandbox::restricted(name, owner);
        let id = sandbox.id;

        {
            let mut sandboxes = self.sandboxes.write();
            sandboxes.insert(id, sandbox);
        }

        Ok(id)
    }

    /// Get a sandbox by ID
    pub fn get_sandbox(&self, id: SandboxId) -> Option<Sandbox> {
        let sandboxes = self.sandboxes.read();
        sandboxes.get(&id).cloned()
    }

    /// Update a sandbox
    pub fn update_sandbox<F>(&self, id: SandboxId, f: F) -> Result<(), SandboxError>
    where
        F: FnOnce(&mut Sandbox),
    {
        let mut sandboxes = self.sandboxes.write();
        let sandbox = sandboxes.get_mut(&id).ok_or(SandboxError::NotFound(id))?;
        f(sandbox);
        Ok(())
    }

    /// Start a sandbox
    pub fn start_sandbox(&self, id: SandboxId) -> Result<(), SandboxError> {
        self.update_sandbox(id, |s| {
            let _ = s.start();
        })
    }

    /// Stop a sandbox
    pub fn stop_sandbox(&self, id: SandboxId) -> Result<(), SandboxError> {
        self.update_sandbox(id, |s| {
            let _ = s.stop();
        })
    }

    /// Kill a sandbox
    pub fn kill_sandbox(&self, id: SandboxId) -> Result<(), SandboxError> {
        self.update_sandbox(id, |s| {
            let _ = s.kill();
        })
    }

    /// Create a process in a sandbox
    pub fn create_process(
        &self,
        sandbox_id: SandboxId,
        subject_id: SubjectId,
        name: &str,
    ) -> Result<SandboxedProcessId, SandboxError> {
        let process = {
            let mut sandboxes = self.sandboxes.write();
            let sandbox = sandboxes.get_mut(&sandbox_id).ok_or(SandboxError::NotFound(sandbox_id))?;

            if !sandbox.state.is_active() {
                return Err(SandboxError::InvalidState {
                    current: sandbox.state,
                    expected: SandboxState::Running,
                });
            }

            let process = SandboxedProcess::new(sandbox_id, subject_id, name);
            sandbox.add_process(process.id)?;
            process
        };

        let process_id = process.id;

        {
            let mut processes = self.processes.write();
            processes.insert(process_id, process);
        }

        {
            let mut map = self.process_sandbox_map.write();
            map.insert(process_id, sandbox_id);
        }

        Ok(process_id)
    }

    /// Get a process by ID
    pub fn get_process(&self, id: SandboxedProcessId) -> Option<SandboxedProcess> {
        let processes = self.processes.read();
        processes.get(&id).cloned()
    }

    /// Terminate a process
    pub fn terminate_process(&self, id: SandboxedProcessId, exit_code: i32) -> Result<(), SandboxError> {
        let sandbox_id = {
            let map = self.process_sandbox_map.read();
            *map.get(&id).ok_or(SandboxError::ProcessNotFound(id))?
        };

        {
            let mut sandboxes = self.sandboxes.write();
            if let Some(sandbox) = sandboxes.get_mut(&sandbox_id) {
                sandbox.remove_process(id);
            }
        }

        {
            let mut processes = self.processes.write();
            if let Some(process) = processes.get_mut(&id) {
                process.running = false;
                process.exit_code = Some(exit_code);
            }
        }

        {
            let mut map = self.process_sandbox_map.write();
            map.remove(&id);
        }

        Ok(())
    }

    /// Check if a syscall is allowed for a process
    pub fn check_syscall(
        &self,
        process_id: SandboxedProcessId,
        syscall_number: u32,
    ) -> Result<SyscallAction, SandboxError> {
        let sandbox_id = {
            let map = self.process_sandbox_map.read();
            *map.get(&process_id).ok_or(SandboxError::ProcessNotFound(process_id))?
        };

        let sandboxes = self.sandboxes.read();
        let sandbox = sandboxes.get(&sandbox_id).ok_or(SandboxError::NotFound(sandbox_id))?;

        Ok(sandbox.check_syscall(syscall_number))
    }

    /// Allocate memory for a sandbox
    pub fn allocate_memory(&self, sandbox_id: SandboxId, size: u64) -> Result<(), SandboxError> {
        let mut sandboxes = self.sandboxes.write();
        let sandbox = sandboxes.get_mut(&sandbox_id).ok_or(SandboxError::NotFound(sandbox_id))?;
        sandbox.resource_limits.allocate_memory(size)?;
        sandbox.touch();
        Ok(())
    }

    /// Free memory for a sandbox
    pub fn free_memory(&self, sandbox_id: SandboxId, size: u64) -> Result<(), SandboxError> {
        let mut sandboxes = self.sandboxes.write();
        let sandbox = sandboxes.get_mut(&sandbox_id).ok_or(SandboxError::NotFound(sandbox_id))?;
        sandbox.resource_limits.free_memory(size);
        sandbox.touch();
        Ok(())
    }

    /// Check file access for a sandbox
    pub fn check_file_access(
        &self,
        sandbox_id: SandboxId,
        path: &str,
        write: bool,
    ) -> Result<bool, SandboxError> {
        let sandboxes = self.sandboxes.read();
        let sandbox = sandboxes.get(&sandbox_id).ok_or(SandboxError::NotFound(sandbox_id))?;

        if !sandbox.filesystem.is_path_accessible(path) {
            return Err(SandboxError::PathNotAccessible(String::from(path)));
        }

        if write && sandbox.filesystem.is_path_read_only(path) {
            return Ok(false);
        }

        Ok(true)
    }

    /// Check network access for a sandbox
    pub fn check_network_access(
        &self,
        sandbox_id: SandboxId,
        address: &str,
        port: u16,
    ) -> Result<bool, SandboxError> {
        let sandboxes = self.sandboxes.read();
        let sandbox = sandboxes.get(&sandbox_id).ok_or(SandboxError::NotFound(sandbox_id))?;

        if !sandbox.network.allowed {
            return Err(SandboxError::NetworkNotAllowed {
                address: String::from(address),
                port,
            });
        }

        if !sandbox.network.is_ip_allowed(address) {
            return Err(SandboxError::NetworkNotAllowed {
                address: String::from(address),
                port,
            });
        }

        if !sandbox.network.is_port_allowed(port) {
            return Err(SandboxError::NetworkNotAllowed {
                address: String::from(address),
                port,
            });
        }

        Ok(true)
    }

    /// Get sandbox statistics
    pub fn get_sandbox_stats(&self, id: SandboxId) -> Result<SandboxStats, SandboxError> {
        let sandboxes = self.sandboxes.read();
        let sandbox = sandboxes.get(&id).ok_or(SandboxError::NotFound(id))?;
        Ok(sandbox.stats())
    }

    /// Get all sandbox IDs
    pub fn get_all_sandbox_ids(&self) -> Vec<SandboxId> {
        let sandboxes = self.sandboxes.read();
        sandboxes.keys().copied().collect()
    }

    /// Get sandboxes by owner
    pub fn get_sandboxes_by_owner(&self, owner: SubjectId) -> Vec<SandboxId> {
        let sandboxes = self.sandboxes.read();
        sandboxes
            .values()
            .filter(|s| s.owner == owner)
            .map(|s| s.id)
            .collect()
    }

    /// Get sandbox manager statistics
    pub fn stats(&self) -> SandboxManagerStats {
        let sandboxes = self.sandboxes.read();
        let processes = self.processes.read();

        let mut running = 0;
        let mut stopped = 0;
        let mut total_memory = 0u64;
        let mut total_processes = 0;

        for sandbox in sandboxes.values() {
            match sandbox.state {
                SandboxState::Running => running += 1,
                SandboxState::Stopped | SandboxState::Crashed | SandboxState::Killed => stopped += 1,
                _ => {}
            }
            total_memory += sandbox.resource_limits.current_memory;
            total_processes += sandbox.processes.len() as u64;
        }

        SandboxManagerStats {
            total_sandboxes: sandboxes.len(),
            running_sandboxes: running,
            stopped_sandboxes: stopped,
            total_processes: processes.len(),
            active_processes: processes.values().filter(|p| p.running).count(),
            total_memory_usage: total_memory,
        }
    }

    /// Cleanup terminated sandboxes
    pub fn cleanup_terminated(&self) -> usize {
        let mut sandboxes = self.sandboxes.write();
        let mut processes = self.processes.write();
        let mut map = self.process_sandbox_map.write();

        let terminated: Vec<SandboxId> = sandboxes
            .iter()
            .filter(|(_, s)| s.state.is_terminated())
            .map(|(id, _)| *id)
            .collect();

        let count = terminated.len();

        for id in &terminated {
            if let Some(sandbox) = sandboxes.remove(id) {
                for process_id in &sandbox.processes {
                    processes.remove(process_id);
                    map.remove(process_id);
                }
            }
        }

        count
    }
}

impl Default for SandboxManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Sandbox manager statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SandboxManagerStats {
    /// Total number of sandboxes
    pub total_sandboxes: usize,
    /// Number of running sandboxes
    pub running_sandboxes: usize,
    /// Number of stopped sandboxes
    pub stopped_sandboxes: usize,
    /// Total number of processes
    pub total_processes: usize,
    /// Number of active processes
    pub active_processes: usize,
    /// Total memory usage across all sandboxes
    pub total_memory_usage: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::time::Duration;

    #[test]
    fn test_resource_limits() {
        let mut limits = ResourceLimits::new();
        limits.max_memory = 1024;

        assert!(limits.allocate_memory(512).is_ok());
        assert_eq!(limits.current_memory, 512);

        assert!(limits.allocate_memory(512).is_ok());
        assert_eq!(limits.current_memory, 1024);

        assert!(limits.allocate_memory(1).is_err());
    }

    #[test]
    fn test_syscall_filter() {
        let mut filter = SyscallFilter::new();
        filter.add_rule(SyscallFilterRule::allow(1));
        filter.add_rule(SyscallFilterRule::deny(2));

        assert_eq!(filter.check(1), SyscallAction::Allow);
        assert_eq!(filter.check(2), SyscallAction::Deny);
        assert_eq!(filter.check(999), SyscallAction::Deny); // default
    }

    #[test]
    fn test_network_isolation() {
        let mut net = NetworkIsolation::new();
        net.add_allowed_ip("192.168.1.");
        net.add_denied_ip("192.168.1.100");

        assert!(net.is_ip_allowed("192.168.1.50"));
        assert!(!net.is_ip_allowed("192.168.1.100"));
        assert!(!net.is_ip_allowed("10.0.0.1"));
    }

    #[test]
    fn test_sandbox_creation() {
        let manager = SandboxManager::new();
        let sandbox_id = manager.create_sandbox("test", SubjectId::new(1)).unwrap();

        let sandbox = manager.get_sandbox(sandbox_id).unwrap();
        assert_eq!(sandbox.name, "test");
        assert_eq!(sandbox.state, SandboxState::Creating);
    }

    #[test]
    fn test_sandbox_lifecycle() {
        let manager = SandboxManager::new();
        let sandbox_id = manager.create_sandbox("test", SubjectId::new(1)).unwrap();

        manager.start_sandbox(sandbox_id).unwrap();
        let sandbox = manager.get_sandbox(sandbox_id).unwrap();
        assert_eq!(sandbox.state, SandboxState::Running);

        manager.stop_sandbox(sandbox_id).unwrap();
        let sandbox = manager.get_sandbox(sandbox_id).unwrap();
        assert_eq!(sandbox.state, SandboxState::Stopped);
    }

    #[test]
    fn test_process_creation() {
        let manager = SandboxManager::new();
        let sandbox_id = manager.create_sandbox("test", SubjectId::new(1)).unwrap();
        manager.start_sandbox(sandbox_id).unwrap();

        let process_id = manager.create_process(sandbox_id, SubjectId::new(2), "test_proc").unwrap();
        let process = manager.get_process(process_id).unwrap();

        assert_eq!(process.name, "test_proc");
        assert_eq!(process.sandbox_id, sandbox_id);
    }
}
