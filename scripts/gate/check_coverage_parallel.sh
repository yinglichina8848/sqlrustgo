#!/usr/bin/env bash
# Parallel Coverage Script - 使用 cargo llvm-cov 并行测试
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/artifacts/coverage"

PARALLEL=4
WAVE="all"
TIMEOUT=300

# macOS 兼容：检测 timeout 命令
if ! command -v timeout &>/dev/null; then
    if command -v gtimeout &>/dev/null; then
        timeout() { gtimeout "$@"; }
    else
        # 使用 perl 实现 timeout
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
    --parallel N    并行度 (default: 4)
    --wave N        运行哪一波 (1,2,3,4,all; default: all)
    --timeout N     单模块超时秒数 (default: 300)
    --help          显示帮助

Examples:
    $0 --parallel 4 --wave all
    $0 --parallel 2 --wave 1
    $0 --parallel 6 --wave 2
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --parallel)
            PARALLEL="${2:-4}"
            shift 2
            ;;
        --wave)
            WAVE="${2:-all}"
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

# Wave 1: 轻量模块
WAVE1_MODULES=(
    "sqlrustgo-common"
    "sqlrustgo-types"
    "sqlrustgo-expr"
    "sqlrustgo-catalog"
    "query-stats"
    "information-schema"
    "sqlrustgo-telemetry"
    "sqlrustgo-security"
    "sqlrustgo-tools"
)

# Wave 2: 中型模块
WAVE2_MODULES=(
    "sqlrustgo-parser"
    "sqlrustgo-planner"
    "sqlrustgo-optimizer"
    "sqlrustgo-executor"
    "sqlrustgo-network"
    "sqlrustgo-mysql-server"
    "sqlrustgo-transaction"
    "sqlrustgo-server"
)

# Wave 3: 重要/复杂模块
WAVE3_MODULES=(
    "sqlrustgo-storage"
    "sqlrustgo-distributed"
    "sqlrustgo-vector"
    "sqlrustgo-graph"
    "sqlrustgo-sql-corpus"
)

# Wave 4: 辅助工具
WAVE4_MODULES=(
    "sqlrustgo-agentsql"
    "sqlrustgo-gmp"
    "sqlrustgo-rag"
    "sqlrustgo-qmd-bridge"
    "sqlrustgo-unified-storage"
    "sqlrustgo-unified-query"
)

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
        run_wave 1 "${WAVE1_MODULES[@]}"
        run_wave 2 "${WAVE2_MODULES[@]}"
        run_wave 3 "${WAVE3_MODULES[@]}"
        run_wave 4 "${WAVE4_MODULES[@]}"
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
