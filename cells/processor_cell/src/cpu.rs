//! CPU Core Management
//!
//! Simulates and manages CPU cores with utilization tracking.

use std::collections::HashMap;
use sysinfo::{System, SystemExt, CpuExt};

use super::ProcessorCellError;

/// CPU Manager - Core management logic
pub struct CpuManager {
    cores: HashMap<u32, CpuCore>,
    system: System,
}

impl CpuManager {
    /// Create a new CPU manager
    pub fn new(core_count: u32) -> Self {
        let mut cores = HashMap::new();
        for i in 0..core_count {
            cores.insert(
                i,
                CpuCore {
                    core_id: i,
                    state: CpuCoreState::Idle,
                    utilization: 0.0,
                    frequency_mhz: 0,
                    temperature: 0,
                    running_task: None,
                },
            );
        }

        Self {
            cores,
            system: System::new(),
        }
    }

    /// Initialize the CPU manager
    pub fn initialize(&mut self) -> Result<(), ProcessorCellError> {
        self.system.refresh_cpu();
        log::info!("CPU Manager initialized with {} cores", self.cores.len());
        Ok(())
    }

    /// Get number of cores
    pub fn core_count(&self) -> u32 {
        self.cores.len() as u32
    }

    /// Find an available core
    pub fn find_available_core(&self) -> Option<u32> {
        self.cores
            .values()
            .filter(|c| c.state != CpuCoreState::Offline)
            .min_by(|a, b| {
                a.utilization.partial_cmp(&b.utilization).unwrap()
            })
            .map(|c| c.core_id)
    }

    /// Get CPU statistics
    pub fn get_stats(&self) -> CpuStats {
        let cores: Vec<CpuCoreInfo> = self
            .cores
            .values()
            .map(|c| CpuCoreInfo {
                core_id: c.core_id,
                state: c.state,
                utilization: c.utilization,
                frequency_mhz: c.frequency_mhz,
            })
            .collect();

        let active_cores = cores.iter().filter(|c| c.state != CpuCoreState::Offline).count() as u32;
        let total_utilization = if cores.is_empty() {
            0.0
        } else {
            cores.iter().map(|c| c.utilization).sum::<f64>() / cores.len() as f64
        };

        CpuStats {
            total_cores: self.cores.len() as u32,
            active_cores,
            total_utilization,
            total_frequency_mhz: cores.iter().map(|c| c.frequency_mhz).sum(),
            cores,
        }
    }

    /// Update core utilization
    pub fn update_utilization(&mut self, core_id: u32, utilization: f64) -> Result<(), ProcessorCellError> {
        if let Some(core) = self.cores.get_mut(&core_id) {
            core.utilization = utilization;
            Ok(())
        } else {
            Err(ProcessorCellError::CoreNotFound(core_id))
        }
    }

    /// Set core state
    pub fn set_core_state(&mut self, core_id: u32, state: CpuCoreState) -> Result<(), ProcessorCellError> {
        if let Some(core) = self.cores.get_mut(&core_id) {
            core.state = state;
            log::info!("Core {} state changed to {:?}", core_id, state);
            Ok(())
        } else {
            Err(ProcessorCellError::CoreNotFound(core_id))
        }
    }

    /// Assign task to core
    pub fn assign_task(&mut self, core_id: u32, task_id: &str) -> Result<(), ProcessorCellError> {
        if let Some(core) = self.cores.get_mut(&core_id) {
            core.running_task = Some(task_id.to_string());
            core.state = CpuCoreState::Active;
            Ok(())
        } else {
            Err(ProcessorCellError::CoreNotFound(core_id))
        }
    }

    /// Clear task from core
    pub fn clear_task(&mut self, core_id: u32) -> Result<(), ProcessorCellError> {
        if let Some(core) = self.cores.get_mut(&core_id) {
            core.running_task = None;
            core.state = CpuCoreState::Idle;
            Ok(())
        } else {
            Err(ProcessorCellError::CoreNotFound(core_id))
        }
    }

    /// Get core information
    pub fn get_core(&self, core_id: u32) -> Option<&CpuCore> {
        self.cores.get(&core_id)
    }

    /// List all cores
    pub fn list_cores(&self) -> Vec<&CpuCore> {
        self.cores.values().collect()
    }
}

/// CPU Core representation
#[derive(Debug, Clone)]
pub struct CpuCore {
    pub core_id: u32,
    pub state: CpuCoreState,
    pub utilization: f64,
    pub frequency_mhz: u64,
    pub temperature: u32,
    pub running_task: Option<String>,
}

/// Core state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuCoreState {
    Idle,
    Active,
    Sleep,
    Offline,
}

/// CPU statistics
#[derive(Debug, Clone)]
pub struct CpuStats {
    pub total_cores: u32,
    pub active_cores: u32,
    pub total_utilization: f64,
    pub total_frequency_mhz: u64,
    pub cores: Vec<CpuCoreInfo>,
}

/// Simplified core info for stats
#[derive(Debug, Clone)]
pub struct CpuCoreInfo {
    pub core_id: u32,
    pub state: CpuCoreState,
    pub utilization: f64,
    pub frequency_mhz: u64,
}
