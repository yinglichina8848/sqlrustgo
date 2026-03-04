# SQLRustGo 开发指南

## 1. 开发环境

### 1.1 环境要求

- Rust 1.75+
- Cargo
- Git

### 1.2 本地开发

```bash
# 克隆项目
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 开发模式编译
cargo build

# 运行测试
cargo test

# 代码检查
cargo clippy

# 代码格式化
cargo fmt
```

## 2. 项目结构

```
src/
├── main.rs              # REPL 入口
├── lib.rs               # 库入口
├── lexer/               # 词法分析
│   ├── token.rs
│   └── lexer.rs
├── parser/              # 语法分析
│   └── mod.rs
├── executor/            # 查询执行
│   └── mod.rs
├── storage/             # 存储引擎
│   ├── mod.rs
│   ├── file_storage.rs
│   ├── buffer_pool.rs
│   ├── page.rs
│   └── bplus_tree/
├── transaction/         # 事务处理
│   ├── mod.rs
│   ├── wal.rs
│   └── manager.rs
├── network/             # 网络协议
│   └── mod.rs
└── types/               # 类型定义
    ├── mod.rs
    ├── value.rs
    └── error.rs
```

## 3. 添加新功能

### 3.1 添加新的 SQL 关键字

1. 在 `src/lexer/token.rs` 添加 Token 变体
2. 在 `src/lexer/lexer.rs` 的 `next_token` 方法中添加识别
3. 在 `src/parser/mod.rs` 添加解析逻辑
4. 在 `src/executor/mod.rs` 添加执行逻辑

### 3.2 添加新的数据类型

1. 在 `src/types/value.rs` 的 `Value` 枚举中添加变体
2. 实现相关 trait：`Clone`, `Debug`, `PartialEq`, `Serialize`
3. 在 parser 中添加解析支持
4. 在 executor 中添加执行支持

### 3.3 添加新的测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_new_feature() {
        // 测试代码
    }
}
```

## 4. 代码规范

### 4.1 提交规范

使用conventional commits：

```
feat: 添加新功能
fix: 修复 bug
docs: 文档更新
test: 测试更新
refactor: 代码重构
style: 代码格式
```

### 4.2 分支策略

- `main` - 生产分支
- `feature/*` - 功能开发分支
- `fix/*` - 修复分支

### 4.3 代码检查

提交前执行：

```bash
cargo fmt
cargo clippy
cargo test
```

## 5. 调试

### 5.1 日志调试

```rust
log::info!("message: {:?}", data);
log::error!("error: {}", e);
```

### 5.2 测试调试

```bash
# 运行单个测试
cargo test test_name

# 显示测试输出
cargo test -- --nocapture

# 运行文档测试
cargo test --doc
```

## 6. 发布

### 6.1 发布流程

1. 更新版本号 (`Cargo.toml`)
2. 运行完整测试
3. 创建 Git tag
4. 构建发布版本

```bash
cargo build --release
```

### 6.2 发布检查清单

- [ ] 所有测试通过
- [ ] Clippy 无警告
- [ ] 代码格式化
- [ ] 更新 CHANGELOG.md
- [ ] 创建 Git tag

## 7. 依赖管理

### 7.1 添加依赖

```bash
cargo add crate_name
```

### 7.2 更新依赖

```bash
cargo update
```

## 8. 性能优化

### 8.1 性能分析

```bash
cargo bench
```

### 8.2 覆盖率分析

```bash
cargo tarpaulin
```

## 9. 常见问题

### 9.1 编译错误

```bash
# 清理并重新编译
cargo clean
cargo build
```

### 9.2 测试失败

```bash
# 查看详细错误
cargo test -- --nocapture
```
