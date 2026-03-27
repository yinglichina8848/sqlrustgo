# SQLRustGo v1.9.0 测试说明

## 测试概览

v1.9.0 版本包含以下测试类型：

| 测试类型 | 数量 | 运行方式 | 状态 |
|----------|------|----------|------|
| 单元测试 (unit) | ~70+ | `cargo test --test <test_name>` | ✅ 通过 |
| 集成测试 (integration) | ~150+ | `cargo test --test <test_name>` | ✅ 通过 |
| 异常测试 (anomaly) | ~50+ | `cargo test --test <test_name>` | ✅ 通过 |
| 压力测试 (stress) | ~30+ | `cargo test --test <test_name>` | ✅ 通过 |
| 性能测试 (performance) | 22 | `cargo test --test performance_test` | ✅ 通过 |
| 教学场景测试 (teaching) | 35 | `cargo test --test teaching_scenario_test` | ✅ 通过 |
| **总计** | **~370+** | - | ✅ 全部通过 |

---

## 测试目录结构

```
tests/
├── unit/                    # 单元测试 (~70+ 测试)
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
├── integration/             # 集成测试 (~150+ 测试)
│   ├── autoinc_test.rs
│   ├── checksum_corruption_test.rs
│   ├── executor_test.rs
│   ├── foreign_key_test.rs
│   ├── fk_actions_test.rs
│   ├── index_integration_test.rs
│   ├── integration_test.rs
│   ├── mysql_tpch_test.rs          # MySQL TPC-H 对比测试
│   ├── optimizer_stats_test.rs
│   ├── page_test.rs
│   ├── performance_test.rs        # 性能测试 (22 测试)
│   ├── planner_test.rs
│   ├── query_cache_test.rs
│   ├── savepoint_test.rs
│   ├── server_integration_test.rs # 服务器集成测试
│   ├── session_config_test.rs
│   ├── storage_integration_test.rs
│   ├── teaching_scenario_test.rs   # 教学场景测试 (35 测试)
│   ├── upsert_test.rs
│   └── wal_integration_test.rs
│
├── stress/                  # 压力测试 (~30+ 测试)
│   ├── chaos_test.rs              # 混沌工程测试 (12 测试)
│   ├── concurrency_stress_test.rs
│   ├── crash_recovery_test.rs     # 崩溃恢复测试 (9 测试)
│   ├── production_scenario_test.rs
│   └── stress_test.rs
│
├── anomaly/                 # 异常测试 (~50+ 测试)
│   ├── aggregate_type_test.rs
│   ├── boundary_test.rs
│   ├── catalog_consistency_test.rs
│   ├── checksum_corruption_test.rs
│   ├── crash_injection_test.rs
│   ├── datetime_type_test.rs
│   ├── error_handling_test.rs
│   ├── fk_constraint_test.rs
│   ├── join_test.rs
│   ├── long_run_stability_test.rs
│   ├── mvcc_concurrency_test.rs   # MVCC 并发测试 (6 测试)
│   ├── null_handling_test.rs
│   ├── qps_benchmark_test.rs
│   ├── set_operations_test.rs
│   ├── snapshot_isolation_test.rs # 快照隔离测试 (17 测试)
│   ├── transaction_isolation_test.rs
│   ├── transaction_timeout_test.rs
│   ├── view_test.rs
│   └── wal_deterministic_test.rs
│
└── ci/                      # CI 测试
    ├── ci_test.rs                 # CI 环境检查 (5 测试)
    └── ...
```

---

## 本地运行测试

### 1. 快速验证 (推荐)

运行所有单元测试和核心集成测试：

```bash
cargo test --workspace
```

### 2. 单元测试

```bash
# 运行所有单元测试
cargo test --tests/unit

# 单独运行各单元测试
cargo test --test bplus_tree_test           # B+树测试 (6 测试)
cargo test --test buffer_pool_test          # BufferPool测试 (16 测试)
cargo test --test types_value_test          # 类型值测试 (13 测试)
cargo test --test vectorization_test        # 向量化测试 (10 测试)
cargo test --test query_cache_test          # 查询缓存测试 (9 测试)
cargo test --test local_executor_test       # 本地执行器测试 (4 测试)
cargo test --test optimizer_cost_test       # 优化器成本测试
cargo test --test optimizer_rules_test      # 优化器规则测试
cargo test --test server_health_test        # 服务器健康检查测试
```

### 3. 集成测试

