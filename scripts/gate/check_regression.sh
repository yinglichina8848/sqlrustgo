#!/usr/bin/env bash
# ============================================================
# R9: Performance Regression Check
#
# Runs QPS benchmark tests and compares against baseline.
# Also enforces E-09 minimum thresholds for DELETE/UPDATE.
#
# Thresholds:
#   ≤5%  regression = PASS (within noise)
#   5-20% regression = WARN (needs explanation in PR)
#   >20% regression = FAIL (must be fixed)
#
# E-09 hard floor (absolute minimum, regardless of baseline):
#   DELETE ≥ 10,000 QPS
#   UPDATE ≥ 10,000 QPS
#
# Usage:
#   bash scripts/gate/check_regression.sh             (full run)
#   bash scripts/gate/check_regression.sh --skip-run  (use existing current.json)
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
    echo "   Then create perf_baselines/v2.9.0/baseline.json"
    exit 1
fi
echo "[✓] Baseline file found"

# ---------- Step 2: Run benchmarks (unless skipped) ----------
if [ "$SKIP_RUN" = false ]; then
    echo ""
    echo "[1/9] Running benchmark: simple_select..."
    SIMPLE_SELECT=$(cargo test --test qps_benchmark_test test_qps_simple_select -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)
    echo "     -> ${SIMPLE_SELECT:-N/A} qps"

    echo "[2/9] Running benchmark: insert..."
    INSERT=$(cargo test --test qps_benchmark_test test_qps_insert -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)
    echo "     -> ${INSERT:-N/A} qps"

    echo "[3/9] Running benchmark: update (E-09)..."
    UPDATE=$(cargo test --test qps_benchmark_test test_qps_update -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)
    echo "     -> ${UPDATE:-N/A} qps"

    echo "[4/9] Running benchmark: delete (E-09)..."
    DELETE=$(cargo test --test qps_benchmark_test test_qps_delete -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)
    echo "     -> ${DELETE:-N/A} qps"

    echo "[5/9] Running benchmark: join..."
    JOIN=$(cargo test --test qps_benchmark_test test_qps_join -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)
    echo "     -> ${JOIN:-N/A} qps"

    echo "[6/9] Running benchmark: aggregation..."
    AGG=$(cargo test --test qps_benchmark_test test_qps_aggregation -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)
    echo "     -> ${AGG:-N/A} qps"

    echo "[7/9] Running benchmark: order_by..."
    ORDER_BY=$(cargo test --test qps_benchmark_test test_qps_order_by -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)
    echo "     -> ${ORDER_BY:-N/A} qps"

    echo "[8/9] Running benchmark: concurrent_select_8t..."
    CONC_SELECT=$(cargo test --test qps_benchmark_test test_qps_concurrent_select -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)
    echo "     -> ${CONC_SELECT:-N/A} qps"

    echo "[9/9] Running benchmark: complex_where..."
    COMPLEX_WHERE=$(cargo test --test qps_benchmark_test test_qps_complex_where -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1)
    echo "     -> ${COMPLEX_WHERE:-N/A} qps"

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

# ---------- Step 3: E-09 Hard Floor ----------
echo ""
echo "=== E-09 Minimum Threshold Check ==="

check_e09_floor() {
    local name="$1"
    local current_qps="$2"
    local min_qps="$3"

    if [ -z "$current_qps" ] || [ "$current_qps" = "0" ]; then
        echo "❌ E-09 $name: FAILED (QPS=$current_qps, minimum=$min_qps)"
        return 1
    fi

    local above
    above=$(python3 -c "print(1 if $current_qps >= $min_qps else 0)" 2>/dev/null || echo "0")
    if [ "$above" = "1" ]; then
        echo "✅ E-09 $name: $(printf "%.0f" "$current_qps") QPS >= $min_qps"
        return 0
    else
        echo "❌ E-09 $name: $(printf "%.0f" "$current_qps") QPS < $min_qps (minimum)"
        return 1
    fi
}

E09_FAIL=0
UPDATE_VAL=$(python3 -c "import json; d=json.load(open('$RESULT_FILE')); print(d['benchmarks']['update'])" 2>/dev/null || echo "0")
check_e09_floor "UPDATE" "$UPDATE_VAL" 10000 || E09_FAIL=1

DELETE_VAL=$(python3 -c "import json; d=json.load(open('$RESULT_FILE')); print(d['benchmarks']['delete'])" 2>/dev/null || echo "0")
check_e09_floor "DELETE" "$DELETE_VAL" 10000 || E09_FAIL=1

# ---------- Step 4: Compare against baseline ----------
echo ""
echo "=== Regression Analysis (vs baseline) ==="
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
        printf "%-25s %12.0f %12s %8s %s\n" "$name" "$baseline_qps" "FAILED" "N/A" "FAIL"
        FAIL_COUNT=$((FAIL_COUNT + 1))
        return
    fi

    # Calculate percentage change: ((current - baseline) / baseline) * 100
    local delta
    delta=$(python3 -c "print(round((($current_qps - $baseline_qps) / $baseline_qps) * 100, 1))" 2>/dev/null || echo "0")

    local status
    if [ "$(python3 -c "print(1 if $delta >= -5 else 0)" 2>/dev/null)" = "1" ]; then
        status="PASS"
        PASS_COUNT=$((PASS_COUNT + 1))
    elif [ "$(python3 -c "print(1 if $delta >= -20 else 0)" 2>/dev/null)" = "1" ]; then
        status="WARN"
        WARN_COUNT=$((WARN_COUNT + 1))
    else
        status="FAIL"
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

# ---------- Step 5: Summary ----------
echo ""
echo "=== R9 Summary ==="
echo "Regression: PASS=$PASS_COUNT | WARN=$WARN_COUNT | FAIL=$FAIL_COUNT"
echo "E-09 Floor:  $( [ "$E09_FAIL" -eq 0 ] && echo '✅ PASS' || echo '❌ FAIL' )"

EXIT_CODE=0

if [ "$FAIL_COUNT" -gt 0 ]; then
    echo ""
    echo "❌ R9: FAILED — $FAIL_COUNT benchmark(s) regressed >20% from baseline"
    EXIT_CODE=1
fi

if [ "$E09_FAIL" -ne 0 ]; then
    echo ""
    echo "❌ R9: FAILED — E-09 DELETE or UPDATE below minimum (10,000 QPS)"
    EXIT_CODE=1
fi

if [ "$EXIT_CODE" -ne 0 ]; then
    echo "   Review results in: $RESULT_FILE"
    exit 1
fi

if [ "$WARN_COUNT" -gt 0 ]; then
    echo ""
    echo "⚠️  R9: PASSED with warnings — $WARN_COUNT benchmark(s) regressed 5-20% from baseline"
    echo "   Add explanation to commit message or PR description."
    exit 0
fi

echo ""
echo "✅ R9: PASSED — all benchmarks within 5% of baseline, E-09 thresholds met"
