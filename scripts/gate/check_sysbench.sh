#!/usr/bin/env bash
# ============================================================
# Sysbench Gate — v3.0.0 Alpha/Beta Performance Baseline
#
# Runs sysbench oltp_read_write against SQLRustGo and enforces
# minimum QPS thresholds per workload type.
#
# NOTE: Current thresholds are calibrated for MySQL protocol overhead.
# The bottleneck is in the protocol layer (flush() per packet), not execution.
# See docs/releases/v3.1.0/MYSQL_PROTOCOL_OPTIMIZATION.md for details.
#
# Thresholds (Alpha):
#   point_select:     ≥ 2,000 QPS  (实测 1,688 + 20% buffer)
#   oltp_read_write:  ≥ 100 QPS    (实测 71 + 40% buffer)
#   oltp_write_only:  ≥ 250 QPS    (实测 190 + 30% buffer)
#   update_index:     ≥ 500 QPS    (实测 468 + 7% buffer)
#
# Target thresholds (after MySQL protocol optimization):
#   point_select:     ≥ 30,000 QPS
#   oltp_read_write:  ≥ 10,000 QPS
#   oltp_write_only:  ≥  8,000 QPS
#   update_index:     ≥  8,000 QPS
#
# Usage:
#   bash scripts/gate/check_sysbench.sh              (auto-detect phase)
#   bash scripts/gate/check_sysbench.sh --alpha      (alpha thresholds)
#   bash scripts/gate/check_sysbench.sh --beta       (beta thresholds)
#   bash scripts/gate/check_sysbench.sh --skip-run   (use cached results)
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
RESULT_FILE="$PROJECT_ROOT/perf_baselines/v3.0.0/sysbench_current.json"

cd "$PROJECT_ROOT"

PHASE="${1:-auto}"
SKIP_RUN=false
if [[ "${1:-}" == "--skip-run" ]]; then
    SKIP_RUN=true
    PHASE="auto"
fi

# Auto-detect phase from branch
detect_phase() {
    local branch
    branch=$(git symbolic-ref --short HEAD 2>/dev/null || git rev-parse --short HEAD 2>/dev/null)
    if [[ "$branch" =~ alpha ]]; then
        echo "alpha"
    elif [[ "$branch" =~ beta|develop/v3 ]]; then
        echo "beta"
    else
        echo "alpha"
    fi
}

if [[ "$PHASE" == "auto" ]]; then
    PHASE=$(detect_phase)
fi

echo "=== Sysbench Gate (v3.0.0, phase=$PHASE) ==="
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# Thresholds per phase
case "$PHASE" in
    alpha)
        # 当前阈值基于实测数据 + buffer，允许协议瓶颈存在时通过
        THRESHOLDS='{"point_select":2000,"oltp_read_write":100,"oltp_write_only":250,"update_index":500}'
        ;;
    beta)
        # Beta 阶段阈值收紧，但仍考虑协议开销
        THRESHOLDS='{"point_select":3000,"oltp_read_write":150,"oltp_write_only":350,"update_index":700}'
        ;;
    *)
        echo "❌ Unknown phase: $PHASE"
        exit 1
        ;;
esac

echo "Thresholds (phase=$PHASE):"
echo "$THRESHOLDS" | python3 -c "import sys,json; d=json.load(sys.stdin); [print(f'  {k}: ≥{v:,} QPS') for k,v in d.items()]"
echo ""

# ---------- Step 1: Build ----------
if [ "$SKIP_RUN" = false ]; then
    echo "[1/5] Building SQLRustGo (release)..."
    if ! cargo build --release --quiet 2>&1; then
        echo "❌ Build failed"
        exit 1
    fi
    echo "[✓] Build complete"
fi

# ---------- Step 2: Start SQLRustGo server ----------
echo ""
echo "[2/5] Starting SQLRustGo server on port 15995..."

# Kill any existing instance
lsof -ti:15995 | xargs kill -9 2>/dev/null || true
sleep 1

SQLRUSTGO_BIN="$PROJECT_ROOT/target/release/sqlrustgo-mysql-server"

