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
OUT_FILE="$OUT_DIR/TEST_AUDIT.md"

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

TEST_PLAN="docs/releases/$VERSION/TEST_PLAN.md"
if [ -f "$TEST_PLAN" ]; then
  add_pass "测试计划存在：$TEST_PLAN"
else
  add_fail "测试计划缺失：$TEST_PLAN"
  add_capa "P0|补齐版本测试计划并绑定门禁阈值。"
fi

ROOT_TESTS_TMP="$(mktemp)"
DOC_TESTS_TMP="$(mktemp)"
INVALID_TESTS_TMP="$(mktemp)"
UNDOC_TESTS_TMP="$(mktemp)"

cleanup() {
  rm -f "$ROOT_TESTS_TMP" "$DOC_TESTS_TMP" "$INVALID_TESTS_TMP" "$UNDOC_TESTS_TMP"
}
trap cleanup EXIT

awk '
  /^\[\[test\]\]/{in_test=1; next}
  in_test && /^name = /{
    gsub(/name = "|"/, "", $0);
    print $0;
    in_test=0
  }
' Cargo.toml | sort -u > "$ROOT_TESTS_TMP"

if [ -f "$TEST_PLAN" ]; then
  rg -o --no-filename "cargo test --test [a-zA-Z0-9_\\-]+" "$TEST_PLAN" | awk '{print $4}' | sort -u > "$DOC_TESTS_TMP" || true
fi

if [ -s "$DOC_TESTS_TMP" ]; then
  comm -23 "$DOC_TESTS_TMP" "$ROOT_TESTS_TMP" > "$INVALID_TESTS_TMP" || true
  comm -13 "$DOC_TESTS_TMP" "$ROOT_TESTS_TMP" > "$UNDOC_TESTS_TMP" || true
fi

INVALID_DOC_TEST_COUNT="$(wc -l < "$INVALID_TESTS_TMP" | tr -d ' ')"
UNDOC_ROOT_TEST_COUNT="$(wc -l < "$UNDOC_TESTS_TMP" | tr -d ' ')"
ROOT_TEST_COUNT="$(wc -l < "$ROOT_TESTS_TMP" | tr -d ' ')"

if [ "$INVALID_DOC_TEST_COUNT" -eq 0 ]; then
  add_pass "测试计划中的 --test target 与 Cargo.toml 一致"
else
  add_warn "检测到无效测试 target：$INVALID_DOC_TEST_COUNT"
  add_capa "P1|修正 TEST_PLAN 中无效的 cargo test --test 目标。"
fi

if [ "$UNDOC_ROOT_TEST_COUNT" -le 5 ]; then
  add_pass "未文档化 root test target 数量可接受：$UNDOC_ROOT_TEST_COUNT"
else
  add_warn "未文档化 root test target 偏多：$UNDOC_ROOT_TEST_COUNT"
  add_capa "P2|在 TEST_PLAN 增加 root test target 覆盖清单。"
fi

if [ -f "crates/sql-corpus/tests/corpus_test.rs" ]; then
  add_pass "SQL Corpus 测试入口存在"
else
  add_fail "SQL Corpus 测试入口缺失：crates/sql-corpus/tests/corpus_test.rs"
  add_capa "P0|恢复 SQL Corpus 测试入口并加入门禁。"
fi

if rg -n "TPC-H|TPC-H|tpch" "$TEST_PLAN" >/dev/null 2>&1; then
  add_pass "测试计划包含 TPC-H 维度"
else
  add_warn "测试计划未明确 TPC-H 测试策略"
  add_capa "P1|补充 TPC-H 阶段策略（SF1 必跑，SF10 夜间）。"
fi

if rg -n "Sysbench|sysbench" "$TEST_PLAN" >/dev/null 2>&1; then
  add_pass "测试计划包含 Sysbench 维度"
else
  add_warn "测试计划未明确 Sysbench 测试"
  add_capa "P1|补充 Sysbench 三场景与阈值。"
fi

if rg -n "备份|恢复|崩溃|crash|backup" "$TEST_PLAN" >/dev/null 2>&1; then
  add_pass "测试计划包含恢复类测试维度"
else
  add_warn "测试计划未明确备份/恢复/崩溃测试"
  add_capa "P1|补充备份恢复与崩溃注入测试要求。"
fi

if [ -f "tarpaulin.toml" ]; then
  add_pass "覆盖率配置存在：tarpaulin.toml"
else
  add_warn "覆盖率配置缺失：tarpaulin.toml"
  add_capa "P2|补充覆盖率工具配置并固化阈值。"
fi

RESULT="PASS"
if [ "$FAIL_COUNT" -gt 0 ]; then
  RESULT="FAIL"
elif [ "$WARN_COUNT" -gt 0 ]; then
  RESULT="WARN"
fi

{
  echo "# TEST_AUDIT"
  echo
  echo "- Version: \`$VERSION\`"
  echo "- Stage: \`$STAGE\`"
  echo "- Date: $(date '+%Y-%m-%d %H:%M:%S %z')"
  echo "- RESULT: $RESULT"
  echo
  echo "## Metrics"
  echo
  echo "- ROOT_TEST_COUNT: $ROOT_TEST_COUNT"
  echo "- INVALID_DOC_TEST_TARGET_COUNT: $INVALID_DOC_TEST_COUNT"
  echo "- UNDOCUMENTED_ROOT_TEST_COUNT: $UNDOC_ROOT_TEST_COUNT"
  echo
  echo "## Invalid Doc Targets"
  if [ "$INVALID_DOC_TEST_COUNT" -eq 0 ]; then
    echo "- None"
  else
    sed 's/^/- /' "$INVALID_TESTS_TMP"
  fi
  echo
  echo "## Undocumented Root Tests"
  if [ "$UNDOC_ROOT_TEST_COUNT" -eq 0 ]; then
    echo "- None"
  else
    sed 's/^/- /' "$UNDOC_TESTS_TMP"
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
    echo "- CAPA: P3|测试策略稳定，继续执行分层测试。"
  else
    for item in "${CAPA_ITEMS[@]}"; do
      echo "- CAPA: $item"
    done
  fi
} > "$OUT_FILE"

echo "Generated $OUT_FILE"
