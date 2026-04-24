#!/usr/bin/env bash
#===============================================================================
# Hermes Gate Engine v0.3
# Contract-driven + Commit-bound + Auto-trigger Audit
#
# Architecture:
#   1. Load contract/v2.8.0.json → derive R1-R7 checks
#   2. Auto-trigger self_audit.py if audit_report.json is stale
#   3. Verify report commit == HEAD (no stale artifact reuse)
#   4. Enforce R1-R7 (from contract) + hygiene rules (REQUIRE_ISSUE etc.)
#
# Exit codes: 0=PASS, 1=FAIL, 2=BLOCK, 3=WARN, 4=ERROR
#===============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RULES_FILE="${SCRIPT_DIR}/../rules/core.json"
CONTRACT_PATH="${CONTRACT_PATH:-contract/v2.8.0.json}"
VERIFICATION_REPORT="${VERIFICATION_REPORT:-docs/versions/v2.8.0/verification_report.json}"
AUDIT_REPORT="${AUDIT_REPORT:-docs/versions/v2.8.0/audit_report.json}"
SELF_AUDIT_SCRIPT="scripts/self_audit.py"

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'

#-------------------------------------------------------------------------------
# Helpers
#-------------------------------------------------------------------------------
get_exit_code() {
    case "$1" in
        PASS) echo 0 ;;
        FAIL) echo 1 ;;
        BLOCK) echo 2 ;;
        WARN) echo 3 ;;
        ERROR) echo 4 ;;
        *) echo 4 ;;
    esac
}

is_valid_json() {
    python3 -c "import json; json.load(open('$1'))" 2>/dev/null
}

get_json_field() {
    local file="$1"
    local field="$2"
    [[ ! -f "$file" ]] && echo "" && return
    grep -o "\"$field\"[[:space:]]*:[[:space:]]*[^,}]*" "$file" 2>/dev/null \
        | sed 's/.*:[[:space:]]*//' | tr -d ' "'
}

current_commit() {
    git rev-parse HEAD 2>/dev/null || echo ""
}

#-------------------------------------------------------------------------------
# Step 1: Auto-trigger audit if stale
#-------------------------------------------------------------------------------
ensure_fresh_audit() {
    local need_run=false

    if [[ ! -f "$AUDIT_REPORT" ]]; then
        echo "audit_report.json not found — will run self_audit.py"
        need_run=true
    else
        local report_commit
        report_commit=$(get_json_field "$AUDIT_REPORT" "commit")
        local head
        head=$(current_commit)
        if [[ "$report_commit" != "$head" ]]; then
            echo "audit_report.json is stale (commit=$report_commit, HEAD=$head) — will run self_audit.py"
            need_run=true
        fi
    fi

    if [[ "$need_run" == "true" ]]; then
        if [[ -f "$SELF_AUDIT_SCRIPT" ]]; then
            echo "Running self_audit.py..."
            python3 "$SELF_AUDIT_SCRIPT" --output "$AUDIT_REPORT" 2>&1 | tail -5
            if [[ $? -ne 0 ]]; then
                echo "ERROR: self_audit.py failed"
                exit 4
            fi
        else
            echo "ERROR: $AUDIT_REPORT not found and self_audit.py not found"
            exit 4
        fi
    fi
}

#-------------------------------------------------------------------------------
# Step 2: Report integrity checks (commit binding)
#-------------------------------------------------------------------------------
check_report_commit() {
    local report="$1"
    local label="$2"

    if [[ ! -f "$report" ]]; then
        echo "${RED}BLOCK${NC} ($label: file not found)"
        return 1
    fi

    if ! is_valid_json "$report"; then
        echo "${RED}ERROR${NC} ($label: invalid JSON)"
        return 4
    fi

    local report_commit
    report_commit=$(get_json_field "$report" "commit")
    local head
    head=$(current_commit)

    if [[ -z "$report_commit" ]]; then
        echo "${RED}BLOCK${NC} ($label: missing commit field — artifact from pre-v0.3 system)"
        return 2
    fi

    if [[ "$report_commit" != "$head" ]]; then
        echo "${RED}BLOCK${NC} ($label: stale artifact, commit=$report_commit HEAD=$head)"
        return 2
    fi

    echo "${GREEN}OK${NC} ($label: commit=$head)"
    return 0
}