if [ ! -f "$SQLRUSTGO_BIN" ]; then
    echo "❌ sqlrustgo-mysql-server binary not found at $SQLRUSTGO_BIN"
    echo "   Build with: cargo build --release -p sqlrustgo-mysql-server"
    exit 1
fi

# Start server in background
$SQLRUSTGO_BIN --port 15995 &
SQLRUSTGO_PID=$!
sleep 3

# Verify server is up
if ! kill -0 "$SQLRUSTGO_PID" 2>/dev/null; then
    echo "❌ SQLRustGo server failed to start"
    exit 1
fi

cleanup() {
    echo ""
    echo "[cleanup] Stopping SQLRustGo server (PID=$SQLRUSTGO_PID)..."
    kill "$SQLRUSTGO_PID" 2>/dev/null || true
    lsof -ti:15995 | xargs kill -9 2>/dev/null || true
}
trap cleanup EXIT

echo "[✓] Server running on port 15995"

# ---------- Step 3: Run sysbench benchmarks ----------
echo ""
echo "[3/5] Running sysbench benchmarks..."

# Find sysbench scripts
SYSBENCH_LUA="/opt/homebrew/share/sysbench"
if [ ! -d "$SYSBENCH_LUA" ]; then
    SYSBENCH_LUA="/usr/share/sysbench"
fi

if ! command -v sysbench &>/dev/null; then
    echo "⚠️  sysbench not installed, using internal bench-cli"
    USE_INTERNAL_BENCH=true
else
    USE_INTERNAL_BENCH=false
fi

BENCHMARK_RESULTS="{}"

