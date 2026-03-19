//! gRPC Service Implementation for Processor Cell

use tonic::{Request, Response, Status};

// Generated protobuf modules
pub mod proto {
    pub mod cell_common {
        tonic::include_proto!("cell_common");
    }
    pub mod processor_cell {
        tonic::include_proto!("processor_cell");
    }
}

use proto::processor_cell::{
    processor_cell_service_server::ProcessorCellService,
    CpuCore as ProtoCpuCore, CpuCoreState, CpuStats as ProtoCpuStats,
    ExecuteTaskRequest, ExecuteTaskResponse, ListTasksRequest,
    SetCoreStateRequest, TaskPriority as ProtoTaskPriority, TaskState as ProtoTaskState,
    TaskStatus as ProtoTaskStatus,
};
use proto::cell_common::{CellStatus, CommandResponse, Empty, HealthStatus, Heartbeat, Metrics};

use crate::{CellState, HealthStatus as CellHealthStatus, ProcessorCell};

/// gRPC Processor Cell Service
pub struct ProcessorCellGrpcService {
    cell: std::sync::Arc<ProcessorCell>,
}

impl ProcessorCellGrpcService {
    /// Create a new gRPC service
    pub fn new(cell: std::sync::Arc<ProcessorCell>) -> Self {
        Self { cell }
    }
}

#[tonic::async_trait]
impl ProcessorCellService for ProcessorCellGrpcService {
    async fn execute_task(
        &self,
        request: Request<ExecuteTaskRequest>,
    ) -> Result<Response<ExecuteTaskResponse>, Status> {
        let req = request.into_inner();

        let priority = match req.priority {
            0 => crate::task::TaskPriority::Idle,
            1 => crate::task::TaskPriority::Low,
            2 => crate::task::TaskPriority::Normal,
            3 => crate::task::TaskPriority::High,
            4 => crate::task::TaskPriority::RealTime,
            _ => crate::task::TaskPriority::Normal,
        };

        match self.cell.execute_task(&req.executable_path, req.args, priority).await {
            Ok(task) => Ok(Response::new(ExecuteTaskResponse {
                success: true,
                task_id: task.id,
                pid: 0, // Simulated
                error_message: String::new(),
            })),
            Err(e) => Ok(Response::new(ExecuteTaskResponse {
                success: false,
                task_id: String::new(),
                pid: 0,
                error_message: e.to_string(),
            })),
        }
    }

    async fn cancel_task(
        &self,
        request: Request<proto::processor_cell::CancelTaskRequest>,
    ) -> Result<Response<CommandResponse>, Status> {
        let req = request.into_inner();

        match self.cell.cancel_task(&req.task_id, req.force).await {
            Ok(()) => Ok(Response::new(CommandResponse {
                success: true,
                message: format!("Task {} cancelled", req.task_id),
                error_code: 0,
                metadata: std::collections::HashMap::new(),
            })),
            Err(e) => Ok(Response::new(CommandResponse {
                success: false,
                message: e.to_string(),
                error_code: 1,
                metadata: std::collections::HashMap::new(),
            })),
        }
    }

