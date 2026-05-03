#!/usr/bin/env bash
#===============================================================================
# SQLRustGo pre-receive hook
# 
# Deploy to: /var/lib/gitea/data/gitea-repositories/<owner>/<repo>.git/hooks/pre-receive
# 
# Policy: BLOCK push if fmt/clippy/quick-test fails
#         CI handles full test suite + coverage
#===============================================================================
set -euo pipefail

LOG_PREFIX="[pre-receive hook]"
WORKTREE="${GIT_WORK_TREE:-$(mktemp -d)}"
FAILED=0

log_info()  { echo "$LOG_PREFIX $*" >&2; }
log_pass()  { echo "$LOG_PREFIX [PASS] $*" >&2; }
log_fail()  { echo "$LOG_PREFIX [FAIL] $*" >&2; }

cleanup() { [ -d "$WORKTREE" ] && rm -rf "$WORKTREE"; }
trap cleanup EXIT

#-------------------------------------------------------------------------------
# Phase 1: Format check (fastest fail)
#-------------------------------------------------------------------------------
log_info "Checking cargo fmt..."
cd "$WORKTREE"
if ! cargo fmt --all -- --check 2>/dev/null; then
    log_fail "Code is not formatted. Run 'cargo fmt --all' before pushing."
    FAILED=1
else
    log_pass "Format OK"
fi

#-------------------------------------------------------------------------------
# Phase 2: Clippy (no release, fast)
#-------------------------------------------------------------------------------
log_info "Checking cargo clippy..."
if ! cargo clippy \
    --all-features \
    --all-targets \
    -- \
    -D warnings 2>/dev/null; then
    log_fail "Clippy found issues. Fix warnings before pushing."
    FAILED=1
else
    log_pass "Clippy OK"
fi

#-------------------------------------------------------------------------------
# Phase 3: Quick unit tests (library only, skip integration tests)
#-------------------------------------------------------------------------------
log_info "Running cargo test --lib..."
if ! cargo test --workspace --lib --quiet 2>/dev/null; then
    log_fail "Library tests failed. Run 'cargo test --workspace --lib' locally."
    FAILED=1
else
    log_pass "Unit tests OK"
fi

#-------------------------------------------------------------------------------
# Result
#-------------------------------------------------------------------------------
echo "" >&2
if [ $FAILED -eq 0 ]; then
    log_info "=== ALL CHECKS PASSED ==="
    exit 0
else
    log_info "=== CHECKS FAILED - PUSH BLOCKED ==="
    log_info "Tip: Run locally: bash gate/hermes_gate.sh"
    exit 1
fi
