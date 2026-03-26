# SQLRustGo v1.9.0 测试说明

## 测试概览

v1.9.0 版本包含以下测试类型：

| 测试类型 | 数量 | 运行方式 |
|----------|------|----------|
| 单元测试 (unit) | 100+ | `cargo test` |
| 集成测试 (integration) | 50+ | `cargo test` |
| 性能测试 (performance) | 16 | `cargo test --test performance_test` |
| 教学场景测试 (teaching) | 18 | `cargo test --test teaching_scenario_test` |
| 压力测试 (stress) | 41 | `cargo test --test stress_test` |
| TPC-H 测试 | 5 | `cargo test --test tpch_test` |

---

## 测试目录结构

```
tests/
├── unit/                    # 单元测试
│   ├── bplus_tree_test.rs
│   ├── buffer_pool_test.rs
│   ├── file_storage_test.rs
│   ├── local_executor_test.rs
│   ├── optimizer_cost_test.rs
│   ├── optimizer_rules_test.rs
│   ├── parser_token_test.rs
│   ├── query_cache_config_test.rs
│   ├── query_cache_test.rs
│   ├── server_health_test.rs
│   ├── types_value_test.rs
│   └── vectorization_test.rs
│
├── integration/             # 集成测试
│   ├── executor_test.rs
│   ├── index_integration_test.rs
│   ├── integration_test.rs
│   ├── optimizer_stats_test.rs
│   ├── page_test.rs
│   ├── performance_test.rs  # 性能测试
│   ├── planner_test.rs
│   ├── query_cache_test.rs
│   ├── session_config_test.rs
│   ├── storage_integration_test.rs
│   ├── teaching_scenario_test.rs  # 教学场景测试
│   └── tpch_test.rs
│
├── stress/                  # 压力测试
│   ├── concurrency_stress_test.rs
│   ├── crash_recovery_test.rs
│   ├── production_scenario_test.rs
│   └── stress_test.rs
│
├── ci/                      # CI 测试
│   ├── buffer_pool_benchmark_test.rs
│   ├── buffer_pool_test.rs
│   └── ci_test.rs
│
└── e2e/                    # 端到端测试
    ├── e2e_query_test.rs
    ├── monitoring_test.rs
    └── observability_test.rs
```

---

## 本地运行测试

### 1. 快速验证 (推荐)

运行所有单元测试和核心集成测试：

```bash
cargo test --workspace
```

### 2. 性能测试 (必须运行)

```bash
# 运行所有性能测试
cargo test --test performance_test -- --nocapture

# 单独运行 Join 性能测试
cargo test --test performance_test test_join_performance -- --nocapture

# 单独运行 TPC-H 测试
cargo test --test tpch_test -- --nocapture
```

### 3. 教学场景测试

```bash
cargo test --test teaching_scenario_test -- --nocapture
```

### 4. 压力测试

```bash
cargo test --test stress_test -- --nocapture
```

### 5. 完整测试 (全面验证)

```bash
# 运行所有测试
cargo test --workspace --all

# 带详细输出
cargo test --workspace --all -- --nocapture
```

---

## CI 运行方式

### GitHub Actions CI (`.github/workflows/ci.yml`)

**触发条件**:
- 推送到 main, develop/*, release/*, rc/*, ga/*, feature/*
- PR 到 main, develop/* 等分支
- 手动触发

**运行命令**:
```bash
cargo fmt --all -- --check          # 格式检查
cargo clippy --all-targets -- -D warnings  # Lint
cargo build --all                   # 构建
cargo test --all                   # 测试
```

### 性能基准测试 (`.github/workflows/benchmark.yml`)

```bash
# 运行基准测试
cargo bench

# 运行 TPC-H 对比
python3 scripts/tpch_simple.py
```

---

## 测试清单

### 开发自测 (每次提交前)

```bash
# 1. 编译检查
cargo build --all

# 2. 快速测试
cargo test --workspace

# 3. 性能测试 (推荐)
cargo test --test performance_test -- --nocapture
```

### 发布前检查

```bash
# 1. 格式化
cargo fmt --all
cargo fmt --all -- --check

# 2. Lint
cargo clippy --all-targets -- -D warnings

# 3. 完整测试
cargo test --workspace --all

# 4. 性能测试
cargo test --test performance_test -- --nocapture
cargo test --test teaching_scenario_test -- --nocapture
cargo test --test stress_test -- --nocapture
```

---

## 性能测试说明

### performance_test.rs (16 个测试)

| 测试名称 | 描述 |
|----------|------|
| test_batch_insert_performance | 批量插入 10000 行 |
| test_batch_insert_single_statement | 单语句批量插入 |
| test_cache_hit_performance | 查询缓存命中 |
| test_concurrent_reads_performance | 并发读取 |
| test_connection_pool_basic_operations | 连接池基本操作 |
| test_connection_pool_concurrent_stress | 连接池并发压力 |
| test_index_scan_performance_vs_seqscan | 索引扫描 vs 顺序扫描 |
| test_join_performance_hash_join | Hash Join 性能 |
| test_limit_performance | LIMIT 性能 |
| test_mixed_workload_performance | 混合工作负载 |
| test_order_by_performance | ORDER BY 性能 |
| test_parallel_scan_operations | 并行扫描 |
| test_query_optimization_predicate_pushdown | 谓词下推优化 |
| test_query_optimization_projection | 投影优化 |
| test_vectorization_bulk_operations | 向量化批量操作 |
| test_vectorization_record_batch | 向量化 RecordBatch |

### teaching_scenario_test.rs (18 个测试)

| 测试名称 | 描述 |
|----------|------|
| test_analyze_updates_statistics | ANALYZE 统计更新 |
| test_basic_select_operations | 基础 SELECT |
| test_btree_index_operations | B+Tree 索引操作 |
| test_cbo_optimizer_cost_based_selection | CBO 优化器 |
| test_column_statistics_for_optimizer | 列统计 |
| test_delete_operations | DELETE 操作 |
| test_explain_shows_query_plan | EXPLAIN 执行计划 |
| test_foreign_key_constraint_enforcement | 外键约束 |
| test_hash_join_with_condition | Hash Join |
| test_insert_operations | INSERT 操作 |
| test_isolation_level_read_committed | 隔离级别 |
| test_lock_manager_with_deadlock_detector | 死锁检测 |
| test_multiple_joins | 多表 Join |
| test_query_cache_basic | 查询缓存 |
| test_table_creation_ddl | 表创建 DDL |
| test_transaction_rollback | 事务回滚 |
| test_update_operations | UPDATE 操作 |
| test_where_clause_filtering | WHERE 过滤 |

---

## 故障排查

### 测试失败

1. **查看详细输出**:
```bash
cargo test <test_name> -- --nocapture --show-output
```

2. **运行单个测试**:
```bash
cargo test test_join_performance -- --nocapture
```

3. **查看测试日志**:
```bash
RUST_LOG=debug cargo test
```

### 性能测试不稳定

性能测试结果可能因系统负载波动。如有明显下降：

1. 多次运行确认
2. 检查系统资源
3. 关闭其他应用

---

## 相关文档

- `docs/releases/v1.9.0/TPCH_DEVELOPMENT_LOG.md` - TPC-H 测试发展记录
- `docs/releases/v1.9.0/PERFORMANCE_COMPARISON.md` - 性能对比报告
- `.github/workflows/ci.yml` - CI 配置

---

*最后更新: 2026-03-26*
*版本: v1.9.0*
