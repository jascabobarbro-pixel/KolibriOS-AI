# Makefile for KolibriOS AI

# Toolchain
CARGO ?= cargo
RUSTC ?= rustc
CC ?= gcc
CXX ?= g++
AS ?= nasm

# Directories
KERNEL_DIR := kernel
CELLS_DIR := cells
KOLI_DIR := koli_lang
AGENT_DIR := unified_ai_agent
BUILD_DIR := build
TARGET_DIR := target

# Build targets
.PHONY: all kernel cells koli agent clean test run

all: kernel cells koli agent

kernel:
	@echo "Building kernel..."
	$(CARGO) build --package kolibrios-kernel

cells:
	@echo "Building cells..."
	$(CARGO) build --package memory_cell
	$(CARGO) build --package io_cell
	$(CARGO) build --package network_cell
	$(CARGO) build --package process_cell
	$(CARGO) build --package ai_cell

koli:
	@echo "Building Koli language..."
	$(CARGO) build --package koli-compiler
	$(CARGO) build --package koli-runtime

agent:
	@echo "Building Unified AI Agent..."
	$(CARGO) build --package unified-ai-agent

test:
	@echo "Running tests..."
	$(CARGO) test --all

test-coverage:
	@echo "Running tests with coverage..."
	$(CARGO) tarpaulin --all --out Html

doc:
	@echo "Building documentation..."
	$(CARGO) doc --no-deps --all

clean:
	@echo "Cleaning build artifacts..."
	$(CARGO) clean
	rm -rf $(BUILD_DIR)

# Build bootable ISO
iso: all
	@echo "Creating bootable ISO..."
	mkdir -p $(BUILD_DIR)/iso/boot/grub
	cp $(TARGET_DIR)/x86_64-kolibrios/release/kolibrios-kernel $(BUILD_DIR)/iso/boot/kernel
	echo 'set timeout=0\nset default=0\n\nmenuentry "KolibriOS AI" {\n    multiboot /boot/kernel\n}' > $(BUILD_DIR)/iso/boot/grub/grub.cfg
	grub-mkrescue -o $(BUILD_DIR)/kolibrios-ai.iso $(BUILD_DIR)/iso

# Run in QEMU
run: iso
	@echo "Running in QEMU..."
	qemu-system-x86_64 -cdrom $(BUILD_DIR)/kolibrios-ai.iso -m 512M

# Run in QEMU with debug
debug: iso
	@echo "Running in QEMU with GDB server..."
	qemu-system-x86_64 -cdrom $(BUILD_DIR)/kolibrios-ai.iso -m 512M -s -S

# Format code
fmt:
	$(CARGO) fmt --all
	find . -name '*.cpp' -o -name '*.h' | xargs clang-format -i

# Lint code
lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

# Security audit
audit:
	$(CARGO) audit

# Build release
release:
	$(CARGO) build --release --all

# Install dependencies
deps:
	rustup target add x86_64-unknown-none
	rustup component add rust-src
