#!/usr/bin/env bash
# Aggregate Coverage Script - 聚合所有模块的 llvm-cov JSON 报告
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
COVERAGE_DIR="$PROJECT_ROOT/artifacts/coverage"

# 按模块优先级分组
WAVE1=(
    "common"
    "types"
    "expr"
    "catalog"
    "query-stats"
    "information-schema"
    "telemetry"
    "security"
    "tools"
)

WAVE2=(
    "parser"
    "planner"
    "optimizer"
    "executor"
    "network"
    "mysql-server"
    "transaction"
    "server"
)

WAVE3=(
    "storage"
    "distributed"
    "vector"
    "graph"
    "sql-corpus"
)

WAVE4=(
    "agentsql"
    "gmp"
    "rag"
    "qmd-bridge"
    "unified-storage"
    "unified-query"
)

# 模块 -> Cargo package name 映射
get_package_name() {
    local module="$1"
    case "$module" in
        common)         echo "sqlrustgo-common" ;;
        types)           echo "sqlrustgo-types" ;;
        expr)            echo "sqlrustgo-expr" ;;
        catalog)         echo "sqlrustgo-catalog" ;;
        query-stats)     echo "sqlrustgo-query-stats" ;;
        information-schema) echo "sqlrustgo-information-schema" ;;
        telemetry)       echo "sqlrustgo-telemetry" ;;
        security)        echo "sqlrustgo-security" ;;
        tools)           echo "sqlrustgo-tools" ;;
        parser)          echo "sqlrustgo-parser" ;;
        planner)         echo "sqlrustgo-planner" ;;
        optimizer)       echo "sqlrustgo-optimizer" ;;
        executor)        echo "sqlrustgo-executor" ;;
        network)         echo "sqlrustgo-network" ;;
        mysql-server)    echo "sqlrustgo-mysql-server" ;;
        transaction)     echo "sqlrustgo-transaction" ;;
        server)          echo "sqlrustgo-server" ;;
        storage)         echo "sqlrustgo-storage" ;;
        distributed)     echo "sqlrustgo-distributed" ;;
        vector)          echo "sqlrustgo-vector" ;;
        graph)           echo "sqlrustgo-graph" ;;
        sql-corpus)      echo "sqlrustgo-sql-corpus" ;;
        agentsql)        echo "sqlrustgo-agentsql" ;;
        gmp)             echo "sqlrustgo-gmp" ;;
        rag)             echo "sqlrustgo-rag" ;;
        qmd-bridge)      echo "sqlrustgo-qmd-bridge" ;;
        unified-storage) echo "sqlrustgo-unified-storage" ;;
        unified-query)   echo "sqlrustgo-unified-query" ;;
        *)               echo "sqlrustgo-$module" ;;
    esac
}

# 从 JSON 提取行覆盖率百分比
get_coverage_pct() {
    local json_file="$1"
    if [[ ! -f "$json_file" ]]; then
        echo "MISSING" ; return 0
    fi
    local size=$(wc -c < "$json_file")
    if [[ "$size" -lt 1000 ]]; then
        echo "INVALID" ; return 0
    fi
    if ! python3 -c "
import json, sys
d = json.load(open('$json_file'))
t = d['data'][0]['totals']['lines']
pct = t.get('percent', 0)
print(f'{pct:.2f}')
" ; then
        echo "PARSE_ERR" ; return 0
    fi
}

# 计算加权平均覆盖率
calculate_weighted_avg() {
    local modules=("$@")
    local total_weight=0
    local weighted_sum=0

    for mod in "${modules[@]}"; do
        local json_file="$COVERAGE_DIR/${mod}.json"
        local pct
        pct=$(get_coverage_pct "$json_file" 2>/dev/null)
        if [[ "$pct" != "MISSING" && "$pct" != "INVALID" && "$pct" != "PARSE_ERR" && "$pct" != "0.00" ]]; then
            if python3 -c "exit(0 if float('$pct') >= 0 else 1)" 2>/dev/null; then
                local weight=1
                weighted_sum=$(python3 -c "print($weighted_sum + $pct * $weight)")
                total_weight=$((total_weight + weight))
            fi
        fi
    done

    if [[ "$total_weight" -eq 0 ]]; then
        echo "0.00"
    else
        python3 -c "print(f'{$weighted_sum / $total_weight:.2f}')"
    fi
}

