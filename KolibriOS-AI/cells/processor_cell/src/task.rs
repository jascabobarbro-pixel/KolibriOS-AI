//! Task Management
//!
//! Manages task lifecycle and execution.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::ProcessorCellError;

/// Task Manager - Handles task lifecycle
pub struct TaskManager {
    tasks: HashMap<String, Task>,
    next_task_id: AtomicU64,
}

impl TaskManager {
    /// Create a new task manager
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            next_task_id: AtomicU64::new(1),
        }
    }

    /// Create a new task
    pub fn create_task(
        &mut self,
        executable: &str,
        args: Vec<String>,
        priority: TaskPriority,
        core_affinity: Option<u32>,
    ) -> Result<Task, ProcessorCellError> {
        let id = self.next_task_id.fetch_add(1, Ordering::SeqCst);
        let task_id = format!("task-{}", id);

        let task = Task {
            id: task_id.clone(),
            executable: executable.to_string(),
            args,
            priority,
            state: TaskState::Pending,
            core_affinity,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            started_at: None,
            completed_at: None,
            exit_code: None,
            user_time_ms: 0,
            system_time_ms: 0,
            memory_used: 0,
        };

        self.tasks.insert(task_id, task.clone());

        // Simulate starting the task
        if let Some(task) = self.tasks.get_mut(&task.id) {
            task.state = TaskState::Running;
            task.started_at = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            );
        }

        Ok(task)
    }

    /// Cancel a task
    pub fn cancel_task(&mut self, task_id: &str, force: bool) -> Result<(), ProcessorCellError> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.state = if force {
                TaskState::Cancelled
            } else {
                TaskState::Completed // Graceful stop
            };
            task.completed_at = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            );
            Ok(())
        } else {
            Err(ProcessorCellError::TaskNotFound(task_id.to_string()))
        }
    }

    /// Cancel all tasks
    pub fn cancel_all(&mut self) -> Result<(), ProcessorCellError> {
        for task in self.tasks.values_mut() {
            if task.state == TaskState::Running || task.state == TaskState::Pending {
                task.state = TaskState::Cancelled;
            }
        }
        Ok(())
    }

    /// Get task status
    pub fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        self.tasks.get(task_id).map(|t| TaskStatus {
            task_id: t.id.clone(),
            state: t.state,
            exit_code: t.exit_code,
            user_time_ms: t.user_time_ms,
            system_time_ms: t.system_time_ms,
            memory_used: t.memory_used,
            start_time: t.started_at,
            end_time: t.completed_at,
        })
    }

    /// List all tasks
    pub fn list_tasks(&self) -> Vec<Task> {
        self.tasks.values().cloned().collect()
    }

    /// Rebalance tasks across cores
    pub fn rebalance(&mut self) -> Result<(), ProcessorCellError> {
        // Simple rebalancing logic
        log::info!("Rebalancing tasks across cores");
        Ok(())
    }

    /// Get pending task count
    pub fn pending_count(&self) -> usize {
        self.tasks.values().filter(|t| t.state == TaskState::Pending).count()
    }

    /// Get running task count
    pub fn running_count(&self) -> usize {
        self.tasks.values().filter(|t| t.state == TaskState::Running).count()
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Task representation
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub executable: String,
    pub args: Vec<String>,
    pub priority: TaskPriority,
    pub state: TaskState,
    pub core_affinity: Option<u32>,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub exit_code: Option<i32>,
    pub user_time_ms: u64,
    pub system_time_ms: u64,
    pub memory_used: u64,
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Idle,
    Low,
    Normal,
    High,
    RealTime,
}

/// Task state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Pending,
    Running,
    Blocked,
    Completed,
    Failed,
    Cancelled,
}

/// Task status summary
#[derive(Debug, Clone)]
pub struct TaskStatus {
    pub task_id: String,
    pub state: TaskState,
    pub exit_code: Option<i32>,
    pub user_time_ms: u64,
    pub system_time_ms: u64,
    pub memory_used: u64,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
}
