#!/usr/bin/env bash
# Performance Regression Detection Script
# Compares benchmark results and detects performance regressions

set -e

BENCHMARK_FILE="${1:-benchmark.json}"
THRESHOLD="${2:-20}"

if [ ! -f "$BENCHMARK_FILE" ]; then
    echo "Error: Benchmark file '$BENCHMARK_FILE' not found"
    exit 1
fi

echo "=== Performance Regression Detection ==="
echo "File: $BENCHMARK_FILE"
echo "Threshold: ${THRESHOLD}%"
echo ""

TOTAL_REGRESSIONS=0
TOTAL_IMPROVEMENTS=0

parse_benchmark_json() {
    local file="$1"
    python3 -c "
import json
import sys

try:
    with open('$file', 'r') as f:
        data = json.load(f)
    
    if 'benchmarks' in data:
        for bench in data['benchmarks']:
            name = bench.get('full_name', bench.get('name', ''))
            mean = bench.get('mean', {})
            if isinstance(mean, dict):
                value = mean.get('value', 0)
                unit = mean.get('unit', 'ns')
            else:
                value = mean
                unit = 'ns'
            print(f'{name}|{value}|{unit}')
except Exception as e:
    print(f'Error: {e}', file=sys.stderr)
    sys.exit(1)
"
}

current_results=$(parse_benchmark_json "$BENCHMARK_FILE")

if [ -z "$current_results" ]; then
    echo "Warning: No benchmark results found in $BENCHMARK_FILE"
    exit 0
fi

echo "Checking benchmarks against baseline..."
echo ""

echo "| Benchmark | Current | Unit | Status |"
echo "|-----------|---------|------|--------|"

while IFS='|' read -r name value unit; do
    if [ -z "$name" ] || [ -z "$value" ]; then
        continue
    fi
    
    baseline_file="scripts/benchmark/baseline_$(echo "$name" | tr '[:upper:]' '[:lower:]' | tr -cd '[:alnum:]').txt"
    
    if [ -f "$baseline_file" ]; then
        baseline=$(cat "$baseline_file")
        
        if (( $(echo "$value < $baseline" | bc -l) )); then
            improvement=$(echo "scale=2; (($baseline - $value) / $baseline) * 100" | bc -l)
            echo "| $name | $value | $unit | ✅ Improved ${improvement}% |"
            TOTAL_IMPROVEMENTS=$((TOTAL_IMPROVEMENTS + 1))
        elif (( $(echo "($value - $baseline) / $baseline * 100 > $THRESHOLD" | bc -l) )); then
            regression=$(echo "scale=2; (($value - $baseline) / $baseline) * 100" | bc -l)
            echo "| $name | $value | $unit | ❌ Regressed ${regression}% |"
            TOTAL_REGRESSIONS=$((TOTAL_REGRESSIONS + 1))
        else
            echo "| $name | $value | $unit | ➖ Within threshold |"
        fi
    else
        echo "| $name | $value | $unit | 🆕 New benchmark |"
        echo "$value" > "$baseline_file"
    fi
done <<< "$current_results"

echo ""
echo "=== Summary ==="
echo "Regressions: $TOTAL_REGRESSIONS"
echo "Improvements: $TOTAL_IMPROVEMENTS"

if [ $TOTAL_REGRESSIONS -gt 0 ]; then
    echo ""
    echo "❌ Performance regression detected!"
    exit 1
else
    echo ""
    echo "✅ No performance regressions detected"
    exit 0
fi
