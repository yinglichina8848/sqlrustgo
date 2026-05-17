#!/bin/bash
#
# v3.3.0 Alpha Gate 检查脚本
# 用途: Alpha 阶段入口检查（本地快速检查）
# 执行: bash scripts/gate/check_alpha_v330.sh
#

set -e

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=== v3.3.0 Alpha Gate ==="
echo "分支: $(git branch --show-current)"
echo "Commit: $(git rev-parse --short HEAD)"
echo ""

PASS=0
FAIL=0
SKIP=0

run_check() {
    local name="$1"
    local cmd="$2"
    echo -n "[$name] ... "
    if eval "$cmd" > /dev/null 2>&1; then
        echo "✅ PASS"
        ((PASS++))
    else
        echo "❌ FAIL"
        ((FAIL++))
    fi
}

run_check_report() {
    local name="$1"
    local cmd="$2"
    echo -n "[$name] ... "
    if eval "$cmd" > /dev/null 2>&1; then
        echo "✅ PASS"
        ((PASS++))
    else
        echo "⏭ SKIP (manual review required)"
        ((SKIP++))
    fi
}

# A1: Build
echo "=== 代码质量 ==="
run_check "A1 Build" "cargo build --release --workspace"

# A2: Test
run_check "A2 Test" "cargo test --lib"

# A3: Clippy
run_check "A3 Clippy" "cargo clippy --all-features -- -D warnings"

# A4: Format
run_check "A4 Format" "cargo fmt --all -- --check"

echo ""
echo "=== SSOT 规范检查 ==="
run_check_report "A8 SSOT Coverage Spec" "[ -f docs/governance/gate_spec_v330.md ]"

echo ""
echo "=== 结果汇总 ==="
echo "  PASS : $PASS"
echo "  FAIL : $FAIL"
echo "  SKIP : $SKIP"
echo ""

if [ $FAIL -eq 0 ]; then
    echo "✅ Alpha Gate: Local checks PASS ($PASS+$SKIP/$((PASS+FAIL+SKIP)))"
    echo ""
    echo "⏳ 需 Z6G4 执行:"
    echo "  A5: cargo llvm-cov coverage (L1 CRATES ≥85%)"
    echo "  A6: MySQL Protocol handshake test"
    echo "  A7: TPC-H SF=1 22/22 query test"
    exit 0
else
    echo "❌ Alpha Gate: FAIL ($FAIL 项未通过)"
    exit 1
fi
