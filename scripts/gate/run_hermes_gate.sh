#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== Hermes Gate v0.3 ==="
echo "Date: $(date)"
echo "Project: SQLRustGo"
echo ""

cd "$PROJECT_ROOT"

VERIFICATION_PASS=false
AUDIT_PASS=false
PROOF_MATCH=false
EXIT_CODE=0

echo "Step 1: Check audit_report.json..."
if [ ! -f "audit_report.json" ]; then
    echo "audit_report.json not found, running self_audit.py..."
    python3 scripts/self_audit.py
fi

if [ -f "verification_report.json" ]; then
    BASELINE_VERIFIED=$(python3 -c "import json; v = json.load(open('verification_report.json')).get('baseline_verified', False); print('true' if v else 'false')" 2>/dev/null)
    if [ "$BASELINE_VERIFIED" = "true" ]; then
        VERIFICATION_PASS=true
        echo "verification: PASS (baseline_verified=$BASELINE_VERIFIED)"
    else
        echo "verification: FAIL (baseline_verified=$BASELINE_VERIFIED)"
        EXIT_CODE=2
    fi
else
    echo "verification: FAIL (file not found)"
    EXIT_CODE=2
fi

echo "Step 2: Check audit_report.json..."
if [ -f "audit_report.json" ]; then
    STATUS=$(python3 -c "import json; print(json.load(open('audit_report.json')).get('status', 'WEAKENED'))" 2>/dev/null || echo "WEAKENED")
    PROOF_MATCH=$(python3 -c "import json; v = json.load(open('audit_report.json')).get('proof_match', False); print('true' if v else 'false')" 2>/dev/null)
    if [ "$STATUS" = "TRUSTED" ]; then
        AUDIT_PASS=true
        echo "audit: TRUSTED (proof_match=$PROOF_MATCH)"
    else
        echo "audit: WEAKENED (status=$STATUS)"
        EXIT_CODE=2
    fi
else
    echo "audit: FAIL (file not found)"
    EXIT_CODE=2
fi

echo ""
echo "Step 3: Proof vs Reality check..."
if [ "$PROOF_MATCH" != "true" ]; then
    echo "PROOF MISMATCH: verification and audit counts do not match"
    EXIT_CODE=2
else
    echo "proof_match: verified"
fi

echo ""
echo "=== Hermes Gate Result ==="
if [ $EXIT_CODE -eq 0 ]; then
    echo "PASS"
    echo "All checks verified:"
    echo "  - verification: PASS"
    echo "  - audit: TRUSTED"
    echo "  - proof_match: true"
else
    echo "BLOCK"
    echo "One or more checks failed:"
    [ "$VERIFICATION_PASS" = "false" ] && echo "  - verification: FAIL"
    [ "$AUDIT_PASS" = "false" ] && echo "  - audit: WEAKENED"
    [ "$PROOF_MATCH" != "true" ] && echo "  - proof_match: FAIL"
fi

echo ""
exit $EXIT_CODE