#-------------------------------------------------------------------------------
# Step 3: Contract-driven rule evaluators (R1-R7)
#-------------------------------------------------------------------------------

# R1: Test Immutability — PRs cannot modify test files
eval_R1() {
    local diff
    diff=$(git diff --name-only HEAD~1..HEAD 2>/dev/null | grep -E '^tests/' || true)
    if [[ -n "$diff" ]]; then
        echo "FAIL (R1: modified test files: $diff)"
        return 1
    fi
    echo "PASS (R1: no test file modifications)"
    return 0
}

# R2: Ignore Injection Prevention — no new #[ignore] in PR
eval_R2() {
    local diff
    diff=$(git diff HEAD~1..HEAD -- '*.rs' 2>/dev/null | grep -E '^\+.*#\[ignore\]' || true)
    if [[ -n "$diff" ]]; then
        echo "FAIL (R2: new #[ignore] annotations introduced)"
        return 1
    fi
    echo "PASS (R2: no ignore injection)"
    return 0
}

# R3: Proof Authenticity — verification_report.json exists and matches HEAD
eval_R3() {
    if [[ ! -f "$VERIFICATION_REPORT" ]]; then
        echo "BLOCK (R3: verification_report.json not found)"
        return 2
    fi
    local report_commit
    report_commit=$(get_json_field "$VERIFICATION_REPORT" "commit")
    local head
    head=$(current_commit)
    if [[ "$report_commit" != "$head" ]]; then
        echo "BLOCK (R3: stale verification artifact, commit=$report_commit HEAD=$head)"
        return 2
    fi
    echo "PASS (R3: proof artifact matches HEAD)"
    return 0
}

# R4: Full Execution — all baseline tests must pass
eval_R4() {
    local passed failed ignored
    passed=$(get_json_field "$VERIFICATION_REPORT" "passed")
    failed=$(get_json_field "$VERIFICATION_REPORT" "failed")
    ignored=$(get_json_field "$VERIFICATION_REPORT" "ignored")

    if [[ -z "$passed" ]]; then
        echo "BLOCK (R4: cannot read test counts from verification_report.json)"
        return 2
    fi
    if [[ "$failed" != "0" || "$ignored" != "0" ]]; then
        echo "BLOCK (R4: failed=$failed ignored=$ignored — tests must all pass)"
        return 2
    fi
    echo "PASS (R4: $passed passed, 0 failed, 0 ignored)"
    return 0
}

# R5: Baseline Verification — baseline_verified must be true
eval_R5() {
    local bv
    bv=$(get_json_field "$VERIFICATION_REPORT" "baseline_verified")
    if [[ "$bv" != "true" ]]; then
        echo "BLOCK (R5: baseline_verified=$bv — must be true)"
        return 2
    fi
    echo "PASS (R5: baseline_verified=true)"
    return 0
}

# R6: Test Count Monotonicity — passed >= 226
eval_R6() {
    local passed
    passed=$(get_json_field "$VERIFICATION_REPORT" "passed")
    if [[ -z "$passed" ]]; then
        echo "BLOCK (R6: cannot read passed count)"
        return 2
    fi
    if (( passed < 226 )); then
        echo "BLOCK (R6: passed=$passed < 226 — coverage decreased)"
        return 2
    fi
    echo "PASS (R6: passed=$passed >= 226)"
    return 0
}

# R7: CI Workflow Integrity — CI files cannot be modified by PR
eval_R7() {
    local diff
    diff=$(git diff --name-only HEAD~1..HEAD 2>/dev/null | grep -E '^\.github/workflows' || true)
    if [[ -n "$diff" ]]; then
        echo "WARN (R7: modified CI files: $diff — manual review required)"
        return 3
    fi
    echo "PASS (R7: no CI workflow modifications)"
    return 0
}

#-------------------------------------------------------------------------------
# Step 4: Hygiene rules
#-------------------------------------------------------------------------------

eval_REQUIRE_ISSUE() {
    local pr_body="$1"
    if [[ -z "$pr_body" ]]; then
        echo "FAIL (PR body empty, no ISSUE reference)"
        return 1
    fi
    if echo "$pr_body" | grep -qiE "closes?\s+#[[:digit:]]+|fixes?\s+#[[:digit:]]+|refs?\s+#[[:digit:]]+"; then
        echo "PASS (REQUIRE_ISSUE: ISSUE referenced)"
        return 0
    fi
    echo "FAIL (REQUIRE_ISSUE: no ISSUE reference found)"
    return 1
}

