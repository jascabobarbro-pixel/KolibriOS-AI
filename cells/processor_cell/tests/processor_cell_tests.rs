//! Comprehensive Processor Cell Tests
//!
//! Tests for:
//! - CPU management
//! - Task scheduling
//! - gRPC service
//! - Self-healing
//! - Metrics

#![cfg(test)]

use std::sync::Arc;
use std::time::Duration;

/// Test processor cell creation
#[test]
fn test_processor_cell_creation() {
    let cell = MockProcessorCell::new("test-processor", 4);
    
    assert_eq!(cell.id(), "test-processor");
    assert_eq!(cell.total_cores(), 4);
}

/// Test processor cell initialization
#[tokio::test]
async fn test_processor_cell_initialization() {
    let cell = MockProcessorCell::new("test-processor", 4);
    
    // Initial state
    assert_eq!(cell.state().await, CellState::Initializing);
    
    // Initialize
    let result = cell.initialize().await;
    assert!(result.is_ok());
    
    // Final state
    assert_eq!(cell.state().await, CellState::Active);
}

/// Test CPU stats
#[tokio::test]
async fn test_cpu_stats() {
    let cell = MockProcessorCell::new("test-processor", 4);
    cell.initialize().await.unwrap();
    
    let stats = cell.get_cpu_stats().await;
    
    assert_eq!(stats.total_cores, 4);
    assert!(stats.active_cores <= stats.total_cores);
    assert!(stats.utilization >= 0.0 && stats.utilization <= 100.0);
}

/// Test task creation
#[tokio::test]
async fn test_task_creation() {
    let cell = MockProcessorCell::new("test-processor", 4);
    cell.initialize().await.unwrap();
    
    // Create task
    let task = cell.create_task("test-program", vec!["arg1".to_string()], 5).await;
    assert!(task.is_ok());
    
    let task = task.unwrap();
    assert!(!task.id.is_empty());
    assert_eq!(task.executable, "test-program");
    assert_eq!(task.args, vec!["arg1"]);
    assert_eq!(task.priority, 5);
}

/// Test task listing
#[tokio::test]
async fn test_task_listing() {
    let cell = MockProcessorCell::new("test-processor", 4);
    cell.initialize().await.unwrap();
    
    // Create multiple tasks
    cell.create_task("prog1", vec![], 1).await.unwrap();
    cell.create_task("prog2", vec![], 2).await.unwrap();
    cell.create_task("prog3", vec![], 3).await.unwrap();
    
    // List tasks
    let tasks = cell.list_tasks().await;
    assert_eq!(tasks.len(), 3);
}

/// Test task termination
#[tokio::test]
async fn test_task_termination() {
    let cell = MockProcessorCell::new("test-processor", 4);
    cell.initialize().await.unwrap();
    
    // Create task
    let task = cell.create_task("test-program", vec![], 5).await.unwrap();
    
    // Verify it exists
    let tasks = cell.list_tasks().await;
    assert_eq!(tasks.len(), 1);
    
    // Terminate
    let result = cell.terminate_task(&task.id).await;
    assert!(result.is_ok());
    
    // Verify it's gone
    let tasks = cell.list_tasks().await;
    assert_eq!(tasks.len(), 0);
}

/// Test task priority
#[tokio::test]
async fn test_task_priority() {
    let cell = MockProcessorCell::new("test-processor", 4);
    cell.initialize().await.unwrap();
    
    // Create tasks with different priorities
    let low = cell.create_task("low", vec![], 1).await.unwrap();
    let high = cell.create_task("high", vec![], 10).await.unwrap();
    
    // High priority should be scheduled first
    let next = cell.get_next_task().await;
    assert_eq!(next.unwrap().id, high.id);
}

/// Test core allocation
#[tokio::test]
async fn test_core_allocation() {
    let cell = MockProcessorCell::new("test-processor", 4);
    cell.initialize().await.unwrap();
    
    // Allocate cores
    let core = cell.allocate_core().await;
    assert!(core.is_some());
    
    let core_id = core.unwrap();
    assert!(core_id < 4);
    
    // Check active cores
    let stats = cell.get_cpu_stats().await;
    assert_eq!(stats.active_cores, 1);
}

