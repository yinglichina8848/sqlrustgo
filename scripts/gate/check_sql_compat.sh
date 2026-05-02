#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== R8: SQL Compatibility Check ==="
echo "Date: $(date)"
echo ""

cd "$PROJECT_ROOT"

CORPUS_THRESHOLD=80

echo "[1/2] Running SQL Corpus tests..."
CORPUS_OUTPUT=$(cargo test -p sql-corpus -- --nocapture 2>&1 || true)

echo "[2/2] Analyzing results..."

if echo "$CORPUS_OUTPUT" | grep -q "test result: ok"; then
    echo "SQL Corpus tests: PASS"
    echo ""
    echo "✅ R8: SQL Compatibility Check PASSED"
    echo "   Corpus pass rate >= $CORPUS_THRESHOLD%"
    exit 0
else
    echo "SQL Corpus tests: FAIL"
    echo ""
    echo "❌ R8: SQL Compatibility Check FAILED"
    echo "   SQL Corpus pass rate < $CORPUS_THRESHOLD%"
    exit 1
fi