```bash
# 核心集成测试
cargo test --test executor_test             # 执行器测试 (19 测试)
cargo test --test planner_test              # 规划器测试 (29 测试)
cargo test --test page_test                 # 页面测试 (16 测试)
cargo test --test foreign_key_test          # 外键测试 (21 测试)
cargo test --test server_integration_test   # 服务器集成测试 (27 测试)
cargo test --test upsert_test               # Upsert测试 (4 测试)
cargo test --test autoinc_test              # 自增列测试

# TPC-H 测试 (需要 MySQL 对比，测试默认忽略)
cargo test --test mysql_tpch_test           # MySQL TPC-H 对比 (4 测试，默认忽略)
```

### 4. 异常测试 (Anomaly Tests)

```bash
# MVCC 并发测试
cargo test --test mvcc_concurrency_test     # MVCC并发测试 (6 测试)

# 快照隔离测试
cargo test --test snapshot_isolation_test   # 快照隔离测试 (17 测试)

# 混沌工程测试
cargo test --test chaos_test                # 混沌工程测试 (12 测试)

# 崩溃恢复测试
cargo test --test crash_recovery_test       # 崩溃恢复测试 (9 测试)

# WAL 集成测试
cargo test --test wal_integration_test      # WAL集成测试 (16 测试)

# 其他异常测试
cargo test --test error_handling_test       # 错误处理测试
cargo test --test boundary_test             # 边界测试
cargo test --test null_handling_test        # NULL处理测试
```

### 5. 教学场景测试

```bash
# 基础教学场景测试
cargo test --test teaching_scenario_test    # 教学场景测试 (35 测试)

# 覆盖范围:
# - 基础 CRUD 操作 (SELECT, INSERT, UPDATE, DELETE)
# - 事务处理 (BEGIN, COMMIT, ROLLBACK, SAVEPOINT)
# - 索引操作 (B+Tree 索引创建和使用)
# - JOIN 操作 (INNER JOIN, 多种 JOIN 类型)
# - 聚合查询 (COUNT, SUM, AVG, GROUP BY, HAVING)
# - 子查询 (WHERE 子句和 FROM 子句中的子查询)
# - 视图操作 (CREATE VIEW, 查询视图)
# - 优化器 (CBO, 谓词下推, 投影优化)
# - 查询缓存
# - 隔离级别
# - 死锁检测
```

### 6. 压力测试

```bash
cargo test --test chaos_test                # 混沌工程测试 (12 测试)
cargo test --test crash_recovery_test       # 崩溃恢复测试 (9 测试)
cargo test --test stress_test               # 压力测试
cargo test --test concurrency_stress_test  # 并发压力测试
cargo test --test production_scenario_test  # 生产场景测试
```

### 7. 性能测试

```bash
# 运行所有性能测试
cargo test --test performance_test -- --nocapture

# 关键性能测试
test_batch_insert_performance              # 批量插入性能
test_single_insert_qps                    # 单条插入 QPS
test_index_scan_performance_vs_seqscan    # 索引扫描 vs 顺序扫描
test_join_performance_hash_join           # Hash Join 性能
test_cache_hit_performance                # 缓存命中性能
test_query_optimization_predicate_pushdown # 谓词下推优化
test_vectorization_bulk_operations         # 向量化批量操作
```

### 8. 完整测试 (全面验证)

```bash
# 运行所有测试
cargo test --workspace

# 带详细输出
cargo test --workspace -- --nocapture

# 运行所有测试并显示输出
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
cargo test --all                    # 运行所有测试
```

### 性能基准测试 (`.github/workflows/benchmark.yml`)

```bash
# 运行基准测试
cargo bench

# 运行 TPC-H 对比
python3 scripts/tpch_simple.py
```

---

## 测试结果摘要 (v1.9.0)

### 最新测试运行结果

