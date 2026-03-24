//! KolibriOS AI Living Kernel
//!
//! A revolutionary kernel featuring:
//! - Modular "genes" for kernel functionalities
//! - Neural Scheduler for intelligent task scheduling
//! - Living memory management with adaptation

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

extern crate alloc;

pub mod arch;
pub mod genes;
pub mod ipc;
pub mod mm;
pub mod neural_scheduler;
pub mod sched;
pub mod security;

use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};

use genes::{GeneRegistry, IOGene, MemoryGene, ProcessGene};
use neural_scheduler::{NeuralScheduler, SchedulingDecision, SystemStateInput};

/// Kernel version
pub const KERNEL_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const KERNEL_NAME: &str = "KolibriOS AI Living Kernel";

/// Kernel state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelState {
    Uninitialized, EarlyBoot, GenesInitializing, NeuralSchedulerReady, FullyOperational, Degraded, ShuttingDown,
}

/// Living Kernel
pub struct LivingKernel {
    state: KernelState,
    gene_registry: GeneRegistry,
    neural_scheduler: NeuralScheduler,
    memory_gene: MemoryGene,
    process_gene: ProcessGene,
    io_gene: IOGene,
    boot_time: AtomicU64,
    tick_count: AtomicU64,
}

impl LivingKernel {
    pub fn new(total_memory: usize) -> Self {
        Self {
            state: KernelState::Uninitialized,
            gene_registry: GeneRegistry::new(),
            neural_scheduler: NeuralScheduler::default(),
            memory_gene: MemoryGene::new(total_memory),
            process_gene: ProcessGene::new(),
            io_gene: IOGene::new(),
            boot_time: AtomicU64::new(0),
            tick_count: AtomicU64::new(0),
        }
    }

    pub fn init(&mut self) -> Result<(), KernelError> {
        self.state = KernelState::EarlyBoot;
        self.init_genes()?;
        self.neural_scheduler = NeuralScheduler::default();
        self.state = KernelState::NeuralSchedulerReady;
        self.state = KernelState::FullyOperational;
        Ok(())
    }

    fn init_genes(&mut self) -> Result<(), KernelError> {
        self.state = KernelState::GenesInitializing;
        self.gene_registry.register(ProcessGene::new());
        self.gene_registry.register(MemoryGene::new(1024 * 1024 * 1024));
        self.gene_registry.register(IOGene::new());
        Ok(())
    }

    pub fn run(&mut self) -> ! {
        self.state = KernelState::FullyOperational;
        loop {
            self.tick();
            arch::enable_interrupts();
            arch::halt();
        }
    }

    pub fn tick(&mut self) {
        self.tick_count.fetch_add(1, Ordering::SeqCst);
        let state = self.collect_system_state();
        let decision = self.neural_scheduler.decide(&state);
        self.execute_decision(decision);
        let _ = self.gene_registry.update_all(1);
    }

    fn collect_system_state(&self) -> SystemStateInput {
        let mem_stats = self.memory_gene.stats();
        SystemStateInput {
            cpu_utilization: vec![0.5, 0.5, 0.5, 0.5],
            memory_pressure: mem_stats.used as f32 / mem_stats.total as f32,
            ready_tasks: self.process_gene.process_count() as f32 / 100.0,
            blocked_tasks: 0.1, avg_priority: 0.5, io_bound_ratio: 0.3,
            cache_hit_ratio: 0.85, load_average: 0.5, context_switches: 0.1,
            interrupt_rate: 0.05, time_since_decision: 0.01,
        }
    }

    fn execute_decision(&mut self, _decision: SchedulingDecision) {}

    pub fn process_memory_feedback(&mut self, cell_id: &str, pressure: f32, recommendation: &str) {
        use genes::memory_gene::MemoryFeedback;
        let feedback = MemoryFeedback {
            cell_id: String::from(cell_id), pressure,
            recommendation: String::from(recommendation),
            timestamp: self.tick_count.load(Ordering::SeqCst),
        };
        self.memory_gene.process_feedback(&feedback);
    }

    pub fn state(&self) -> KernelState { self.state }
    pub fn memory_stats(&self) -> genes::memory_gene::MemoryStats { self.memory_gene.stats() }
    pub fn scheduler_stats(&self) -> neural_scheduler::SchedulerStats { self.neural_scheduler.stats() }

    pub fn shutdown(&mut self) -> Result<(), KernelError> {
        self.state = KernelState::ShuttingDown;
        Ok(())
    }
}

impl Default for LivingKernel {
    fn default() -> Self { Self::new(1024 * 1024 * 1024) }
}

#[derive(Debug, Clone)]
pub enum KernelError {
    MemoryInit(String),
    IpcInit(String),
    SchedulerInit(String),
    GeneError(String),
    CellConnection(String),
    InvalidState,
}

impl core::fmt::Display for KernelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            KernelError::MemoryInit(s) => write!(f, "Memory initialization failed: {}", s),
            KernelError::IpcInit(s) => write!(f, "IPC initialization failed: {}", s),
            KernelError::SchedulerInit(s) => write!(f, "Scheduler initialization failed: {}", s),
            KernelError::GeneError(s) => write!(f, "Gene error: {}", s),
            KernelError::CellConnection(s) => write!(f, "Cell connection failed: {}", s),
            KernelError::InvalidState => write!(f, "Invalid kernel state"),
        }
    }
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    let mut kernel = LivingKernel::default();
    if let Err(e) = kernel.init() { panic!("Kernel initialization failed: {}", e); }
    kernel.run()
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    loop { arch::halt(); }
}

#[alloc_error_handler]
fn alloc_error(layout: alloc::alloc::Layout) -> ! {
    panic!("Memory allocation error: {:?}", layout);
}
