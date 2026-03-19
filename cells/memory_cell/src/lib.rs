//! Memory Cell - Autonomous Memory Management
//!
//! This cell provides intelligent, self-organizing memory management
//! with predictive allocation, automatic optimization, and Prometheus metrics.

pub mod memory;
pub mod metrics;
pub mod grpc;
pub mod diagnostics;

use std::sync::Arc;
use tokio::sync::RwLock;
use memory::MemoryManager;
use metrics::MemoryMetrics;
use thiserror::Error;

/// Memory Cell error types
#[derive(Debug, Error)]
pub enum MemoryCellError {
    #[error("Allocation failed: {0}")]
    AllocationFailed(String),

    #[error("Deallocation failed: {0}")]
    DeallocationFailed(String),

    #[error("Pool not found: {0}")]
    PoolNotFound(String),

    #[error("Insufficient memory: requested {requested}, available {available}")]
    InsufficientMemory { requested: u64, available: u64 },

    #[error("Invalid allocation ID: {0}")]
    InvalidAllocationId(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Memory Cell state
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

/// Memory Cell - The main structure
pub struct MemoryCell {
    id: String,
    state: Arc<RwLock<CellState>>,
    health: Arc<RwLock<HealthStatus>>,
    memory_manager: Arc<RwLock<MemoryManager>>,
    metrics: MemoryMetrics,
}

impl MemoryCell {
    /// Create a new Memory Cell
    pub fn new(id: &str, total_memory: u64) -> Self {
        Self {
            id: id.to_string(),
            state: Arc::new(RwLock::new(CellState::Initializing)),
            health: Arc::new(RwLock::new(HealthStatus::Healthy)),
            memory_manager: Arc::new(RwLock::new(MemoryManager::new(total_memory))),
            metrics: MemoryMetrics::new(id),
        }
    }

    /// Initialize the memory cell
    pub async fn initialize(&self) -> Result<(), MemoryCellError> {
        let mut state = self.state.write().await;

        // Create default memory pools
        let mut manager = self.memory_manager.write().await;
        manager.create_pool("kernel", 16 * 1024 * 1024, memory::PoolType::Kernel)?;
        manager.create_pool("user", 512 * 1024 * 1024, memory::PoolType::User)?;
        manager.create_pool("shared", 256 * 1024 * 1024, memory::PoolType::Shared)?;

        *state = CellState::Active;
        self.metrics.record_state(CellState::Active);
        log::info!("Memory Cell {} initialized successfully", self.id);

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

    /// Allocate memory
    pub async fn allocate(
        &self,
        size: u64,
        pool_name: &str,
        flags: memory::AllocationFlags,
    ) -> Result<memory::Allocation, MemoryCellError> {
        let mut manager = self.memory_manager.write().await;

        let allocation = manager.allocate(size, pool_name, flags)?;

        // Update metrics
        self.metrics.record_allocation(&allocation);
        self.metrics.record_pool_stats(&manager);

        log::debug!(
            "Allocated {} bytes in pool '{}' with ID {}",
            size,
            pool_name,
            allocation.id
        );

        Ok(allocation)
    }

    /// Deallocate memory
    pub async fn deallocate(&self, allocation_id: &str) -> Result<(), MemoryCellError> {
        let mut manager = self.memory_manager.write().await;

        manager.deallocate(allocation_id)?;

        // Update metrics
        self.metrics.record_deallocation();
        self.metrics.record_pool_stats(&manager);

        log::debug!("Deallocated memory with ID {}", allocation_id);

        Ok(())
    }

    /// Get memory statistics
    pub async fn get_stats(&self) -> memory::MemoryStats {
        let manager = self.memory_manager.read().await;
        manager.get_stats()
    }

    /// Run diagnostics
    pub async fn run_diagnostics(&self) -> diagnostics::DiagnosticsResult {
        let manager = self.memory_manager.read().await;
        diagnostics::run_diagnostics(&manager)
    }

    /// Attempt self-healing
    pub async fn heal(&self) -> Result<(), MemoryCellError> {
        let mut state = self.state.write().await;
        *state = CellState::Healing;

        drop(state);

        // Run diagnostics first
        let result = self.run_diagnostics().await;

        if !result.healthy {
            log::warn!("Memory Cell {} attempting self-healing", self.id);

            let mut manager = self.memory_manager.write().await;

            for issue in &result.issues {
                match issue.severity {
                    diagnostics::Severity::Warning => {
                        log::info!("Attempting to fix warning: {}", issue.description);
                    }
                    diagnostics::Severity::Critical => {
                        log::error!("Critical issue detected: {}", issue.description);
                    }
                    _ => {}
                }
            }

            // Try to defragment pools
            manager.defragment_all()?;

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
    pub async fn shutdown(&self) -> Result<(), MemoryCellError> {
        let mut state = self.state.write().await;
        *state = CellState::Shutdown;

        log::info!("Memory Cell {} shut down", self.id);

        Ok(())
    }

    /// Get metrics registry for Prometheus
    pub fn metrics_registry(&self) -> &prometheus::Registry {
        self.metrics.registry()
    }
}

impl Default for MemoryCell {
    fn default() -> Self {
        Self::new("memory-cell-default", 1024 * 1024 * 1024) // 1GB default
    }
}
