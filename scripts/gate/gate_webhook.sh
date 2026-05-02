#!/usr/bin/env bash
# Webhook notification script for gate violations

set -e

WEBHOOK_URL="${GATE_WEBHOOK_URL:-}"
WEBHOOK_SECRET="${GATE_WEBHOOK_SECRET:-}"

send_webhook() {
    local status="$1"
    local gate="$2"
    local message="$3"
    local details="$4"

    if [ -z "$WEBHOOK_URL" ]; then
        echo "⚠️  Webhook not configured (GATE_WEBHOOK_URL not set)"
        return 0
    fi

    local payload=$(cat <<EOF
{
  "event": "gate_violation",
  "gate": "$gate",
  "status": "$status",
  "message": "$message",
  "details": "$details",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "branch": "${GITHUB_REF:-unknown}",
  "commit": "${GITHUB_SHA:-unknown}"
}
EOF
)

    if [ -n "$WEBHOOK_SECRET" ]; then
        local signature=$(echo -n "$payload" | openssl dgst -sha256 -hmac "$WEBHOOK_SECRET" | cut -d' ' -f2)
        curl -s -X POST "$WEBHOOK_URL" \
            -H "Content-Type: application/json" \
            -H "X-Signature: sha256=$signature" \
            -d "$payload"
    else
        curl -s -X POST "$WEBHOOK_URL" \
            -H "Content-Type: application/json" \
            -d "$payload"
    fi

    echo "✅ Webhook sent for $gate ($status)"
}

case "$1" in
    violation)
        send_webhook "VIOLATION" "$2" "$3" "$4"
        ;;
    passed)
        send_webhook "PASSED" "$2" "Gate passed successfully" ""
        ;;
    *)
        echo "Usage: $0 {violation|passed} <gate> <message> [details]"
        exit 1
        ;;
esac
