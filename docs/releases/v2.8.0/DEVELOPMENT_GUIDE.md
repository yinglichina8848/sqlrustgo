# SQLRustGo v2.8.0 开发指南

> **版本**: v2.8.0
> **代号**: Production+Distributed+Secure
> **最后更新**: 2026-04-30

---

## 一、开发环境搭建

### 1.1 系统要求

| 组件 | 最低要求 | 推荐配置 |
|------|----------|----------|
| CPU | 2 核 | 8 核+ (AVX2/AVX-512 for SIMD) |
| 内存 | 4 GB | 16 GB+ |
| 磁盘 | 10 GB | 50 GB+ SSD |
| 操作系统 | macOS/Linux/Windows | macOS/Linux |

### 1.2 安装 Rust

```bash
# 安装 Rust (如果尚未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 验证安装
rustc --version
# 输出: rustc 1.85.0 或更高

# 更新到最新
rustup update
```

### 1.3 克隆代码

```bash
# 克隆仓库
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 切换到 v2.8.0
git checkout develop/v2.8.0

# 查看版本
cat VERSION
# 输出: 2.8.0
```

### 1.4 编译项目

```bash
# Debug 编译 (快速，用于开发)
cargo build

# Release 编译 (优化，用于部署)
cargo build --release

# 仅编译指定 crate
cargo build -p sqlrustgo-parser
cargo build -p sqlrustgo-executor

# 使用所有特性编译
cargo build --all-features

# 编译 MySQL 服务器
cargo build --release -p sqlrustgo-mysql-server
```

---

## 二、项目结构

### 2.1 工作空间结构

```
sqlrustgo/
├── crates/
│   ├── parser/              # SQL 解析器
│   │   └── src/
│   │       ├── parser.rs    # 解析器入口
│   │       ├── sql.lalrpop # LALRPOP 语法定义
│   │       └── ast.rs       # AST 定义
│   ├── planner/             # 查询规划器
│   ├── optimizer/            # 查询优化器 (CBO)
│   ├── executor/             # 执行引擎
│   │   └── src/
│   │       ├── engine.rs    # ExecutionEngine
│   │       ├── join.rs      # JOIN 执行器
│   │       └── agg.rs       # 聚合执行器
│   ├── storage/              # 存储引擎
│   │   └── src/
│   │       ├── buffer_pool.rs   # Buffer Pool
│   │       ├── btree.rs         # B+ Tree 索引
│   │       └── wal.rs           # WAL
│   ├── transaction/          # 事务管理
│   │   └── src/
│   │       ├── mvcc.rs       # MVCC 实现
│   │       └── ssi.rs        # SSI 隔离
│   ├── catalog/              # 元数据管理
│   ├── types/                # 数据类型
│   │   └── src/
│   │       ├── value.rs      # SqlValue
│   │       ├── tribool.rs     # TriBool 三值逻辑 (v2.8.0)
│   │       └── mod.rs
│   ├── vector/               # 向量索引
│   │   └── src/
│   │       ├── hnsw.rs       # HNSW 索引
│   │       ├── ivfpq.rs      # IVF-PQ 索引
│   │       └── simd_explicit.rs  # SIMD 向量化 (v2.8.0)
│   ├── graph/                # 图引擎 (GMP)
│   ├── unified-query/        # 统一查询
│   ├── network/              # 网络层
│   │   └── src/
│   │       ├── mysql/        # MySQL 协议
│   │       │   ├── server.rs # MySQL 服务器
│   │       │   ├── auth.rs   # 认证
│   │       │   └── command.rs# 命令处理
│   │       └── http/         # REST API
│   ├── server/               # 服务器入口
│   ├── security/             # 安全模块 (v2.8.0)
│   │   └── src/
│   │       ├── audit.rs      # 审计日志
│   │       ├── firewall.rs   # SQL 防火墙
│   │       └── privilege.rs  # 权限控制
│   ├── replication/          # 复制模块 (v2.8.0)
│   │   └── src/
│   │       ├── gtid.rs       # GTID 协议
│   │       ├── semi_sync.rs  # 半同步复制
│   │       └── failover.rs   # 故障转移
│   ├── bench/                # 基准测试
│   └── tools/                # 工具
├── tests/                    # 集成测试
├── benches/                  # 性能基准
├── docs/                     # 文档
└── Cargo.toml                # 工作空间配置
```

### 2.2 新增 Crate (v2.8.0)

| Crate | 说明 | PR |
|-------|------|-----|
| `sqlrustgo-security` | 安全模块 | - |
| `sqlrustgo-replication` | 复制模块 | #78 |
| `sqlrustgo-simd` | SIMD 向量化 | #32 |

---

## 三、代码规范

### 3.1 Rust 代码规范

遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

