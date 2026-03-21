//! Prometheus Metrics for Memory Cell
//!
//! Exposes metrics for monitoring and observability.

use lazy_static::lazy_static;
use prometheus::{
    register_counter, register_gauge, register_histogram, Counter, Gauge, Histogram, Registry,
};

use super::memory::{MemoryManager, MemoryStats};
use super::CellState;

/// Memory metrics collector
pub struct MemoryMetrics {
    cell_id: String,
    registry: Registry,

    // Gauges
    total_memory: Gauge,
    used_memory: Gauge,
    available_memory: Gauge,
    utilization: Gauge,
    allocation_count: Gauge,
    pool_count: Gauge,

    // Counters
    allocations_total: Counter,
    deallocations_total: Counter,
    healing_events: Counter,

    // Histograms
    allocation_size: Histogram,
    allocation_duration: Histogram,

    // State
    cell_state: Gauge,
}

impl MemoryMetrics {
    /// Create new metrics collector
    pub fn new(cell_id: &str) -> Self {
        let registry = Registry::new();

        // Create metrics
        let total_memory = Gauge::new("memory_total_bytes", "Total managed memory in bytes").unwrap();
        let used_memory = Gauge::new("memory_used_bytes", "Used memory in bytes").unwrap();
        let available_memory = Gauge::new("memory_available_bytes", "Available memory in bytes").unwrap();
        let utilization = Gauge::new("memory_utilization_percent", "Memory utilization percentage").unwrap();
        let allocation_count = Gauge::new("memory_allocations_active", "Number of active allocations").unwrap();
        let pool_count = Gauge::new("memory_pools_total", "Number of memory pools").unwrap();

        let allocations_total = Counter::new(
            "memory_allocations_total",
            "Total number of allocations",
        ).unwrap();
        let deallocations_total = Counter::new(
            "memory_deallocations_total",
            "Total number of deallocations",
        ).unwrap();
        let healing_events = Counter::new(
            "memory_healing_events_total",
            "Total number of self-healing events",
        ).unwrap();

        let allocation_size = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "memory_allocation_size_bytes",
                "Distribution of allocation sizes",
            )
            .buckets(vec![64.0, 256.0, 1024.0, 4096.0, 16384.0, 65536.0, 262144.0, 1048576.0]),
        )
        .unwrap();

        let allocation_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "memory_allocation_duration_seconds",
                "Time spent on allocations",
            )
            .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]),
        )
        .unwrap();

        let cell_state = Gauge::new("memory_cell_state", "Current cell state (0=Init, 1=Active, 2=Degraded, 3=Healing, 4=Shutdown)").unwrap();

        // Register metrics
        registry.register(Box::new(total_memory.clone())).unwrap();
        registry.register(Box::new(used_memory.clone())).unwrap();
        registry.register(Box::new(available_memory.clone())).unwrap();
        registry.register(Box::new(utilization.clone())).unwrap();
        registry.register(Box::new(allocation_count.clone())).unwrap();
        registry.register(Box::new(pool_count.clone())).unwrap();
        registry.register(Box::new(allocations_total.clone())).unwrap();
        registry.register(Box::new(deallocations_total.clone())).unwrap();
        registry.register(Box::new(healing_events.clone())).unwrap();
        registry.register(Box::new(allocation_size.clone())).unwrap();
        registry.register(Box::new(allocation_duration.clone())).unwrap();
        registry.register(Box::new(cell_state.clone())).unwrap();

        Self {
            cell_id: cell_id.to_string(),
            registry,
            total_memory,
            used_memory,
            available_memory,
            utilization,
            allocation_count,
            pool_count,
            allocations_total,
            deallocations_total,
            healing_events,
            allocation_size,
            allocation_duration,
            cell_state,
        }
    }

    /// Record an allocation
    pub fn record_allocation(&self, allocation: &super::memory::Allocation) {
        self.allocations_total.inc();
        self.allocation_size.observe(allocation.size as f64);
    }

    /// Record a deallocation
    pub fn record_deallocation(&self) {
        self.deallocations_total.inc();
    }

    /// Record pool statistics
    pub fn record_pool_stats(&self, manager: &MemoryManager) {
        let stats = manager.get_stats();

        self.total_memory.set(stats.total_memory as f64);
        self.used_memory.set(stats.used_memory as f64);
        self.available_memory.set(stats.available_memory as f64);
        self.utilization.set(stats.utilization_percent);
        self.allocation_count.set(stats.allocation_count as f64);
        self.pool_count.set(stats.pool_count as f64);
    }

    /// Record a healing event
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

    /// Get the Prometheus registry
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Get cell ID
    pub fn cell_id(&self) -> &str {
        &self.cell_id
    }
}
