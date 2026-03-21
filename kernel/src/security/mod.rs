//! Security Subsystem for KolibriOS AI
//!
//! This module provides a comprehensive security framework implementing:
//!
//! # Capability-based Access Control (CbAC)
//! A capability is an unforgeable token that grants specific permissions to resources.
//! Unlike traditional ACLs, capabilities are bearer tokens - possession implies authority.
//!
//! # Access Control Lists (ACLs)
//! Fine-grained access control with allow/deny rules, audit logging, and
//! support for both users and processes as subjects.
//!
//! # Sandbox Isolation
//! Process isolation with resource limits, syscall filtering, and
//! filesystem/network isolation for secure execution of untrusted code.
//!
//! # Architecture
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Security Subsystem                        │
//! ├─────────────────┬─────────────────┬─────────────────────────┤
//! │   Capability    │  Access Control │       Sandbox          │
//! │     Manager     │     Manager     │       Manager          │
//! ├─────────────────┼─────────────────┼─────────────────────────┤
//! │ - Token gen     │ - ACLs          │ - Resource limits      │
//! │ - Permission    │ - ACEs          │ - Syscall filtering    │
//! │   flags         │ - Audit log     │ - FS isolation         │
//! │ - Validation    │ - Subject mgmt  │ - Network isolation    │
//! │ - Derivation    │ - Object mgmt   │ - Process isolation    │
//! └─────────────────┴─────────────────┴─────────────────────────┘
//! ```
//!
//! # Usage Example
//! ```rust
//! use kolibrios_kernel::security::{
//!     AccessControlManager, CapabilityManager, SandboxManager,
//!     Permission, ResourceDescriptor, SubjectId,
//! };
//!
//! // Create security managers
//! let acm = AccessControlManager::new();
//! let sandbox_mgr = SandboxManager::new();
//!
//! // Grant a capability
//! let cap = acm.grant_capability(
//!     SubjectId::new(1000),
//!     ResourceDescriptor::memory_region(0x1000, 0x1000),
//!     Permission::READ | Permission::WRITE,
//!     core::time::Duration::from_secs(3600),
//!     SubjectId::ROOT,
//! ).unwrap();
//!
//! // Create a sandbox
//! let sandbox_id = sandbox_mgr.create_sandbox("untrusted", SubjectId::new(1000)).unwrap();
//! sandbox_mgr.start_sandbox(sandbox_id).unwrap();
//! ```

extern crate alloc;

pub mod access_control;
pub mod capability;
pub mod sandbox;

// Re-export main types for convenience
pub use access_control::{
    AccessControlEntry, AccessControlError, AccessControlList, AccessControlManager,
    AccessControlStats, AccessDecision, AceId, AceType, AclId, AuditEntry, ObjectInfo,
    SubjectInfo, SubjectType,
};
pub use capability::{
    Capability, CapabilityError, CapabilityId, CapabilityManager, CapabilityStats,
    CapabilityToken, Permission, ResourceDescriptor, ResourceType, SubjectId, Timestamp,
};
pub use sandbox::{
    FilesystemIsolation, IsolationConfig, NetworkIsolation, ResourceLimits, Sandbox,
    SandboxError, SandboxId, SandboxManager, SandboxManagerStats, SandboxState, SandboxStats,
    SandboxedProcess, SandboxedProcessId, SyscallAction, SyscallFilter, SyscallFilterRule,
};

use alloc::string::String;
use alloc::sync::Arc;
use spin::RwLock;

