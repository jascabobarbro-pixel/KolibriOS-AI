//! Capability-based Access Control System for KolibriOS AI
//!
//! This module implements a capability-based security model where capabilities
//! are unforgeable tokens that grant specific permissions to resources.
//! Each capability has an associated set of permissions, an expiry time,
//! and a cryptographically secure token for validation.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::{BitAnd, BitOr, BitXor, Not};
use core::time::Duration;
use spin::RwLock;

extern crate alloc;

/// Unique identifier for a capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapabilityId(pub u64);

impl fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CAP-{:016X}", self.0)
    }
}

/// Permission flags for capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Permission(u32);

impl Permission {
    /// No permissions
    pub const NONE: Permission = Permission(0);
    /// Read permission
    pub const READ: Permission = Permission(1 << 0);
    /// Write permission
    pub const WRITE: Permission = Permission(1 << 1);
    /// Execute permission
    pub const EXECUTE: Permission = Permission(1 << 2);
    /// Admin permission (full control)
    pub const ADMIN: Permission = Permission(1 << 3);
    /// Create permission (for containers/directories)
    pub const CREATE: Permission = Permission(1 << 4);
    /// Delete permission
    pub const DELETE: Permission = Permission(1 << 5);
    /// Grant permission (can delegate capabilities)
    pub const GRANT: Permission = Permission(1 << 6);
    /// Revoke permission
    pub const REVOKE: Permission = Permission(1 << 7);
    /// All permissions
    pub const ALL: Permission = Permission(0xFFFFFFFF);

    /// Create a new Permission from a raw u32 value
    pub const fn new(bits: u32) -> Self {
        Permission(bits)
    }

    /// Check if a specific permission is set
    pub const fn contains(&self, other: Permission) -> bool {
        (self.0 & other.0) != 0
    }

    /// Check if all specified permissions are set
    pub const fn contains_all(&self, perms: &[Permission]) -> bool {
        let mut combined = 0u32;
        let mut i = 0;
        while i < perms.len() {
            combined |= perms[i].0;
            i += 1;
        }
        (self.0 & combined) == combined
    }

    /// Get the raw bits
    pub const fn bits(&self) -> u32 {
        self.0
    }

    /// Check if any permission is set
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Add a permission
    pub const fn add(&self, other: Permission) -> Self {
        Permission(self.0 | other.0)
    }

    /// Remove a permission
    pub const fn remove(&self, other: Permission) -> Self {
        Permission(self.0 & !other.0)
    }

    /// Intersect permissions
    pub const fn intersect(&self, other: Permission) -> Self {
        Permission(self.0 & other.0)
    }

    /// Convert to a human-readable string
    pub fn to_string_list(&self) -> Vec<String> {
        let mut perms = Vec::new();
        if self.contains(Permission::READ) {
            perms.push(String::from("read"));
        }
        if self.contains(Permission::WRITE) {
            perms.push(String::from("write"));
        }
        if self.contains(Permission::EXECUTE) {
            perms.push(String::from("execute"));
        }
        if self.contains(Permission::ADMIN) {
            perms.push(String::from("admin"));
        }
        if self.contains(Permission::CREATE) {
            perms.push(String::from("create"));
        }
        if self.contains(Permission::DELETE) {
            perms.push(String::from("delete"));
        }
        if self.contains(Permission::GRANT) {
            perms.push(String::from("grant"));
        }
        if self.contains(Permission::REVOKE) {
            perms.push(String::from("revoke"));
        }
        perms
    }
}

impl BitAnd for Permission {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Permission(self.0 & rhs.0)
    }
}

impl BitOr for Permission {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Permission(self.0 | rhs.0)
    }
}

impl BitXor for Permission {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Permission(self.0 ^ rhs.0)
    }
}

impl Not for Permission {
    type Output = Self;
    fn not(self) -> Self::Output {
        Permission(!self.0)
    }
}

/// Represents a resource type that can be accessed
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceType {
    /// Memory region
    Memory,
    /// I/O port range
    IoPort,
    /// Process
    Process,
    /// File
    File,
    /// Network socket
    Socket,
    /// Device
    Device,
    /// IPC channel
    IpcChannel,
    /// System configuration
    SystemConfig,
    /// Security capability
    Capability,
    /// Kernel resource
    Kernel,
    /// Custom resource type
    Custom(u16),
}

