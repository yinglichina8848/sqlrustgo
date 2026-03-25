# SQLRustGo 测试指南

## 测试目录结构

```
tests/
├── data/              # 测试数据文件 (JSON)
├── e2e/               # 端到端测试
├── integration/       # 模块集成测试
├── stress/            # 压力测试
├── ci/                # CI 专用测试
├── unit/              # 独立单元测试
├── data_loader.rs     # 统一数据加载
└── README.md
```

## 测试类型

| 类型 | 目录 | 说明 |
|------|------|------|
| 单元测试 | `crates/*/src` | 模块内 `#[cfg(test)]` |
| 模块集成 | `tests/integration/` | 模块间调用测试 |
| E2E | `tests/e2e/` | 完整查询流程测试 |
| 压力测试 | `tests/stress/` | 并发、崩溃恢复等 |
| CI | `tests/ci/` | CI 环境专用测试 |

## 运行测试

```bash
# 单元测试
cargo test --lib

# 集成测试
cargo test --test '*'

# E2E 测试
cargo test --test e2e_*

# 压力测试
cargo test --test stress_*

# 所有测试
cargo test
```

## 数据加载

使用 `tests/data_loader.rs`:

```rust
use crate::data_loader::TestDataLoader;

let data = TestDataLoader::load_json("test_data.json");
```
