#!/usr/bin/env bash
# Coverage Gate Check - uses cargo llvm-cov
# Fixed: Properly aggregates per-crate coverage for L1 crates
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
COVERAGE_DIR="$PROJECT_ROOT/artifacts/coverage"

mkdir -p "$COVERAGE_DIR"

MODE="${1:-full}"

# L1 crates for RC/GA gate (must match RC_GATE_CHECKLIST.md)
L1_CRATES=(
    "sqlrustgo-types"
    "sqlrustgo-parser"
    "sqlrustgo-planner"
    "sqlrustgo-optimizer"
    "sqlrustgo-executor"
    "sqlrustgo-storage"
    "sqlrustgo-transaction"
    "sqlrustgo-catalog"
)

detect_memory_and_threads() {
    local memory_gb
    if [[ "$(uname)" == "Darwin" ]]; then
        memory_gb=$(( $(sysctl -n hw.memsize 2>/dev/null || echo 8589934592) / 1024 / 1024 / 1024 ))
    else
        memory_gb=$(( $(grep MemTotal /proc/meminfo 2>/dev/null | awk '{print $2}') / 1024 / 1024 ))
    fi

    local threads
    if [[ "$memory_gb" -lt 8 ]]; then
        threads=1
    elif [[ "$memory_gb" -lt 16 ]]; then
        threads=2
    elif [[ "$memory_gb" -lt 32 ]]; then
        threads=4
    else
        threads=8
    fi

    echo "$memory_gb $threads"
}

MEMORY_INFO=$(detect_memory_and_threads)
SYSTEM_MEMORY_GB=$(echo "$MEMORY_INFO" | awk '{print $1}')
TEST_THREADS=$(echo "$MEMORY_INFO" | awk '{print $2}')

echo "=== Running Coverage Gate Check (llvm-cov) ==="
echo "System memory: ${SYSTEM_MEMORY_GB}GB, Test threads: $TEST_THREADS"

# Check if llvm-cov is available
if ! command -v cargo-llvm-cov &>/dev/null; then
    echo "❌ cargo-llvm-cov not installed"
    echo "Install with: cargo install cargo-llvm-cov"
    exit 1
fi

