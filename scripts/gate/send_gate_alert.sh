#!/usr/bin/env bash
# Gate violation alert script - sends notifications when gates fail

set -e

GATE_NAME="${1:-unknown}"
GATE_STATUS="${2:-unknown}"
GATE_MESSAGE="${3:-}"
GATE_DETAILS="${4:-}"

send_alert() {
    local channel="${SLACK_CHANNEL:-#gate-alerts}"
    local webhook_url="${SLACK_WEBHOOK_URL:-}"

    if [ -z "$webhook_url" ]; then
        echo "⚠️  Slack webhook not configured"
        return 1
    fi

    local color="good"
    local emoji="✅"

    case "$GATE_STATUS" in
        VIOLATION)
            color="danger"
            emoji="❌"
            ;;
        WARNING)
            color="warning"
            emoji="⚠️"
            ;;
        PASSED)
            color="good"
            emoji="✅"
            ;;
    esac

    local payload=$(cat <<EOF
{
  "channel": "$channel",
  "username": "Gate Bot",
  "icon_emoji": "$emoji",
  "attachments": [
    {
      "color": "$color",
      "title": "Gate Alert: $GATE_NAME",
      "text": "$GATE_MESSAGE",
      "fields": [
        {
          "title": "Status",
          "value": "$GATE_STATUS",
          "short": true
        },
        {
          "title": "Branch",
          "value": "${GITHUB_REF:-local}",
          "short": true
        }
      ],
      "footer": "SQLRustGo Gate System",
      "ts": $(date +%s)
    }
  ]
}
EOF
)

    curl -s -X POST "$webhook_url" -H "Content-Type: application/json" -d "$payload"
    echo "✅ Alert sent to Slack"
}

send_alert
