#!/bin/bash
#
# SQLRustGo v1.5.0 安装脚本
#
# 存储引擎 & 表达式优化版本
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

REPO_URL="https://github.com/minzuuniversity/sqlrustgo"
VERSION="1.5.0"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="sqlrustgo"

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

show_help() {
    cat << EOF
SQLRustGo v${VERSION} 安装脚本

用法:
    $0 [选项]

选项:
    --cargo       使用 Cargo 安装
    --binary      使用预编译二进制
    --source      从源码编译安装
    --version     指定版本号 (默认: $VERSION)
    --install-dir 指定安装目录
    --help        显示帮助

更多信息: $REPO_URL
EOF
}

check_dependencies() {
    local missing=()
    if ! command -v curl &> /dev/null && ! command -v wget &> /dev/null; then
        missing+=("curl or wget")
    fi
    if ! command -v tar &> /dev/null; then
        missing+=("tar")
    fi
    if [ ${#missing[@]} -gt 0 ]; then
        log_error "缺少依赖: ${missing[*]}"
        return 1
    fi
    return 0
}

check_rust() { command -v cargo &> /dev/null; }

get_os() {
    case "$(uname -s)" in
        Linux*) echo "linux" ;;
        Darwin*) echo "macos" ;;
        *) echo "unknown" ;;
    esac
}

get_arch() {
    case "$(uname -m)" in
        x86_64) echo "x86_64" ;;
        aarch64|arm64) echo "arm64" ;;
        *) echo "unknown" ;;
    esac
}

install_rust() {
    log_info "安装 Rust..."
    if command -v rustup &> /dev/null; then
        rustup update
    else
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    log_success "Rust 安装完成"
}

install_cargo() {
    log_info "使用 Cargo 安装 SQLRustGo v${VERSION}..."
    if ! check_rust; then install_rust; fi
    [ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
    cargo install sqlrustgo --locked || cargo install sqlrustgo
    log_success "SQLRustGo v${VERSION} 安装完成!"
}

install_binary() {
    local os arch
    os=$(get_os); arch=$(get_arch)
    [ "$os" = "unknown" ] || [ "$arch" = "unknown" ] && { log_error "不支持的系统"; return 1; }

    local filename="sqlrustgo-${VERSION}-${os}-${arch}.tar.gz"
    local download_url="${REPO_URL}/releases/download/v${VERSION}/${filename}"
    local temp_dir=$(mktemp -d)
    trap "rm -rf $temp_dir" EXIT

    log_info "下载: $download_url"
    curl -L -o "$temp_dir/$filename" "$download_url" || { log_error "下载失败"; return 1; }

    tar -xzf "$temp_dir/$filename" -C "$temp_dir"
    mkdir -p "$INSTALL_DIR"
    cp "$temp_dir/sqlrustgo" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    log_success "安装完成: $INSTALL_DIR/$BINARY_NAME"
}

install_from_source() {
    log_info "从源码安装 SQLRustGo v${VERSION}..."
    if ! check_rust; then install_rust; fi
    [ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"

    local repo_dir="/tmp/sqlrustgo-$VERSION"
    [ ! -d "$repo_dir" ] && git clone --depth 1 --branch "release/v1.5.0" "$REPO_URL" "$repo_dir"
    cd "$repo_dir"
    cargo build --release
    mkdir -p "$INSTALL_DIR"
    cp "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    log_success "安装完成"
}

main() {
    local method=""
    while [ $# -gt 0 ]; do
        case "$1" in
            --cargo) method="cargo" ;;
            --binary) method="binary" ;;
            --source) method="source" ;;
            --version) VERSION="$2"; shift 2 ;;
            --install-dir) INSTALL_DIR="$2"; shift 2 ;;
            --help|-h) show_help; exit 0 ;;
            *) show_help; exit 1 ;;
        esac
        shift
    done

    check_dependencies || exit 1

    if [ -z "$method" ]; then
        echo "SQLRustGo v${VERSION} 安装"
        echo "1) Cargo  2) Binary  3) Source"
        read -p "选择: " choice
        case "$choice" in
            1) method="cargo" ;;
            2) method="binary" ;;
            3) method="source" ;;
            *) exit 0 ;;
        esac
    fi

    case "$method" in
        cargo) install_cargo ;;
        binary) install_binary ;;
        source) install_from_source ;;
    esac

    echo ""
    echo "完成! 运行 'sqlrustgo' 启动"
}

main "$@"
