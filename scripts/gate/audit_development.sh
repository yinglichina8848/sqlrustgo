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
OUT_FILE="$OUT_DIR/DEV_AUDIT.md"

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

DEV_PLAN="docs/releases/$VERSION/DEVELOPMENT_PLAN.md"
if [ -f "$DEV_PLAN" ]; then
  add_pass "开发计划存在：$DEV_PLAN"
else
  add_fail "开发计划缺失：$DEV_PLAN"
  add_capa "P0|补齐版本开发计划并与阶段任务对齐。"
fi

TODO_COUNT="$(rg -n "TODO|FIXME" src crates tests 2>/dev/null | wc -l | tr -d ' ')"
if [ "$TODO_COUNT" -le 150 ]; then
  add_pass "TODO/FIXME 规模可控：$TODO_COUNT"
else
  add_warn "TODO/FIXME 偏多：$TODO_COUNT"
  add_capa "P1|建立技术债清理配额，每阶段至少清理 Top 10 风险 TODO/FIXME。"
fi

UNWRAP_COUNT="$(rg -n "\.unwrap\(|\.expect\(" src crates --glob '!**/tests/**' 2>/dev/null | wc -l | tr -d ' ')"
if [ "$UNWRAP_COUNT" -le 300 ]; then
  add_pass "unwrap/expect 使用规模可控：$UNWRAP_COUNT"
else
  add_warn "unwrap/expect 使用偏多：$UNWRAP_COUNT"
  add_capa "P1|关键路径替换 unwrap/expect，增加错误传播与上下文。"
fi

UNSAFE_COUNT="$(rg -n "\bunsafe\b" src crates 2>/dev/null | wc -l | tr -d ' ')"
if [ "$UNSAFE_COUNT" -eq 0 ]; then
  add_pass "未检测到 unsafe 块"
else
  add_warn "检测到 unsafe 使用：$UNSAFE_COUNT"
  add_capa "P2|为 unsafe 代码补齐安全性注释与测试证明。"
fi

RUST_FILES="$(rg --files -g '*.rs' src crates tests 2>/dev/null | wc -l | tr -d ' ')"
TEST_FILES="$(rg --files -g '*test*.rs' tests crates 2>/dev/null | wc -l | tr -d ' ')"
if [ "$RUST_FILES" -gt 0 ]; then
  TEST_RATIO=$(( (TEST_FILES * 100) / RUST_FILES ))
else
  TEST_RATIO=0
fi

if [ "$TEST_RATIO" -ge 8 ]; then
  add_pass "测试文件密度良好：$TEST_FILES/$RUST_FILES (~${TEST_RATIO}%)"
else
  add_warn "测试文件密度偏低：$TEST_FILES/$RUST_FILES (~${TEST_RATIO}%)"
  add_capa "P1|按模块补齐测试文件密度，优先覆盖事务/存储/执行器。"
fi

if rg -n "Issue|任务|Milestone|Phase|阶段" "$DEV_PLAN" >/dev/null 2>&1; then
  add_pass "开发计划包含任务分解与阶段语义"
else
  add_warn "开发计划缺少明确任务分解语义"
  add_capa "P2|在开发计划中加入 Issue 映射与阶段任务矩阵。"
fi

RESULT="PASS"
if [ "$FAIL_COUNT" -gt 0 ]; then
  RESULT="FAIL"
elif [ "$WARN_COUNT" -gt 0 ]; then
  RESULT="WARN"
fi

{
  echo "# DEV_AUDIT"
  echo
  echo "- Version: \`$VERSION\`"
  echo "- Stage: \`$STAGE\`"
  echo "- Date: $(date '+%Y-%m-%d %H:%M:%S %z')"
  echo "- RESULT: $RESULT"
  echo
  echo "## Metrics"
  echo
  echo "- TODO_FIXME_COUNT: $TODO_COUNT"
  echo "- UNWRAP_EXPECT_COUNT: $UNWRAP_COUNT"
  echo "- UNSAFE_COUNT: $UNSAFE_COUNT"
  echo "- RUST_FILE_COUNT: $RUST_FILES"
  echo "- TEST_FILE_COUNT: $TEST_FILES"
  echo "- TEST_FILE_RATIO_PERCENT: $TEST_RATIO"
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
    echo "- CAPA: P3|开发质量稳定，持续执行当前策略。"
  else
    for item in "${CAPA_ITEMS[@]}"; do
      echo "- CAPA: $item"
    done
  fi
} > "$OUT_FILE"

echo "Generated $OUT_FILE"
