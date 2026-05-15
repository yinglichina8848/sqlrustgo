#!/usr/bin/env bash
# ============================================================
# Hermes Gate - Pre-commit quality gate
# Fails if any check fails
# ============================================================
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPORT="${REPORT_DIR:-.}/gate_report.json"
FAILED=0

echo "=== Hermes Gate ==="
echo ""

# Check 1: Clippy (if Cargo.toml exists)
if [ -f Cargo.toml ]; then
    echo "[GATE] Running clippy..."
    if cargo clippy --all-features -- -D warnings 2>&1; then
        echo "[PASS] Clippy OK"
    else
        echo "[FAIL] Clippy found issues"
        FAILED=1
    fi
else
    echo "[SKIP] clippy (no Cargo.toml)"
fi

# Check 2: Format (if rustfmt available)
if command -v cargo-fmt &>/dev/null; then
    echo "[GATE] Checking format..."
    if cargo fmt --check --all 2>&1; then
        echo "[PASS] Format OK"
    else
        echo "[FAIL] Format issues found"
        FAILED=1
    fi
fi

# Check 3: Python syntax (if .py files exist)
PY_FILES=$(find . -name "*.py" -not -path "./.git/*" 2>/dev/null | head -5)
if [ -n "$PY_FILES" ]; then
    echo "[GATE] Checking Python syntax..."
    for f in $PY_FILES; do
        python3 -m py_compile "$f" 2>&1 || { echo "[FAIL] Syntax error in $f"; FAILED=1; }
    done
fi

# Check 4: Shell syntax (if .sh files exist)
SH_FILES=$(find . -name "*.sh" -not -path "./.git/*" 2>/dev/null | head -5)
if [ -n "$SH_FILES" ]; then
    echo "[GATE] Checking shell scripts..."
    for f in $SH_FILES; do
        bash -n "$f" 2>&1 || { echo "[FAIL] Syntax error in $f"; FAILED=1; }
    done
fi

# Generate report
mkdir -p "$(dirname "$REPORT")"
cat > "$REPORT" <<EOF
{
  "gate": "hermes_gate",
  "passed": $([ $FAILED -eq 0 ] && echo "true" || echo "false"),
  "timestamp": "$(date -Iseconds)"
}
EOF

echo ""
echo "=== Gate Result: $([ $FAILED -eq 0 ] && echo 'PASSED' || echo 'FAILED') ==="
exit $FAILED
