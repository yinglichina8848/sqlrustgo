#!/bin/bash
#
# SQLRustGo 安装脚本
# 支持两种安装方式:
#   1. Cargo 安装 (需要 Rust 工具链)
#   2. 预编译二进制安装
#
# 用法:
#   ./install.sh              # 交互式安装
#   ./install.sh --cargo      # 使用 Cargo 安装
#   ./install.sh --binary     # 使用预编译二进制
#   ./install.sh --help       # 显示帮助
#

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
REPO_URL="https://github.com/minzuuniversity/sqlrustgo"
VERSION="${VERSION:-1.0.0}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="sqlrustgo"

# 函数定义
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

show_help() {
    cat << EOF
SQLRustGo 安装脚本

用法:
    $0 [选项]

选项:
    --cargo       使用 Cargo 安装 (需要 Rust 工具链)
    --binary      使用预编译二进制安装
    --version     指定版本号 (默认: $VERSION)
    --install-dir 指定安装目录 (默认: $INSTALL_DIR)
    --help        显示此帮助信息

示例:
    $0                    # 交互式安装
    $0 --cargo           # 使用 Cargo 安装
    $0 --binary          # 使用预编译二进制
    $0 --version 1.0.0   # 安装指定版本
    $0 --install-dir /usr/local/bin  # 安装到指定目录

更多信息请访问: $REPO_URL
EOF
}

# 检查依赖
check_dependencies() {
    local missing=()

    # 检查 curl 或 wget
    if ! command -v curl &> /dev/null && ! command -v wget &> /dev/null; then
        missing+=("curl or wget")
    fi

    # 检查 tar
    if ! command -v tar &> /dev/null; then
        missing+=("tar")
    fi

    if [ ${#missing[@]} -gt 0 ]; then
        log_error "缺少依赖: ${missing[*]}"
        return 1
    fi

    return 0
}

# 检查 Rust 环境
check_rust() {
    if command -v cargo &> /dev/null; then
        return 0
    else
        return 1
    fi
}

# 获取系统信息
get_os() {
    local os
    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="macos" ;;
        CYGWIN*|MINGW*|MSYS*) os="windows" ;;
        *)          os="unknown" ;;
    esac
    echo "$os"
}

get_arch() {
    local arch
    case "$(uname -m)" in
        x86_64)    arch="x86_64" ;;
        aarch64|arm64) arch="arm64" ;;
        *)         arch="unknown" ;;
    esac
    echo "$arch"
}

# 安装 Rust (如果需要)
install_rust() {
    log_info "正在安装 Rust..."

    if command -v rustup &> /dev/null; then
        log_info "Rust 已安装，更新中..."
        rustup update
    else
        log_info "下载并安装 Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    log_success "Rust 安装完成"
}

# Cargo 安装
install_cargo() {
    log_info "开始使用 Cargo 安装 SQLRustGo..."

    if ! check_rust; then
        install_rust
    fi

    # 确保 cargo 环境可用
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi

    log_info "正在编译安装 (首次编译可能需要几分钟)..."
    cargo install sqlrustgo --locked || {
        log_warn "使用 --locked 失败，尝试不使用锁定文件..."
        cargo install sqlrustgo
    }

    log_success "SQLRustGo 安装完成!"
    log_info "运行 'sqlrustgo' 启动数据库"
}

