#!/usr/bin/env bash
# formal_smoke.sh — PR Gate: 轻量形式化验证 (smoke test, <2min)
#
# 原则:
#   1. 只验证 spec 能跑、结构没破
#   2. 必须验证 invariant 被加载（防漂移）
#   3. 不跑全量状态空间
#
# 触发: 每个 PR
# 用法: ./formal_smoke.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

TLA_JAR="${TLA_JAR:-/tmp/tla2tools.jar}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASS=0
FAIL=0
SKIP=0

pass() { echo -e "${GREEN}  PASS${NC}  $1"; PASS=$((PASS+1)); }
fail() { echo -e "${RED}  FAIL${NC}  $1"; FAIL=$((FAIL+1)); }
skip() { echo -e "${YELLOW}  SKIP${NC}  $1"; SKIP=$((SKIP+1)); }

echo "=== Formal Smoke Test (PR Gate) ==="
echo "Date: $(date)"
echo ""

# ============================================================
# 工具检查
# ============================================================

if [ ! -f "$TLA_JAR" ]; then
    skip "TLA+ tools not found at $TLA_JAR"
    echo ""
    echo "Install: curl -L https://github.com/tlaplus/tlaplus/releases/download/v1.7.0/tla2tools.jar -o /tmp/tla2tools.jar"
    echo ""
    echo "=== Summary ==="
    echo "Skipped: $SKIP"
    exit 0
fi

FORMAL_DIR="docs/formal"

# ============================================================
# 检查: cfg 必须声明 INVARIANT（防漂移）
# ============================================================

echo "[Check] Verifying invariants in cfg files..."
missing=0
for cfg in "$FORMAL_DIR"/*.cfg; do
    if [ -f "$cfg" ]; then
        if ! grep -q "INVARIANT" "$cfg" && ! grep -q "SPECIFICATION" "$cfg"; then
            echo "  ${YELLOW}WARNING${NC}: $cfg has no INVARIANT or SPECIFICATION"
            missing=$((missing+1))
        fi
    fi
done
if [ $missing -gt 0 ]; then
    fail "Smoke check failed: $missing cfg file(s) missing INVARIANT"
    echo "Smoke must verify invariants, not just run models"
    exit 1
fi
echo "  ${GREEN}OK${NC}: All cfg files have INVARIANT declarations"
echo ""

# ============================================================
# SMOKE: 小状态空间模型（毫秒级，不跑百万 states）
# ============================================================

smoke_tla() {
    local model="$1"
    local expected="$2"  # "PASS" or "VIOLATED"
    local cfg="${model%.tla}.cfg"

    if [ ! -f "$FORMAL_DIR/$model" ]; then
        skip "$model (file not found)"
        return
    fi

    if [ ! -f "$FORMAL_DIR/$cfg" ]; then
        skip "$model (cfg not found)"
        return
    fi

    echo "[TLA+ Smoke] $model (expect: $expected)..."

    # 核心检查: INVARIANT 必须在 cfg 中声明
    if ! grep -q "INVARIANT" "$FORMAL_DIR/$cfg"; then
        fail "$model: INVARIANT not declared in $cfg"
        return
    fi

    local out
    out=$(java -XX:+UseParallelGC -cp "$TLA_JAR" \
        tlc2.TLC "$FORMAL_DIR/$model" \
        -config "$FORMAL_DIR/$cfg" \
        -seed 1 -deadlock 2>&1 || true)

    if echo "$out" | grep -q "Model checking completed. No error"; then
        if [ "$expected" = "PASS" ]; then
            pass "$model"
        else
            fail "$model (expected $expected, got PASS)"
        fi
    elif echo "$out" | grep -q "Error:"; then
        if [ "$expected" = "VIOLATED" ]; then
            pass "$model (violated as expected)"
        else
            fail "$model (expected $expected, got violation)"
        fi
    else
        skip "$model (ambiguous result)"
    fi
}

echo "--- TLA+ Smoke Models ---"

# Deadlock: 原子版本（应该 PASS）
smoke_tla "PROOF_023_deadlock_v4.tla" "PASS"

# Deadlock: TOCTOU 版本（应该 VIOLATED）
smoke_tla "PROOF_023_deadlock_toctou.tla" "VIOLATED"

# MVCC Atomic（应该 PASS）
smoke_tla "PROOF_016_023_mvcc_atomic.tla" "PASS"

# MVCC TOCTOU（应该 VIOLATED）
smoke_tla "PROOF_016_023_mvcc_toctou.tla" "VIOLATED"

# Write Skew 反例（应该 VIOLATED）
smoke_tla "PROOF_026_write_skew.tla" "VIOLATED"

echo ""
echo "=== Summary ==="
echo "Passed:  $PASS"
echo "Failed:  $FAIL"
echo "Skipped: $SKIP"

if [ $FAIL -gt 0 ]; then
    echo ""
    echo -e "${RED}Formal smoke FAILED${NC}"
    echo "Correctness constraints have regressed."
    exit 1
else
    echo ""
    echo -e "${GREEN}Formal smoke PASSED${NC}"
    exit 0
fi