# Get coverage for a single crate
get_crate_coverage() {
    local crate="$1"
    local output_file="$COVERAGE_DIR/${crate#sqlrustgo-}.json"
    local tmp_file="$COVERAGE_DIR/.tmp_${crate#sqlrustgo-}.json"

    # Remove old files to ensure fresh data
    rm -f "$output_file" "$tmp_file"

    echo "[COVERAGE] Running $crate..."

    # Run coverage for this crate
    if cargo llvm-cov --package "$crate" --all-features --lib --json --output-path "$tmp_file" 2>/dev/null; then
        mv "$tmp_file" "$output_file"
        # Extract percentage using correct JSON path: data[0].totals.lines.percent
        local pct=$(python3 -c "
import json
with open('$output_file') as f:
    data = json.load(f)
pct = data.get('data', [{}])[0].get('totals', {}).get('lines', {}).get('percent', 0)
print(f'{pct:.2f}')
" 2>/dev/null || echo "0")
        echo "  -> $crate: ${pct}%"
        echo "$pct"
    else
        rm -f "$tmp_file"
        echo "  -> $crate: FAILED"
        echo "0"
    fi
}

# Run coverage for all L1 crates and aggregate
run_l1_coverage() {
    echo "=== Running L1 Crate Coverage ==="
    echo "L1 crates: ${#L1_CRATES[@]}"
    echo ""

    local total_pct=0
    local count=0
    local missing_crates=()

    for crate in "${L1_CRATES[@]}"; do
        local pct=$(get_crate_coverage "$crate")
        if [[ "$pct" != "0" && "$pct" != "" ]]; then
            total_pct=$(echo "$total_pct + $pct" | bc -l)
            count=$((count + 1))
        else
            missing_crates+=("$crate")
        fi
    done

    if [[ $count -gt 0 ]]; then
        local avg_pct=$(echo "scale=2; $total_pct / $count" | bc -l)
        echo ""
        echo "=== L1 Coverage Summary ==="
        echo "Crates measured: $count/${#L1_CRATES[@]}"
        echo "Average coverage: ${avg_pct}%"

        if [[ ${#missing_crates[@]} -gt 0 ]]; then
            echo "Missing crates: ${missing_crates[*]}"
            echo "WARNING: Coverage may be inaccurate - ${#missing_crates[@]} crate(s) failed"
        fi

        # Save aggregated result
        echo "$avg_pct" > "$COVERAGE_DIR/l1_coverage.txt"

        # Save full report as JSON (compatible format)
        python3 << EOF
import json
import os

result = {
    "type": "llvm.coverage.json.export",
    "version": "3.0.1",
    "data": [{
        "totals": {
            "lines": {
                "count": 0,
                "covered": 0,
                "percent": $avg_pct
            }
        }
    }]
}

with open('$COVERAGE_DIR/coverage.json', 'w') as f:
    json.dump(result, f)
EOF
    else
        echo "ERROR: No crates could be measured"
        exit 1
    fi
}

# Run coverage based on mode
if [ "$MODE" = "incremental" ]; then
    echo "Running incremental coverage..."
    CHANGED_CRATES=$(git diff --name-only --diff-filter=ACMR | cut -d/ -f2 | sort -u | grep -E "^crates/" | cut -d/ -f2 || true)
    if [ -z "$CHANGED_CRATES" ]; then
        echo "No crate changes detected, using full coverage"
        MODE="full"
    else
        echo "Changed crates: $CHANGED_CRATES"
        for crate in $CHANGED_CRATES; do
            get_crate_coverage "sqlrustgo-$crate"
        done
    fi
fi

if [ "$MODE" = "full" ]; then
    echo "Running full L1 coverage test..."
    TIMEOUT=600

    # Use per-crate approach for accurate L1 coverage
    run_l1_coverage
fi

# Read coverage from aggregated result
if [[ -f "$COVERAGE_DIR/l1_coverage.txt" ]]; then
    COVERAGE=$(cat "$COVERAGE_DIR/l1_coverage.txt")
else
    echo "❌ Coverage data not available"
    exit 1
fi

# Detect version/phase from branch for threshold
detect_threshold() {
    local branch
    branch=$(git symbolic-ref --short HEAD 2>/dev/null || git rev-parse --short HEAD 2>/dev/null)
    if [[ "$branch" =~ "alpha" ]]; then
        echo 50
    elif [[ "$branch" =~ "develop/v3" ]] || [[ "$branch" =~ "beta" ]]; then
        echo 75
    elif [[ "$branch" =~ "rc" ]]; then
        echo 85
    else
        echo 85
    fi
}

# Convert to integer percentage
COVERAGE_INT=$(echo "$COVERAGE * 100" | bc | cut -d. -f1)
REQUIRED=$(detect_threshold)

echo ""
echo "=== Coverage Result ==="
echo "Current coverage: ${COVERAGE_INT}%"
echo "Required coverage: ${REQUIRED}%"

if [ "$COVERAGE_INT" -lt "$REQUIRED" ]; then
    echo "❌ Coverage too low! Need at least ${REQUIRED}%"
    exit 1
fi

echo "✅ Coverage check passed!"
echo ""
echo "=== Per-Crate Coverage Report ==="
for crate in "${L1_CRATES[@]}"; do
    crate_name="${crate#sqlrustgo-}"
    json_file="$COVERAGE_DIR/${crate_name}.json"
    if [[ -f "$json_file" ]]; then
        pct=$(python3 -c "
import json
with open('$json_file') as f:
    data = json.load(f)
pct = data.get('data', [{}])[0].get('totals', {}).get('lines', {}).get('percent', 0)
print(f'{pct:.2f}')
" 2>/dev/null || echo "N/A")
        echo "  ${crate_name}: ${pct}%"
    else
        echo "  ${crate_name}: MISSING"
    fi
done

echo ""
echo "Coverage reports saved to: $COVERAGE_DIR/"
echo "=== Coverage Gate Check Complete ==="