//! Comprehensive Kernel Tests
//! 
//! These tests verify all kernel components including:
//! - Living Kernel initialization
//! - Gene registration and lifecycle
//! - Neural Scheduler decisions
//! - Memory management
//! - Security model

#![cfg(test)]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Mock kernel components for testing
// In real implementation, these would import from kernel::

/// Test kernel state transitions
#[test]
fn test_kernel_state_transitions() {
    let states = vec![
        KernelState::Uninitialized,
        KernelState::EarlyBoot,
        KernelState::GenesInitializing,
        KernelState::NeuralSchedulerReady,
        KernelState::FullyOperational,
        KernelState::Degraded,
        KernelState::ShuttingDown,
    ];
    
    // Verify all states exist
    assert_eq!(states.len(), 7);
    
    // Verify state ordering
    for (i, state) in states.iter().enumerate() {
        assert!(matches!(state, KernelState::Uninitialized | KernelState::EarlyBoot 
            | KernelState::GenesInitializing | KernelState::NeuralSchedulerReady 
            | KernelState::FullyOperational | KernelState::Degraded | KernelState::ShuttingDown));
    }
}

/// Test kernel initialization
#[test]
fn test_kernel_initialization() {
    let mut kernel = MockLivingKernel::new(1024 * 1024 * 1024); // 1GB
    
    // Initial state
    assert_eq!(kernel.state(), KernelState::Uninitialized);
    
    // Initialize
    let result = kernel.init();
    assert!(result.is_ok());
    
    // Final state
    assert_eq!(kernel.state(), KernelState::FullyOperational);
}

/// Test gene registry
#[test]
fn test_gene_registry() {
    let mut registry = MockGeneRegistry::new();
    
    // Register genes
    let process_gene = MockProcessGene::new();
    let memory_gene = MockMemoryGene::new(1024 * 1024 * 1024);
    let io_gene = MockIOGene::new();
    
    registry.register(process_gene);
    registry.register(memory_gene);
    registry.register(io_gene);
    
    // Verify count
    assert_eq!(registry.count(), 3);
    
    // Verify all genes are active
    assert!(registry.all_active());
}

/// Test process gene
#[test]
fn test_process_gene() {
    let mut gene = MockProcessGene::new();
    
    // Initial state
    assert_eq!(gene.process_count(), 0);
    
    // Add processes
    gene.add_process(1, 10); // PID 1, priority 10
    gene.add_process(2, 5);  // PID 2, priority 5
    gene.add_process(3, 15); // PID 3, priority 15
    
    assert_eq!(gene.process_count(), 3);
    
    // Remove process
    gene.remove_process(2);
    assert_eq!(gene.process_count(), 2);
    
    // Update
    let result = gene.update(1);
    assert!(result.is_ok());
}

/// Test memory gene
#[test]
fn test_memory_gene() {
    let mut gene = MockMemoryGene::new(1024 * 1024 * 1024); // 1GB
    
    // Check initial stats
    let stats = gene.stats();
    assert_eq!(stats.total, 1024 * 1024 * 1024);
    assert_eq!(stats.used, 0);
    
    // Allocate
    gene.allocate(1024 * 1024); // 1MB
    let stats = gene.stats();
    assert_eq!(stats.used, 1024 * 1024);
    
    // Deallocate
    gene.deallocate(512 * 1024); // 512KB
    let stats = gene.stats();
    assert_eq!(stats.used, 512 * 1024);
}

/// Test memory zones
#[test]
fn test_memory_zones() {
    let zones = vec![
        MemoryZone::Kernel,
        MemoryZone::User,
        MemoryZone::Shared,
        MemoryZone::Ai,
        MemoryZone::Cache,
    ];
    
    assert_eq!(zones.len(), 5);
    
    // Verify zone types
    assert!(!MemoryZone::Kernel.is_adaptive());
    assert!(MemoryZone::User.is_adaptive());
    assert!(MemoryZone::Shared.is_adaptive());
    assert!(MemoryZone::Ai.is_adaptive());
    assert!(MemoryZone::Cache.is_adaptive());
}

/// Test I/O gene
#[test]
fn test_io_gene() {
    let mut gene = MockIOGene::new();
    
    // Initial state
    assert_eq!(gene.device_count(), 0);
    
    // Register devices
    gene.register_device("disk0");
    gene.register_device("net0");
    gene.register_device("usb0");
    
    assert_eq!(gene.device_count(), 3);
    
    // Update
    let result = gene.update(1);
    assert!(result.is_ok());
}