if [ "$USE_INTERNAL_BENCH" = true ]; then
    # Use internal bench-cli when external sysbench is not available
    echo "[✓] Using internal bench-cli for benchmarks"

    # Build bench-cli if needed
    cargo build -p sqlrustgo-bench-cli --quiet 2>/dev/null || true

    BENCH_CLI="$PROJECT_ROOT/target/release/sqlrustgo-bench-cli"
    if [ ! -f "$BENCH_CLI" ]; then
        BENCH_CLI="$PROJECT_ROOT/target/debug/sqlrustgo-bench-cli"
    fi

    run_internal_bench() {
        local name="$1"
        local workload="$2"
        local threads="${3:-4}"
        local duration="${4:-10}"

        echo -n "  $name (threads=$threads, duration=${duration}s) ... "

        local output
        output=$($BENCH_CLI oltp --workload="$workload" --threads="$threads" --duration="$duration" 2>&1 || echo "FAILED")

        local qps
        qps=$(echo "$output" | grep -oE 'queries: [0-9]+' | head -1 | grep -oE '[0-9]+' || echo "0")
        local tps
        tps=$(echo "$output" | grep -oE 'tps: [0-9]+' | head -1 | grep -oE '[0-9]+' || echo "0")

        if [ "$qps" = "0" ] && [ "$tps" = "0" ]; then
            # Fallback: try to parse differently
            qps=$(echo "$output" | grep -oE 'QPS: [0-9]+' | head -1 | grep -oE '[0-9]+' || echo "0")
            tps="$qps"
        fi

        echo "QPS=$qps TPS=$tps"

        BENCHMARK_RESULTS=$(python3 -c "
import json, sys
d = json.loads('$BENCHMARK_RESULTS')
d['$name'] = {'qps': float($qps), 'tps': float($tps), 'threads': $threads}
print(json.dumps(d))
" 2>/dev/null || echo "{}")
    }

    # Run internal benchmarks (workload: point_select -> point, read -> oltp_read_write, write -> oltp_write_only)
    run_internal_bench "point_select" "point" 4 10
    run_internal_bench "oltp_read_write" "read_write" 4 10
    run_internal_bench "oltp_write_only" "write" 4 10
    run_internal_bench "update_index" "update_index" 4 10

else
    # External sysbench path
    # Always prepare fresh data (fast if tables already exist)
    echo "  Preparing test data..."
    sysbench --db-driver=mysql --mysql-host=127.0.0.1 --mysql-port=15995 \
        --mysql-user=root --mysql-password="" \
        --mysql-db=sbtest \
        --tables=1 --table_size=10000 \
        "$SYSBENCH_LUA/oltp_common.lua" prepare >/dev/null 2>&1

    run_sysbench_test() {
        local name="$1"
        local script="$2"
        local threads="${3:-4}"
        local time_sec="${4:-10}"

        echo -n "  $name (threads=$threads, time=${time_sec}s) ... "

        # Check if script exists
        if [ ! -f "$script" ]; then
            echo "SKIP (script not found: $script)"
            return
        fi

        local output
        output=$(sysbench \
            --db-driver=mysql \
            --mysql-host=127.0.0.1 \
            --mysql-port=15995 \
            --mysql-user=root \
            --mysql-password="" \
            --mysql-db=sbtest \
            --time="$time_sec" \
            --threads="$threads" \
            "$script" run 2>&1)

        local qps
        qps=$(echo "$output" | grep -oE '[0-9]+\.[0-9]+ per sec' | head -1 | grep -oE '[0-9]+\.[0-9]+' || echo "0")
        local tps
        tps=$(echo "$output" | grep 'transactions:' | grep -oE '[0-9]+[.][0-9]+ per sec' | head -1 | grep -oE '[0-9]+[.][0-9]+' || echo "0")

        echo "QPS=$qps TPS=$tps"

        # Store results (use qps for comparison)
        BENCHMARK_RESULTS=$(python3 -c "
import json, sys
d = json.loads('$BENCHMARK_RESULTS')
d['$name'] = {'qps': float($qps), 'tps': float($tps), 'threads': $threads}
print(json.dumps(d))
" 2>/dev/null || echo "{}")
    }

    # Run actual sysbench tests
    run_sysbench_test "point_select" "$SYSBENCH_LUA/oltp_point_select.lua" 4 10
    run_sysbench_test "oltp_read_write" "$SYSBENCH_LUA/oltp_read_write.lua" 4 10
    run_sysbench_test "oltp_write_only" "$SYSBENCH_LUA/oltp_write_only.lua" 4 10
    run_sysbench_test "update_index" "$SYSBENCH_LUA/oltp_update_index.lua" 4 10
fi

# ---------- Step 5: Evaluate against thresholds ----------
echo ""
echo "[4/5] Evaluating results..."

mkdir -p "$(dirname "$RESULT_FILE")"
echo "$BENCHMARK_RESULTS" > "$RESULT_FILE"

PASS_COUNT=0
FAIL_COUNT=0
TOTAL_COUNT=0

evaluate() {
    local name="$1"
    local threshold="$2"

    TOTAL_COUNT=$((TOTAL_COUNT + 1))

    local actual
    actual=$(python3 -c "import json; d=json.load(open('$RESULT_FILE')); print(int(d.get('$name', {}).get('qps', 0)))" 2>/dev/null || echo "0")

    if [ "$actual" -ge "$threshold" ]; then
        echo "✅ $name: $actual QPS ≥ $threshold QPS (threshold)"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        echo "❌ $name: $actual QPS < $threshold QPS (threshold)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
}

# Parse thresholds and evaluate
for key in point_select oltp_read_write oltp_write_only update_index; do
    threshold=$(echo "$THRESHOLDS" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['$key'])")
    evaluate "$key" "$threshold"
done

# ---------- Step 6: Summary ----------
echo ""
echo "[5/5] Summary"
echo "=== Sysbench Gate Results ==="
echo "Phase: $PHASE"
echo "PASS: $PASS_COUNT / $TOTAL_COUNT"
echo ""

if [ "$FAIL_COUNT" -gt 0 ]; then
    echo "❌ Sysbench Gate FAILED — $FAIL_COUNT / $TOTAL_COUNT benchmark(s) below threshold"
    echo "   Results saved to: $RESULT_FILE"
    exit 1
fi

echo "✅ Sysbench Gate PASSED — all $TOTAL_COUNT benchmarks meet $PHASE thresholds"
echo "   Results saved to: $RESULT_FILE"
exit 0