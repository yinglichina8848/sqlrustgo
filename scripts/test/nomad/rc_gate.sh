#!/usr/bin/env bash
# rc_gate.sh - RC Gate 完整测试入口
# Beta → RC 门禁，约 6 小时执行
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
ARTIFACT_DIR="${ARTIFACT_DIR:-/tmp/rc_gate_artifacts}"
mkdir -p "$ARTIFACT_DIR"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 测试状态
PASS=0
FAIL=0
SKIP=0
TOTAL_START=$(date +%s)

log_info() { echo -e "${YELLOW}⏳ [$(date +%H:%M:%S)] $*${NC}"; }
log_pass() { echo -e "${GREEN}✅ $*${NC}"; ((PASS++)); }
log_fail() { echo -e "${RED}❌ $*${NC}"; ((FAIL++)); }
log_skip() { echo -e "${YELLOW}⏭️  $*${NC}"; ((SKIP++)); }

header() {
    echo ""
    echo "============================================================"
    echo "  RC Gate - Beta → RC 门禁验证"
    echo "  Timestamp: $TIMESTAMP"
    echo "============================================================"
    echo ""
}

footer() {
    local total_secs=$(($(date +%s) - TOTAL_START))
    local hours=$((total_secs / 3600))
    local mins=$(((total_secs % 3600) / 60))
    local secs=$((total_secs % 60))
    
    echo ""
    echo "============================================================"
    echo "  RC Gate 完成报告"
    echo "============================================================"
    echo "  开始: $TIMESTAMP"
    echo "  结束: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "  耗时: ${hours}h ${mins}m ${secs}s"
    echo ""
    echo "  测试结果:"
    echo "    ✅ 通过: $PASS"
    echo "    ❌ 失败: $FAIL"
    echo "    ⏭️  跳过: $SKIP"
    echo ""
    
    # 生成 JSON 报告
    cat > "$ARTIFACT_DIR/rc_gate_report.json" << EOF
{
  "gate": "RC-GATE",
  "version": "v3.1.0",
  "timestamp_start": "$TIMESTAMP",
  "timestamp_end": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "duration_seconds": $total_secs,
  "status": "$([ $FAIL -eq 0 ] && echo "PASS" || echo "FAIL")",
  "results": {
    "passed": $PASS,
    "failed": $FAIL,
    "skipped": $SKIP
  },
  "tests": []
}
EOF
    
    if [ $FAIL -eq 0 ]; then
        echo -e "${GREEN}✅ RC Gate: PASSED - Ready for GA${NC}"
        exit 0
    else
        echo -e "${RED}❌ RC Gate: FAILED - $FAIL blocker(s)${NC}"
        exit 1
    fi
}

trap footer EXIT

header

# ============================================================
# Phase 1: 基础门禁 (约 30min)
# ============================================================
log_info "【Phase 1】基础门禁检查 (~30min)"

# 1.1 构建
log_info "  B1: cargo build --release --all-features"
if cargo build --release --all-features >/dev/null 2>&1; then
    log_pass "  B1 Build"
else
    log_fail "  B1 Build"
fi

# 1.2 测试
log_info "  B2: cargo test --lib"
if cargo test --lib >/dev/null 2>&1; then
    log_pass "  B2 Test"
else
    log_fail "  B2 Test"
fi

# 1.3 Clippy
log_info "  B3: cargo clippy"
if cargo clippy --all-features -- -D warnings >/dev/null 2>&1; then
    log_pass "  B3 Clippy"
else
    log_fail "  B3 Clippy"
fi

# 1.4 格式
log_info "  B4: cargo fmt --check"
if cargo fmt --check --all >/dev/null 2>&1; then
    log_pass "  B4 Format"
else
    log_fail "  B4 Format"
fi

# 1.5 覆盖率检查 (>=85%)
log_info "  B5: coverage check (>=85%)"
COV=$(cargo llvm-cov --all-features --json 2>/dev/null | grep -o '"total_line"\:[0-9.]*' | grep -o '[0-9.]*$' | head -1 || echo "0")
if (( $(echo "$COV >= 85" | bc -l 2>/dev/null || echo 0) )); then
    log_pass "  B5 Coverage ($COV%)"
else
    log_fail "  B5 Coverage ($COV% < 85%)"
fi

