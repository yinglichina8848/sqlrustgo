# 安装指南

> **版本**: alpha/v2.6.0

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
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

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
cargo test --workspace

# 运行 SQL Corpus 测试
cargo test -p sqlrustgo-sql-corpus
```

### 3.2 启动 REPL

```bash
# 启动交互式 SQL 终端
cargo run --release

# 或者
./target/release/sqlrustgo
```

---

## 4. 依赖项

### 4.1 Rust 工具链

```bash
# 确保工具链是最新的
rustup update

# 检查版本
rustc --version  # 应该 >= 1.85
cargo --version
```

### 4.2 系统依赖

**macOS:**
```bash
# 通常无需额外安装
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev
```

**Windows:**
```bash
# 安装 Visual Studio Build Tools
# 或使用 MSYS2
```

---

## 5. 常见问题

### Q1: 编译失败

确保 Rust 版本 >= 1.85:
```bash
rustc --version
```

### Q2: 依赖下载慢

使用国内镜像:
```bash
# 添加 Rust 中国镜像
export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
```

### Q3: 内存不足

减少并行编译:
```bash
cargo build -j 2
```

---

## 6. 卸载

```bash
# 卸载 Rust
rustup self uninstall
```

---

## 7. 下一步

- [快速开始](./QUICK_START.md)
- [用户手册](./oo/user-guide/USER_MANUAL.md)
- [升级指南](./UPGRADE_GUIDE.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-18*
