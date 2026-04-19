#!/usr/bin/env bash

set -euo pipefail

VERSION="${1:-}"
STAGE="${2:-unknown}"
OUT_DIR="${3:-}"

if [ -z "$OUT_DIR" ]; then
  echo "Usage: $0 <version> <stage> <out_dir>"
  exit 2
fi

mkdir -p "$OUT_DIR"

METRIC_FILE="$OUT_DIR/METRIC_SNAPSHOT.md"
CAPA_FILE="$OUT_DIR/CAPA_LIST.md"
SUMMARY_FILE="$OUT_DIR/SUMMARY.md"

AUDITS=(
  "$OUT_DIR/PROCESS_AUDIT.md"
  "$OUT_DIR/DEV_AUDIT.md"
  "$OUT_DIR/TEST_AUDIT.md"
  "$OUT_DIR/DOCUMENT_AUDIT.md"
)

PASS=0
WARN=0
FAIL=0

for audit in "${AUDITS[@]}"; do
  if [ -f "$audit" ]; then
    result="$(rg -n "^- RESULT:" "$audit" | awk '{print $3}' | tail -n 1 || true)"
    case "$result" in
      PASS) PASS=$((PASS + 1)) ;;
      WARN) WARN=$((WARN + 1)) ;;
      FAIL) FAIL=$((FAIL + 1)) ;;
      *) WARN=$((WARN + 1)) ;;
    esac
  else
    FAIL=$((FAIL + 1))
  fi
done

CAPA_TMP="$(mktemp)"
cleanup() {
  rm -f "$CAPA_TMP"
}
trap cleanup EXIT

for audit in "${AUDITS[@]}"; do
  if [ -f "$audit" ]; then
    rg -n "^- CAPA:" "$audit" | sed 's/^[0-9]*:- CAPA: //' >> "$CAPA_TMP" || true
  fi
done

sort -u "$CAPA_TMP" > "$CAPA_TMP.sorted" || true
mv "$CAPA_TMP.sorted" "$CAPA_TMP"

TOTAL_AUDITS="${#AUDITS[@]}"
if [ "$FAIL" -gt 0 ]; then
  OVERALL="FAIL"
elif [ "$WARN" -gt 0 ]; then
  OVERALL="WARN"
else
  OVERALL="PASS"
fi

{
  echo "# METRIC_SNAPSHOT"
  echo
  echo "- Version: \`$VERSION\`"
  echo "- Stage: \`$STAGE\`"
  echo "- Date: $(date '+%Y-%m-%d %H:%M:%S %z')"
  echo
  echo "## Audit Status"
  echo
  echo "- TOTAL_AUDITS: $TOTAL_AUDITS"
  echo "- PASS_AUDITS: $PASS"
  echo "- WARN_AUDITS: $WARN"
  echo "- FAIL_AUDITS: $FAIL"
  echo "- OVERALL_RESULT: $OVERALL"
  echo
  echo "## Suggested Core Metrics"
  echo
  echo "- A_GATE_PASS_RATE: $(( (PASS * 100) / TOTAL_AUDITS ))%"
  echo "- AUDIT_WARNING_RATE: $(( (WARN * 100) / TOTAL_AUDITS ))%"
  echo "- AUDIT_FAIL_RATE: $(( (FAIL * 100) / TOTAL_AUDITS ))%"
} > "$METRIC_FILE"

{
  echo "# CAPA_LIST"
  echo
  echo "- Version: \`$VERSION\`"
  echo "- Stage: \`$STAGE\`"
  echo "- Date: $(date '+%Y-%m-%d %H:%M:%S %z')"
  echo
  echo "## Prioritized CAPA"
  if [ -s "$CAPA_TMP" ]; then
    sed 's/^/- /' "$CAPA_TMP"
  else
    echo "- P3|No urgent CAPA. Continue current governance baseline."
  fi
} > "$CAPA_FILE"

{
  echo "# SELF_OPTIMIZATION_SUMMARY"
  echo
  echo "- Version: \`$VERSION\`"
  echo "- Stage: \`$STAGE\`"
  echo "- Date: $(date '+%Y-%m-%d %H:%M:%S %z')"
  echo "- OVERALL_RESULT: $OVERALL"
  echo
  echo "## Reports"
  echo "- PROCESS_AUDIT.md"
  echo "- DEV_AUDIT.md"
  echo "- TEST_AUDIT.md"
  echo "- DOCUMENT_AUDIT.md"
  echo "- METRIC_SNAPSHOT.md"
  echo "- CAPA_LIST.md"
  echo
  echo "## Next Actions"
  if [ "$OVERALL" = "FAIL" ]; then
    echo "1. Block stage promotion."
    echo "2. Execute P0 CAPA items first."
    echo "3. Re-run optimization cycle after fixes."
  elif [ "$OVERALL" = "WARN" ]; then
    echo "1. Stage promotion allowed with risk review."
    echo "2. Execute P1/P2 CAPA in next iteration."
  else
    echo "1. Stage promotion allowed."
    echo "2. Keep monitoring metrics trend."
  fi
} > "$SUMMARY_FILE"

echo "Generated $METRIC_FILE"
echo "Generated $CAPA_FILE"
echo "Generated $SUMMARY_FILE"

if [ "$OVERALL" = "FAIL" ]; then
  exit 3
fi
