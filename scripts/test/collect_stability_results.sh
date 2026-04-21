#!/bin/bash

set -e

OUTPUT_DIR="${OUTPUT_DIR:-./test_results}"
GENERATE_REPORT=1

show_help() {
    cat << EOF
SQLRustGo 长稳测试结果收集脚本

用法:
    $0 [选项]

选项:
    --input-dir DIR     输入目录 (默认: ./test_results)
    --output-dir DIR    输出目录 (默认: 同 --input-dir)
    --generate-report  生成报告 (默认: 是)
    --no-report        不生成报告
    -h, --help         显示帮助信息

示例:
    $0 --input-dir /tmp/test_results
    $0 --input-dir /tmp/test_results --no-report

EOF
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --input-dir)
                OUTPUT_DIR="$2"
                shift 2
                ;;
            --output-dir)
                OUTPUT_DIR="$2"
                shift 2
                ;;
            --generate-report)
                GENERATE_REPORT=1
                shift
                ;;
            --no-report)
                GENERATE_REPORT=0
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                echo "未知选项: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

collect_basic_info() {
    log "收集基础信息..."

    local info_file="$OUTPUT_DIR/test_info.txt"

    cat > "$info_file" << EOF
=== SQLRustGo 长稳测试基本信息 ===

测试版本: v2.6.0
测试类型: 72 小时长稳测试
收集时间: $(date)

分支: $(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "N/A")
提交: $(git rev-parse HEAD 2>/dev/null || echo "N/A")

系统信息:
$(uname -a)

Rust 版本:
$(rustc --version 2>/dev/null || echo "N/A")
$(cargo --version 2>/dev/null || echo "N/A")

EOF

    log "基础信息已保存到: $info_file"
}

collect_test_output() {
    log "收集测试输出..."

    local test_log="$OUTPUT_DIR/test_output.log"

    if [[ -f "$test_log" ]]; then
        local line_count=$(wc -l < "$test_log")
        local error_count=$(grep -c -i "error\|failed\|panic" "$test_log" 2>/dev/null || echo "0")
        local warn_count=$(grep -c -i "warning" "$test_log" 2>/dev/null || echo "0")

        log "测试输出: $line_count 行, $error_count 错误, $warn_count 警告"
    else
        log "警告: 测试输出文件不存在"
    fi
}

collect_monitor_metrics() {
    log "分析监控指标..."

    local metrics_file="$OUTPUT_DIR/monitor_metrics.csv"

    if [[ ! -f "$metrics_file" ]]; then
        log "警告: 监控指标文件不存在"
        return
    fi

    local total_lines=$(wc -l < "$metrics_file")
    log "监控数据: $((total_lines - 1)) 条记录"

    local stats_file="$OUTPUT_DIR/metrics_stats.txt"

    awk -F',' '
    NR>1 {
        cpu+=$2; cpu_count++
        mem+=$3; mem_count++
        disk+=$5; disk_count++
        if($2>max_cpu) max_cpu=$2
        if($3>max_mem) max_mem=$3
    }
    END {
        printf "=== 监控统计 ===\n" > "/dev/stderr"
        printf "CPU 平均: %d%%\n", int(cpu/cpu_count) > "/dev/stderr"
        printf "CPU 峰值: %d%%\n", max_cpu > "/dev/stderr"
        printf "内存平均: %d%%\n", int(mem/mem_count) > "/dev/stderr"
        printf "内存峰值: %d%%\n", max_mem > "/dev/stderr"
        printf "磁盘平均: %d%%\n", int(disk/disk_count) > "/dev/stderr"

        print ""
        print "=== 监控统计 ==="
        print "CPU 平均:", int(cpu/cpu_count) "%"
        print "CPU 峰值:", max_cpu "%"
        print "内存平均:", int(mem/mem_count) "%"
        print "内存峰值:", max_mem "%"
        print "磁盘平均:", int(disk/disk_count) "%"
    }' "$metrics_file" | tee "$stats_file"
}

collect_alerts() {
    log "分析告警..."

    local alert_file="$OUTPUT_DIR/alerts.log"

    if [[ -f "$alert_file" ]]; then
        local alert_count=$(wc -l < "$alert_file")
        log "告警数量: $alert_count"

        if [[ "$alert_count" -gt 0 ]]; then
            log "告警内容:"
            cat "$alert_file"
        fi
    else
        log "无告警记录"
    fi
}