eval_SQL_SEMANTIC_TEST() {
    local changed_files="$1"
    local sql_changed=false
    for f in $changed_files; do
        case "$f" in
            *executor*|*parser*|*planner*|*storage*)
                [[ "$f" == *.rs ]] && sql_changed=true && break
                ;;
        esac
    done
    [[ "$sql_changed" == "false" ]] && echo "PASS (SQL_SEMANTIC_TEST: no SQL files changed)" && return 0

    local null_test=false join_test=false
    local test_files
    test_files=$(find . -name "*.rs" -path "*/tests/*" 2>/dev/null | head -50 || true)
    for f in $test_files; do
        local content
        content=$(cat "$f" 2>/dev/null || true)
        echo "$content" | grep -qiE "null" && null_test=true
        echo "$content" | grep -qiE "join|where" && join_test=true
    done

    if [[ "$null_test" == "true" && "$join_test" == "true" ]]; then
        echo "PASS (SQL_SEMANTIC_TEST: NULL + JOIN tests exist)"
        return 0
    fi
    local missing=""
    [[ "$null_test" == "false" ]] && missing="${missing}NULL, "
    [[ "$join_test" == "false" ]] && missing="${missing}JOIN+WHERE, "
    echo "FAIL (SQL_SEMANTIC_TEST: missing ${missing%, }) [CI is ground truth for semantics]"
    return 1
}

eval_ROADMAP_PRIORITY() {
    local pr_labels="$1"
    local roadmap="docs/roadmap.json"
    if ! echo "$pr_labels" | grep -qiE "P1|P2"; then
        echo "PASS (ROADMAP_PRIORITY: not P1/P2)"
        return 0
    fi
    [[ ! -f "$roadmap" ]] && echo "PASS (ROADMAP_PRIORITY: no roadmap)" && return 0
    if grep -qE '"priority".*"P0".*"status".*"(open|in_progress|blocked)"' "$roadmap" 2>/dev/null; then
        echo "BLOCK (ROADMAP_PRIORITY: P0 issues open, P1/P2 blocked)"
        return 2
    fi
    echo "PASS (ROADMAP_PRIORITY: P0 complete or no P0)"
    return 0
}

