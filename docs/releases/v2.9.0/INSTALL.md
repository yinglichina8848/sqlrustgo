# 安装指南

> **版本**: v2.9.0
> **代号**: Enterprise Resilience
> **发布日期**: 2026-05-05

---

## 1. 前置要求

### 1.1 系统要求

| 要求 | 最低版本 | 推荐版本 |
|------|----------|----------|
| Rust | 1.85+ | 1.94.1 |
| Cargo | 最新版 | 1.85.0 |
| 操作系统 | macOS / Linux | Ubuntu 22.04 / macOS 14 |
| 内存 | 4GB | 16GB+ |
| 磁盘 | 5GB | 50GB+ |

### 1.2 必需依赖

| 依赖 | 版本 | 说明 |
|------|------|------|
| OpenSSL | 1.1+ | TLS 加密支持 |
| CMake | 3.10+ | 编译构建 |
| LLVM | 15+ | 覆盖率工具 |

### 1.3 可选依赖

| 依赖 | 用途 |
|------|------|
| Valgrind | 内存检测 |
| perf | 性能分析 |

---

## 2. 从源码安装

### 2.1 克隆仓库

```bash
git clone git@192.168.0.252:openclaw/sqlrustgo.git
cd sqlrustgo
```

### 2.2 切换版本

```bash
git checkout v2.9.0
```

### 2.3 编译

```bash
# Debug 构建（快速编译）
cargo build

# Release 构建（生产环境）
cargo build --release

# 仅构建核心二进制
cargo build --release --bin sqlrustgo
```

### 2.4 安装

```bash
# 安装到 ~/.cargo/bin
cargo install --path .
```

---

## 3. Docker 安装

### 3.1 使用预构建镜像

```bash
docker pull ghcr.io/openclaw/sqlrustgo:v2.9.0

docker run -d \
  --name sqlrustgo \
  -p 5432:5432 \
  -v /data/sqlrustgo:/data \
  ghcr.io/openclaw/sqlrustgo:v2.9.0
```

### 3.2 从源码构建镜像

```bash
docker build -t sqlrustgo:v2.9.0 .
```

---

## 4. 验证安装

### 4.1 检查版本

```bash
sqlrustgo --version
# 输出: sqlrustgo 2.9.0
```

### 4.2 快速测试

```bash
cargo run --bin sqlrustgo -- -e "SELECT 1+1 AS result"
# 输出: +-------+
#       | result|
#       +-------+
#       | 2     |
#       +-------+
```

### 4.3 运行测试套件

```bash
cargo test --all-features
```

---

## 5. 配置

### 5.1 初始化数据目录

```bash
mkdir -p data logs
```

### 5.2 启动服务

```bash
cargo run --release --bin sqlrustgo
```

服务默认监听 `localhost:5432`。

---

## 6. 卸载

```bash
# 删除二进制
cargo uninstall sqlrustgo

# 删除数据目录（谨慎！）
rm -rf data/
```

---

*本文档由 Hermes Agent 维护*
*更新日期: 2026-05-05*
