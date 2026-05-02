#!/usr/bin/env bash
# TLA+ Model Check Script
# Part of SQLRustGo E2E Formal Verification Workflow
#
# Usage: ./tla-check.sh <module-name> [config-file]
# Example: ./tla-check.sh WAL_Recovery

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

MODULE_NAME="${1:-}"
CONFIG_FILE="${2:-}"

if [ -z "$MODULE_NAME" ]; then
    echo "Usage: $0 <module-name> [config-file]"
    echo "Example: $0 WAL_Recovery"
    exit 1
fi

echo "=== Running TLA+ Model Checking ==="
echo "Module: $MODULE_NAME"
echo "Date: $(date)"
echo ""

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo "❌ Error: Docker not installed"
    echo "   Required for TLA+ Toolbox"
    exit 1
fi

# Try to run TLA+ Toolbox
echo "[1/2] Running TLA+ model check..."
docker run --rm -v "$PROJECT_ROOT:/workspace" -w /workspace \
    tlatools/tlatools \
    bash -c "tlc -workers auto -modelcheck $MODULE_NAME" 2>&1 || {
    # Fallback: try tlc directly if available
    if command -v tlc &> /dev/null; then
        tlc -workers auto -modelcheck "$MODULE_NAME" 2>&1
    else
        echo "⚠️ TLA+ Toolbox not available"
        echo "   Install: docker pull tlatools/tlatools"
        echo "   Or run manually: docker run --rm -v \$(pwd):/workspace tlatools/tlatools"
        exit 1
    fi
}

echo "[2/2] Checking results..."

# TLA+ model check passes if no error
if [ $? -eq 0 ]; then
    echo ""
    echo "✅ TLA+ model check PASSED"
    exit 0
else
    echo "❌ TLA+ model check FAILED"
    exit 1
fi