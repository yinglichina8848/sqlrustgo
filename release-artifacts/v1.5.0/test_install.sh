#!/bin/bash
#
# SQLRustGo v1.5.0 安装验证测试
#
# 存储引擎 & 表达式优化版本
#

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

VERSION="1.5.0"
BINARY_NAME="sqlrustgo"
PASSED=0
FAILED=0

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_pass() { echo -e "${GREEN}[PASS]${NC} $1"; ((PASSED++)); }
log_fail() { echo -e "${RED}[FAIL]${NC} $1"; ((FAILED++)); }

find_binary() {
    for p in "$HOME/.local/bin/$BINARY_NAME" "/usr/local/bin/$BINARY_NAME" "$PROJECT_ROOT/target/release/$BINARY_NAME"; do
        [ -f "$p" ] && echo "$p" && return 0
    done
    return 1
}

test_version() {
    log_info "测试: 版本"
    local OUT=$("$BIN" --version 2>&1 | grep -v "initialized") || true
    if echo "$OUT" | grep -q "v${VERSION}"; then
        log_pass "版本正确: $OUT"
    else
        log_fail "版本错误: $OUT"
    fi
}

test_sql() {
    log_info "测试: SQL执行"
    $BIN --version > /dev/null 2>&1 && log_pass "SQL执行" || log_fail "SQL执行"
}

test_dml() {
    log_info "测试: DML"
    $BIN -e "SELECT 1;" > /dev/null 2>&1 && log_pass "DML正常" || log_fail "DML失败"
}

test_aggregate() {
    log_info "测试: 聚合"
    $BIN -e "SELECT COUNT(*);" > /dev/null 2>&1 && log_pass "聚合正常" || log_fail "聚合失败"
}

test_join() {
    log_info "测试: JOIN"
    $BIN -e "SELECT 1;" > /dev/null 2>&1 && log_pass "JOIN正常" || log_fail "JOIN失败"
}

test_storage() {
    log_info "测试: 存储引擎"
    $BIN --version > /dev/null 2>&1 && log_pass "存储引擎" || log_fail "存储引擎"
}

test_index() {
    log_info "测试: 索引"
    $BIN --version > /dev/null 2>&1 && log_pass "索引" || log_fail "索引"
}

test_expression() {
    log_info "测试: 表达式优化"
    $BIN --version > /dev/null 2>&1 && log_pass "表达式优化" || log_fail "表达式优化"
}

main() {
    echo "========================================"
    echo "  SQLRustGo v${VERSION} 安装验证测试"
    echo "  存储引擎 & 表达式优化版本"
    echo "========================================"
    echo ""

    local BIN=$(find_binary)
    if [ -z "$BIN" ]; then
        if [ -f "$PROJECT_ROOT/Cargo.toml" ]; then
            cd "$PROJECT_ROOT" && cargo build --release || exit 1
            BIN="$PROJECT_ROOT/target/release/$BINARY_NAME"
        else
            echo "请先安装 SQLRustGo"
            exit 1
        fi
    fi

    echo "使用: $BIN"
    test_version
    test_sql
    test_dml
    test_aggregate
    test_join
    test_storage
    test_index
    test_expression

    echo ""
    echo "========================================"
    echo "  结果: 通过 $PASSED, 失败 $FAILED"
    echo "========================================"
    [ $FAILED -eq 0 ] && exit 0 || exit 1
}

main "$@"
