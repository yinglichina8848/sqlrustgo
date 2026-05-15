#!/bin/bash
set -e

DATA_DIR="${HOME}/sqlrustgo-tpch/data"

if [ ! -d "$DATA_DIR" ]; then
    echo "Error: $DATA_DIR does not exist"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GEN_BIN="$SCRIPT_DIR/../target/release/gen_tpch"

if [ ! -f "$GEN_BIN" ]; then
    echo "Error: $GEN_BIN not found. Building generator..."
    cd "$SCRIPT_DIR/.."
    cargo build --release -p gen_tpch 2>/dev/null || true
    
    cd /tmp/gen_tpch
    cargo build --release
    GEN_BIN="/tmp/gen_tpch/target/release/gen_tpch"
fi

OUT_DIR="$DATA_DIR" "$GEN_BIN"

echo ""
echo "Verification:"
wc -l "$DATA_DIR"/*.tbl 2>/dev/null | while read count file; do
    name=$(basename "$file")
    echo "  $name: $count rows"
done
