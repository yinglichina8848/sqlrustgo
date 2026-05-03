#!/usr/bin/env bash
# chaos_test.sh — 验证 CI gate 真的有约束力
#
# 原理: 人为引入 bug，验证 CI 是否能检测出来
# 如果检测不出来 → CI 是假的
#
# 用法:
#   ./chaos_test.sh --inject-deadlock    # 注释 would_create_cycle
#   ./chaos_test.sh --restore           # 恢复原状
#   ./chaos_test.sh --verify            # 验证 CI 检测能力

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DEADLOCK_FILE="$PROJECT_ROOT/crates/transaction/src/deadlock.rs"
BACKUP="$DEADLOCK_FILE.bak.chaos"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo "[chaos] $1"; }

inject_deadlock_bug() {
    log "Injecting deadlock bug: commenting out would_create_cycle..."
    cp "$DEADLOCK_FILE" "$BACKUP"
    # 让所有等待都通过（绕过 deadlock 检测）
    sed -i 's/if inner.would_create_cycle(tx_id, &holders)/if false \&\& inner.would_create_cycle(tx_id, \&holders)/' "$DEADLOCK_FILE"
    if grep -q "if false && inner.would_create_cycle" "$DEADLOCK_FILE"; then
        log "${GREEN}Bug injected successfully${NC}"
    else
        log "${RED}Failed to inject bug${NC}"
        exit 1
    fi
}

restore() {
    if [ -f "$BACKUP" ]; then
        log "Restoring original..."
        cp "$BACKUP" "$DEADLOCK_FILE"
        rm "$BACKUP"
        log "${GREEN}Restored${NC}"
    else
        log "${YELLOW}No backup found${NC}"
    fi
}

verify_ci_detects() {
    log "Running deadlock tests with bug injected..."
    cd "$PROJECT_ROOT"

    set +e
    output=$(cargo test -p sqlrustgo-transaction deadlock -- --nocapture 2>&1 || true)
    set -e

    if echo "$output" | grep -q "test result: FAILED\|failures:"; then
        log "${GREEN}PASS: CI detected the bug!${NC}"
        log "Concurrent deadlock test correctly failed"
        return 0
    else
        log "${RED}FAIL: CI did NOT detect the bug!${NC}"
        log "This means your correctness system is BROKEN"
        return 1
    fi
}

case "${1:-}" in
    --inject-deadlock) inject_deadlock_bug ;;
    --restore) restore ;;
    --verify) verify_ci_detects ;;
    *)
        echo "Usage: $0 [--inject-deadlock|--restore|--verify]"
        echo ""
        echo "Workflow:"
        echo "  1. ./chaos_test.sh --inject-deadlock   # inject bug"
        echo "  2. ./chaos_test.sh --verify            # CI should fail"
        echo "  3. ./chaos_test.sh --restore           # restore"
        ;;
esac
