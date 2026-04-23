# 安装指南

> **版本**: v2.8.0
> **代号**: Production+Distributed+Secure
> **发布日期**: (待定)

---

## 1. 前置要求

### 1.1 系统要求

| 要求 | 最低版本 |
|------|----------|
| Rust | 1.85+ |
| Cargo | 最新版 |
| 操作系统 | macOS / Linux / Windows |

### 1.2 硬件要求

| 资源 | 最低配置 | 推荐配置 |
|------|----------|----------|
| 内存 | 4GB | 16GB+ |
| 磁盘 | 10GB | 50GB+ |
| CPU | 2 核 | 4 核+ |

---

## 2. 安装步骤

### 2.1 安装 Rust

```bash
# 安装 Rust (如果没有)
curl --proto '=https' --tls v1.2 -sSf https://sh.rustup.rs | sh

# 加载环境变量
source ~/.cargo/env

# 验证安装
rustc --version
```

### 2.2 克隆项目

```bash
# 克隆仓库
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 切换到 v2.8.0 分支
git checkout develop/v2.8.0
```

### 2.3 构建

```bash
# Debug 构建 (快速开发)
cargo build

# Release 构建 (生产部署)
cargo build --release

# 全特性构建 (包含所有功能)
cargo build --all-features
```

---

## 3. 验证安装

### 3.1 运行测试

```bash
# 运行所有测试
cargo test --all-features

# 运行 SQL Corpus 测试
cargo test -p sqlrustgo-sql-corpus

# 运行 clippy 检查
cargo clippy --all-features -- -D warnings
```

### 3.2 启动 REPL

```bash
cargo run --release --bin sqlrustgo
```

---

## 4. Docker 安装 (可选)

### 4.1 构建 Docker 镜像

```bash
# 构建镜像
docker build -t sqlrustgo:v2.8.0 .

# 运行容器
docker run -p 3306:3306 -p 8080:8080 sqlrustgo:v2.8.0
```

### 4.2 使用预构建镜像

```bash
# 从 Docker Hub 拉取 (如有)
docker pull minzuuniversity/sqlrustgo:v2.8.0
```

---

## 5. 依赖项

### 5.1 运行时依赖

| 依赖 | 说明 | 必需 |
|------|------|------|
| libc | 系统库 | 是 |
| libssl | OpenSSL | SSL 连接需要 |
| libclang | Rust bindings | 部分功能需要 |

### 5.2 安装依赖 (Linux)

```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev libclang-dev

# CentOS/RHEL
sudo yum install gcc pkgconfig openssl-devel clang-devel
```

### 5.3 安装依赖 (macOS)

```bash
# 使用 Homebrew
brew install llvm
export LIBCLANG_PATH=$(brew --prefix llvm)/lib
```

---

## 6. 故障排查

### 6.1 构建失败

```bash
# 清理构建缓存
cargo clean

# 重新构建
cargo build --all-features
```

### 6.2 测试失败

```bash
# 运行单个测试
cargo test --all-features <test_name>

# 查看详细错误
RUST_BACKTRACE=1 cargo test --all-features <test_name>
```

---

## 7. 卸载

```bash
# 删除项目目录
rm -rf ~/sqlrustgo

# 或保留数据目录
# rm -rf ~/.sqlrustgo
```

---

## 8. 相关文档

- [快速开始](./QUICK_START.md)
- [客户端连接](./CLIENT_CONNECTION.md)
- [用户手册](./user-guide/USER_MANUAL.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-23*
