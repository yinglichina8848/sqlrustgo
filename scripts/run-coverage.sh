#!/bin/bash
# 覆盖率测试 - 只测试 lib，不测 examples
# 使用: ./scripts/run-coverage.sh [crate_name]

set -e

OUTPUT_DIR="coverage-reports"
mkdir -p "$OUTPUT_DIR"

# 性能测试列表（会被跳过）
PERF_TESTS=(
    "test_hnsw_1k_build_and_search"
    "test_hnsw_100k"
    "test_hnsw_10k"
    "test_ivfpq_100k_performance"
    "test_ivfpq_10k_performance"
    "test_ivfpq_1m_performance"
    "test_pq_train_and_encode"
    "test_parallel_knn_index_basic"
    "test_hnsw_1m_search_performance"
    "test_parallel_knn_1m_search_performance"
    "test_parallel_knn_scale_performance"
)

# 构建 skip 参数
SKIP_ARGS=""
for test in "${PERF_TESTS[@]}"; do
    SKIP_ARGS="$SKIP_ARGS --skip $test"
done

run_coverage() {
    local crate=$1
    local start_time=$(date +%s)
    local log_file="$OUTPUT_DIR/${crate}-coverage.log"

    echo "[$(date '+%H:%M:%S')] 开始覆盖率测试: $crate"

    # 只运行 lib 测试，跳过 examples 和性能测试
    # 使用 LIB-ONLY 模式
    if cargo tarpaulin -p "$crate" \
        --skip-clean \
        --lib \
        -t 900 \
        --out Html \
        --output-dir "$OUTPUT_DIR" \
        -- $SKIP_ARGS > "$log_file" 2>&1; then

        local end_time=$(date +%s)
        local elapsed=$((end_time - start_time))

        # 查找覆盖率结果
        local report_file=$(find "$OUTPUT_DIR" -name "${crate}*.html" 2>/dev/null | head -1)
        if [ -n "$report_file" ]; then
            # 从日志提取覆盖率
            local coverage=$(grep -oP 'Coverage: \d+\.\d+%' "$log_file" 2>/dev/null | head -1 || echo "查看HTML报告")
            echo "[$(date '+%H:%M:%S')] 完成: $crate - $coverage - 耗时: ${elapsed}s - 报告: $report_file"
        else
            echo "[$(date '+%H:%M:%S')] 完成: $crate - 耗时: ${elapsed}s"
        fi
    else
        echo "[$(date '+%H:%M:%S')] 失败: $crate (可能超时)"
    fi
}

# 导出函数供后台使用
export -f run_coverage

if [ -n "$1" ]; then
    # 只运行指定的 crate
    run_coverage "$1"
else
    # 运行所有主要 crates（按大小排序）
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

    echo "=== 开始覆盖率测试 ==="
    echo "输出目录: $OUTPUT_DIR"
    echo "crates: ${#CRATES[@]}"
    echo ""

    for crate in "${CRATES[@]}"; do
        run_coverage "$crate"
    done

    echo ""
    echo "=== 覆盖率测试完成 ==="
    echo "查看日志: ls -la $OUTPUT_DIR/*.log"
fi