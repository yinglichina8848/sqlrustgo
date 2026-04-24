#!/usr/bin/env bash
#===============================================================================
# Hermes Gate CI Runner
# Exit codes: 0=PASS, 1=FAIL, 2=BLOCK, 3=ERROR
#
# Usage:
#   ./scripts/gate/run_hermes_gate.sh \
#       [--pr-body "..."] \
#       [--pr-labels "P0,P1"] \
#       [--changed-files "file1.rs file2.rs"] \
#       [--verification-report docs/versions/v2.8.0/verification_report.json] \
#       [--audit-report docs/versions/v2.8.0/audit_report.json]
#===============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GATE_ENGINE="${SCRIPT_DIR}/../hermes/engine/hermes_gate.sh"

# Check engine exists
if [[ ! -x "$GATE_ENGINE" ]]; then
    echo "ERROR: hermes_gate.sh not found or not executable" >&2
    exit 3
fi

# Input defaults
PR_BODY="${CI_PR_BODY:-}"
PR_LABELS="${CI_PR_LABELS:-}"
VERIFICATION_REPORT="${CI_VERIFICATION_REPORT:-docs/versions/v2.8.0/verification_report.json}"
AUDIT_REPORT="${CI_AUDIT_REPORT:-docs/versions/v2.8.0/audit_report.json}"

# Get changed files
if [[ -n "${CI_BASE_SHA:-}" && -n "${CI_PR_SHA:-}" ]]; then
    CHANGED_FILES=$(git diff --name-only "$CI_BASE_SHA".."$CI_PR_SHA" 2>/dev/null | tr '\n' ' ' || echo "")
elif [[ -n "${GITHUB_BASE_REF:-}" ]]; then
    CHANGED_FILES=$(git diff --name-only "origin/${GITHUB_BASE_REF}..HEAD" 2>/dev/null | tr '\n' ' ' || echo "")
else
    CHANGED_FILES=$(git diff --name-only HEAD~1..HEAD 2>/dev/null | tr '\n' ' ' || echo "")
fi

# Execute gate
exec "$GATE_ENGINE" \
    "$PR_BODY" \
    "$PR_LABELS" \
    "$CHANGED_FILES" \
    "$VERIFICATION_REPORT" \
    "$AUDIT_REPORT"
