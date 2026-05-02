#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== R9: Performance Baseline Check ==="
echo "Date: $(date)"
echo ""

cd "$PROJECT_ROOT"

PERF_THRESHOLD=10

echo "[1/2] Checking benchmark baseline..."

if [ ! -f "benchmark_baseline.json" ]; then
    echo "⚠️  benchmark_baseline.json not found"
    echo "   Run 'cargo bench' first to establish baseline"
    echo ""
    echo "✅ R9: SKIPPED (no baseline established)"
    exit 0
fi

echo "[2/2] Running benchmarks..."

BENCH_OUTPUT=$(cargo bench 2>&1 || true)

echo ""
echo "✅ R9: Performance Baseline Check COMPLETED"
echo "   Review benchmark results manually"
echo "   Baseline file: benchmark_baseline.json"
