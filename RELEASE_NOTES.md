# KolibriOS AI - Release v0.6.0

## Overview

KolibriOS AI is a revolutionary microkernel-based operating system with native AI integration. This release includes a complete kernel, living cell architecture, AI-native programming language, and virtual machine support.

---

## 🎉 Major Features

### Core System
- ✅ **Living Kernel** - Microkernel with genes (Process, Memory, I/O)
- ✅ **Neural Scheduler** - ML-based task scheduling (Feed-Forward Network)
- ✅ **Security System** - Capability-based access control, sandboxing

### Living Cell Architecture
- ✅ **Memory Cell** - Autonomous memory management with gRPC
- ✅ **Processor Cell** - Task scheduling and CPU management
- ✅ **I/O Cell** - Device abstraction
- ✅ **Network Cell** - Networking stack
- ✅ **AI Cell** - AI inference capabilities

### AI Integration
- ✅ **Unified Mind** - Central AI agent with LLM integration
- ✅ **Gemini API Client** - Google Gemini integration
- ✅ **Local Llama Support** - Offline AI capabilities

### Programming Language
- ✅ **Koli Language** - AI-native programming language
  - Lexer with AI keywords (ai, ask, cell, spawn)
  - Parser with cell definitions
  - Code generator (Rust, LLVM, Wasm, Bytecode)

### Applications
- ✅ **Adaptive GUI** - Iced-based adaptive interface
- ✅ **File Manager** - Context-aware file manager
- ✅ **Creative Assistant** - LLM-powered creative tool

### Virtual Machine
- ✅ **QEMU Support** - Complete VM setup scripts
- ✅ **Bootable ISO** - Multiboot2 compliant bootloader

---

## 📊 Statistics

| Metric | Value |
|--------|-------|
| Total Files | 621+ |
| Lines of Code | 40,000+ |
| Rust Code | 27,555 lines |
| Python Code | 7,435 lines |
| Test Files | 19 |
| Tests Passed | 107 |
| Components | 17+ |

---

## 🧪 Test Results

### Passed: 107 tests
- CND Orchestrator: 18/18 ✅
- Unified Mind Core: 15/15 ✅
- Security Tests: 20/20 ✅
- Mock Tests: 54/54 ✅

### Needs Dependencies: 33 tests
- grpc module required
- google-generativeai required
- llama-cpp-python required

---

## 🚀 Quick Start

### Clone and Build

```bash
git clone https://github.com/jascabobarbro-pixel/KolibriOS-AI.git
cd KolibriOS-AI
make all
```

### Run Tests

```bash
# Python tests
pytest

# Rust tests (requires Cargo)
cargo test
```

### Launch VM

```bash
# Install QEMU first
sudo apt-get install qemu-system-x86 qemu-utils xorriso grub-pc-bin

# Build and launch
./scripts/setup_vm.sh --all
```

---

## 📁 Project Structure

```
KolibriOS-AI/
├── kernel/           # Microkernel (Rust)
├── cells/            # Living cells (Rust + gRPC)
├── apps/             # GUI and applications
├── koli_lang/        # Koli language compiler
├── unified_ai_agent/ # Unified Mind AI agent
├── cnd_orchestrator/ # Central Neural Device
├── scripts/          # Build and VM scripts
├── boot/             # Bootloader assembly
└── docs/             # Documentation
```

---

## 📖 Documentation

- [AI Agents Guide](AI_AGENTS_GUIDE.md) - Guide for external AI agents
- [Contributing](CONTRIBUTING_AGENTS.md) - Contribution guidelines
- [VM Setup](docs/vm_setup/README.md) - Virtual machine setup
- [Build Log](BUILD_LOG.md) - Development history

---

## 🔧 Requirements

### Runtime
- Rust 1.70+ (for kernel and cells)
- Python 3.10+ (for orchestrator and AI)
- QEMU 7.0+ (for VM testing)

### Optional
- google-generativeai (for Gemini API)
- llama-cpp-python (for local LLM)

---

## 🤝 Contributing

See [CONTRIBUTING_AGENTS.md](CONTRIBUTING_AGENTS.md) for guidelines.

---

## 📜 License

MIT License - See [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

- Inspired by KolibriOS
- Microkernel design from MINIX and seL4
- AI architecture from modern LLM systems

---

**Repository**: https://github.com/jascabobarbro-pixel/KolibriOS-AI