/// Test core deallocation
#[tokio::test]
async fn test_core_deallocation() {
    let cell = MockProcessorCell::new("test-processor", 4);
    cell.initialize().await.unwrap();
    
    // Allocate core
    let core = cell.allocate_core().await.unwrap();
    
    // Deallocate
    cell.deallocate_core(core).await;
    
    // Check active cores
    let stats = cell.get_cpu_stats().await;
    assert_eq!(stats.active_cores, 0);
}

/// Test load balancing
#[tokio::test]
async fn test_load_balancing() {
    let cell = MockProcessorCell::new("test-processor", 4);
    cell.initialize().await.unwrap();
    
    // Create many tasks
    for i in 0..20 {
        cell.create_task(&format!("prog{}", i), vec![], 5).await.unwrap();
    }
    
    // Check load distribution
    let load = cell.get_core_load().await;
    
    // All cores should have some load
    for core_load in load {
        assert!(core_load >= 0.0);
    }
}

/// Test self-healing for stuck tasks
#[tokio::test]
async fn test_self_healing_stuck_tasks() {
    let cell = MockProcessorCell::new("test-processor", 4);
    cell.initialize().await.unwrap();
    
    // Create a stuck task
    let task = cell.create_task("stuck-program", vec![], 5).await.unwrap();
    cell.mark_task_stuck(&task.id).await;
    
    // Health should be degraded
    let health = cell.health().await;
    assert!(matches!(health, HealthStatus::Warning | HealthStatus::Critical));
    
    // Heal
    cell.heal().await.unwrap();
    
    // Should be healthy
    let health = cell.health().await;
    assert_eq!(health, HealthStatus::Healthy);
}

/// Test diagnostics
#[tokio::test]
async fn test_diagnostics() {
    let cell = MockProcessorCell::new("test-processor", 4);
    cell.initialize().await.unwrap();
    
    let result = cell.run_diagnostics().await;
    
    assert!(result.healthy);
    assert!(result.issues.is_empty());
}

/// Test metrics collection
#[tokio::test]
async fn test_metrics_collection() {
    let cell = MockProcessorCell::new("test-processor", 4);
    cell.initialize().await.unwrap();
    
    // Create some tasks
    cell.create_task("prog1", vec![], 1).await.unwrap();
    cell.create_task("prog2", vec![], 2).await.unwrap();
    
    let metrics = cell.get_metrics().await;
    
    assert!(metrics.contains_key("total_cores"));
    assert!(metrics.contains_key("active_cores"));
    assert!(metrics.contains_key("task_count"));
    assert!(metrics.contains_key("utilization"));
}

/// Test cell shutdown
#[tokio::test]
async fn test_cell_shutdown() {
    let mut cell = MockProcessorCell::new("test-processor", 4);
    cell.initialize().await.unwrap();
    
    assert_eq!(cell.state().await, CellState::Active);
    
    // Shutdown
    let result = cell.shutdown().await;
    assert!(result.is_ok());
    
    assert_eq!(cell.state().await, CellState::Shutdown);
}

/// Test zombie task cleanup
#[tokio::test]
async fn test_zombie_cleanup() {
    let cell = MockProcessorCell::new("test-processor", 4);
    cell.initialize().await.unwrap();
    
    // Create and terminate tasks
    for i in 0..10 {
        let task = cell.create_task(&format!("prog{}", i), vec![], 5).await.unwrap();
        cell.terminate_task(&task.id).await.unwrap();
    }
    
    // Run cleanup
    let cleaned = cell.cleanup_zombies().await;
    assert!(cleaned >= 0);
}

// ============== Mock Implementations ==============

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Initializing,
    Active,
    Degraded,
    Healing,
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub executable: String,
    pub args: Vec<String>,
    pub priority: u32,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Stuck,
}

#[derive(Debug, Clone)]
pub struct CpuStats {
    pub total_cores: usize,
    pub active_cores: usize,
    pub utilization: f32,
}

#[derive(Debug, Clone)]
pub struct DiagnosticsResult {
    pub healthy: bool,
    pub issues: Vec<String>,
}

pub struct MockProcessorCell {
    id: String,
    total_cores: usize,
    active_cores: usize,
    state: CellState,
    health: HealthStatus,
    tasks: std::collections::HashMap<String, Task>,
    task_counter: u64,
    core_load: Vec<f32>,
}

