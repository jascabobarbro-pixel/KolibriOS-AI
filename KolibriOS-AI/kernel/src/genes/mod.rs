//! Kernel Genes - Modular kernel components
//!
//! Each gene is a modular kernel component that provides specific functionality.
//! Genes follow the biological metaphor:
//! - DNA: Configuration and parameters
//! - RNA: Runtime state
//! - Protein: Exposed interfaces/APIs

pub mod process_gene;
pub mod memory_gene;
pub mod io_gene;
pub mod gene_trait;

pub use gene_trait::Gene;
pub use process_gene::ProcessGene;
pub use memory_gene::MemoryGene;
pub use io_gene::IOGene;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// Gene identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeneId(u64);

impl GeneId {
    fn new() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

/// Gene DNA - Configuration and parameters
#[derive(Debug, Clone)]
pub struct GeneDNA {
    /// Gene name
    pub name: String,
    /// Gene version
    pub version: u32,
    /// Configuration parameters
    pub config: BTreeMap<String, GeneValue>,
    /// Activation threshold (0.0 - 1.0)
    pub activation_threshold: f32,
    /// Whether the gene can be disabled
    pub critical: bool,
}

impl Default for GeneDNA {
    fn default() -> Self {
        Self {
            name: String::from("unnamed"),
            version: 1,
            config: BTreeMap::new(),
            activation_threshold: 0.5,
            critical: false,
        }
    }
}

/// Gene RNA - Runtime state
#[derive(Debug, Clone)]
pub struct GeneRNA {
    /// Current activity level (0.0 - 1.0)
    pub activity: f32,
    /// Number of activations
    pub activation_count: u64,
    /// Last activation timestamp
    pub last_activation: u64,
    /// Health status (0.0 - 1.0)
    pub health: f32,
    /// Error count
    pub error_count: u32,
}

impl Default for GeneRNA {
    fn default() -> Self {
        Self {
            activity: 0.0,
            activation_count: 0,
            last_activation: 0,
            health: 1.0,
            error_count: 0,
        }
    }
}

/// Gene value for configuration
#[derive(Debug, Clone)]
pub enum GeneValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Bytes(Vec<u8>),
}

impl GeneValue {
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            GeneValue::Integer(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            GeneValue::Float(v) => Some(*v),
            GeneValue::Integer(v) => Some(*v as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            GeneValue::Boolean(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match self {
            GeneValue::String(v) => Some(v),
            _ => None,
        }
    }
}

/// Gene activation result
#[derive(Debug, Clone)]
pub struct GeneActivation {
    /// Whether the gene was activated
    pub activated: bool,
    /// Result value (if any)
    pub result: Option<GeneValue>,
    /// Side effects to apply
    pub effects: Vec<GeneEffect>,
}

/// Gene side effects
#[derive(Debug, Clone)]
pub enum GeneEffect {
    /// Modify another gene's DNA
    ModifyDNA { gene_name: String, key: String, value: GeneValue },
    /// Request memory allocation
    RequestMemory { size: usize, purpose: String },
    /// Request process creation
    RequestProcess { name: String, priority: u8 },
    /// Trigger another gene
    TriggerGene { gene_name: String },
    /// Log message
    Log { level: String, message: String },
}

/// Gene error
#[derive(Debug, Clone)]
pub enum GeneError {
    NotFound(String),
    ActivationFailed(String),
    InvalidConfig(String),
    Disabled(String),
    ResourceUnavailable(String),
}

impl core::fmt::Display for GeneError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GeneError::NotFound(s) => write!(f, "Gene not found: {}", s),
            GeneError::ActivationFailed(s) => write!(f, "Gene activation failed: {}", s),
            GeneError::InvalidConfig(s) => write!(f, "Invalid configuration: {}", s),
            GeneError::Disabled(s) => write!(f, "Gene is disabled: {}", s),
            GeneError::ResourceUnavailable(s) => write!(f, "Resource unavailable: {}", s),
        }
    }
}
