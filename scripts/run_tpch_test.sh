#!/bin/bash
# Run TPC-H compliance test with memory limit
# Usage: ./scripts/run_tpch_test.sh [--sf001|--tiny]

set -e

# Memory limit: 8GB in KB (8 * 1024 * 1024)
MEMORY_LIMIT_KB=8388608

# Parse arguments
TEST_MODE="${1:-sf001}"

if [ "$TEST_MODE" = "--tiny" ]; then
    echo "Running TPC-H test with TINY dataset..."
    export TBL_DATA_DIR="data/tpch-tiny"
    export TBL_SQLITE_DB="data/tpch-tiny/tpch.db"
elif [ "$TEST_MODE" = "--sf001" ]; then
    echo "Running TPC-H test with SF=0.1 dataset..."
    export TBL_DATA_DIR="data/tpch-sf001"
    export TBL_SQLITE_DB="data/tpch-sf001/tpch.db"
else
    echo "Unknown mode: $TEST_MODE"
    echo "Usage: $0 [--sf001|--tiny]"
    exit 1
fi

# Clean up any existing test processes
pkill -9 -f "tpch_compliance_test" 2>/dev/null || true
sleep 1

# Run test with timeout (10 minutes max) and memory limit via ulimit
# Note: ulimit -v limits virtual memory, not physical RSS
# For true memory protection, use tools like `ulimit -m` (macOS not supported)
echo "Starting test with memory limit ${MEMORY_LIMIT_KB}KB..."
echo "Timeout: 600 seconds (10 minutes)"

# Create a subshell with limits
(
    # Set virtual memory limit (soft limit)
    ulimit -v $MEMORY_LIMIT_KB || {
        echo "Warning: Could not set memory limit (ulimit -v failed)"
        echo "Running without memory limit..."
    }

    # Set CPU time limit (5 minutes per query)
    ulimit -t 600 || true

    cargo test --test tpch_compliance_test test_tpch_q1_simple -- --nocapture --test-threads=1 2>&1
)

EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    echo ""
    echo "=========================================="
    echo "TPC-H compliance test PASSED!"
    echo "=========================================="
elif [ $EXIT_CODE -eq 137 ] || [ $EXIT_CODE -eq 143 ]; then
    echo ""
    echo "=========================================="
    echo "TPC-H compliance test TIMEOUT or KILLED"
    echo "(exit code: $EXIT_CODE)"
    echo "=========================================="
elif [ $EXIT_CODE -eq 9 ]; then
    echo ""
    echo "=========================================="
    echo "TPC-H compliance test KILLED (SIGKILL)"
    echo "Likely exceeded memory limit"
    echo "=========================================="
else
    echo ""
    echo "=========================================="
    echo "TPC-H compliance test FAILED (exit code: $EXIT_CODE)"
    echo "=========================================="
fi

exit $EXIT_CODE