```rust
// 1. 模块组织
mod my_module {
    pub struct PublicStruct {
        pub field: PublicType,
        (crate) internal: InternalType,
    }

    pub fn public_function() -> Result<Output, Error> {
        // 实现
    }
}

// 2. Error 处理
#[derive(Debug, Clone)]
pub enum MyError {
    NotFound(String),
    InvalidInput(String),
    Internal(Box<dyn std::error::Error>),
}

// 3. 文档注释
/// MyStruct 是我的数据结构
///
/// # Examples
///
/// ```
/// let s = MyStruct::new();
/// ```
pub struct MyStruct { }
```

### 3.2 命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| 模块 | snake_case | `my_module` |
| 结构体 | PascalCase | `MyStruct` |
| 枚举 | PascalCase | `MyEnum` |
| 函数 | snake_case | `my_function` |
| 变量 | snake_case | `my_variable` |
| 常量 | SCREAMING_SNAKE | `MY_CONSTANT` |

### 3.3 必须通过的检查

提交前必须通过以下检查:

```bash
# 格式化检查
cargo fmt --check --all

# Clippy 检查 (必须无警告)
cargo clippy --all-features -- -D warnings

# 文档链接检查
bash scripts/gate/check_docs_links.sh
```

---

## 四、常用 Cargo 命令

### 4.1 构建命令

```bash
# 编译整个项目
cargo build --all-features

# 编译单个 crate
cargo build -p sqlrustgo-parser

# 快速检查 (不生成二进制)
cargo check --all-features

# 发布构建
cargo build --release --all-features

# 构建 MySQL 服务器
cargo build --release -p sqlrustgo-mysql-server
```

### 4.2 测试命令

```bash
# 运行所有测试
cargo test --all-features

# 运行单个测试
cargo test <test_name> --all-features

# 运行文档测试
cargo test --doc

# 运行特定 crate 的测试
cargo test -p sqlrustgo-executor --all-features
cargo test -p sqlrustgo-parser --all-features
cargo test -p sqlrustgo-vector --all-features  # 包括 SIMD 测试

# 运行集成测试
cargo test --test '*' --all-features

# 运行带 coverage 的测试
cargo llvm-cov --all-features
```

### 4.3 代码质量

```bash
# 格式化代码
cargo fmt --all

# 运行 clippy
cargo clippy --all-features -- -D warnings

# 运行 clippy 并自动修复
cargo clippy --fix --all-features --allow-dirty --allow-staged

# 运行 miri 检查未定义行为
cargo miri test
```

### 4.4 其他常用命令

```bash
# 查看依赖树
cargo tree

# 运行 REPL
cargo run --bin sqlrustgo

# 生成文档
cargo doc --all-features --no-deps

# 运行基准测试
cargo bench --all-features

# 查看编译时间
cargo build --time
```

---

## 五、调试技巧

### 5.1 日志调试

```rust
use tracing::{info, warn, error, debug};

info!("Starting operation");
debug!("Debug info: {:?}", context);
warn!("Potential issue: {:?}", context);
error!("Operation failed: {:?}", err);
```

运行时启用日志:

```bash
RUST_LOG=debug cargo run
RUST_LOG=sqlrustgo_executor=trace cargo run
RUST_LOG=sqlrustgo_vector::simd=debug cargo run  # SIMD 调试
```

### 5.2 SIMD 调试

```bash
# 检测 SIMD 能力
cargo test -p sqlrustgo-vector -- detect_simd

# 运行 SIMD 性能测试
cargo bench -p sqlrustgo-vector -- simd_benchmark

# 查看 SIMD 加速比
RUST_LOG=info cargo run --bin sqlrustgo-vector-bench
```

### 5.3 断点调试

使用 LLDB:

```bash
# 启动调试
lldb -- target/debug/sqlrustgo

# 设置断点
(lldb) breakpoint set --name my_function
(lldb) breakpoint set --file executor.rs --line 100

# 运行
(lldb) run

# 单步执行
(lldb) step
(lldb) next
```

---

## 六、模块开发指南

### 6.1 新增 Crate

1. 创建 crate 目录:

```bash
mkdir -p crates/my-crate/src
```

2. 创建 `Cargo.toml`:

```toml
[package]
name = "sqlrustgo-my-crate"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlrustgo-types = { path = "../types" }
sqlrustgo-error = { path = "../error" }
```

3. 添加到工作空间 (`Cargo.toml`):

```toml
[workspace]
members = [
    # ...
    "crates/my-crate",
]
```

### 6.2 添加新功能

1. 在相应的 crate 中添加模块:

```rust
// crates/my-crate/src/lib.rs
pub mod my_feature;

pub use my_feature::MyFeature;
```

2. 实现功能:

```rust
// crates/my-crate/src/my_feature.rs
pub struct MyFeature { }

