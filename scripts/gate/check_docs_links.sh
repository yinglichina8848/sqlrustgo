#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

echo "=== Running Markdown Link Check ==="

MODE="${1:-entry}"
if [ "$MODE" != "entry" ] && [ "$MODE" != "--all" ] && [ "$MODE" != "all" ]; then
    echo "Usage: $0 [entry|--all]"
    exit 2
fi

MD_FILES=()
if [ "$MODE" = "--all" ] || [ "$MODE" = "all" ]; then
    while IFS= read -r -d '' md_file; do
        MD_FILES+=("$md_file")
    done < <(git ls-files -z '*.md')
else
    ENTRY_FILES=(
        "README.md"
        "docs/README.md"
        "crates/README.md"
        "tests/README.md"
        "docs/releases/v2.6.0/README.md"
    )
    for f in "${ENTRY_FILES[@]}"; do
        if [ -f "$f" ]; then
            MD_FILES+=("$f")
        fi
    done
fi

if [ "${#MD_FILES[@]}" -eq 0 ]; then
    echo "No markdown files found."
    exit 0
fi

BROKEN=0
TMP_FILE="$(mktemp)"

cleanup() {
    rm -f "$TMP_FILE"
}
trap cleanup EXIT

for file in "${MD_FILES[@]}"; do
    file_dir="$(dirname "$file")"
    while IFS= read -r raw_target; do
        target="$raw_target"
        target="${target#<}"
        target="${target%>}"
        target="${target%%#*}"
        target="${target%%\?*}"

        if [[ "$target" =~ ^[^[:space:]]+[[:space:]]+\".*\"$ ]]; then
            target="${target%% *}"
        fi

        if [ -z "$target" ]; then
            continue
        fi

        case "$target" in
            http://*|https://*|mailto:*|tel:*|ftp://*|data:*|javascript:*)
                continue
                ;;
        esac

        resolved=""
        if [[ "$target" = /* ]]; then
            if [ -e "$target" ]; then
                continue
            fi
            resolved="${REPO_ROOT}${target}"
        else
            resolved="${file_dir}/${target}"
        fi

        if [ ! -e "$resolved" ]; then
            printf '%s -> %s\n' "$file" "$raw_target" >> "$TMP_FILE"
            BROKEN=1
        fi
    done < <(perl -ne 'while(/\[[^\]]+\]\(([^)]+)\)/g){print "$1\n"}' "$file" 2>/dev/null)
done

if [ "$BROKEN" -ne 0 ]; then
    echo "Broken markdown links detected:"
    sort -u "$TMP_FILE"
    exit 1
fi

echo "All markdown links are valid."