    async fn get_task_status(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<ProtoTaskStatus>, Status> {
        Err(Status::unimplemented("Use task_id parameter"))
    }

    type ListTasksStream = std::pin::Pin<
        Box<dyn futures::Stream<Item = Result<ProtoTaskStatus, Status>> + Send>,
    >;

    async fn list_tasks(
        &self,
        _request: Request<ListTasksRequest>,
    ) -> Result<Response<Self::ListTasksStream>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn schedule_task(
        &self,
        _request: Request<proto::processor_cell::ScheduleTaskRequest>,
    ) -> Result<Response<CommandResponse>, Status> {
        Ok(Response::new(CommandResponse {
            success: true,
            message: "Task scheduled".to_string(),
            error_code: 0,
            metadata: std::collections::HashMap::new(),
        }))
    }

    async fn set_priority(
        &self,
        _request: Request<proto::processor_cell::ScheduleTaskRequest>,
    ) -> Result<Response<CommandResponse>, Status> {
        Ok(Response::new(CommandResponse {
            success: true,
            message: "Priority set".to_string(),
            error_code: 0,
            metadata: std::collections::HashMap::new(),
        }))
    }

    type GetCoreInfoStream = std::pin::Pin<
        Box<dyn futures::Stream<Item = Result<ProtoCpuCore, Status>> + Send>,
    >;

    async fn get_core_info(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::GetCoreInfoStream>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn set_core_state(
        &self,
        _request: Request<SetCoreStateRequest>,
    ) -> Result<Response<CommandResponse>, Status> {
        Ok(Response::new(CommandResponse {
            success: true,
            message: "Core state set".to_string(),
            error_code: 0,
            metadata: std::collections::HashMap::new(),
        }))
    }

    async fn get_cpu_stats(&self, _request: Request<Empty>) -> Result<Response<ProtoCpuStats>, Status> {
        let stats = self.cell.get_cpu_stats().await;

        Ok(Response::new(ProtoCpuStats {
            total_cores: stats.total_cores,
            active_cores: stats.active_cores,
            total_utilization_percent: stats.total_utilization,
            total_frequency_mhz: stats.total_frequency_mhz,
            cores: stats.cores.into_iter().map(|c| ProtoCpuCore {
                core_id: c.core_id,
                state: match c.state {
                    crate::cpu::CpuCoreState::Idle => CpuCoreState::Idle,
                    crate::cpu::CpuCoreState::Active => CpuCoreState::Active,
                    crate::cpu::CpuCoreState::Sleep => CpuCoreState::Sleep,
                    crate::cpu::CpuCoreState::Offline => CpuCoreState::Offline,
                },
                utilization_percent: c.utilization,
                frequency_mhz: c.frequency_mhz,
                temperature_celsius: 0,
                running_tasks: vec![],
            }).collect(),
            running_tasks: 0,
            pending_tasks: 0,
            context_switches: 0,
            interrupts: 0,
        }))
    }

    async fn get_heartbeat(&self, _request: Request<Empty>) -> Result<Response<Heartbeat>, Status> {
        let state = self.cell.state().await;
        let health = self.cell.health().await;

        Ok(Response::new(Heartbeat {
            cell_id: self.cell.id().to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
            status: match state {
                CellState::Initializing => CellStatus::Initializing,
                CellState::Active => CellStatus::Active,
                CellState::Degraded => CellStatus::Degraded,
                CellState::Healing => CellStatus::Healing,
                CellState::Shutdown => CellStatus::Shutdown,
            },
            health: match health {
                CellHealthStatus::Healthy => HealthStatus::Healthy,
                CellHealthStatus::Warning => HealthStatus::Warning,
                CellHealthStatus::Critical => HealthStatus::Critical,
            },
        }))
    }

    async fn get_metrics(&self, _request: Request<Empty>) -> Result<Response<Metrics>, Status> {
        let stats = self.cell.get_cpu_stats().await;
        let state = self.cell.state().await;
        let health = self.cell.health().await;

        let mut gauge_metrics = std::collections::HashMap::new();
        gauge_metrics.insert("total_utilization".to_string(), stats.total_utilization);
        gauge_metrics.insert("active_cores".to_string(), stats.active_cores as f64);

        let mut counter_metrics = std::collections::HashMap::new();
        counter_metrics.insert("total_cores".to_string(), stats.total_cores as i64);

        Ok(Response::new(Metrics {
            cell_id: self.cell.id().to_string(),
            cell_type: "processor".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
            gauge_metrics,
            counter_metrics,
            status: match state {
                CellState::Initializing => CellStatus::Initializing,
                CellState::Active => CellStatus::Active,
                CellState::Degraded => CellStatus::Degraded,
                CellState::Healing => CellStatus::Healing,
                CellState::Shutdown => CellStatus::Shutdown,
            },
            health: match health {
                CellHealthStatus::Healthy => HealthStatus::Healthy,
                CellHealthStatus::Warning => HealthStatus::Warning,
                CellHealthStatus::Critical => HealthStatus::Critical,
            },
        }))
    }

    async fn run_diagnostics(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<proto::processor_cell::DiagnosticsResult>, Status> {
        let result = self.cell.run_diagnostics().await;

        Ok(Response::new(proto::processor_cell::DiagnosticsResult {
            healthy: result.healthy,
            issues: result.issues.into_iter().map(|i| proto::processor_cell::DiagnosticIssue {
                severity: format!("{}", i.severity),
                component: i.component,
                description: i.description,
                suggested_action: i.suggested_action,
            }).collect(),
            recommendations: result.recommendations,
        }))
    }
}
