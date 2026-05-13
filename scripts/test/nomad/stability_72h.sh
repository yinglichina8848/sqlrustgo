#!/usr/bin/env bash
# stability_72h.sh - 72 小时稳定性测试
# RC/GA 门禁验证
set -euo pipefail

HOURS="${HOURS:-72}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
ARTIFACT_DIR="${ARTIFACT_DIR:-/tmp/stability_artifacts}"
SERVER_PID_FILE="$ARTIFACT_DIR/server.pid"
mkdir -p "$ARTIFACT_DIR"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${YELLOW}⏳ $*${NC}"; }
log_pass() { echo -e "${GREEN}✅ $*${NC}"; }
log_fail() { echo -e "${RED}❌ $*${NC}"; }

echo "=== Stability Test ($HOURS hours) ==="
echo "Start: $TIMESTAMP"
echo "Artifact dir: $ARTIFACT_DIR"
echo ""

# 检查资源
check_resources() {
    local mem_gb=$(free -g | awk '/^Mem:/{print $2}')
    local disk_gb=$(df -BG / | awk 'NR==2{gsub(/G/,"",$4); print $4}')
    
    log_info "Checking resources..."
    echo "  Memory: ${mem_gb}GB"
    echo "  Disk: ${disk_gb}GB"
    
    if [ "$mem_gb" -lt 8 ]; then
        log_fail "Insufficient memory: ${mem_gb}GB < 8GB"
        exit 1
    fi
}

# 启动服务器
start_server() {
    log_info "Starting SQLRustGo server..."
    
    # 使用 release 模式启动
    cargo run --release --bin sqlrustgo-server &
    local pid=$!
    
    # 等待启动
    sleep 10
    
    # 验证启动
    if kill -0 $pid 2>/dev/null; then
        echo $pid > "$SERVER_PID_FILE"
        log_pass "Server started (PID: $pid)"
        return 0
    else
        log_fail "Failed to start server"
        exit 1
    fi
}

# 停止服务器
stop_server() {
    if [ -f "$SERVER_PID_FILE" ]; then
        local pid=$(cat "$SERVER_PID_FILE")
        if kill -0 $pid 2>/dev/null; then
            kill $pid 2>/dev/null || true
            sleep 2
            kill -9 $pid 2>/dev/null || true
        fi
        rm -f "$SERVER_PID_FILE"
    fi
}

# 运行监控
run_monitoring() {
    local duration_secs=$((HOURS * 3600))
    local start_time=$(date +%s)
    local end_time=$((start_time + duration_secs))
    local check_count=0
    
    log_info "Running stability monitoring for ${HOURS}h..."
    echo "End time: $(date -u -d @$end_time +%Y-%m-%dT%H:%M:%SZ)"
    
    while [ $(date +%s) -lt $end_time ]; do
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))
        local remaining=$((end_time - current_time))
        
        # 每分钟执行一次混合负载
        for i in {1..60}; do
            # SELECT 负载 (异步)
            (cargo run --release --bin bench-cli run-sql --query "SELECT 1" >/dev/null 2>&1 || true) &
        done
        
        # 等待一秒
        sleep 1
        
        # 每分钟记录一次指标
        if [ $((elapsed % 60)) -eq 0 ]; then
            ((check_count++))
            
            # 检查服务器存活
            if [ -f "$SERVER_PID_FILE" ]; then
                local pid=$(cat "$SERVER_PID_FILE")
                if ! kill -0 $pid 2>/dev/null; then
                    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ): CRASH" >> "$ARTIFACT_DIR/crash.log"
                    log_fail "Server crashed at $(date)"
                    stop_server
                    return 1
                fi
            fi
            
            # 记录状态
            echo "$(date -u +%Y-%m-%dT%H:%M:%SZ): OK (${elapsed}s elapsed)" >> "$ARTIFACT_DIR/status.log"
            
            # 显示进度
            local hours_rem=$((remaining / 3600))
            local mins_rem=$(((remaining % 3600) / 60))
            echo -ne "\r  Progress: ${check_count}/$((HOURS * 60)) checks, ${hours_rem}h ${mins_rem}m remaining"
        fi
        
        # 每小时额外检查
        if [ $((elapsed % 3600)) -eq 0 ] && [ $elapsed -gt 0 ]; then
            # 内存使用
            local mem_usage=$(free | awk '/^Mem:/{printf "%.1f", $3/$2 * 100}')
            echo "$(date -u +%Y-%m-%dT%H:%M:%SZ): MEM=${mem_usage}%" >> "$ARTIFACT_DIR/metrics.log"
        fi
    done
    
    echo ""
    log_pass "Monitoring completed"
    return 0
}

# 生成报告
generate_report() {
    log_info "Generating report..."
    
    python3 << EOF
import json
import os

artifact_dir = os.environ.get('ARTIFACT_DIR', '/tmp/stability_artifacts')
hours = int(os.environ.get('HOURS', '72'))

status_log = []
crash_log = []
metrics_log = []

if os.path.exists(f'{artifact_dir}/status.log'):
    with open(f'{artifact_dir}/status.log') as f:
        status_log = [l.strip() for l in f.readlines()]

if os.path.exists(f'{artifact_dir}/crash.log'):
    with open(f'{artifact_dir}/crash.log') as f:
        crash_log = [l.strip() for l in f.readlines()]

if os.path.exists(f'{artifact_dir}/metrics.log'):
    with open(f'{artifact_dir}/metrics.log') as f:
        metrics_log = [l.strip() for l in f.readlines()]

report = {
    'test_type': 'stability',
    'hours': hours,
    'status': 'PASS' if len(crash_log) == 0 else 'FAIL',
    'crashes': len(crash_log),
    'crash_log': crash_log,
    'status_checks': len(status_log),
    'metrics': metrics_log[-10:] if metrics_log else [],
    'timestamp_start': os.environ.get('TIMESTAMP', ''),
    'timestamp_end': '$(date -u +%Y-%m-%dT%H:%M:%SZ)'
}

with open(f'{artifact_dir}/stability_report.json', 'w') as f:
    json.dump(report, f, indent=2)

print(f"Stability Test Results:")
print(f"  Status: {report['status']}")
print(f"  Crashes: {len(crash_log)}")
print(f"  Status checks: {len(status_log)}")
if crash_log:
    print(f"  Crash log: {crash_log}")
EOF

    cp "$ARTIFACT_DIR/stability_report.json" "$ARTIFACT_DIR/stability_report.json.bak" 2>/dev/null || true
}

# 主流程
check_resources
stop_server  # 确保没有残留

trap stop_server EXIT

start_server || exit 1

if run_monitoring; then
    generate_report
    log_pass "Stability test PASSED"
    exit 0
else
    generate_report
    log_fail "Stability test FAILED"
    exit 1
fi
