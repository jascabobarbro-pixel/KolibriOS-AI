//! Unified AI Agent
//!
//! The central AI orchestration system that provides seamless integration
//! between the OS, AI capabilities, and user interactions.

#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

pub mod agent;
pub mod context;
pub mod commands;
pub mod memory;

/// Agent configuration
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub model: String,
    pub max_context_tokens: usize,
    pub enable_learning: bool,
    pub personality: Personality,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: String::from("Kolibri"),
            model: String::from("default"),
            max_context_tokens: 4096,
            enable_learning: true,
            personality: Personality::default(),
        }
    }
}

/// Agent personality settings
#[derive(Debug, Clone)]
pub struct Personality {
    pub tone: Tone,
    pub verbosity: Verbosity,
    pub formality: Formality,
    pub helpfulness: Helpfulness,
}

impl Default for Personality {
    fn default() -> Self {
        Self {
            tone: Tone::Friendly,
            verbosity: Verbosity::Balanced,
            formality: Formality::Casual,
            helpfulness: Helpfulness::Proactive,
        }
    }
}

/// Tone settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Professional,
    Friendly,
    Playful,
    Serious,
}

/// Verbosity settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    Concise,
    Balanced,
    Detailed,
}

/// Formality settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Formality {
    Formal,
    SemiFormal,
    Casual,
}

/// Helpfulness settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Helpfulness {
    Reactive,
    Balanced,
    Proactive,
}

/// Agent error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum AgentError {
    #[error("Context overflow")]
    ContextOverflow,
    
    #[error("Model error: {0}")]
    ModelError(String),
    
    #[error("Command not found: {0}")]
    CommandNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Memory error: {0}")]
    MemoryError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}
