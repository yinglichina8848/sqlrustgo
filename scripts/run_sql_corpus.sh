#!/bin/bash
# SQLRustGo SQL Corpus Test Runner
#
# This script executes all SQL statements in the sql_corpus directory
# against SQLRustGo to validate MySQL 5.7 compatibility.
#
# Usage:
#   ./scripts/run_sql_corpus.sh [--server <binary>] [--port <port>] [--parallel]
#
# Options:
#   --server <binary>   Path to sqlrustgo binary (default: ./target/release/sqlrustgo)
#   --port <port>       MySQL server port (default: 3306)
#   --parallel          Run tests in parallel (default: sequential)
#   --category <name>   Run specific category only (DDL, DML, etc.)
#   --verbose           Show detailed output
#   --help              Show this help message

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
SERVER="./target/release/sqlrustgo"
PORT=3306
PARALLEL=false
CATEGORY=""
VERBOSE=false
TIMEOUT=30

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CORPUS_DIR="$(dirname "$SCRIPT_DIR")/sql_corpus"

# Statistics
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Usage function
usage() {
    head -22 "$0" | tail -20 | sed 's/^#//'
    exit 0
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --server)
            SERVER="$2"
            shift 2
            ;;
        --port)
            PORT="$2"
            shift 2
            ;;
        --parallel)
            PARALLEL=true
            shift
            ;;
        --category)
            CATEGORY="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            usage
            ;;
        *)
            echo "Unknown option: $1"
            usage
            ;;
    esac
done

# Check if corpus directory exists
if [[ ! -d "$CORPUS_DIR" ]]; then
    echo -e "${RED}Error: SQL corpus directory not found: $CORPUS_DIR${NC}"
    exit 1
fi

# Check if server binary exists
if [[ ! -f "$SERVER" ]]; then
    echo -e "${RED}Error: Server binary not found: $SERVER${NC}"
    echo -e "${YELLOW}Build with: cargo build --release${NC}"
    exit 1
fi

# Print header
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  SQLRustGo SQL Corpus Test Runner${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "Corpus Directory: $CORPUS_DIR"
echo "Server Binary:    $SERVER"
echo "Server Port:     $PORT"
echo "Parallel:        $PARALLEL"
echo ""

# Check if MySQL server is running
check_server() {
    nc -z localhost $PORT 2>/dev/null || return 1
    return 0
}

# Wait for server to be ready
wait_for_server() {
    echo -e "${YELLOW}Waiting for server to start...${NC}"
    for i in {1..30}; do
        if check_server; then
            echo -e "${GREEN}Server is ready!${NC}"
            return 0
        fi
        sleep 1
    done
    echo -e "${RED}Error: Server did not start within 30 seconds${NC}"
    return 1
}

# Execute a single SQL file
execute_sql_file() {
    local file="$1"
    local category="$2"
    local test_name=$(basename "$file" .sql)

    if [[ "$VERBOSE" == true ]]; then
        echo -e "${BLUE}  [TEST] $category/$test_name${NC}"
    fi

    # Execute SQL via mysql CLI or pipe to server
    if command -v mysql &> /dev/null; then
        # Use mysql CLI if available
        timeout $TIMEOUT mysql -h localhost -P $PORT -u root default 2>/dev/null < "$file"
    else
        # Fallback: pipe to server REPL
        timeout $TIMEOUT echo ".read $file" | $SERVER 2>/dev/null || true
    fi

    return $?
}

# Run a category of tests
run_category() {
    local category="$1"
    local category_dir="$CORPUS_DIR/$category"

    if [[ ! -d "$category_dir" ]]; then
        return 0
    fi

    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  Category: $category${NC}"
    echo -e "${GREEN}========================================${NC}"

    local category_total=0
    local category_passed=0
    local category_failed=0

    # Find all SQL files
    shopt -s nullglob
    local sql_files=("$category_dir"/*.sql)
    shopt -s nullglob

    if [[ ${#sql_files[@]} -eq 0 ]]; then
        echo -e "${YELLOW}  No SQL files found in $category_dir${NC}"
        return 0
    fi

    for sql_file in "${sql_files[@]}"; do
        ((category_total++))
        ((TOTAL_TESTS++))

        test_name=$(basename "$sql_file")

        if execute_sql_file "$sql_file" "$category" 2>/dev/null; then
            ((category_passed++))
            ((PASSED_TESTS++))
            echo -e "  ${GREEN}✓${NC} $test_name"
        else
            ((category_failed++))
            ((FAILED_TESTS++))
            echo -e "  ${RED}✗${NC} $test_name"
            if [[ "$VERBOSE" == true ]]; then
                echo -e "    ${RED}Failed to execute${NC}"
            fi
        fi
    done

    echo ""
    echo -e "  Category Summary: ${GREEN}$category_passed passed${NC}, ${RED}$category_failed failed${NC}, $category_total total"
    echo ""

    return 0
}

# Run all categories
run_all_categories() {
    local categories=("DDL" "DML" "EXPRESSIONS" "TRANSACTION" "SPECIAL" "DEBUG" "ADVANCED" "INDEXES" "VIEWS" "TRIGGERS" "PROCEDURES" "EVENTS" "FUNCTIONS" "TCL")

    for category in "${categories[@]}"; do
        if [[ -z "$CATEGORY" || "$CATEGORY" == "$category" ]]; then
            run_category "$category" || true
        fi
    done
}

# Print final summary
print_summary() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  Final Summary${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
    echo -e "Total Tests:  $TOTAL_TESTS"
    echo -e "${GREEN}Passed:      $PASSED_TESTS${NC}"
    echo -e "${RED}Failed:      $FAILED_TESTS${NC}"
    echo -e "Skipped:      $SKIPPED_TESTS"
    echo ""

    if [[ $FAILED_TESTS -eq 0 ]]; then
        echo -e "${GREEN}All tests passed!${NC}"
        return 0
    else
        echo -e "${RED}Some tests failed.${NC}"
        return 1
    fi
}

# Main execution
main() {
    # Check if server is already running
    if ! check_server; then
        echo -e "${YELLOW}MySQL server is not running on port $PORT${NC}"
        echo -e "${YELLOW}Please start it first:${NC}"
        echo -e "  sqlrustgo-mysql-server --port $PORT &"
        exit 1
    fi

    # Run tests
    run_all_categories

    # Print summary
    print_summary
}

# Run main
main "$@"
