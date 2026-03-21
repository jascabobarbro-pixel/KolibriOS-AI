//! Access Control System for KolibriOS AI
//!
//! This module implements a comprehensive access control system that combines
//! capability-based security with Access Control Lists (ACLs). It manages
//! subjects (users/processes), objects (resources), and their associated
//! permissions with full audit logging.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use spin::RwLock;

extern crate alloc;

use super::capability::{
    Capability, CapabilityError, CapabilityId, CapabilityManager, Permission, ResourceDescriptor,
    ResourceType, SubjectId, Timestamp,
};

/// Unique identifier for an Access Control Entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AceId(pub u64);

impl fmt::Display for AceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ACE-{:016X}", self.0)
    }
}

/// Unique identifier for an Access Control List
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AclId(pub u64);

impl fmt::Display for AclId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ACL-{:016X}", self.0)
    }
}

/// Access Control Entry type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AceType {
    /// Allow access
    Allow,
    /// Deny access (takes precedence over Allow)
    Deny,
    /// Audit access (log but don't enforce)
    Audit,
    /// Alarm on access (alert + log)
    Alarm,
}

impl AceType {
    /// Check if this ACE type allows access
    pub const fn allows(&self) -> bool {
        matches!(self, Self::Allow)
    }

    /// Check if this ACE type denies access
    pub const fn denies(&self) -> bool {
        matches!(self, Self::Deny)
    }
}

/// Access Control Entry
#[derive(Debug, Clone)]
pub struct AccessControlEntry {
    /// Unique identifier for this ACE
    pub id: AceId,
    /// Type of this ACE
    pub ace_type: AceType,
    /// Subject this ACE applies to (None = all subjects)
    pub subject: Option<SubjectId>,
    /// Permissions granted or denied
    pub permissions: Permission,
    /// Resource this ACE applies to
    pub resource: ResourceDescriptor,
    /// Whether this ACE can be inherited
    pub inheritable: bool,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Who created this ACE
    pub creator: SubjectId,
    /// Optional description
    pub description: Option<String>,
}

impl AccessControlEntry {
    /// Create a new ACE
    pub fn new(
        ace_type: AceType,
        subject: Option<SubjectId>,
        permissions: Permission,
        resource: ResourceDescriptor,
        creator: SubjectId,
    ) -> Self {
        static COUNTER: spin::Mutex<u64> = spin::Mutex::new(1);
        let id = {
            let mut counter = COUNTER.lock();
            let id = *counter;
            *counter += 1;
            AceId(id)
        };

        Self {
            id,
            ace_type,
            subject,
            permissions,
            resource,
            inheritable: true,
            created_at: Timestamp::now(),
            creator,
            description: None,
        }
    }

    /// Create an allow ACE
    pub fn allow(
        subject: SubjectId,
        permissions: Permission,
        resource: ResourceDescriptor,
        creator: SubjectId,
    ) -> Self {
        Self::new(AceType::Allow, Some(subject), permissions, resource, creator)
    }

    /// Create a deny ACE
    pub fn deny(
        subject: SubjectId,
        permissions: Permission,
        resource: ResourceDescriptor,
        creator: SubjectId,
    ) -> Self {
        Self::new(AceType::Deny, Some(subject), permissions, resource, creator)
    }

    /// Create an audit ACE
    pub fn audit(
        subject: SubjectId,
        permissions: Permission,
        resource: ResourceDescriptor,
        creator: SubjectId,
    ) -> Self {
        Self::new(AceType::Audit, Some(subject), permissions, resource, creator)
    }

    /// Create an alarm ACE
    pub fn alarm(
        subject: SubjectId,
        permissions: Permission,
        resource: ResourceDescriptor,
        creator: SubjectId,
    ) -> Self {
        Self::new(AceType::Alarm, Some(subject), permissions, resource, creator)
    }

