//! Process Gene - Manages process lifecycle

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::gene_trait::Gene;
use super::{GeneActivation, GeneDNA, GeneError, GeneId, GeneRNA, GeneValue};

/// Process Gene - Handles process management
pub struct ProcessGene {
    id: GeneId,
    dna: GeneDNA,
    rna: GeneRNA,
    processes: BTreeMap<u32, ProcessInfo>,
    next_pid: u32,
}

/// Process information
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub state: ProcessState,
    pub priority: u8,
    pub cpu_time: u64,
    pub memory_used: usize,
    pub parent: Option<u32>,
    pub children: Vec<u32>,
}

/// Process state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Creating,
    Ready,
    Running,
    Blocked,
    Zombie,
    Terminated,
}

/// Scheduling hint from the gene
#[derive(Debug, Clone)]
pub struct SchedulingHint {
    pub next_pid: Option<u32>,
    pub time_quantum_ms: u64,
    pub core_affinity: Option<u32>,
    pub should_preempt: bool,
}

impl ProcessGene {
    /// Create a new ProcessGene
    pub fn new() -> Self {
        let mut config = BTreeMap::new();
        config.insert(String::from("max_processes"), GeneValue::Integer(4096));
        config.insert(String::from("default_quantum_ms"), GeneValue::Integer(10));
        config.insert(String::from("enable_preemption"), GeneValue::Boolean(true));

        Self {
            id: GeneId::new(),
            dna: GeneDNA {
                name: String::from("process"),
                version: 1,
                config,
                activation_threshold: 0.3,
                critical: true,
            },
            rna: GeneRNA::default(),
            processes: BTreeMap::new(),
            next_pid: 1,
        }
    }

    /// Create a new process
    pub fn create_process(&mut self, name: &str, priority: u8, parent: Option<u32>) -> Result<u32, GeneError> {
        let max = self.dna.config.get("max_processes")
            .and_then(|v| v.as_integer()).unwrap_or(4096) as usize;

        if self.processes.len() >= max {
            return Err(GeneError::ResourceUnavailable(String::from("Maximum process limit reached")));
        }

        let pid = self.next_pid;
        self.next_pid += 1;

        let process = ProcessInfo {
            pid, name: String::from(name), state: ProcessState::Creating,
            priority, cpu_time: 0, memory_used: 0, parent, children: Vec::new(),
        };

        if let Some(parent_pid) = parent {
            if let Some(parent_proc) = self.processes.get_mut(&parent_pid) {
                parent_proc.children.push(pid);
            }
        }

        self.processes.insert(pid, process);
        self.rna.activation_count += 1;
        Ok(pid)
    }

    /// Terminate a process
    pub fn terminate_process(&mut self, pid: u32, force: bool) -> Result<(), GeneError> {
        if let Some(mut process) = self.processes.remove(&pid) {
            if process.state == ProcessState::Running && !force {
                return Err(GeneError::ActivationFailed(
                    String::from("Cannot terminate running process without force")));
            }
            process.state = ProcessState::Terminated;
            if let Some(parent_pid) = process.parent {
                if let Some(parent) = self.processes.get_mut(&parent_pid) {
                    parent.children.retain(|&c| c != pid);
                }
            }
            for child_pid in process.children.clone() {
                let _ = self.terminate_process(child_pid, true);
            }
            Ok(())
        } else {
            Err(GeneError::NotFound(format!("Process {}", pid)))
        }
    }

    /// Get process by PID
    pub fn get_process(&self, pid: u32) -> Option<&ProcessInfo> { self.processes.get(&pid) }

    /// Get scheduling hint
    pub fn get_scheduling_hint(&mut self) -> SchedulingHint {
        let quantum = self.dna.config.get("default_quantum_ms")
            .and_then(|v| v.as_integer()).unwrap_or(10) as u64;
        let next = self.processes.values()
            .filter(|p| p.state == ProcessState::Ready)
            .max_by_key(|p| p.priority).map(|p| p.pid);
        SchedulingHint {
            next_pid: next, time_quantum_ms: quantum,
            core_affinity: None,
            should_preempt: self.dna.config.get("enable_preemption")
                .and_then(|v| v.as_bool()).unwrap_or(true),
        }
    }

    /// Set process state
    pub fn set_state(&mut self, pid: u32, state: ProcessState) -> Result<(), GeneError> {
        if let Some(process) = self.processes.get_mut(&pid) {
            process.state = state;
            Ok(())
        } else {
            Err(GeneError::NotFound(format!("Process {}", pid)))
        }
    }

    /// List all processes
    pub fn list_processes(&self) -> Vec<&ProcessInfo> { self.processes.values().collect() }

    /// Get process count
    pub fn process_count(&self) -> usize { self.processes.len() }
}

impl Default for ProcessGene {
    fn default() -> Self { Self::new() }
}

impl Gene for ProcessGene {
    fn id(&self) -> GeneId { self.id }
    fn name(&self) -> &str { &self.dna.name }
    fn dna(&self) -> &GeneDNA { &self.dna }
    fn dna_mut(&mut self) -> &mut GeneDNA { &mut self.dna }
    fn rna(&self) -> &GeneRNA { &self.rna }
    fn rna_mut(&mut self) -> &mut GeneRNA { &mut self.rna }

    fn activate(&mut self, input: Option<&GeneValue>) -> Result<GeneActivation, GeneError> {
        self.rna.activity = 1.0;
        self.rna.activation_count += 1;
        let result = input.and_then(|v| v.as_string()).map(|s| {
            match s.as_str() {
                "hint" => {
                    let hint = self.get_scheduling_hint();
                    GeneValue::String(alloc::format!("next:{:?} quantum:{}ms", hint.next_pid, hint.time_quantum_ms))
                }
                "list" => {
                    let procs: Vec<String> = self.processes.values()
                        .map(|p| format!("{}(pid:{})", p.name, p.pid)).collect();
                    GeneValue::String(procs.join(","))
                }
                _ => GeneValue::String(String::from("unknown command")),
            }
        });
        Ok(GeneActivation { activated: true, result, effects: Vec::new() })
    }

    fn update(&mut self, _delta_ms: u64) -> Result<(), GeneError> {
        self.rna.activity *= 0.95;
        Ok(())
    }
}
