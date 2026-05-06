#!/usr/bin/env bash
# ============================================================
# R9: Performance Regression Check
#
# Runs QPS benchmark tests and compares against baseline.
# Thresholds (from baseline.json):
#   ≤5%  = PASS (within noise)
#   5-20%  = WARN (needs explanation)
#   >20% = FAIL (regression)
#
# Usage:
#   bash scripts/gate/check_regression.sh
#   bash scripts/gate/check_regression.sh --skip-run  (use existing results)
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BASELINE_FILE="$PROJECT_ROOT/perf_baselines/v2.9.0/baseline.json"
RESULT_FILE="$PROJECT_ROOT/perf_baselines/v2.9.0/current.json"

cd "$PROJECT_ROOT"

SKIP_RUN=false
if [[ "${1:-}" == "--skip-run" ]]; then
    SKIP_RUN=true
fi

echo "=== R9: Performance Regression Check ==="
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "Baseline: $BASELINE_FILE"
echo ""

# ---------- Step 1: Baseline existence ----------
if [ ! -f "$BASELINE_FILE" ]; then
    echo "❌ R9: FAILED — baseline file not found at $BASELINE_FILE"
    echo "   Run: cargo test --test qps_benchmark_test -- --ignored"
    echo "   Then manually create perf_baselines/v2.9.0/baseline.json"
    exit 1
fi

echo "[✓] Baseline file found"

# ---------- Step 2: Run benchmarks (unless skipped) ----------
if [ "$SKIP_RUN" = false ]; then
    echo ""
    echo "[1/9] Running benchmark: simple_select..."
    SIMPLE_SELECT=$(cargo test --test qps_benchmark_test test_qps_simple_select -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)

    echo "[2/9] Running benchmark: insert..."
    INSERT=$(cargo test --test qps_benchmark_test test_qps_insert -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)

    echo "[3/9] Running benchmark: update..."
    UPDATE=$(cargo test --test qps_benchmark_test test_qps_update -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)

    echo "[4/9] Running benchmark: delete..."
    DELETE=$(cargo test --test qps_benchmark_test test_qps_delete -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)

    echo "[5/9] Running benchmark: join..."
    JOIN=$(cargo test --test qps_benchmark_test test_qps_join -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)

    echo "[6/9] Running benchmark: aggregation..."
    AGG=$(cargo test --test qps_benchmark_test test_qps_aggregation -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)

    echo "[7/9] Running benchmark: order_by..."
    ORDER_BY=$(cargo test --test qps_benchmark_test test_qps_order_by -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)

    echo "[8/9] Running benchmark: concurrent_select_8t..."
    CONC_SELECT=$(cargo test --test qps_benchmark_test test_qps_concurrent_select -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)

    echo "[9/9] Running benchmark: complex_where..."
    COMPLEX_WHERE=$(cargo test --test qps_benchmark_test test_qps_complex_where -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)

    # Write current results
    cat > "$RESULT_FILE" << JSONEOF
{
  "date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "benchmarks": {
    "simple_select": ${SIMPLE_SELECT:-0},
    "insert": ${INSERT:-0},
    "update": ${UPDATE:-0},
    "delete": ${DELETE:-0},
    "join": ${JOIN:-0},
    "aggregation": ${AGG:-0},
    "order_by": ${ORDER_BY:-0},
    "concurrent_select_8t": ${CONC_SELECT:-0},
    "complex_where": ${COMPLEX_WHERE:-0}
  }
}
JSONEOF
else
    echo ""
    echo "[skip] Using existing results from $RESULT_FILE"
fi

# ---------- Step 3: Compare against baseline ----------
echo ""
echo "=== Regression Analysis ==="
printf "%-25s %12s %12s %8s %s\n" "Benchmark" "Baseline" "Current" "Δ%" "Status"
printf "%-25s %12s %12s %8s %s\n" "-------------------------" "------------" "------------" "--------" "------"

FAIL_COUNT=0
WARN_COUNT=0
PASS_COUNT=0

compare_benchmark() {
    local name="$1"
    local baseline_qps="$2"
    local current_qps="$3"

    if [ "$baseline_qps" = "null" ] || [ "$baseline_qps" = "0" ] || [ -z "$baseline_qps" ]; then
        printf "%-25s %12s %12.0f %8s %s\n" "$name" "N/A" "$current_qps" "N/A" "NEW"
        return
    fi

    if [ "$current_qps" = "0" ] || [ -z "$current_qps" ]; then
        printf "%-25s %12.0f %12s %8s %s\n" "$name" "$baseline_qps" "FAILED" "N/A" "❌ FAIL"
        FAIL_COUNT=$((FAIL_COUNT + 1))
        return
    fi

    # Calculate percentage change: ((current - baseline) / baseline) * 100
    # Positive = improvement, negative = regression
    local delta
    delta=$(python3 -c "print(round((($current_qps - $baseline_qps) / $baseline_qps) * 100, 1))" 2>/dev/null || echo "0")

    # Regression check (negative delta means slower than baseline)
    local regression_pct
    regression_pct=$(python3 -c "print(abs(min(0, $delta)))" 2>/dev/null || echo "0")

    local status
    if [ "$(python3 -c "print(1 if $delta >= -5 else 0)" 2>/dev/null)" = "1" ]; then
        status="✅ PASS"
        PASS_COUNT=$((PASS_COUNT + 1))
    elif [ "$(python3 -c "print(1 if $delta >= -20 else 0)" 2>/dev/null)" = "1" ]; then
        status="⚠️  WARN"
        WARN_COUNT=$((WARN_COUNT + 1))
    else
        status="❌ FAIL"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi

    printf "%-25s %12.0f %12.0f %7.0f%% %s\n" "$name" "$baseline_qps" "$current_qps" "$delta" "$status"
}

# Extract baseline values and compare
for bench in simple_select insert update delete join aggregation order_by concurrent_select_8t complex_where; do
    baseline_val=$(python3 -c "import json; d=json.load(open('$BASELINE_FILE')); print(d['benchmarks']['$bench']['qps'])" 2>/dev/null || echo "null")
    current_val=$(python3 -c "import json; d=json.load(open('$RESULT_FILE')); print(d['benchmarks']['$bench'])" 2>/dev/null || echo "0")
    compare_benchmark "$bench" "$baseline_val" "$current_val"
done

# ---------- Step 4: Summary ----------
echo ""
echo "=== R9 Summary ==="
echo "PASS: $PASS_COUNT | WARN: $WARN_COUNT | FAIL: $FAIL_COUNT"

if [ "$FAIL_COUNT" -gt 0 ]; then
    echo ""
    echo "❌ R9: FAILED — $FAIL_COUNT benchmark(s) regressed >20%"
    echo "   Review results in: $RESULT_FILE"
    exit 1
fi

if [ "$WARN_COUNT" -gt 0 ]; then
    echo ""
    echo "⚠️  R9: PASSED with warnings — $WARN_COUNT benchmark(s) regressed 5-20%"
    echo "   Add explanation to commit message or PR description."
    exit 0
fi

echo ""
echo "✅ R9: PASSED — all benchmarks within 5% of baseline"