    /// Check if this ACE matches a subject
    pub fn matches_subject(&self, subject: SubjectId) -> bool {
        self.subject.is_none() || self.subject == Some(subject)
    }

    /// Check if this ACE matches a resource
    pub fn matches_resource(&self, resource: &ResourceDescriptor) -> bool {
        self.resource.matches(resource)
    }

    /// Check if this ACE applies to a specific permission
    pub fn applies_to(&self, permission: Permission) -> bool {
        self.permissions.contains(permission)
    }

    /// Set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(String::from(desc));
        self
    }

    /// Set inheritable flag
    pub fn with_inheritable(mut self, inheritable: bool) -> Self {
        self.inheritable = inheritable;
        self
    }
}

/// Access Control List
#[derive(Debug, Clone)]
pub struct AccessControlList {
    /// Unique identifier for this ACL
    pub id: AclId,
    /// The resource this ACL protects
    pub resource: ResourceDescriptor,
    /// List of Access Control Entries
    pub entries: Vec<AccessControlEntry>,
    /// Parent ACL (for inheritance)
    pub parent: Option<AclId>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last modified timestamp
    pub modified_at: Timestamp,
    /// Owner of this ACL
    pub owner: SubjectId,
}

impl AccessControlList {
    /// Create a new ACL for a resource
    pub fn new(resource: ResourceDescriptor, owner: SubjectId) -> Self {
        static COUNTER: spin::Mutex<u64> = spin::Mutex::new(1);
        let id = {
            let mut counter = COUNTER.lock();
            let id = *counter;
            *counter += 1;
            AclId(id)
        };

        let now = Timestamp::now();

        Self {
            id,
            resource,
            entries: Vec::new(),
            parent: None,
            created_at: now,
            modified_at: now,
            owner,
        }
    }

    /// Add an entry to this ACL
    pub fn add_entry(&mut self, entry: AccessControlEntry) {
        self.entries.push(entry);
        self.modified_at = Timestamp::now();
    }

    /// Remove an entry from this ACL
    pub fn remove_entry(&mut self, entry_id: AceId) -> bool {
        let initial_len = self.entries.len();
        self.entries.retain(|e| e.id != entry_id);
        if self.entries.len() != initial_len {
            self.modified_at = Timestamp::now();
            true
        } else {
            false
        }
    }

    /// Get all entries for a specific subject
    pub fn get_entries_for_subject(&self, subject: SubjectId) -> Vec<&AccessControlEntry> {
        self.entries
            .iter()
            .filter(|e| e.matches_subject(subject))
            .collect()
    }

    /// Check access for a subject on a specific permission
    pub fn check_access(&self, subject: SubjectId, permission: Permission) -> AccessDecision {
        let mut allow_perms = Permission::NONE;
        let mut deny_perms = Permission::NONE;
        let mut audit_entries = Vec::new();
        let mut alarm_entries = Vec::new();

        for entry in &self.entries {
            if !entry.matches_subject(subject) || !entry.applies_to(permission) {
                continue;
            }

            match entry.ace_type {
                AceType::Allow => {
                    allow_perms = allow_perms | entry.permissions;
                }
                AceType::Deny => {
                    deny_perms = deny_perms | entry.permissions;
                }
                AceType::Audit => {
                    audit_entries.push(entry.id);
                }
                AceType::Alarm => {
                    alarm_entries.push(entry.id);
                }
            }
        }

        // Deny takes precedence
        if deny_perms.contains(permission) {
            return AccessDecision::Denied {
                reason: String::from("Explicit deny in ACL"),
                ace_ids: self
                    .entries
                    .iter()
                    .filter(|e| e.ace_type == AceType::Deny && e.applies_to(permission))
                    .map(|e| e.id)
                    .collect(),
            };
        }

        // Check if allowed
        if allow_perms.contains(permission) {
            if !audit_entries.is_empty() || !alarm_entries.is_empty() {
                return AccessDecision::AllowedWithAudit {
                    ace_ids: self
                        .entries
                        .iter()
                        .filter(|e| e.ace_type == AceType::Allow && e.applies_to(permission))
                        .map(|e| e.id)
                        .collect(),
                    audit_ace_ids: audit_entries,
                    alarm_ace_ids: alarm_entries,
                };
            }
            return AccessDecision::Allowed {
                ace_ids: self
                    .entries
                    .iter()
                    .filter(|e| e.ace_type == AceType::Allow && e.applies_to(permission))
                    .map(|e| e.id)
                    .collect(),
            };
        }

        // No matching entry - implicit deny
        AccessDecision::Denied {
            reason: String::from("No matching allow entry"),
            ace_ids: Vec::new(),
        }
    }