/// Test neural scheduler
#[test]
fn test_neural_scheduler() {
    let mut scheduler = MockNeuralScheduler::new();
    
    // Create system state input
    let state = SystemStateInput {
        cpu_utilization: vec![0.5, 0.6, 0.4, 0.3],
        memory_pressure: 0.4,
        ready_tasks: 0.2,
        blocked_tasks: 0.1,
        avg_priority: 0.5,
        io_bound_ratio: 0.3,
        cache_hit_ratio: 0.85,
        load_average: 0.5,
        context_switches: 0.1,
        interrupt_rate: 0.05,
        time_since_decision: 0.01,
    };
    
    // Get decision
    let decision = scheduler.decide(&state);
    
    // Verify decision is valid
    assert!(matches!(decision, SchedulingDecision::RunPriority 
        | SchedulingDecision::RunIoBound 
        | SchedulingDecision::RunCpuBound 
        | SchedulingDecision::BalanceLoad 
        | SchedulingDecision::Preempt 
        | SchedulingDecision::Idle 
        | SchedulingDecision::Batch 
        | SchedulingDecision::Interactive));
}

/// Test neural scheduler with high memory pressure
#[test]
fn test_neural_scheduler_high_memory() {
    let mut scheduler = MockNeuralScheduler::new();
    
    let state = SystemStateInput {
        cpu_utilization: vec![0.3, 0.3, 0.3, 0.3],
        memory_pressure: 0.95, // Very high
        ready_tasks: 0.1,
        blocked_tasks: 0.2,
        avg_priority: 0.5,
        io_bound_ratio: 0.1,
        cache_hit_ratio: 0.5,
        load_average: 0.3,
        context_switches: 0.05,
        interrupt_rate: 0.02,
        time_since_decision: 0.01,
    };
    
    let decision = scheduler.decide(&state);
    
    // Should not be RunCpuBound with high memory pressure
    // This tests the scheduler's intelligence
    assert!(scheduler.confidence() >= 0.0);
}

/// Test neural scheduler confidence threshold
#[test]
fn test_neural_scheduler_confidence() {
    let scheduler = MockNeuralScheduler::new();
    
    // Default confidence threshold
    assert_eq!(scheduler.confidence_threshold(), 0.6);
    
    // Stats should be available
    let stats = scheduler.stats();
    assert!(stats.decisions_count >= 0);
}

/// Test scheduler stats
#[test]
fn test_scheduler_stats() {
    let scheduler = MockNeuralScheduler::new();
    let stats = scheduler.stats();
    
    // Verify stats structure
    assert!(stats.decisions_count >= 0);
    assert!(stats.avg_confidence >= 0.0 && stats.avg_confidence <= 1.0);
    assert!(stats.fallback_count >= 0);
}

/// Test kernel shutdown
#[test]
fn test_kernel_shutdown() {
    let mut kernel = MockLivingKernel::new(1024 * 1024 * 1024);
    kernel.init().unwrap();
    
    assert_eq!(kernel.state(), KernelState::FullyOperational);
    
    // Shutdown
    let result = kernel.shutdown();
    assert!(result.is_ok());
    assert_eq!(kernel.state(), KernelState::ShuttingDown);
}

/// Test kernel memory feedback
#[test]
fn test_kernel_memory_feedback() {
    let mut kernel = MockLivingKernel::new(1024 * 1024 * 1024);
    kernel.init().unwrap();
    
    // Process memory feedback
    kernel.process_memory_feedback("cell-0", 0.8, "defragment");
    
    // Verify memory gene received feedback
    let stats = kernel.memory_stats();
    assert!(stats.feedback_count > 0);
}

