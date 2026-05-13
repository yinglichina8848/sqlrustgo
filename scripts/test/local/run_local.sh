#!/usr/bin/env bash
# run_local.sh - 本地测试统一入口
# 支持 L0/L1/L2/L3 分层测试
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

LEVEL="${1:-0}"
CRATE="${2:-}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

usage() {
    cat << 'EOF'
SQLRustGo Local Test Runner

Usage:
    ./run_local.sh [LEVEL] [CRATE]

Levels:
    L0  - Smoke test (<5min)
    L1  - Unit tests (10min)
    L2  - Integration tests (15min)
    L3  - Full regression (20min)
    all - Run all levels

Examples:
    ./run_local.sh L0
    ./run_local.sh L1 executor
    ./run_local.sh L2
    ./run_local.sh all

EOF
    exit 0
}

if [ "$#" -eq 0 ] || [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    usage
fi

case "$LEVEL" in
    L0|l0)
        echo -e "${YELLOW}Running L0 Smoke Test...${NC}"
        "$SCRIPT_DIR/l0_smoke.sh"
        ;;
    L1|l1)
        echo -e "${YELLOW}Running L1 Unit Tests...${NC}"
        "$SCRIPT_DIR/l1_unit.sh"
        ;;
    L2|l2)
        echo -e "${YELLOW}Running L2 Integration Tests...${NC}"
        "$SCRIPT_DIR/l2_integration.sh"
        ;;
    L3|l3)
        echo -e "${YELLOW}Running L3 Regression Tests...${NC}"
        "$SCRIPT_DIR/l3_regression.sh" 2>/dev/null || {
            echo -e "${RED}L3 regression script not yet implemented${NC}"
            exit 1
        }
        ;;
    all)
        echo -e "${YELLOW}Running All Local Tests...${NC}"
        echo ""
        "$SCRIPT_DIR/l0_smoke.sh" || exit 1
        echo ""
        "$SCRIPT_DIR/l1_unit.sh" || exit 1
        echo ""
        "$SCRIPT_DIR/l2_integration.sh" || exit 1
        echo ""
        echo -e "${GREEN}✅ All Local Tests Passed${NC}"
        ;;
    *)
        echo -e "${RED}Unknown level: $LEVEL${NC}"
        usage
        ;;
esac
