#!/bin/bash
# 使用 cargo-llvm-cov 运行覆盖率测试 - 生成摘要报告
# 使用: ./scripts/run-coverage-summary.sh [crate_name]

set -e

OUTPUT_DIR="coverage-reports"
mkdir -p "$OUTPUT_DIR"

run_coverage() {
    local crate=$1
    local start_time=$(date +%s)

    echo "[$(date '+%H:%M:%S')] 测试覆盖率: $crate"

    # 生成覆盖率报告
    cargo llvm-cov -p "$crate" --lib --lcov --output-path "$OUTPUT_DIR/${crate}.lcov" 2>&1 | grep -v "^info:" | grep -v "^$"

    local end_time=$(date +%s)
    local elapsed=$((end_time - start_time))

    # 计算覆盖率
    if [ -f "$OUTPUT_DIR/${crate}.lcov" ]; then
        # 从 lcov 计算覆盖率
        local lines_found=$(grep "LF:" "$OUTPUT_DIR/${crate}.lcov" | cut -d: -f2 | paste -sd+ | bc 2>/dev/null || echo "0")
        local lines_hit=$(grep "LH:" "$OUTPUT_DIR/${crate}.lcov" | cut -d: -f2 | paste -sd+ | bc 2>/dev/null || echo "0")

        if [ "$lines_found" -gt 0 ]; then
            local coverage=$(echo "scale=2; $lines_hit / $lines_found * 100" | bc)
            echo "  -> 覆盖率: ${coverage}% ($lines_hit/$lines_found 行)"
        fi
    fi

    echo "  -> 耗时: ${elapsed}s"
    echo ""
}

# 运行所有 crates
echo "=== 覆盖率测试 ==="
echo ""

CRATES=(
    "sqlrustgo-common"
    "sqlrustgo-types"
    "sqlrustgo-telemetry"
    "sqlrustgo-query-stats"
    "sqlrustgo-distributed"
    "sqlrustgo-security"
    "sqlrustgo-unified-query"
    "sqlrustgo-gmp"
    "sqlrustgo-unified-storage"
    "sqlrustgo-graph"
    "sqlrustgo-planner"
    "sqlrustgo-optimizer"
    "sqlrustgo-catalog"
    "sqlrustgo-server"
    "sqlrustgo-agentsql"
    "sqlrustgo-executor"
    "sqlrustgo-parser"
    "sqlrustgo-storage"
    "sqlrustgo-transaction"
)

TOTAL_COVERAGE=0
COUNT=0

for crate in "${CRATES[@]}"; do
    run_coverage "$crate"
done

echo "=== 覆盖率报告已保存到 $OUTPUT_DIR/*.lcov ==="