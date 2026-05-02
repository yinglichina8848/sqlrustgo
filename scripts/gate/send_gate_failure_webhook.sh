#!/usr/bin/env bash
# Gate CI失败时发送webhook通知

set -e

WEBHOOK_URL="${GATE_WEBHOOK_URL:-}"
WEBHOOK_SECRET="${GATE_WEBHOOK_SECRET:-}"

send_gate_failure() {
    local gate_name="$1"
    local error_message="$2"
    local commit_sha="${GITHUB_SHA:-unknown}"
    local branch="${GITHUB_REF:-unknown}"

    if [ -z "$WEBHOOK_URL" ]; then
        echo "⚠️ Webhook not configured (GATE_WEBHOOK_URL not set)"
        return 0
    fi

    local payload=$(cat <<EOF
{
  "event": "gate_failure",
  "gate": "$gate_name",
  "status": "FAILED",
  "message": "$error_message",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "branch": "$branch",
  "commit": "$commit_sha",
  "run_url": "${GITHUB_RUN_URL:-N/A}"
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

    echo "✅ Gate failure webhook sent for $gate_name"
}

gate_name="${1:-unknown}"
error_msg="${2:-Gate check failed}"

send_gate_failure "$gate_name" "$error_msg"