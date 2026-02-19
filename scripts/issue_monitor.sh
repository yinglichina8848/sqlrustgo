#!/bin/bash

PROJECT_ROOT="/Users/liying/workspace/yinglichina/sqlrustgo"
REPO="yinglichina8848/sqlrustgo"
STATUS_FILE="$PROJECT_ROOT/.issue_status.json"
LOG_FILE="$PROJECT_ROOT/.issue_monitor.log"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

log "=== ISSUE Monitor Started ==="

cd "$PROJECT_ROOT" || exit 1

OPEN_ISSUES=$(gh issue list --repo "$REPO" --state open --limit 100 --json number,title,labels,createdAt,updatedAt 2>/dev/null)

if [ -z "$OPEN_ISSUES" ]; then
    log "ERROR: Failed to fetch issues"
    exit 1
fi

ISSUE_COUNT=$(echo "$OPEN_ISSUES" | jq 'length')
log "Found $ISSUE_COUNT open issues"

BLOCKER_COUNT=$(echo "$OPEN_ISSUES" | jq '[.[] | select(.labels[]?.name == "blocker" or .labels[]?.name == "Blocker")] | length')
HIGH_PRIORITY_COUNT=$(echo "$OPEN_ISSUES" | jq '[.[] | select(.labels[]?.name == "high-priority" or .labels[]?.name == "High Priority")] | length')

echo "$OPEN_ISSUES" > "$STATUS_FILE"

log "Summary: $ISSUE_COUNT open, $BLOCKER_COUNT blockers, $HIGH_PRIORITY_COUNT high priority"

if [ "$BLOCKER_COUNT" -gt 0 ]; then
    log "⚠️  WARNING: $BLOCKER_COUNT blocker issues found!"
    echo "$OPEN_ISSUES" | jq -r '.[] | select(.labels[]?.name == "blocker" or .labels[]?.name == "Blocker") | "  - #\(.number): \(.title)"' >> "$LOG_FILE"
fi

if [ "$HIGH_PRIORITY_COUNT" -gt 0 ]; then
    log "⚠️  ATTENTION: $HIGH_PRIORITY_COUNT high priority issues!"
    echo "$OPEN_ISSUES" | jq -r '.[] | select(.labels[]?.name == "high-priority" or .labels[]?.name == "High Priority") | "  - #\(.number): \(.title)"' >> "$LOG_FILE"
fi

log "=== ISSUE Monitor Completed ==="