    /// Check if subject has any access to the resource
    pub fn has_any_access(&self, subject: SubjectId) -> bool {
        self.entries.iter().any(|e| {
            e.matches_subject(subject) && e.ace_type == AceType::Allow && !e.permissions.is_empty()
        })
    }

    /// Get all permissions for a subject
    pub fn get_subject_permissions(&self, subject: SubjectId) -> Permission {
        let mut perms = Permission::NONE;
        for entry in &self.entries {
            if entry.matches_subject(subject) && entry.ace_type == AceType::Allow {
                perms = perms | entry.permissions;
            }
        }
        // Remove denied permissions
        for entry in &self.entries {
            if entry.matches_subject(subject) && entry.ace_type == AceType::Deny {
                perms = perms.remove(entry.permissions);
            }
        }
        perms
    }
}

/// Result of an access check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessDecision {
    /// Access is allowed
    Allowed {
        /// ACEs that granted access
        ace_ids: Vec<AceId>,
    },
    /// Access is allowed but should be audited
    AllowedWithAudit {
        /// ACEs that granted access
        ace_ids: Vec<AceId>,
        /// ACEs that require audit logging
        audit_ace_ids: Vec<AceId>,
        /// ACEs that require alarms
        alarm_ace_ids: Vec<AceId>,
    },
    /// Access is denied
    Denied {
        /// Reason for denial
        reason: String,
        /// ACEs that caused denial
        ace_ids: Vec<AceId>,
    },
}

impl AccessDecision {
    /// Check if access is allowed
    pub const fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed { .. } | Self::AllowedWithAudit { .. })
    }

    /// Check if access is denied
    pub const fn is_denied(&self) -> bool {
        matches!(self, Self::Denied { .. })
    }

    /// Check if audit is required
    pub const fn requires_audit(&self) -> bool {
        matches!(self, Self::AllowedWithAudit { .. })
    }
}

/// Audit log entry
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// Unique identifier for this audit entry
    pub id: u64,
    /// Subject that performed the action
    pub subject: SubjectId,
    /// Resource accessed
    pub resource: ResourceDescriptor,
    /// Permission requested
    pub permission: Permission,
    /// Whether access was granted
    pub granted: bool,
    /// Timestamp of the access
    pub timestamp: Timestamp,
    /// Description of the access
    pub description: String,
    /// Associated ACE IDs
    pub ace_ids: Vec<AceId>,
    /// Associated capability ID (if any)
    pub capability_id: Option<CapabilityId>,
}

impl AuditEntry {
    /// Create a new audit entry
    pub fn new(
        subject: SubjectId,
        resource: ResourceDescriptor,
        permission: Permission,
        granted: bool,
        description: &str,
    ) -> Self {
        static COUNTER: spin::Mutex<u64> = spin::Mutex::new(1);
        let id = {
            let mut counter = COUNTER.lock();
            let id = *counter;
            *counter += 1;
            id
        };

        Self {
            id,
            subject,
            resource,
            permission,
            granted,
            timestamp: Timestamp::now(),
            description: String::from(description),
            ace_ids: Vec::new(),
            capability_id: None,
        }
    }

    /// Set associated ACE IDs
    pub fn with_ace_ids(mut self, ace_ids: Vec<AceId>) -> Self {
        self.ace_ids = ace_ids;
        self
    }