impl ResourceType {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceType::Memory => "memory",
            ResourceType::IoPort => "io_port",
            ResourceType::Process => "process",
            ResourceType::File => "file",
            ResourceType::Socket => "socket",
            ResourceType::Device => "device",
            ResourceType::IpcChannel => "ipc_channel",
            ResourceType::SystemConfig => "system_config",
            ResourceType::Capability => "capability",
            ResourceType::Kernel => "kernel",
            ResourceType::Custom(_) => "custom",
        }
    }
}

/// A unique token for secure capability reference
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityToken {
    /// The token bytes (cryptographically secure)
    bytes: [u8; 32],
    /// Version for future token format changes
    version: u8,
}

impl CapabilityToken {
    /// Create a new random token
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        
        // Simple deterministic token generation for no_std environment
        // In a real system, this would use a CSPRNG
        let mut state = Self::get_time_seed();
        for i in 0..32 {
            state = state.wrapping_mul(6364136223846793005) ^ state;
            state = state.wrapping_add(1442695040888963407);
            bytes[i] = (state >> 56) as u8;
        }
        
        Self {
            bytes,
            version: 1,
        }
    }

    /// Get a seed from system time (simplified for no_std)
    fn get_time_seed() -> u64 {
        // In a real kernel, this would read from a hardware timer
        // For now, use a simple counter-based seed
        use core::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        let seed = COUNTER.fetch_add(1, Ordering::SeqCst);
        seed.wrapping_mul(0x5851F42D4C957F2D)
    }

    /// Create a token from raw bytes (use with caution)
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self {
            bytes,
            version: 1,
        }
    }

    /// Get the raw bytes of the token
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Convert token to hex string
    pub fn to_hex(&self) -> String {
        let mut hex = String::with_capacity(64);
        for byte in &self.bytes {
            let high = (byte >> 4) & 0x0F;
            let low = byte & 0x0F;
            hex.push(char::from(b'0' + high));
            if low < 10 {
                hex.push(char::from(b'0' + low));
            } else {
                hex.push(char::from(b'a' + low - 10));
            }
        }
        hex
    }

    /// Check if this token is valid (non-zero)
    pub const fn is_valid(&self) -> bool {
        let mut sum: u8 = 0;
        let mut i = 0;
        while i < 32 {
            sum |= self.bytes[i];
            i += 1;
        }
        sum != 0
    }
}

/// Time representation for capability expiry
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp {
    /// Seconds since kernel boot
    pub secs: u64,
    /// Nanoseconds component
    pub nanos: u32,
}

impl Timestamp {
    /// Create a new timestamp
    pub const fn new(secs: u64, nanos: u32) -> Self {
        Self { secs, nanos }
    }

    /// Get current timestamp (simplified)
    pub fn now() -> Self {
        // In a real kernel, this would read from a hardware timer
        use core::sync::atomic::{AtomicU64, Ordering};
        static TIME: AtomicU64 = AtomicU64::new(0);
        let secs = TIME.fetch_add(1, Ordering::SeqCst);
        Self { secs, nanos: 0 }
    }

    /// Create a timestamp from Duration
    pub const fn from_duration(d: Duration) -> Self {
        Self {
            secs: d.as_secs(),
            nanos: d.subsec_nanos(),
        }
    }

    /// Add a duration to this timestamp
    pub const fn add_duration(&self, d: Duration) -> Self {
        let new_secs = self.secs.saturating_add(d.as_secs());
        let new_nanos = self.nanos.saturating_add(d.subsec_nanos());
        Self {
            secs: new_secs,
            nanos: if new_nanos >= 1_000_000_000 { new_nanos - 1_000_000_000 } else { new_nanos },
        }
    }

    /// Check if this timestamp is before another
    pub const fn is_before(&self, other: &Timestamp) -> bool {
        if self.secs < other.secs {
            return true;
        }
        if self.secs > other.secs {
            return false;
        }
        self.nanos < other.nanos
    }

    /// Check if this timestamp is after another
    pub const fn is_after(&self, other: &Timestamp) -> bool {
        other.is_before(self)
    }

    /// Check if this timestamp represents "never expires"
    pub const fn is_never(&self) -> bool {
        self.secs == u64::MAX
    }

    /// Create a "never expires" timestamp
    pub const fn never() -> Self {
        Self { secs: u64::MAX, nanos: 0 }
    }
}

