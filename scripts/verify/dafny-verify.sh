#!/usr/bin/env bash
# Dafny Formal Verification Script
# Part of SQLRustGo E2E Formal Verification Workflow
# 
# Usage: ./dafny-verify.sh <path-to-dafny-file>
# Example: ./dafny-verify.sh docs/proof/PROOF-011-type-safety.dfy

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

DAFNY_FILE="${1:-}"
OUTPUT_FILE=""

if [ -z "$DAFNY_FILE" ]; then
    echo "Usage: $0 <path-to-dafny-file>"
    echo "Example: $0 docs/proof/PROOF-011-type-safety.dfy"
    exit 1
fi

# Resolve to absolute path
if [[ "$DAFNY_FILE" != /* ]]; then
    DAFNY_FILE="$PROJECT_ROOT/$DAFNY_FILE"
fi

if [ ! -f "$DAFNY_FILE" ]; then
    echo "❌ Error: File not found: $DAFNY_FILE"
    exit 1
fi

# Generate output filename
OUTPUT_FILE="${DAFNY_FILE%.dfy}.verify Output"

echo "=== Running Dafny Verification ==="
echo "File: $DAFNY_FILE"
echo "Date: $(date)"
echo ""

# Check if dafny is installed
if ! command -v dafny &> /dev/null; then
    echo "❌ Error: Dafny not installed"
    echo "   Install with: dotnet tool install -g Dafny"
    exit 1
fi

# Run Dafny verification
echo "[1/2] Running dafny verify..."
dafny verify "$DAFNY_FILE" > "$OUTPUT_FILE" 2>&1 || {
    echo "❌ Dafny verification FAILED"
    echo ""
    echo "=== Verification Output ==="
    cat "$OUTPUT_FILE"
    exit 1
}

echo "[2/2] Parsing results..."

# Check for errors in output
if grep -q "Dafny program verifier.*0 errors" "$OUTPUT_FILE"; then
    echo ""
    echo "✅ Dafny verification PASSED"
    echo "   Output: $OUTPUT_FILE"
    exit 0
elif grep -q "Errors: 0" "$OUTPUT_FILE"; then
    echo ""
    echo "✅ Dafny verification PASSED"
    echo "   Output: $OUTPUT_FILE"
    exit 0
else
    echo "❌ Dafny verification FAILED"
    echo ""
    echo "=== Verification Output ==="
    cat "$OUTPUT_FILE"
    exit 1
fi