impl MyFeature {
    pub fn new() -> Self { Self { } }
    pub fn process(&self, input: Input) -> Result<Output, Error> { }
}
```

### 6.3 v2.8.0 新模块开发

#### 添加 SIMD 函数 (vector crate)

```rust
// crates/vector/src/simd_explicit.rs

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub fn l2_distance_simd(a: &[f32], b: &[f32]) -> f32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { l2_distance_avx2(a, b) };
        }
    }
    // Fallback to scalar
    l2_distance_scalar(a, b)
}

pub fn detect_simd_lanes() -> usize {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") { 16 }
        else if is_x86_feature_detected!("avx2") { 8 }
        else { 1 }
    }
    #[cfg(not(target_arch = "x86_64"))]
    { 1 }
}
```

#### 添加审计事件 (security crate)

```rust
// crates/security/src/audit.rs

#[derive(Debug, Clone)]
pub enum AuditEvent {
    Login { user: String, session_id: u64 },
    Logout { user: String, session_id: u64 },
    ExecuteSql { user: String, sql: String, duration_ms: u64 },
    DDL { user: String, statement: String },
    DML { user: String, statement: String },
    Grant { user: String, target: String, privilege: String },
    Revoke { user: String, target: String, privilege: String },
    Error { user: String, error: String },
}
```

---

## 七、PR 流程

### 7.1 创建 PR

1. 创建分支 (从 develop/v2.8.0):

```bash
git checkout -b feat/my-feature develop/v2.8.0
```

2. 提交代码:

```bash
git add .
git commit -m "feat: add my feature"
```

3. 推送并创建 PR:

```bash
git push origin feat/my-feature
```

### 7.2 PR 检查清单

- [ ] 代码编译通过 (`cargo build --all-features`)
- [ ] 测试通过 (`cargo test --all-features`)
- [ ] 代码格式化 (`cargo fmt --all`)
- [ ] Clippy 检查通过 (`cargo clippy --all-features -- -D warnings`)
- [ ] 文档已更新
- [ ] 变更日志已更新

### 7.3 Commit Message 规范

```
<type>(<scope>): <subject>

<body>

<footer>
```

类型:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式
- `refactor`: 重构
- `test`: 测试
- `perf`: 性能优化
- `ci`: CI/CD

示例:

```
feat(executor): add FULL OUTER JOIN support

Implement hash-based matching algorithm for FULL OUTER JOIN.
- Add HashJoin executor variant
- Add match_indicator array for left/right matches
- Update output generation for unmatched rows

Closes #1733
```

---

## 八、测试指南

### 8.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tribool_and() {
        assert_eq!(TriBool::True.and(TriBool::False), TriBool::False);
        assert_eq!(TriBool::True.and(TriBool::Unknown), TriBool::Unknown);
    }

    #[test]
    fn test_simd_l2_distance() {
        let a = vec![0.1; 128];
        let b = vec![0.2; 128];
        let result = l2_distance_simd(&a, &b);
        let expected = l2_distance_scalar(&a, &b);
        assert!((result - expected).abs() < 1e-6);
    }
}
```

### 8.2 集成测试

```bash
# 运行 SQL corpus 测试
cargo test -p sqlrustgo-sql-corpus

# 运行 TPC-H 测试
cargo test -p sqlrustgo-tpch

# 运行网络协议测试
cargo test -p sqlrustgo-network

# 运行 MySQL 兼容性测试
cargo test -p sqlrustgo-mysql-tests
```

### 8.3 Coverage

```bash
# 使用 tarpaulin (可能 segfault)
cargo tarpaulin --all-features --out xml --workspace

# 使用 cargo-llvm-cov (推荐)
cargo llvm-cov --all-features --lcov --output-path lcov.info
cargo llvm-cov report --open
```

---

## 九、CI/CD

### 9.1 GitHub Actions 工作流

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [develop/v2.8.0]
  pull_request:
    branches: [develop/v2.8.0]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable
      - name: Build
        run: cargo build --all-features
      - name: Test
        run: cargo test --all-features
      - name: Clippy
        run: cargo clippy --all-features -- -D warnings
      - name: Fmt
        run: cargo fmt --check --all
```

### 9.2 本地 CI 检查

```bash
# 运行完整的 CI 检查
bash scripts/ci/local-check.sh

# 或分别运行
cargo fmt --check --all
cargo clippy --all-features -- -D warnings
cargo test --all-features
```

---

## 十、相关文档

| 文档 | 说明 |
|------|------|
| [测试手册](./TEST_PLAN.md) | 测试计划 |
| [架构决策](../architecture.md) | ADR |
| [API 使用示例](./API_USAGE_EXAMPLES.md) | Rust API 示例 |
| [SIMD 基准报告](./SIMD_BENCHMARK_REPORT.md) | SIMD 性能数据 |

---

*开发指南 v2.8.0*
*最后更新: 2026-04-30*