| 测试类型 | 测试文件 | 通过 | 失败 | 忽略 | 总数 |
|----------|----------|------|------|------|------|
| 单元测试 | bplus_tree_test | 6 | 0 | 0 | 6 |
| 单元测试 | buffer_pool_test | 16 | 0 | 0 | 16 |
| 单元测试 | types_value_test | 13 | 0 | 0 | 13 |
| 单元测试 | vectorization_test | 10 | 0 | 0 | 10 |
| 单元测试 | query_cache_test | 9 | 0 | 0 | 9 |
| 单元测试 | local_executor_test | 4 | 0 | 0 | 4 |
| CI 测试 | ci_test | 5 | 0 | 0 | 5 |
| 集成测试 | executor_test | 19 | 0 | 0 | 19 |
| 集成测试 | planner_test | 29 | 0 | 0 | 29 |
| 集成测试 | page_test | 16 | 0 | 0 | 16 |
| 集成测试 | foreign_key_test | 21 | 0 | 6 | 27 |
| 集成测试 | server_integration_test | 27 | 0 | 4 | 31 |
| 集成测试 | performance_test | 22 | 0 | 0 | 22 |
| 集成测试 | teaching_scenario_test | 35 | 0 | 0 | 35 |
| 集成测试 | upsert_test | 4 | 0 | 2 | 6 |
| 集成测试 | mysql_tpch_test | 0 | 0 | 4 | 4 |
| 异常测试 | mvcc_concurrency_test | 6 | 0 | 0 | 6 |
| 异常测试 | snapshot_isolation_test | 17 | 0 | 0 | 17 |
| 异常测试 | chaos_test | 12 | 0 | 0 | 12 |
| 异常测试 | crash_recovery_test | 9 | 0 | 0 | 9 |
| 异常测试 | wal_integration_test | 16 | 0 | 0 | 16 |
| **总计** | - | **~300+** | **0** | **~16** | **~320+** |

> 注: mysql_tpch_test 需要 MySQL 数据库连接，默认忽略。外键测试部分需要特定条件触发。

### 关键功能测试覆盖

| 功能模块 | 测试覆盖 | 状态 |
|----------|----------|------|
| SQL 解析 (Parser) | parser_token_test, planner_test | ✅ |
| SQL 执行 (Executor) | executor_test, teaching_scenario_test | ✅ |
| 存储引擎 (Storage) | buffer_pool_test, page_test, bplus_tree_test | ✅ |
| 事务管理 (Transaction) | mvcc_concurrency_test, snapshot_isolation_test | ✅ |
| 查询优化 (Optimizer) | optimizer_cost_test, optimizer_rules_test, performance_test | ✅ |
| 索引操作 (Index) | bplus_tree_test, foreign_key_test | ✅ |
| 向量化执行 (Vectorization) | vectorization_test, performance_test | ✅ |
| 崩溃恢复 (Recovery) | crash_recovery_test, wal_integration_test, chaos_test | ✅ |
| 并发控制 (Concurrency) | mvcc_concurrency_test, chaos_test | ✅ |
| 教学场景 (Teaching) | teaching_scenario_test | ✅ |

---

## 测试清单

### 开发自测 (每次提交前)

```bash
# 1. 编译检查
cargo build --all

# 2. 快速测试 (核心功能)
cargo test --test executor_test
cargo test --test planner_test
cargo test --test local_executor_test

# 3. 单元测试
cargo test --test buffer_pool_test
cargo test --test bplus_tree_test

# 4. 教学场景测试 (验证 SQL 功能)
cargo test --test teaching_scenario_test
```

### 发布前检查

```bash
# 1. 格式化
cargo fmt --all
cargo fmt --all -- --check

# 2. Lint
cargo clippy --all-targets -- -D warnings

# 3. 单元测试
cargo test --test bplus_tree_test
cargo test --test buffer_pool_test
cargo test --test types_value_test
cargo test --test vectorization_test

# 4. 集成测试
cargo test --test executor_test
cargo test --test planner_test
cargo test --test server_integration_test
cargo test --test foreign_key_test

# 5. 性能测试
cargo test --test performance_test -- --nocapture

# 6. 教学场景测试
cargo test --test teaching_scenario_test -- --nocapture

# 7. 异常测试
cargo test --test mvcc_concurrency_test
cargo test --test snapshot_isolation_test
cargo test --test chaos_test

# 8. 完整测试
cargo test --workspace
```

### 快速回归测试

```bash
# 快速验证核心功能 (~1 分钟)
cargo test --test executor_test
cargo test --test planner_test
cargo test --test teaching_scenario_test
cargo test --test local_executor_test
```

### 完整验证 (~5-10 分钟)

```bash
# 完整测试套件
cargo test --workspace -- --nocapture
```

---

## 性能测试说明

### performance_test.rs (22 个测试)

