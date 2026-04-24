#!/usr/bin/env bash
#===============================================================================
# Hermes Gate Engine v0.1
# SQLRustGo 可执行规则系统
#===============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RULES_FILE="${SCRIPT_DIR}/../rules/core.json"
ROADMAP_FILE="docs/roadmap.json"

#-------------------------------------------------------------------------------
# Input parsing
#-------------------------------------------------------------------------------
PR_BODY=""
PR_LABELS=""
CHANGED_FILES=""
ROADMAP_CONTENT=""

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --pr-body)
                PR_BODY="$2"; shift 2 ;;
            --pr-labels)
                PR_LABELS="$2"; shift 2 ;;
            --changed-files)
                CHANGED_FILES="$2"; shift 2 ;;
            --roadmap)
                ROADMAP_CONTENT="$2"; shift 2 ;;
            *)
                echo "Unknown option: $1" >&2; exit 3 ;;
        esac
    done
}

get_exit_code() {
    case "$1" in
        PASS) echo 0 ;;
        FAIL) echo 1 ;;
        BLOCK) echo 2 ;;
        ERROR) echo 3 ;;
        *) echo 3 ;;
    esac
}

#-------------------------------------------------------------------------------
# Rule evaluators
#-------------------------------------------------------------------------------

eval_REQUIRE_ISSUE() {
    local pr_body="$1"
    if [[ -z "$pr_body" ]]; then
        echo "FAIL"
        return
    fi
    if echo "$pr_body" | grep -qiE "closes?\s+#[[:digit:]]+|fixes?\s+#[[:digit:]]+|refs?\s+#[[:digit:]]+"; then
        echo "PASS"
    else
        echo "FAIL"
    fi
}

eval_SQL_SEMANTIC_TEST() {
    local changed_files="$1"

    # Check if any SQL-related files changed
    local sql_pattern="executor|parser|planner|storage"
    local sql_changed=false
    for f in $changed_files; do
        case "$f" in
            *executor*|*parser*|*planner*|*storage*)
                [[ "$f" == *.rs ]] && sql_changed=true && break
                ;;
        esac
    done

    if [[ "$sql_changed" == "false" ]]; then
        echo "PASS"
        return
    fi

    # SQL changed — require NULL and JOIN+WHERE tests
    local null_test=false
    local join_test=false

    local test_files
    test_files=$(find . -name "*.rs" -path "*/tests/*" 2>/dev/null | head -50 || true)

    for f in $test_files; do
        local content
        content=$(cat "$f" 2>/dev/null || true)
        if echo "$content" | grep -qiE "null"; then
            null_test=true
        fi
        if echo "$content" | grep -qiE "join|where"; then
            join_test=true
        fi
    done

    if [[ "$null_test" == "true" && "$join_test" == "true" ]]; then
        echo "PASS"
    elif [[ "$null_test" == "true" ]]; then
        echo "FAIL (missing JOIN+WHERE tests)"
    elif [[ "$join_test" == "true" ]]; then
        echo "FAIL (missing NULL logic tests)"
    else
        echo "FAIL (missing NULL tests AND JOIN+WHERE tests)"
    fi
}

eval_ROADMAP_PRIORITY() {
    local pr_labels="$1"

    if ! echo "$pr_labels" | grep -qiE "P1|P2"; then
        echo "PASS"
        return
    fi

    # Check P0 completeness
    local roadmap_path="${ROADMAP_CONTENT:-$ROADMAP_FILE}"
    if [[ ! -f "$roadmap_path" ]]; then
        echo "PASS"
        return
    fi

    # Check for unfinished P0 issues
    if grep -q "P0" "$roadmap_path" 2>/dev/null; then
        # Simple check: if file has "P0" and "open" or "in_progress" nearby
        if grep -E '"priority".*"P0".*"status".*"(open|in_progress|blocked)"' "$roadmap_path" >/dev/null 2>&1; then
            echo "BLOCK"
            return
        fi
    fi

    echo "PASS"
}

eval_TEST_COMPLETENESS() {
    local changed_files="$1"

    # Skip for docs-only changes
    local docs_only=true
    for f in $changed_files; do
        case "$f" in
            *.md|docs/*|wiki/*) ;;
            *) docs_only=false; break ;;
        esac
    done

    if [[ "$docs_only" == "true" ]]; then
        echo "PASS"
        return
    fi

    # Check test coverage categories
    local test_files
    test_files=$(find . -name "*.rs" -path "*/tests/*" 2>/dev/null | head -50 || true)
    local happy=false edge=false regression=false

    for f in $test_files; do
        local content
        content=$(cat "$f" 2>/dev/null || true)
        echo "$content" | grep -qiE "happy|success|basic|smoke" && happy=true
        echo "$content" | grep -qiE "edge|boundary" && edge=true
        echo "$content" | grep -qiE "regression|regress|bug.*fix|fix.*bug" && regression=true
    done

    local missing=""
    [[ "$happy" == "false" ]] && missing="${missing}happy_path, "
    [[ "$edge" == "false" ]] && missing="${missing}edge_case, "
    [[ "$regression" == "false" ]] && missing="${missing}regression, "

    if [[ -z "$missing" ]]; then
        echo "PASS"
    else
        echo "FAIL (missing: ${missing%, })"
    fi
}

#-------------------------------------------------------------------------------
# Main gate logic
#-------------------------------------------------------------------------------
run_gate() {
    parse_args "$@"

    echo ""
    echo "=============================================="
    echo "           Hermes Gate v0.1"
    echo "=============================================="
    echo ""

    local final_decision="PASS"

    # Rule 1: REQUIRE_ISSUE
    local r1
    r1=$(eval_REQUIRE_ISSUE "$PR_BODY")
    echo "[REQUIRE_ISSUE] $r1"
    [[ "$r1" == "FAIL"* ]] && final_decision="FAIL"

    # Rule 2: SQL_SEMANTIC_TEST
    local r2
    r2=$(eval_SQL_SEMANTIC_TEST "$CHANGED_FILES")
    echo "[SQL_SEMANTIC_TEST] $r2"
    [[ "$r2" == "FAIL"* ]] && final_decision="FAIL"
    [[ "$r2" == "BLOCK"* ]] && final_decision="BLOCK"

    # Rule 3: ROADMAP_PRIORITY
    local r3
    r3=$(eval_ROADMAP_PRIORITY "$PR_LABELS")
    echo "[ROADMAP_PRIORITY] $r3"
    [[ "$r3" == "BLOCK"* ]] && final_decision="BLOCK"

    # Rule 4: TEST_COMPLETENESS
    local r4
    r4=$(eval_TEST_COMPLETENESS "$CHANGED_FILES")
    echo "[TEST_COMPLETENESS] $r4"
    [[ "$r4" == "FAIL"* && "$final_decision" != "BLOCK" ]] && final_decision="FAIL"

    echo ""
    echo "=============================================="
    echo "  Final Decision: $final_decision"
    echo "=============================================="
    echo ""

    exit $(get_exit_code "$final_decision")
}

run_gate "$@"
