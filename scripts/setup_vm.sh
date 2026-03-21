#!/bin/bash
#
# KolibriOS AI - QEMU Virtual Machine Setup Script
# This script sets up and launches a QEMU VM for KolibriOS AI
#
# Requirements:
#   - QEMU (qemu-system-x86_64)
#   - xorriso (for ISO creation)
#   - grub-pc-bin (for bootloader)
#
# Usage:
#   ./setup_vm.sh [--build] [--launch] [--debug]
#

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
VM_DIR="${PROJECT_ROOT}/vm"
ISO_DIR="${PROJECT_ROOT}/boot"
DISK_IMAGE="${VM_DIR}/kolibrios_ai.qcow2"
ISO_IMAGE="${ISO_DIR}/kolibrios_ai.iso"

# VM Configuration
VM_NAME="KolibriOS-AI"
VM_CPUS=2
VM_MEMORY=4096
VM_DISK_SIZE=10G

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check for required tools
check_dependencies() {
    log_info "Checking dependencies..."
    
    local missing=()
    
    if ! command -v qemu-system-x86_64 &> /dev/null; then
        missing+=("qemu-system-x86")
    fi
    
    if ! command -v qemu-img &> /dev/null; then
        missing+=("qemu-utils")
    fi
    
    if ! command -v xorriso &> /dev/null; then
        missing+=("xorriso")
    fi
    
    if ! command -v grub-mkrescue &> /dev/null; then
        missing+=("grub-pc-bin")
    fi
    
    if [ ${#missing[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing[*]}"
        log_info "Install with: sudo apt-get install ${missing[*]}"
        return 1
    fi
    
    log_success "All dependencies satisfied"
    return 0
}

# Create VM directory structure
create_vm_directories() {
    log_info "Creating VM directory structure..."
    
    mkdir -p "${VM_DIR}"
    mkdir -p "${ISO_DIR}"
    mkdir -p "${ISO_DIR}/boot/grub"
    
    log_success "VM directories created"
}

# Create QCOW2 disk image
create_disk_image() {
    log_info "Creating ${VM_DISK_SIZE} QCOW2 disk image..."
    
    if [ -f "${DISK_IMAGE}" ]; then
        log_warning "Disk image already exists, skipping creation"
        return 0
    fi
    
    qemu-img create -f qcow2 "${DISK_IMAGE}" "${VM_DISK_SIZE}"
    
    log_success "Disk image created: ${DISK_IMAGE}"
}

# Build the kernel
build_kernel() {
    log_info "Building KolibriOS AI kernel..."
    
    cd "${PROJECT_ROOT}"
    
    # Build for x86_64 target
    if cargo build --target x86_64-unknown-none --release 2>/dev/null; then
        log_success "Kernel built successfully"
        return 0
    else
        log_warning "x86_64-unknown-none target not available, building default..."
        cargo build --release
    fi
    
    log_success "Kernel build complete"
}

# Build bootable ISO
build_iso() {
    log_info "Building bootable ISO image..."
    
    local KERNEL_PATH="${PROJECT_ROOT}/target/x86_64-unknown-none/release/kolibrios-kernel"
    
    if [ ! -f "${KERNEL_PATH}" ]; then
        KERNEL_PATH="${PROJECT_ROOT}/target/release/kolibrios-kernel"
    fi
    
    if [ ! -f "${KERNEL_PATH}" ]; then
        log_error "Kernel not found. Run with --build first"
        return 1
    fi
    
    # Copy kernel to boot directory
    cp "${KERNEL_PATH}" "${ISO_DIR}/boot/kernel.bin" 2>/dev/null || \
        cp "${KERNEL_PATH}.rlib" "${ISO_DIR}/boot/kernel.rlib" 2>/dev/null || \
        log_warning "Kernel file copy issue"
    
    # Create GRUB config
    cat > "${ISO_DIR}/boot/grub/grub.cfg" << 'EOF'
set timeout=5
set default=0

menuentry "KolibriOS AI - Normal Boot" {
    multiboot /boot/kernel.bin
    boot
}

menuentry "KolibriOS AI - Debug Mode" {
    multiboot /boot/kernel.bin debug=1
    boot
}

menuentry "KolibriOS AI - Safe Mode" {
    multiboot /boot/kernel.bin safe=1
    boot
}
EOF

    # Create ISO
    grub-mkrescue -o "${ISO_IMAGE}" "${ISO_DIR}" 2>/dev/null || {
        log_warning "GRUB mkrescue failed, creating raw bootable image"
        # Create a simple bootable disk image instead
        create_raw_boot_image
        return $?
    }
    
    log_success "ISO created: ${ISO_IMAGE}"
}

# Create raw bootable image (fallback)
create_raw_boot_image() {
    log_info "Creating raw bootable disk image..."
    
    local RAW_IMAGE="${VM_DIR}/kolibrios_ai.raw"
    local KERNEL_PATH="${PROJECT_ROOT}/target/release/kolibrios-kernel"
    
    # Create 100MB raw image
    dd if=/dev/zero of="${RAW_IMAGE}" bs=1M count=100 2>/dev/null
    
    # Format as ext2
    mkfs.ext2 -F "${RAW_IMAGE}" 2>/dev/null || true
    
    # Mount and copy kernel
    local MOUNT_DIR="/tmp/kolibrios_mount"
    mkdir -p "${MOUNT_DIR}"
    
    # Copy kernel to the image (simplified approach)
    if [ -f "${KERNEL_PATH}" ]; then
        # Write kernel at offset 1MB (after boot sector area)
        dd if="${KERNEL_PATH}" of="${RAW_IMAGE}" bs=512 seek=2048 conv=notrunc 2>/dev/null
    fi
    
    log_success "Raw boot image created: ${RAW_IMAGE}"
}

# Launch QEMU VM
launch_vm() {
    log_info "Launching QEMU Virtual Machine..."
    
    local BOOT_MEDIA="${ISO_IMAGE}"
    
    if [ ! -f "${BOOT_MEDIA}" ]; then
        BOOT_MEDIA="${VM_DIR}/kolibrios_ai.raw"
    fi
    
    if [ ! -f "${BOOT_MEDIA}" ]; then
        log_error "No bootable media found. Run with --build first"
        return 1
    fi
    
    local QEMU_OPTS=(
        -name "${VM_NAME}"
        -machine q35,accel=tcg
        -cpu qemu64
        -smp "${VM_CPUS}"
        -m "${VM_MEMORY}"
        -drive file="${DISK_IMAGE}",format=qcow2,if=virtio
        -cdrom "${BOOT_MEDIA}"
        -boot d
        -netdev user,id=net0,hostfwd=tcp::2222-:22
        -device virtio-net-pci,netdev=net0
        -serial stdio
        -monitor telnet:localhost:4444,server,nowait
        -vga std
        -vnc :0
    )
    
    log_info "VM Configuration:"
    echo "  - CPUs: ${VM_CPUS}"
    echo "  - Memory: ${VM_MEMORY}MB"
    echo "  - Disk: ${DISK_IMAGE}"
    echo "  - Boot: ${BOOT_MEDIA}"
    echo "  - Serial: stdio (kernel debug output)"
    echo "  - VNC: localhost:5900"
    echo "  - Monitor: telnet localhost:4444"
    echo ""
    log_info "Starting VM (Ctrl+A, X to quit)..."
    
    qemu-system-x86_64 "${QEMU_OPTS[@]}"
}

# Launch QEMU in debug mode
launch_vm_debug() {
    log_info "Launching QEMU in debug mode..."
    
    local BOOT_MEDIA="${ISO_IMAGE}"
    [ ! -f "${BOOT_MEDIA}" ] && BOOT_MEDIA="${VM_DIR}/kolibrios_ai.raw"
    [ ! -f "${BOOT_MEDIA}" ] && BOOT_MEDIA="${DISK_IMAGE}"
    
    qemu-system-x86_64 \
        -name "${VM_NAME} Debug" \
        -machine q35,accel=tcg \
        -cpu qemu64 \
        -smp "${VM_CPUS}" \
        -m "${VM_MEMORY}" \
        -drive file="${DISK_IMAGE}",format=qcow2,if=virtio \
        -cdrom "${BOOT_MEDIA}" \
        -boot d \
        -netdev user,id=net0 \
        -device virtio-net-pci,netdev=net0 \
        -serial stdio \
        -monitor stdio \
        -d in_asm,cpu_reset \
        -D "${VM_DIR}/qemu_debug.log" \
        -S -gdb tcp::1234 \
        -vga std
    
    log_info "Debug log: ${VM_DIR}/qemu_debug.log"
}

# Print usage
print_usage() {
    echo "KolibriOS AI VM Setup Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --check      Check dependencies only"
    echo "  --build      Build kernel and create bootable image"
    echo "  --launch     Launch the VM"
    echo "  --debug      Launch VM in debug mode with GDB support"
    echo "  --all        Build and launch"
    echo "  --help       Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 --build          # Build kernel and ISO"
    echo "  $0 --launch         # Start the VM"
    echo "  $0 --all            # Build and start VM"
    echo "  $0 --debug          # Debug mode with GDB on port 1234"
}

# Main
main() {
    local action="help"
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --check)
                action="check"
                shift
                ;;
            --build)
                action="build"
                shift
                ;;
            --launch)
                action="launch"
                shift
                ;;
            --debug)
                action="debug"
                shift
                ;;
            --all)
                action="all"
                shift
                ;;
            --help|-h)
                action="help"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
        esac
    done
    
    case $action in
        check)
            check_dependencies
            ;;
        build)
            check_dependencies && \
            create_vm_directories && \
            create_disk_image && \
            build_kernel && \
            build_iso
            ;;
        launch)
            launch_vm
            ;;
        debug)
            launch_vm_debug
            ;;
        all)
            check_dependencies && \
            create_vm_directories && \
            create_disk_image && \
            build_kernel && \
            build_iso && \
            launch_vm
            ;;
        help|*)
            print_usage
            ;;
    esac
}

main "$@"
