#!/bin/bash
# 覆盖率测试框架 - 逐个 crate 运行 tarpaulin

set -e

OUTPUT_DIR="coverage-reports"
mkdir -p "$OUTPUT_DIR"

# 需要测试的 crates 列表（按大小排序，小的在前）
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

# 跳过的性能测试
SKIP_TESTS=(
    "test_hnsw_1k_build_and_search"
    "test_hnsw_100k"
    "test_hnsw_10k"
    "test_ivfpq_100k_performance"
    "test_ivfpq_10k_performance"
    "test_ivfpq_1m_performance"
    "test_pq_train_and_encode"
    "test_parallel_knn_index_basic"
)

# 生成 --skip 参数
SKIP_ARGS=""
for test in "${SKIP_TESTS[@]}"; do
    SKIP_ARGS="$SKIP_ARGS --skip $test"
done

echo "=== 覆盖率测试框架 ==="
echo "输出目录: $OUTPUT_DIR"
echo "crates 数量: ${#CRATES[@]}"
echo ""

# 存储结果
TOTAL_COVERAGE=0
COVERAGE_COUNT=0

run_coverage() {
    local crate=$1
    local log_file="$OUTPUT_DIR/${crate}.log"
    local html_file="$OUTPUT_DIR/${crate}.html"

    echo "[$(date '+%H:%M:%S')] 开始测试: $crate"

    # 运行 tarpaulin，输出到文件
    timeout 600 cargo tarpaulin -p "$crate" \
        --skip-clean \
        -t 300 \
        --out Html \
        --output-dir "$OUTPUT_DIR" \
        -- --skip test_hnsw_1k_build_and_search \
           --skip test_hnsw_100k \
           --skip test_hnsw_10k \
           --skip test_ivfpq_100k_performance \
           --skip test_ivfpq_10k_performance \
           --skip test_ivfpq_1m_performance \
           --skip test_pq_train_and_encode \
           --skip test_parallel_knn_index_basic \
        > "$log_file" 2>&1

    # 提取覆盖率
    if [ -f "$html_file" ]; then
        # 从 HTML 文件提取覆盖率
        local coverage=$(grep -oP 'Total.*?\d+\.\d+%' "$html_file" 2>/dev/null | head -1 || echo "N/A")
        echo "[$(date '+%H:%M:%S')] 完成: $crate - $coverage"
    else
        echo "[$(date '+%H:%M:%S')] 完成: $crate - HTML 未生成"
    fi
}

export -f run_coverage
export OUTPUT_DIR
export SKIP_ARGS

# 后台运行所有 crate
PIDS=()
for crate in "${CRATES[@]}"; do
    run_coverage "$crate" &
    PIDS+=($!)

    # 限制并发数，避免系统过载
    if [ ${#PIDS[@]} -ge 3 ]; then
        wait ${PIDS[0]}
        PIDS=("${PIDS[@]:1}")
    fi

    # 每个 crate 之间稍作延迟
    sleep 2
done

# 等待所有后台任务完成
for pid in "${PIDS[@]}"; do
    wait $pid
done

echo ""
echo "=== 覆盖率测试完成 ==="
echo "查看报告: ls -la $OUTPUT_DIR/"