# 开发者文档

## 开发环境搭建

### 环境要求

- **Rust**: 1.75+
- **Cargo**: 随 Rust 安装
- **Git**: 版本控制

### 快速开始

```bash
# 1. 克隆项目
git clone https://github.com/yinglichina8848/sqlrustgo.git
cd sqlrustgo

# 2. 使用 rustup 安装 Rust (如未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 3. 验证 Rust 安装
rustc --version
cargo --version

# 4. 构建项目
cargo build --all-features

# 5. 运行测试
cargo test --all-features

# 6. 运行 REPL
cargo run
```

## 项目架构

### 模块概览

```
src/
├── executor/      # SQL 执行引擎 (Volcano 模型)
├── parser/       # SQL 解析器 (递归下降)
├── lexer/        # 词法分析器
├── storage/      # 存储层
│   ├── buffer_pool.rs  # 缓冲池管理
│   ├── page.rs        # 页面管理
│   └── bplus_tree/    # B+ 树索引
├── transaction/  # 事务层
│   ├── wal.rs          # Write-Ahead Log
│   └── manager.rs      # 事务管理器
├── network/      # 网络层 (MySQL 协议)
└── types/        # 类型系统
    ├── value.rs       # SQL 值类型
    └── error.rs       # 错误定义
```

### 数据流

```
用户输入 → Lexer → Parser → Executor → Storage
                ↓                        ↓
           Statement AST           B+ Tree / Buffer Pool
                ↓
           ExecutionResult ← TransactionManager ← WAL
```

## 代码规范

### 提交规范

使用 Conventional Commits:

```
<type>: <description>

[optional body]
```

**类型 (type)**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `test`: 测试相关
- `refactor`: 代码重构
- `perf`: 性能优化
- `chore`: 构建/工具

**示例**:
```bash
git commit -m "feat: 添加 B+ 树范围查询功能"
git commit -m "fix: 修复解析器对空表的处理"
git commit -m "docs: 更新 README 安装说明"
```

### 代码风格

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy --all-features -- -D warnings` 检查
- 遵循 Rust 官方代码规范
- 添加 What-Why-How 文档注释

### 测试规范

- 单元测试放在 `mod tests` 中
- 集成测试放在 `tests/` 目录
- 目标覆盖率: 80%+
- 新功能必须包含测试

## 测试指南

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块
cargo test --lib executor

# 运行集成测试
cargo test --test integration_test

# 运行特定测试
cargo test test_name --all-features
```

### 覆盖率

```bash
# 安装 llvm-cov
cargo install cargo-llvm-cov

# 生成覆盖率报告
cargo llvm-cov --all-features --all --workspace --html

# 查看文本报告
cargo llvm-cov --all-features --all --workspace --text
```

### 添加新测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature() {
        // Arrange
        let input = "...";

        // Act
        let result = process(input);

        // Assert
        assert!(result.is_ok());
    }
}
```

## 调试技巧

### 日志调试

```rust
use log::{info, debug, error};

fn some_function() {
    debug!("Entering function with value: {}", value);
    // ... logic
    info!("Operation completed successfully");
}
```

### 运行时日志

```bash
# 设置日志级别
RUST_LOG=debug cargo run

# 仅显示特定模块
RUST_LOG=sqlrustgo::executor=trace cargo run
```

### GDB/LLDB 调试

```bash
# 编译调试版本
cargo build

# 启动调试器
lldb target/debug/sqlrustgo

# 或使用 rust-gdb
rust-gdb target/debug/sqlrustgo
```

## 贡献指南

### 贡献流程

1. **Fork** 项目
2. 创建特性分支: `git checkout -b feature/xxx`
3. 编写代码和测试
4. 提交更改: `git commit -m "feat: ..."`
5. 推送分支: `git push origin feature/xxx`
6. 创建 Pull Request

### 代码审查要点

- [ ] 代码逻辑正确
- [ ] 有适当的测试覆盖
- [ ] 通过 clippy 检查
- [ ] 代码格式化
- [ ] 文档注释完整

## 常见问题

### 编译错误

```bash
# 清理并重新编译
cargo clean
cargo build --all-features
```

### 测试失败

```bash
# 运行单个测试查看详细输出
cargo test test_name --all-features -- --nocapture
```

### 依赖问题

```bash
# 更新依赖
cargo update

# 检查依赖安全性
cargo audit
```