    /// Set associated capability ID
    pub fn with_capability(mut self, cap_id: CapabilityId) -> Self {
        self.capability_id = Some(cap_id);
        self
    }
}

/// Subject information
#[derive(Debug, Clone)]
pub struct SubjectInfo {
    /// Subject identifier
    pub id: SubjectId,
    /// Subject name
    pub name: String,
    /// Subject type
    pub subject_type: SubjectType,
    /// Groups this subject belongs to
    pub groups: Vec<SubjectId>,
    /// Whether the subject is active
    pub active: bool,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last activity timestamp
    pub last_activity: Timestamp,
}

/// Subject type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectType {
    /// Kernel process
    Kernel,
    /// User process
    Process,
    /// User account
    User,
    /// Group
    Group,
    /// Service
    Service,
}

impl SubjectInfo {
    /// Create a new subject
    pub fn new(id: SubjectId, name: &str, subject_type: SubjectType) -> Self {
        let now = Timestamp::now();
        Self {
            id,
            name: String::from(name),
            subject_type,
            groups: Vec::new(),
            active: true,
            created_at: now,
            last_activity: now,
        }
    }

    /// Add this subject to a group
    pub fn add_to_group(&mut self, group: SubjectId) {
        if !self.groups.contains(&group) {
            self.groups.push(group);
        }
    }

    /// Remove this subject from a group
    pub fn remove_from_group(&mut self, group: SubjectId) {
        self.groups.retain(|g| *g != group);
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Timestamp::now();
    }
}

/// Resource information
#[derive(Debug, Clone)]
pub struct ObjectInfo {
    /// Resource descriptor
    pub resource: ResourceDescriptor,
    /// Human-readable name
    pub name: String,
    /// Owning subject
    pub owner: SubjectId,
    /// Associated ACL ID
    pub acl_id: Option<AclId>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last accessed timestamp
    pub last_accessed: Timestamp,
    /// Last modified timestamp
    pub last_modified: Timestamp,
    /// Whether the resource is active
    pub active: bool,
}

impl ObjectInfo {
    /// Create a new object info
    pub fn new(resource: ResourceDescriptor, name: &str, owner: SubjectId) -> Self {
        let now = Timestamp::now();
        Self {
            resource,
            name: String::from(name),
            owner,
            acl_id: None,
            created_at: now,
            last_accessed: now,
            last_modified: now,
            active: true,
        }
    }

    /// Set the associated ACL
    pub fn with_acl(mut self, acl_id: AclId) -> Self {
        self.acl_id = Some(acl_id);
        self
    }

    /// Update last accessed timestamp
    pub fn touch_access(&mut self) {
        self.last_accessed = Timestamp::now();
    }

    /// Update last modified timestamp
    pub fn touch_modify(&mut self) {
        self.last_modified = Timestamp::now();
    }
}

/// Errors that can occur during access control operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessControlError {
    /// Subject not found
    SubjectNotFound(SubjectId),
    /// Object not found
    ObjectNotFound(ResourceDescriptor),
    /// ACL not found
    AclNotFound(AclId),
    /// ACE not found
    AceNotFound(AceId),
    /// Access denied
    AccessDenied { subject: SubjectId, resource: ResourceDescriptor, permission: Permission },
    /// Permission denied for operation
    PermissionDenied(String),
    /// Invalid operation
    InvalidOperation(String),
    /// Capability error
    CapabilityError(CapabilityError),
    /// Internal error
    Internal(String),
}

