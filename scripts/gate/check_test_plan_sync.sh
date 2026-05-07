#!/usr/bin/env bash
# check_test_plan_sync.sh — 检查 TEST_PLAN 与 gate_spec 一致性
# 确保：TEST_PLAN 中的检查项与 gate_spec 定义一致，阈值一致
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "=== TEST_PLAN 与门禁同步检查 ==="
echo "时间: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

PASS=0
WARN=0
FAIL=0

TEST_PLAN="docs/releases/v3.0.0/TEST_PLAN.md"
GATE_SPEC="docs/governance/gate_spec_v300.md"

# ========== 检查 1: TEST_PLAN 是否存在 ==========
echo "【检查 1】TEST_PLAN 存在性"
echo "-----------------------------------"

if [ ! -f "$TEST_PLAN" ]; then
    echo "  ❌ $TEST_PLAN 不存在"
    FAIL=$((FAIL+1))
else
    echo "  ✅ $TEST_PLAN 存在"
    PASS=$((PASS+1))
fi
echo ""

# ========== 检查 2: gate_spec 是否存在 ==========
echo "【检查 2】gate_spec 存在性"
echo "-----------------------------------"

if [ ! -f "$GATE_SPEC" ]; then
    echo "  ❌ $GATE_SPEC 不存在"
    FAIL=$((FAIL+1))
else
    echo "  ✅ $GATE_SPEC 存在"
    PASS=$((PASS+1))
fi
echo ""

# ========== 检查 3: Alpha 门禁项同步 ==========
echo "【检查 3】Alpha 门禁 (A1-A7) 同步"
echo "-----------------------------------"

for item in A1 A2 A3 A4 A5 A6 A7; do
    IN_TEST_PLAN=$(grep -c "$item" "$TEST_PLAN" 2>/dev/null || echo "0")
    IN_GATE_SPEC=$(grep -c "$item" "$GATE_SPEC" 2>/dev/null || echo "0")

    if [ "$IN_TEST_PLAN" -gt 0 ] && [ "$IN_GATE_SPEC" -gt 0 ]; then
        echo "  ✅ $item: TEST_PLAN ✅ gate_spec ✅"
        PASS=$((PASS+1))
    elif [ "$IN_TEST_PLAN" -gt 0 ] && [ "$IN_GATE_SPEC" -eq 0 ]; then
        echo "  ❌ $item: TEST_PLAN 有，gate_spec 无"
        FAIL=$((FAIL+1))
    elif [ "$IN_TEST_PLAN" -eq 0 ] && [ "$IN_GATE_SPEC" -gt 0 ]; then
        echo "  ⚠️  $item: gate_spec 有，TEST_PLAN 无"
        WARN=$((WARN+1))
    fi
done
echo ""

# ========== 检查 4: Beta 门禁项同步 ==========
echo "【检查 4】Beta 门禁 (B1-B9) 同步"
echo "-----------------------------------"

for item in B1 B2 B3 B4 B5 B6 B7 B8 B9; do
    IN_TEST_PLAN=$(grep -c "$item" "$TEST_PLAN" 2>/dev/null || echo "0")
    IN_GATE_SPEC=$(grep -c "$item" "$GATE_SPEC" 2>/dev/null || echo "0")

    if [ "$IN_TEST_PLAN" -gt 0 ] && [ "$IN_GATE_SPEC" -gt 0 ]; then
        echo "  ✅ $item: TEST_PLAN ✅ gate_spec ✅"
        PASS=$((PASS+1))
    elif [ "$IN_TEST_PLAN" -gt 0 ] && [ "$IN_GATE_SPEC" -eq 0 ]; then
        echo "  ❌ $item: TEST_PLAN 有，gate_spec 无"
        FAIL=$((FAIL+1))
    elif [ "$IN_TEST_PLAN" -eq 0 ] && [ "$IN_GATE_SPEC" -gt 0 ]; then
        echo "  ⚠️  $item: gate_spec 有，TEST_PLAN 无"
        WARN=$((WARN+1))
    fi
done
echo ""

# ========== 检查 5: GA 门禁项同步 ==========
echo "【检查 5】GA 门禁 (G1-G13) 同步"
echo "-----------------------------------"

