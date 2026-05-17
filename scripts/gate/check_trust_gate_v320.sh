#!/usr/bin/env bash
# check_trust_gate_v320.sh — v3.2.0 Trust Gate 可信性门禁系统
#
# Issue #1163: Trust Gate 可信性门禁系统
# 规范来源: Issue #1163 (SSOT)
# 版本: 1.0
#
# 6 个必须通过的门禁:
#   1. Recovery Gate: crash replay    — 崩溃恢复验证
#   2. Audit Gate: chain verify       — 审计链验证
#   3. Signature Gate: tamper detect — 签名篡改检测
#   4. MVCC Gate: anomaly detect      — MVCC异常检测
#   5. Performance Gate: regression   — 性能回归检测
#   6. Coverage Gate: critical paths — 关键路径覆盖率
#
# 验收标准: 6个门禁全部可执行 (PASS)
#
set -uo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

PASS=0; TOTAL=0; BLOCKERS=0
FAIL_REASONS=()
SKIP_REASONS=()

log_info() { echo "[trust-gate-v3.2.0] $1"; }
log_pass() { echo "✅ PASS: $1"; PASS=$((PASS+1)); TOTAL=$((TOTAL+1)); }
log_fail() { echo "❌ FAIL: $1"; BLOCKERS=$((BLOCKERS+1)); TOTAL=$((TOTAL+1)); FAIL_REASONS+=("$1"); }
log_skip() { echo "⏭️  SKIP: $1"; TOTAL=$((TOTAL+1)); SKIP_REASONS+=("$1"); }
log_warn() { echo "⚠️  WARN: $1"; }

check() {
    local name="$1" cmd="$2"
    local label="${3:-TG}"
    TOTAL=$((TOTAL+1))
    echo -n "[trust-gate-v3.2.0] $name ... "
    if eval "$cmd" >/dev/null 2>&1; then
        echo "PASS"; PASS=$((PASS+1))
    else
        echo "FAIL"; BLOCKERS=$((BLOCKERS+1)); FAIL_REASONS+=("【$label】$name")
    fi
}

check_test() {
    local name="$1" cmd="$2"
    local label="$3"
    TOTAL=$((TOTAL+1))
    echo -n "[trust-gate-v3.2.0] $name ... "
    local output
    output=$(eval "$cmd" 2>&1) || true
    local passed failed
    passed=$(echo "$output" | grep -c "test result: ok\." || true)
    failed=$(echo "$output" | grep -c "test result: FAILED" || true)
    passed=${passed:-0}
    failed=${failed:-0}
    passed=$(echo "$passed" | head -1)
    failed=$(echo "$failed" | head -1)
    if [ "$failed" -eq 0 ] && [ "$passed" -gt 0 ]; then
        echo "PASS ($passed tests)"; PASS=$((PASS+1))
    else
        echo "FAIL ($passed passed, $failed failed)"; BLOCKERS=$((BLOCKERS+1))
        FAIL_REASONS+=("【$label】$name")
    fi
}

echo "=========================================="
echo "  v3.2.0 Trust Gate — 可信性门禁系统"
echo "  Issue #1163: 真正 GMP Native"
echo "=========================================="
echo "日期: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# ============================================================
# 6 个 Trust Gate
# ============================================================
echo "━━━ 6 个 Trust Gate ━━━"

# TG1: Recovery Gate — crash replay
log_info "TG1: Recovery Gate (crash replay)"
check_test "TG1: crash_recovery_test" "cargo test --test crash_recovery_test 2>&1" "TG1"

# TG2: Audit Gate — chain verify
log_info "TG2: Audit Gate (chain verify)"
check_test "TG2: gmp_audit_chain_verify_test" "cargo test -p sqlrustgo-gmp --test gmp_audit_chain_verify_test 2>&1" "TG2"

# TG3: Signature Gate — tamper detect
log_info "TG3: Signature Gate (tamper detect)"
check_test "TG3: gmp_digital_signature_test" "cargo test -p sqlrustgo-gmp --test gmp_digital_signature_test 2>&1" "TG3"

# TG4: MVCC Gate — anomaly detect
log_info "TG4: MVCC Gate (anomaly detect)"
check_test "TG4: ssi_stress_test" "cargo test -p sqlrustgo-transaction --test ssi_stress_test 2>&1" "TG4"

# TG5: Performance Gate — regression
log_info "TG5: Performance Gate (regression)"
TOTAL=$((TOTAL+1))
echo -n "[trust-gate-v3.2.0] TG5: check_regression.sh ... "
if timeout 300 bash "$SCRIPT_DIR/check_regression.sh" >/dev/null 2>&1; then
    echo "PASS"; PASS=$((PASS+1))
else
    echo "FAIL"; BLOCKERS=$((BLOCKERS+1)); FAIL_REASONS+=("【TG5】Performance Gate: regression")
fi

# TG6: Coverage Gate — critical paths
log_info "TG6: Coverage Gate (critical paths)"
TOTAL=$((TOTAL+1))
echo -n "[trust-gate-v3.2.0] TG6: check_coverage.sh ... "
if bash "$SCRIPT_DIR/check_coverage.sh" >/dev/null 2>&1; then
    echo "PASS"; PASS=$((PASS+1))
else
    echo "FAIL"; BLOCKERS=$((BLOCKERS+1)); FAIL_REASONS+=("【TG6】Coverage Gate: critical paths")
fi

# ============================================================
# 结果汇总
# ============================================================
echo ""
echo "=========================================="
echo "  Trust Gate 结果汇总"
echo "=========================================="
echo "通过: $PASS / $TOTAL"
echo "阻塞: $BLOCKERS"

if [ ${#SKIP_REASONS[@]} -gt 0 ]; then
    echo ""
    echo "跳过检查:"
    for reason in "${SKIP_REASONS[@]}"; do
        echo "  - $reason"
    done
fi

if [ $BLOCKERS -gt 0 ]; then
    echo ""
    echo "失败的检查:"
    for reason in "${FAIL_REASONS[@]}"; do
        echo "  ❌ $reason"
    done
    echo ""
    echo "❌ Trust Gate FAILED — $BLOCKERS 个门禁未通过"
    exit 1
else
    echo ""
    echo "✅ Trust Gate PASSED — 6个门禁全部通过"
    echo "   这才是真正 GMP Native (Issue #1163)"
    exit 0
fi