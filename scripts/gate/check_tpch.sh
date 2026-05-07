#!/usr/bin/env bash
# ============================================================
# R9 / B8: TPC-H Performance Gate
#
# Runs TPC-H queries against SQLRustGo and enforces time thresholds.
# Supports SF=0.1 (alpha/beta) and SF=1 (RC/GA).
#
# Thresholds (Alpha/Beta — SF=0.1):
#   Q1:  ≤ 10.0s
#   Q6:  ≤  6.0s
#   All 22 queries must complete without OOM.
#
# Thresholds (RC/GA — SF=1):
#   Q1:  ≤ 30.0s
#   Q6:  ≤ 15.0s
#   All 22 queries must complete without OOM.
#
# Usage:
#   bash scripts/gate/check_tpch.sh              (SF=0.1, alpha/beta)
#   bash scripts/gate/check_tpch.sh sf=1         (SF=1, RC/GA)
#   bash scripts/gate/check_tpch.sh --sf1        (SF=1, RC/GA)
#   bash scripts/gate/check_tpch.sh --skip-data  (skip data check)
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TPCH_DIR="$PROJECT_ROOT/scripts/tpch"
BASELINE_FILE="$PROJECT_ROOT/perf_baselines/v3.0.0/tpch_baseline.json"
RESULT_FILE="$PROJECT_ROOT/perf_baselines/v3.0.0/tpch_current.json"

# Default TPC-H data location
DATA_DIR_DEFAULT="$HOME/sqlrustgo-tpch/data"
DATA_DIR="${TPCH_DATA_DIR:-$DATA_DIR_DEFAULT}"

cd "$PROJECT_ROOT"

SF="${TPCH_SF:-0.1}"
SKIP_DATA=false
for arg in "$@"; do
    case "$arg" in
        sf=*) SF="${arg#sf=}"; shift ;;
        --sf1) SF="1"; shift ;;
        --sf0.1) SF="0.1"; shift ;;
        --skip-data) SKIP_DATA=true; shift ;;
    esac
done

# Thresholds by SF (in milliseconds)
case "$SF" in
    0.1)
        TIME_Q1=10000; TIME_Q6=6000
        Q_COUNT=22; TABLE_COUNT=8
        ;;
    1)
        TIME_Q1=30000; TIME_Q6=15000
        Q_COUNT=22; TABLE_COUNT=8
        ;;
    *)
        echo "❌ Unsupported SF: $SF (use 0.1 or 1)"
        exit 1
        ;;
esac

echo "=== R9/B8: TPC-H Performance Gate (SF=$SF) ==="
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "Data dir: $DATA_DIR"
echo "Result file: $RESULT_FILE"
echo ""

# ---------- Step 1: Data availability ----------
if [ "$SKIP_DATA" = false ]; then
    echo "[1/4] Checking TPC-H data (SF=$SF)..."
    if [ ! -d "$DATA_DIR" ] || [ -z "$(ls -A "$DATA_DIR"/*.tbl 2>/dev/null)" ]; then
        echo "⚠️  TPC-H data not found at $DATA_DIR"
        echo "   Generate with: ~/tpch-tools/dbgen/dbgen -s $SF -f -d"
        echo "   Or set TPCH_DATA_DIR env var."
        echo ""
        echo "⏭️  TPC-H Gate: SKIPPED (no TPC-H data)"
        exit 0
    fi
    echo "[✓] TPC-H data found"
else
    echo "[1/4] Skipping data check (--skip-data)"
fi

# ---------- Step 2: Build bench CLI ----------
echo ""
echo "[2/4] Building bench CLI..."
if ! cargo build -p sqlrustgo-bench-cli --quiet 2>&1; then
    echo "❌ Failed to build sqlrustgo-bench-cli"
    exit 1
fi

# Check for bench binary
BENCH_BIN="$PROJECT_ROOT/target/release/sqlrustgo-bench-cli"
if [ ! -f "$BENCH_BIN" ]; then
    echo "❌ bench CLI not found at $BENCH_BIN"
    exit 1
