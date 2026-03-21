#!/bin/bash
#
# KolibriOS AI - Kernel Test Runner
# Runs tests and verifies functionality
#

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"

echo "======================================"
echo "  KolibriOS AI - Test Runner"
echo "======================================"
echo ""

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name=$1
    local test_cmd=$2
    
    echo -n "Running: ${test_name}... "
    
    if eval "${test_cmd}"; then
        echo "  ✓ PASSED"
        ((TESTS_PASSED++))
        return 0
    else
        echo "  ✗ FAILED"
        ((TESTS_FAILED++))
        return 1
    fi
}

echo "=== Building Project ==="
cargo build --release 2>&1 | head -20

echo ""
echo "=== Running Rust Tests ==="
cargo test 2>&1 | tail -30
TEST_RESULT=$?

if [ $TEST_RESULT -eq 0 ]; then
    echo "✓ All Rust tests passed"
    ((TESTS_PASSED++))
else
    echo "✗ Some Rust tests failed"
    ((TESTS_FAILED++))
fi

echo ""
echo "=== Running Python Tests ==="
if command -v pytest &> /dev/null; then
    pytest "${PROJECT_ROOT}" -v --tb=short 2>&1 | tail -30
    PYTEST_RESULT=$?
    
    if [ $PYTEST_RESULT -eq 0 ]; then
        echo "✓ All Python tests passed"
        ((TESTS_PASSED++))
    else
        echo "✗ Some Python tests failed"
        ((TESTS_FAILED++))
    fi
else
    echo "⚠ pytest not installed, skipping Python tests"
fi

echo ""
echo "=== Checking Code Quality ==="

# Check for TODO/FIXME/HACK
TODOS=$(grep -r "TODO\|FIXME\|HACK\|XXX" "${PROJECT_ROOT}" --include="*.rs,*.py" 2>/dev/null | grep -c "^[^#]" | wc -l || true)

if [ -z "$TODOS" ]; then
    echo "✓ No TODO/FIXME/HACK comments found"
else
    echo "⚠ Found TODO/FIXME/HACK comments:"
    echo "$TODOS"
fi

# Check for unwrap
UNWRAPS=$(grep -r "\.unwrap()" "${PROJECT_ROOT}" --include="*.rs" 2>/dev/null | wc -l || true)

if [ -z "$UNWRAPS" ]; then
    echo "✓ No .unwrap() calls found"
else
    echo "⚠ Found .unwrap() calls:"
    echo "$UNWRAPS"
fi

echo ""
echo "=== Test Summary ==="
echo "Passed: ${TESTS_PASSED}"
echo "Failed: ${TESTS_FAILED}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo "✓ All tests passed!"
    exit 0
else
    echo "✗ Some tests failed"
    exit 1
fi
