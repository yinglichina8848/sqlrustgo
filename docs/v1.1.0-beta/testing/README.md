# SQLRustGo 测试指南

## 1. 测试基础

### 1.1 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test --lib

# 运行集成测试
cargo test --test integration_test

# 显示测试输出
cargo test -- --nocapture
```

### 1.2 测试类型

| 类型 | 命令 | 说明 |
|------|------|------|
| 单元测试 | `cargo test --lib` | 测试单个模块 |
| 集成测试 | `cargo test --test *` | 测试模块交互 |
| 文档测试 | `cargo test --doc` | 测试文档中的代码示例 |

## 2. 功能测试

### 2.1 测试 SQL 解析

```bash
# 运行 parser 相关测试
cargo test parser

# 运行特定测试
cargo test test_parse_select
```

### 2.2 测试执行引擎

```bash
cargo test executor

# 测试聚合函数
cargo test aggregate
```

### 2.3 测试存储引擎

```bash
cargo test storage

# 测试 B+ Tree
cargo test bplus_tree

# 测试 WAL
cargo test wal
```

### 2.4 测试网络协议

```bash
cargo test network
```

## 3. 覆盖率测试

### 3.1 生成覆盖率报告

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成 HTML 报告
cargo tarpaulin --out Html

# 生成 JSON 报告
cargo tarpaulin --out Json

# 查看覆盖率
cargo tarpaulin
```

### 3.2 覆盖率目标

| 模块 | 目标覆盖率 |
|------|-----------|
| lexer | 90%+ |
| parser | 85%+ |
| executor | 85%+ |
| storage | 80%+ |
| network | 90%+ |
| **整体** | **80%+** |

### 3.3 覆盖率分析

```bash
# 查看特定文件覆盖率
cargo tarpaulin --include-files "src/parser/mod.rs"

# 排除测试文件
cargo tarpaulin --exclude-tests
```

## 4. 性能测试

### 4.1 基准测试

```bash
# 运行基准测试
cargo bench

# 运行特定基准测试
cargo bench bench_name
```

### 4.2 性能指标

```bash
# 测试查询性能
# 在 REPL 中执行 SELECT 查询并记录时间

# 测试批量插入性能
# 测试大数据量查询性能
```

### 4.3 内存使用

```bash
# 使用 valgrind（如果可用）
valgrind --tool=massif cargo run --bin sqlrustgo
```

## 5. 集成测试

### 5.1 端到端测试

```bash
# 运行集成测试
cargo test --test integration_test
```

### 5.2 网络测试

```bash
# 启动服务器并测试连接
# 测试 TCP 连接、查询响应等
```

## 6. 测试开发

### 6.1 编写单元测试

在 `src/` 目录的模块中添加：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // 测试代码
        assert!(result.is_ok());
    }
}
```

### 6.2 编写集成测试

在 `tests/` 目录创建文件：

```rust
// tests/integration_test.rs
use sqlrustgo::*;

#[test]
fn test_full_flow() {
    let mut engine = ExecutionEngine::new();
    // 测试完整流程
}
```

### 6.3 测试命名规范

- `test_module_functionality` - 功能测试
- `test_module_edge_case` - 边界情况测试
- `test_module_error_handling` - 错误处理测试

## 7. CI/CD 测试

### 7.1 GitHub Actions

测试在 CI 中自动运行：

```bash
# 构建
cargo build

# 测试
cargo test

# Clippy 检查
cargo clippy

# 格式化检查
cargo fmt --check
```

### 7.2 测试命令脚本

```bash
# 完整测试流程
./scripts/run-tests.sh
```

## 8. 测试报告

### 8.1 生成测试报告

```bash
# 生成 JSON 报告
cargo test -- --report-time json > test-report.json

# 生成覆盖率 XML（适用于 CI）
cargo tarpaulin --out Xml
```

### 8.2 查看测试历史

在 GitHub Actions 中查看测试历史。

## 9. 常见问题

### 9.1 测试失败

```bash
# 查看详细错误
cargo test -- --nocapture

# 运行单个失败的测试
cargo test test_name -- --nocapture
```

### 9.2 测试超时

某些集成测试可能需要较长时间，可以使用：

```bash
cargo test -- --test-threads=1
```

### 9.3 测试数据清理

测试应该清理创建的数据：

```rust
#[test]
fn test_with_cleanup() {
    // 测试代码
    // 清理数据
    std::fs::remove_file("data/test.db").ok();
}
```

## 10. 测试检查清单

提交前确保：

- [ ] 所有单元测试通过
- [ ] 所有集成测试通过
- [ ] Clippy 无警告
- [ ] 代码格式化
- [ ] 覆盖率达标
- [ ] 新功能有对应测试
- [ ] 边界情况有对应测试
