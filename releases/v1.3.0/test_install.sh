#!/bin/bash
#
# SQLRustGo v1.3.0 安装验证测试脚本
#
# 用法:
#   ./test_install.sh              # 运行所有测试
#   ./test_install.sh --quick      # 快速测试
#   ./test_install.sh --help       # 显示帮助
#

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 配置
VERSION="1.3.0"
BINARY_NAME="sqlrustgo"
TEST_PASSED=0
TEST_FAILED=0

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((TEST_PASSED++))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((TEST_FAILED++))
}

show_help() {
    cat << EOF
SQLRustGo v${VERSION} 安装验证测试

用法:
  $0 [选项]

选项:
  --all       运行所有测试 (默认)
  --quick     仅运行快速测试
  --help      显示帮助信息

测试项目:
  - 版本信息验证
  - REPL 模式测试
  - SQL 执行测试
  - 健康检查端点测试
  - 指标端点测试
  - 服务器模式测试
EOF
}

# 查找已安装的二进制
find_binary() {
    local paths=("$HOME/.local/bin/$BINARY_NAME" "/usr/local/bin/$BINARY_NAME" "./target/release/$BINARY_NAME")
    for path in "${paths[@]}"; do
        if [ -f "$path" ]; then
            echo "$path"
            return 0
        fi
    done
    return 1
}

# 测试版本信息
test_version() {
    log_info "测试: 版本信息"

    local BINARY
    BINARY=$(find_binary) || BINARY="./target/release/$BINARY_NAME"

    if [ ! -f "$BINARY" ]; then
        log_fail "找不到 $BINARY_NAME 二进制文件"
        return 1
    fi

    local VERSION_OUTPUT
    VERSION_OUTPUT=$("$BINARY" --version 2>&1 || "$BINARY" --help 2>&1 | head -1)

    if echo "$VERSION_OUTPUT" | grep -q "$VERSION"; then
        log_success "版本信息正确: $VERSION_OUTPUT"
    else
        log_fail "版本信息不匹配，期望: $VERSION, 实际: $VERSION_OUTPUT"
    fi
}

# 测试帮助信息
test_help() {
    log_info "测试: 帮助信息"

    local BINARY
    BINARY=$(find_binary) || BINARY="./target/release/$BINARY_NAME"

    if "$BINARY" --help &> /dev/null; then
        log_success "帮助信息正常"
    else
        log_fail "帮助信息异常"
    fi
}

# 测试 REPL 模式
test_repl() {
    log_info "测试: REPL 模式"

    local BINARY
    BINARY=$(find_binary) || BINARY="./target/release/$BINARY_NAME"

    # 启动 REPL，发送命令，退出
    local OUTPUT
    OUTPUT=$(echo -e "version\n.exit" | timeout 5 "$BINARY" 2>&1 || true)

    if echo "$OUTPUT" | grep -q "SQLRustGo"; then
        log_success "REPL 模式正常"
    else
        log_fail "REPL 模式异常"
    fi
}

# 测试 SQL 执行
test_sql_execution() {
    log_info "测试: SQL 执行"

    local BINARY
    BINARY=$(find_binary) || BINARY="./target/release/$BINARY_NAME"

    # 执行简单 SQL
    local OUTPUT
    OUTPUT=$(echo -e "CREATE TABLE test (id INTEGER, name TEXT);\nINSERT INTO test VALUES (1, 'Alice');\nSELECT * FROM test;\n.exit" | timeout 5 "$BINARY" 2>&1 || true)

    if echo "$OUTPUT" | grep -qE "(Alice|1.*Alice)"; then
        log_success "SQL 执行正常"
    else
        log_fail "SQL 执行异常"
    fi
}

# 测试健康检查端点 (需要服务器模式)
test_health_endpoint() {
    log_info "测试: 健康检查端点"

    local BINARY
    BINARY=$(find_binary) || BINARY="./target/release/$BINARY_NAME"

    # 启动服务器
    "$BINARY" --server &
    SERVER_PID=$!

    sleep 2

    # 测试 /health/live
    local HEALTH_LIVE
    HEALTH_LIVE=$(curl -s http://localhost:5432/health/live 2>/dev/null || echo "")

    if echo "$HEALTH_LIVE" | grep -q "alive"; then
        log_success "/health/live 端点正常"
    else
        log_fail "/health/live 端点异常"
    fi

    # 测试 /health/ready
    local HEALTH_READY
    HEALTH_READY=$(curl -s http://localhost:5432/health/ready 2>/dev/null || echo "")

    if echo "$HEALTH_READY" | grep -q "ready"; then
        log_success "/health/ready 端点正常"
    else
        log_fail "/health/ready 端点异常"
    fi

    # 清理
    kill $SERVER_PID 2>/dev/null || true
}

# 测试指标端点 (需要服务器模式)
test_metrics_endpoint() {
    log_info "测试: 指标端点"

    local BINARY
    BINARY=$(find_binary) || BINARY="./target/release/$BINARY_NAME"

    # 启动服务器
    "$BINARY" --server &
    SERVER_PID=$!

    sleep 2

    # 测试 /metrics
    local METRICS
    METRICS=$(curl -s http://localhost:5432/metrics 2>/dev/null || echo "")

    if echo "$METRICS" | grep -q "sqlrustgo"; then
        log_success "/metrics 端点正常"
    else
        log_fail "/metrics 端点异常"
    fi

    # 清理
    kill $SERVER_PID 2>/dev/null || true
}

# 快速测试
test_quick() {
    log_info "开始快速测试..."
    echo ""

    test_version
    test_help
    test_repl
    test_sql_execution
}

# 完整测试
test_all() {
    log_info "开始完整测试..."
    echo ""

    test_version
    test_help
    test_repl
    test_sql_execution

    # 端点测试 (可选)
    if command -v curl &> /dev/null; then
        test_health_endpoint
        test_metrics_endpoint
    else
        log_info "跳过端点测试 (curl 未安装)"
    fi
}

# 构建测试
test_build() {
    log_info "测试: 从源码构建"

    cd /Users/liying/workspace/dev/yinglichina163/sqlrustgo

    if cargo build --release &> /dev/null; then
        log_success "源码构建成功"
    else
        log_fail "源码构建失败"
    fi
}

# 测试覆盖率
test_coverage() {
    log_info "测试: 覆盖率"

    cd /Users/liying/workspace/dev/yinglichina163/sqlrustgo

    if cargo test --workspace &> /dev/null; then
        log_success "所有测试通过"
    else
        log_fail "测试失败"
    fi
}

# 主程序
main() {
    echo ""
    echo "========================================"
    echo "  SQLRustGo v${VERSION} 安装验证测试"
    echo "========================================"
    echo ""

    case "${1:-}" in
        --quick)
            test_quick
            ;;
        --all)
            test_all
            ;;
        --build)
            test_build
            ;;
        --coverage)
            test_coverage
            ;;
        --help)
            show_help
            exit 0
            ;;
        "")
            test_quick
            ;;
        *)
            log_error "未知选项: $1"
            show_help
            exit 1
            ;;
    esac

    echo ""
    echo "========================================"
    echo "  测试结果"
    echo "========================================"
    echo -e "  通过: ${GREEN}$TEST_PASSED${NC}"
    echo -e "  失败: ${RED}$TEST_FAILED${NC}"
    echo ""

    if [ $TEST_FAILED -eq 0 ]; then
        log_success "所有测试通过!"
        exit 0
    else
        log_fail "有测试失败"
        exit 1
    fi
}

main "$@"
