# SQLRustGo v1.6.0 测试手册

> **版本**: v1.6.0
> **发布日期**: 2026-03-19
> **适用用户**: 开发人员、测试工程师

---

## 一、测试概览

### 1.1 测试目标

v1.6.0 版本测试覆盖以下关键领域：

| 模块 | 覆盖率目标 | 实际覆盖率 |
|------|-----------|-----------|
| Transaction | ≥90% | **93.15%** |
| Storage | ≥80% | **82.8%** |
| Planner | ≥75% | 71.7% |
| **总计** | **≥75%** | **70.72%** |

### 1.2 测试类型

| 类型 | 说明 | 位置 |
|------|------|------|
| 单元测试 | 各模块独立测试 | `crates/*/src/` |
| 集成测试 | 多模块协作测试 | `tests/` |
| 基准测试 | 性能测试 | `benches/` |

---

## 二、运行测试

### 2.1 快速开始

```bash
# 运行所有测试
cargo test --workspace

# 运行特定 crate 的测试
cargo test -p sqlrustgo-transaction
cargo test -p sqlrustgo-executor
cargo test -p sqlrustgo-planner

# 运行特定测试
cargo test test_mvcc
cargo test test_deadlock
cargo test test_tpch

# 查看测试覆盖率
cargo tarpaulin --workspace
```

### 2.2 测试命令参考

| 命令 | 说明 |
|------|------|
| `cargo test` | 运行所有测试 |
| `cargo test --workspace` | 运行工作区所有测试 |
| `cargo test --doc` | 运行文档测试 |
| `cargo test --no-run` | 编译测试但不运行 |
| `cargo test -- --nocapture` | 显示测试输出 |

---

## 三、测试模块

### 3.1 Transaction 模块

| 测试文件 | 覆盖内容 |
|----------|----------|
| `crates/transaction/src/mvcc.rs` | MVCC、快照、版本链 |
| `crates/transaction/src/manager.rs` | 事务管理 |
| `crates/transaction/src/lock.rs` | 行级锁 |
| `crates/transaction/src/deadlock.rs` | 死锁检测 |

**运行**:
```bash
cargo test -p sqlrustgo-transaction
```

### 3.2 Storage 模块

| 测试文件 | 覆盖内容 |
|----------|----------|
| `crates/storage/src/wal.rs` | WAL 写入/恢复 |
| `crates/storage/src/buffer_pool.rs` | Buffer Pool LRU |
| `crates/storage/src/bplus_tree/index.rs` | B+Tree 索引 |

**运行**:
```bash
cargo test -p sqlrustgo-storage
```

### 3.3 Executor 模块

| 测试文件 | 覆盖内容 |
|----------|----------|
| `crates/executor/src/executor.rs` | 查询执行 |
| `crates/executor/src/query_cache.rs` | 查询缓存 |
| `crates/executor/tests/tpch_test.rs` | TPC-H 测试 |

**运行**:
```bash
cargo test -p sqlrustgo-executor
```

---

## 四、集成测试

### 4.1 索引集成测试

```bash
cargo test --test index_integration_test
```

### 4.2 存储集成测试

```bash
cargo test --test storage_integration_test
```

### 4.3 TPC-H 测试

```bash
cargo test --test tpch_test
```

---

## 五、基准测试

### 5.1 TPC-H 基准

```bash
cargo bench --bench tpch_bench
```

### 5.2 性能测试

```bash
cargo bench --bench bench_aggregate
cargo bench --bench bench_scan
```

---

## 六、测试覆盖率

### 6.1 生成覆盖率报告

```bash
# 使用 tarpaulin
cargo tarpaulin --workspace --out Html

# 使用 llvm-cov
cargo llvm-cov --all-features --text
```

### 6.2 覆盖率目标

| 模块 | 目标 | 当前 |
|------|------|------|
| Transaction | 90% | 93.15% ✅ |
| Storage | 80% | 82.8% ✅ |
| Planner | 75% | 71.7% ⚠️ |
| Executor | 70% | 36% ⚠️ |

---

## 七、常见问题

### 7.1 测试失败

**问题**: 测试失败

**解决**:
```bash
# 查看详细错误
cargo test -- --nocapture

# 运行单个测试
cargo test test_name -- --nocapture
```

### 7.2 编译错误

**问题**: 依赖编译失败

**解决**:
```bash
cargo clean
cargo build --workspace
```

---

## 八、CI/CD 集成

### 8.1 GitHub Actions

测试在 CI 中自动运行：

```yaml
- name: Run tests
  run: cargo test --workspace

- name: Check coverage
  run: cargo tarpaulin --workspace
```

---

*本文档由 AI 辅助分析生成*
*生成日期: 2026-03-19*
