#!/bin/bash
# 快速覆盖率测试 - 使用 cargo-llvm-cov
# 使用: ./scripts/quick-coverage.sh

set -e

OUTPUT_DIR="coverage-reports"
mkdir -p "$OUTPUT_DIR"

echo "=== 快速覆盖率测试 ==="
echo ""

# 存储结果
declare -A COVERAGE_RESULTS

run_test() {
    local crate=$1
    local start=$(date +%s)

    echo -n "[$crate] "

    # 运行测试并生成 lcov
    cargo llvm-cov -p "$crate" --lib --lcov --output-path "$OUTPUT_DIR/${crate}.lcov" 2>&1 | grep -E "^(test result|running|Finished)" || true

    # 计算覆盖率
    if [ -f "$OUTPUT_DIR/${crate}.lcov" ]; then
        local lf=$(grep "LF:" "$OUTPUT_DIR/${crate}.lcov" | cut -d: -f2 | paste -sd+ | bc 2>/dev/null || echo "0")
        local lh=$(grep "LH:" "$OUTPUT_DIR/${crate}.lcov" | cut -d: -f2 | paste -sd+ | bc 2>/dev/null || echo "0")

        if [ "$lf" -gt 0 ]; then
            local pct=$(echo "scale=1; $lh * 100 / $lf" | bc)
            echo "覆盖率: ${pct}%"
            COVERAGE_RESULTS["$crate"]="${pct}%"
        else
            echo "覆盖率: N/A"
            COVERAGE_RESULTS["$crate"]="N/A"
        fi
    else
        echo "覆盖率: ERROR"
        COVERAGE_RESULTS["$crate"]="ERROR"
    fi
}

# 按大小排序的 crates
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

for crate in "${CRATES[@]}"; do
    run_test "$crate"
done

echo ""
echo "=== 汇总 ==="
echo "crate | 覆盖率"
echo "------|--------"
for crate in "${CRATES[@]}"; do
    echo "$crate | ${COVERAGE_RESULTS[$crate]}"
done