#!/usr/bin/env bash
# ============================================================
# check_oo_docs.sh — OO 文档真实性门禁
#
# 验证 docs/releases/v3.0.0/oo/README.md 中标记的文档
# 是否真实存在于文件系统中。
#
# Exit code 0 = all OK, non-zero = mismatch found
# ============================================================

set -e

OO_DIR="docs/releases/v3.0.0/oo"
README="$OO_DIR/README.md"

echo "============================================"
echo "OO Document Authenticity Gate"
echo "============================================"

errors=0

# Extract "✅" entries from README and check if files exist
# Format: | \`path/to/file.md\` | ✅ | description |
while IFS= read -r line; do
    # Skip non-markdown table lines
    [[ "$line" =~ \|.*✅.*\| ]] || continue
    
    # Extract file path from backticks
    if [[ "$line" =~ \`([^\`]+)\` ]]; then
        filepath="${BASH_REMATCH[1]}"
        fullpath="$OO_DIR/$filepath"
        
        if [[ ! -f "$fullpath" ]]; then
            echo "❌ MISSING: $filepath (README claims ✅)"
            errors=$((errors + 1))
        else
            size=$(wc -c < "$fullpath" 2>/dev/null || echo 0)
            echo "✅ OK:     $filepath (${size}B)"
        fi
    fi
done < "$README"

echo ""
echo "============================================"
if [[ $errors -eq 0 ]]; then
    echo "✅ PASS — All documented files exist"
    exit 0
else
    echo "❌ FAIL — $errors file(s) missing but claimed ✅ in README"
    exit 1
fi
