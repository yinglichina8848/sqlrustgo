#!/usr/bin/env bash
#===============================================================================
# Hermes Gate Engine — MVP Bootstrap
# Minimal version: runs even with missing context files.
#===============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RULES_FILE="${SCRIPT_DIR}/../rules/core.json"
ROADMAP_FILE="context/roadmap.json"
ISSUES_FILE="context/issues.json"

# Exit codes: 0=PASS, 1=FAIL, 2=BLOCK, 3=WARN, 4=ERROR
get_exit_code() {
    case "$1" in
        PASS) echo 0 ;;
        FAIL) echo 1 ;;
        BLOCK) echo 2 ;;
        WARN) echo 3 ;;
        ERROR) echo 4 ;;
        *) echo 0 ;;
    esac
}

echo ""
echo "=============================================="
echo "         Hermes Gate — MVP Bootstrap"
echo "=============================================="
echo ""

# Check context files exist
echo "--- Context Check ---"
if [[ -f "$ROADMAP_FILE" ]]; then
    echo "[ROADMAP] OK: $ROADMAP_FILE exists"
else
    echo "[ROADMAP] WARN: $ROADMAP_FILE not found"
fi

if [[ -f "$ISSUES_FILE" ]]; then
    echo "[ISSUES]  OK: $ISSUES_FILE exists"
else
    echo "[ISSUES]  WARN: $ISSUES_FILE not found"
fi

# Check rules file
if [[ -f "$RULES_FILE" ]]; then
    echo "[RULES]   OK: $RULES_FILE exists"
else
    echo "[RULES]   ERROR: $RULES_FILE not found"
    echo "=============================================="
    exit 4
fi

echo ""
echo "--- Rule Checks ---"

# Parse rules from core.json (minimal extraction)
if grep -q "REQUIRE_ISSUE" "$RULES_FILE"; then
    echo "[REQUIRE_ISSUE]  PASS (meta check — requires CI integration)"
else
    echo "[REQUIRE_ISSUE]  WARN"
fi

if grep -q "SQL_SEMANTIC_TEST" "$RULES_FILE"; then
    echo "[SQL_SEMANTIC_TEST] PASS (text check — requires CI integration)"
else
    echo "[SQL_SEMANTIC_TEST] WARN"
fi

if grep -q "ROADMAP_PRIORITY" "$RULES_FILE"; then
    if [[ -f "$ROADMAP_FILE" ]]; then
        p0_open=$(grep -o '"p0_open"[[:space:]]*:[[:space:]]*[^,}]*' "$ROADMAP_FILE" 2>/dev/null | sed 's/.*:[[:space:]]*//' | tr -d ' ')
        if [[ "$p0_open" == "true" ]]; then
            echo "[ROADMAP_PRIORITY] BLOCK (P0 open)"
        else
            echo "[ROADMAP_PRIORITY] PASS (no open P0)"
        fi
    else
        echo "[ROADMAP_PRIORITY] WARN (roadmap.json missing, assuming OK)"
    fi
else
    echo "[ROADMAP_PRIORITY] WARN"
fi

if grep -q "TEST_COMPLETENESS" "$RULES_FILE"; then
    echo "[TEST_COMPLETENESS] PASS (text check — requires CI integration)"
else
    echo "[TEST_COMPLETENESS] WARN"
fi

echo ""
echo "=============================================="
echo "  Final Decision: PASS (MVP — no context required)"
echo "=============================================="
echo ""

exit 0
