//! Architecture-specific code

#[cfg(target_arch = "x86_64")]
mod x86_64;

/// Enable CPU interrupts
pub fn enable_interrupts() {
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("sti"); }
    #[cfg(target_arch = "aarch64")]
    unsafe { core::arch::asm!("msr daifclr, #2"); }
}

/// Disable CPU interrupts
pub fn disable_interrupts() {
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("cli"); }
    #[cfg(target_arch = "aarch64")]
    unsafe { core::arch::asm!("msr daifset, #2"); }
}

/// Halt the CPU
pub fn halt() {
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("hlt"); }
    #[cfg(target_arch = "aarch64")]
    unsafe { core::arch::asm!("wfi"); }
}

/// Get CPU ID
pub fn cpu_id() -> u32 { 0 }
