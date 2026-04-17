#!/bin/bash
# 逐个运行 tarpaulin 覆盖率测试（简化版）
# 使用说明: ./scripts/run-coverage-single.sh [crate_name]

set -e

OUTPUT_DIR="coverage-reports"
mkdir -p "$OUTPUT_DIR"

# 跳过的性能测试
SKIP_ARGS="--skip test_hnsw_1k_build_and_search --skip test_hnsw_100k --skip test_hnsw_10k --skip test_ivfpq_100k_performance --skip test_ivfpq_10k_performance --skip test_ivfpq_1m_performance --skip test_pq_train_and_encode --skip test_parallel_knn_index_basic"

run_coverage() {
    local crate=$1
    local start_time=$(date +%s)
    local log_file="$OUTPUT_DIR/${crate}.log"

    echo "[$(date '+%Y-%m-%d %H:%M:%S')] 开始: $crate"

    # 运行 tarpaulin
    if timeout 600 cargo tarpaulin -p "$crate" \
        --skip-clean \
        -t 300 \
        --out Html \
        --output-dir "$OUTPUT_DIR" \
        -- $SKIP_ARGS > "$log_file" 2>&1; then

        local end_time=$(date +%s)
        local elapsed=$((end_time - start_time))

        # 查找生成的 HTML 文件
        local html_file=$(find "$OUTPUT_DIR" -name "*.html" -newer "$log_file" 2>/dev/null | head -1)

        if [ -n "$html_file" ]; then
            # 提取覆盖率数字
            local coverage=$(grep -oP '\d+\.\d+%' "$html_file" | head -1 || echo "N/A")
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] 完成: $crate - 覆盖率: $coverage - 耗时: ${elapsed}s"
        else
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] 完成: $crate - HTML 未找到 - 耗时: ${elapsed}s"
        fi
    else
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] 失败: $crate (超时或错误)"
    fi
}

# 如果指定了 crate 名称，只运行这一个
if [ -n "$1" ]; then
    run_coverage "$1"
else
    # 运行所有 crates
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
        run_coverage "$crate"
    done

    echo ""
    echo "=== 所有覆盖率测试完成 ==="
fi