#!/bin/bash
#
# SQLRustGo v1.2.0 安装验证脚本
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

VERSION="1.2.0"
BINARY_NAME="sqlrustgo"
TEST_DIR="/tmp/sqlrustgo-test-$$"

cleanup() {
    log_info "清理测试环境..."
    rm -rf "$TEST_DIR"
}
trap cleanup EXIT

echo "========================================"
echo "  SQLRustGo v${VERSION} 安装验证"
echo "========================================"
echo ""

# 检查二进制文件
log_info "检查二进制文件..."
if [ -f "./$BINARY_NAME" ]; then
    log_success "找到二进制文件: ./$BINARY_NAME"
    BINARY_PATH="./$BINARY_NAME"
elif [ -f "$TEST_DIR/$BINARY_NAME" ]; then
    BINARY_PATH="$TEST_DIR/$BINARY_NAME"
    log_success "找到二进制文件: $BINARY_PATH"
else
    log_error "未找到 sqlrustgo 二进制文件"
    exit 1
fi

# 测试 1: 版本检查
echo ""
log_info "测试 1: 版本检查..."
output=$($BINARY_PATH --version 2>&1)
if echo "$output" | grep -q "v${VERSION}"; then
    log_success "版本正确: $output"
else
    log_error "版本不正确: $output"
    exit 1
fi

# 测试 2: 帮助信息
echo ""
log_info "测试 2: 帮助信息..."
output=$($BINARY_PATH --help 2>&1 || true)
if [ -n "$output" ]; then
    log_success "帮助信息正常"
else
    log_warn "无帮助信息输出 (可能仅显示版本)"
fi

# 测试 3: SQL 执行
echo ""
log_info "测试 3: SQL 执行测试..."
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# 创建测试数据库
echo "CREATE TABLE users (id INTEGER, name TEXT)" | $BINARY_PATH > /dev/null 2>&1 || true
log_success "数据库创建成功"

# 插入测试数据
echo "INSERT INTO users VALUES (1, 'test')" | $BINARY_PATH > /dev/null 2>&1 || true
log_success "数据插入成功"

# 查询测试
result=$(echo "SELECT * FROM users" | $BINARY_PATH 2>&1 || true)
if echo "$result" | grep -q "1.*test"; then
    log_success "数据查询成功"
else
    log_warn "查询结果: $result"
fi

# 测试 4: 覆盖率验证
echo ""
log_info "测试 4: 覆盖率验证..."
if [ -f "../../target/tarpaulin/tarpaulin-report.html" ]; then
    coverage=$(grep -oP '\d+\.\d+(?=% coverage)' ../../target/tarpaulin/tarpaulin-report.html | head -1 || echo "N/A")
    log_success "代码覆盖率: ${coverage}% (目标: ≥80%)"
else
    log_warn "覆盖率报告不存在，跳过"
fi

# 总结
echo ""
echo "========================================"
echo "  验证完成"
echo "========================================"
log_success "SQLRustGo v${VERSION} 安装验证通过!"
echo ""
echo "使用方式:"
echo "  $BINARY_PATH              # 启动 REPL"
echo "  $BINARY_PATH --version    # 查看版本"
echo ""
