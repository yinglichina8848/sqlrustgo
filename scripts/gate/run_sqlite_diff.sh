#!/bin/bash
# SQLite Differential Testing Integration
# Compares SQLRustGo results with SQLite reference implementation
# 
# CI Integration:
#   - Run after: cargo build --all-features
#   - Pass criteria: 0 diffs OR all diffs are documented
#   - Output: JSON report for CI gate

set -euo pipefail

OUTPUT_FILE="${1:-test_results/sqlite_diff_report.json}"
SQLITE="${SQLITE:-sqlite3}"
TEMP_DIR=$(mktemp -d)

cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

echo "=== SQLite Differential Testing ==="
echo "Output: $OUTPUT_FILE"

# Test categories
PASS=0
FAIL=0
SKIP=0

# Create output structure
cat > "$OUTPUT_FILE" << 'EOF'
{
  "timestamp": "",
  "total": 0,
  "passed": 0,
  "failed": 0,
  "skipped": 0,
  "diffs": []
}
EOF

# Test function - compare results
# Args: $1=SQL, $2=test_name
run_diff_test() {
    local sql="$1"
    local name="$2"
    
    # Skip non-SELECT
    if ! echo "$sql" | grep -qi "^SELECT"; then
        echo "SKIP: $name (non-SELECT)"
        ((SKIP++)) || true
        return
    fi
    
    # Run SQLite
    local sqlite_result
    sqlite_result=$("$SQLITE" :memory: "$sql" 2>/dev/null || echo "ERROR")
    
    # Run SQLRustGo
    local srg_result
    srg_result=$(cargo run --bin sqlrustgo -- -q "$sql" 2>/dev/null || echo "ERROR")
    
    # Compare (normalize)
    sqlite_result=$(echo "$sqlite_result" | sort | tr '\n' ' ')
    srg_result=$(echo "$srg_result" | sort | tr '\n' ' ')
    
    if [[ "$sqlite_result" == "$srg_result" ]]; then
        echo "PASS: $name"
        ((PASS++)) || true
    else
        echo "FAIL: $name"
        echo "  SQLite: $sqlite_result"
        echo "  SQLRustGo: $srg_result"
        ((FAIL++)) || true
    fi
}

# Run corpus tests
echo ""
echo "Running core SQL tests..."
CORPUS_DIR="sql_corpus/DML/SELECT"
if [[ -d "$CORPUS_DIR" ]]; then
    for sql in "$CORPUS_DIR"/*.sql; do
        name=$(basename "$sql" .sql)
        # Extract first SELECT
        first_select=$(grep -m1 "^SELECT" "$sql" || echo "")
        if [[ -n "$first_select" ]]; then
            run_diff_test "$first_select" "$name"
        fi
    done
fi

# Summary
echo ""
echo "=== Summary ==="
echo "Passed: $PASS"
echo "Failed: $FAIL" 
echo "Skipped: $SKIP"
echo "Total: $((PASS+FAIL+SKIP))"

# Update JSON
timestamp=$(date -Iseconds)
jq ".timestamp = \"$timestamp\" | .total = $((PASS+FAIL+SKIP)) | .passed = $PASS | .failed = $FAIL | .skipped = $SKIP" "$OUTPUT_FILE" > "${OUTPUT_FILE}.tmp"
mv "${OUTPUT_FILE}.tmp" "$OUTPUT_FILE"

# Exit code: 0 if all pass, 1 if any fail
if [[ "$FAIL" -gt 0 ]]; then
    echo "❌ FAIL: $FAIL tests differ from SQLite"
    exit 1
else
    echo "✅ PASS: All tests match SQLite"
    exit 0
fi