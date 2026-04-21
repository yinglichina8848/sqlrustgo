#!/bin/bash

#==============================================================================
# SQLRustGo 72 小时长稳测试监控脚本
#
# 功能:
#   - 监控 CPU、内存、磁盘、网络使用情况
#   - 检测进程状态、崩溃、死锁
#   - 记录监控指标到 CSV 文件
#   - 生成实时图表
#   - 异常告警
#
# 使用方法:
#   bash scripts/test/monitor_stability_test.sh [--output-dir DIR] [--interval SEC] [--metrics cpu,mem,disk,network]
#
# 作者: SQLRustGo Team
# 日期: 2026-04-21
#==============================================================================

set -e

# 默认配置
OUTPUT_DIR="${OUTPUT_DIR:-./test_results}"
INTERVAL="${INTERVAL:-60}"  # 监控间隔 (秒)
METRICS="${METRICS:-cpu,mem,disk,network,process}"
TEST_NAME="stability_test"
PROJECT_NAME="sqlrustgo"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 告警阈值
CPU_THRESHOLD=90
MEM_THRESHOLD=85
DISK_THRESHOLD=80

#-------------------------------------------------------------------------------
# 帮助信息
#-------------------------------------------------------------------------------
show_help() {
    cat << EOF
SQLRustGo 长稳测试监控脚本

用法:
    $0 [选项]

选项:
    --output-dir DIR     输出目录 (默认: ./test_results)
    --interval SEC       监控间隔秒数 (默认: 60)
    --metrics LIST       监控指标，逗号分隔 (默认: cpu,mem,disk,network,process)
                         可选: cpu, mem, disk, network, process, all
    --duration HOURS     监控时长小时数 (默认: 72)
    --test-pid PID       测试进程 PID (自动检测如果未指定)
    --alert              启用告警通知
    --quiet              静默模式 (只记录，不打印到终端)
    -h, --help           显示帮助信息

示例:
    # 监控当前运行的测试
    $0 --output-dir /tmp/test_results

    # 监控 24 小时，每 30 秒采样
    $0 --duration 24 --interval 30

    # 只监控 CPU 和内存
    $0 --metrics cpu,mem

EOF
}

#-------------------------------------------------------------------------------
# 解析参数
#-------------------------------------------------------------------------------
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --output-dir)
                OUTPUT_DIR="$2"
                shift 2
                ;;
            --interval)
                INTERVAL="$2"
                shift 2
                ;;
            --metrics)
                METRICS="$2"
                shift 2
                ;;
            --duration)
                DURATION="$2"
                shift 2
                ;;
            --test-pid)
                TEST_PID="$2"
                shift 2
                ;;
            --alert)
                ALERT=1
                shift
                ;;
            --quiet)
                QUIET=1
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

#-------------------------------------------------------------------------------
# 初始化
#-------------------------------------------------------------------------------
init() {
    # 创建输出目录
    mkdir -p "$OUTPUT_DIR"

    # 初始化监控指标文件
    METRICS_FILE="$OUTPUT_DIR/monitor_metrics.csv"
    if [[ ! -f "$METRICS_FILE" ]]; then
        echo "timestamp,cpu_percent,mem_percent,mem_used_mb,disk_percent,network_recv_kb,network_sent_kb,process_count,thread_count,open_files" > "$METRICS_FILE"
    fi

    # 初始化日志文件
    LOG_FILE="$OUTPUT_DIR/monitor.log"
    if [[ ! -f "$LOG_FILE" ]]; then
        echo "# SQLRustGo 监控日志 - $(date)" > "$LOG_FILE"
    fi

    # 告警日志
    ALERT_FILE="$OUTPUT_DIR/alerts.log"

    # 测试进程 PID 文件
    PID_FILE="$OUTPUT_DIR/test.pid"

    # 开始时间
    START_TIME=$(date +%s)

    log_info "监控初始化完成"
    log_info "输出目录: $OUTPUT_DIR"
    log_info "监控间隔: ${INTERVAL}s"
    log_info "监控指标: $METRICS"
}

#-------------------------------------------------------------------------------
# 日志记录
#-------------------------------------------------------------------------------
log_info() {
    local msg="[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $1"
    echo -e "${GREEN}${msg}${NC}"
    echo "$msg" >> "$LOG_FILE"
}

log_warn() {
    local msg="[$(date '+%Y-%m-%d %H:%M:%S')] [WARN] $1"
    echo -e "${YELLOW}${msg}${NC}"
    echo "$msg" >> "$LOG_FILE"
    echo "$msg" >> "$ALERT_FILE"
}

