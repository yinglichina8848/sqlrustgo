#!/usr/bin/env bash
# v2.9.0 Beta Gate — 进入 Beta 阶段必须通过
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PASS=0; TOTAL=0; BLOCKERS=0

check() {
    local name="$1" cmd="$2"
    TOTAL=$((TOTAL+1))
    echo -n "[beta] $name ... "
    if eval "$cmd" >/dev/null 2>&1; then echo "PASS"; PASS=$((PASS+1))
    else echo "FAIL"; BLOCKERS=$((BLOCKERS+1)); fi
}

echo "=== v2.9.0 Beta Gate ==="
echo ""

# === Test & Code Quality (inherited from Alpha) ===
check "R4: cargo test --all-features" "cargo test --all-features --quiet"
check "R4: Integration tests 28 files" "bash scripts/test/run_integration.sh --quick"
check "R7: clippy zero warnings" "cargo clippy --all-features -- -D warnings --quiet"
check "R7: cargo fmt" "cargo fmt --check --quiet"

# === Feature Gates ===
check "A1: SQL Corpus >=85%" "cargo test -p sqlrustgo-sql-corpus --quiet"

# === Documentation (inherited from Alpha) ===
check "Docs: VERSION_PLAN.md" "test -f docs/releases/v2.9.0/VERSION_PLAN.md"
check "Docs: RELEASE_NOTES.md" "test -f docs/releases/v2.9.0/RELEASE_NOTES.md"
check "Docs: CHANGELOG.md" "test -f docs/releases/v2.9.0/CHANGELOG.md"
check "Docs: FEATURE_MATRIX.md" "test -f docs/releases/v2.9.0/FEATURE_MATRIX.md"
check "Docs: INTEGRATION_STATUS.md" "test -f docs/releases/v2.9.0/INTEGRATION_STATUS.md"
check "Docs: TEST_PLAN.md" "test -f docs/releases/v2.9.0/TEST_PLAN.md"
check "Docs: RELEASE_GATE_CHECKLIST.md" "test -f docs/releases/v2.9.0/RELEASE_GATE_CHECKLIST.md"
check "Docs: PERFORMANCE_TARGETS.md" "test -f docs/releases/v2.9.0/PERFORMANCE_TARGETS.md"

# === Verification Binding ===
check "R0: commit binding" 'python3 -c "
import json,subprocess
v=json.load(open(\"verification_report.json\"))
h=subprocess.check_output([\"git\",\"rev-parse\",\"HEAD\"]).decode().strip()
assert v[\"commit\"]==h, \"Commit mismatch\"
print(f\"HEAD={h[:12]} verified\")
"'

# === Beta-specific: Coverage ===
check "B1: total coverage >=75%" "python3 -c 'import json; d=json.load(open(\"artifacts/coverage/total.json\"));lines=d.get(\"data\",[{}])[0].get(\"totals\",{}).get(\"lines\",{});assert lines.get(\"percent\",0)>=75'"
check "B2: executor coverage >=60%" "python3 -c 'import json; d=json.load(open(\"artifacts/coverage/executor.json\"));lines=d.get(\"data\",[{}])[0].get(\"totals\",{}).get(\"lines\",{});assert lines.get(\"percent\",0)>=60'"

# === Beta-specific: Formal Verification ===
check "B3: formal proofs all verified" "bash scripts/verify/run_all_proofs.sh"
check "B4: proof registry integrity" "python3 scripts/verify_proof_registry.py"

# === Beta-specific: Performance baselines ===
check "B5: test count >=3597" "python3 -c 'import subprocess; r=subprocess.run([\"grep\",\"-r\",\"fn test_\",\"--include=*.rs\",\"crates/\",\"src/\"],capture_output=True); count=len(r.stdout.decode().splitlines()); assert count>=3597'"

echo ""
echo "=== Beta Gate Results: PASS=$PASS / $TOTAL, BLOCKERS=$BLOCKERS ==="
echo "(Coverage check moved to CI workflow_dispatch: Gitea Actions -> Run workflow -> profile=quick)"
[ "$BLOCKERS" -eq 0 ] || { echo "BLOCKED"; exit 2; }
echo "PASS — can promote to beta/v2.9.0"
