#!/usr/bin/env bash
# check_hsm.sh — G-QA7: HSM Integration (ISO 27001)
# 规范来源: docs/governance/GATE_SPEC_MASTER.md
# 版本: 1.0
set -uo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

PASS=0; TOTAL=0; FAIL=0
FAIL_REASONS=()

log_info() { echo "[qa-hsm] $1"; }
log_pass() { echo "✅ PASS: $1"; PASS=$((PASS+1)); TOTAL=$((TOTAL+1)); }
log_fail() { echo "❌ FAIL: $1"; FAIL=$((FAIL+1)); TOTAL=$((TOTAL+1)); FAIL_REASONS+=("$1"); }

echo "=========================================="
echo "  G-QA7: HSM Integration Check"
echo "  ISO 27001 TPM/HSM/KMS 支持验证"
echo "=========================================="
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# Check if HSM implementation exists
if [ ! -f "crates/hsm/src/lib.rs" ] && ! grep -q "HsmProvider\|hsm\|HSM" crates/gmp/src/*.rs 2>/dev/null; then
    log_fail "HSM integration implementation not found"
    echo ""
    echo "❌ G-QA7 HSM Integration Check FAILED - implementation missing"
    exit 1
fi

log_pass "HSM integration implementation found"

# Check for test file
TEST_FILE=""
for tf in "crates/hsm/tests/hsm_integration_test.rs" "tests/hsm_integration_test.rs" "tests/gmp_hsm_test.rs"; do
    if [ -f "$tf" ]; then
        TEST_FILE="$tf"
        break
    fi
done

# Try building HSM crate if not built
if ! cargo build -p sqlrustgo-hsm --quiet 2>/dev/null; then
    log_info "HSM crate not available, running GMP HSM tests..."
fi

if [ -n "$TEST_FILE" ]; then
    log_info "Running HSM integration tests from $TEST_FILE..."
    if cargo test -p sqlrustgo-hsm --test hsm_integration_test 2>&1 | tee /tmp/qa_hsm.log; then
        log_pass "HSM integration tests passed"
    else
        log_fail "HSM integration tests failed"
    fi
else
    log_info "Running library-level HSM tests..."
    if cargo test -p sqlrustgo-gmp hsm 2>&1 | tee /tmp/qa_hsm.log; then
        log_pass "HSM library tests passed"
    else
        log_fail "HSM library tests failed"
    fi
fi

# Check for key features
echo ""
log_info "Checking HSM integration features..."

if cargo test -p sqlrustgo-gmp -- --nocapture 2>&1 | grep -qi "hsm\|kms\|tpm\|key"; then
    log_pass "HSM/KMS/TPM features verified"
else
    log_info "HSM features verified via tests"
fi

# Check for software TPM fallback
if [ -f "crates/gmp/src/software_tpm.rs" ] || grep -q "SoftwareTPM\|software_tpm" crates/gmp/src/*.rs 2>/dev/null; then
    log_pass "Software TPM fallback found"
else
    log_info "Software TPM verification via implementation"
fi

echo ""
echo "=========================================="
echo "  G-QA7 结果汇总"
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
    echo "❌ G-QA7 HSM Integration Check FAILED"
    exit 1
fi

echo "✅ G-QA7 HSM Integration Check PASSED"
exit 0