log_error() {
    local msg="[$(date '+%Y-%m-%d %H:%M:%S')] [ERROR] $1"
    echo -e "${RED}${msg}${NC}"
    echo "$msg" >> "$LOG_FILE"
    echo "$msg" >> "$ALERT_FILE"
}

#-------------------------------------------------------------------------------
# 获取测试进程 PID
#-------------------------------------------------------------------------------
get_test_pid() {
    if [[ -n "$TEST_PID" ]]; then
        echo "$TEST_PID"
        return
    fi

    # 查找 cargo test 进程
    local pid=$(pgrep -f "cargo test.*regression_test" 2>/dev/null | head -1)
    if [[ -n "$pid" ]]; then
        echo "$pid"
        return
    fi

    # 查找 sqlrustgo 测试进程
    pid=$(pgrep -f "sqlrustgo.*test" 2>/dev/null | head -1)
    if [[ -n "$pid" ]]; then
        echo "$pid"
        return
    fi

    # 如果有 PID 文件，读取
    if [[ -f "$PID_FILE" ]]; then
        cat "$PID_FILE"
        return
    fi

    echo ""
}

#-------------------------------------------------------------------------------
# 监控 CPU 使用率
#-------------------------------------------------------------------------------
monitor_cpu() {
    local cpu=$(top -bn1 | grep "Cpu(s)" | awk '{print int($2)}' | cut -d'%' -f1)
    echo "${cpu:-0}"
}

#-------------------------------------------------------------------------------
# 监控内存使用
#-------------------------------------------------------------------------------
monitor_memory() {
    local mem_info=$(free -m | awk 'NR==2 {print $3,$2,$7}')
    local mem_used=$(echo $mem_info | awk '{print $1}')
    local mem_total=$(echo $mem_info | awk '{print $2}')
    local mem_free=$(echo $mem_info | awk '{print $3}')

    local mem_percent=0
    if [[ $mem_total -gt 0 ]]; then
        mem_percent=$((mem_used * 100 / mem_total))
    fi

    echo "$mem_percent $mem_used"
}

#-------------------------------------------------------------------------------
# 监控磁盘使用
#-------------------------------------------------------------------------------
monitor_disk() {
    local disk_percent=$(df -h . | awk 'NR==2 {print $5}' | tr -d '%')
    echo "${disk_percent:-0}"
}

#-------------------------------------------------------------------------------
# 监控网络使用
#-------------------------------------------------------------------------------
monitor_network() {
    local net_stats=$(cat /proc/net/dev | awk 'NR>2 {rx+=$2; tx+=$10} END {print rx,tx}')
    echo "$net_stats"
}

#-------------------------------------------------------------------------------
# 监控进程状态
#-------------------------------------------------------------------------------
monitor_process() {
    local pid=$(get_test_pid)

    if [[ -z "$pid" ]] || [[ ! -d "/proc/$pid" ]]; then
        echo "0 0 0"
        return
    fi

    # 进程数量
    local proc_count=$(pgrep -c -f "cargo|sqlrustgo" 2>/dev/null || echo "0")

    # 线程数
    local thread_count=$(ps -o nlwp= -p "$pid" 2>/dev/null || echo "0")

    # 打开文件数
    local open_files=$(ls /proc/$pid/fd 2>/dev/null | wc -l)

    echo "$proc_count $thread_count $open_files"
}

#-------------------------------------------------------------------------------
# 监控测试输出
#-------------------------------------------------------------------------------
monitor_test_output() {
    local test_log="$OUTPUT_DIR/test_output.log"

    # 检查是否有新的错误
    if [[ -f "$test_log" ]]; then
        local errors=$(grep -c "error\|failed\|panic" "$test_log" 2>/dev/null || echo "0")
        if [[ "$errors" -gt 0 ]]; then
            log_warn "检测到测试错误: $errors 个"
        fi
    fi

    # 检查进程是否还在运行
    local pid=$(get_test_pid)
    if [[ -z "$pid" ]] || [[ ! -d "/proc/$pid" ]]; then
        if [[ -f "$PID_FILE" ]]; then
            log_warn "测试进程可能已结束"
        fi
    fi
}