/// Unified security manager that coordinates all security subsystems
pub struct SecurityManager {
    /// Access control manager
    access_control: AccessControlManager,
    /// Sandbox manager
    sandbox: SandboxManager,
    /// Security configuration
    config: SecurityConfig,
    /// Security statistics
    stats: RwLock<SecurityStats>,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Self {
        Self {
            access_control: AccessControlManager::new(),
            sandbox: SandboxManager::new(),
            config: SecurityConfig::default(),
            stats: RwLock::new(SecurityStats::default()),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: SecurityConfig) -> Self {
        Self {
            access_control: AccessControlManager::with_audit_log_size(config.max_audit_log_size),
            sandbox: SandboxManager::with_limits(config.max_sandboxes),
            config,
            stats: RwLock::new(SecurityStats::default()),
        }
    }

    /// Get the access control manager
    pub fn access_control(&self) -> &AccessControlManager {
        &self.access_control
    }

    /// Get the capability manager
    pub fn capability_manager(&self) -> &CapabilityManager {
        self.access_control.capability_manager()
    }

    /// Get the sandbox manager
    pub fn sandbox(&self) -> &SandboxManager {
        &self.sandbox
    }

    /// Get security configuration
    pub fn config(&self) -> &SecurityConfig {
        &self.config
    }

    /// Initialize default security setup
    pub fn init_default(&self) -> Result<(), SecurityError> {
        // Initialize default ACL
        access_control::init_default_acl(&self.access_control)
            .map_err(SecurityError::AccessControl)?;

        // Update stats
        self.update_stats();
        Ok(())
    }

    /// Check if a subject has access to a resource
    pub fn check_access(
        &self,
        subject: SubjectId,
        resource: &ResourceDescriptor,
        permission: Permission,
    ) -> AccessDecision {
        let decision = self.access_control.check_access(subject, resource, permission);
        self.update_stats();
        decision
    }

    /// Check access and create audit log entry
    pub fn check_and_audit(
        &self,
        subject: SubjectId,
        resource: &ResourceDescriptor,
        permission: Permission,
        description: &str,
    ) -> Result<AccessDecision, SecurityError> {
        let decision = self
            .access_control
            .check_and_audit(subject, resource, permission, description)
            .map_err(SecurityError::AccessControl)?;
        self.update_stats();
        Ok(decision)
    }

    /// Create a sandbox for a subject
    pub fn create_sandbox(
        &self,
        name: &str,
        owner: SubjectId,
        restricted: bool,
    ) -> Result<SandboxId, SecurityError> {
        let sandbox_id = if restricted {
            self.sandbox.create_restricted(name, owner)
        } else {
            self.sandbox.create_sandbox(name, owner)
        }
        .map_err(SecurityError::Sandbox)?;

        self.update_stats();
        Ok(sandbox_id)
    }

    /// Execute a process in a sandbox
    pub fn execute_in_sandbox(
        &self,
        sandbox_id: SandboxId,
        subject: SubjectId,
        process_name: &str,
    ) -> Result<SandboxedProcessId, SecurityError> {
        let process_id = self
            .sandbox
            .create_process(sandbox_id, subject, process_name)
            .map_err(SecurityError::Sandbox)?;

        self.update_stats();
        Ok(process_id)
    }

    /// Validate a capability token
    pub fn validate_capability(
        &self,
        token: &CapabilityToken,
    ) -> Result<Capability, SecurityError> {
        let cap = self
            .access_control
            .capability_manager()
            .validate(token)
            .map_err(SecurityError::Capability)?;
        self.update_stats();
        Ok(cap)
    }

    /// Grant a capability to a subject
    pub fn grant_capability(
        &self,
        subject: SubjectId,
        resource: ResourceDescriptor,
        permissions: Permission,
        ttl: core::time::Duration,
        granter: SubjectId,
    ) -> Result<CapabilityId, SecurityError> {
        let cap_id = self
            .access_control
            .grant_capability(subject, resource, permissions, ttl, granter)
            .map_err(SecurityError::AccessControl)?;
        self.update_stats();
        Ok(cap_id)
    }

    /// Revoke a capability
    pub fn revoke_capability(
        &self,
        cap_id: CapabilityId,
        revoker: SubjectId,
    ) -> Result<(), SecurityError> {
        self.access_control
            .revoke_capability(cap_id, revoker)
            .map_err(SecurityError::AccessControl)?;
        self.update_stats();
        Ok(())
    }

    /// Cleanup expired capabilities and terminated sandboxes
    pub fn cleanup(&self) -> CleanupStats {
        let capabilities_cleaned = self.access_control.capability_manager().cleanup_expired();
        let sandboxes_cleaned = self.sandbox.cleanup_terminated();

        self.update_stats();

        CleanupStats {
            capabilities_cleaned,
            sandboxes_cleaned,
        }
    }

    /// Get security statistics
    pub fn stats(&self) -> SecurityStats {
        *self.stats.read()
    }

    /// Update internal statistics
    fn update_stats(&self) {
        let ac_stats = self.access_control.stats();
        let sb_stats = self.sandbox.stats();

        let mut stats = self.stats.write();
        stats.total_subjects = ac_stats.subjects;
        stats.active_subjects = ac_stats.active_subjects;
        stats.total_objects = ac_stats.objects;
        stats.total_acls = ac_stats.acls;
        stats.total_aces = ac_stats.total_aces;
        stats.audit_log_entries = ac_stats.audit_entries;
        stats.capabilities = ac_stats.capabilities;
        stats.total_sandboxes = sb_stats.total_sandboxes;
        stats.running_sandboxes = sb_stats.running_sandboxes;
        stats.total_processes = sb_stats.total_processes;
        stats.active_processes = sb_stats.active_processes;
        stats.total_memory_usage = sb_stats.total_memory_usage;
    }

    /// Check system health
    pub fn health_check(&self) -> SecurityHealth {
        let stats = self.stats();
        let config = &self.config;

        let mut issues = alloc::vec::Vec::new();

        // Check capability limit
        if stats.capabilities.total > config.max_capabilities * 90 / 100 {
            issues.push(String::from(
                "Capability usage is above 90% of maximum limit",
            ));
        }

        // Check sandbox limit
        if stats.total_sandboxes > config.max_sandboxes * 90 / 100 {
            issues.push(String::from("Sandbox usage is above 90% of maximum limit"));
        }

        // Check audit log size
        if stats.audit_log_entries > config.max_audit_log_size * 90 / 100 {
            issues.push(String::from("Audit log is above 90% of maximum size"));
        }

        // Check memory usage
        if stats.total_memory_usage > config.max_total_memory * 90 / 100 {
            issues.push(String::from("Total memory usage is above 90% of limit"));
        }

        let status = if issues.is_empty() {
            SecurityStatus::Healthy
        } else if issues.len() < 3 {
            SecurityStatus::Warning
        } else {
            SecurityStatus::Critical
        };

        SecurityHealth { status, issues }
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Security configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecurityConfig {
    /// Maximum number of sandboxes
    pub max_sandboxes: usize,
    /// Maximum number of capabilities
    pub max_capabilities: usize,
    /// Maximum audit log size
    pub max_audit_log_size: usize,
    /// Maximum total memory for sandboxes
    pub max_total_memory: u64,
    /// Default capability TTL in seconds
    pub default_capability_ttl: u64,
    /// Enable audit logging by default
    pub audit_enabled: bool,
    /// Enable sandbox syscall logging
    pub syscall_logging: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_sandboxes: 1024,
            max_capabilities: 65536,
            max_audit_log_size: 10000,
            max_total_memory: 4 * 1024 * 1024 * 1024, // 4 GB
            default_capability_ttl: 3600,             // 1 hour
            audit_enabled: true,
            syscall_logging: false,
        }
    }
}

/// Security statistics
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SecurityStats {
    /// Total number of subjects
    pub total_subjects: usize,
    /// Number of active subjects
    pub active_subjects: usize,
    /// Total number of objects
    pub total_objects: usize,
    /// Total number of ACLs
    pub total_acls: usize,
    /// Total number of ACEs
    pub total_aces: usize,
    /// Audit log entries
    pub audit_log_entries: usize,
    /// Capability statistics
    pub capabilities: CapabilityStats,
    /// Total sandboxes
    pub total_sandboxes: usize,
    /// Running sandboxes
    pub running_sandboxes: usize,
    /// Total processes
    pub total_processes: usize,
    /// Active processes
    pub active_processes: usize,
    /// Total memory usage
    pub total_memory_usage: u64,
}

/// Cleanup statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CleanupStats {
    /// Number of capabilities cleaned
    pub capabilities_cleaned: usize,
    /// Number of sandboxes cleaned
    pub sandboxes_cleaned: usize,
}

