#!/usr/bin/env bash
# check_l0_smoke.sh — L0 冒烟测试，<5min 完成
# 用于快速判断分支是否可用
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "=== L0 Smoke Test ==="
echo "时间: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

PASS=0
TOTAL=0

check() {
    local name="$1"; shift
    local cmd="$*"
    TOTAL=$((TOTAL+1))
    echo -n "[L0] $name ... "
    if eval "$cmd" >/dev/null 2>&1; then
        echo "✅ PASS"
        PASS=$((PASS+1))
    else
        echo "❌ FAIL"
    fi
}

echo "【构建】"
check "cargo build --release --workspace" cargo build --release --workspace

echo ""
echo "【格式】"
check "cargo fmt --check" cargo fmt --all -- --check

echo ""
echo "【Clippy】"
check "cargo clippy --all-features -- -D warnings" cargo clippy --all-features -- -D warnings

echo ""
echo "【Types 冒烟】"
check "cargo test -p sqlrustgo-types --lib" cargo test -p sqlrustgo-types --lib -- --test-threads=4

echo ""
echo "【Parser 冒烟】"
check "cargo test -p sqlrustgo-parser --lib" cargo test -p sqlrustgo-parser --lib -- --test-threads=4

echo ""
echo "=== L0 结果: $PASS / $TOTAL ==="
if [ $PASS -eq $TOTAL ]; then
    echo "✅ 全部通过，分支可用"
    exit 0
else
    echo "⚠️  $((TOTAL-PASS)) 项失败，请检查"
    exit 1
fi
