//! Koli Language Runtime
//!
//! The execution environment for Koli programs, providing
//! memory management, garbage collection, and AI integration.

#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

pub mod value;
pub mod vm;
pub mod gc;
pub mod ai_bridge;

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub heap_size: usize,
    pub stack_size: usize,
    pub gc_threshold: usize,
    pub enable_ai: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            heap_size: 16 * 1024 * 1024, // 16 MB
            stack_size: 1024 * 1024,     // 1 MB
            gc_threshold: 8 * 1024 * 1024, // 8 MB
            enable_ai: true,
        }
    }
}

/// Koli Runtime
pub struct Runtime {
    config: RuntimeConfig,
    vm: vm::VirtualMachine,
    gc: gc::GarbageCollector,
    ai_bridge: Option<ai_bridge::AiBridge>,
}

impl Runtime {
    /// Create a new runtime
    pub fn new(config: RuntimeConfig) -> Self {
        let ai_bridge = if config.enable_ai {
            Some(ai_bridge::AiBridge::new())
        } else {
            None
        };

        Self {
            config,
            vm: vm::VirtualMachine::new(),
            gc: gc::GarbageCollector::new(),
            ai_bridge,
        }
    }

    /// Execute bytecode
    pub fn execute(&mut self, bytecode: &[u8]) -> Result<value::Value, RuntimeError> {
        self.vm.run(bytecode, &mut self.gc)
    }

    /// Call a function
    pub fn call_function(&mut self, name: &str, args: &[value::Value]) -> Result<value::Value, RuntimeError> {
        self.vm.call(name, args, &mut self.gc)
    }

    /// Invoke AI capability
    pub fn ai_invoke(&mut self, model: &str, input: &str) -> Result<String, RuntimeError> {
        if let Some(ref mut bridge) = self.ai_bridge {
            bridge.invoke(model, input)
        } else {
            Err(RuntimeError::AiDisabled)
        }
    }

    /// Force garbage collection
    pub fn gc_collect(&mut self) {
        self.gc.collect();
    }

    /// Get runtime statistics
    pub fn stats(&self) -> RuntimeStats {
        RuntimeStats {
            heap_used: self.gc.used(),
            heap_total: self.config.heap_size,
            allocations: self.gc.allocations(),
        }
    }
}

/// Runtime statistics
#[derive(Debug, Clone, Copy)]
pub struct RuntimeStats {
    pub heap_used: usize,
    pub heap_total: usize,
    pub allocations: usize,
}

/// Runtime error
#[derive(Debug, Clone, thiserror::Error)]
pub enum RuntimeError {
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("Stack overflow")]
    StackOverflow,
    
    #[error("Out of memory")]
    OutOfMemory,
    
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
    
    #[error("Type error: {0}")]
    TypeError(String),
    
    #[error("AI capabilities disabled")]
    AiDisabled,
    
    #[error("AI error: {0}")]
    AiError(String),
}
