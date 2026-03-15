#!/bin/bash
#
# SQLRustGo v1.3.0 安装脚本
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
VERSION="1.3.0"
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
SQLRustGo v${VERSION} 安装脚本

用法:
  $0 [选项]

选项:
  --cargo      使用 Cargo 从源码安装
  --binary     使用预编译二进制安装
  --docker     使用 Docker 安装
  --help       显示此帮助信息

示例:
  $0              # 交互式安装
  $0 --cargo      # Cargo 安装
  $0 --binary     # 二进制安装
  $0 --docker     # Docker 安装

更多信息请访问: ${REPO_URL}
EOF
}

check_requirements() {
    log_info "检查系统要求..."

    if [ "$EUID" -eq 0 ]; then
        INSTALL_DIR="/usr/local/bin"
    fi

    if [ ! -d "$INSTALL_DIR" ]; then
        log_info "创建安装目录: $INSTALL_DIR"
        mkdir -p "$INSTALL_DIR"
    fi

    log_success "系统要求检查完成"
}

install_cargo() {
    log_info "使用 Cargo 从源码安装..."

    if ! command -v cargo &> /dev/null; then
        log_error "Cargo 未安装，请先安装 Rust: https://rustup.rs/"
        exit 1
    fi

    log_info "克隆仓库..."
    TEMP_DIR=$(mktemp -d)
    git clone --depth 1 --branch v${VERSION} ${REPO_URL}.git "$TEMP_DIR" 2>/dev/null || \
    git clone --depth 1 ${REPO_URL}.git "$TEMP_DIR"

    cd "$TEMP_DIR"

    log_info "构建 Release 版本..."
    cargo build --release

    log_info "安装二进制文件..."
    cp "target/release/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    cd /
    rm -rf "$TEMP_DIR"

    log_success "安装完成!"
    log_info "运行: ${INSTALL_DIR}/${BINARY_NAME}"
}

install_binary() {
    log_info "使用预编译二进制安装..."

    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$ARCH" in
        x86_64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="arm64"
            ;;
        *)
            log_error "不支持的架构: $ARCH"
            exit 1
            ;;
    esac

    FILENAME="sqlrustgo-v${VERSION}-${OS}-${ARCH}"
    DOWNLOAD_URL="${REPO_URL}/releases/download/v${VERSION}/${FILENAME}.tar.gz"

    log_info "下载: $FILENAME"
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    if command -v curl &> /dev/null; then
        curl -L -o "${FILENAME}.tar.gz" "$DOWNLOAD_URL"
    elif command -v wget &> /dev/null; then
        wget -O "${FILENAME}.tar.gz" "$DOWNLOAD_URL"
    else
        log_error "需要 curl 或 wget"
        exit 1
    fi

    log_info "解压..."
    tar -xzf "${FILENAME}.tar.gz"

    log_info "安装..."
    cp -r "${FILENAME}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    cd /
    rm -rf "$TEMP_DIR"

    log_success "安装完成!"
    log_info "运行: ${INSTALL_DIR}/${BINARY_NAME}"
}

install_docker() {
    log_info "使用 Docker 安装..."

    if ! command -v docker &> /dev/null; then
        log_error "Docker 未安装，请先安装 Docker: https://docker.com/"
        exit 1
    fi

    log_info "拉取镜像..."
    docker pull minzuuniversity/sqlrustgo:v${VERSION}

    log_success "Docker 镜像拉取完成!"
    log_info "运行 REPL: docker run -it minzuuniversity/sqlrustgo:v${VERSION}"
    log_info "运行服务器: docker run -p 5432:5432 minzuuniversity/sqlrustgo:v${VERSION} --server"
}

main() {
    echo ""
    echo "========================================"
    echo "  SQLRustGo v${VERSION} 安装程序"
    echo "========================================"
    echo ""

    case "${1:-}" in
        --cargo)
            check_requirements
            install_cargo
            ;;
        --binary)
            check_requirements
            install_binary
            ;;
        --docker)
            install_docker
            ;;
        --help)
            show_help
            ;;
        "")
            show_help
            echo ""
            echo "请选择安装方式:"
            echo "  1) Cargo (从源码编译)"
            echo "  2) Binary (预编译二进制)"
            echo "  3) Docker"
            echo "  0) 退出"
            echo ""
            read -p "请输入选项 [1-3]: " choice
            case "$choice" in
                1)
                    check_requirements
                    install_cargo
                    ;;
                2)
                    check_requirements
                    install_binary
                    ;;
                3)
                    install_docker
                    ;;
                0)
                    log_info "退出安装"
                    exit 0
                    ;;
                *)
                    log_error "无效选项"
                    exit 1
                    ;;
            esac
            ;;
        *)
            log_error "未知选项: $1"
            show_help
            exit 1
            ;;
    esac

    echo ""
    echo "========================================"
    echo "  安装完成！"
    echo "========================================"
    echo ""
}

main "$@"
