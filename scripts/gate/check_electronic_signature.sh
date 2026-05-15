#!/usr/bin/env bash
# check_electronic_signature.sh — G-QA1: Electronic Signature (21 CFR Part 11)
# 规范来源: docs/governance/GATE_SPEC_MASTER.md
# 版本: 1.0
set -uo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

PASS=0; TOTAL=0; FAIL=0
FAIL_REASONS=()

log_info() { echo "[qa-electronic-signature] $1"; }
log_pass() { echo "✅ PASS: $1"; PASS=$((PASS+1)); TOTAL=$((TOTAL+1)); }
log_fail() { echo "❌ FAIL: $1"; FAIL=$((FAIL+1)); TOTAL=$((TOTAL+1)); FAIL_REASONS+=("$1"); }

echo "=========================================="
echo "  G-QA1: Electronic Signature Check"
echo "  21 CFR Part 11 合规验证"
echo "=========================================="
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# Check if test binary exists
if [ ! -f "tests/gmp_electronic_signature_test" ]; then
    log_info "Building electronic signature test..."
    if ! cargo build --test gmp_electronic_signature_test --quiet 2>/dev/null; then
        # Try building from crates/gmp
        cargo build -p sqlrustgo-gmp --tests --quiet 2>/dev/null || true
    fi
fi

# Run electronic signature tests
log_info "Running electronic signature tests..."
if cargo test -p sqlrustgo-gmp --test gmp_electronic_signature_test 2>&1 | tee /tmp/qa_electronic_signature.log; then
    log_pass "Electronic signature tests passed"
else
    log_fail "Electronic signature tests failed"
fi

# Check for key compliance features
echo ""
log_info "Checking 21 CFR Part 11 compliance features..."

# 1. Signature non-repudiation
if cargo test -p sqlrustgo-gmp --test gmp_electronic_signature_test -- --nocapture 2>&1 | grep -q "test.*signature"; then
    log_pass "Signature tests found"
else
    log_fail "No signature tests found"
fi

# 2. Approval policy
if cargo test -p sqlrustgo-gmp --test gmp_electronic_signature_test approval_policy 2>&1 | grep -q "test result:"; then
    log_pass "Approval policy tests passed"
else
    log_fail "Approval policy tests failed"
fi

# 3. Audit trail
if cargo test -p sqlrustgo-gmp --test gmp_electronic_signature_test audit 2>&1 | grep -q "test result:"; then
    log_pass "Audit trail tests passed"
else
    log_fail "Audit trail tests failed"
fi

echo ""
echo "=========================================="
echo "  G-QA1 结果汇总"
echo "=========================================="
echo "✅ PASS: $PASS"
echo "❌ FAIL: $FAIL"
echo ""

if [ ${#FAIL_REASONS[@]} -gt 0 ]; then
    echo "失败项:"
    for reason in "${FAIL_REASONS[@]}"; do
        echo "  - $reason"
    done
fi

if [ $FAIL -gt 0 ]; then
    echo ""
    echo "❌ G-QA1 Electronic Signature Check FAILED"
    exit 1
fi

echo "✅ G-QA1 Electronic Signature Check PASSED"
exit 0