# 生成 Markdown 表格
generate_report() {
    local output_file="${1:-}"
    local report_date=$(date '+%Y-%m-%d %H:%M:%S')

    {
        echo "# Coverage Report"
        echo ""
        echo "Generated: $report_date"
        echo "Tool: cargo llvm-cov"
        echo ""
        echo "## Summary by Wave"
        echo ""

        local all_modules=("${WAVE1[@]}" "${WAVE2[@]}" "${WAVE3[@]}" "${WAVE4[@]}")

        echo "| Module | Coverage | Package | Status |"
        echo "|--------|----------|---------|--------|"

        local total_pct=0
        local total_count=0

        for mod in "${all_modules[@]}"; do
            local pkg=$(get_package_name "$mod")
            local json_file="$COVERAGE_DIR/${mod}.json"
            local pct
        pct=$(get_coverage_pct "$json_file" 2>/dev/null)
            local status="✅"
            if [[ "$pct" == "MISSING" || "$pct" == "INVALID" || "$pct" == "PARSE_ERR" ]]; then
                status="❌"
            elif [[ "$pct" == "—" ]]; then
                status="⏳"
            elif python3 -c "exit(0 if $pct >= 60 else 1)" 2>/dev/null; then
                :
            else
                status="⚠️"
            fi
            echo "| $mod | $pct% | $pkg | $status |"
            if [[ "$pct" != "—" && "$pct" != "MISSING" && "$pct" != "INVALID" && "$pct" != "PARSE_ERR" ]]; then
                total_pct=$(python3 -c "print($total_pct + $pct)")
                total_count=$((total_count + 1))
            fi
        done

        echo ""
        echo "## Aggregate Statistics"
        echo ""

        if [[ "$total_count" -gt 0 ]]; then
            local avg=$(python3 -c "print(f'{$total_pct / $total_count:.2f}')")
            echo "- **Total modules**: ${#all_modules[@]}"
            echo "- **Modules with coverage**: $total_count"
            echo "- **Average coverage**: $avg%"
        fi

        echo ""
        echo "## Per-Wave Summary"
        echo ""

        echo "### Wave 1: Lightweight Modules"
        echo "| Module | Coverage |"
        echo "|--------|----------|"
        for mod in "${WAVE1[@]}"; do
            local pct=$(get_coverage_pct "$COVERAGE_DIR/${mod}.json" 2>/dev/null)
            echo "| $mod | $pct% |"
        done

        echo ""
        echo "### Wave 2: Medium Modules"
        echo "| Module | Coverage |"
        echo "|--------|----------|"
        for mod in "${WAVE2[@]}"; do
            local pct=$(get_coverage_pct "$COVERAGE_DIR/${mod}.json" 2>/dev/null)
            echo "| $mod | $pct% |"
        done

        echo ""
        echo "### Wave 3: Critical Modules"
        echo "| Module | Coverage |"
        echo "|--------|----------|"
        for mod in "${WAVE3[@]}"; do
            local pct=$(get_coverage_pct "$COVERAGE_DIR/${mod}.json" 2>/dev/null)
            echo "| $mod | $pct% |"
        done

        echo ""
        echo "### Wave 4: Utility Modules"
        echo "| Module | Coverage |"
        echo "|--------|----------|"
        for mod in "${WAVE4[@]}"; do
            local pct=$(get_coverage_pct "$COVERAGE_DIR/${mod}.json" 2>/dev/null)
            echo "| $mod | $pct% |"
        done

    } > "${output_file:=/dev/stdout}"
}

# 主报告生成
main() {
    echo "=== Coverage Aggregation ==="
    echo ""

    local all_modules=("${WAVE1[@]}" "${WAVE2[@]}" "${WAVE3[@]}" "${WAVE4[@]}")

    echo "Checking ${#all_modules[@]} modules..."
    echo ""

    for mod in "${all_modules[@]}"; do
        local json_file="$COVERAGE_DIR/${mod}.json"
        local pct
        pct=$(get_coverage_pct "$json_file" 2>/dev/null)
        printf "  %-20s %s%%\n" "$mod" "$pct"
    done

    echo ""
    generate_report
}

# 仅生成 Markdown 报告
report-only() {
    generate_report "$COVERAGE_DIR/coverage-report.md"
    echo "Report saved to: $COVERAGE_DIR/coverage-report.md"
}

# 列出缺失/无效的模块
check() {
    local all_modules=("${WAVE1[@]}" "${WAVE2[@]}" "${WAVE3[@]}" "${WAVE4[@]}")
    local missing=()
    local invalid=()

    for mod in "${all_modules[@]}"; do
        local json_file="$COVERAGE_DIR/${mod}.json"
        local pct
        pct=$(get_coverage_pct "$json_file" 2>/dev/null)
        if [[ "$pct" == "MISSING" || "$pct" == "INVALID" || "$pct" == "PARSE_ERR" ]]; then
            missing+=("$mod")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        echo "Missing/invalid modules: ${missing[*]}"
        return 1
    else
        echo "All modules have valid coverage data."
        return 0
    fi
}

case "${1:-main}" in
    main)    main ;;
    report)  report-only ;;
    check)   check ;;
    *)       echo "Usage: $0 {main|report|check}" ;;
esac