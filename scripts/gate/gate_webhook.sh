#!/usr/bin/env bash
# ============================================================
# Gate Webhook — sends notification on gate PASS/FAIL
#
# Supports:
#   - Generic HTTP webhook (GATE_WEBHOOK_URL)
#   - Gitea Actions (GITEA_ACTIONS_WEBHOOK)
#   - Discord/Slack via incoming webhook URL
#
# Usage:
#   bash gate_webhook.sh violation <gate> <message> [details]
#   bash gate_webhook.sh passed  <gate> [message]
# ============================================================
set -euo pipefail

WEBHOOK_URL="${GATE_WEBHOOK_URL:-}"
WEBHOOK_SECRET="${GATE_WEBHOOK_SECRET:-}"
GITEA_ACTIONS_WEBHOOK="${GITEA_ACTIONS_WEBHOOK:-}"

send_generic() {
    local status="$1"; shift
    local gate="$1"; shift
    local message="$1"; shift
    local details="${1:-}"

    if [ -z "$WEBHOOK_URL" ]; then
        echo "⚠️  Generic webhook not configured (GATE_WEBHOOK_URL not set)"
        return 0
    fi

    local payload=$(cat <<PAYLOAD
{
  "event": "gate_${status}",
  "gate": "$gate",
  "status": "$status",
  "message": "$message",
  "details": "$details",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "branch": "${GITHUB_REF:-${GITEA_BRANCH:-unknown}}",
  "commit": "${GITHUB_SHA:-${GITEA_COMMIT:-unknown}}"
}
PAYLOAD
)

    if [ -n "$WEBHOOK_SECRET" ]; then
        local sig=$(echo -n "$payload" | openssl dgst -sha256 -hmac "$WEBHOOK_SECRET" | cut -d' ' -f2)
        curl -s -X POST "$WEBHOOK_URL" \
            -H "Content-Type: application/json" \
            -H "X-Signature: sha256=$sig" \
            -d "$payload" >/dev/null
    else
        curl -s -X POST "$WEBHOOK_URL" \
            -H "Content-Type: application/json" \
            -d "$payload" >/dev/null
    fi
    echo "✅ Generic webhook sent ($gate: $status)"
}

send_gitea_actions() {
    local status="$1"; shift
    local gate="$1"; shift
    local message="$1"; shift

    if [ -z "$GITEA_ACTIONS_WEBHOOK" ]; then
        return 0
    fi

    # Gitea Actions uses a specific payload format
    local payload=$(cat <<PAYLOAD
{
  "event": "gate_check",
  "gate": "$gate",
  "status": "$status",
  "message": "$message",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "branch": "${GITEA_BRANCH:-unknown}",
  "commit": "${GITEA_COMMIT:-unknown}"
}
PAYLOAD
)

    curl -s -X POST "$GITEA_ACTIONS_WEBHOOK" \
        -H "Content-Type: application/json" \
        -d "$payload" >/dev/null
    echo "✅ Gitea Actions webhook sent ($gate: $status)"
}

send_all() {
    local status="$1"; shift
    local gate="$1"; shift
    local message="$1"; shift
    local details="${1:-}"

    send_generic "$status" "$gate" "$message" "$details"
    send_gitea_actions "$status" "$gate" "$message"
}

case "${1:-}" in
    violation)
        send_all "VIOLATION" "${2:-unknown}" "${3:-gate failed}" "${4:-}"
        ;;
    passed)
        send_all "PASSED" "${2:-unknown}" "${3:-Gate passed successfully}"
        ;;
    *)
        echo "Usage: $0 {violation|passed} <gate> <message> [details]"
        echo ""
        echo "Environment variables:"
        echo "  GATE_WEBHOOK_URL       - Generic HTTP webhook URL"
        echo "  GATE_WEBHOOK_SECRET    - HMAC secret for signature"
        echo "  GITEA_ACTIONS_WEBHOOK - Gitea Actions webhook URL"
        exit 1
        ;;
esac