/// A capability representing a permission to access a resource
#[derive(Debug, Clone)]
pub struct Capability {
    /// Unique identifier for this capability
    pub id: CapabilityId,
    /// Secure token for validation
    pub token: CapabilityToken,
    /// The subject (user/process) that owns this capability
    pub subject_id: SubjectId,
    /// The resource this capability grants access to
    pub resource: ResourceDescriptor,
    /// Permissions granted by this capability
    pub permissions: Permission,
    /// When this capability expires
    pub expiry: Timestamp,
    /// Who created this capability
    pub creator: SubjectId,
    /// Parent capability (if derived)
    pub parent: Option<CapabilityId>,
    /// Whether this capability can be delegated
    pub delegatable: bool,
    /// Additional metadata
    pub metadata: BTreeMap<String, String>,
}

impl Capability {
    /// Create a new capability
    pub fn new(
        subject_id: SubjectId,
        resource: ResourceDescriptor,
        permissions: Permission,
        ttl: Duration,
        creator: SubjectId,
    ) -> Self {
        static COUNTER: spin::Mutex<u64> = spin::Mutex::new(1);
        let id = {
            let mut counter = COUNTER.lock();
            let id = *counter;
            *counter += 1;
            CapabilityId(id)
        };

        let now = Timestamp::now();
        let expiry = now.add_duration(ttl);

        Self {
            id,
            token: CapabilityToken::generate(),
            subject_id,
            resource,
            permissions,
            expiry,
            creator,
            parent: None,
            delegatable: permissions.contains(Permission::GRANT),
            metadata: BTreeMap::new(),
        }
    }

    /// Create a capability that never expires
    pub fn permanent(
        subject_id: SubjectId,
        resource: ResourceDescriptor,
        permissions: Permission,
        creator: SubjectId,
    ) -> Self {
        let mut cap = Self::new(
            subject_id,
            resource,
            permissions,
            Duration::from_secs(u64::MAX),
            creator,
        );
        cap.expiry = Timestamp::never();
        cap
    }

    /// Check if this capability has expired
    pub fn is_expired(&self) -> bool {
        if self.expiry.is_never() {
            return false;
        }
        let now = Timestamp::now();
        now.is_after(&self.expiry)
    }

    /// Check if this capability is valid
    pub fn is_valid(&self) -> bool {
        self.token.is_valid() && !self.is_expired()
    }

    /// Check if this capability grants a specific permission
    pub fn has_permission(&self, perm: Permission) -> bool {
        self.permissions.contains(perm)
    }

    /// Check if this capability grants all specified permissions
    pub fn has_permissions(&self, perms: &[Permission]) -> bool {
        self.permissions.contains_all(perms)
    }

    /// Derive a child capability with restricted permissions
    pub fn derive(
        &self,
        new_permissions: Permission,
        new_subject: SubjectId,
        ttl: Duration,
    ) -> Result<Capability, CapabilityError> {
        // Check if derivation is allowed
        if !self.delegatable {
            return Err(CapabilityError::NotDelegatable);
        }

        // Check if capability is still valid
        if !self.is_valid() {
            return Err(CapabilityError::Expired);
        }

        // Check if new permissions are a subset
        let restricted = new_permissions.intersect(self.permissions);
        if restricted != new_permissions {
            return Err(CapabilityError::PermissionEscalation);
        }

        // Create derived capability
        let mut derived = Capability::new(
            new_subject,
            self.resource.clone(),
            new_permissions,
            ttl,
            self.creator,
        );
        derived.parent = Some(self.id);

        Ok(derived)
    }

    /// Add metadata to this capability
    pub fn add_metadata(&mut self, key: String, value: String) -> &mut Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if this capability matches a resource pattern
    pub fn matches_resource(&self, pattern: &ResourceDescriptor) -> bool {
        self.resource.matches(pattern)
    }
}

/// Unique identifier for a subject (user or process)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubjectId(pub u64);

impl SubjectId {
    /// Create a new subject ID
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// The root/admin subject
    pub const ROOT: SubjectId = SubjectId(0);
    
    /// The kernel subject
    pub const KERNEL: SubjectId = SubjectId(1);
    
    /// The first user subject
    pub const FIRST_USER: SubjectId = SubjectId(1000);
}

impl fmt::Display for SubjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SUBJ-{:016X}", self.0)
    }
}

/// Descriptor for a resource
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceDescriptor {
    /// Type of the resource
    pub resource_type: ResourceType,
    /// Resource-specific identifier
    pub resource_id: u64,
    /// Optional path (for files, etc.)
    pub path: Option<String>,
    /// Resource-specific flags
    pub flags: u32,
}

