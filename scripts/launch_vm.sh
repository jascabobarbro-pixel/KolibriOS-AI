#!/bin/bash
#
# KolibriOS AI - QEMU VM Launcher
# Quick launch script for the KolibriOS AI virtual machine
#

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Configuration
VM_DIR="${PROJECT_ROOT}/vm"
ISO_IMAGE="${PROJECT_ROOT}/boot/kolibrios_ai.iso"
DISK_IMAGE="${VM_DIR}/kolibrios_ai.qcow2"

# Default values
CPUS=${VM_CPUS:-2}
MEMORY=${VM_MEMORY:-4096}
VNC_PORT=${VM_VNC:-0}

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  KolibriOS AI Virtual Machine${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Check for boot media
BOOT_MEDIA=""
if [ -f "${ISO_IMAGE}" ]; then
    BOOT_MEDIA="${ISO_IMAGE}"
    echo -e "${BLUE}[INFO]${NC} Using ISO: ${ISO_IMAGE}"
elif [ -f "${VM_DIR}/kolibrios_ai.raw" ]; then
    BOOT_MEDIA="${VM_DIR}/kolibrios_ai.raw"
    echo -e "${BLUE}[INFO]${NC} Using raw image: ${BOOT_MEDIA}"
elif [ -f "${DISK_IMAGE}" ]; then
    BOOT_MEDIA="${DISK_IMAGE}"
    echo -e "${BLUE}[INFO]${NC} Using disk image: ${BOOT_MEDIA}"
else
    echo "[ERROR] No bootable media found!"
    echo "Run: ./scripts/setup_vm.sh --build"
    exit 1
fi

echo ""
echo "VM Configuration:"
echo "  CPUs:    ${CPUS}"
echo "  Memory:  ${MEMORY} MB"
echo "  VNC:     :${VNC_PORT} (port 590${VNC_PORT})"
echo "  Serial:  stdio (kernel output)"
echo ""

# QEMU command
exec qemu-system-x86_64 \
    -name "KolibriOS-AI" \
    -machine q35 \
    -cpu qemu64 \
    -smp "${CPUS}" \
    -m "${MEMORY}" \
    -drive file="${DISK_IMAGE}",format=qcow2,if=virtio \
    -cdrom "${BOOT_MEDIA}" \
    -boot d \
    -netdev user,id=net0,hostfwd=tcp::2222-:22 \
    -device virtio-net-pci,netdev=net0 \
    -serial stdio \
    -vga std \
    -vnc ":${VNC_PORT}"
