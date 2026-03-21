# KolibriOS AI - VM Setup Guide

This document provides complete instructions for setting up a QEMU-based virtual machine to run and test KolibriOS AI.

---

## Prerequisites

### Software Requirements

| Package | Version | Purpose |
|---------|---------|---------|
| QEMU | 7.0+ | Virtual Machine |
| xorriso | 1.5+ | ISO creation |
| grub-pc-bin | latest | Bootloader |
| mtools | latest | Disk tools |
| nasm | 2.5+ | Assembly |
| Rust | 1.70+ | Kernel compilation |

### Installation

#### Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install -y qemu-system-x86 qemu-utils xorriso grub-pc-bin grub-common mtools nasm
```

#### Fedora

```bash
sudo dnf install -y qemu-system-x86 qemu-img xorriso grub2-pc nasm
```

#### Arch Linux

```bash
sudo pacman -S qemu-system-x86 qemu-img xorriso grub nasm
```

### Rust Setup

Add the x86_64 bare metal target:

```bash
rustup target add x86_64-unknown-none
rustup toolchain install nightly
rustup target add x86_64-unknown-none --toolchain nightly
```

---

## Quick Start

### 1. Build

```bash
# Clone and Build
git clone https://github.com/jascabobarbro-pixel/KolibriOS-AI.git
cd KolibriOS-AI
make all
```

### 2. Launch VM

```bash
# Quick launch
./scripts/launch_vm.sh

# Or with custom settings
VM_CPUS=4 VM_MEMORY=8192 ./scripts/launch_vm.sh
```

### 3. Debug Mode

```bash
# Launch with GDB debugging
./scripts/setup_vm.sh --debug

# Connect GDB
gdb
(gdb) target remote localhost:1234
```

---

## VM Configuration

The Parameter | Default | Description |
| --------- | ------- | ----------- |
| CPUs | 2 | Number of virtual CPU cores |
| Memory | 4GB | RAM allocated to VM |
| Disk | 10GB | QCOW2 disk image size |
| VNC Port | 5900 | VNC display port |
| Serial | stdio | Kernel debug output |
| Monitor | 4444 | QEMU monitor port |

---

## Files Reference

| File | Purpose |
| ---- | ------- |
| `scripts/setup_vm.sh` | Full VM setup script |
| `scripts/launch_vm.sh` | Quick launch script |
| `scripts/build_iso.sh` | ISO build script |
| `boot/kolibritos_ai.iso` | Bootable ISO image |
| `vm/kolibrios_ai.qcow2` | QCOW2 disk image |

---

## Testing

### Automated Tests

Run from the project root:

```bash
# Run all tests
make test

# Or run specific VM tests
pytest tests/vm_tests.py -v
```

### Manual Testing Checklist

- [ ] VM boots successfully
- [ ] Kernel outputs to serial console
- [ ] Cells start responding
- [ ] CND orchestrator coordinates cells
- [ ] Unified Mind accepts commands
- [ ] Network connectivity works
- [ ] Disk I/O functions correctly
- [ ] Memory allocation works
- [ ] Self-healing triggers correctly

---

## Troubleshooting

### Common Issues

1. **QEMU not found**
   - Install: `sudo apt-get install qemu-system-x86`

2. **Kernel won't boot**
   - Check serial output for errors
   - Verify kernel binary is correct format
   - Ensure GRUB configuration is correct

3. **Network not working**
   - Check QEMU user-mode networking
   - Try: `-netdev user,id=net0 -device e1000,netdev=net0`

4. **VNC connection refused**
   - Try different VNC port: `-vnc :1`
   - Check firewall settings

### Debug Tips

1. Enable verbose kernel output:
   ```bash
   QEMU_DEBUG=1 ./scripts/launch_vm.sh
   ```

2. Monitor QEMU log:
   ```bash
   tail -f vm/qemu.log
   ```

3. Use QEMU monitor:
   ```bash
   telnet localhost 4444
   info registers
   ```

---

## License

MIT License - See LICENSE file for details.
