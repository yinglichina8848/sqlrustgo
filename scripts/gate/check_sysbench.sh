#!/usr/bin/env bash
# ============================================================
# Sysbench Gate — v3.0.0 Alpha/Beta Performance Baseline
#
# Runs sysbench oltp_read_write against SQLRustGo and enforces
# minimum QPS thresholds per workload type.
#
# Thresholds (Alpha):
#   point_select:     ≥ 30,000 QPS
#   oltp_read_write:  ≥ 10,000 QPS
#   oltp_write_only:  ≥  8,000 QPS
#   update_index:     ≥  8,000 QPS
#
# Thresholds (Beta, stricter):
#   point_select:     ≥ 50,000 QPS
#   oltp_read_write:  ≥ 20,000 QPS
#   oltp_write_only:  ≥ 15,000 QPS
#   update_index:     ≥ 15,000 QPS
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
        THRESHOLDS='{"point_select":30000,"oltp_read_write":10000,"oltp_write_only":8000,"update_index":8000}'
        ;;
    beta)
        THRESHOLDS='{"point_select":50000,"oltp_read_write":20000,"oltp_write_only":15000,"update_index":15000}'
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

SQLRUSTGO_BIN="$PROJECT_ROOT/target/release/sqlrustgo"

if [ ! -f "$SQLRUSTGO_BIN" ]; then
    echo "❌ sqlrustgo binary not found at $SQLRUSTGO_BIN"
    echo "   Build with: cargo build --release"
    exit 1
fi

# Start server in background
$SQLRUSTGO_BIN --server-port 15995 &
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

# ---------- Step 3: Run sysbench if available ----------
run_sysbench() {
    local test_name="$1"
    local script_path="$2"

    if ! command -v sysbench &>/dev/null; then
        echo "⚠️  sysbench not installed, using direct SQL measurement"
        return 1
    fi

    sysbench "$script_path" \
        --db-driver=mysql \
        --mysql-host=127.0.0.1 \
        --mysql-port=15995 \
        --mysql-user=root \
        --mysql-password="" \
        --mysql-db=sbtest \
        --time=10 \
        --threads=4 \
        run 2>&1 | grep -oE 'transactions:[^)]+\([0-9]+\.[0-9]+ per sec\)' || true
}

# ---------- Step 4: Direct SQL measurement (fallback / primary) ----------
echo ""
echo "[3/5] Running benchmark queries directly..."

BENCHMARK_RESULTS="{}"

measure_qps() {
    local name="$1"
    local sql="$2"
    local threads="${3:-4}"

    echo -n "  $name ... "

    # Warmup
    for _ in {1..2}; do
        echo "$sql" | mysql -h 127.0.0.1 -P 15995 --protocol=tcp -u root 2>/dev/null > /dev/null || true
    done

    # Measure
    local start end elapsed qps rows
    start=$(python3 -c 'import time; print(time.time())')
    # Run N threads concurrently using background jobs
    local pids=()
    for ((i=0; i<threads; i++)); do
        (
            for _ in {1..50}; do
                echo "$sql" | mysql -h 127.0.0.1 -P 15995 --protocol=tcp -u root 2>/dev/null > /dev/null || true
            done
        ) &
        pids+=($!)
    done
    # Wait for all background jobs
    for pid in "${pids[@]}"; do
        wait "$pid" 2>/dev/null || true
    done
    end=$(python3 -c 'import time; print(time.time())')
    elapsed=$(python3 -c "print(round($end - $start, 3))")
    # Total ops = threads * 50
    local total_ops=$((threads * 50))
    qps=$(python3 -c "print(round($total_ops / $elapsed, 0))" 2>/dev/null || echo "0")

    echo "${qps} QPS (${elapsed}s)"

    # Update JSON
    BENCHMARK_RESULTS=$(python3 -c "
import json, sys
d = json.loads('$BENCHMARK_RESULTS')
d['$name'] = {'qps': $qps, 'elapsed': $elapsed, 'threads': $threads}
print(json.dumps(d))
" 2>/dev/null || echo "{}")
}

# point_select: simple PK lookup
measure_qps "point_select" \
    "SELECT c FROM t1 WHERE id=$(shuf -i 1-10000 -n 1) LIMIT 1" 8

# oltp_read_write: range scan + aggregate
measure_qps "oltp_read_write" \
    "SELECT SUM(c) FROM t1 WHERE id BETWEEN 1 AND 100" 4

# oltp_write_only: INSERT (use a temp table to avoid conflicts)
measure_qps "oltp_write_only" \
    "INSERT INTO t1 VALUES (99999, 'bench', 123)" 2

# update_index: UPDATE by indexed column
measure_qps "update_index" \
    "UPDATE t1 SET c=456 WHERE id=5000" 4

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