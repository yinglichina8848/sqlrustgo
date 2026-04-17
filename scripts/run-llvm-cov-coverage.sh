#!/bin/bash
# 使用 cargo-llvm-cov 运行覆盖率测试
# 使用: ./scripts/run-llvm-cov-coverage.sh [crate_name]

set -e

OUTPUT_DIR="coverage-reports"
mkdir -p "$OUTPUT_DIR"

# 所有 crates
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

run_coverage() {
    local crate=$1
    local start_time=$(date +%s)

    echo "[$(date '+%H:%M:%S')] 开始覆盖率测试: $crate"

    # 生成 HTML 报告
    if cargo llvm-cov -p "$crate" --lib --html --output-dir "$OUTPUT_DIR" 2>&1 | tail -5; then
        local end_time=$(date +%s)
        local elapsed=$((end_time - start_time))

        # 提取覆盖率数字
        local lcov_file="$OUTPUT_DIR/${crate}.lcov"
        cargo llvm-cov -p "$crate" --lib --lcov --output-path "$lcov_file" 2>&1 > /dev/null

        # 读取覆盖率
        if [ -f "$lcov_file" ]; then
            local coverage=$(grep -c "DA:" "$lcov_file" | head -1)
            echo "[$(date '+%H:%M:%S')] 完成: $crate - 报告: $OUTPUT_DIR/${crate}/index.html - 耗时: ${elapsed}s"
        else
            echo "[$(date '+%H:%M:%S')] 完成: $crate - 耗时: ${elapsed}s"
        fi
    else
        echo "[$(date '+%H:%M:%S')] 失败: $crate"
    fi
}

if [ -n "$1" ]; then
    # 只运行指定的 crate
    run_coverage "$1"
else
    echo "=== 开始覆盖率测试 (cargo-llvm-cov) ==="
    echo "输出目录: $OUTPUT_DIR"
    echo ""

    for crate in "${CRATES[@]}"; do
        run_coverage "$crate"
    done

    echo ""
    echo "=== 覆盖率测试完成 ==="
    echo "查看报告: ls -la $OUTPUT_DIR/"
fi