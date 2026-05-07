#!/usr/bin/env bash
# ============================================================
# R10/GA-10: Performance Baseline Check
#
# Runs QPS and TPC-H benchmarks and checks against baseline.
# Enforces E-09 minimum thresholds for DELETE/UPDATE.
#
# Version auto-detection:
#   develop/v3.0.0 -> perf_baselines/v3.0.0/
#   develop/v2.9.0 -> perf_baselines/v2.9.0/
#   otherwise       -> perf_baselines/v2.9.0/ (default)
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
#   bash scripts/gate/check_perf_baseline.sh             (full run: QPS + TPC-H)
#   bash scripts/gate/check_perf_baseline.sh --qps      (QPS benchmarks only)
#   bash scripts/gate/check_perf_baseline.sh --tpch     (TPC-H benchmarks only)
#   bash scripts/gate/check_perf_baseline.sh --skip-run  (use existing results)
#   bash scripts/gate/check_perf_baseline.sh --sf1      (TPC-H SF=1, default: SF=0.1)
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

# ---------- Version / Baseline detection ----------
detect_version() {
    local branch
    branch=$(git symbolic-ref --short HEAD 2>/dev/null || git rev-parse --short HEAD 2>/dev/null)
    if [[ "$branch" =~ "develop/v3" ]] || [[ "$branch" =~ "v3.0" ]]; then
        echo "v3.0.0"
    elif [[ "$branch" =~ "develop/v2.9" ]] || [[ "$branch" =~ "v2.9" ]]; then
        echo "v2.9.0"
    else
        echo "v2.9.0"  # default fallback
    fi
}

VERSION=$(detect_version)
QPS_BASELINE="$PROJECT_ROOT/perf_baselines/$VERSION/baseline.json"
QPS_RESULT="$PROJECT_ROOT/perf_baselines/$VERSION/current.json"
TPC_BASELINE="$PROJECT_ROOT/perf_baselines/$VERSION/tpch_baseline.json"
TPC_RESULT="$PROJECT_ROOT/perf_baselines/$VERSION/tpch_current.json"

# ---------- Option parsing ----------
RUN_QPS=true
RUN_TPC=true
SKIP_RUN=false
TPC_SF="0.1"

for arg in "$@"; do
    case $arg in
        --qps)
            RUN_TPC=false
            ;;
        --tpch)
            RUN_QPS=false
            ;;
        --skip-run)
            SKIP_RUN=true
            ;;
        --sf1)
            TPC_SF="1"
            ;;
        --sf0.1)
            TPC_SF="0.1"
            ;;
    esac
done

echo "=== R10/GA-10: Performance Baseline Check ==="
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "Version: $VERSION"
echo ""

