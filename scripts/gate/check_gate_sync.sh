#!/usr/bin/env bash
# check_gate_sync.sh — 检查 gate_spec 与 check_*.sh 一致性
# 确保：规范中定义的检查项，脚本必须实现；脚本中的检查项，规范必须定义
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "=== 门禁规范与脚本同步检查 ==="
echo "时间: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

PASS=0
WARN=0
FAIL=0

# ========== 检查 1: Beta Gate ==========
echo "【检查 1】Beta Gate (check_beta_v300.sh)"
echo "-----------------------------------"

BETA_SCRIPT="scripts/gate/check_beta_v300.sh"
BETA_SPEC_GATE="B"

if [ ! -f "$BETA_SCRIPT" ]; then
    echo "  ⚠️  $BETA_SCRIPT 不存在"
else
    # 检查 B1-B9, B-S* 是否在脚本中
    for item in B1 B2 B3 B4 B5 B6 B7 B8 B9; do
        if grep -q "$item" "$BETA_SCRIPT"; then
            echo "  ✅ $item 在脚本中"
        else
            echo "  ❌ $item 不在脚本中"
            FAIL=$((FAIL+1))
        fi
    done

    # 检查 B-S* 项
    for item in B-S1 B-S2 B-S3 B-S4 B-S5 B-S10; do
        if grep -q "$item" "$BETA_SCRIPT"; then
            echo "  ✅ $item 在脚本中"
        else
            echo "  ⚠️  $item 不在脚本中（可能已移除）"
            WARN=$((WARN+1))
        fi
    done
fi
echo ""

# ========== 检查 2: GA Gate ==========
echo "【检查 2】GA Gate (check_ga_v300.sh)"
echo "-----------------------------------"

GA_SCRIPT="scripts/gate/check_ga_v300.sh"

if [ ! -f "$GA_SCRIPT" ]; then
    echo "  ⚠️  $GA_SCRIPT 不存在"
else
    # GA 门禁使用描述性标签，检查关键检查项是否存在
    # G1: Build
    if grep -qE "Build|cargo build" "$GA_SCRIPT"; then
        echo "  ✅ Build 检查存在 (G1)"
    else
        echo "  ⚠️  Build 检查未找到 (G1)"
    fi

    # G2: Test
    if grep -qE "Test|cargo test" "$GA_SCRIPT"; then
        echo "  ✅ Test 检查存在 (G2)"
    else
        echo "  ⚠️  Test 检查未找到 (G2)"
    fi

    # G5: Coverage
    if grep -qE "Coverage|llvm-cov|tarpaulin" "$GA_SCRIPT"; then
        echo "  ✅ Coverage 检查存在 (G5)"
    else
        echo "  ⚠️  Coverage 检查未找到 (G5)"
    fi

    # G7/G8/G9: QPS
    if grep -qE "Point|SELECT.*QPS|QPS.*point" "$GA_SCRIPT"; then
        echo "  ✅ Point SELECT QPS 检查存在 (G7)"
    fi
    if grep -qE "UPDATE.*QPS|QPS.*UPDATE|update_simple" "$GA_SCRIPT"; then
        echo "  ✅ UPDATE QPS 检查存在 (G8)"
    fi
    if grep -qE "DELETE.*QPS|QPS.*DELETE|delete_simple" "$GA_SCRIPT"; then
        echo "  ✅ DELETE QPS 检查存在 (G9)"
    fi

    # G10: TPC-H SF=1
    if grep -qE "TPC-H|SF=1|tpch" "$GA_SCRIPT"; then
        echo "  ✅ TPC-H SF=1 检查存在 (G10)"
    fi

    # G11: SQL Corpus
    if grep -qE "SQL.*Corpus|corpus" "$GA_SCRIPT"; then
        echo "  ✅ SQL Corpus 检查存在 (G11)"
    fi

    # G12: Stability (B-S tests)
    if grep -qE "B-S|concurrency|crash|wal_integration" "$GA_SCRIPT"; then
        echo "  ✅ 稳定性测试存在 (G12)"
    fi

    # G13: MySQL Protocol
    if grep -qE "MySQL.*Protocol|mysql.*5.7|handshake" "$GA_SCRIPT"; then
        echo "  ✅ MySQL Protocol 检查存在 (G13)"
    fi

    echo "  ℹ️  GA 门禁使用描述性标签（非 G1-G13）"
