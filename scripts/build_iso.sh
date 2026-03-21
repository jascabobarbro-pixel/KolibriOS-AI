#!/bin/bash
#
# KolibriOS AI - Build Bootable ISO
# Creates a bootable ISO image from the kernel
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"
ISO_DIR="${BUILD_DIR}/iso"
BOOT_DIR="${ISO_DIR}/boot"
GRUB_DIR="${BOOT_DIR}/grub"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${BLUE}[BUILD]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

log "Building KolibriOS AI bootable image..."

# Create build directories
mkdir -p "${BOOT_DIR}/grub"
mkdir -p "${BUILD_DIR}/kernel"

# Build kernel (no_std for bare metal)
log "Building kernel for x86_64 bare metal..."

cd "${PROJECT_ROOT}"

# Try to build with x86_64-unknown-none target first
if rustup target list | grep -q "x86_64-unknown-none (installed)"; then
    cargo build --target x86_64-unknown-none --release
    KERNEL="${PROJECT_ROOT}/target/x86_64-unknown-none/release/kolibrios-kernel"
else
    warn "x86_64-unknown-none target not installed"
    log "Building with default target..."
    cargo build --release
    KERNEL="${PROJECT_ROOT}/target/release/kolibrios-kernel"
fi

if [ -f "${KERNEL}" ]; then
    cp "${KERNEL}" "${BOOT_DIR}/kernel.bin"
    success "Kernel copied to boot directory"
elif [ -f "${KERNEL}.bin" ]; then
    cp "${KERNEL}.bin" "${BOOT_DIR}/kernel.bin"
    success "Kernel binary copied"
else
    # Create a minimal bootable kernel stub
    warn "Kernel binary not found, creating boot stub..."
    create_boot_stub
fi

# Create GRUB configuration
log "Creating GRUB configuration..."

cat > "${GRUB_DIR}/grub.cfg" << 'EOF'
#
# KolibriOS AI GRUB Configuration
#

set timeout=5
set default=0

# Set graphics mode
set gfxpayload=1024x768x32

# Main entry
menuentry "KolibriOS AI - Normal Boot" {
    echo "Loading KolibriOS AI kernel..."
    multiboot /boot/kernel.bin
    echo "Booting..."
    boot
}

# Debug entry
menuentry "KolibriOS AI - Debug Mode" {
    echo "Loading KolibriOS AI kernel (debug)..."
    multiboot /boot/kernel.bin debug=1 console=serial
    boot
}

# Safe mode
menuentry "KolibriOS AI - Safe Mode" {
    echo "Loading KolibriOS AI kernel (safe mode)..."
    multiboot /boot/kernel.bin safe=1 nomodeset
    boot
}

# Memory test entry
menuentry "KolibriOS AI - Memory Test" {
    multiboot /boot/kernel.bin memtest=1
    boot
}

# Reboot
menuentry "Reboot" {
    reboot
}

# Shutdown
menuentry "Shutdown" {
    halt
}
EOF

success "GRUB configuration created"

# Create multiboot header assembly stub (for testing)
create_boot_stub() {
    local STUB_ASM="${BUILD_DIR}/boot_stub.asm"
    
    cat > "${STUB_ASM}" << 'EOF'
; KolibriOS AI Boot Stub
; Minimal multiboot2 compliant bootloader

BITS 32

; Multiboot2 header
section .multiboot
align 8
header_start:
    dd 0xE85250D6                ; magic
    dd 0                          ; architecture (i386)
    dd header_end - header_start  ; header length
    dd 0x100000000 - (0xE85250D6 + 0 + (header_end - header_start)) ; checksum
    
    ; End tag
    dw 0    ; type
    dw 0    ; flags
    dd 8    ; size
header_end:

; Text section
section .text
global _start
extern kernel_main

_start:
    ; Set up stack
    mov esp, stack_top
    
    ; Clear screen
    mov dword [0xB8000], 0x0F4B0F4B  ; "KK" in white on black
    mov dword [0xB8004], 0x0F4B0F4B
    
    ; Print boot message
    mov esi, boot_msg
    call print_string
    
    ; Call kernel
    ; call kernel_main
    
    ; Halt
    cli
.halt:
    hlt
    jmp .halt

; Print string function
print_string:
    mov edi, 0xB8000 + 160  ; Second line
.loop:
    lodsb
    test al, al
    jz .done
    mov ah, 0x0F
    stosw
    jmp .loop
.done:
    ret

; Data
section .rodata
boot_msg: db "KolibriOS AI - Kernel Loading...", 0

; BSS
section .bss
align 16
stack_bottom:
    resb 16384  ; 16 KB stack
stack_top:
EOF

    # Assemble if nasm available
    if command -v nasm &> /dev/null; then
        nasm -f elf32 "${STUB_ASM}" -o "${BUILD_DIR}/boot_stub.o"
        success "Boot stub assembled"
    fi
}

# Build ISO with GRUB
log "Creating ISO image..."

OUTPUT_ISO="${BUILD_DIR}/kolibrios_ai.iso"

if command -v grub-mkrescue &> /dev/null; then
    grub-mkrescue -o "${OUTPUT_ISO}" "${ISO_DIR}"
    success "ISO created: ${OUTPUT_ISO}"
else
    warn "grub-mkrescue not found, creating raw disk image instead"
    
    # Create raw disk image
    RAW_IMAGE="${BUILD_DIR}/kolibrios_ai.raw"
    dd if=/dev/zero of="${RAW_IMAGE}" bs=1M count=64 status=progress
    
    # Write kernel at 1MB offset
    if [ -f "${BOOT_DIR}/kernel.bin" ]; then
        dd if="${BOOT_DIR}/kernel.bin" of="${RAW_IMAGE}" bs=512 seek=2048 conv=notrunc
    fi
    
    success "Raw image created: ${RAW_IMAGE}"
fi

# Summary
echo ""
success "Build complete!"
echo ""
echo "Output files:"
ls -lh "${BUILD_DIR}/"*.iso 2>/dev/null || ls -lh "${BUILD_DIR}/"*.raw 2>/dev/null || echo "  Check ${BUILD_DIR}/"
echo ""
echo "To run: ./scripts/launch_vm.sh"
