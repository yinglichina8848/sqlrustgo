#!/usr/bin/env bash
# stability_short.sh - 短期稳定性测试 (2-24h 可配置)
# 用于 RC 门禁前的基础稳定性验证
set -euo pipefail

HOURS="${HOURS:-2}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
ARTIFACT_DIR="${ARTIFACT_DIR:-/tmp/stability_short_artifacts}"
SERVER_PID_FILE="$ARTIFACT_DIR/server.pid"
mkdir -p "$ARTIFACT_DIR"

log_info() { echo -e "\033[1;33m⏳ $*...\033[0m"; }
log_pass() { echo -e "\033[0;32m✅ $*...\033[0m"; }
log_fail() { echo -e "\033[0;31m❌ $*...\033[0m"; }

echo "=== Short Stability Test (${HOURS}h) ==="
echo "Start: $TIMESTAMP"

# 资源检查
check_resources() {
    local mem_gb=$(free -g | awk '/^Mem:/{print $2}')
    if [ "$mem_gb" -lt 4 ]; then
        log_fail "Insufficient memory: ${mem_gb}GB < 4GB"
        exit 1
    fi
}

# 启动服务器
start_server() {
    log_info "Starting server..."
    cargo run --release --bin sqlrustgo-server &
    local pid=$!
    sleep 10
    if kill -0 $pid 2>/dev/null; then
        echo $pid > "$SERVER_PID_FILE"
        log_pass "Server started (PID: $pid)"
        return 0
    fi
    log_fail "Server failed to start"
    return 1
}

# 停止服务器
stop_server() {
    if [ -f "$SERVER_PID_FILE" ]; then
        local pid=$(cat "$SERVER_PID_FILE")
        kill $pid 2>/dev/null || true
        rm -f "$SERVER_PID_FILE"
    fi
}

# 运行监控
run_monitoring() {
    local secs=$((HOURS * 3600))
    local end_time=$(($(date +%s) + secs))
    
    log_info "Monitoring for ${HOURS}h..."
    
    while [ $(date +%s) -lt $end_time ]; do
        # 混合负载
        for i in {1..30}; do
            (cargo run --release --bin bench-cli run-sql --query "SELECT $i" >/dev/null 2>&1 || true) &
        done
        
        sleep 1
        
        # 每分钟检查
        if [ $((($(date +%s) - (end_time - secs)) % 60)) -eq 0 ]; then
            if ! kill -0 $(cat "$SERVER_PID_FILE" 2>/dev/null) 2>/dev/null; then
                log_fail "Server crashed at $(date)"
                return 1
            fi
            echo "$(date +%H:%M:%S): OK" >> "$ARTIFACT_DIR/status.log"
        fi
    done
    
    log_pass "Monitoring completed"
    return 0
}

# 生成报告
generate_report() {
    local crashes=$(grep -c "CRASH" "$ARTIFACT_DIR/status.log" 2>/dev/null || echo "0")
    cat > "$ARTIFACT_DIR/stability_short_report.json" << EOF
{
  "test_type": "stability_short",
  "hours": $HOURS,
  "status": "PASS",
  "crashes": $crashes,
  "timestamp_start": "$TIMESTAMP",
  "timestamp_end": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
    log_pass "Report: $ARTIFACT_DIR/stability_short_report.json"
}

trap stop_server EXIT

check_resources
start_server || exit 1
run_monitoring && generate_report
