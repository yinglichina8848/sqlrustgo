#!/bin/bash
# Benchmark runner script
# Runs all benchmarks and saves results with timestamp

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BENCHMARK_DIR="$PROJECT_DIR/benchmark_results"

# Create benchmark results directory
mkdir -p "$BENCHMARK_DIR"

echo "Running benchmarks..."
echo "Timestamp: $TIMESTAMP"
echo ""

# Run all benchmarks
cargo bench --all -- --save-baseline="$TIMESTAMP" 2>&1 | tee "$BENCHMARK_DIR/bench_$TIMESTAMP.log"

echo ""
echo "Benchmark results saved with baseline: $TIMESTAMP"
echo "To compare with previous baseline:"
echo "  cargo bench --all -- --compare-with=$TIMESTAMP"
echo ""
echo "To run comparison script:"
echo "  python3 scripts/compare_benchmarks.py \\"
echo "    --baseline1 $BENCHMARK_DIR/baseline1.json \\"
echo "    --baseline2 $BENCHMARK_DIR/baseline2.json \\"
echo "    --threshold 10"
