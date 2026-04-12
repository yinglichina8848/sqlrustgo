#!/bin/bash
#
# SQLRustGo v1.4.0 性能基准测试
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

VERSION="1.4.0"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }

check_env() {
    log_info "检查环境..."
    command -v cargo &> /dev/null || { echo "Rust 未安装"; exit 1; }
    log_success "环境OK"
}

bench_agg() {
    log_info "聚合基准测试..."
    cd "$PROJECT_DIR"
    cargo bench --benches aggregate 2>&1 | tail -15
    log_success "完成"
}

bench_lexer() {
    log_info "词法分析基准测试..."
    cd "$PROJECT_DIR"
    cargo bench --benches lexer 2>&1 | tail -15
    log_success "完成"
}

main() {
    echo "========================================"
    echo "  SQLRustGo v${VERSION} 基准测试"
    echo "========================================"
    echo ""

    check_env
    bench_agg
    bench_lexer

    echo ""
    echo "基准测试完成"
}

main "$@"
