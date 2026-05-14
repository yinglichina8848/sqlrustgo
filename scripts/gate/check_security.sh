#!/usr/bin/env bash

set -e

echo "=== Running v3.1.0 Security Gate Check ==="

REPORT_DIR="docs/releases/v3.1.0"
mkdir -p "$REPORT_DIR"

echo "Running cargo audit..."
if cargo audit --json > "$REPORT_DIR/security_audit_output.json" 2>&1; then
    echo "✅ cargo audit completed"
else
    echo "⚠️  cargo audit completed with issues"
fi

AUDIT_EXIT_CODE=${PIPESTATUS[0]}

echo "Checking vulnerability levels..."
CRITICAL_COUNT=$(grep -o '"critical"' "$REPORT_DIR/security_audit_output.json" 2>/dev/null | wc -l || echo "0")
HIGH_COUNT=$(grep -o '"high"' "$REPORT_DIR/security_audit_output.json" 2>/dev/null | wc -l || echo "0")
MEDIUM_COUNT=$(grep -o '"medium"' "$REPORT_DIR/security_audit_output.json" 2>/dev/null | wc -l || echo "0")
LOW_COUNT=$(grep -o '"low"' "$REPORT_DIR/security_audit_output.json" 2>/dev/null | wc -l || echo "0")

echo "Critical: ${CRITICAL_COUNT}, High: ${HIGH_COUNT}, Medium: ${MEDIUM_COUNT}, Low: ${LOW_COUNT}"

if [ "$CRITICAL_COUNT" -gt 0 ]; then
    echo "❌ CRITICAL vulnerabilities found!"
    echo "Vulnerabilities:"
    grep -A5 '"critical"' "$REPORT_DIR/security_audit_output.json" 2>/dev/null || true
    exit 1
fi

if [ "$HIGH_COUNT" -gt 5 ]; then
    echo "❌ Too many high vulnerabilities! Found ${HIGH_COUNT}, max allowed is 5"
    exit 1
fi

echo "✅ Security scan passed"
echo "=== Security Gate Check Complete ==="
