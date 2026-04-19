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
OUT_FILE="$OUT_DIR/DOCUMENT_AUDIT.md"

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
if [ -d "$RELEASE_DIR" ]; then
  add_pass "版本文档目录存在：$RELEASE_DIR"
else
  add_fail "版本文档目录缺失：$RELEASE_DIR"
  add_capa "P0|创建版本文档目录并补齐最低文档集合。"
fi

for required in README.md VERSION_PLAN.md DEVELOPMENT_PLAN.md TEST_PLAN.md RELEASE_GATE_CHECKLIST.md RELEASE_NOTES.md; do
  if [ -f "$RELEASE_DIR/$required" ]; then
    add_pass "文档存在：$RELEASE_DIR/$required"
  else
    add_warn "文档缺失：$RELEASE_DIR/$required"
    add_capa "P1|补齐版本核心文档：$required。"
  fi
done

BROKEN_OUTPUT="$OUT_DIR/_broken_links.txt"
if [ -f "scripts/gate/check_docs_links.sh" ]; then
  set +e
  bash scripts/gate/check_docs_links.sh --all > "$OUT_DIR/_docs_link_check_raw.txt" 2>&1
  set -e
  rg "^docs/releases/$VERSION/" "$OUT_DIR/_docs_link_check_raw.txt" > "$BROKEN_OUTPUT" || true
  BROKEN_COUNT="$(wc -l < "$BROKEN_OUTPUT" | tr -d ' ')"
  if [ "$BROKEN_COUNT" -eq 0 ]; then
    add_pass "版本文档链接检查通过（未发现坏链）"
  else
    add_warn "发现版本文档坏链：$BROKEN_COUNT"
    add_capa "P1|修复 docs/releases/$VERSION 下坏链并加入门禁。"
  fi
else
  add_warn "缺少链接检查脚本：scripts/gate/check_docs_links.sh"
  add_capa "P2|补齐链接检查脚本并接入门禁。"
fi

if [ -d "docs/ai_collaboration" ] && [ -d "docs/AI增强软件工程" ]; then
  DUP_COUNT="$(find docs/ai_collaboration -type f 2>/dev/null | wc -l | tr -d ' ')"
  if [ "$DUP_COUNT" -gt 0 ]; then
    add_warn "检测到潜在重复目录并存：docs/ai_collaboration 与 docs/AI增强软件工程"
    add_capa "P2|统一 canonical 目录并提供迁移说明页。"
  else
    add_pass "重复目录风险较低（ai_collaboration 为空或仅索引）"
  fi
fi

VERSION_MISMATCH_COUNT=0
if [ -d "$RELEASE_DIR" ]; then
  VERSION_MISMATCH_COUNT="$(rg -n "v[0-9]+\\.[0-9]+\\.[0-9]+" "$RELEASE_DIR" -g '*.md' | rg -v "$VERSION" | wc -l | tr -d ' ')"
  if [ "$VERSION_MISMATCH_COUNT" -le 5 ]; then
    add_pass "版本文本基本一致（疑似异构引用数：$VERSION_MISMATCH_COUNT）"
  else
    add_warn "版本文本可能不一致（疑似异构引用数：$VERSION_MISMATCH_COUNT）"
    add_capa "P2|统一版本文档中的版本号文本与状态口径。"
  fi
fi

RESULT="PASS"
if [ "$FAIL_COUNT" -gt 0 ]; then
  RESULT="FAIL"
elif [ "$WARN_COUNT" -gt 0 ]; then
  RESULT="WARN"
fi

{
  echo "# DOCUMENT_AUDIT"
  echo
  echo "- Version: \`$VERSION\`"
  echo "- Stage: \`$STAGE\`"
  echo "- Date: $(date '+%Y-%m-%d %H:%M:%S %z')"
  echo "- RESULT: $RESULT"
  echo
  echo "## Metrics"
  echo
  echo "- BROKEN_LINK_COUNT_IN_VERSION_DOCS: ${BROKEN_COUNT:-0}"
  echo "- VERSION_TEXT_MISMATCH_HINT_COUNT: $VERSION_MISMATCH_COUNT"
  echo
  echo "## Broken Links (version scope)"
  if [ -f "$BROKEN_OUTPUT" ] && [ -s "$BROKEN_OUTPUT" ]; then
    sed 's/^/- /' "$BROKEN_OUTPUT"
  else
    echo "- None"
  fi
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
    echo "- CAPA: P3|文档治理状态稳定，持续执行链接与命令一致性检查。"
  else
    for item in "${CAPA_ITEMS[@]}"; do
      echo "- CAPA: $item"
    done
  fi
} > "$OUT_FILE"

echo "Generated $OUT_FILE"