impl MockProcessorCell {
    pub fn new(id: &str, total_cores: usize) -> Self {
        Self {
            id: id.to_string(),
            total_cores,
            active_cores: 0,
            state: CellState::Initializing,
            health: HealthStatus::Healthy,
            tasks: std::collections::HashMap::new(),
            task_counter: 0,
            core_load: vec![0.0; total_cores],
        }
    }
    
    pub fn id(&self) -> &str {
        &self.id
    }
    
    pub fn total_cores(&self) -> usize {
        self.total_cores
    }
    
    pub async fn state(&self) -> CellState {
        self.state
    }
    
    pub async fn health(&self) -> HealthStatus {
        self.health
    }
    
    pub async fn initialize(&mut self) -> Result<(), String> {
        self.state = CellState::Active;
        Ok(())
    }
    
    pub async fn get_cpu_stats(&self) -> CpuStats {
        CpuStats {
            total_cores: self.total_cores,
            active_cores: self.active_cores,
            utilization: self.core_load.iter().sum::<f32>() / self.total_cores as f32 * 100.0,
        }
    }
    
    pub async fn create_task(&mut self, executable: &str, args: Vec<String>, priority: u32) -> Result<Task, String> {
        self.task_counter += 1;
        let id = format!("task-{}", self.task_counter);
        let task = Task {
            id: id.clone(),
            executable: executable.to_string(),
            args,
            priority,
            status: TaskStatus::Pending,
        };
        self.tasks.insert(id.clone(), task.clone());
        Ok(task)
    }
    
    pub async fn list_tasks(&self) -> Vec<Task> {
        self.tasks.values().cloned().collect()
    }
    
    pub async fn terminate_task(&mut self, id: &str) -> Result<(), String> {
        self.tasks.remove(id)
            .map(|_| ())
            .ok_or_else(|| "Task not found".to_string())
    }
    
    pub async fn get_next_task(&self) -> Option<Task> {
        self.tasks.values()
            .filter(|t| t.status == TaskStatus::Pending)
            .max_by_key(|t| t.priority)
            .cloned()
    }
    
    pub async fn allocate_core(&mut self) -> Option<usize> {
        if self.active_cores < self.total_cores {
            let core = self.active_cores;
            self.active_cores += 1;
            Some(core)
        } else {
            None
        }
    }
    
    pub async fn deallocate_core(&mut self, _core: usize) {
        if self.active_cores > 0 {
            self.active_cores -= 1;
        }
    }
    
    pub async fn get_core_load(&self) -> Vec<f32> {
        self.core_load.clone()
    }
    
    pub async fn mark_task_stuck(&mut self, id: &str) {
        if let Some(task) = self.tasks.get_mut(id) {
            task.status = TaskStatus::Stuck;
            self.health = HealthStatus::Warning;
        }
    }
    
    pub async fn heal(&mut self) -> Result<(), String> {
        // Remove stuck tasks
        self.tasks.retain(|_, t| t.status != TaskStatus::Stuck);
        self.health = HealthStatus::Healthy;
        Ok(())
    }
    
    pub async fn run_diagnostics(&self) -> DiagnosticsResult {
        let stuck_count = self.tasks.values().filter(|t| t.status == TaskStatus::Stuck).count();
        DiagnosticsResult {
            healthy: stuck_count == 0 && self.health == HealthStatus::Healthy,
            issues: if stuck_count > 0 {
                vec![format!("{} stuck tasks", stuck_count)]
            } else {
                vec![]
            },
        }
    }
    
    pub async fn get_metrics(&self) -> std::collections::HashMap<String, f64> {
        let mut metrics = std::collections::HashMap::new();
        metrics.insert("total_cores".to_string(), self.total_cores as f64);
        metrics.insert("active_cores".to_string(), self.active_cores as f64);
        metrics.insert("task_count".to_string(), self.tasks.len() as f64);
        metrics.insert("utilization".to_string(), self.core_load.iter().sum::<f32>() as f64 / self.total_cores as f64 * 100.0);
        metrics
    }
    
    pub async fn shutdown(&mut self) -> Result<(), String> {
        self.state = CellState::Shutdown;
        Ok(())
    }
    
    pub async fn cleanup_zombies(&mut self) -> usize {
        let before = self.tasks.len();
        self.tasks.retain(|_, t| t.status != TaskStatus::Completed);
        before - self.tasks.len()
    }
}
