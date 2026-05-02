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

check "R4: cargo test --all-features" "cargo test --all-features --quiet"
check "R4: Integration tests 28 files" "bash scripts/test/run_integration.sh --quick"
check "R7: clippy zero warnings" "cargo clippy --all-features -- -D warnings --quiet"
check "R7: cargo fmt" "cargo fmt --check --quiet"
check "A1: SQL Corpus >=85%" "cargo test -p sqlrustgo-sql-corpus --quiet"
check "A3: coverage >=50%" "cargo tarpaulin --ignore-tests --out Json 2>&1 | grep -q '\"coverage\":[5-9][0-9]'"
check "R0: commit binding" 'python3 -c "
import json,subprocess
v=json.load(open(\"verification_report.json\"))
h=subprocess.check_output([\"git\",\"rev-parse\",\"HEAD\"]).decode().strip()
assert v[\"commit\"]==h, \"Commit mismatch\"
print(f\"HEAD={h[:12]} verified\")
"'

echo ""
echo "Alpha Gate: $PASS/$TOTAL passed ($BLOCKERS blockers)"
[ "$BLOCKERS" -eq 0 ] || { echo "BLOCKED"; exit 2; }
echo "PASS — can promote to alpha/v2.9.0"