impl ResourceDescriptor {
    /// Create a new resource descriptor
    pub const fn new(resource_type: ResourceType, resource_id: u64) -> Self {
        Self {
            resource_type,
            resource_id,
            path: None,
            flags: 0,
        }
    }

    /// Create a resource descriptor with a path
    pub fn with_path(resource_type: ResourceType, resource_id: u64, path: &str) -> Self {
        Self {
            resource_type,
            resource_id,
            path: Some(String::from(path)),
            flags: 0,
        }
    }

    /// Create a memory region descriptor
    pub const fn memory_region(start: u64, size: u64) -> Self {
        Self {
            resource_type: ResourceType::Memory,
            resource_id: start,
            path: None,
            flags: size as u32,
        }
    }

    /// Create an I/O port range descriptor
    pub const fn io_port_range(start: u16, count: u16) -> Self {
        Self {
            resource_type: ResourceType::IoPort,
            resource_id: start as u64,
            path: None,
            flags: count as u32,
        }
    }

    /// Create a process descriptor
    pub const fn process(pid: u64) -> Self {
        Self::new(ResourceType::Process, pid)
    }

    /// Create a file descriptor
    pub fn file(path: &str, fd: u64) -> Self {
        Self::with_path(ResourceType::File, fd, path)
    }

    /// Create a socket descriptor
    pub const fn socket(socket_id: u64) -> Self {
        Self::new(ResourceType::Socket, socket_id)
    }

    /// Create a device descriptor
    pub const fn device(device_id: u64) -> Self {
        Self::new(ResourceType::Device, device_id)
    }

    /// Create an IPC channel descriptor
    pub const fn ipc_channel(channel_id: u64) -> Self {
        Self::new(ResourceType::IpcChannel, channel_id)
    }

    /// Check if this descriptor matches another (with wildcard support)
    pub fn matches(&self, pattern: &ResourceDescriptor) -> bool {
        // Exact type match required
        if self.resource_type != pattern.resource_type {
            return false;
        }

        // Wildcard resource_id (0 matches all)
        if pattern.resource_id == 0 {
            return true;
        }

        // Exact ID match
        if self.resource_id != pattern.resource_id {
            return false;
        }

        // Path matching (if both have paths)
        match (&self.path, &pattern.path) {
            (Some(path), Some(pattern_path)) => {
                // Simple prefix matching for now
                path.starts_with(pattern_path) || pattern_path == "*"
            }
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (None, None) => true,
        }
    }
}

/// Errors that can occur during capability operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityError {
    /// The capability has expired
    Expired,
    /// The capability is invalid
    Invalid,
    /// The capability token doesn't match
    TokenMismatch,
    /// Permission escalation attempted
    PermissionEscalation,
    /// The capability is not delegatable
    NotDelegatable,
    /// The capability was not found
    NotFound,
    /// The subject is not authorized
    Unauthorized,
    /// Resource not found
    ResourceNotFound,
    /// Maximum capability limit reached
    LimitReached,
    /// Invalid capability state
    InvalidState,
    /// Internal error
    Internal(String),
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expired => write!(f, "Capability has expired"),
            Self::Invalid => write!(f, "Capability is invalid"),
            Self::TokenMismatch => write!(f, "Capability token does not match"),
            Self::PermissionEscalation => write!(f, "Permission escalation attempted"),
            Self::NotDelegatable => write!(f, "Capability is not delegatable"),
            Self::NotFound => write!(f, "Capability not found"),
            Self::Unauthorized => write!(f, "Subject is not authorized"),
            Self::ResourceNotFound => write!(f, "Resource not found"),
            Self::LimitReached => write!(f, "Maximum capability limit reached"),
            Self::InvalidState => write!(f, "Invalid capability state"),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl core::error::Error for CapabilityError {}

/// Capability manager for storing and managing capabilities
pub struct CapabilityManager {
    /// All capabilities indexed by ID
    capabilities: RwLock<BTreeMap<CapabilityId, Capability>>,
    /// Token to ID mapping for fast lookup
    token_index: RwLock<BTreeMap<[u8; 32], CapabilityId>>,
    /// Subject to capabilities mapping
    subject_index: RwLock<BTreeMap<SubjectId, Vec<CapabilityId>>>,
    /// Resource to capabilities mapping
    resource_index: RwLock<BTreeMap<(ResourceType, u64), Vec<CapabilityId>>>,
    /// Maximum number of capabilities per subject
    max_per_subject: usize,
    /// Total maximum capabilities
    max_total: usize,
}