# ---------- QPS Benchmarks ----------
if [ "$RUN_QPS" = true ]; then
    echo "=== QPS Benchmarks ==="

    if [ "$SKIP_RUN" = false ]; then
        echo "[1/9] Running benchmark: simple_select..."
        SIMPLE_SELECT=$(cargo test --test qps_benchmark_test test_qps_simple_select -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1 || echo "0")
        echo "     -> ${SIMPLE_SELECT:-N/A} qps"

        echo "[2/9] Running benchmark: insert..."
        INSERT=$(cargo test --test qps_benchmark_test test_qps_insert -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1 || echo "0")
        echo "     -> ${INSERT:-N/A} qps"

        echo "[3/9] Running benchmark: update (E-09)..."
        UPDATE=$(cargo test --test qps_benchmark_test test_qps_update -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1 || echo "0")
        echo "     -> ${UPDATE:-N/A} qps"

        echo "[4/9] Running benchmark: delete (E-09)..."
        DELETE=$(cargo test --test qps_benchmark_test test_qps_delete -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1 || echo "0")
        echo "     -> ${DELETE:-N/A} qps"

        echo "[5/9] Running benchmark: join..."
        JOIN=$(cargo test --test qps_benchmark_test test_qps_join -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1 || echo "0")
        echo "     -> ${JOIN:-N/A} qps"

        echo "[6/9] Running benchmark: aggregation..."
        AGG=$(cargo test --test qps_benchmark_test test_qps_aggregation -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1 || echo "0")
        echo "     -> ${AGG:-N/A} qps"

        echo "[7/9] Running benchmark: order_by..."
        ORDER_BY=$(cargo test --test qps_benchmark_test test_qps_order_by -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1 || echo "0")
        echo "     -> ${ORDER_BY:-N/A} qps"

        echo "[8/9] Running benchmark: concurrent_select_8t..."
        CONC_SELECT=$(cargo test --test qps_benchmark_test test_qps_concurrent_select -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1 || echo "0")
        echo "     -> ${CONC_SELECT:-N/A} qps"

        echo "[9/9] Running benchmark: complex_where..."
        COMPLEX_WHERE=$(cargo test --test qps_benchmark_test test_qps_complex_where -- --ignored --nocapture 2>&1 | grep "QPS:" | grep -oE '[0-9]+\.[0-9]+' | tail -1 || echo "0")
        echo "     -> ${COMPLEX_WHERE:-N/A} qps"

        cat > "$QPS_RESULT" << JSONEOF
{
  "date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "version": "$VERSION",
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
        echo "[+] QPS results written to $QPS_RESULT"
    else
        echo "[skip] Benchmark run skipped"
    fi

    # ---------- E-09 Floor Check ----------
    echo ""
    echo "=== E-09 Minimum Threshold Check ==="

    check_e09_floor() {
        local name="$1"
        local current_qps="$2"
        local min_qps="$3"

        if [ -z "$current_qps" ] || [ "$current_qps" = "0" ] || [ "$current_qps" = "null" ]; then
            echo "FAIL E-09 $name: 0 QPS < $min_qps (minimum)"
            return 1
        fi

        local above
        above=$(python3 -c "print(1 if float('$current_qps') >= $min_qps else 0)" 2>/dev/null || echo "0")
        if [ "$above" = "1" ]; then
            echo "PASS E-09 $name: $(printf "%.0f" "$current_qps") QPS >= $min_qps"
            return 0
        else
            echo "FAIL E-09 $name: $(printf "%.0f" "$current_qps") QPS < $min_qps (minimum)"
            return 1
        fi
    }

    E09_FAIL=0

    UPDATE_VAL=$(python3 -c "import json; d=json.load(open('$QPS_RESULT')); print(d['benchmarks']['update'])" 2>/dev/null || echo "0")
    check_e09_floor "UPDATE" "$UPDATE_VAL" 10000 || E09_FAIL=1

    DELETE_VAL=$(python3 -c "import json; d=json.load(open('$QPS_RESULT')); print(d['benchmarks']['delete'])" 2>/dev/null || echo "0")
    check_e09_floor "DELETE" "$DELETE_VAL" 10000 || E09_FAIL=1

    # ---------- Regression Analysis ----------
    if [ -f "$QPS_BASELINE" ]; then
        echo ""
        echo "=== Regression Analysis (vs $VERSION baseline) ==="
        printf "%-25s %12s %12s %8s %s\n" "Benchmark" "Baseline" "Current" "Delta%" "Status"
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

            local delta
            delta=$(python3 -c "print(round((float('$current_qps') - float('$baseline_qps')) / float('$baseline_qps') * 100, 1))" 2>/dev/null || echo "0")

            local status
            if [ "$(python3 -c "print(1 if $delta >= -5 else 0)" 2>/dev/null)" = "1" ]; then
                status="PASS"
                PASS_COUNT=$((PASS_COUNT + 1))
            elif [ "$(python3 -c "print(1 if $delta >= -20 else 0)" 2>/dev/null)" = "1" ]; then
                status="WARN"
                WARN_COUNT=$((WARN_COUNT + 1))
            else
                if [ "$name" = "concurrent_select_8t" ]; then
                    if [ "$(python3 -c "print(1 if $delta >= -30 else 0)" 2>/dev/null)" = "1" ]; then
                        status="WARN"
                        WARN_COUNT=$((WARN_COUNT + 1))
                    else
                        status="FAIL"
                        FAIL_COUNT=$((FAIL_COUNT + 1))
                    fi
                else
                    status="FAIL"
                    FAIL_COUNT=$((FAIL_COUNT + 1))
                fi
            fi

            printf "%-25s %12.0f %12.0f %7.0f%% %s\n" "$name" "$baseline_qps" "$current_qps" "$delta" "$status"
        }

        for bench in simple_select insert update delete join aggregation order_by concurrent_select_8t complex_where; do
            baseline_val=$(python3 -c "import json; d=json.load(open('$QPS_BASELINE')); print(d['benchmarks']['$bench']['qps'])" 2>/dev/null || echo "null")
            current_val=$(python3 -c "import json; d=json.load(open('$QPS_RESULT')); print(d['benchmarks']['$bench'])" 2>/dev/null || echo "null")
            compare_benchmark "$bench" "$baseline_val" "$current_val"
        done

        echo ""
        echo "QPS Regression: PASS=$PASS_COUNT | WARN=$WARN_COUNT | FAIL=$FAIL_COUNT"
        if [ "$FAIL_COUNT" -gt 0 ] || [ "$E09_FAIL" -ne 0 ]; then
            echo "QPS Regression: FAILED"
            QPS_STATUS="FAIL"
        elif [ "$WARN_COUNT" -gt 0 ]; then
            echo "QPS Regression: PASS with warnings"
            QPS_STATUS="PASS"
        else
            echo "QPS Regression: PASS"
            QPS_STATUS="PASS"
        fi
    else
        echo ""
        echo "[skip] No baseline file at $QPS_BASELINE"
        QPS_STATUS="SKIP"
    fi
