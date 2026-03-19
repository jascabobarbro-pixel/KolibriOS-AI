//! Diagnostics for Memory Cell
//!
//! Self-diagnosis and health checking capabilities.

use super::memory::{MemoryManager, PoolStats};

/// Run diagnostics on memory manager
pub fn run_diagnostics(manager: &MemoryManager) -> DiagnosticsResult {
    let stats = manager.get_stats();
    let mut issues = Vec::new();
    let mut recommendations = Vec::new();

    // Check overall memory utilization
    if stats.utilization_percent > 90.0 {
        issues.push(DiagnosticIssue {
            severity: Severity::Critical,
            component: "memory".to_string(),
            description: format!(
                "Memory utilization critically high: {:.1}%",
                stats.utilization_percent
            ),
            suggested_action: "Consider expanding memory pools or deallocating unused memory".to_string(),
        });
    } else if stats.utilization_percent > 75.0 {
        issues.push(DiagnosticIssue {
            severity: Severity::Warning,
            component: "memory".to_string(),
            description: format!(
                "Memory utilization high: {:.1}%",
                stats.utilization_percent
            ),
            suggested_action: "Monitor memory usage and consider optimization".to_string(),
        });
    }

    // Check individual pools
    for pool in &stats.pools {
        let pool_utilization = if pool.total_size > 0 {
            (pool.used_size as f64 / pool.total_size as f64) * 100.0
        } else {
            0.0
        };

        if pool_utilization > 95.0 {
            issues.push(DiagnosticIssue {
                severity: Severity::Critical,
                component: format!("pool:{}", pool.name),
                description: format!(
                    "Pool '{}' is nearly full ({:.1}% utilized)",
                    pool.name, pool_utilization
                ),
                suggested_action: format!(
                    "Expand pool '{}' or deallocate unused memory",
                    pool.name
                ),
            });
        }

        // Check fragmentation
        if pool.fragmentation_percent > 50.0 {
            issues.push(DiagnosticIssue {
                severity: Severity::Warning,
                component: format!("pool:{}", pool.name),
                description: format!(
                    "Pool '{}' is highly fragmented ({:.1}%)",
                    pool.name, pool.fragmentation_percent
                ),
                suggested_action: "Run defragmentation on this pool".to_string(),
            });
            recommendations.push(format!("Defragment pool '{}'", pool.name));
        }
    }

    // Check for potential memory leaks (very high allocation count)
    if stats.allocation_count > 10000 {
        issues.push(DiagnosticIssue {
            severity: Severity::Warning,
            component: "memory".to_string(),
            description: format!(
                "High number of active allocations: {}",
                stats.allocation_count
            ),
            suggested_action: "Investigate potential memory leaks".to_string(),
        });
    }

    let healthy = !issues.iter().any(|i| {
        matches!(i.severity, Severity::Critical | Severity::Error)
    });

    if healthy && issues.is_empty() {
        recommendations.push("Memory system is healthy".to_string());
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