| 测试名称 | 描述 |
|----------|------|
| test_batch_insert_performance | 批量插入 10000 行 |
| test_batch_insert_single_statement | 单语句批量插入 |
| test_cache_hit_performance | 查询缓存命中 |
| test_concurrent_reads_performance | 并发读取 |
| test_connection_pool_basic_operations | 连接池基本操作 |
| test_connection_pool_concurrent_stress | 连接池并发压力 |
| test_covering_index | 覆盖索引 |
| test_index_scan_performance_vs_seqscan | 索引扫描 vs 顺序扫描 |
| test_insert_batch_optimization | 插入批量优化 |
| test_insert_with_transaction | 事务中插入 |
| test_join_performance_hash_join | Hash Join 性能 |
| test_limit_performance | LIMIT 性能 |
| test_mixed_workload_performance | 混合工作负载 |
| test_order_by_performance | ORDER BY 性能 |
| test_parallel_scan_operations | 并行扫描 |
| test_query_optimization_predicate_pushdown | 谓词下推优化 |
| test_query_optimization_projection | 投影优化 |
| test_single_insert_qps | 单条插入 QPS |
| test_vectorization_bulk_operations | 向量化批量操作 |
| test_vectorization_record_batch | 向量化 RecordBatch |
| test_wal_write_optimization | WAL 写入优化 |
| test_composite_index | 复合索引 |

### teaching_scenario_test.rs (35 个测试)

| 测试名称 | 描述 |
|----------|------|
| test_analyze_updates_statistics | ANALYZE 统计更新 |
| test_basic_select_operations | 基础 SELECT |
| test_btree_index_operations | B+Tree 索引操作 |
| test_cbo_optimizer_cost_based_selection | CBO 优化器 |
| test_column_statistics_for_optimizer | 列统计 |
| test_delete_operations | DELETE 操作 |
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
| test_teaching_aggregate_count | 聚合 COUNT |
| test_teaching_delete_basic | 基础 DELETE |
| test_teaching_group_by | GROUP BY |
| test_teaching_having | HAVING |
| test_teaching_index_creation | 索引创建 |
| test_teaching_insert_basic | 基础 INSERT |
| test_teaching_foreign_key | 外键 |
| test_teaching_savepoint | SAVEPOINT |
| test_teaching_select_basic | 基础 SELECT |
| test_teaching_subquery_basic | 基础子查询 |
| test_teaching_subquery_in_where | WHERE 子查询 |
| test_teaching_transaction_basic | 基础事务 |
| test_teaching_transaction_commit | 事务提交 |
| test_teaching_transaction_isolation | 事务隔离 |
| test_teaching_transaction_rollback | 事务回滚 |
| test_teaching_update_basic | 基础 UPDATE |
| test_teaching_view_basic | 基础视图 |
| test_teaching_join_inner | 内连接 |

### anomaly/mvcc_concurrency_test.rs (6 个测试)

| 测试名称 | 描述 |
|----------|------|
| test_concurrent_insert_stability | 并发插入稳定性 |
| test_concurrent_mixed_operations | 混合并发操作 |
| test_concurrent_read_stability | 并发读取稳定性 |
| test_deadlock_scenario | 死锁场景 |
| test_large_dataset_concurrent_ops | 大数据集并发操作 |
| test_rapid_create_drop | 快速创建删除 |

### anomaly/snapshot_isolation_test.rs (17 个测试)

| 测试名称 | 描述 |
|----------|------|
| test_aborted_transaction_not_visible | 中止事务不可见 |
| test_long_running_with_concurrent_commits | 长事务与并发提交 |
| test_mvcc_global_timestamp_increments | MVCC 全局时间戳递增 |
| test_mvcc_snapshot_visibility_rules | MVCC 快照可见性规则 |
| test_mvcc_transaction_lifecycle | MVCC 事务生命周期 |
| test_multi_statement_isolation | 多语句隔离 |
| test_read_committed_no_dirty_read | 读已提交无脏读 |
| test_read_committed_refreshes_snapshot | 读已提交刷新快照 |
| test_read_committed_sees_committed_data | 读已提交看到已提交数据 |
| test_read_committed_snapshot_timestamp_fixed | 读已提交快照时间戳固定 |
| test_read_uncommitted_visibility | 读未提交可见性 |
| test_repeatable_read_no_new_commits | 可重复读无新提交 |
| test_repeatable_read_own_writes_visible | 可重复读自己的写可见 |
| test_repeatable_read_same_read_same_result | 可重复读相同读相同结果 |
| test_self_always_visible | 自己总是可见 |
| test_snapshot_isolation_concurrent_update_handling | 快照隔离并发更新处理 |
| test_snapshot_isolation_consistent_snapshot | 快照隔离一致快照 |

### stress/chaos_test.rs (12 个测试)

