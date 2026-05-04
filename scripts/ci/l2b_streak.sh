#!/bin/bash
# =============================================================================
# l2b_streak.sh - L2b 延迟阻塞机制 (V3 Enhancement #5)
# =============================================================================
# 规则: L2b failure → 第1/2次 warning, 第3次（连续）→ block PR
# 实现: SQLite 本地存储 failure streak

set -euo pipefail

DB_PATH="${1:-ci_artifacts/.l2b_streak.db}"
ACTION="${2:-check}"  # check | record | reset
CRATE="${3:-workspace}"

mkdir -p "$(dirname "$DB_PATH")"

init_db() {
    if [ ! -f "$DB_PATH" ]; then
        if command -v sqlite3 &>/dev/null; then
            sqlite3 "$DB_PATH" "CREATE TABLE l2b_streak (
                crate TEXT PRIMARY KEY,
                failure_count INTEGER DEFAULT 0,
                last_failure TEXT
            );"
        else
            touch "$DB_PATH"
        fi
    fi
}

do_check() {
    init_db
    local streak=0

    if [ -f "$DB_PATH" ] && command -v sqlite3 &>/dev/null && file "$DB_PATH" 2>/dev/null | grep -q SQLite; then
        streak=$(sqlite3 "$DB_PATH" "SELECT failure_count FROM l2b_streak WHERE crate='$CRATE';" 2>/dev/null || echo "0")
    elif [ -f "$DB_PATH" ]; then
        streak=$(grep "^$CRATE:" "$DB_PATH" 2>/dev/null | cut -d: -f2 || echo "0")
    fi

    # Handle empty/non-numeric streak
    streak=${streak:-0}
    streak=${streak//[^0-9]/}
    [ -z "$streak" ] && streak=0

    echo "[l2b] crate=$CRATE streak=$streak"

    if [ "$streak" -ge 3 ]; then
        echo "[l2b] BLOCK: continuous $streak failures"
        return 1
    elif [ "$streak" -ge 1 ]; then
        echo "[l2b] WARN: $streak/3 failures (blocks at 3)"
        return 0
    fi

    echo "[l2b] OK"
    return 0
}

do_record() {
    init_db

    if [ -f "$DB_PATH" ] && command -v sqlite3 &>/dev/null && file "$DB_PATH" 2>/dev/null | grep -q SQLite; then
        sqlite3 "$DB_PATH" "
            INSERT INTO l2b_streak (crate, failure_count, last_failure)
            VALUES ('$CRATE', 1, datetime('now'))
            ON CONFLICT(crate) DO UPDATE SET
                failure_count = failure_count + 1,
                last_failure = datetime('now');
        " 2>/dev/null
    else
        local current=$(grep "^$CRATE:" "$DB_PATH" 2>/dev/null | cut -d: -f2 || echo "0")
        current=${current:-0}
        current=${current//[^0-9]/}
        [ -z "$current" ] && current=0
        local new=$((current + 1))
        grep -v "^$CRATE:" "$DB_PATH" > "${DB_PATH}.tmp" 2>/dev/null || true
        echo "$CRATE:$new" >> "${DB_PATH}.tmp"
        mv "${DB_PATH}.tmp" "$DB_PATH"
    fi

    echo "[l2b] Recorded failure for $CRATE"
}

do_reset() {
    init_db

    if [ -f "$DB_PATH" ] && command -v sqlite3 &>/dev/null && file "$DB_PATH" 2>/dev/null | grep -q SQLite; then
        sqlite3 "$DB_PATH" "DELETE FROM l2b_streak WHERE crate='$CRATE';" 2>/dev/null
    else
        grep -v "^$CRATE:" "$DB_PATH" > "${DB_PATH}.tmp" 2>/dev/null || true
        mv "${DB_PATH}.tmp" "$DB_PATH"
    fi

    echo "[l2b] Reset streak for $CRATE"
}

case "$ACTION" in
    check)   do_check ;;
    record)  do_record ;;
    reset)   do_reset ;;
    *)
        echo "Usage: $0 [check|record|reset] [crate]"
        exit 1 ;;
esac
