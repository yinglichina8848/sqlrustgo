# SQLRustGo v2.6.0 开发指南

**版本**: v2.6.0 (生产就绪版本)
**发布日期**: 2026-04-17

---

## 一、开发环境搭建

### 1.1 系统要求

| 组件 | 最低要求 | 推荐配置 |
|------|----------|----------|
| CPU | 2 核 | 8 核+ |
| 内存 | 4 GB | 16 GB+ |
| 磁盘 | 10 GB | 50 GB+ SSD |
| 操作系统 | macOS/Linux/Windows | macOS/Linux |

### 1.2 安装 Rust

```bash
# 安装 Rust (如果尚未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 验证安装
rustc --version
# 输出: rustc 1.75.0 或更高

# 更新到最新
rustup update
```

### 1.3 克隆代码

```bash
# 克隆仓库
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 切换到 v2.6.0
git checkout v2.6.0

# 查看版本
cat VERSION
# 输出: 2.6.0
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
```

---

## 二、项目结构

### 2.1 目录结构

```
sqlrustgo/
├── src/                      # 主程序入口
├── crates/
│   ├── parser/              # SQL 解析器
│   ├── planner/             # 查询规划器
│   ├── optimizer/           # 查询优化器
│   ├── executor/            # 执行引擎
│   ├── storage/             # 存储引擎
│   ├── transaction/         # 事务管理
│   ├── catalog/             # 元数据管理
│   ├── types/               # 数据类型
│   ├── vector/              # 向量索引
│   ├── graph/               # 图引擎
│   ├── unified-query/       # 统一查询
│   ├── agentsql/            # Agent 框架
│   ├── server/              # 服务器
│   ├── bench/               # 基准测试
│   └── tools/               # 工具
├── tests/                   # 集成测试
├── benches/                # 基准测试
├── docs/                   # 文档
└── Cargo.toml              # 工作空间配置
```

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

---

## 四、模块开发指南

### 4.1 新增 Crate

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
```

### 4.2 添加新功能

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

---

## 五、调试技巧

### 5.1 日志调试

```rust
use tracing::{info, warn, error};

info!("Starting operation");
warn!("Potential issue: {:?}", context);
error!("Operation failed: {:?}", err);
```

运行时启用日志:

```bash
RUST_LOG=debug cargo run
RUST_LOG=sqlrustgo_executor=trace cargo run
```

### 5.2 断点调试

使用 LLDB:

```bash
# 启动调试
lldb -- target/debug/sqlrustgo

# 设置断点
(lldb) breakpoint set --name my_function

# 运行
(lldb) run
```

---

## 六、PR 流程

### 6.1 创建 PR

1. 创建分支:

```bash
git checkout -b feat/my-feature
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

### 6.2 PR 检查清单

- [ ] 代码编译通过
- [ ] 测试通过
- [ ] 代码格式化
- [ ] 文档已更新
- [ ] 变更日志已更新

---

## 七、相关文档

| 文档 | 说明 |
|------|------|
| [TEST_MANUAL.md](./TEST_MANUAL.md) | 测试手册 |
| [TEST_PLAN.md](./TEST_PLAN.md) | 测试计划 |
| [ARCHITECTURE_V2.6.md](./oo/architecture/ARCHITECTURE_V2.6.md) | 架构设计 |

---

*开发指南 v2.6.0*
*最后更新: 2026-04-17*
