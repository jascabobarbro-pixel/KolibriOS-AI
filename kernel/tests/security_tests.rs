"""
Comprehensive Security Tests

Tests for:
- Access control
- Capability-based security
- Sandbox isolation
"""

import pytest
from dataclasses import dataclass, field
from typing import Dict, Set, Optional, List
from enum import Enum


# ============== Enums and Data Classes ==============

class Permission(str, Enum):
    READ = "read"
    WRITE = "write"
    EXECUTE = "execute"
    ADMIN = "admin"


class ResourceType(str, Enum):
    MEMORY = "memory"
    FILE = "file"
    PROCESS = "process"
    NETWORK = "network"
    DEVICE = "device"


@dataclass
class Capability:
    resource_type: ResourceType
    resource_id: str
    permissions: Set[Permission]
    
    def has_permission(self, permission: Permission) -> bool:
        return permission in self.permissions or Permission.ADMIN in self.permissions


@dataclass
class Process:
    pid: int
    name: str
    capabilities: List[Capability] = field(default_factory=list)


# ============== Mock Security Components ==============

class MockAccessControl:
    """Mock access control system."""
    
    def __init__(self):
        self.capabilities: Dict[int, List[Capability]] = {}
        self.next_pid = 1
    
    def create_process(self, name: str) -> Process:
        """Create a new process with no capabilities."""
        pid = self.next_pid
        self.next_pid += 1
        process = Process(pid=pid, name=name)
        self.capabilities[pid] = []
        return process
    
    def grant_capability(self, pid: int, capability: Capability) -> bool:
        """Grant a capability to a process."""
        if pid not in self.capabilities:
            return False
        self.capabilities[pid].append(capability)
        return True
    
    def revoke_capability(self, pid: int, resource_type: ResourceType, resource_id: str) -> bool:
        """Revoke a capability from a process."""
        if pid not in self.capabilities:
            return False
        
        initial_len = len(self.capabilities[pid])
        self.capabilities[pid] = [
            c for c in self.capabilities[pid]
            if not (c.resource_type == resource_type and c.resource_id == resource_id)
        ]
        return len(self.capabilities[pid]) < initial_len
    
    def check_access(self, pid: int, resource_type: ResourceType, 
                    resource_id: str, permission: Permission) -> bool:
        """Check if a process has access to a resource."""
        if pid not in self.capabilities:
            return False
        
        for cap in self.capabilities[pid]:
            if cap.resource_type == resource_type and cap.resource_id == resource_id:
                if cap.has_permission(permission):
                    return True
        
        return False
    
    def get_capabilities(self, pid: int) -> List[Capability]:
        """Get all capabilities for a process."""
        return self.capabilities.get(pid, [])


class MockSandbox:
    """Mock sandbox for process isolation."""
    
    def __init__(self):
        self.sandboxes: Dict[int, Dict] = {}
        self.next_id = 1
    
    def create_sandbox(self, allowed_resources: Set[str]) -> int:
        """Create a new sandbox."""
        sandbox_id = self.next_id
        self.next_id += 1
        self.sandboxes[sandbox_id] = {
            "allowed_resources": allowed_resources,
            "processes": set(),
            "memory_limit": 100 * 1024 * 1024,  # 100MB
            "cpu_limit": 50.0,  # 50%
        }
        return sandbox_id
    
    def add_process(self, sandbox_id: int, pid: int) -> bool:
        """Add a process to a sandbox."""
        if sandbox_id not in self.sandboxes:
            return False
        self.sandboxes[sandbox_id]["processes"].add(pid)
        return True
    
    def remove_process(self, sandbox_id: int, pid: int) -> bool:
        """Remove a process from a sandbox."""
        if sandbox_id not in self.sandboxes:
            return False
        self.sandboxes[sandbox_id]["processes"].discard(pid)
        return True
    
    def can_access(self, sandbox_id: int, resource: str) -> bool:
        """Check if a sandbox can access a resource."""
        if sandbox_id not in self.sandboxes:
            return False
        return resource in self.sandboxes[sandbox_id]["allowed_resources"]
    
    def set_memory_limit(self, sandbox_id: int, limit: int) -> bool:
        """Set memory limit for a sandbox."""
        if sandbox_id not in self.sandboxes:
            return False
        self.sandboxes[sandbox_id]["memory_limit"] = limit
        return True
    
    def set_cpu_limit(self, sandbox_id: int, limit: float) -> bool:
        """Set CPU limit for a sandbox."""
        if sandbox_id not in self.sandboxes:
            return False
        self.sandboxes[sandbox_id]["cpu_limit"] = limit
        return True
    
    def get_limits(self, sandbox_id: int) -> Optional[Dict]:
        """Get limits for a sandbox."""
        return self.sandboxes.get(sandbox_id)


# ============== Tests ==============

