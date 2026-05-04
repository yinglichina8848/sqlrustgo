#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${BLUE}[verify]${NC} $1"; }
pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; }

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

FAILED=0

log "SQLRustGo Local Verification Gate"
log "================================="
echo ""

log "1. Quick unit tests"
if cargo test --lib --all-features --quiet 2>/dev/null; then
    pass "Unit tests passed"
else
    fail "Unit tests failed"
    FAILED=1
fi
echo ""

log "2. Clippy lint check"
if cargo clippy --all-features -- -D warnings --quiet 2>/dev/null; then
    pass "Clippy passed"
else
    fail "Clippy found issues"
    FAILED=1
fi
echo ""

log "3. Format check"
if cargo fmt --all -- --check --quiet 2>/dev/null; then
    pass "Format OK"
else
    fail "Format issues - run: cargo fmt --all"
    FAILED=1
fi
echo ""

log "4. Core executor smoke"
if cargo test --test engine_test --all-features --quiet 2>/dev/null; then
    pass "Engine tests passed"
else
    fail "Engine tests failed"
    FAILED=1
fi
echo ""

log "5. Planner basic"
if cargo test -p sqlrustgo-planner --lib --all-features --quiet 2>/dev/null; then
    pass "Planner tests passed"
else
    fail "Planner tests failed"
    FAILED=1
fi
echo ""

log "6. Optimizer basic"
if cargo test -p sqlrustgo-optimizer --lib --all-features --quiet 2>/dev/null; then
    pass "Optimizer tests passed"
else
    fail "Optimizer tests failed"
    FAILED=1
fi
echo ""

log "7. Storage smoke"
if cargo test -p sqlrustgo-storage --lib --all-features --quiet 2>/dev/null; then
    pass "Storage tests passed"
else
    fail "Storage tests failed"
    FAILED=1
fi
echo ""

log "================================="
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All checks passed${NC}"
    exit 0
else
    echo -e "${RED}Some checks failed${NC}"
    exit 1
fi
