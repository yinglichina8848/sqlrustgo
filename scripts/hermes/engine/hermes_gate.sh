#!/usr/bin/env bash
#===============================================================================
# Hermes Gate Engine v0.2
# Architecture: Gate (pre-merge) + CI/Proof/Audit (ground truth)
#
# Layer 1 — BLOCKING (system state gates):
#   PROOF_VERIFIED  → checks verification_report.json baseline_verified
#   AUDIT_TRUSTED   → checks audit_report.json status == TRUSTED
#
# Layer 2 — PR hygiene (static checks):
#   REQUIRE_ISSUE, SQL_SEMANTIC_TEST, ROADMAP_PRIORITY, TEST_COMPLETENESS
#
# Exit codes: 0=PASS, 1=FAIL, 2=BLOCK, 3=ERROR
#===============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RULES_FILE="${SCRIPT_DIR}/../rules/core.json"
VERIFICATION_REPORT="${3:-docs/versions/v2.8.0/verification_report.json}"
AUDIT_REPORT="${4:-docs/versions/v2.8.0/audit_report.json}"

#-------------------------------------------------------------------------------
# Helpers
#-------------------------------------------------------------------------------
get_exit_code() {
    case "$1" in
        PASS) echo 0 ;;
        FAIL) echo 1 ;;
        BLOCK) echo 2 ;;
        ERROR) echo 3 ;;
        *) echo 3 ;;
    esac
}

json_val() {
    local file="$1"
    local key="$2"
    # Extract JSON value without jq dependency
    if [[ ! -f "$file" ]]; then
        echo ""
        return
    fi
    grep -o "\"$key\"[[:space:]]*:[[:space:]]*[^,}]*" "$file" 2>/dev/null \
        | sed 's/.*:[[:space:]]*//' \
        | tr -d ' "'
}

#-------------------------------------------------------------------------------
# Layer 1: System State Gates (BLOCK by default — no proof = no merge)
#-------------------------------------------------------------------------------

eval_PROOF_VERIFIED() {
    if [[ ! -f "$VERIFICATION_REPORT" ]]; then
        echo "BLOCK (verification_report.json not found)"
        return
    fi

    local baseline_verified
    baseline_verified=$(json_val "$VERIFICATION_REPORT" "baseline_verified")

    if [[ "$baseline_verified" == "true" ]]; then
        echo "PASS"
    else
        echo "BLOCK (baseline_verified=$baseline_verified, expected=true)"
    fi
}

eval_AUDIT_TRUSTED() {
    if [[ ! -f "$AUDIT_REPORT" ]]; then
        echo "BLOCK (audit_report.json not found)"
        return
    fi

    local status
    status=$(json_val "$AUDIT_REPORT" "status")

    if [[ "$status" == "TRUSTED" ]]; then
        echo "PASS"
    else
        echo "BLOCK (status=$status, expected=TRUSTED)"
    fi
}

#-------------------------------------------------------------------------------
# Layer 2: PR Hygiene Checks
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

    # SQL changed — require NULL and JOIN+WHERE tests exist in test files
    local null_test=false
    local join_test=false

    local test_files
    test_files=$(find . -name "*.rs" -path "*/tests/*" 2>/dev/null | head -50 || true)

    for f in $test_files; do
        local content
        content=$(cat "$f" 2>/dev/null || true)
        echo "$content" | grep -qiE "null" && null_test=true
        echo "$content" | grep -qiE "join|where" && join_test=true
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
    local roadmap="docs/roadmap.json"

    if ! echo "$pr_labels" | grep -qiE "P1|P2"; then
        echo "PASS"
        return
    fi

    if [[ ! -f "$roadmap" ]]; then
        echo "PASS"
        return
    fi

    # Block if P0 exists and is unfinished
    if grep -q "P0" "$roadmap" 2>/dev/null; then
        if grep -E '"priority".*"P0".*"status".*"(open|in_progress|blocked)"' "$roadmap" >/dev/null 2>&1; then
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
# Main
#-------------------------------------------------------------------------------
run_gate() {
    local pr_body="${1:-}"
    local pr_labels="${2:-}"
    local changed_files="${3:-}"

    echo ""
    echo "=============================================="
    echo "           Hermes Gate v0.2"
    echo "=============================================="
    echo ""

    local final_decision="PASS"

    # ── Layer 1: System State (BLOCKING) ──────────────────────────────────────
    echo "--- System State Gates ---"

    local r1 r2
    r1=$(eval_PROOF_VERIFIED)
    echo "[PROOF_VERIFIED] $r1"
    [[ "$r1" == "BLOCK"* ]] && final_decision="BLOCK"

    r2=$(eval_AUDIT_TRUSTED)
    echo "[AUDIT_TRUSTED] $r2"
    [[ "$r2" == "BLOCK"* ]] && final_decision="BLOCK"

    # ── Layer 2: PR Hygiene ───────────────────────────────────────────────────
    echo ""
    echo "--- PR Hygiene Gates ---"

    local r3 r4 r5 r6

    r3=$(eval_REQUIRE_ISSUE "$pr_body")
    echo "[REQUIRE_ISSUE] $r3"
    [[ "$r3" == "FAIL"* && "$final_decision" != "BLOCK" ]] && final_decision="FAIL"

    r4=$(eval_SQL_SEMANTIC_TEST "$changed_files")
    echo "[SQL_SEMANTIC_TEST] $r4"
    [[ "$r4" == "FAIL"* && "$final_decision" != "BLOCK" ]] && final_decision="FAIL"

    r5=$(eval_ROADMAP_PRIORITY "$pr_labels")
    echo "[ROADMAP_PRIORITY] $r5"
    [[ "$r5" == "BLOCK"* ]] && final_decision="BLOCK"

    r6=$(eval_TEST_COMPLETENESS "$changed_files")
    echo "[TEST_COMPLETENESS] $r6"
    [[ "$r6" == "FAIL"* && "$final_decision" != "BLOCK" ]] && final_decision="FAIL"

    echo ""
    echo "=============================================="
    echo "  Final Decision: $final_decision"
    echo "=============================================="
    echo ""

    exit $(get_exit_code "$final_decision")
}

run_gate "$@"