impl CapabilityManager {
    /// Create a new capability manager
    pub fn new() -> Self {
        Self {
            capabilities: RwLock::new(BTreeMap::new()),
            token_index: RwLock::new(BTreeMap::new()),
            subject_index: RwLock::new(BTreeMap::new()),
            resource_index: RwLock::new(BTreeMap::new()),
            max_per_subject: 1024,
            max_total: 65536,
        }
    }

    /// Create with custom limits
    pub fn with_limits(max_per_subject: usize, max_total: usize) -> Self {
        Self {
            capabilities: RwLock::new(BTreeMap::new()),
            token_index: RwLock::new(BTreeMap::new()),
            subject_index: RwLock::new(BTreeMap::new()),
            resource_index: RwLock::new(BTreeMap::new()),
            max_per_subject,
            max_total,
        }
    }

    /// Store a capability
    pub fn store(&self, capability: Capability) -> Result<CapabilityId, CapabilityError> {
        // Check limits
        {
            let caps = self.capabilities.read();
            if caps.len() >= self.max_total {
                return Err(CapabilityError::LimitReached);
            }
        }

        {
            let subject_index = self.subject_index.read();
            if let Some(caps) = subject_index.get(&capability.subject_id) {
                if caps.len() >= self.max_per_subject {
                    return Err(CapabilityError::LimitReached);
                }
            }
        }

        let id = capability.id;
        let token_bytes = *capability.token.as_bytes();
        let subject = capability.subject_id;
        let resource_type = capability.resource.resource_type;
        let resource_id = capability.resource.resource_id;

        // Store capability
        {
            let mut caps = self.capabilities.write();
            caps.insert(id, capability);
        }

        // Update indices
        {
            let mut token_idx = self.token_index.write();
            token_idx.insert(token_bytes, id);
        }

        {
            let mut subject_idx = self.subject_index.write();
            subject_idx.entry(subject).or_default().push(id);
        }

        {
            let mut resource_idx = self.resource_index.write();
            resource_idx
                .entry((resource_type, resource_id))
                .or_default()
                .push(id);
        }

        Ok(id)
    }

    /// Look up a capability by ID
    pub fn get(&self, id: CapabilityId) -> Option<Capability> {
        let caps = self.capabilities.read();
        caps.get(&id).cloned()
    }

    /// Look up a capability by token
    pub fn get_by_token(&self, token: &CapabilityToken) -> Option<Capability> {
        let token_idx = self.token_index.read();
        let id = token_idx.get(token.as_bytes())?;
        let caps = self.capabilities.read();
        caps.get(id).cloned()
    }

    /// Validate a capability token
    pub fn validate(&self, token: &CapabilityToken) -> Result<Capability, CapabilityError> {
        let cap = self.get_by_token(token).ok_or(CapabilityError::NotFound)?;
        
        if cap.is_expired() {
            return Err(CapabilityError::Expired);
        }

        if !cap.token.is_valid() {
            return Err(CapabilityError::Invalid);
        }

        Ok(cap)
    }

