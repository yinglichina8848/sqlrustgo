#!/bin/bash
# scripts/gate/check_phase_entry.sh - 阶段入口文档检查
# 在进入每个阶段前必须执行此检查

set -e

VERSION=${1:-"v3.1.0"}
PHASE=${2:-""}

if [ -z "$PHASE" ]; then
    echo "Usage: check_phase_entry.sh <version> <phase>"
    echo "  e.g., check_phase_entry.sh v3.1.0 beta"
    exit 1
fi

DOCS_DIR="docs/releases/${VERSION}"
PHASE_UPPER=$(echo "$PHASE" | tr '[:lower:]' '[:upper:]')

echo "=== ${VERSION} ${PHASE_UPPER} 阶段入口文档检查 ==="
echo ""

ERRORS=0

# 核心必需文档检查
check_doc() {
    local doc="$1"
    local desc="$2"
    if [ -f "$DOCS_DIR/$doc" ]; then
        echo "✅ $desc: $doc"
        return 0
    else
        echo "❌ 缺失 $desc: $doc"
        ERRORS=$((ERRORS + 1))
        return 1
    fi
}

# Alpha 阶段必需
check_alpha_docs() {
    echo "--- Alpha 阶段必需文档 ---"
    check_doc "DEVELOPMENT_PLAN.md" "开发计划"
    check_doc "TEST_PLAN.md" "测试计划"
    check_doc "README.md" "README"
    check_doc "QUICK_START.md" "快速开始"
    echo ""
}

# Beta 阶段必需（Alpha + 以下）
check_beta_docs() {
    echo "--- Beta 阶段必需文档 ---"
    check_alpha_docs
    check_doc "BETA_GATE_CHECKLIST.md" "Beta 门禁清单"
    check_doc "ALPHA_GATE_REPORT.md" "Alpha 门禁报告"
    check_doc "COVERAGE_ANALYSIS_REPORT.md" "覆盖率分析报告"
    echo ""
}

# RC 阶段必需（Beta + 以下）
check_rc_docs() {
    echo "--- RC 阶段必需文档 ---"
    check_beta_docs
    check_doc "RC_GATE_CHECKLIST.md" "RC 门禁清单"
    check_doc "BETA_GATE_REPORT.md" "Beta 门禁报告"
    check_doc "SECURITY_REPORT.md" "安全报告"
    check_doc "PERFORMANCE_TARGETS.md" "性能目标"
    echo ""
}

# GA 阶段必需（RC + 以下）
check_ga_docs() {
    echo "--- GA 阶段必需文档 ---"
    check_rc_docs
    check_doc "GA_GATE_CHECKLIST.md" "GA 门禁清单"
    check_doc "RC_GATE_REPORT.md" "RC 门禁报告"
    check_doc "USER_MANUAL.md" "用户手册"
    check_doc "API_REFERENCE.md" "API 参考"
    check_doc "RELEASE_NOTES.md" "发布说明"
    check_doc "CHANGELOG.md" "变更日志"
    check_doc "UPGRADE_GUIDE.md" "升级指南"
    check_doc "BENCHMARK.md" "基准测试报告"
    check_doc "TEST_REPORT.md" "测试报告"
    check_doc "SECURITY_ANALYSIS.md" "安全分析"
    echo ""
}

case "$PHASE" in
    alpha)
        check_alpha_docs
        ;;
    beta)
        check_beta_docs
        ;;
    rc)
        check_rc_docs
        ;;
    ga)
        check_ga_docs
        ;;
    *)
        echo "未知阶段: $PHASE"
        echo "支持的阶段: alpha, beta, rc, ga"
        exit 1
        ;;
esac

echo ""
echo "=== 检查结果 ==="
if [ $ERRORS -eq 0 ]; then
    echo "✅ 所有必需文档存在"
    exit 0
else
    echo "❌ 缺失 $ERRORS 个必需文档"
    echo ""
    echo "请在进入 ${PHASE_UPPER} 阶段前补全上述缺失文档"
    exit 1
fi
