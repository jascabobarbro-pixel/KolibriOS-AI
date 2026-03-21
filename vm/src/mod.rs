//! Virtual Machine Module for KolibriOS AI.
//!
//! Provides QEMU-based virtualization for running KolibriOS AI
//! in isolated virtual machines with hardware acceleration.

pub mod qemu;
pub mod vmm;
pub mod device;
pub mod memory;
pub mod cpu;

pub use qemu::*;
pub use vmm::*;
pub use device::*;
pub use memory::*;
pub use cpu::*;