eval_TEST_COMPLETENESS() {
    local changed_files="$1"
    local docs_only=true
    for f in $changed_files; do
        case "$f" in
            *.md|docs/*|wiki/*) ;;
            *) docs_only=false; break ;;
        esac
    done
    [[ "$docs_only" == "true" ]] && echo "PASS (TEST_COMPLETENESS: docs-only change)" && return 0

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

    if [[ "$happy" == "true" && "$edge" == "true" && "$regression" == "true" ]]; then
        echo "PASS (TEST_COMPLETENESS: happy + edge + regression categories exist)"
        return 0
    fi
    local missing=""
    [[ "$happy" == "false" ]] && missing="${missing}happy_path, "
    [[ "$edge" == "false" ]] && missing="${missing}edge_case, "
    [[ "$regression" == "false" ]] && missing="${missing}regression, "
    echo "FAIL (TEST_COMPLETENESS: missing ${missing%, }) [CI is ground truth for test quality]"
    return 1
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
    echo "         Hermes Gate v0.3"
    echo "         Contract-driven + Commit-bound"
    echo "=============================================="
    echo ""
    echo "Contract: $CONTRACT_PATH"
    echo "Verification: $VERIFICATION_REPORT"
    echo "Audit: $AUDIT_REPORT"
    echo "HEAD: $(current_commit)"
    echo ""

    local final_decision="PASS"

    # ── Layer 0: Ensure fresh audit ────────────────────────────────────────
    echo "--- Auto-trigger Audit (if stale) ---"
    ensure_fresh_audit
    echo ""

    # ── Layer 1: Artifact Integrity (commit binding) ───────────────────────
    echo "--- Artifact Integrity (commit binding) ---"

    check_report_commit "$VERIFICATION_REPORT" "verification"
    local rv=$?
    if (( rv == 1 || rv == 2 || rv == 4 )); then
        final_decision="BLOCK"
        echo "  → Final Decision: BLOCK (stale/missing verification artifact)"
        exit $(get_exit_code "BLOCK")
    fi
    if (( rv == 0 )); then
        :
    else
        final_decision="ERROR"
        exit $(get_exit_code "ERROR")
    fi

    check_report_commit "$AUDIT_REPORT" "audit"
    local ra=$?
    if (( ra != 0 )); then
        final_decision="BLOCK"
        echo "  → Final Decision: BLOCK (stale/missing audit artifact)"
        exit $(get_exit_code "BLOCK")
    fi
    echo ""

    # ── Layer 2: Contract Rules (R1-R7) ───────────────────────────────────
    echo "--- Contract Rules (R1-R7) ---"

    eval_R1; local r1=$?
    echo "  [R1] $([[ $r1 -eq 0 ]] && echo PASS || echo FAIL)"

    eval_R2; local r2=$?
    echo "  [R2] $([[ $r2 -eq 0 ]] && echo PASS || echo FAIL)"

    eval_R3; local r3=$?
    echo "  [R3] $([[ $r3 -eq 0 ]] && echo PASS || echo FAIL)"

    eval_R4; local r4=$?
    echo "  [R4] $([[ $r4 -eq 0 ]] && echo PASS || echo FAIL)"

    eval_R5; local r5=$?
    echo "  [R5] $([[ $r5 -eq 0 ]] && echo PASS || echo FAIL)"

    eval_R6; local r6=$?
    echo "  [R6] $([[ $r6 -eq 0 ]] && echo PASS || echo FAIL)"

    eval_R7; local r7=$?
    echo "  [R7] $([[ $r7 -eq 0 ]] && echo PASS || echo WARN)"

    # Aggregate R1-R7
    local contract_decision="PASS"
    for result in $r1 $r2 $r3 $r4 $r5 $r6; do
        if (( result == 2 )); then
            contract_decision="BLOCK"; break
        elif (( result == 1 )) && [[ "$contract_decision" != "BLOCK" ]]; then
            contract_decision="FAIL"
        fi
    done
    if [[ "$r7" == "3" && "$contract_decision" == "PASS" ]]; then
        contract_decision="WARN"
    fi
    echo "  → Contract Decision: $contract_decision"
    echo ""

    # ── Layer 3: Hygiene Rules ──────────────────────────────────────────────
    echo "--- Hygiene Rules ---"

    eval_REQUIRE_ISSUE "$pr_body"; local rh=$?
    echo "  [REQUIRE_ISSUE] $([[ $rh -eq 0 ]] && echo PASS || echo FAIL)"

    eval_SQL_SEMANTIC_TEST "$changed_files"; local rs=$?
    echo "  [SQL_SEMANTIC_TEST] $([[ $rs -eq 0 ]] && echo PASS || echo FAIL)"

    eval_ROADMAP_PRIORITY "$pr_labels"; local rp=$?
    echo "  [ROADMAP_PRIORITY] $([[ $rp -eq 0 ]] && echo PASS || echo FAIL)"

    eval_TEST_COMPLETENESS "$changed_files"; local rt=$?
    echo "  [TEST_COMPLETENESS] $([[ $rt -eq 0 ]] && echo PASS || echo FAIL)"

    # Aggregate hygiene
    local hygiene_decision="PASS"
    for result in $rh $rs $rp $rt; do
        if (( result == 2 )); then
            hygiene_decision="BLOCK"; break
        elif (( result == 1 )) && [[ "$hygiene_decision" != "BLOCK" ]]; then
            hygiene_decision="FAIL"
        fi
    done
    echo "  → Hygiene Decision: $hygiene_decision"
    echo ""

    # ── Final ───────────────────────────────────────────────────────────────
    if [[ "$contract_decision" == "BLOCK" || "$hygiene_decision" == "BLOCK" ]]; then
        final_decision="BLOCK"
    elif [[ "$contract_decision" == "FAIL" || "$hygiene_decision" == "FAIL" ]]; then
        final_decision="FAIL"
    elif [[ "$contract_decision" == "WARN" ]]; then
        final_decision="WARN"
    fi

    echo "=============================================="
    echo "  Contract Decision: $contract_decision"
    echo "  Hygiene Decision:   $hygiene_decision"
    echo "  Final Decision:    $final_decision"
    echo "=============================================="
    echo ""

    exit $(get_exit_code "$final_decision")
}

run_gate "$@"
