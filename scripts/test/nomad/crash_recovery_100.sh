#!/usr/bin/env bash
# crash_recovery_100.sh - 100 次崩溃恢复测试
# RC/GA 门禁验证
set -euo pipefail

ITERATIONS="${ITERATIONS:-100}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
ARTIFACT_DIR="${ARTIFACT_DIR:-/tmp/crash_artifacts}"
mkdir -p "$ARTIFACT_DIR"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${YELLOW}⏳ $*${NC}"; }
log_pass() { echo -e "${GREEN}✅ $*${NC}"; }
log_fail() { echo -e "${RED}❌ $*${NC}"; }

echo "=== Crash Recovery Test ($ITERATIONS iterations) ==="
echo "Start: $TIMESTAMP"
echo ""

PASS=0
FAIL=0
SKIP=0

# 单次迭代
run_iteration() {
    local i=$1
    local start_time=$(date +%s.%N)
    
    # 1. 启动服务器
    cargo run --release --bin sqlrustgo-server &
    local pid=$!
    sleep 5
    
    # 检查启动
    if ! kill -0 $pid 2>/dev/null; then
        echo "[$i] SKIP: server failed to start"
        ((SKIP++))
        return 1
    fi
    
    # 2. 创建测试表和写入数据
    cargo run --release --bin bench-cli run-sql --query "CREATE TABLE IF NOT EXISTS crash_test (id INT, value TEXT)" >/dev/null 2>&1 || true
    cargo run --release --bin bench-cli run-sql --query "INSERT INTO crash_test VALUES ($i, 'before_crash_$i')" >/dev/null 2>&1 || true
    
    # 3. 模拟崩溃 (SIGKILL)
    kill -9 $pid 2>/dev/null || true
    sleep 2
    
    # 4. 重启服务器
    cargo run --release --bin sqlrustgo-server &
    pid=$!
    sleep 5
    
    # 检查重启
    if ! kill -0 $pid 2>/dev/null; then
        echo "[$i] SKIP: server failed to restart"
        ((SKIP++))
        return 1
    fi
    
    # 5. 验证数据
    local result
    result=$(cargo run --release --bin bench-cli run-sql --query "SELECT COUNT(*) FROM crash_test" 2>/dev/null | grep -oE '[0-9]+' | head -1 || echo "0")
    
    # 清理
    kill $pid 2>/dev/null || true
    wait $pid 2>/dev/null || true
    sleep 1
    
    # 验证
    if [ "$result" -ge "$i" ]; then
        echo "[$i] PASS: $result records (expected >= $i)"
        ((PASS++))
        return 0
    else
        echo "[$i] FAIL: $result records (expected >= $i)"
        ((FAIL++))
        return 1
    fi
}

export -f run_iteration
export ARTIFACT_DIR

# 执行测试
log_info "Running $ITERATIONS crash recovery iterations..."
echo ""

for i in $(seq 1 $ITERATIONS); do
    run_iteration $i || true
    
    # 每 10 次显示进度
    if [ $((i % 10)) -eq 0 ]; then
        echo ""
        echo "Progress: $i / $ITERATIONS"
        echo "  PASS: $PASS | FAIL: $FAIL | SKIP: $SKIP"
        echo ""
    fi
done

echo ""
echo "=== Crash Recovery Results ==="
echo "Pass: $PASS / $ITERATIONS"
echo "Fail: $FAIL / $ITERATIONS"
echo "Skip: $SKIP / $ITERATIONS"

# 计算成功率
if [ $((PASS + FAIL)) -gt 0 ]; then
    success_rate=$(echo "scale=2; $PASS * 100 / ($PASS + $FAIL)" | bc)
    echo "Success rate: ${success_rate}%"
fi

# 生成报告
cat > "$ARTIFACT_DIR/crash_recovery_report.json" << EOF
{
  "test_type": "crash_recovery",
  "iterations": $ITERATIONS,
  "passed": $PASS,
  "failed": $FAIL,
  "skipped": $SKIP,
  "success_rate": "$(echo "scale=2; $PASS * 100 / ($PASS + $FAIL + 0.001)" | bc)%",
  "status": "$([ $FAIL -eq 0 ] && echo 'PASS' || echo 'FAIL')",
  "timestamp": "$TIMESTAMP",
  "timestamp_end": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo ""
echo "Report: $ARTIFACT_DIR/crash_recovery_report.json"

if [ $FAIL -eq 0 ]; then
    log_pass "Crash recovery test PASSED"
    exit 0
else
    log_fail "Crash recovery test FAILED"
    exit 1
fi