/// Security health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityStatus {
    /// System is healthy
    Healthy,
    /// System has warnings
    Warning,
    /// System is in critical state
    Critical,
}

/// Security health check result
#[derive(Debug, Clone)]
pub struct SecurityHealth {
    /// Overall status
    pub status: SecurityStatus,
    /// List of issues found
    pub issues: alloc::vec::Vec<String>,
}

/// Unified security error type
#[derive(Debug, Clone)]
pub enum SecurityError {
    /// Capability error
    Capability(CapabilityError),
    /// Access control error
    AccessControl(AccessControlError),
    /// Sandbox error
    Sandbox(SandboxError),
    /// Configuration error
    Config(String),
    /// Internal error
    Internal(String),
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Capability(e) => write!(f, "Capability error: {}", e),
            Self::AccessControl(e) => write!(f, "Access control error: {}", e),
            Self::Sandbox(e) => write!(f, "Sandbox error: {}", e),
            Self::Config(msg) => write!(f, "Configuration error: {}", msg),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl From<CapabilityError> for SecurityError {
    fn from(e: CapabilityError) -> Self {
        Self::Capability(e)
    }
}

impl From<AccessControlError> for SecurityError {
    fn from(e: AccessControlError) -> Self {
        Self::AccessControl(e)
    }
}

impl From<SandboxError> for SecurityError {
    fn from(e: SandboxError) -> Self {
        Self::Sandbox(e)
    }
}

impl core::error::Error for SecurityError {}

#[cfg(test)]
mod tests {
    use super::*;
    use core::time::Duration;

    #[test]
    fn test_security_manager_creation() {
        let manager = SecurityManager::new();
        assert!(manager.init_default().is_ok());
    }

    #[test]
    fn test_security_manager_sandbox() {
        let manager = SecurityManager::new();
        let sandbox_id = manager
            .create_sandbox("test", SubjectId::new(1), false)
            .unwrap();

        let sandbox = manager.sandbox().get_sandbox(sandbox_id).unwrap();
        assert_eq!(sandbox.name, "test");
    }

    #[test]
    fn test_security_manager_capability() {
        let manager = SecurityManager::new();
        manager.init_default().unwrap();

        let cap_id = manager
            .grant_capability(
                SubjectId::new(1),
                ResourceDescriptor::memory_region(0x1000, 0x1000),
                Permission::READ,
                Duration::from_secs(3600),
                SubjectId::ROOT,
            )
            .unwrap();

        let cap = manager.capability_manager().get(cap_id).unwrap();
        assert!(cap.has_permission(Permission::READ));
    }

    #[test]
    fn test_security_health_check() {
        let manager = SecurityManager::new();
        manager.init_default().unwrap();

        let health = manager.health_check();
        assert_eq!(health.status, SecurityStatus::Healthy);
    }
}