fi
echo "[✓] bench CLI built"

# ---------- Step 3: Run TPC-H queries via bench CLI ----------
echo ""
echo "[3/4] Running TPC-H SF=$SF queries (22/22)..."

mkdir -p "$(dirname "$RESULT_FILE")"

# Run all 22 TPC-H queries using bench CLI
# The bench CLI handles parameter substitution automatically
BENCH_OUTPUT=$("$BENCH_BIN" tpch-bench \
    --ddl "$TPCH_DIR/tpch_schema.sql" \
    --data "$DATA_DIR" \
    --queries all \
    --iterations 1 \
    --output "$RESULT_FILE" 2>&1)

# Parse the output to show results
echo "$BENCH_OUTPUT" | grep -E "^(Q[0-9]+|TOTAL)" | while read -r line; do
    echo "  $line"
done

# Check if bench CLI encountered errors
if echo "$BENCH_OUTPUT" | grep -qi "error\|panic\|OOM\|memory"; then
    echo ""
    echo "❌ TPC-H benchmark encountered errors:"
    echo "$BENCH_OUTPUT" | grep -iE "error|panic|oom|memory" | head -5
    exit 1
fi

# Verify result file exists
if [ ! -f "$RESULT_FILE" ]; then
    echo "❌ Result file not created: $RESULT_FILE"
    exit 1
fi

# ---------- Step 4: Evaluate thresholds ----------
echo ""
echo "[4/4] Evaluating TPC-H performance (SF=$SF)..."

FAIL_COUNT=0
WARN_COUNT=0
PASS_COUNT=0

evaluate_q() {
    local name="$1"
    local threshold_ms="$2"

    local ms
    ms=$(python3 -c "
import json
try:
    with open('$RESULT_FILE') as f:
        d = json.load(f)
    for q in d.get('queries', []):
        if q.get('name') == '$name':
            print(int(q.get('avg_ms', 999999)))
            break
except: print(999999)
" 2>/dev/null || echo "999999")

    if [ "$ms" -ge 999000 ]; then
        echo "⚠️  $name: no result"
        WARN_COUNT=$((WARN_COUNT + 1))
        return
    fi

    if [ "$ms" -le "$threshold_ms" ]; then
        echo "✅ $name: ${ms}ms ≤ ${threshold_ms}ms"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        echo "❌ $name: ${ms}ms > ${threshold_ms}ms"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
}

# Evaluate key queries with thresholds
evaluate_q "Q1" "$TIME_Q1"
evaluate_q "Q6" "$TIME_Q6"

# Count total queries in result
TOTAL_QUERIES=$(python3 -c "
import json
try:
    with open('$RESULT_FILE') as f:
        d = json.load(f)
    print(len(d.get('queries', [])))
except: print('0')
" 2>/dev/null || echo "0")

echo ""
echo "=== TPC-H Gate (SF=$SF) ==="
echo "Total queries run: $TOTAL_QUERIES / $Q_COUNT"
echo "Key queries (Q1, Q6): PASS=$PASS_COUNT | WARN=$WARN_COUNT | FAIL=$FAIL_COUNT"
echo ""

# For SF=1, we primarily care about no OOM and all 22 running
# Performance thresholds are indicative for initial验收标准
if [ "$FAIL_COUNT" -gt 0 ]; then
    echo "❌ TPC-H Gate: FAILED — $FAIL_COUNT key query(s) exceeded thresholds"
    echo "   Results: $RESULT_FILE"
    exit 1
fi

if [ "$TOTAL_QUERIES" -lt "$Q_COUNT" ]; then
    echo "❌ TPC-H Gate: FAILED — only $TOTAL_QUERIES/$Q_COUNT queries ran"
    echo "   Results: $RESULT_FILE"
    exit 1
fi

echo "✅ TPC-H Gate: PASSED — all $Q_COUNT queries completed without OOM (SF=$SF)"
echo "   Results: $RESULT_FILE"
exit 0