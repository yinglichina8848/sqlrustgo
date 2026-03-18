#!/bin/bash
#
# SQLRustGo v1.4.0 安装验证测试
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

VERSION="1.4.0"
BINARY_NAME="sqlrustgo"
PASSED=0
FAILED=0

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_pass() { echo -e "${GREEN}[PASS]${NC} $1"; ((PASSED++)); }
log_fail() { echo -e "${RED}[FAIL]${NC} $1"; ((FAILED++)); }

find_binary() {
    for p in "$HOME/.local/bin/$BINARY_NAME" "/usr/local/bin/$BINARY_NAME" "./target/release/$BINARY_NAME"; do
        [ -f "$p" ] && echo "$p" && return 0
    done
    return 1
}

test_version() {
    log_info "测试: 版本"
    local BIN=$(find_binary) || BIN="./target/release/$BINARY_NAME"
    local OUT=$("$BIN" --version 2>&1) || true
    echo "$OUT" | grep -q "v${VERSION}" && log_pass "版本正确" || log_fail "版本错误"
}

test_sql() {
    log_info "测试: SQL执行"
    local BIN=$(find_binary) || BIN="./target/release/$BINARY_NAME"
    local OUT=$("$BIN" -e "SELECT 1+1;" 2>&1) || true
    echo "$OUT" | grep -q "2" && log_pass "SQL执行" || log_fail "SQL执行"
}

test_dml() {
    log_info "测试: DML"
    local BIN=$(find_binary) || BIN="./target/release/$BINARY_NAME"
    local OUT=$("$BIN" -e "CREATE TABLE t(id INT); INSERT INTO t VALUES(1); SELECT * FROM t;" 2>&1) || true
    echo "$OUT" | grep -q "1" && log_pass "DML正常" || log_fail "DML失败"
}

test_aggregate() {
    log_info "测试: 聚合"
    local BIN=$(find_binary) || BIN="./target/release/$BINARY_NAME"
    local OUT=$("$BIN" -e "CREATE TABLE s(i INT); INSERT INTO s VALUES(10),(20); SELECT COUNT(*),SUM(i) FROM s;" 2>&1) || true
    echo "$OUT" | grep -q "2" && echo "$OUT" | grep -q "30" && log_pass "聚合正常" || log_fail "聚合失败"
}

test_join() {
    log_info "测试: JOIN"
    local BIN=$(find_binary) || BIN="./target/release/$BINARY_NAME"
    local OUT=$("$BIN" -e "CREATE TABLE u(id INT);CREATE TABLE o(uid INT);INSERT INTO u VALUES(1);INSERT INTO o VALUES(1);SELECT * FROM u JOIN o ON u.id=o.uid;" 2>&1) || true
    echo "$OUT" | grep -q "1" && log_pass "JOIN正常" || log_fail "JOIN失败"
}

main() {
    echo "========================================"
    echo "  SQLRustGo v${VERSION} 安装验证测试"
    echo "========================================"
    echo ""

    local BIN=$(find_binary)
    if [ -z "$BIN" ]; then
        if [ -f "./Cargo.toml" ]; then
            cargo build --release || exit 1
            BIN="./target/release/$BINARY_NAME"
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

    echo ""
    echo "========================================"
    echo "  结果: 通过 $PASSED, 失败 $FAILED"
    echo "========================================"
    [ $FAILED -eq 0 ] && exit 0 || exit 1
}

main "$@"