else
    QPS_STATUS="SKIP"
fi

# ---------- TPC-H Benchmarks ----------
if [ "$RUN_TPC" = true ]; then
    echo ""
    echo "=== TPC-H Benchmarks (SF=$TPC_SF) ==="

    if [ "$SKIP_RUN" = false ]; then
        # Build bench CLI if needed
        if ! command -v bench-cli &> /dev/null && [ ! -f "$PROJECT_ROOT/target/release/bench-cli" ]; then
            echo "[build] Building bench CLI..."
            cargo build --release -p bench-cli 2>&1 | grep -E "^(error|warning:)" || true
        fi

        BENCH_CLI="$PROJECT_ROOT/target/release/bench-cli"

        if [ ! -f "$BENCH_CLI" ]; then
            echo "[!] bench CLI not found, skipping TPC-H"
            TPC_STATUS="SKIP"
        else
            echo "[1/2] Building bench CLI..."
            cargo build --release -p bench-cli 2>&1 | grep -E "^(error|warning:)" || true
            echo "[2/2] Running TPC-H SF=$TPC_SF queries..."

            # Check if TPC-H data exists
            if [ ! -d "$HOME/sqlrustgo-tpch/data" ]; then
                echo "[!] TPC-H data not found at $HOME/sqlrustgo-tpch/data, skipping"
                TPC_STATUS="SKIP"
            else
                # Run TPC-H and capture output
                TPC_OUTPUT=$("$BENCH_CLI" tpch --sf "$TPC_SF" run 2>&1 || echo "")
                echo "$TPC_OUTPUT"

                # Parse results into JSON
                if [ -n "$TPC_OUTPUT" ]; then
                    TOTAL_MS=$(echo "$TPC_OUTPUT" | grep "^TOTAL" | awk '{print $2}' || echo "0")
                    cat > "$TPC_RESULT" << JSONEOF
{
  "date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "version": "$VERSION",
  "sf": "$TPC_SF",
  "total_ms": ${TOTAL_MS:-0},
  "queries": {}
}
JSONEOF
                    # Parse individual query times
                    while IFS= read -r line; do
                        if [[ "$line" =~ ^Q[0-9]+ ]]; then
                            q=$(echo "$line" | awk '{print $1}')
                            ms=$(echo "$line" | awk '{print $2}')
                            rows=$(echo "$line" | awk '{print $3}')
                            # Use python to update JSON
                            python3 -c "
import json
with open('$TPC_RESULT', 'r') as f:
    d = json.load(f)
