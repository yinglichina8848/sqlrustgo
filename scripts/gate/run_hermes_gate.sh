#!/usr/bin/env bash
#===============================================================================
# Hermes Gate CI Runner v0.3
# Contract-driven + Commit-bound + Auto-trigger Audit
#
# Exit codes: 0=PASS, 1=FAIL, 2=BLOCK, 3=WARN, 4=ERROR
#
# Environment variables:
#   CI_PR_BODY         PR body text
#   CI_PR_LABELS       Comma-separated labels
#   CI_BASE_SHA        Base commit SHA (merge base)
#   CI_PR_SHA          Head commit SHA
#   CONTRACT_PATH      Path to contract JSON (default: contract/v2.8.0.json)
#   VERIFICATION_REPORT Path to verification report (default: docs/versions/v2.8.0/verification_report.json)
#   AUDIT_REPORT       Path to audit report (default: docs/versions/v2.8.0/audit_report.json)
#===============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GATE_ENGINE="${SCRIPT_DIR}/../hermes/engine/hermes_gate.sh"

if [[ ! -x "$GATE_ENGINE" ]]; then
    echo "ERROR: hermes_gate.sh not found or not executable" >&2
    exit 4
fi

# Inputs
PR_BODY="${CI_PR_BODY:-}"
PR_LABELS="${CI_PR_LABELS:-}"
CONTRACT_PATH="${CONTRACT_PATH:-contract/v2.8.0.json}"
VERIFICATION_REPORT="${VERIFICATION_REPORT:-docs/versions/v2.8.0/verification_report.json}"
AUDIT_REPORT="${AUDIT_REPORT:-docs/versions/v2.8.0/audit_report.json}"

# Get changed files
if [[ -n "${CI_BASE_SHA:-}" && -n "${CI_PR_SHA:-}" ]]; then
    CHANGED_FILES=$(git diff --name-only "$CI_BASE_SHA".."$CI_PR_SHA" 2>/dev/null | tr '\n' ' ' || echo "")
elif [[ -n "${GITHUB_BASE_REF:-}" ]]; then
    CHANGED_FILES=$(git diff --name-only "origin/${GITHUB_BASE_REF}..HEAD" 2>/dev/null | tr '\n' ' ' || echo "")
else
    CHANGED_FILES=$(git diff --name-only HEAD~1..HEAD 2>/dev/null | tr '\n' ' ' || echo "")
fi

export CONTRACT_PATH VERIFICATION_REPORT AUDIT_REPORT

exec "$GATE_ENGINE" \
    "$PR_BODY" \
    "$PR_LABELS" \
    "$CHANGED_FILES"
