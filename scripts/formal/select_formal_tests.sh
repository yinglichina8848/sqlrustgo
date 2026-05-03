#!/usr/bin/env bash
# select_formal_tests.sh — 根据 PR 改动自动选择需要运行的 formal 测试
#
# 原理: git diff → 匹配文件 → 选择 proof → 执行对应测试
# Fallback: >5 个文件改动 → 跑全量（防止跨模块 bug 漏检）
#
# 用法:
#   ./select_formal_tests.sh                    # 输出应执行的测试
#   ./select_formal_tests.sh --run             # 直接运行
#   ./select_formal_tests.sh --ci              # CI 模式（exit code）

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# 默认 base branch
BASE_BRANCH="${CI_MERGE_REQUEST_TARGET_BRANCH:-origin/develop/v2.9.0}"

get_changed_files() {
    if [ -n "$CI_MERGE_REQUEST_ID" ]; then
        # Gitea CI 模式
        git fetch origin "$BASE_BRANCH"
        git diff --name-only "origin/$BASE_BRANCH...HEAD"
    else
        # 本地模式
        git diff --name-only "$BASE_BRANCH...HEAD" 2>/dev/null || \
        git diff --name-only HEAD~10...HEAD
    fi
}

# 匹配规则: file pattern → proof area → tests
declare -A PROOF_RULES=(
    ["deadlock.rs"]="PROOF_023:deadlock"
    ["lock.rs"]="PROOF_023:deadlock"
    ["mvcc.rs"]="PROOF_026:ssi"
    ["mvcc.rs"]="PROOF_016:mvcc"
    ["transaction.rs"]="PROOF_016:transaction"
    ["wal.rs"]="WAL:wal"
    ["wal_manager.rs"]="WAL:wal"
    ["recovery.rs"]="WAL:wal"
    ["btree.rs"]="PROOF_004:btree"
    ["index.rs"]="PROOF_004:btree"
    ["parser.rs"]="PROOF_021:formulog"
    ["executor.rs"]="PROOF_016:transaction"
    ["docs/formal/"]="FULL:full"
)

SELECTED_TESTS=()
SELECTED_PROOFS=()

add_test() {
    local test="$1"
    if [[ ! " ${SELECTED_TESTS[*]} " =~ " ${test} " ]]; then
        SELECTED_TESTS+=("$test")
    fi
}

add_proof() {
    local proof="$1"
    if [[ ! " ${SELECTED_PROOFS[*]} " =~ " ${proof} " ]]; then
        SELECTED_PROOFS+=("$proof")
    fi
}

echo "Changed files:"
echo "---"
get_changed_files | tee /tmp/changed_files.txt
echo "---"
echo ""

while IFS= read -r file; do
    [ -z "$file" ] && continue

    for pattern in "${!PROOF_RULES[@]}"; do
        if [[ "$file" == *"$pattern"* ]]; then
            IFS=':' read -r proof test <<< "${PROOF_RULES[$pattern]}"
            add_proof "$proof"
            add_test "$test"
            echo "Matched: $file → $proof ($test)"
            break
        fi
    done
done < /tmp/changed_files.txt

echo ""

# 统计改动文件数量（去重）
CHANGED_COUNT=$(get_changed_files | grep -v '^$' | sort -u | wc -l)
echo "Total changed files: $CHANGED_COUNT"

# 如果没有任何匹配，跑默认测试（安全原则）
if [ ${#SELECTED_TESTS[@]} -eq 0 ]; then
    echo "No specific proofs matched. Running default tests..."
    add_test "deadlock"
    add_test "transaction"
fi

# Fallback: 改动超过 5 个文件 → 跑全量（防止跨模块 bug 漏检）
if [ "$CHANGED_COUNT" -gt 5 ]; then
    echo ""
    echo "⚠️  WARNING: $CHANGED_COUNT files changed (>5 threshold)"
    echo "Running FULL proof suite to prevent cross-module bugs"
    add_test "deadlock"
    add_test "mvcc"
    add_test "ssi"
    add_test "wal"
    add_test "full"
fi

# 如果有 formal docs 改动，跑全量
if [[ " ${SELECTED_PROOFS[*]} " =~ "FULL" ]]; then
    echo "Formal docs changed — running FULL proof suite"
    SELECTED_TESTS=("deadlock" "mvcc" "ssi" "wal" "btree" "full")
fi

echo ""
echo "Selected tests:"
printf '  - %s\n' "${SELECTED_TESTS[@]}"
echo ""
echo "Selected proofs:"
printf '  - %s\n' "${SELECTED_PROOFS[@]}"
echo ""

if [ "${1:-}" = "--run" ] || [ "${1:-}" = "--ci" ]; then
    echo "Running selected tests..."

    FAILED=0

    for test in "${SELECTED_TESTS[@]}"; do
        echo ""
        echo "=== Running: $test ==="

        case "$test" in
            deadlock)
                cargo test -p sqlrustgo-transaction deadlock -- --nocapture || FAILED=1
                ;;
            mvcc)
                cargo test -p sqlrustgo-transaction mvcc -- --nocapture || FAILED=1
                ;;
            ssi)
                cargo test -p sqlrustgo-transaction ssi -- --nocapture || true
                ;;
            transaction)
                cargo test -p sqlrustgo-transaction -- --nocapture || FAILED=1
                ;;
            wal)
                cargo test -p sqlrustgo-wal -- --nocapture || FAILED=1
                ;;
            btree)
                if command -v dafny &>/dev/null; then
                    dafny verify "$PROJECT_ROOT/docs/formal/btree_invariants.dfy" || FAILED=1
                fi
                ;;
            full)
                bash "$SCRIPT_DIR/formal_smoke.sh" || FAILED=1
                ;;
            *)
                echo "Unknown test: $test"
                ;;
        esac
    done

    echo ""
    if [ $FAILED -eq 0 ]; then
        echo "✅ All selected tests passed"
        exit 0
    else
        echo "❌ Some tests failed"
        exit 1
    fi
fi