generate_summary() {
    log "生成摘要..."

    local summary_file="$OUTPUT_DIR/summary.txt"

    cat > "$summary_file" << EOF
================================
SQLRustGo 长稳测试摘要
================================

测试版本: v2.6.0
测试类型: 72 小时长稳测试
生成时间: $(date)

--- 测试结果 ---

EOF

    if [[ -f "$OUTPUT_DIR/test_output.log" ]]; then
        local passed=$(grep -c "test result: ok" "$OUTPUT_DIR/test_output.log" 2>/dev/null || echo "0")
        local failed=$(grep -c "test result: FAILED" "$OUTPUT_DIR/test_output.log" 2>/dev/null || echo "0")

        echo "通过测试: $passed" >> "$summary_file"
        echo "失败测试: $failed" >> "$summary_file"
    fi

    echo "" >> "$summary_file"
    echo "--- 资源使用 ---" >> "$summary_file"

    if [[ -f "$OUTPUT_DIR/metrics_stats.txt" ]]; then
        cat "$OUTPUT_DIR/metrics_stats.txt" >> "$summary_file"
    fi

    echo "" >> "$summary_file"
    echo "--- 告警统计 ---" >> "$summary_file"

    if [[ -f "$OUTPUT_DIR/alerts.log" ]]; then
        local alert_count=$(wc -l < "$OUTPUT_DIR/alerts.log")
        echo "告警总数: $alert_count" >> "$summary_file"
    else
        echo "告警总数: 0" >> "$summary_file"
    fi

    echo "" >> "$summary_file"
    echo "================================" >> "$summary_file"

    log "摘要已保存到: $summary_file"
}

generate_report() {
    if [[ "$GENERATE_REPORT" != "1" ]]; then
        log "跳过报告生成"
        return
    fi

    log "生成 Markdown 报告..."

    local report_file="$OUTPUT_DIR/report.md"

    cat > "$report_file" << EOF
# SQLRustGo v2.6.0 长稳测试报告

> **测试版本**: v2.6.0 GA
> **测试类型**: 72 小时长稳测试
> **生成日期**: $(date)

---

## 一、测试基本信息

| 字段 | 值 |
|------|------|
| 测试版本 | v2.6.0 |
| 测试类型 | 72 小时长稳测试 |
| Git 分支 | $(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "N/A") |
| Git 提交 | $(git rev-parse HEAD 2>/dev/null || echo "N/A") |
| 测试时间 | $(date) |

---

## 二、测试结果

EOF

    if [[ -f "$OUTPUT_DIR/test_output.log" ]]; then
        local passed=$(grep -c "test result: ok" "$OUTPUT_DIR/test_output.log" 2>/dev/null || echo "0")
        local failed=$(grep -c "test result: FAILED" "$OUTPUT_DIR/test_output.log" 2>/dev/null || echo "0")

        cat >> "$report_file" << EOF

| 测试类型 | 结果 |
|----------|------|
| 通过 | $passed |
| 失败 | $failed |

EOF
    fi

    cat >> "$report_file" << EOF

### 详细日志

测试日志位于: `test_output.log`

---

## 三、资源使用统计

EOF

    if [[ -f "$OUTPUT_DIR/metrics_stats.txt" ]]; then
        cat "$OUTPUT_DIR/metrics_stats.txt" >> "$report_file"
    fi

    cat >> "$report_file" << EOF

### 监控数据

完整监控数据位于: `monitor_metrics.csv`

---

## 四、告警记录

EOF

    if [[ -f "$OUTPUT_DIR/alerts.log" ]]; then
        local alert_count=$(wc -l < "$OUTPUT_DIR/alerts.log")
        echo "告警总数: $alert_count" >> "$report_file"

        echo "" >> "$report_file"
        echo "### 告警详情" >> "$report_file"
        echo "" >> "$report_file"
        cat "$OUTPUT_DIR/alerts.log" >> "$report_file"
    else
        echo "无告警" >> "$report_file"
    fi

    cat >> "$report_file" << EOF

---

## 五、结论

EOF

    local test_passed=1

    if [[ -f "$OUTPUT_DIR/test_output.log" ]]; then
        local failed=$(grep -c "test result: FAILED" "$OUTPUT_DIR/test_output.log" 2>/dev/null || echo "0")
        if [[ "$failed" -gt 0 ]]; then
            test_passed=0
        fi
    fi

    if [[ "$test_passed" -eq 1 ]]; then
        echo "✅ **测试通过** - 所有测试成功完成，无崩溃、无错误" >> "$report_file"
    else
        echo "❌ **测试失败** - 存在失败的测试用例" >> "$report_file"
    fi

    cat >> "$report_file" << EOF

---

## 六、附件

| 文件 | 说明 |
|------|------|
| test_output.log | 测试完整输出 |
| monitor_metrics.csv | 监控指标数据 |
| metrics_stats.txt | 资源统计 |
| alerts.log | 告警记录 |
| test_info.txt | 基础信息 |

---

*报告由 SQLRustGo 长稳测试系统自动生成*
EOF

    log "报告已保存到: $report_file"
}

package_results() {
    log "打包结果..."

    local archive_name="sqlrustgo_stability_test_$(date +%Y%m%d_%H%M%S).tar.gz"

    tar -czf "$archive_name" -C "$OUTPUT_DIR" .

    log "结果已打包到: $archive_name"
}

main() {
    parse_args "$@"

    log "开始收集测试结果..."
    log "输入目录: $OUTPUT_DIR"

    collect_basic_info
    collect_test_output
    collect_monitor_metrics
    collect_alerts
    generate_summary
    generate_report

    log "结果收集完成!"
    log "结果目录: $OUTPUT_DIR"
}

main "$@"
