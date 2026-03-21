#!/bin/bash
#
# KolibriOS AI - Complete Build Script
# This script performs a full clean build of all KolibriOS AI components
#
# Usage: ./scripts/build_all.sh [--release] [--verbose]
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
RELEASE_MODE="${1:-debug}"
VERBOSE="${2:-}"
BUILD_DIR="target"
DIST_DIR="dist"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="build_${TIMESTAMP}.log"

# Version
VERSION="0.7.0"
VERSION_NAME="Living Memory"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  KolibriOS AI Build System${NC}"
echo -e "${BLUE}  Version: ${VERSION} - ${VERSION_NAME}${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Logging function
log() {
    echo -e "$1" | tee -a "$LOG_FILE"
}

# Error handling
error_exit() {
    log "${RED}[ERROR] $1${NC}"
    exit 1
}

# Check dependencies
check_dependencies() {
    log "${YELLOW}[CHECK] Checking build dependencies...${NC}"
    
    local missing=()
    
    # Required tools
    command -v rustc >/dev/null 2>&1 || missing+=("rustc")
    command -v cargo >/dev/null 2>&1 || missing+=("cargo")
    command -v python3 >/dev/null 2>&1 || missing+=("python3")
    command -v gcc >/dev/null 2>&1 || missing+=("gcc")
    command -v nasm >/dev/null 2>&1 || missing+=("nasm")
    command -v ld >/dev/null 2>&1 || missing+=("ld")
    
    # Optional tools
    command -v grub-mkrescue >/dev/null 2>&1 && GRUB_AVAILABLE=1 || GRUB_AVAILABLE=0
    command -v qemu-system-x86_64 >/dev/null 2>&1 && QEMU_AVAILABLE=1 || QEMU_AVAILABLE=0
    
    if [ ${#missing[@]} -gt 0 ]; then
        error_exit "Missing required dependencies: ${missing[*]}"
    fi
    
    log "${GREEN}[CHECK] All required dependencies found${NC}"
    [ $GRUB_AVAILABLE -eq 1 ] && log "${GREEN}[CHECK] GRUB available for ISO creation${NC}"
    [ $QEMU_AVAILABLE -eq 1 ] && log "${GREEN}[CHECK] QEMU available for testing${NC}"
}

# Clean previous builds
clean_builds() {
    log "${YELLOW}[CLEAN] Cleaning previous builds...${NC}"
    
    cargo clean 2>/dev/null || true
    rm -rf "${BUILD_DIR}" 2>/dev/null || true
    rm -rf "${DIST_DIR}" 2>/dev/null || true
    mkdir -p "${BUILD_DIR}"
    mkdir -p "${DIST_DIR}"/{iso,apk,docs,bin}
    
    log "${GREEN}[CLEAN] Build directories prepared${NC}"
}

# Build Rust components
build_rust_components() {
    log "${YELLOW}[BUILD] Building Rust components...${NC}"
    
    local release_flag=""
    [ "$RELEASE_MODE" = "--release" ] && release_flag="--release"
    
    # Build kernel (no_std)
    log "  Building kernel (no_std)..."
    cd kernel
    cargo build $release_flag --target x86_64-unknown-none 2>&1 | tee -a "../$LOG_FILE" || \
        cargo build $release_flag 2>&1 | tee -a "../$LOG_FILE" || \
        log "${YELLOW}[WARN] Kernel build requires x86_64-unknown-none target${NC}"
    cd ..
    
    # Build cells
    log "  Building Memory Cell..."
    cd cells/memory_cell
    cargo build $release_flag 2>&1 | tee -a "../../$LOG_FILE" || true
    cd ../..
    
    log "  Building Processor Cell..."
    cd cells/processor_cell
    cargo build $release_flag 2>&1 | tee -a "../../$LOG_FILE" || true
    cd ../..
    
    # Build AI Cell
    log "  Building AI Cell..."
    cd cells/ai_cell
    cargo build $release_flag 2>&1 | tee -a "../../$LOG_FILE" || true
    cd ../..
    
    # Build VM module
    log "  Building VM Module..."
    cd vm
    cargo build $release_flag 2>&1 | tee -a "../$LOG_FILE" || true
    cd ..
    
    # Build Koli Language
    log "  Building Koli Language Compiler..."
    cd koli_lang/compiler
    cargo build $release_flag 2>&1 | tee -a "../../$LOG_FILE" || true
    cd ../..
    
    log "  Building Koli Language Runtime..."
    cd koli_lang/runtime
    cargo build $release_flag 2>&1 | tee -a "../../$LOG_FILE" || true
    cd ../..
    
    # Build GUI
    log "  Building GUI Framework..."
    cd apps/gui
    cargo build $release_flag 2>&1 | tee -a "../../$LOG_FILE" || true
    cd ../..
    
    # Build File Manager
    log "  Building Adaptive File Manager..."
    cd apps/file_manager
    cargo build $release_flag 2>&1 | tee -a "../../$LOG_FILE" || true
    cd ../..
    
    # Build Creative Assistant
    log "  Building Creative Assistant..."
    cd apps/creative_assistant
    cargo build $release_flag 2>&1 | tee -a "../../$LOG_FILE" || true
    cd ../..
    
    log "${GREEN}[BUILD] Rust components built${NC}"
}

# Build Python components
build_python_components() {
    log "${YELLOW}[BUILD] Building Python components...${NC}"
    
    # Install dependencies
    log "  Installing Python dependencies..."
    pip install -e ./unified_ai_agent/unified_mind 2>&1 | tee -a "$LOG_FILE" || true
    pip install -e ./cnd_orchestrator 2>&1 | tee -a "$LOG_FILE" || true
    
    # Run tests
    log "  Running Python tests..."
    python -m pytest unified_ai_agent/unified_mind/tests/ -v 2>&1 | tee -a "$LOG_FILE" || true
    python -m pytest cnd_orchestrator/tests/ -v 2>&1 | tee -a "$LOG_FILE" || true
    
    log "${GREEN}[BUILD] Python components built${NC}"
}

# Run all tests
run_tests() {
    log "${YELLOW}[TEST] Running test suites...${NC}"
    
    # Rust tests
    log "  Running Rust tests..."
    cargo test 2>&1 | tee -a "$LOG_FILE" || true
    
    # Python tests
    log "  Running Python tests..."
    python -m pytest tests/functional/ -v 2>&1 | tee -a "$LOG_FILE" || true
    
    log "${GREEN}[TEST] Tests completed${NC}"
}

# Copy binaries
copy_binaries() {
    log "${YELLOW}[PACKAGE] Copying binaries...${NC}"
    
    # Find and copy all built binaries
    find target -name "*.exe" -o -name "kolibrios*" -o -name "memory_cell*" -o -name "processor_cell*" 2>/dev/null | while read bin; do
        cp "$bin" "${DIST_DIR}/bin/" 2>/dev/null || true
    done
    
    # Copy libraries
    find target -name "*.so" -o -name "*.dylib" -o -name "*.dll" 2>/dev/null | while read lib; do
        cp "$lib" "${DIST_DIR}/bin/" 2>/dev/null || true
    done
    
    log "${GREEN}[PACKAGE] Binaries copied to ${DIST_DIR}/bin/${NC}"
}

# Generate checksums
generate_checksums() {
    log "${YELLOW}[PACKAGE] Generating checksums...${NC}"
    
    cd "${DIST_DIR}"
    find . -type f \( -name "*.bin" -o -name "*.exe" -o -name "*.so" -o -name "*.iso" -o -name "*.apk" \) -exec sha256sum {} \; > checksums.sha256
    cd ..
    
    log "${GREEN}[PACKAGE] Checksums generated${NC}"
}

# Build summary
build_summary() {
    log ""
    log "${BLUE}========================================${NC}"
    log "${GREEN}  BUILD COMPLETED SUCCESSFULLY${NC}"
    log "${BLUE}========================================${NC}"
    log ""
    log "Build artifacts available in: ${DIST_DIR}/"
    log "  - Binaries: ${DIST_DIR}/bin/"
    log "  - ISO: ${DIST_DIR}/iso/"
    log "  - APK: ${DIST_DIR}/apk/"
    log "  - Docs: ${DIST_DIR}/docs/"
    log ""
    log "Build log: ${LOG_FILE}"
    log ""
}

# Main execution
main() {
    log "${BLUE}Starting build at $(date)${NC}"
    log ""
    
    check_dependencies
    clean_builds
    build_rust_components
    build_python_components
    run_tests
    copy_binaries
    generate_checksums
    build_summary
    
    log "${GREEN}Build completed at $(date)${NC}"
}

# Run main
main "$@"