| 测试名称 | 描述 |
|----------|------|
| test_buffer_pool_pressure | BufferPool 压力 |
| test_connection_memory_limits | 连接内存限制 |
| test_crash_recovery_integration | 崩溃恢复集成 |
| test_disk_full_handling | 磁盘满处理 |
| test_graceful_degradation | 优雅降级 |
| test_memory_allocation_handling | 内存分配处理 |
| test_multiple_failure_scenario | 多重故障场景 |
| test_network_delay_handling | 网络延迟处理 |
| test_partial_commit_recovery | 部分提交恢复 |
| test_replication_timeout_handling | 复制超时处理 |
| test_wal_disk_full_recovery | WAL 磁盘满恢复 |
| test_wal_integrity_after_crash | 崩溃后 WAL 完整性 |

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

4. **调试特定模块**:
```bash
# 只运行指定测试
cargo test --test executor_test test_batch_insert

# 查看更详细的错误信息
RUST_BACKTRACE=1 cargo test
```

### 性能测试不稳定

性能测试结果可能因系统负载波动。如有明显下降：

1. 多次运行确认
2. 检查系统资源
3. 关闭其他应用
4. 在安静环境中运行

### 编译警告处理

```bash
# 自动修复警告
cargo fix --lib -p sqlrustgo

# 查看所有警告
cargo clippy --all-targets -- -W unused
```

### 常见问题

| 问题 | 解决方案 |
|------|----------|
| 编译错误 | `cargo clean && cargo build --all` |
| 测试超时 | 增加超时时间或检查死锁 |
| 性能下降 | 检查系统资源使用情况 |
| 内存不足 | 减少并行测试数量 |

---

## 相关文档

- `docs/releases/v1.9.0/TPCH_DEVELOPMENT_LOG.md` - TPC-H 测试发展记录
- `docs/releases/v1.9.0/PERFORMANCE_COMPARISON.md` - 性能对比报告
- `docs/releases/v1.9.0/TEST_ENHANCEMENT_REPORT.md` - 测试增强报告
- `.github/workflows/ci.yml` - CI 配置
- `docs/superpowers/specs/teaching-test-architecture-2026-03-27.md` - 教学测试架构设计
- `docs/superpowers/plans/teaching-client-server-test-plan-2026-03-27.md` - Client-Server 测试计划

---

## 附录: Client-Server 教学测试说明

### 概述

Client-Server 教学测试通过 HTTP 协议与 TeachingHttpServer 交互，验证真实的客户端-服务器 SQL 执行流程。这是教学场景测试的重要组成部分，确保学生能够通过 HTTP API 访问数据库系统。

### 测试结构

```rust
// tests/integration/teaching_scenario_client_server_test.rs 结构
struct TestServer {
    handle: thread::JoinHandle<()>,
    port: u16,
}

impl TestServer {
    fn new() -> Self { ... }
    fn sql(&self, sql: &str) -> SqlResponse { ... }
    fn get(&self, path: &str) -> Value { ... }
}
```

### 测试端点

| 端点 | 方法 | 描述 |
|------|------|------|
| `/sql` | POST | 执行 SQL 语句 |
| `/health/live` | GET | 存活检查 |
| `/health/ready` | GET | 就绪检查 |
| `/metrics` | GET | 指标数据 |
| `/teaching/pipeline/json` | GET | 查询计划可视化 |

### SQL 执行请求格式

```json
POST /sql
Content-Type: application/json

{
  "sql": "SELECT * FROM users WHERE id = 1"
}
```

### SQL 执行响应格式

```json
{
  "columns": ["id", "name", "email"],
  "rows": [[1, "Alice", "alice@example.com"]],
  "affected_rows": 0,
  "error": null
}
```

### 错误响应格式

```json
{
  "columns": null,
  "rows": null,
  "affected_rows": 0,
  "error": "Table 'users' does not exist"
}
```

### 测试覆盖场景

| 场景 | SQL | 验证内容 |
|------|-----|----------|
| 创建表 | `CREATE TABLE t (id INT, name TEXT)` | DDL 执行 |
| 插入数据 | `INSERT INTO t VALUES (1, 'Alice')` | affected_rows=1 |
| 查询数据 | `SELECT * FROM t` | rows 包含插入数据 |
| 更新数据 | `UPDATE t SET name='Bob' WHERE id=1` | affected_rows=1 |
| 删除数据 | `DELETE FROM t WHERE id=1` | affected_rows=1 |
| 条件查询 | `SELECT * FROM t WHERE id=1` | 过滤后结果 |
| 解释计划 | `EXPLAIN SELECT * FROM t` | 返回查询计划 |
| 事务开始 | `BEGIN` | 事务状态 |
| 事务提交 | `COMMIT` | 数据持久化 |
| 事务回滚 | `ROLLBACK` | 数据回滚 |

---

*最后更新: 2026-03-28*
*版本: v1.9.0*