impl fmt::Display for AccessControlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SubjectNotFound(id) => write!(f, "Subject not found: {}", id),
            Self::ObjectNotFound(res) => write!(f, "Object not found: {:?}", res),
            Self::AclNotFound(id) => write!(f, "ACL not found: {}", id),
            Self::AceNotFound(id) => write!(f, "ACE not found: {}", id),
            Self::AccessDenied { subject, resource, permission } => {
                write!(f, "Access denied for {} to {:?} with {:?}", subject, resource, permission)
            }
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Self::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            Self::CapabilityError(e) => write!(f, "Capability error: {}", e),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl From<CapabilityError> for AccessControlError {
    fn from(e: CapabilityError) -> Self {
        Self::CapabilityError(e)
    }
}

impl core::error::Error for AccessControlError {}

/// Access Control Manager
pub struct AccessControlManager {
    /// Capability manager
    capability_manager: CapabilityManager,
    /// All ACLs indexed by ID
    acls: RwLock<BTreeMap<AclId, AccessControlList>>,
    /// Resource to ACL mapping
    resource_acl_map: RwLock<BTreeMap<(ResourceType, u64), AclId>>,
    /// All subjects
    subjects: RwLock<BTreeMap<SubjectId, SubjectInfo>>,
    /// All objects
    objects: RwLock<BTreeMap<(ResourceType, u64), ObjectInfo>>,
    /// Audit log
    audit_log: RwLock<Vec<AuditEntry>>,
    /// Maximum audit log size
    max_audit_log_size: usize,
}

impl AccessControlManager {
    /// Create a new access control manager
    pub fn new() -> Self {
        Self {
            capability_manager: CapabilityManager::new(),
            acls: RwLock::new(BTreeMap::new()),
            resource_acl_map: RwLock::new(BTreeMap::new()),
            subjects: RwLock::new(BTreeMap::new()),
            objects: RwLock::new(BTreeMap::new()),
            audit_log: RwLock::new(Vec::new()),
            max_audit_log_size: 10000,
        }
    }

    /// Create with custom audit log size
    pub fn with_audit_log_size(max_size: usize) -> Self {
        Self {
            capability_manager: CapabilityManager::new(),
            acls: RwLock::new(BTreeMap::new()),
            resource_acl_map: RwLock::new(BTreeMap::new()),
            subjects: RwLock::new(BTreeMap::new()),
            objects: RwLock::new(BTreeMap::new()),
            audit_log: RwLock::new(Vec::new()),
            max_audit_log_size: max_size,
        }
    }

    /// Register a new subject
    pub fn register_subject(&self, info: SubjectInfo) -> Result<(), AccessControlError> {
        let mut subjects = self.subjects.write();
        subjects.insert(info.id, info);
        Ok(())
    }

    /// Unregister a subject
    pub fn unregister_subject(&self, id: SubjectId) -> Result<(), AccessControlError> {
        let mut subjects = self.subjects.write();
        subjects.remove(&id).ok_or(AccessControlError::SubjectNotFound(id))?;
        Ok(())
    }

    /// Get subject information
    pub fn get_subject(&self, id: SubjectId) -> Option<SubjectInfo> {
        let subjects = self.subjects.read();
        subjects.get(&id).cloned()
    }

    /// Update subject activity
    pub fn touch_subject(&self, id: SubjectId) -> Result<(), AccessControlError> {
        let mut subjects = self.subjects.write();
        let subject = subjects.get_mut(&id).ok_or(AccessControlError::SubjectNotFound(id))?;
        subject.touch();
        Ok(())
    }

    /// Register a new object
    pub fn register_object(&self, info: ObjectInfo) -> Result<(), AccessControlError> {
        let key = (info.resource.resource_type, info.resource.resource_id);
        let mut objects = self.objects.write();
        objects.insert(key, info);
        Ok(())
    }

    /// Unregister an object
    pub fn unregister_object(&self, resource: &ResourceDescriptor) -> Result<(), AccessControlError> {
        let key = (resource.resource_type, resource.resource_id);
        let mut objects = self.objects.write();
        objects.remove(&key).ok_or(AccessControlError::ObjectNotFound(resource.clone()))?;
        Ok(())
    }

