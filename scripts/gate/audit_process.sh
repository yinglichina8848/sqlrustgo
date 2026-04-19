#!/usr/bin/env bash

set -euo pipefail

VERSION="${1:-}"
STAGE="${2:-unknown}"
OUT_DIR="${3:-}"

if [ -z "$OUT_DIR" ]; then
  echo "Usage: $0 <version> <stage> <out_dir>"
  exit 2
fi

mkdir -p "$OUT_DIR"
OUT_FILE="$OUT_DIR/PROCESS_AUDIT.md"

PASS_COUNT=0
WARN_COUNT=0
FAIL_COUNT=0

PASS_ITEMS=()
WARN_ITEMS=()
FAIL_ITEMS=()
CAPA_ITEMS=()

add_pass() { PASS_ITEMS+=("$1"); PASS_COUNT=$((PASS_COUNT + 1)); }
add_warn() { WARN_ITEMS+=("$1"); WARN_COUNT=$((WARN_COUNT + 1)); }
add_fail() { FAIL_ITEMS+=("$1"); FAIL_COUNT=$((FAIL_COUNT + 1)); }
add_capa() { CAPA_ITEMS+=("$1"); }

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

if [ -z "$VERSION" ]; then
  if [ -f "CURRENT_VERSION.md" ]; then
    VERSION="$(head -n 1 CURRENT_VERSION.md | sed 's#.*/##' | tr -d '[:space:]')"
  else
    VERSION="unknown"
  fi
fi

RELEASE_DIR="docs/releases/$VERSION"

if [ -f "docs/governance/ENGINEERING_EVOLUTION_STANDARD.md" ]; then
  add_pass "工程演进主标准存在"
else
  add_fail "缺失工程演进主标准：docs/governance/ENGINEERING_EVOLUTION_STANDARD.md"
  add_capa "P0|补齐并冻结工程主标准文档，作为单一流程真相源。"
fi

if [ -f "docs/governance/AI_EXECUTION_PROMPTS.md" ] && [ -f "docs/governance/AI_SELF_OPTIMIZATION_PROMPTS.md" ]; then
  add_pass "AI 执行与自优化提示词文档齐备"
else
  add_fail "AI 提示词体系不完整（执行提示词或自优化提示词缺失）"
  add_capa "P0|补齐 AI 执行与自优化提示词文档。"
fi

if [ -d "$RELEASE_DIR" ]; then
  add_pass "版本目录存在：$RELEASE_DIR"
else
  add_fail "版本目录不存在：$RELEASE_DIR"
  add_capa "P0|创建版本目录并建立最小版本包文档。"
fi

for required in README.md VERSION_PLAN.md DEVELOPMENT_PLAN.md TEST_PLAN.md RELEASE_GATE_CHECKLIST.md; do
  if [ -f "$RELEASE_DIR/$required" ]; then
    add_pass "版本包文档存在：$RELEASE_DIR/$required"
  else
    add_warn "版本包文档缺失：$RELEASE_DIR/$required"
    add_capa "P1|补齐版本包最小文档集合，避免多口径。"
  fi
done

if [ -f "scripts/gate/run_self_optimization_cycle.sh" ]; then
  add_pass "闭环总控脚本存在"
else
  add_fail "闭环总控脚本缺失：scripts/gate/run_self_optimization_cycle.sh"
  add_capa "P0|创建阶段闭环总控脚本并纳入门禁。"
fi

for required_script in scripts/gate/audit_process.sh scripts/gate/audit_development.sh scripts/gate/audit_testing.sh scripts/gate/audit_documentation.sh; do
  if [ -f "$required_script" ]; then
    add_pass "专项审计脚本存在：$required_script"
  else
    add_warn "专项审计脚本缺失：$required_script"
    add_capa "P1|补齐四类专项审计脚本。"
  fi
done

BRANCH_NAME="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo unknown)"
if echo "$BRANCH_NAME" | rg -q '^develop/|^release/|^feature/|^fix/|^codex/'; then
  add_pass "分支命名符合治理约定：$BRANCH_NAME"
else
  add_warn "当前分支未命中常见治理前缀：$BRANCH_NAME"
  add_capa "P2|统一分支命名策略并在贡献指南中显式声明。"
fi

RESULT="PASS"
if [ "$FAIL_COUNT" -gt 0 ]; then
  RESULT="FAIL"
elif [ "$WARN_COUNT" -gt 0 ]; then
  RESULT="WARN"
fi

{
  echo "# PROCESS_AUDIT"
  echo
  echo "- Version: \`$VERSION\`"
  echo "- Stage: \`$STAGE\`"
  echo "- Branch: \`$BRANCH_NAME\`"
  echo "- Date: $(date '+%Y-%m-%d %H:%M:%S %z')"
  echo "- RESULT: $RESULT"
  echo
  echo "## Findings Summary"
  echo
  echo "- PASS: $PASS_COUNT"
  echo "- WARN: $WARN_COUNT"
  echo "- FAIL: $FAIL_COUNT"
  echo
  echo "## PASS Items"
  for item in "${PASS_ITEMS[@]}"; do echo "- $item"; done
  echo
  echo "## WARN Items"
  if [ "${#WARN_ITEMS[@]}" -eq 0 ]; then
    echo "- None"
  else
    for item in "${WARN_ITEMS[@]}"; do echo "- $item"; done
  fi
  echo
  echo "## FAIL Items"
  if [ "${#FAIL_ITEMS[@]}" -eq 0 ]; then
    echo "- None"
  else
    for item in "${FAIL_ITEMS[@]}"; do echo "- $item"; done
  fi
  echo
  echo "## CAPA Recommendations"
  if [ "${#CAPA_ITEMS[@]}" -eq 0 ]; then
    echo "- CAPA: P3|流程状态稳定，保持当前策略并持续监控。"
  else
    for item in "${CAPA_ITEMS[@]}"; do
      echo "- CAPA: $item"
    done
  fi
} > "$OUT_FILE"

echo "Generated $OUT_FILE"