    /// Check if a subject has a specific permission on a resource
    pub fn check_permission(
        &self,
        subject: SubjectId,
        resource: &ResourceDescriptor,
        permission: Permission,
    ) -> bool {
        let resource_idx = self.resource_index.read();
        let caps = self.capabilities.read();

        // Check direct resource match
        if let Some(cap_ids) = resource_idx.get(&(resource.resource_type, resource.resource_id)) {
            for cap_id in cap_ids {
                if let Some(cap) = caps.get(cap_id) {
                    if cap.subject_id == subject && cap.has_permission(permission) && cap.is_valid() {
                        return true;
                    }
                }
            }
        }

        // Check wildcard resource (id = 0)
        if let Some(cap_ids) = resource_idx.get(&(resource.resource_type, 0)) {
            for cap_id in cap_ids {
                if let Some(cap) = caps.get(cap_id) {
                    if cap.subject_id == subject && cap.has_permission(permission) && cap.is_valid() {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Get all capabilities for a subject
    pub fn get_subject_capabilities(&self, subject: SubjectId) -> Vec<Capability> {
        let subject_idx = self.subject_index.read();
        let caps = self.capabilities.read();

        subject_idx
            .get(&subject)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| caps.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Revoke a capability
    pub fn revoke(&self, id: CapabilityId, revoker: SubjectId) -> Result<(), CapabilityError> {
        let cap = self.get(id).ok_or(CapabilityError::NotFound)?;

        // Check if revoker has REVOKE permission or is the creator
        if revoker != cap.creator && revoker != SubjectId::ROOT {
            if !self.check_permission(
                revoker,
                &cap.resource,
                Permission::REVOKE,
            ) {
                return Err(CapabilityError::Unauthorized);
            }
        }

        // Remove from all indices
        {
            let mut token_idx = self.token_index.write();
            token_idx.remove(cap.token.as_bytes());
        }

        {
            let mut subject_idx = self.subject_index.write();
            if let Some(caps) = subject_idx.get_mut(&cap.subject_id) {
                caps.retain(|&cap_id| cap_id != id);
            }
        }

        {
            let mut resource_idx = self.resource_index.write();
            if let Some(caps) = resource_idx.get_mut(&(cap.resource.resource_type, cap.resource.resource_id)) {
                caps.retain(|&cap_id| cap_id != id);
            }
        }

        {
            let mut caps = self.capabilities.write();
            caps.remove(&id);
        }

        Ok(())
    }

    /// Clean up expired capabilities
    pub fn cleanup_expired(&self) -> usize {
        let mut expired_ids = Vec::new();
        
        {
            let caps = self.capabilities.read();
            for (id, cap) in caps.iter() {
                if cap.is_expired() {
                    expired_ids.push(*id);
                }
            }
        }

        let count = expired_ids.len();
        for id in expired_ids {
            let _ = self.revoke(id, SubjectId::KERNEL);
        }

        count
    }

    /// Get statistics about capability usage
    pub fn stats(&self) -> CapabilityStats {
        let caps = self.capabilities.read();
        let subject_idx = self.subject_index.read();

        let mut total = 0;
        let mut valid = 0;
        let mut expired = 0;

        for cap in caps.values() {
            total += 1;
            if cap.is_expired() {
                expired += 1;
            } else {
                valid += 1;
            }
        }

        CapabilityStats {
            total,
            valid,
            expired,
            subjects: subject_idx.len(),
            resources: self.resource_index.read().len(),
        }
    }
}

impl Default for CapabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityStats {
    /// Total number of capabilities
    pub total: usize,
    /// Number of valid (non-expired) capabilities
    pub valid: usize,
    /// Number of expired capabilities
    pub expired: usize,
    /// Number of unique subjects
    pub subjects: usize,
    /// Number of unique resources
    pub resources: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_operations() {
        let read_write = Permission::READ | Permission::WRITE;
        assert!(read_write.contains(Permission::READ));
        assert!(read_write.contains(Permission::WRITE));
        assert!(!read_write.contains(Permission::EXECUTE));

        let all = Permission::ALL;
        assert!(all.contains(Permission::READ));
        assert!(all.contains(Permission::WRITE));
        assert!(all.contains(Permission::ADMIN));
    }

    #[test]
    fn test_capability_creation() {
        let subject = SubjectId::new(1);
        let resource = ResourceDescriptor::memory_region(0x1000, 0x1000);
        let cap = Capability::new(
            subject,
            resource,
            Permission::READ | Permission::WRITE,
            Duration::from_secs(3600),
            SubjectId::ROOT,
        );

        assert!(cap.is_valid());
        assert!(cap.has_permission(Permission::READ));
        assert!(cap.has_permission(Permission::WRITE));
        assert!(!cap.has_permission(Permission::ADMIN));
    }

    #[test]
    fn test_capability_derivation() {
        let parent = Capability::new(
            SubjectId::new(1),
            ResourceDescriptor::memory_region(0x1000, 0x1000),
            Permission::READ | Permission::WRITE | Permission::GRANT,
            Duration::from_secs(3600),
            SubjectId::ROOT,
        );

        let child = parent.derive(
            Permission::READ,
            SubjectId::new(2),
            Duration::from_secs(1800),
        ).unwrap();

        assert!(child.has_permission(Permission::READ));
        assert!(!child.has_permission(Permission::WRITE));
        assert_eq!(child.parent, Some(parent.id));
    }

    #[test]
    fn test_capability_manager() {
        let manager = CapabilityManager::new();
        
        let cap = Capability::new(
            SubjectId::new(1),
            ResourceDescriptor::memory_region(0x1000, 0x1000),
            Permission::READ | Permission::WRITE,
            Duration::from_secs(3600),
            SubjectId::ROOT,
        );

        let token = cap.token.clone();
        let id = manager.store(cap).unwrap();

        let retrieved = manager.get(id).unwrap();
        assert_eq!(retrieved.id, id);

        let validated = manager.validate(&token).unwrap();
        assert_eq!(validated.id, id);
    }
}