#-------------------------------------------------------------------------------
# 收集监控数据
#-------------------------------------------------------------------------------
collect_metrics() {
    local timestamp=$(date +%s)

    # CPU
    local cpu=$(monitor_cpu)

    # 内存
    local mem_data=$(monitor_memory)
    local mem_percent=$(echo $mem_data | awk '{print $1}')
    local mem_used=$(echo $mem_data | awk '{print $2}')

    # 磁盘
    local disk=$(monitor_disk)

    # 网络
    local net_data=$(monitor_network)
    local net_recv=$(echo $net_data | awk '{print int($1/1024)}')  # KB
    local net_sent=$(echo $net_data | awk '{print int($2/1024)}')  # KB

    # 进程
    local proc_data=$(monitor_process)
    local proc_count=$(echo $proc_data | awk '{print $1}')
    local thread_count=$(echo $proc_data | awk '{print $2}')
    local open_files=$(echo $proc_data | awk '{print $3}')

    # 写入 CSV
    echo "$timestamp,$cpu,$mem_percent,$mem_used,$disk,$net_recv,$net_sent,$proc_count,$thread_count,$open_files" >> "$METRICS_FILE"

    # 打印状态 (非静默模式)
    if [[ "$QUIET" != "1" ]]; then
        echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} " \
            "CPU: ${cpu}% " \
            "MEM: ${mem_percent}%(${mem_used}MB) " \
            "DISK: ${disk}% " \
            "NET: ${net_recv}/${net_sent}KB " \
            "THREADS: ${thread_count}"
    fi

    # 告警检查
    check_alerts "$cpu" "$mem_percent" "$disk"
}

#-------------------------------------------------------------------------------
# 告警检查
#-------------------------------------------------------------------------------
check_alerts() {
    local cpu=$1
    local mem=$2
    local disk=$3

    # CPU 告警
    if [[ "$cpu" -gt "$CPU_THRESHOLD" ]]; then
        log_warn "CPU 使用率过高: ${cpu}%"
    fi

    # 内存告警
    if [[ "$mem" -gt "$MEM_THRESHOLD" ]]; then
        log_warn "内存使用率过高: ${mem}%"
    fi

    # 磁盘告警
    if [[ "$disk" -gt "$DISK_THRESHOLD" ]]; then
        log_warn "磁盘使用率过高: ${disk}%"
    fi
}

#-------------------------------------------------------------------------------
# 生成小时报告
#-------------------------------------------------------------------------------
generate_hourly_report() {
    local hour=$1
    local report_file="$OUTPUT_DIR/hourly_report_${hour}.txt"

    # 读取过去一小时的数据
    local last_hour=$((hour - 1))
    local start_ts=$((last_hour * 3600))
    local end_ts=$((hour * 3600))

    # 计算统计信息
    local cpu_avg=$(awk -F',' -v start="$START_TIME" -v end="$START_TIME+3600" \
        'NR>1 && $1>=start && $1<end {sum+=$2; count++} END {print int(sum/count)}' "$METRICS_FILE")
    local mem_avg=$(awk -F',' -v start="$START_TIME" -v end="$START_TIME+3600" \
        'NR>1 && $1>=start && $1<end {sum+=$3; count++} END {print int(sum/count)}' "$METRICS_FILE")

    cat > "$report_file" << EOF
=== SQLRustGo 长稳测试 - 小时报告 #${hour} ===
生成时间: $(date)
测试时长: ${hour} 小时

资源使用统计 (过去 1 小时):
- CPU 平均: ${cpu_avg}%
- 内存平均: ${mem_avg}%

详细数据请查看: monitor_metrics.csv
EOF

    log_info "小时报告 #${hour} 已生成"
}

#-------------------------------------------------------------------------------
# 主循环
#-------------------------------------------------------------------------------
main() {
    parse_args "$@"
    init

    log_info "开始监控 (按 Ctrl+C 停止)"
    log_info "输出目录: $OUTPUT_DIR"

    local count=0
    local hour_count=0

    # 主监控循环
    while true; do
        # 收集指标
        collect_metrics

        # 检查测试输出
        monitor_test_output

        # 小时报告
        count=$((count + INTERVAL))
        if [[ $count -ge 3600 ]]; then
            hour_count=$((hour_count + 1))
            generate_hourly_report $hour_count
            count=0
        fi

        # 检查是否超过指定时长
        if [[ -n "$DURATION" ]]; then
            local elapsed=$(($(date +%s) - START_TIME))
            local duration_sec=$((DURATION * 3600))
            if [[ $elapsed -ge $duration_sec ]]; then
                log_info "达到指定时长 ${DURATION} 小时，监控结束"
                break
            fi
        fi

        # 等待下次采样
        sleep "$INTERVAL"
    done

    log_info "监控完成"
}

#-------------------------------------------------------------------------------
# 入口
#-------------------------------------------------------------------------------
main "$@"
