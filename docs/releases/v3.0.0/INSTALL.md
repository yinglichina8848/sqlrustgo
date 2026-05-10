# SQLRustGo v3.0.0 安装指南

> **版本**: v3.0.0
> **发布日期**: 2026-05-07
> **阶段**: GA (General Availability)

---

## 一、系统要求

### 1.1 硬件要求

| 组件 | 最低要求 | 推荐配置 |
|------|----------|----------|
| CPU | 2 核心 | 4+ 核心 |
| 内存 | 4 GB | 8+ GB |
| 磁盘 | 10 GB 可用空间 | 20+ GB SSD |
| 操作系统 | macOS 12+ / Linux (glibc 2.17+) / Windows (WSL2) | macOS 13+ / Ubuntu 22.04+ |

### 1.2 软件依赖

| 依赖 | 版本要求 | 说明 |
|------|----------|------|
| Rust | 1.85+ | 使用 rustup 安装 |
| Cargo | 最新稳定版 | Rust 包管理器 |
| LLVM | 15+ | 用于覆盖率测试 (llvm-cov) |
| Python | 3.10+ | 用于 TPC-H 工具 |

---

## 二、快速安装

### 2.1 使用 Cargo 安装（从源码）

```bash
# 克隆仓库
git clone http://192.168.0.252:3000/openclaw/sqlrustgo.git
cd sqlrustgo

# 切换到 v3.0.0 分支
git checkout develop/v3.0.0

# 构建所有特性
cargo build --all-features --release

# 运行测试
cargo test --all-features

# 启动 REPL
cargo run --bin sqlrustgo
```

### 2.2 使用预编译二进制

> 预编译二进制将在 GA 正式发布后提供。

```bash
# 下载对应平台的二进制
curl -L https://example.com/sqlrustgo-v3.0.0-x86_64-unknown-linux-gnu.tar.gz | tar xz

# 运行
./sqlrustgo --version
```

---

## 三、开发环境安装

### 3.1 Rust 环境

```bash
# 安装 Rust (如果尚未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 验证安装
rustc --version
cargo --version

# 更新到最新稳定版
rustup update stable
```

### 3.2 LLVM 环境

```bash
# macOS
brew install llvm

# Ubuntu/Debian
sudo apt-get install llvm-15 clang-15

# 验证 llvm-cov
llvm-cov --version
```

### 3.3 Python 环境 (用于 TPC-H)

```bash
# 使用 pyenv (推荐)
curl https://pyenv.run | bash
pyenv install 3.11
pyenv global 3.11

# 验证
python3 --version
```

### 3.4 工具链验证

```bash
# 验证所有工具
./scripts/gate/check_tools.sh

# 输出示例:
# rustc: 1.85.0
# cargo: 1.85.0
# llvm-cov: 15.0.0
# python: 3.11.0
```

---

## 四、IDE 配置

### 4.1 VS Code

安装扩展:
- `rust-analyzer` - Rust 语言支持
- `CodeLLDB` - 调试支持
- `crates` - Rust 依赖管理

配置 `settings.json`:

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.allTargets": true,
  "rust-analyzer.cargo.features": ["all-features"]
}
```

### 4.2 IntelliJ IDEA / CLion

安装插件:
- Rust plugin (IntelliJ Rust)

配置:
- Enable "All features" in Rust toolchain settings

### 4.3 Neovim

```lua
-- 使用 nvim-lspconfig
require('lspconfig').rust_analyzer.setup({
  settings = {
    ['rust-analyzer'] = {
      checkOnSave = { command = 'clippy' },
      cargo = { allFeatures = true }
    }
  }
})
```

---

## 五、依赖说明

### 5.1 Workspace 结构

```
sqlrustgo/
├── Cargo.toml          # Workspace 配置
├── crates/
│   ├── parser/         # SQL 解析器
│   ├── planner/        # 查询规划器
│   ├── optimizer/      # 查询优化器
│   ├── executor/       # 查询执行器
│   ├── storage/        # 存储引擎
│   ├── transaction/    # 事务管理
│   ├── network/        # 网络协议
│   └── ...
└── scripts/            # 工具脚本
```

### 5.2 特性标志

| 特性 | 说明 | 默认启用 |
|------|------|----------|
| `full` | 所有特性 | ✅ |
| `vector` | 向量存储 | ✅ |
| `graph` | 图存储 | ✅ |
| `cloud` | 云存储支持 | ❌ |

---

## 六、常见问题

### Q1: 编译失败，提示 "linker `cc` not found"

```bash
# macOS
xcode-select --install

# Ubuntu
sudo apt-get install build-essential
```

### Q2: llvm-cov 找不到

```bash
# macOS: 确保 LLVM 路径正确
export PATH="/opt/homebrew/opt/llvm/bin:$PATH"

# 或使用 rustup 安装
rustup component add llvm-tools-preview
```

### Q3: 测试超时

某些集成测试可能需要较长时间。使用 `--test-threads=1` 限制并发:

```bash
cargo test --all-features -- --test-threads=1
```

### Q4: Windows 上的问题

推荐使用 WSL2:

```powershell
wsl --install -d Ubuntu-22.04
```

---

## 七、验证安装

### 7.1 基本验证

```bash
# 验证构建
cargo build --all-features --release
./target/release/sqlrustgo --version

# 验证测试
cargo test --all-features -- --test-threads=4
```

### 7.2 门禁验证

```bash
# 运行 GA 门禁检查
bash scripts/gate/check_ga_v300.sh

# 预期输出:
# === v3.0.0 GA Gate ===
# GA-1: cargo build --release ... PASS
# GA-2: cargo test --all-features ... PASS
# ...
# === GA Gate Results: PASS=15 / 15, BLOCKERS=0 ===
```

---

## 八、卸载

```bash
# 删除构建产物
cargo clean

# 删除整个项目
cd ..
rm -rf sqlrustgo
```

---

## 相关文档

- [快速开始](./QUICK_START.md)
- [部署指南](./DEPLOYMENT_GUIDE.md)
- [发布说明](./RELEASE_NOTES.md)

---

*最后更新: 2026-05-07*
