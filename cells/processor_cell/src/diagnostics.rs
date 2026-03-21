//! Diagnostics for Processor Cell

use super::cpu::{CpuManager, CpuStats};
use super::task::TaskManager;

/// Run diagnostics on processor cell
pub fn run_diagnostics(cpu_manager: &CpuManager, task_manager: &TaskManager) -> DiagnosticsResult {
    let stats = cpu_manager.get_stats();
    let mut issues = Vec::new();
    let mut recommendations = Vec::new();

    // Check overall CPU utilization
    if stats.total_utilization > 90.0 {
        issues.push(DiagnosticIssue {
            severity: Severity::Critical,
            component: "cpu".to_string(),
            description: format!(
                "CPU utilization critically high: {:.1}%",
                stats.total_utilization
            ),
            suggested_action: "Consider load balancing or scaling".to_string(),
        });
    } else if stats.total_utilization > 75.0 {
        issues.push(DiagnosticIssue {
            severity: Severity::Warning,
            component: "cpu".to_string(),
            description: format!(
                "CPU utilization high: {:.1}%",
                stats.total_utilization
            ),
            suggested_action: "Monitor for potential performance issues".to_string(),
        });
    }

    // Check for offline cores
    let offline_count = stats.cores.iter().filter(|c| c.state == super::cpu::CpuCoreState::Offline).count();
    if offline_count > 0 {
        issues.push(DiagnosticIssue {
            severity: Severity::Warning,
            component: "cores".to_string(),
            description: format!("{} cores are offline", offline_count),
            suggested_action: "Check hardware status of offline cores".to_string(),
        });
    }

    // Check task queue
    let pending = task_manager.pending_count();
    let running = task_manager.running_count();

    if pending > 100 {
        issues.push(DiagnosticIssue {
            severity: Severity::Warning,
            component: "tasks".to_string(),
            description: format!("Large task backlog: {} pending", pending),
            suggested_action: "Consider increasing processing capacity".to_string(),
        });
        recommendations.push("Scale up processing resources".to_string());
    }

    let healthy = !issues.iter().any(|i| {
        matches!(i.severity, Severity::Critical | Severity::Error)
    });

    if healthy && issues.is_empty() {
        recommendations.push("Processor system is healthy".to_string());
    }

    DiagnosticsResult {
        healthy,
        issues,
        recommendations,
    }
}

/// Diagnostics result
#[derive(Debug, Clone)]
pub struct DiagnosticsResult {
    pub healthy: bool,
    pub issues: Vec<DiagnosticIssue>,
    pub recommendations: Vec<String>,
}

/// Diagnostic issue
#[derive(Debug, Clone)]
pub struct DiagnosticIssue {
    pub severity: Severity,
    pub component: String,
    pub description: String,
    pub suggested_action: String,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Error => write!(f, "ERROR"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}
