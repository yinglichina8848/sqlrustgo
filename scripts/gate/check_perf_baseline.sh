#!/usr/bin/env bash
# ============================================================
# R9: Performance Baseline Check (delegates to check_regression.sh)
#
# This is a thin wrapper for backward compatibility.
# The actual regression check logic lives in check_regression.sh.
# ============================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== R9: Performance Baseline Check ==="
echo "Date: $(date)"
echo ""

# Delegate to the real regression check
bash "$SCRIPT_DIR/check_regression.sh" "${1:-}"

exit $?