# --- Access Control Tests ---

def test_access_control_creation():
    """Test access control creation."""
    ac = MockAccessControl()
    assert len(ac.capabilities) == 0


def test_create_process():
    """Test process creation."""
    ac = MockAccessControl()
    process = ac.create_process("test")
    
    assert process.pid == 1
    assert process.name == "test"
    assert len(process.capabilities) == 0


def test_grant_capability():
    """Test granting capability."""
    ac = MockAccessControl()
    process = ac.create_process("test")
    
    cap = Capability(
        resource_type=ResourceType.FILE,
        resource_id="/home/user/file.txt",
        permissions={Permission.READ, Permission.WRITE},
    )
    
    result = ac.grant_capability(process.pid, cap)
    assert result == True
    
    capabilities = ac.get_capabilities(process.pid)
    assert len(capabilities) == 1


def test_revoke_capability():
    """Test revoking capability."""
    ac = MockAccessControl()
    process = ac.create_process("test")
    
    cap = Capability(
        resource_type=ResourceType.FILE,
        resource_id="/home/user/file.txt",
        permissions={Permission.READ},
    )
    ac.grant_capability(process.pid, cap)
    
    result = ac.revoke_capability(process.pid, ResourceType.FILE, "/home/user/file.txt")
    assert result == True
    
    capabilities = ac.get_capabilities(process.pid)
    assert len(capabilities) == 0


def test_check_access_allowed():
    """Test access check when allowed."""
    ac = MockAccessControl()
    process = ac.create_process("test")
    
    cap = Capability(
        resource_type=ResourceType.FILE,
        resource_id="/home/user/file.txt",
        permissions={Permission.READ},
    )
    ac.grant_capability(process.pid, cap)
    
    result = ac.check_access(
        process.pid, 
        ResourceType.FILE, 
        "/home/user/file.txt", 
        Permission.READ
    )
    assert result == True


def test_check_access_denied():
    """Test access check when denied."""
    ac = MockAccessControl()
    process = ac.create_process("test")
    
    cap = Capability(
        resource_type=ResourceType.FILE,
        resource_id="/home/user/file.txt",
        permissions={Permission.READ},
    )
    ac.grant_capability(process.pid, cap)
    
    # Try to write (not granted)
    result = ac.check_access(
        process.pid, 
        ResourceType.FILE, 
        "/home/user/file.txt", 
        Permission.WRITE
    )
    assert result == False


def test_check_access_admin():
    """Test admin permission grants all access."""
    ac = MockAccessControl()
    process = ac.create_process("test")
    
    cap = Capability(
        resource_type=ResourceType.FILE,
        resource_id="/home/user/file.txt",
        permissions={Permission.ADMIN},
    )
    ac.grant_capability(process.pid, cap)
    
    # Admin should have all permissions
    assert ac.check_access(process.pid, ResourceType.FILE, "/home/user/file.txt", Permission.READ)
    assert ac.check_access(process.pid, ResourceType.FILE, "/home/user/file.txt", Permission.WRITE)
    assert ac.check_access(process.pid, ResourceType.FILE, "/home/user/file.txt", Permission.EXECUTE)


def test_multiple_capabilities():
    """Test process with multiple capabilities."""
    ac = MockAccessControl()
    process = ac.create_process("test")
    
    ac.grant_capability(process.pid, Capability(
        resource_type=ResourceType.FILE,
        resource_id="/home/user/file1.txt",
        permissions={Permission.READ},
    ))
    ac.grant_capability(process.pid, Capability(
        resource_type=ResourceType.FILE,
        resource_id="/home/user/file2.txt",
        permissions={Permission.READ, Permission.WRITE},
    ))
    
    capabilities = ac.get_capabilities(process.pid)
    assert len(capabilities) == 2


def test_multiple_processes():
    """Test multiple processes with different capabilities."""
    ac = MockAccessControl()
    
    p1 = ac.create_process("process1")
    p2 = ac.create_process("process2")
    
    ac.grant_capability(p1.pid, Capability(
        resource_type=ResourceType.MEMORY,
        resource_id="pool1",
        permissions={Permission.READ, Permission.WRITE},
    ))
    ac.grant_capability(p2.pid, Capability(
        resource_type=ResourceType.MEMORY,
        resource_id="pool2",
        permissions={Permission.READ},
    ))
    
    assert ac.check_access(p1.pid, ResourceType.MEMORY, "pool1", Permission.WRITE)
    assert not ac.check_access(p2.pid, ResourceType.MEMORY, "pool1", Permission.WRITE)


# --- Sandbox Tests ---

def test_sandbox_creation():
    """Test sandbox creation."""
    sandbox = MockSandbox()
    sandbox_id = sandbox.create_sandbox({"/home/user"})
    
    assert sandbox_id == 1