for item in G1 G2 G3 G4 G5 G6 G7 G8 G9 G10 G11 G12 G13; do
    IN_TEST_PLAN=$(grep -c "$item" "$TEST_PLAN" 2>/dev/null || echo "0")
    IN_GATE_SPEC=$(grep -c "$item" "$GATE_SPEC" 2>/dev/null || echo "0")

    if [ "$IN_TEST_PLAN" -gt 0 ] && [ "$IN_GATE_SPEC" -gt 0 ]; then
        echo "  ✅ $item: TEST_PLAN ✅ gate_spec ✅"
        PASS=$((PASS+1))
    elif [ "$IN_TEST_PLAN" -gt 0 ] && [ "$IN_GATE_SPEC" -eq 0 ]; then
        echo "  ❌ $item: TEST_PLAN 有，gate_spec 无"
        FAIL=$((FAIL+1))
    elif [ "$IN_TEST_PLAN" -eq 0 ] && [ "$IN_GATE_SPEC" -gt 0 ]; then
        echo "  ⚠️  $item: gate_spec 有，TEST_PLAN 无"
        WARN=$((WARN+1))
    fi
done
echo ""

# ========== 检查 6: 覆盖率阈值一致性 ==========
echo "【检查 6】覆盖率阈值一致性"
echo "-----------------------------------"

# Beta 覆盖率 75%
if grep -q "覆盖率.*75%" "$TEST_PLAN" 2>/dev/null; then
    echo "  ✅ TEST_PLAN: Beta 覆盖率 75%"
    PASS=$((PASS+1))
else
    echo "  ⚠️  TEST_PLAN: Beta 覆盖率阈值未找到"
    WARN=$((WARN+1))
fi

if grep -q "≥75%" "$GATE_SPEC" 2>/dev/null; then
    echo "  ✅ gate_spec: Beta 覆盖率 ≥75%"
    PASS=$((PASS+1))
else
    echo "  ⚠️  gate_spec: Beta 覆盖率阈值未找到"
    WARN=$((WARN+1))
fi

# GA 覆盖率 85%
if grep -q "覆盖率.*85%" "$TEST_PLAN" 2>/dev/null; then
    echo "  ✅ TEST_PLAN: GA 覆盖率 85%"
    PASS=$((PASS+1))
else
    echo "  ⚠️  TEST_PLAN: GA 覆盖率阈值未找到"
    WARN=$((WARN+1))
fi

if grep -q "≥85%" "$GATE_SPEC" 2>/dev/null; then
    echo "  ✅ gate_spec: GA 覆盖率 ≥85%"
    PASS=$((PASS+1))
else
    echo "  ⚠️  gate_spec: GA 覆盖率阈值未找到"
    WARN=$((WARN+1))
fi
echo ""

# ========== 检查 7: QPS 阈值一致性 ==========
echo "【检查 7】QPS 阈值一致性"
echo "-----------------------------------"

# Point SELECT ≥10K
if grep -q "Point.*10,000" "$TEST_PLAN" 2>/dev/null || grep -q "Point.*10000" "$TEST_PLAN" 2>/dev/null; then
    echo "  ✅ TEST_PLAN: Point SELECT 10K QPS"
    PASS=$((PASS+1))
else
    echo "  ⚠️  TEST_PLAN: Point SELECT 阈值未找到"
    WARN=$((WARN+1))
fi

if grep -q "≥10,000" "$GATE_SPEC" 2>/dev/null || grep -q "10000" "$GATE_SPEC" 2>/dev/null; then
    echo "  ✅ gate_spec: Point SELECT ≥10K QPS"
    PASS=$((PASS+1))
else
    echo "  ⚠️  gate_spec: Point SELECT 阈值未找到"
    WARN=$((WARN+1))
fi
echo ""

# ========== 总结 ==========
echo "=== 同步检查结果 ==="
echo "  ✅ PASS: $PASS"
echo "  ⚠️  WARN: $WARN"
echo "  ❌ FAIL: $FAIL"

if [ $FAIL -gt 0 ]; then
    echo ""
    echo "❌ 同步检查失败: $FAIL 项不一致"
    echo "建议: 更新 TEST_PLAN 或 gate_spec 使其一致"
    exit 1
elif [ $WARN -gt 0 ]; then
    echo ""
    echo "⚠️  同步检查有警告: $WARN 项可能需要同步"
    exit 0
else
    echo ""
    echo "✅ 同步检查通过"
    exit 0
fi
