//! Processor Cell - CPU Core Management
//!
//! Provides intelligent CPU core management, task scheduling,
//! and performance monitoring with Prometheus metrics.

pub mod cpu;
pub mod task;
pub mod metrics;
pub mod grpc;
pub mod diagnostics;

use std::sync::Arc;
use tokio::sync::RwLock;
use cpu::CpuManager;
use task::TaskManager;
use metrics::ProcessorMetrics;
use thiserror::Error;

/// Processor Cell error types
#[derive(Debug, Error)]
pub enum ProcessorCellError {
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Core not found: {0}")]
    CoreNotFound(u32),

    #[error("Invalid core state: {0}")]
    InvalidCoreState(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Cell state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Initializing,
    Active,
    Degraded,
    Healing,
    Shutdown,
}

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

/// Processor Cell - Main structure
pub struct ProcessorCell {
    id: String,
    state: Arc<RwLock<CellState>>,
    health: Arc<RwLock<HealthStatus>>,
    cpu_manager: Arc<RwLock<CpuManager>>,
    task_manager: Arc<RwLock<TaskManager>>,
    metrics: ProcessorMetrics,
}

impl ProcessorCell {
    /// Create a new Processor Cell
    pub fn new(id: &str) -> Self {
        let core_count = num_cpus::get() as u32;
        Self {
            id: id.to_string(),
            state: Arc::new(RwLock::new(CellState::Initializing)),
            health: Arc::new(RwLock::new(HealthStatus::Healthy)),
            cpu_manager: Arc::new(RwLock::new(CpuManager::new(core_count))),
            task_manager: Arc::new(RwLock::new(TaskManager::new())),
            metrics: ProcessorMetrics::new(id),
        }
    }

    /// Initialize the processor cell
    pub async fn initialize(&self) -> Result<(), ProcessorCellError> {
        let mut state = self.state.write().await;

        // Initialize CPU cores
        let mut cpu_manager = self.cpu_manager.write().await;
        cpu_manager.initialize()?;

        *state = CellState::Active;
        self.metrics.record_state(CellState::Active);
        log::info!(
            "Processor Cell {} initialized with {} cores",
            self.id,
            cpu_manager.core_count()
        );

        Ok(())
    }

    /// Get cell ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get current state
    pub async fn state(&self) -> CellState {
        *self.state.read().await
    }

    /// Get health status
    pub async fn health(&self) -> HealthStatus {
        *self.health.read().await
    }

    /// Execute a task
    pub async fn execute_task(
        &self,
        executable: &str,
        args: Vec<String>,
        priority: task::TaskPriority,
    ) -> Result<task::Task, ProcessorCellError> {
        let mut task_manager = self.task_manager.write().await;
        let cpu_manager = self.cpu_manager.read().await;

        // Find available core
        let core_id = cpu_manager.find_available_core();

        let task = task_manager.create_task(executable, args, priority, core_id)?;

        // Update metrics
        self.metrics.record_task_started(&task);

        log::info!(
            "Task {} started on core {}",
            task.id,
            core_id.map(|c| c.to_string()).unwrap_or_else(|| "any".to_string())
        );

        Ok(task)
    }

    /// Cancel a task
    pub async fn cancel_task(&self, task_id: &str, force: bool) -> Result<(), ProcessorCellError> {
        let mut task_manager = self.task_manager.write().await;

        task_manager.cancel_task(task_id, force)?;

        self.metrics.record_task_cancelled();

        log::info!("Task {} cancelled (force: {})", task_id, force);

        Ok(())
    }

    /// Get CPU statistics
    pub async fn get_cpu_stats(&self) -> cpu::CpuStats {
        let manager = self.cpu_manager.read().await;
        manager.get_stats()
    }

    /// Get task status
    pub async fn get_task_status(&self, task_id: &str) -> Option<task::TaskStatus> {
        let manager = self.task_manager.read().await;
        manager.get_task_status(task_id)
    }

    /// List all tasks
    pub async fn list_tasks(&self) -> Vec<task::Task> {
        let manager = self.task_manager.read().await;
        manager.list_tasks()
    }

    /// Run diagnostics
    pub async fn run_diagnostics(&self) -> diagnostics::DiagnosticsResult {
        let cpu_manager = self.cpu_manager.read().await;
        let task_manager = self.task_manager.read().await;
        diagnostics::run_diagnostics(&cpu_manager, &task_manager)
    }

    /// Attempt self-healing
    pub async fn heal(&self) -> Result<(), ProcessorCellError> {
        let mut state = self.state.write().await;
        *state = CellState::Healing;
        drop(state);

        let result = self.run_diagnostics().await;

        if !result.healthy {
            log::warn!("Processor Cell {} attempting self-healing", self.id);

            // Handle issues
            for issue in &result.issues {
                log::warn!("Issue: {} - {}", issue.severity, issue.description);
            }

            // Rebalance tasks
            let mut task_manager = self.task_manager.write().await;
            task_manager.rebalance()?;

            self.metrics.record_healing();
        }

        let mut state = self.state.write().await;
        let mut health = self.health.write().await;

        *state = CellState::Active;
        *health = if result.healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Warning
        };

        Ok(())
    }

    /// Shutdown the cell
    pub async fn shutdown(&self) -> Result<(), ProcessorCellError> {
        let mut state = self.state.write().await;
        *state = CellState::Shutdown;

        // Cancel all running tasks
        let mut task_manager = self.task_manager.write().await;
        task_manager.cancel_all()?;

        log::info!("Processor Cell {} shut down", self.id);

        Ok(())
    }

    /// Get metrics registry
    pub fn metrics_registry(&self) -> &prometheus::Registry {
        self.metrics.registry()
    }
}

impl Default for ProcessorCell {
    fn default() -> Self {
        Self::new("processor-cell-default")
    }
}
