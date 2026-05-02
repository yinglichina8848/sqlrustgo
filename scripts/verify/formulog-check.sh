#!/usr/bin/env bash
# Formulog Logic Check Script
# Part of SQLRustGo E2E Formal Verification Workflow
#
# Usage: ./formulog-check.sh <path-to-formulog-file>
# Example: ./formulog-check.sh docs/proof/PROOF-014-query-equivalence.formulog

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

FORMULOG_FILE="${1:-}"

if [ -z "$FORMULOG_FILE" ]; then
    echo "Usage: $0 <path-to-formulog-file>"
    echo "Example: $0 docs/proof/PROOF-014-query-equivalence.formalog"
    exit 1
fi

# Resolve to absolute path
if [[ "$FORMULOG_FILE" != /* ]]; then
    FORMULOG_FILE="$PROJECT_ROOT/$FORMULOG_FILE"
fi

if [ ! -f "$FORMULOG_FILE" ]; then
    echo "❌ Error: File not found: $FORMULOG_FILE"
    exit 1
fi

echo "=== Running Formulog Check ==="
echo "File: $FORMULOG_FILE"
echo "Date: $(date)"
echo ""

# Check if formulog is installed
if ! command -v formulog &> /dev/null; then
    echo "⚠️ Error: Formulog not installed"
    echo "   Install with: pip install formulog"
    echo "   Note: Formulog check may require additional setup"
    exit 1
fi

# Run Formulog check
echo "[1/2] Running formulog check..."
formulog check "$FORMULOG_FILE" 2>&1 || {
    echo "❌ Formulog check FAILED"
    exit 1
}

echo "[2/2] Checking results..."

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Formulog check PASSED"
    exit 0
else
    echo "❌ Formulog check FAILED"
    exit 1
fi