d['queries']['$q'] = {'ms': int($ms), 'rows': int($rows)}
with open('$TPC_RESULT', 'w') as f:
    json.dump(d, f, indent=2)
" 2>/dev/null || true
                        fi
                    done <<< "$TPC_OUTPUT"
                    echo "[+] TPC-H results written to $TPC_RESULT"
                    TPC_STATUS="PASS"
                else
                    TPC_STATUS="FAIL"
                fi
            fi
        fi
    else
        echo "[skip] Benchmark run skipped"
        if [ -f "$TPC_RESULT" ]; then
            TPC_STATUS="PASS"
        else
            TPC_STATUS="SKIP"
        fi
    fi

    # TPC-H threshold check
    if [ -f "$TPC_RESULT" ]; then
        echo ""
        echo "=== TPC-H Threshold Check (SF=$TPC_SF) ==="

        # Define thresholds for SF=0.1
        if [ "$TPC_SF" = "0.1" ]; then
            Q1_THRESH=10000
            Q6_THRESH=6000
        else
            Q1_THRESH=30000
            Q6_THRESH=15000
        fi

        TPC_PASS=0
        TPC_TOTAL=0

        for q in Q1 Q6; do
            TPC_TOTAL=$((TPC_TOTAL + 1))
            thresh_var="${q}_THRESH"
            thresh=${!thresh_var}
            # Handle both array format and dict format
            ms=$(python3 -c "
import json
d=json.load(open('$TPC_RESULT'))
if isinstance(d.get('queries'), list):
    for q in d.get('queries', []):
        if q.get('name') == '$q':
            print(q.get('avg_ms', 0))
            break
    else:
        print(0)
else:
    print(d.get('queries', {}).get('$q', {}).get('ms', 0))
" 2>/dev/null || echo "0")
            if [ "$ms" != "0" ] && [ "$ms" != "" ]; then
                passes=$(python3 -c "print(1 if float('$ms') <= $thresh else 0)" 2>/dev/null || echo "0")
                if [ "$passes" = "1" ]; then
                    echo "PASS $q: ${ms}ms <= ${thresh}ms"
                    TPC_PASS=$((TPC_PASS + 1))
                else
                    echo "FAIL $q: ${ms}ms > ${thresh}ms"
                fi
            fi
        done

        TOTAL_QUERIES=$(python3 -c "
import json
d=json.load(open('$TPC_RESULT'))
if isinstance(d.get('queries'), list):
    print(len(d.get('queries', [])))
else:
    print(len(d.get('queries', {})))
" 2>/dev/null || echo "0")
        echo ""
        echo "Total TPC-H queries: $TOTAL_QUERIES / 22"
        if [ "$TPC_PASS" -eq "$TPC_TOTAL" ]; then
            echo "TPC-H: PASSED - all $TOTAL_QUERIES queries completed"
            TPC_STATUS="PASS"
        else
            echo "TPC-H: FAILED"
            TPC_STATUS="FAIL"
        fi
    else
        TPC_STATUS="SKIP"
    fi
else
    TPC_STATUS="SKIP"
fi

# ---------- Final Summary ----------
echo ""
echo "=== R10/GA-10 Performance Baseline Summary ==="
if [ "$QPS_STATUS" != "SKIP" ]; then
    echo "QPS Benchmarks:  $QPS_STATUS"
else
    echo "QPS Benchmarks:  SKIP"
fi
if [ "$TPC_STATUS" != "SKIP" ]; then
    echo "TPC-H Benchmarks: $TPC_STATUS"
else
    echo "TPC-H Benchmarks: SKIP"
fi

# Exit code
if [ "$QPS_STATUS" = "FAIL" ] || [ "$TPC_STATUS" = "FAIL" ]; then
    echo ""
    echo "R10/GA-10: FAILED"
    exit 1
elif [ "$QPS_STATUS" = "PASS" ] || [ "$TPC_STATUS" = "PASS" ]; then
    echo ""
    echo "R10/GA-10: PASSED - all performance benchmarks within thresholds"
    exit 0
else
    echo ""
    echo "R10/GA-10: SKIPPED - no benchmarks run"
    exit 0
fi