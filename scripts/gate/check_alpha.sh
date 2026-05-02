#!/usr/bin/env bash
# v2.9.0 Alpha Gate — 进入 Alpha 阶段必须通过
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PASS=0; TOTAL=0; BLOCKERS=0

check() {
    local name="$1" cmd="$2"
    TOTAL=$((TOTAL+1))
    echo -n "[alpha] $name ... "
    if eval "$cmd" >/dev/null 2>&1; then echo "PASS"; PASS=$((PASS+1))
    else echo "FAIL"; BLOCKERS=$((BLOCKERS+1)); fi
}

echo "=== v2.9.0 Alpha Gate ==="
echo ""

# Test & integration
check "R4: cargo test --all-features" "cargo test --all-features --quiet"
check "R4: Integration tests 28 files" "bash scripts/test/run_integration.sh --quick"

# Code quality
check "R7: clippy zero warnings" "cargo clippy --all-features -- -D warnings --quiet"
check "R7: cargo fmt" "cargo fmt --check --quiet"

# Feature gates
check "A1: SQL Corpus >=85%" "cargo test -p sqlrustgo-sql-corpus --quiet"

# Documentation
check "A4: VERSION_PLAN.md" "test -f docs/releases/v2.9.0/VERSION_PLAN.md"
check "A4: RELEASE_NOTES.md" "test -f docs/releases/v2.9.0/RELEASE_NOTES.md"
check "A4: CHANGELOG.md" "test -f docs/releases/v2.9.0/CHANGELOG.md"
check "A4: FEATURE_MATRIX.md" "test -f docs/releases/v2.9.0/FEATURE_MATRIX.md"
check "A4: INTEGRATION_STATUS.md" "test -f docs/releases/v2.9.0/INTEGRATION_STATUS.md"
check "A4: TEST_PLAN.md" "test -f docs/releases/v2.9.0/TEST_PLAN.md"
check "A4: RELEASE_GATE_CHECKLIST.md" "test -f docs/releases/v2.9.0/RELEASE_GATE_CHECKLIST.md"
check "A4: PERFORMANCE_TARGETS.md" "test -f docs/releases/v2.9.0/PERFORMANCE_TARGETS.md"

# Verification binding
check "R0: commit binding" 'python3 -c "
import json,subprocess
v=json.load(open(\"verification_report.json\"))
h=subprocess.check_output([\"git\",\"rev-parse\",\"HEAD\"]).decode().strip()
assert v[\"commit\"]==h, \"Commit mismatch\"
print(f\"HEAD={h[:12]} verified\")
"'

echo ""
echo "=== Alpha Gate Results: PASS=$PASS / $TOTAL, BLOCKERS=$BLOCKERS ==="
echo "(Coverage check moved to CI workflow_dispatch: Gitea Actions -> Run workflow -> profile=quick)"
[ "$BLOCKERS" -eq 0 ] || { echo "BLOCKED"; exit 2; }
echo "PASS — can promote to alpha/v2.9.0"