# ============================================================
# Phase 2: SQL 兼容性测试 (约 1h)
# ============================================================
log_info "【Phase 2】SQL 兼容性测试 (~1h)"

# 2.1 SQL Corpus
log_info "  R7: SQL Operations >=95%"
if bash scripts/gate/check_sql_compat.sh >/dev/null 2>&1; then
    log_pass "  R7 SQL Operations"
else
    log_fail "  R7 SQL Operations"
fi

# 2.2 TPC-H SF=0.1 (快速验证)
log_info "  TPC-H SF=0.1 (快速验证)"
if bash scripts/gate/check_tpch.sh sf=0.1 >/dev/null 2>&1; then
    log_pass "  TPC-H SF=0.1"
else
    log_fail "  TPC-H SF=0.1"
fi

# ============================================================
# Phase 3: 性能测试 (约 2h)
# ============================================================
log_info "【Phase 3】性能测试 (~2h)"

# 3.1 TPC-H SF=1
log_info "  TPC-H SF=1 (22 queries, 约 1h)"
if [ -x "$SCRIPT_DIR/tpch_sf1.sh" ]; then
    if bash "$SCRIPT_DIR/tpch_sf1.sh" >/dev/null 2>&1; then
        log_pass "  TPC-H SF=1"
    else
        log_fail "  TPC-H SF=1"
    fi
else
    log_info "  TPC-H SF=1 (使用 check_tpch.sh)"
    if bash scripts/gate/check_tpch.sh sf=1 >/dev/null 2>&1; then
        log_pass "  TPC-H SF=1"
    else
        log_fail "  TPC-H SF=1"
    fi
fi

# 3.2 Sysbench
log_info "  Sysbench QPS (约 30min)"
if bash scripts/gate/check_sysbench.sh >/dev/null 2>&1; then
    log_pass "  Sysbench QPS"
else
    log_fail "  Sysbench QPS"
fi

# ============================================================
# Phase 4: 稳定性测试 (约 2h30min)
# ============================================================
log_info "【Phase 4】稳定性测试 (~2h30min)"

# 4.1 崩溃恢复 100 次
log_info "  崩溃恢复测试 (100 iterations, 约 1h)"
if [ -x "$SCRIPT_DIR/crash_recovery_100.sh" ]; then
    if bash "$SCRIPT_DIR/crash_recovery_100.sh" >/dev/null 2>&1; then
        log_pass "  崩溃恢复 100 次"
    else
        log_fail "  崩溃恢复 100 次"
    fi
else
    log_info "  崩溃恢复测试 (使用默认配置)"
    if bash scripts/gate/crash_recovery_loop.sh 50 >/dev/null 2>&1; then
        log_pass "  崩溃恢复 50 次"
    else
        log_fail "  崩溃恢复 50 次"
    fi
fi

# 4.2 并发压力测试
log_info "  并发压力测试 (约 1h)"
if bash scripts/gate/check_concurrency_stress.sh >/dev/null 2>&1; then
    log_pass "  并发压力测试"
else
    log_fail "  并发压力测试"
fi

# 4.3 快速稳定性 (2h 运行)
log_info "  快速稳定性测试 (2h)"
if [ -x "$SCRIPT_DIR/stability_short.sh" ]; then
    if HOURS=2 bash "$SCRIPT_DIR/stability_short.sh" >/dev/null 2>&1; then
        log_pass "  快速稳定性测试"
    else
        log_fail "  快速稳定性测试"
    fi
else
    log_skip "  快速稳定性测试 (未实现)"
fi

# ============================================================
# Phase 5: 安全与回归 (约 30min)
# ============================================================
log_info "【Phase 5】安全与回归检查 (~30min)"

# 5.1 安全审计
log_info "  R6: cargo audit"
if cargo audit >/dev/null 2>&1; then
    log_pass "  R6 Security"
else
    log_fail "  R6 Security"
fi

# 5.2 回归检查
log_info "  R9: regression check"
if bash scripts/gate/check_regression.sh >/dev/null 2>&1; then
    log_pass "  R9 Regression"
else
    log_fail "  R9 Regression"
fi

# 5.3 形式化证明
log_info "  R10: formal proofs (>=30)"
if bash scripts/gate/check_proof.sh >/dev/null 2>&1; then
    log_pass "  R10 Proofs"
else
    log_fail "  R10 Proofs"
fi

echo ""
log_info "RC Gate 所有阶段完成，等待汇总..."
