# SQLRustGo v1.4.0 测试手册

> 版本：v1.4.0
> 发布日期：2026-03-16
> 适用用户：开发人员、测试工程师

---

## 一、测试概览

### 1.1 测试目标

v1.4.0 版本测试覆盖以下关键领域：

| 模块 | 覆盖率目标 | 实际覆盖率 |
|------|-----------|-----------|
| 整体 | ≥80% | **82%+** |
| Executor | ≥85% | 88%+ |
| Planner | ≥76% | 78%+ |
| Optimizer | ≥82% | 85%+ |

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
cargo test -p sqlrustgo-executor
cargo test -p sqlrustgo-optimizer

# 运行特定测试
cargo test test_hash_join
cargo test test_nested_loop

# 查看测试覆盖率
cargo tarpaulin --workspace --all-features
```

### 2.2 测试命令参考

| 命令 | 说明 |
|------|------|
| `cargo test` | 运行所有测试 |
| `cargo test --workspace` | 运行工作区所有测试 |
| `cargo test --doc` | 运行文档测试 |
| `cargo test --no-run` | 编译测试但不运行 |
| `cargo test -- --nocapture` | 显示测试输出 |

### 2.3 覆盖率命令

```bash
# 使用 tarpaulin
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --all-features --out Html

# 使用 grcov
cargo install cargo-grcov
RUSTFLAGS="-C instrument-coverage" cargo test
grcov . --binary-path ./target/debug/ -s . --html-cov > coverage.html
```

---

## 三、模块测试

### 3.1 Executor 测试

```bash
# 运行所有 Executor 测试
cargo test -p sqlrustgo-executor

# 运行特定算子测试
cargo test hash_join
cargo test sort_merge_join
cargo test nested_loop
cargo test aggregate
```

### 3.2 Optimizer 测试

```bash
# 运行优化器测试
cargo test -p sqlrustgo-optimizer

# 运行 CBO 测试
cargo test cost
cargo test stats
cargo test index_select
```

### 3.3 Planner 测试

```bash
# 运行 Planner 测试
cargo test -p sqlrustgo-planner
```

### 3.4 Server 测试

```bash
# 运行 Server 测试 (包含 HTTP 端点)
cargo test -p sqlrustgo-server

# 测试 /metrics 端点
cargo test metrics
```

---

## 四、新增测试用例

### 4.1 Join 算法测试

| 测试名称 | 功能 |
|----------|------|
| `test_hash_join_inner` | HashJoin 内连接 |
| `test_hash_join_left` | HashJoin 左外连接 |
| `test_sort_merge_join_inner` | SortMergeJoin 内连接 |
| `test_sort_merge_join_left` | SortMergeJoin 左外连接 |
| `test_nested_loop_join_inner` | NestedLoopJoin 内连接 |
| `test_nested_loop_join_cross` | Cross Join |
| `test_nested_loop_join_left_outer` | 左外连接 |
| `test_nested_loop_join_right_outer` | 右外连接 |

### 4.2 CBO 测试

| 测试名称 | 功能 |
|----------|------|
| `test_cost_model_scan` | 扫描代价估算 |
| `test_cost_model_join` | Join 代价估算 |
| `test_index_select` | 索引选择 |
| `test_stats_integration` | 统计信息集成 |

### 4.3 向量化测试

| 测试名称 | 功能 |
|----------|------|
| `test_vector_operations` | 向量操作 |
| `test_batch_iterator` | 批量迭代器 |

---

## 五、集成测试

### 5.1 运行集成测试

```bash
# 运行集成测试
cargo test --test integration_test

# 运行特定集成测试
cargo test --test integration_test -- join
```

### 5.2 测试场景

| 测试场景 | 说明 |
|----------|------|
| 端到端 SQL | 完整 SQL 执行流程 |
| Join 性能 | 多表 Join 性能测试 |
| CBO 优化 | 代价模型优化验证 |
| 向量化 | 批量处理性能 |

---

## 六、基准测试

### 6.1 运行基准测试

```bash
# 运行所有基准测试
cargo bench --workspace

# 运行特定基准
cargo bench --package sqlrustgo-executor
```

### 6.2 基准测试类型

| 基准名称 | 说明 |
|----------|------|
| `bench_executor` | 执行器性能 |
| `bench_join` | Join 算法性能 |
| `bench_cbo` | CBO 优化效果 |
| `bench_v140` | v1.4.0 整体性能 |
| `tpch_bench` | TPC-H 基准 |

---

## 七、CI/CD 测试

### 7.1 GitHub Actions

测试在 CI 中自动运行：

```yaml
# .github/workflows/ci.yml
- name: Run tests
  run: cargo test --workspace
  
- name: Run clippy
  run: cargo clippy --workspace -- -D warnings
  
- name: Check formatting
  run: cargo fmt --check
```

### 7.2 测试门禁

| 检查项 | 命令 |
|--------|------|
| 编译 | `cargo build --all-features` |
| 测试 | `cargo test --workspace` |
| Clippy | `cargo clippy -- -D warnings` |
| 格式化 | `cargo fmt --check` |

---

## 八、问题排查

### 常见问题

| 问题 | 解决方案 |
|------|----------|
| 测试失败 | 使用 `-- --nocapture` 查看输出 |
| 编译超时 | 使用 `--release` 模式 |
| 覆盖率低 | 补充单元测试 |

---

**文档版本**: 1.0
**最后更新**: 2026-03-16
