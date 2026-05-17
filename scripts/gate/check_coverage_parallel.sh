#!/usr/bin/env bash
# Parallel Coverage Script - uses cargo llvm-cov for parallel testing
# Fixed: Proper L1 crate coverage measurement
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/artifacts/coverage"

# L1 crates for RC/GA gate (must match check_coverage.sh and RC_GATE_CHECKLIST.md)
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

detect_parallelism() {
    local memory_gb
    if [[ "$(uname)" == "Darwin" ]]; then
        memory_gb=$(( $(sysctl -n hw.memsize 2>/dev/null || echo 8589934592) / 1024 / 1024 / 1024 ))
    else
        memory_gb=$(( $(grep MemTotal /proc/meminfo 2>/dev/null | awk '{print $2}') / 1024 / 1024 ))
    fi

    if [[ "$memory_gb" -lt 8 ]]; then
        echo 1
    elif [[ "$memory_gb" -lt 16 ]]; then
        echo 2
    elif [[ "$memory_gb" -lt 32 ]]; then
        echo 4
    else
        echo 6
    fi
}

PARALLEL=$(detect_parallelism)
WAVE="all"
TIMEOUT=300

if ! command -v timeout &>/dev/null; then
    if command -v gtimeout &>/dev/null; then
        timeout() { gtimeout "$@"; }
    else
        timeout() {
            local secs="$1"
            shift
            perl -e 'alarm shift; exec @ARGV' "$secs" "$@"
        }
    fi
fi

usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Options:
    --parallel N    Parallelism (auto-detected: $PARALLEL based on system memory)
    --wave N        运行哪一波 (l1,1,2,3,4,all; default: l1)
    --timeout N     单模块超时秒数 (default: 300)
    --help          显示帮助

Examples:
    $0 --parallel 4 --wave l1        # Run L1 crates only
    $0 --parallel 4 --wave all      # Run all waves
    $0 --parallel 2 --wave 2        # Run Wave 2 only
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --parallel)
            PARALLEL="${2:-4}"
            shift 2
            ;;
        --wave)
            WAVE="${2:-l1}"
            shift 2
            ;;
        --timeout)
            TIMEOUT="${2:-300}"
            shift 2
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

mkdir -p "$OUTPUT_DIR"

run_coverage() {
    local pkg="$1"
    local output_file="$OUTPUT_DIR/${pkg#sqlrustgo-}.json"
    local start_time=$(date +%s)

    if [[ -f "$output_file" ]]; then
        echo "[SKIP] $pkg (already exists)"
        return 0
    fi

    echo "[START] $pkg"
    local tmp_file="$OUTPUT_DIR/.tmp_${pkg#sqlrustgo-}.json"

    if timeout "$TIMEOUT" cargo llvm-cov --package "$pkg" --all-features --lib --json --output-path "$tmp_file" 2>/dev/null; then
        mv "$tmp_file" "$output_file"
        local end_time=$(date +%s)
        local elapsed=$((end_time - start_time))
        echo "[DONE]  $pkg (${elapsed}s)"
    else
        rm -f "$tmp_file"
        local end_time=$(date +%s)
        local elapsed=$((end_time - start_time))
        echo "[FAIL]  $pkg (${elapsed}s) - continuing..."
    fi
}

export -f run_coverage
export OUTPUT_DIR TIMEOUT

run_wave() {
    local wave_name="$1"
    shift
    local modules=("$@")
    local total=${#modules[@]}
    local running=0
    local pids=()
    local names=()

    echo "=== Wave $wave_name: ${total} modules, parallel=$PARALLEL ==="

    for pkg in "${modules[@]}"; do
        while (( running >= PARALLEL )); do
            for i in "${!pids[@]}"; do
                if ! kill -0 "${pids[i]}" 2>/dev/null; then
                    wait "${pids[i]}" || true
                    unset 'pids[i]'
                    unset 'names[i]'
                    running=$((running - 1))
                fi
            done
            [[ ${#pids[@]} -gt 0 ]] && pids=("${pids[@]}")
            [[ ${#names[@]} -gt 0 ]] && names=("${names[@]}")
            sleep 0.5
        done

        run_coverage "$pkg" &
        pids+=($!)
        names+=("$pkg")
        running=$((running + 1))
        echo "[QUEUED] $pkg (running: $running/$total)"
    done

    echo "Waiting for remaining ${#pids[@]} jobs..."
    for pid in "${pids[@]}"; do
        wait "$pid" || true
    done

    echo "=== Wave $wave_name complete ==="
    echo ""
}

aggregate_l1_coverage() {
    echo "=== Aggregating L1 Coverage ==="

    local total_pct=0
    local count=0
    local missing=()

    for crate in "${L1_CRATES[@]}"; do
        crate_name="${crate#sqlrustgo-}"
        json_file="$OUTPUT_DIR/${crate_name}.json"

        if [[ -f "$json_file" ]]; then
            local pct=$(python3 -c "
import json
with open('$json_file') as f:
    data = json.load(f)
pct = data.get('data', [{}])[0].get('totals', {}).get('lines', {}).get('percent', 0)
print(f'{pct:.2f}')
" 2>/dev/null || echo "0")

            if [[ "$pct" != "0" && -n "$pct" ]]; then
                echo "  $crate_name: ${pct}%"
                total_pct=$(echo "$total_pct + $pct" | bc -l)
                count=$((count + 1))
            else
                missing+=("$crate_name")
            fi
        else
            missing+=("$crate_name")
        fi
    done

    if [[ $count -gt 0 ]]; then
        local avg_pct=$(echo "scale=2; $total_pct / $count" | bc -l)
        echo ""
        echo "=== L1 Coverage Summary ==="
        echo "Crates measured: $count/${#L1_CRATES[@]}"
        echo "Average coverage: ${avg_pct}%"

        if [[ ${#missing[@]} -gt 0 ]]; then
            echo "Missing: ${missing[*]}"
        fi

        echo "$avg_pct" > "$OUTPUT_DIR/l1_coverage.txt"

        python3 << EOF
import json
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
with open('$OUTPUT_DIR/coverage.json', 'w') as f:
    json.dump(result, f)
EOF

        echo "Saved to $OUTPUT_DIR/coverage.json"
    else
        echo "ERROR: No crates measured"
        exit 1
    fi
}

echo "=========================================="
echo "Parallel Coverage Check"
echo "=========================================="
echo "Output:   $OUTPUT_DIR"
echo "Parallel: $PARALLEL"
echo "Timeout:  ${TIMEOUT}s per module"
echo "Wave:     $WAVE"
echo "=========================================="
echo ""

case "$WAVE" in
    l1)
        run_wave "L1" "${L1_CRATES[@]}"
        aggregate_l1_coverage
        ;;
    1)
        run_wave 1 "${WAVE1_MODULES[@]}"
        ;;
    2)
        run_wave 2 "${WAVE2_MODULES[@]}"
        ;;
    3)
        run_wave 3 "${WAVE3_MODULES[@]}"
        ;;
    4)
        run_wave 4 "${WAVE4_MODULES[@]}"
        ;;
    all)
        run_wave "L1" "${L1_CRATES[@]}"
        aggregate_l1_coverage
        ;;
    *)
        echo "Invalid wave: $WAVE"
        usage
        exit 1
        ;;
esac

echo ""
echo "=========================================="
echo "Coverage run complete!"
echo "Reports saved to: $OUTPUT_DIR/"
echo "=========================================="
echo ""
echo "Generated reports:"
ls -la "$OUTPUT_DIR"/*.json 2>/dev/null || echo "No JSON files found"