    /// Get object information
    pub fn get_object(&self, resource: &ResourceDescriptor) -> Option<ObjectInfo> {
        let key = (resource.resource_type, resource.resource_id);
        let objects = self.objects.read();
        objects.get(&key).cloned()
    }

    /// Create an ACL for a resource
    pub fn create_acl(
        &self,
        resource: ResourceDescriptor,
        owner: SubjectId,
    ) -> Result<AclId, AccessControlError> {
        let acl = AccessControlList::new(resource.clone(), owner);
        let acl_id = acl.id;
        let key = (resource.resource_type, resource.resource_id);

        {
            let mut acls = self.acls.write();
            acls.insert(acl_id, acl);
        }

        {
            let mut map = self.resource_acl_map.write();
            map.insert(key, acl_id);
        }

        // Update object info
        if let Some(obj) = self.get_object(&resource) {
            self.register_object(ObjectInfo::new(resource.clone(), &obj.name, obj.owner).with_acl(acl_id))?;
        }

        Ok(acl_id)
    }

    /// Get an ACL by ID
    pub fn get_acl(&self, id: AclId) -> Option<AccessControlList> {
        let acls = self.acls.read();
        acls.get(&id).cloned()
    }

    /// Get ACL for a resource
    pub fn get_acl_for_resource(&self, resource: &ResourceDescriptor) -> Option<AccessControlList> {
        let key = (resource.resource_type, resource.resource_id);
        let map = self.resource_acl_map.read();
        let acl_id = map.get(&key)?;
        self.get_acl(*acl_id)
    }

    /// Add an ACE to an ACL
    pub fn add_ace(&self, acl_id: AclId, entry: AccessControlEntry) -> Result<(), AccessControlError> {
        let mut acls = self.acls.write();
        let acl = acls.get_mut(&acl_id).ok_or(AccessControlError::AclNotFound(acl_id))?;
        acl.add_entry(entry);
        Ok(())
    }

    /// Remove an ACE from an ACL
    pub fn remove_ace(&self, acl_id: AclId, ace_id: AceId) -> Result<(), AccessControlError> {
        let mut acls = self.acls.write();
        let acl = acls.get_mut(&acl_id).ok_or(AccessControlError::AclNotFound(acl_id))?;
        if !acl.remove_entry(ace_id) {
            return Err(AccessControlError::AceNotFound(ace_id));
        }
        Ok(())
    }

    /// Check access for a subject on a resource
    pub fn check_access(
        &self,
        subject: SubjectId,
        resource: &ResourceDescriptor,
        permission: Permission,
    ) -> AccessDecision {
        // First check capabilities
        if self.capability_manager.check_permission(subject, resource, permission) {
            return AccessDecision::Allowed { ace_ids: Vec::new() };
        }

        // Then check ACLs
        if let Some(acl) = self.get_acl_for_resource(resource) {
            return acl.check_access(subject, permission);
        }

        // Check if subject is owner
        if let Some(obj) = self.get_object(resource) {
            if obj.owner == subject {
                return AccessDecision::Allowed { ace_ids: Vec::new() };
            }
        }

        // Check if subject is root
        if subject == SubjectId::ROOT {
            return AccessDecision::Allowed { ace_ids: Vec::new() };
        }

        AccessDecision::Denied {
            reason: String::from("No access rights found"),
            ace_ids: Vec::new(),
        }
    }

