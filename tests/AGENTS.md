# tests/ - Integration Tests

> 全系统集成测试和回归测试。

## 测试组织

| 文件 | 作用 |
|------|------|
| `engine_test.rs` | 核心引擎测试 |
| `regression_test.rs` | 回归测试套件 |
| `sqlite_diff.rs` | SQLite 差异测试 |
| `e2e/` | 端到端测试 |
| `ci/` | CI 专用测试 |

## 常用命令

```bash
# 运行所有测试
cargo test --all-features

# 运行特定测试文件
cargo test --test engine_test --all-features

# 运行 e2e 测试
cargo test --test e2e_query_test --all-features

# 运行回归测试
cargo test --test regression_test --all-features

# 运行 SQLite diff
cargo test --test sqlite_diff --all-features
```

## 约定

- 测试使用 `#[test]` 标记
- 集成测试在 `tests/` 目录
- 单元测试在 crate 内部的 `src/` 或 `tests/` 子目录
- 性能测试使用 `#[ignore]` 或 `#[bench]`