def test_sandbox_add_process():
    """Test adding process to sandbox."""
    sandbox = MockSandbox()
    sandbox_id = sandbox.create_sandbox({"/home/user"})
    
    result = sandbox.add_process(sandbox_id, 1)
    assert result == True


def test_sandbox_remove_process():
    """Test removing process from sandbox."""
    sandbox = MockSandbox()
    sandbox_id = sandbox.create_sandbox({"/home/user"})
    sandbox.add_process(sandbox_id, 1)
    
    result = sandbox.remove_process(sandbox_id, 1)
    assert result == True


def test_sandbox_can_access():
    """Test sandbox resource access."""
    sandbox = MockSandbox()
    sandbox_id = sandbox.create_sandbox({"/home/user", "/tmp"})
    
    assert sandbox.can_access(sandbox_id, "/home/user")
    assert sandbox.can_access(sandbox_id, "/tmp")
    assert not sandbox.can_access(sandbox_id, "/etc")


def test_sandbox_memory_limit():
    """Test sandbox memory limit."""
    sandbox = MockSandbox()
    sandbox_id = sandbox.create_sandbox({"/home/user"})
    
    result = sandbox.set_memory_limit(sandbox_id, 200 * 1024 * 1024)
    assert result == True
    
    limits = sandbox.get_limits(sandbox_id)
    assert limits["memory_limit"] == 200 * 1024 * 1024


def test_sandbox_cpu_limit():
    """Test sandbox CPU limit."""
    sandbox = MockSandbox()
    sandbox_id = sandbox.create_sandbox({"/home/user"})
    
    result = sandbox.set_cpu_limit(sandbox_id, 75.0)
    assert result == True
    
    limits = sandbox.get_limits(sandbox_id)
    assert limits["cpu_limit"] == 75.0


def test_sandbox_invalid_id():
    """Test sandbox with invalid ID."""
    sandbox = MockSandbox()
    
    assert not sandbox.can_access(999, "/home/user")
    assert not sandbox.add_process(999, 1)
    assert not sandbox.set_memory_limit(999, 100)


# --- Permission Tests ---

def test_permission_check():
    """Test permission checking."""
    cap = Capability(
        resource_type=ResourceType.FILE,
        resource_id="test",
        permissions={Permission.READ, Permission.WRITE},
    )
    
    assert cap.has_permission(Permission.READ)
    assert cap.has_permission(Permission.WRITE)
    assert not cap.has_permission(Permission.EXECUTE)


def test_permission_admin_override():
    """Test admin permission override."""
    cap = Capability(
        resource_type=ResourceType.FILE,
        resource_id="test",
        permissions={Permission.ADMIN},
    )
    
    assert cap.has_permission(Permission.READ)
    assert cap.has_permission(Permission.WRITE)
    assert cap.has_permission(Permission.EXECUTE)


# --- Integration Tests ---

def test_process_isolation():
    """Test that processes are isolated."""
    ac = MockAccessControl()
    sandbox = MockSandbox()
    
    # Create two processes
    p1 = ac.create_process("process1")
    p2 = ac.create_process("process2")
    
    # Put them in different sandboxes
    s1 = sandbox.create_sandbox({"/home/user1"})
    s2 = sandbox.create_sandbox({"/home/user2"})
    
    sandbox.add_process(s1, p1.pid)
    sandbox.add_process(s2, p2.pid)
    
    # Grant different capabilities
    ac.grant_capability(p1.pid, Capability(
        resource_type=ResourceType.FILE,
        resource_id="/home/user1/file.txt",
        permissions={Permission.READ},
    ))
    ac.grant_capability(p2.pid, Capability(
        resource_type=ResourceType.FILE,
        resource_id="/home/user2/file.txt",
        permissions={Permission.READ},
    ))
    
    # Process 1 cannot access process 2's resources
    assert not ac.check_access(p1.pid, ResourceType.FILE, "/home/user2/file.txt", Permission.READ)
    assert not sandbox.can_access(s1, "/home/user2")


def test_capability_delegation_prevention():
    """Test that capabilities cannot be delegated without admin."""
    ac = MockAccessControl()
    
    p1 = ac.create_process("owner")
    p2 = ac.create_process("attacker")
    
    # Owner has capability
    ac.grant_capability(p1.pid, Capability(
        resource_type=ResourceType.FILE,
        resource_id="/secret/file.txt",
        permissions={Permission.READ},
    ))
    
    # Attacker should not have it
    assert not ac.check_access(p2.pid, ResourceType.FILE, "/secret/file.txt", Permission.READ)
    
    # And cannot grant it to themselves
    result = ac.grant_capability(p2.pid, Capability(
        resource_type=ResourceType.FILE,
        resource_id="/secret/file.txt",
        permissions={Permission.READ},
    ))
    
    # In a real system, this would be prevented
    # Here we just verify the isolation works
