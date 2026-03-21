//! Prometheus Metrics for Processor Cell

use lazy_static::lazy_static;
use prometheus::{
    Counter, Gauge, Histogram, Registry,
};

use super::cpu::CpuStats;
use super::task::Task;
use super::CellState;

/// Processor metrics collector
pub struct ProcessorMetrics {
    cell_id: String,
    registry: Registry,

    // Gauges
    total_cores: Gauge,
    active_cores: Gauge,
    total_utilization: Gauge,
    running_tasks: Gauge,
    pending_tasks: Gauge,

    // Counters
    tasks_started: Counter,
    tasks_completed: Counter,
    tasks_cancelled: Counter,
    healing_events: Counter,

    // Histograms
    task_duration: Histogram,

    // State
    cell_state: Gauge,
}

impl ProcessorMetrics {
    /// Create new metrics collector
    pub fn new(cell_id: &str) -> Self {
        let registry = Registry::new();

        let total_cores = Gauge::new("processor_cores_total", "Total CPU cores").unwrap();
        let active_cores = Gauge::new("processor_cores_active", "Active CPU cores").unwrap();
        let total_utilization = Gauge::new("processor_utilization_percent", "Total CPU utilization").unwrap();
        let running_tasks = Gauge::new("processor_tasks_running", "Running tasks").unwrap();
        let pending_tasks = Gauge::new("processor_tasks_pending", "Pending tasks").unwrap();

        let tasks_started = Counter::new(
            "processor_tasks_started_total",
            "Total tasks started",
        ).unwrap();
        let tasks_completed = Counter::new(
            "processor_tasks_completed_total",
            "Total tasks completed",
        ).unwrap();
        let tasks_cancelled = Counter::new(
            "processor_tasks_cancelled_total",
            "Total tasks cancelled",
        ).unwrap();
        let healing_events = Counter::new(
            "processor_healing_events_total",
            "Total healing events",
        ).unwrap();

        let task_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "processor_task_duration_seconds",
                "Task execution duration",
            )
            .buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 300.0]),
        ).unwrap();

        let cell_state = Gauge::new(
            "processor_cell_state",
            "Cell state (0=Init, 1=Active, 2=Degraded, 3=Healing, 4=Shutdown)",
        ).unwrap();

        // Register metrics
        registry.register(Box::new(total_cores.clone())).unwrap();
        registry.register(Box::new(active_cores.clone())).unwrap();
        registry.register(Box::new(total_utilization.clone())).unwrap();
        registry.register(Box::new(running_tasks.clone())).unwrap();
        registry.register(Box::new(pending_tasks.clone())).unwrap();
        registry.register(Box::new(tasks_started.clone())).unwrap();
        registry.register(Box::new(tasks_completed.clone())).unwrap();
        registry.register(Box::new(tasks_cancelled.clone())).unwrap();
        registry.register(Box::new(healing_events.clone())).unwrap();
        registry.register(Box::new(task_duration.clone())).unwrap();
        registry.register(Box::new(cell_state.clone())).unwrap();

        Self {
            cell_id: cell_id.to_string(),
            registry,
            total_cores,
            active_cores,
            total_utilization,
            running_tasks,
            pending_tasks,
            tasks_started,
            tasks_completed,
            tasks_cancelled,
            healing_events,
            task_duration,
            cell_state,
        }
    }

    /// Record CPU stats
    pub fn record_cpu_stats(&self, stats: &CpuStats) {
        self.total_cores.set(stats.total_cores as f64);
        self.active_cores.set(stats.active_cores as f64);
        self.total_utilization.set(stats.total_utilization);
    }

    /// Record task started
    pub fn record_task_started(&self, _task: &Task) {
        self.tasks_started.inc();
    }

    /// Record task completed
    pub fn record_task_completed(&self, duration_seconds: f64) {
        self.tasks_completed.inc();
        self.task_duration.observe(duration_seconds);
    }

    /// Record task cancelled
    pub fn record_task_cancelled(&self) {
        self.tasks_cancelled.inc();
    }

    /// Record healing event
    pub fn record_healing(&self) {
        self.healing_events.inc();
    }

    /// Record cell state
    pub fn record_state(&self, state: CellState) {
        let value = match state {
            CellState::Initializing => 0.0,
            CellState::Active => 1.0,
            CellState::Degraded => 2.0,
            CellState::Healing => 3.0,
            CellState::Shutdown => 4.0,
        };
        self.cell_state.set(value);
    }

    /// Get registry
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Get cell ID
    pub fn cell_id(&self) -> &str {
        &self.cell_id
    }
}