# 预编译二进制安装
install_binary() {
    local os arch
    os=$(get_os)
    arch=$(get_arch)

    if [ "$os" = "unknown" ] || [ "$arch" = "unknown" ]; then
        log_error "不支持的系统架构: $(uname -s) $(uname -m)"
        return 1
    fi

    # 确定文件名
    local filename="sqlrustgo-${VERSION}-${os}-${arch}.tar.gz"
    local download_url="${REPO_URL}/releases/download/v${VERSION}/${filename}"

    log_info "系统: $os-$arch"
    log_info "版本: $VERSION"
    log_info "下载地址: $download_url"

    # 创建临时目录
    local temp_dir
    temp_dir=$(mktemp -d)
    trap "rm -rf $temp_dir" EXIT

    # 下载
    log_info "正在下载..."
    if command -v curl &> /dev/null; then
        curl -L -o "$temp_dir/$filename" "$download_url" || {
            log_error "下载失败，版本 $VERSION 可能不存在"
            log_info "尝试使用 Cargo 安装..."
            install_cargo
            return 0
        }
    else
        wget -O "$temp_dir/$filename" "$download_url" || {
            log_error "下载失败，版本 $VERSION 可能不存在"
            log_info "尝试使用 Cargo 安装..."
            install_cargo
            return 0
        }
    fi

    # 解压
    log_info "正在解压..."
    tar -xzf "$temp_dir/$filename" -C "$temp_dir"

    # 创建安装目录
    mkdir -p "$INSTALL_DIR"

    # 安装
    local binary_path="$temp_dir/sqlrustgo"
    if [ -f "$binary_path" ]; then
        cp "$binary_path" "$INSTALL_DIR/$BINARY_NAME"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
        log_success "SQLRustGo 安装完成!"
        log_info "安装位置: $INSTALL_DIR/$BINARY_NAME"
    else
        log_error "解压后的二进制文件不存在"
        return 1
    fi

    # 验证
    if command -v "$BINARY_NAME" &> /dev/null || [ -f "$INSTALL_DIR/$BINARY_NAME" ]; then
        log_success "安装验证通过"
    fi
}

# 源码编译安装
install_from_source() {
    log_info "开始从源码安装 SQLRustGo..."

    if ! check_rust; then
        install_rust
    fi

    # 确保 cargo 环境可用
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi

    # 克隆或更新仓库
    local repo_dir="/tmp/sqlrustgo-$VERSION"
    if [ ! -d "$repo_dir" ]; then
        log_info "克隆仓库..."
        git clone --depth 1 --branch "v$VERSION" "$REPO_URL" "$repo_dir" || {
            log_warn "版本 v$VERSION 不存在，使用 main 分支..."
            git clone --depth 1 "$REPO_URL" "$repo_dir"
        }
    fi

    cd "$repo_dir"

    log_info "正在编译..."
    cargo build --release

    # 创建安装目录
    mkdir -p "$INSTALL_DIR"

    # 安装
    cp "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    log_success "SQLRustGo 安装完成!"
    log_info "运行 'sqlrustgo' 启动数据库"
}

# 主安装流程
main() {
    local install_method=""

    # 解析参数
    while [ $# -gt 0 ]; do
        case "$1" in
            --cargo)
                install_method="cargo"
                shift
                ;;
            --binary)
                install_method="binary"
                shift
                ;;
            --source)
                install_method="source"
                shift
                ;;
            --version)
                VERSION="$2"
                shift 2
                ;;
            --install-dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                log_error "未知参数: $1"
                show_help
                exit 1
                ;;
        esac
    done

    # 检查依赖
    check_dependencies || exit 1

    # 交互式选择安装方式
    if [ -z "$install_method" ]; then
        echo ""
        echo "========================================"
        echo "     SQLRustGo 安装程序 v${VERSION}"
        echo "========================================"
        echo ""
        echo "请选择安装方式:"
        echo "  1) Cargo 安装 (推荐，需要 Rust 工具链)"
        echo "  2) 预编译二进制 (快速，但可能不是最新版本)"
        echo "  3) 源码编译 (获取最新开发版本)"
        echo "  4) 退出"
        echo ""
        read -p "请选择 [1-4]: " choice

        case "$choice" in
            1) install_method="cargo" ;;
            2) install_method="binary" ;;
            3) install_method="source" ;;
            4) exit 0 ;;
            *) log_error "无效选择"; exit 1 ;;
        esac
    fi

    # 执行安装
    echo ""
    case "$install_method" in
        cargo)
            install_cargo
            ;;
        binary)
            install_binary
            ;;
        source)
            install_from_source
            ;;
    esac

    # 提示信息
    echo ""
    echo "========================================"
    echo "  安装完成!"
    echo "========================================"
    echo ""
    echo "快速开始:"
    echo "  sqlrustgo              # 启动 REPL"
    echo "  sqlrustgo --help       # 查看帮助"
    echo ""
    echo "文档: $REPO_URL"
    echo ""
}

# 运行主函数
main "$@"
