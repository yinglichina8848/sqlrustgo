#!/bin/bash
# 分阶段执行回归测试 - 逐个执行测试文件并报告结果
# 用法: ./run_staged_tests.sh [category_index]
# 不带参数则执行所有类别

set -o pipefail

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 测试类别定义 (cargo test --test 使用扁平名称)
CATEGORIES=(
    "单元测试:backup_test,bplus_tree_test,buffer_pool_test,file_storage_test,local_executor_test,mysqldump_test,optimizer_cost_test,optimizer_rules_test,parser_token_test,prometheus_test,query_cache_config_test,query_cache_test,server_health_test,slow_query_log_test,types_value_test,vectorization_test"
    "集成测试-核心:executor_test,planner_test,page_test"
    "集成测试-SQL功能:foreign_key_test,fk_actions_test,server_integration_test,upsert_test,mysql_compatibility_test,savepoint_test,session_config_test,openclaw_api_test"
    "集成测试-存储:query_cache_test,optimizer_stats_test,checksum_corruption_test,columnar_storage_test,parquet_test,storage_integration_test"
    "性能测试:performance_test,batch_insert_test,autoinc_test,index_integration_test"
    "TPC-H测试:tpch_test,tpch_benchmark,tpch_full_test,tpch_compliance_test,tpch_sf03_test,tpch_sf1_test,tpch_text_index_test,tpch_index_test,tpch_comparison_test"
    "异常测试-并发:mvcc_concurrency_test,snapshot_isolation_test,concurrency_stress_test"
    "异常测试-隔离级别:transaction_isolation_test,transaction_timeout_test"
    "异常测试-数据处理:boundary_test,null_handling_test,aggregate_type_test,error_handling_test,datetime_type_test"
    "异常测试-查询:join_test,set_operations_test,view_test,window_function_test"
    "异常测试-约束:fk_constraint_test,catalog_consistency_test"
    "压力测试:chaos_test,crash_recovery_test,stress_test,kill_stress_test,production_scenario_test,wal_deterministic_test,wal_fuzz_test"
    "异常测试-稳定性:long_run_stability_test,qps_benchmark_test"
    "异常测试-崩溃注入:crash_injection_test"
    "CI测试:ci_test"
    "其他测试:binary_format_test,wal_integration_test,distributed_transaction_test"
    "安全测试:auth_rbac_test,logging_test"
    "教学场景测试:teaching_scenario_test,teaching_scenario_client_server_test"
    "工具测试:physical_backup_test"
    "向量检索测试:vector_storage_integration_test,hybrid_search_integration_test"
)

TOTAL_PASSED=0
TOTAL_FAILED=0
TOTAL_IGNORED=0
START_TIME=$(date +%s)

# 运行单个测试文件
run_test() {
    local test_path=$1
    local test_name=$(basename "$test_path")
    local test_start=$(date +%s)
    
    printf "${BLUE}  ⏳ ${test_name}...${NC} "
    
    local output
    local exit_code
    output=$(cargo test --test "$test_path" -- --nocapture 2>&1)
    exit_code=$?
    
    local test_end=$(date +%s)
    local duration=$(((test_end - test_start) * 1000))
    
    local passed=0
    local failed=0
    local ignored=0
    
    while IFS= read -r line; do
        if [[ "$line" =~ test\ result: ]]; then
            if [[ "$line" =~ ([0-9]+)\ passed ]]; then
                passed=${BASH_REMATCH[1]}
            fi
            if [[ "$line" =~ ([0-9]+)\ failed ]]; then
                failed=${BASH_REMATCH[1]}
            fi
            if [[ "$line" =~ ([0-9]+)\ ignored ]]; then
                ignored=${BASH_REMATCH[1]}
            fi
        fi
    done <<< "$output"
    
    if [[ $exit_code -eq 0 ]] && [[ $failed -eq 0 ]]; then
        printf "\r${GREEN}  ✅ ${test_name}${NC} (${passed} passed, ${duration}ms)\n"
        return 0
    else
        printf "\r${RED}  ❌ ${test_name}${NC} (${failed} failed, ${duration}ms)\n"
        echo "$output" | tail -5 | sed 's/^/      /'
        return 1
    fi
}

# 运行类别
run_category() {
    local index=$1
    local category_info=${CATEGORIES[$index]}
    local category_name=${category_info%%:*}
    local test_files=${category_info#*:}
    
    IFS=',' read -ra TESTS <<< "$test_files"
    
    echo ""
    echo "================================================================"
    echo "📂 ${category_name} (${#TESTS[@]} tests)"
    echo "================================================================"
    
    local cat_passed=0
    local cat_failed=0
    
    for test_path in "${TESTS[@]}"; do
        if run_test "$test_path"; then
            ((cat_passed++))
        else
            ((cat_failed++))
        fi
    done
    
    echo ""
    echo "📊 ${category_name}: ${GREEN}${cat_passed} passed${NC}, ${RED}${cat_failed} failed${NC}"
    
    return $cat_failed
}

# 主流程
echo "╔══════════════════════════════════════════════════════════════════════════╗"
echo "║                  SQLRustGo 回归测试 - 分阶段执行                          ║"
echo "╚══════════════════════════════════════════════════════════════════════════╝"

# 检查是否指定了类别索引
if [[ -n "$1" ]]; then
    # 执行指定类别
    run_category $(($1 - 1))
    exit $?
fi

# 执行所有类别
TOTAL_CATEGORIES=${#CATEGORIES[@]}

for i in "${!CATEGORIES[@]}"; do
    ((i++))  # 从1开始计数
    if ! run_category $((i - 1)); then
        echo ""
        echo "❌ 类别 ${i} 有测试失败，是否继续? (y/n)"
        read -r continue
        if [[ "$continue" != "y" && "$continue" != "Y" ]]; then
            echo "中止执行"
            break
        fi
    fi
done

END_TIME=$(date +%s)
TOTAL_TIME=$((END_TIME - START_TIME))

echo ""
echo "================================================================"
echo "📈 最终汇总"
echo "================================================================"
echo "总耗时: ${TOTAL_TIME} 秒"
echo "================================================================"
