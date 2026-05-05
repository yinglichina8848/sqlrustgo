#!/usr/bin/env bash
# Proof Registry自动更新脚本
# 自动扫描PROOF文件并更新REGISTRY_INDEX.json

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PROOF_DIR="$PROJECT_ROOT/docs/proof"

echo "=== Proof Registry Auto-Update ==="
echo "Date: $(date)"
echo ""

cd "$PROJECT_ROOT"

# 创建临时数组存储proof信息
declare -a proofs
total=0

# 扫描所有PROOF-*.json文件
for json_file in "$PROOF_DIR"/PROOF-*.json; do
    if [ -f "$json_file" ]; then
        # 提取proof_id和title
        proof_id=$(basename "$json_file" .json)
        title=$(grep -o '"title": "[^"]*"' "$json_file" | head -1 | sed 's/"title": "//;s/"$//')
        language=$(grep -o '"language": "[^"]*"' "$json_file" | head -1 | sed 's/"language": "//;s/"$//')
        category=$(grep -o '"category": "[^"]*"' "$json_file" | head -1 | sed 's/"category": "//;s/"$//')
        status=$(grep -o '"status": "[^"]*"' "$json_file" | head -1 | sed 's/"status": "//;s/"$//')

        if [ -n "$proof_id" ]; then
            proofs+=("{\"id\": \"$proof_id\", \"title\": \"$title\", \"language\": \"$language\", \"category\": \"$category\", \"status\": \"$status\"}")
            total=$((total + 1))
            echo "  Found: $proof_id - $title"
        fi
    fi
done

echo ""
echo "Total proofs: $total"

# 生成新的REGISTRY_INDEX.json
{
    echo "{"
    echo "  \"registry_version\": \"1.1\","
    echo "  \"updated_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\","
    echo "  \"total_proofs\": $total,"
    echo "  \"proofs\": ["

    # 输出每个proof
    for i in "${!proofs[@]}"; do
        if [ $i -eq $((${#proofs[@]} - 1)) ]; then
            echo "    ${proofs[$i]}"
        else
            echo "    ${proofs[$i]},"
        fi
    done

    echo "  ]"
    echo "}"
} > "$PROOF_DIR/REGISTRY_INDEX.json"

echo ""
echo "✅ REGISTRY_INDEX.json updated successfully"
echo "   Total proofs: $total"