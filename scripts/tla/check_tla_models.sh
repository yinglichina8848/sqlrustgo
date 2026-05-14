#!/bin/bash
# TLA+ Model Checker CI Integration
# Runs TLC model checker for PROOF-003 and PROOF-005

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROOF_DIR="$SCRIPT_DIR/../docs/gmp-compliance/proof"
OUTPUT_DIR="$SCRIPT_DIR/../.tla-output"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "=========================================="
echo "TLA+ Model Checking for SQLRustGo"
echo "=========================================="

# Check for TLA+ Toolbox
if ! command -v java &> /dev/null; then
    echo -e "${YELLOW}Warning: Java not found. TLA+ Toolbox requires Java.${NC}"
    echo "Skipping TLA+ model checking."
    echo "To enable: Install Java 11+ and TLA+ Toolbox"
    exit 0
fi

# Check for TLC
TLC=""
if [ -f "/usr/local/bin/tlc" ]; then
    TLC="/usr/local/bin/tlc"
elif [ -f "$HOME/tlatools/tlc.jar" ]; then
    TLC="java -cp $HOME/tlatools/tla2tools.jar tlc2.TLC"
elif [ -f "$SCRIPT_DIR/tla2tools.jar" ]; then
    TLC="java -cp $SCRIPT_DIR/tla2tools.jar tlc2.TLC"
else
    echo -e "${YELLOW}Warning: TLA+ Toolbox not found.${NC}"
    echo "Download from: https://github.com/tlaplus/tlaplus/releases"
    echo "Place tla2tools.jar in: $SCRIPT_DIR/"
    echo "Skipping TLA+ model checking."
    exit 0
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Model files to check
MODELS=(
    "PROOF-003-wal-recovery"
    "PROOF-005-mvcc-snapshot"
)

run_model_check() {
    local model=$1
    local tla_file="$PROOF_DIR/${model}.tla"
    local cfg_file="$PROOF_DIR/${model}_toolbox/${model}.cfg"
    local output_file="$OUTPUT_DIR/${model}.out"
    
    echo ""
    echo "----------------------------------------"
    echo "Checking: $model"
    echo "----------------------------------------"
    
    if [ ! -f "$tla_file" ]; then
        echo -e "${YELLOW}Warning: $tla_file not found, skipping${NC}"
        return 0
    fi
    
    echo "Running TLC model checker..."
    echo "Output: $output_file"
    
    # Run TLC
    if [ -f "$cfg_file" ]; then
        $TLC -modelcheck -config "$cfg_file" "$tla_file" > "$output_file" 2>&1
    else
        $TLC -modelcheck "$tla_file" > "$output_file" 2>&1
    fi
    
    local result=$?
    
    if [ $result -eq 0 ]; then
        if grep -q "Model checking completed. No error found." "$output_file"; then
            echo -e "${GREEN}✓ $model: PASSED${NC}"
            echo "Model checking completed. No error found." >> "$OUTPUT_DIR/summary.txt"
            return 0
        else
            echo -e "${YELLOW}⚠ $model: COMPLETED WITH WARNINGS${NC}"
            grep -i "warning\|error" "$output_file" | head -5
            return 1
        fi
    else
        if grep -q "Error:" "$output_file"; then
            echo -e "${RED}✗ $model: FAILED${NC}"
            grep "Error:" "$output_file" | head -5
            return 1
        else
            echo -e "${RED}✗ $model: FAILED (exit code $result)${NC}"
            tail -20 "$output_file"
            return 1
        fi
    fi
}

# Main
echo ""
echo "Models to check:"
for model in "${MODELS[@]}"; do
    echo "  - $model"
done
echo ""

# Initialize summary
echo "TLA+ Model Checking Summary" > "$OUTPUT_DIR/summary.txt"
echo "==============================" >> "$OUTPUT_DIR/summary.txt"
echo "Date: $(date)" >> "$OUTPUT_DIR/summary.txt"
echo "" >> "$OUTPUT_DIR/summary.txt"

FAILED=0

for model in "${MODELS[@]}"; do
    run_model_check "$model"
    if [ $? -ne 0 ]; then
        FAILED=$((FAILED + 1))
    fi
done

echo ""
echo "=============================="
echo "Summary:"
echo "=============================="
cat "$OUTPUT_DIR/summary.txt"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo -e "${GREEN}All models passed!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}$FAILED model(s) failed.${NC}"
    exit 1
fi