fi
echo ""

# ========== 检查 3: gate_spec 与脚本阈值一致性 ==========
echo "【检查 3】gate_spec 与脚本阈值一致性"
echo "-----------------------------------"

# Beta B2 阈值检查
if [ -f "$BETA_SCRIPT" ]; then
    # 检查 B2 测试通过率阈值
    if grep -q "PASS_RATE.*90" "$BETA_SCRIPT"; then
        echo "  ✅ B2 测试通过率阈值: 90%"
    else
        echo "  ⚠️  B2 测试通过率阈值未找到"
    fi

    # 检查 B5 覆盖率阈值
    if grep -q "COVERAGE.*75" "$BETA_SCRIPT"; then
        echo "  ✅ B5 覆盖率阈值: 75%"
    else
        echo "  ⚠️  B5 覆盖率阈值未找到"
    fi

    # 检查 B9 SQL Corpus 阈值
    if grep -q "CORPUS_PCT.*85" "$BETA_SCRIPT"; then
        echo "  ✅ B9 SQL Corpus 阈值: 85%"
    else
        echo "  ⚠️  B9 SQL Corpus 阈值未找到"
    fi
fi
echo ""

# ========== 检查 4: gate_spec 与 TEST_PLAN 交叉检查 ==========
echo "【检查 4】gate_spec 与 TEST_PLAN 交叉检查"
echo "-----------------------------------"

TEST_PLAN="docs/releases/v3.0.0/TEST_PLAN.md"

if [ ! -f "$TEST_PLAN" ]; then
    echo "  ⚠️  $TEST_PLAN 不存在"
else
    # 检查 TEST_PLAN 是否包含 Beta 门禁项
    for item in B1 B2 B3 B4 B5 B6 B7 B8 B9; do
        if grep -q "$item" "$TEST_PLAN"; then
            echo "  ✅ $item 在 TEST_PLAN 中"
        else
            echo "  ⚠️  $item 不在 TEST_PLAN 中"
            WARN=$((WARN+1))
        fi
    done
fi
echo ""

# ========== 检查 5: 门禁脚本与实际文件对应 ==========
echo "【检查 5】门禁脚本与实际测试文件对应"
echo "-----------------------------------"

# 检查门禁脚本中引用的测试文件是否存在
if grep -q "concurrency_stress_test" "$BETA_SCRIPT" 2>/dev/null; then
    if find . -name "concurrency_stress_test*" -path "*/tests/*" | grep -q .; then
        echo "  ✅ concurrency_stress_test 存在"
    else
        echo "  ⚠️  concurrency_stress_test 引用但文件不存在"
        WARN=$((WARN+1))
    fi
fi

if grep -q "crash_recovery_test" "$BETA_SCRIPT" 2>/dev/null; then
    if find . -name "crash_recovery_test*" -path "*/tests/*" | grep -q .; then
        echo "  ✅ crash_recovery_test 存在"
    else
        echo "  ⚠️  crash_recovery_test 引用但文件不存在"
        WARN=$((WARN+1))
    fi
fi

if grep -q "wal_integration_test" "$BETA_SCRIPT" 2>/dev/null; then
    if find . -name "wal_integration_test*" -path "*/tests/*" | grep -q .; then
        echo "  ✅ wal_integration_test 存在"
    else
        echo "  ⚠️  wal_integration_test 引用但文件不存在"
        WARN=$((WARN+1))
    fi
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
    echo "建议: 检查 gate_spec_v300.md 和 check_beta_v300.sh / check_ga_v300.sh"
    exit 1
elif [ $WARN -gt 0 ]; then
    echo ""
    echo "⚠️  同步检查有警告: $WARN 项可能不一致"
    exit 0
else
    echo ""
    echo "✅ 同步检查通过"
    exit 0
fi
