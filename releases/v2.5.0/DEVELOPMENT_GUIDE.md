# SQLRustGo v2.5.0 开发指南

**版本**: v2.5.0 (里程碑版本)
**发布日期**: 2026-04-16

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

# 安装 nightly (如需要特定功能)
rustup toolchain install nightly
```

### 1.3 克隆代码

```bash
# 克隆仓库
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 切换到 v2.5.0
git checkout develop/v2.5.0

# 查看版本
cat VERSION
# 输出: 2.5.0
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

### 1.5 IDE 设置

#### VS Code

安装扩展:
- rust-analyzer
- CodeLLDB
- Even Better TOML

创建 `.vscode/settings.json`:

```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": ["all"],
    "editor.formatOnSave": true,
    "editor.rulers": [100]
}
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

### 2.2 Crate 依赖关系

```
┌─────────────────────────────────────────────────────────────┐
│                    sqlrustgo-cli/server                      │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│ sqlrustgo-    │   │ sqlrustgo-    │   │ sqlrustgo-    │
│ executor      │   │ unified-query │   │ bench         │
└───────┬───────┘   └───────┬───────┘   └───────────────┘
        │                   │
        └───────────┬───────┘
                    ▼
        ┌─────────────────────┐
        │ parser | planner |   │
        │ optimizer | storage  │
        └─────────┬───────────┘
                  │
    ┌─────────────┼─────────────┐
    ▼             ▼             ▼
┌────────┐   ┌────────┐   ┌────────┐
│trans-  │   │catalog │   │ types   │
│action  │   │        │   │         │
└────────┘   └────────┘   └────────┘
```

---

## 三、代码规范

### 3.1 Rust 代码规范

遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

```rust
// 1. 模块组织
mod my_module {
    // 公开结构
    pub struct PublicStruct {
        pub field: PublicType,
       (crate) internal: InternalType,
        private: PrivateType,
    }

    // 公开函数
    pub fn public_function() -> Result<Output, Error> {
        // 实现
    }

    // 私有函数
    fn private_function() {
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

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MyError::NotFound(s) => write!(f, "Not found: {}", s),
            MyError::InvalidInput(s) => write!(f, "Invalid input: {}", s),
            MyError::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for MyError {}

// 3. 文档注释
/// MyStruct 是我的数据结构
///
/// # Examples
///
/// ```
/// let s = MyStruct::new();
/// ```
pub struct MyStruct {
    /// 字段说明
    pub field: Type,
}
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
| Trait | PascalCase | `MyTrait` |

### 3.3 测试规范

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_case() {
        let result = my_function(1, 2);
        assert_eq!(result, 3);
    }

    #[test]
    fn test_error_case() {
        let result = my_function(0, 0);
        assert!(result.is_err());
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_panic_case() {
        my_function(1, 0);
    }
}
```

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

3. 添加到工作空间 `Cargo.toml`:

```toml
[workspace]
members = [
    "crates/my-crate",
]
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
pub struct MyFeature {
    // 字段
}

impl MyFeature {
    pub fn new() -> Self {
        Self { }
    }

    pub fn process(&self, input: Input) -> Result<Output, Error> {
        // 实现
    }
}
```

3. 添加测试:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_feature() {
        let feature = MyFeature::new();
        // 测试代码
    }
}
```

### 4.3 数据库模块开发

#### 实现新的 SQL 语法

1. 在 `parser/src/` 添加语法定义:

```rust
// crates/parser/src/ast.rs
pub enum Statement {
    // ... existing statements
    MyNewStatement(MyNewStatement),
}

pub struct MyNewStatement {
    pub param: Type,
}
```

2. 在 `parser/src/parser.rs` 添加解析:

```rust
fn parse_my_statement(&mut self) -> ParseResult<Statement> {
    // 解析逻辑
}
```

#### 实现新的执行器

1. 在 `executor/src/` 添加执行器:

```rust
// crates/executor/src/my_executor.rs
pub struct MyExecutor {
    // 字段
}

impl MyExecutor {
    pub fn new() -> Self {
        Self { }
    }

    pub fn execute(&mut self) -> Result<Row, ExecutorError> {
        // 执行逻辑
    }
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
(lldb) breakpoint set --file my_file.rs --line 42

# 运行
(lldb) run

# 查看变量
(lldb) frame variable
(lldb) p my_variable
```

### 5.3 性能分析

```bash
# 使用 perf
perf record --call-graph dwarf cargo run --release
perf report

# 使用 flamegraph
cargo install cargo-flamegraph
cargo flamegraph --bin sqlrustgo-server
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
git commit -m "feat: add my feature

- Add new functionality
- Add tests
- Update documentation"
```

3. 推送并创建 PR:

```bash
git push origin feat/my-feature
```

### 6.2 PR 模板

```markdown
## 描述
<!-- 简要描述这个 PR -->

## 类型
- [ ] Bug 修复
- [ ] 新功能
- [ ] 文档更新
- [ ] 重构
- [ ] 测试

## 相关 Issue
<!-- 关联的 Issue 编号 -->

## 检查清单
- [ ] 代码编译通过
- [ ] 测试通过
- [ ] 代码格式化
- [ ] 文档已更新
- [ ] 变更日志已更新

## 测试结果
<!-- 测试输出 -->
```

### 6.3 代码审查

审查检查清单:
- [ ] 代码风格符合规范
- [ ] 有适当的测试覆盖
- [ ] 文档已更新
- [ ] 没有引入安全问题
- [ ] 性能没有明显下降
- [ ] 错误处理完善

---

## 七、相关文档

| 文档 | 说明 |
|------|------|
| [TEST_MANUAL.md](./TEST_MANUAL.md) | 测试手册 |
| [API_DOCUMENTATION.md](./oo/api/API_DOCUMENTATION.md) | API 文档 |
| [ARCHITECTURE_V2.5.md](./oo/architecture/ARCHITECTURE_V2.5.md) | 架构设计 |

---

*开发指南 v2.5.0*
*最后更新: 2026-04-16*