    /// Check access and log audit
    pub fn check_and_audit(
        &self,
        subject: SubjectId,
        resource: &ResourceDescriptor,
        permission: Permission,
        description: &str,
    ) -> Result<AccessDecision, AccessControlError> {
        let decision = self.check_access(subject, resource, permission);

        // Create audit entry
        let mut audit = AuditEntry::new(
            subject,
            resource.clone(),
            permission,
            decision.is_allowed(),
            description,
        );

        // Add ACE IDs if present
        match &decision {
            AccessDecision::Allowed { ace_ids } => {
                audit = audit.with_ace_ids(ace_ids.clone());
            }
            AccessDecision::AllowedWithAudit { ace_ids, audit_ace_ids, alarm_ace_ids } => {
                let mut all_ids = ace_ids.clone();
                all_ids.extend(audit_ace_ids.iter().cloned());
                all_ids.extend(alarm_ace_ids.iter().cloned());
                audit = audit.with_ace_ids(all_ids);
            }
            AccessDecision::Denied { ace_ids, .. } => {
                audit = audit.with_ace_ids(ace_ids.clone());
            }
        }

        // Add to audit log
        {
            let mut log = self.audit_log.write();
            log.push(audit);
            // Trim if too large
            while log.len() > self.max_audit_log_size {
                log.remove(0);
            }
        }

        Ok(decision)
    }

    /// Request access (returns error if denied)
    pub fn request_access(
        &self,
        subject: SubjectId,
        resource: &ResourceDescriptor,
        permission: Permission,
    ) -> Result<(), AccessControlError> {
        let decision = self.check_access(subject, resource, permission);
        
        if decision.is_allowed() {
            Ok(())
        } else {
            Err(AccessControlError::AccessDenied {
                subject,
                resource: resource.clone(),
                permission,
            })
        }
    }

    /// Get the capability manager
    pub fn capability_manager(&self) -> &CapabilityManager {
        &self.capability_manager
    }

    /// Grant a capability to a subject
    pub fn grant_capability(
        &self,
        subject: SubjectId,
        resource: ResourceDescriptor,
        permissions: Permission,
        ttl: core::time::Duration,
        granter: SubjectId,
    ) -> Result<CapabilityId, AccessControlError> {
        // Check if granter has GRANT permission
        let granter_can_grant = granter == SubjectId::ROOT
            || self.check_access(granter, &resource, Permission::GRANT).is_allowed();

        if !granter_can_grant {
            return Err(AccessControlError::PermissionDenied(
                String::from("Granter does not have GRANT permission"),
            ));
        }

        // Create and store capability
        let cap = Capability::new(subject, resource, permissions, ttl, granter);
        let cap_id = self.capability_manager.store(cap)?;

        Ok(cap_id)
    }

    /// Revoke a capability
    pub fn revoke_capability(
        &self,
        cap_id: CapabilityId,
        revoker: SubjectId,
    ) -> Result<(), AccessControlError> {
        self.capability_manager.revoke(cap_id, revoker)?;
        Ok(())
    }

    /// Get audit log entries
    pub fn get_audit_log(&self, limit: usize) -> Vec<AuditEntry> {
        let log = self.audit_log.read();
        log.iter().rev().take(limit).cloned().collect()
    }

