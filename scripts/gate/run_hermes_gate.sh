#!/usr/bin/env bash
#===============================================================================
# Hermes Gate CI Runner
# Exit codes: 0=PASS, 1=FAIL, 2=BLOCK, 3=ERROR
#===============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GATE_ENGINE="${SCRIPT_DIR}/../hermes/engine/hermes_gate.sh"

# Check engine exists
if [[ ! -x "$GATE_ENGINE" ]]; then
    echo "ERROR: hermes_gate.sh not found or not executable at $GATE_ENGINE" >&2
    exit 3
fi

# Collect CI context
PR_BODY="${CI_PR_BODY:-}"
PR_LABELS="${CI_PR_LABELS:-}"
ROADMAP="${CI_ROADMAP:-docs/roadmap.json}"

# Get changed files (from git diff for PR)
if [[ -n "${CI_COMMIT_SHA:-}" && -n "${CI_BASE_SHA:-}" ]]; then
    CHANGED_FILES=$(git diff --name-only "${CI_BASE_SHA}".."${CI_PR_SHA:-${CI_COMMIT_SHA}}" 2>/dev/null | tr '\n' ' ' || echo "")
elif [[ -n "${GITHUB_BASE_REF:-}" ]]; then
    CHANGED_FILES=$(git diff --name-only "origin/${GITHUB_BASE_REF}..HEAD" 2>/dev/null | tr '\n' ' ' || echo "")
else
    # Local / manual run — use last commit
    CHANGED_FILES=$(git diff --name-only HEAD~1..HEAD 2>/dev/null | tr '\n' ' ' || echo "")
fi

# Load roadmap content if exists
ROADMAP_CONTENT=""
if [[ -f "$ROADMAP" ]]; then
    ROADMAP_CONTENT=$(cat "$ROADMAP" 2>/dev/null || echo "")
fi

# Execute gate
exec "$GATE_ENGINE" \
    --pr-body "$PR_BODY" \
    --pr-labels "$PR_LABELS" \
    --changed-files "$CHANGED_FILES" \
    --roadmap "$ROADMAP_CONTENT"
