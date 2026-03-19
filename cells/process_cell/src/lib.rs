//! Process Cell - Autonomous Process Management
//!
//! Provides intelligent process lifecycle management with
//! predictive scheduling and automatic resource optimization.

#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// Process Cell - The autonomous process management entity
pub struct ProcessCell {
    id: CellId,
    state: CellState,
    processes: BTreeMap<ProcessId, ManagedProcess>,
    process_tree: ProcessTree,
    resource_manager: ResourceManager,
}

impl ProcessCell {
    /// Create a new process cell
    pub fn new() -> Self {
        Self {
            id: CellId::new(),
            state: CellState::Initializing,
            processes: BTreeMap::new(),
            process_tree: ProcessTree::new(),
            resource_manager: ResourceManager::new(),
        }
    }

    /// Initialize the process cell
    pub fn init(&mut self) -> Result<(), ProcessError> {
        self.state = CellState::Active;
        Ok(())
    }

    /// Spawn a new process
    pub fn spawn(&mut self, name: &str, parent: Option<ProcessId>) -> Result<ProcessId, ProcessError> {
        let id = ProcessId::new();
        let process = ManagedProcess {
            id,
            name: String::from(name),
            state: ProcessState::Ready,
            parent,
            children: Vec::new(),
            resources: ResourceSet::default(),
            priority: ProcessPriority::Normal,
        };
        
        // Add to parent's children
        if let Some(pid) = parent {
            if let Some(parent_proc) = self.processes.get_mut(&pid) {
                parent_proc.children.push(id);
            }
        }
        
        self.processes.insert(id, process);
        self.process_tree.add_process(id, parent);
        Ok(id)
    }

    /// Terminate a process
    pub fn terminate(&mut self, pid: ProcessId) -> Result<(), ProcessError> {
        if let Some(process) = self.processes.get_mut(&pid) {
            process.state = ProcessState::Terminated;
            
            // Terminate children recursively
            let children: Vec<_> = process.children.iter().cloned().collect();
            for child in children {
                let _ = self.terminate(child);
            }
        }
        Ok(())
    }

    /// Get process by ID
    pub fn get_process(&self, pid: ProcessId) -> Option<&ManagedProcess> {
        self.processes.get(&pid)
    }

    /// List all processes
    pub fn list_processes(&self) -> Vec<&ManagedProcess> {
        self.processes.values().collect()
    }
}

impl Default for ProcessCell {
    fn default() -> Self {
        Self::new()
    }
}

/// Identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProcessId(u64);

impl CellId {
    fn new() -> Self { use core::sync::atomic::{AtomicU64, Ordering}; static NEXT: AtomicU64 = AtomicU64::new(1); Self(NEXT.fetch_add(1, Ordering::SeqCst)) }
}

impl ProcessId {
    fn new() -> Self { use core::sync::atomic::{AtomicU64, Ordering}; static NEXT: AtomicU64 = AtomicU64::new(1); Self(NEXT.fetch_add(1, Ordering::SeqCst)) }
}

/// Cell state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Initializing,
    Active,
    Degraded,
    Shutdown,
}

/// Managed process
pub struct ManagedProcess {
    pub id: ProcessId,
    pub name: String,
    pub state: ProcessState,
    pub parent: Option<ProcessId>,
    pub children: Vec<ProcessId>,
    pub resources: ResourceSet,
    pub priority: ProcessPriority,
}

/// Process state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    New,
    Ready,
    Running,
    Blocked,
    Suspended,
    Terminated,
}

/// Process priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessPriority {
    Idle,
    Low,
    Normal,
    High,
    RealTime,
}

/// Resource set
#[derive(Debug, Clone, Default)]
pub struct ResourceSet {
    pub cpu_time: u64,
    pub memory_used: usize,
    pub io_operations: u64,
    pub file_descriptors: u32,
}

/// Process tree
pub struct ProcessTree {
    nodes: BTreeMap<ProcessId, ProcessNode>,
}

impl ProcessTree {
    fn new() -> Self {
        Self { nodes: BTreeMap::new() }
    }

    fn add_process(&mut self, pid: ProcessId, parent: Option<ProcessId>) {
        self.nodes.insert(pid, ProcessNode { pid, parent });
    }
}

/// Process tree node
pub struct ProcessNode {
    pub pid: ProcessId,
    pub parent: Option<ProcessId>,
}

/// Resource manager
pub struct ResourceManager {
    total_cpu_time: u64,
    total_memory: usize,
}

impl ResourceManager {
    fn new() -> Self {
        Self {
            total_cpu_time: 0,
            total_memory: 0,
        }
    }
}

/// Process error
#[derive(Debug, Clone, thiserror::Error)]
pub enum ProcessError {
    #[error("Process not found")]
    ProcessNotFound,
    #[error("Process already terminated")]
    AlreadyTerminated,
    #[error("Resource limit exceeded")]
    ResourceLimitExceeded,
    #[error("Permission denied")]
    PermissionDenied,
}