    /// Get audit log for a specific subject
    pub fn get_subject_audit_log(&self, subject: SubjectId, limit: usize) -> Vec<AuditEntry> {
        let log = self.audit_log.read();
        log.iter()
            .rev()
            .filter(|e| e.subject == subject)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get audit log for a specific resource
    pub fn get_resource_audit_log(
        &self,
        resource: &ResourceDescriptor,
        limit: usize,
    ) -> Vec<AuditEntry> {
        let log = self.audit_log.read();
        log.iter()
            .rev()
            .filter(|e| &e.resource == resource)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Clear audit log
    pub fn clear_audit_log(&self) {
        let mut log = self.audit_log.write();
        log.clear();
    }

    /// Get access control statistics
    pub fn stats(&self) -> AccessControlStats {
        let subjects = self.subjects.read();
        let objects = self.objects.read();
        let acls = self.acls.read();
        let audit_log = self.audit_log.read();
        let cap_stats = self.capability_manager.stats();

        AccessControlStats {
            subjects: subjects.len(),
            active_subjects: subjects.values().filter(|s| s.active).count(),
            objects: objects.len(),
            acls: acls.len(),
            total_aces: acls.values().map(|a| a.entries.len()).sum(),
            audit_entries: audit_log.len(),
            capabilities: cap_stats,
        }
    }
}

impl Default for AccessControlManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Access control statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessControlStats {
    /// Number of subjects
    pub subjects: usize,
    /// Number of active subjects
    pub active_subjects: usize,
    /// Number of objects
    pub objects: usize,
    /// Number of ACLs
    pub acls: usize,
    /// Total number of ACEs
    pub total_aces: usize,
    /// Number of audit log entries
    pub audit_entries: usize,
    /// Capability statistics
    pub capabilities: super::capability::CapabilityStats,
}

/// Initialize the default access control system
pub fn init_default_acl(acm: &AccessControlManager) -> Result<(), AccessControlError> {
    // Create root subject
    let root = SubjectInfo::new(SubjectId::ROOT, "root", SubjectType::User);
    acm.register_subject(root)?;

    // Create kernel subject
    let kernel = SubjectInfo::new(SubjectId::KERNEL, "kernel", SubjectType::Kernel);
    acm.register_subject(kernel)?;

    // Create default ACL for kernel resources
    let kernel_resource = ResourceDescriptor::new(ResourceType::Kernel, 0);
    let acl_id = acm.create_acl(kernel_resource.clone(), SubjectId::KERNEL)?;

    // Allow kernel full access
    let allow_kernel = AccessControlEntry::allow(
        SubjectId::KERNEL,
        Permission::ALL,
        kernel_resource.clone(),
        SubjectId::ROOT,
    );
    acm.add_ace(acl_id, allow_kernel)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::time::Duration;

    #[test]
    fn test_ace_creation() {
        let ace = AccessControlEntry::allow(
            SubjectId::new(1),
            Permission::READ | Permission::WRITE,
            ResourceDescriptor::memory_region(0x1000, 0x1000),
            SubjectId::ROOT,
        );

        assert_eq!(ace.ace_type, AceType::Allow);
        assert!(ace.permissions.contains(Permission::READ));
        assert!(ace.permissions.contains(Permission::WRITE));
    }

    #[test]
    fn test_acl_access_check() {
        let mut acl = AccessControlList::new(
            ResourceDescriptor::memory_region(0x1000, 0x1000),
            SubjectId::ROOT,
        );

        acl.add_entry(AccessControlEntry::allow(
            SubjectId::new(1),
            Permission::READ,
            ResourceDescriptor::memory_region(0x1000, 0x1000),
            SubjectId::ROOT,
        ));

        acl.add_entry(AccessControlEntry::deny(
            SubjectId::new(2),
            Permission::WRITE,
            ResourceDescriptor::memory_region(0x1000, 0x1000),
            SubjectId::ROOT,
        ));

        // Subject 1 should have READ access
        let decision = acl.check_access(SubjectId::new(1), Permission::READ);
        assert!(decision.is_allowed());

        // Subject 1 should not have WRITE access
        let decision = acl.check_access(SubjectId::new(1), Permission::WRITE);
        assert!(decision.is_denied());
    }

    #[test]
    fn test_access_control_manager() {
        let acm = AccessControlManager::new();

        // Register subjects
        let user1 = SubjectInfo::new(SubjectId::new(1), "user1", SubjectType::User);
        acm.register_subject(user1).unwrap();

        // Create ACL
        let resource = ResourceDescriptor::memory_region(0x1000, 0x1000);
        let acl_id = acm.create_acl(resource.clone(), SubjectId::ROOT).unwrap();

        // Add ACE
        let ace = AccessControlEntry::allow(
            SubjectId::new(1),
            Permission::READ,
            resource.clone(),
            SubjectId::ROOT,
        );
        acm.add_ace(acl_id, ace).unwrap();

        // Check access
        let decision = acm.check_access(SubjectId::new(1), &resource, Permission::READ);
        assert!(decision.is_allowed());

        let decision = acm.check_access(SubjectId::new(2), &resource, Permission::READ);
        assert!(decision.is_denied());
    }
}