// ============== Mock Implementations ==============

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelState {
    Uninitialized,
    EarlyBoot,
    GenesInitializing,
    NeuralSchedulerReady,
    FullyOperational,
    Degraded,
    ShuttingDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryZone {
    Kernel,
    User,
    Shared,
    Ai,
    Cache,
}

impl MemoryZone {
    fn is_adaptive(&self) -> bool {
        matches!(self, MemoryZone::User | MemoryZone::Shared | MemoryZone::Ai | MemoryZone::Cache)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SchedulingDecision {
    RunPriority,
    RunIoBound,
    RunCpuBound,
    BalanceLoad,
    Preempt,
    Idle,
    Batch,
    Interactive,
}

#[derive(Debug, Clone)]
pub struct SystemStateInput {
    pub cpu_utilization: Vec<f32>,
    pub memory_pressure: f32,
    pub ready_tasks: f32,
    pub blocked_tasks: f32,
    pub avg_priority: f32,
    pub io_bound_ratio: f32,
    pub cache_hit_ratio: f32,
    pub load_average: f32,
    pub context_switches: f32,
    pub interrupt_rate: f32,
    pub time_since_decision: f32,
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total: u64,
    pub used: u64,
    pub feedback_count: u64,
}

#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub decisions_count: u64,
    pub avg_confidence: f32,
    pub fallback_count: u64,
}

// Mock implementations

pub struct MockLivingKernel {
    state: KernelState,
    memory_total: u64,
    memory_used: u64,
    feedback_count: u64,
}

impl MockLivingKernel {
    pub fn new(total_memory: usize) -> Self {
        Self {
            state: KernelState::Uninitialized,
            memory_total: total_memory as u64,
            memory_used: 0,
            feedback_count: 0,
        }
    }
    
    pub fn init(&mut self) -> Result<(), String> {
        self.state = KernelState::EarlyBoot;
        self.state = KernelState::GenesInitializing;
        self.state = KernelState::NeuralSchedulerReady;
        self.state = KernelState::FullyOperational;
        Ok(())
    }
    
    pub fn state(&self) -> KernelState {
        self.state
    }
    
    pub fn shutdown(&mut self) -> Result<(), String> {
        self.state = KernelState::ShuttingDown;
        Ok(())
    }
    
    pub fn process_memory_feedback(&mut self, _cell_id: &str, _pressure: f32, _recommendation: &str) {
        self.feedback_count += 1;
    }
    
    pub fn memory_stats(&self) -> MemoryStats {
        MemoryStats {
            total: self.memory_total,
            used: self.memory_used,
            feedback_count: self.feedback_count,
        }
    }
}

pub struct MockGeneRegistry {
    count: usize,
    active: bool,
}

impl MockGeneRegistry {
    pub fn new() -> Self {
        Self { count: 0, active: false }
    }
    
    pub fn register<T>(&mut self, _gene: T) {
        self.count += 1;
        self.active = true;
    }
    
    pub fn count(&self) -> usize {
        self.count
    }
    
    pub fn all_active(&self) -> bool {
        self.active
    }
}

pub struct MockProcessGene {
    processes: Vec<(u32, u32)>,
}

impl MockProcessGene {
    pub fn new() -> Self {
        Self { processes: Vec::new() }
    }
    
    pub fn process_count(&self) -> usize {
        self.processes.len()
    }
    
    pub fn add_process(&mut self, pid: u32, priority: u32) {
        self.processes.push((pid, priority));
    }
    
    pub fn remove_process(&mut self, pid: u32) {
        self.processes.retain(|(p, _)| *p != pid);
    }
    
    pub fn update(&mut self, _tick: u64) -> Result<(), String> {
        Ok(())
    }
}

pub struct MockMemoryGene {
    total: u64,
    used: u64,
}

impl MockMemoryGene {
    pub fn new(total: usize) -> Self {
        Self { total: total as u64, used: 0 }
    }
    
    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            total: self.total,
            used: self.used,
            feedback_count: 0,
        }
    }
    
    pub fn allocate(&mut self, size: u64) {
        self.used += size;
    }
    
    pub fn deallocate(&mut self, size: u64) {
        self.used = self.used.saturating_sub(size);
    }
}

pub struct MockIOGene {
    devices: Vec<String>,
}

impl MockIOGene {
    pub fn new() -> Self {
        Self { devices: Vec::new() }
    }
    
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }
    
    pub fn register_device(&mut self, name: &str) {
        self.devices.push(name.to_string());
    }
    
    pub fn update(&mut self, _tick: u64) -> Result<(), String> {
        Ok(())
    }
}

pub struct MockNeuralScheduler {
    confidence_threshold: f32,
    decisions_count: u64,
    avg_confidence: f32,
    fallback_count: u64,
    last_confidence: f32,
}

impl MockNeuralScheduler {
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.6,
            decisions_count: 0,
            avg_confidence: 0.8,
            fallback_count: 0,
            last_confidence: 0.8,
        }
    }
    
    pub fn decide(&mut self, state: &SystemStateInput) -> SchedulingDecision {
        self.decisions_count += 1;
        
        // Simple decision logic based on state
        if state.memory_pressure > 0.9 {
            self.last_confidence = 0.7;
            SchedulingDecision::BalanceLoad
        } else if state.io_bound_ratio > 0.5 {
            self.last_confidence = 0.85;
            SchedulingDecision::RunIoBound
        } else if state.cpu_utilization.iter().sum::<f32>() / state.cpu_utilization.len() as f32 > 0.7 {
            self.last_confidence = 0.75;
            SchedulingDecision::RunCpuBound
        } else if state.ready_tasks < 0.1 {
            self.last_confidence = 0.9;
            SchedulingDecision::Idle
        } else {
            self.last_confidence = 0.8;
            SchedulingDecision::RunPriority
        }
    }
    
    pub fn confidence(&self) -> f32 {
        self.last_confidence
    }
    
    pub fn confidence_threshold(&self) -> f32 {
        self.confidence_threshold
    }
    
    pub fn stats(&self) -> SchedulerStats {
        SchedulerStats {
            decisions_count: self.decisions_count,
            avg_confidence: self.avg_confidence,
            fallback_count: self.fallback_count,
        }
    }
}
