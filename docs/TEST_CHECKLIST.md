# SQLRustGo v2.1 测试清单

> **日期**: 2026-04-01
> **版本**: v2.1.0
> **状态**: ✅ 完整

---

## 一、测试文件总览

### 1.1 Cargo.toml 注册的测试 (69 tests)

| 测试文件 | 测试数 | 回归测试 | 描述 |
|----------|--------|----------|------|
| **单元测试** ||||
| `aggregate_type_test` | - | ✅ | 聚合类型测试 |
| `backup_test` | 17 | ✅ | 备份功能测试 |
| `bplus_tree_test` | - | ✅ | B+树测试 |
| `buffer_pool_test` | - | ✅ | 缓冲区池测试 |
| `file_storage_test` | - | ✅ | 文件存储测试 |
| `local_executor_test` | - | ✅ | 本地执行器测试 |
| `mysqldump_test` | 45 | ✅ | mysqldump 导入测试 |
| `optimizer_cost_test` | - | ✅ | 优化器成本测试 |
| `optimizer_rules_test` | - | ✅ | 优化器规则测试 |
| `parser_token_test` | - | ✅ | 解析器标记测试 |
| `prometheus_test` | - | ✅ | Prometheus 指标测试 |
| `query_cache_config_test` | - | ✅ | 查询缓存配置测试 |
| `query_cache_test` | - | ✅ | 查询缓存测试 |
| `server_health_test` | - | ✅ | 健康检查端点测试 |
| `slow_query_log_test` | - | ✅ | 慢查询日志测试 |
| `types_value_test` | - | ✅ | 类型值测试 |
| `vectorization_test` | 10 | ✅ | 向量化执行测试 |
| **集成测试** ||||
| `autoinc_test` | 4 | ✅ | 自增测试 |
| `batch_insert_test` | 9 | ✅ | 批量插入测试 |
| `binary_format_test` | - | ✅ | 二进制格式测试 |
| `boundary_test` | - | ✅ | 边界条件测试 |
| `catalog_consistency_test` | - | ✅ | 目录一致性测试 |
| `checksum_corruption_test` | - | ✅ | 校验和完整性测试 |
| `columnar_storage_test` | 12 | ✅ | 列式存储测试 |
| `datetime_type_test` | - | ✅ | 日期时间类型测试 |
| `distributed_transaction_test` | 31 | ✅ | 分布式事务测试 |
| `error_handling_test` | - | ✅ | 错误处理测试 |
| `executor_test` | - | ✅ | 执行器测试 |
| `fk_actions_test` | 5 | ✅ | 外键动作测试 |
| `fk_constraint_test` | - | ✅ | 外键约束测试 |
| `foreign_key_test` | - | ✅ | 外键测试 |
| `index_integration_test` | 13 | ✅ | 索引集成测试 |
| `join_test` | - | ✅ | JOIN 测试 |
| `mysql_compatibility_test` | - | ✅ | MySQL 兼容性测试 |
| `mysql_tpch_test` | 4 | ❌ | MySQL TPC-H (需要 MySQL) |
| `page_test` | - | ✅ | 页面测试 |
| `parquet_test` | 7 | ✅ | Parquet 导入导出测试 |
| `performance_test` | - | ✅ | 性能测试 |
| `planner_test` | - | ✅ | 规划器测试 |
| `savepoint_test` | 4 | ✅ | 保存点测试 |
| `server_integration_test` | - | ✅ | 服务器集成测试 |
| `session_config_test` | 4 | ✅ | 会话配置测试 |
| `set_operations_test` | - | ✅ | 集合操作测试 |
| `snapshot_isolation_test` | 17 | ✅ | 快照隔离测试 |
| `storage_integration_test` | 12 | ✅ | 存储集成测试 |
| `teaching_scenario_client_server_test` | - | ✅ | 教学场景客户端/服务器测试 |
| `teaching_scenario_test` | - | ✅ | 教学场景测试 |
| `tpch_benchmark` | 11 | ✅ | TPC-H 基准对比测试 |
| `tpch_full_test` | 28 | ✅ | TPC-H Q1-Q22 完整测试 |
| `tpch_test` | 5 | ✅ | TPC-H 基础测试 |
| `transaction_isolation_test` | - | ✅ | 事务隔离级别测试 |
| `transaction_timeout_test` | - | ✅ | 事务超时测试 |
| `types_value_test` | - | ✅ | 类型值测试 |
| `upsert_test` | - | ✅ | UPSERT 测试 |
| `view_test` | - | ✅ | 视图测试 |
| `wal_integration_test` | 16 | ✅ | WAL 集成测试 |
| `window_function_test` | 21 | ✅ | 窗口函数测试 |
| **异常测试** ||||
| `auth_rbac_test` | 23 | ✅ | RBAC 权限测试 |
| `chaos_test` | - | ✅ | 混沌工程测试 |
| `ci_test` | - | ✅ | CI 环境测试 |
| `concurrency_stress_test` | - | ✅ | 并发压力测试 |
| `crash_injection_test` | - | ✅ | 崩溃注入测试 |
| `crash_recovery_test` | - | ✅ | 崩溃恢复测试 |
| `long_run_stability_test` | - | ✅ | 长时间稳定性测试 |
| `logging_test` | 66 | ✅ | 日志测试 |
| `mvcc_concurrency_test` | - | ✅ | MVCC 并发测试 |
| `null_handling_test` | - | ✅ | NULL 处理测试 |
| `physical_backup_test` | - | ✅ | 物理备份测试 |
| `qps_benchmark_test` | - | ✅ | QPS 基准测试 |
| `stress_test` | - | ✅ | 压力测试 |
| `wal_deterministic_test` | 10 | ✅ | WAL 确定性测试 |
| `wal_fuzz_test` | 10 | ✅ | WAL 模糊测试 |
| **其他** ||||
| `query_cache_benchmark` | - | ❌ | 查询缓存基准 (独立运行) |
| `regression_test` | 1 | ✅ | 回归测试入口 |

---

## 二、TPC-H 测试详解

### 2.1 测试文件说明

| 文件 | 测试数 | 描述 | 运行要求 |
|------|--------|------|----------|
| `tpch_test.rs` | 5 | SQLRustGo TPC-H 基础测试 | 无 |
| `tpch_benchmark.rs` | 11 | SQLRustGo vs SQLite 对比测试 | 无 |
| `tpch_full_test.rs` | 28 | 完整 TPC-H Q1-Q22 + MySQL/PostgreSQL | PostgreSQL 可选 |
| `mysql_tpch_test.rs` | 4 | MySQL TPC-H 测试 | MySQL 服务器 |

### 2.2 运行方式

```bash
# 运行所有 TPC-H 测试
cargo test --test tpch_test
cargo test --test tpch_benchmark
cargo test --test tpch_full_test

# MySQL TPC-H (需要 MySQL 服务器)
MYSQL_HOST=localhost MYSQL_USER=root cargo test --test mysql_tpch_test
```

### 2.3 TPC-H Q1-Q22 覆盖

| 查询 | 描述 | 测试状态 |
|------|------|----------|
| Q1 | Pricing Summary Report | ✅ |
| Q2 | Minimum Cost Supplier | ✅ |
| Q3 | Shipping Priority | ✅ |
| Q4 | Order Priority Checking | ✅ |
| Q5 | Local Supplier Volume | ✅ |
| Q6 | Forecast Revenue Change | ✅ |
| Q7 | Volume Shipping | ✅ |
| Q8 | National Market Share | ✅ |
| Q9 | Product Type Profit | ✅ |
| Q10 | Returned Item Reporting | ✅ |
| Q11 | Important Stock Identification | ✅ |
| Q12 | Shipping Modes and Order Priority | ✅ |
| Q13 | Customer Distribution | ✅ |
| Q14 | Promotion Effect | ✅ |
| Q15 | Top Supplier | ✅ |
| Q16 | Parts/Supplier Relationship | ✅ |
| Q17 | Small-Order Revenue | ✅ |
| Q18 | Large Value Customers | ✅ |
| Q19 | Discounted Revenue | ✅ |
| Q20 | Potential Part Promotion | ✅ |
| Q21 | Suppliers Who Kept Orders Waiting | ✅ |
| Q22 | Global Sales Opportunity | ✅ |

---

## 三、基准测试 (benches/)

### 3.1 微基准测试

| 文件 | 描述 |
|------|------|
| `lexer_bench.rs` | 词法分析器性能 |
| `parser_bench.rs` | 语法分析器性能 |
| `executor_bench.rs` | 执行器性能 |
| `storage_bench.rs` | 存储引擎性能 |

### 3.2 集成基准测试

| 文件 | 描述 |
|------|------|
| `integration_bench.rs` | 端到端性能 |
| `network_bench.rs` | 网络层性能 |

### 3.3 性能目标

| 操作 | 目标 | 实际 |
|------|------|------|
| Insert | 500k rows/s | - |
| Scan | 1M rows/s | - |
| Join | 200k rows/s | - |
| Aggregate | 1M rows/s | - |

---

## 四、回归测试运行

### 4.1 运行完整回归测试

```bash
# 运行回归测试 (包含所有回归测试类别)
cargo test --test regression_test

# 单个测试
cargo test --test backup_test
cargo test --test mysqldump_test
cargo test --test tpch_benchmark
```

### 4.2 回归测试类别

| 类别 | 测试数 | 描述 |
|------|--------|------|
| 单元测试 | 16 | 底层组件测试 |
| 集成测试 - 核心 | 3 | 执行器、规划器、页面 |
| 集成测试 - SQL功能 | 10 | 外键、UPSERT、保存点 |
| 集成测试 - 存储 | 6 | 缓存、列式、Parquet |
| 性能测试 | 7 | TPC-H、批量插入、索引 |
| 异常测试 - 并发 | 3 | MVCC、快照隔离 |
| 异常测试 - 隔离级别 | 2 | 事务隔离、超时 |
| 异常测试 - 数据处理 | 5 | 边界、NULL、错误处理 |
| 异常测试 - 查询 | 4 | JOIN、视图、窗口函数 |
| 异常测试 - 约束 | 2 | 外键约束 |
| 压力测试 | 6 | 混沌、崩溃、WAL |
| 异常测试 - 稳定性 | 2 | 长时间运行 |
| CI 测试 | 1 | CI 环境检查 |
| 其他测试 | 3 | 二进制、WAL、分布式 |
| 安全测试 | 2 | RBAC、日志 |
| 工具测试 | 3 | 物理备份、mysqldump |

---

## 五、门禁检查

### 5.1 必须通过 (PR 合并前)

| 检查项 | 命令 | 标准 |
|--------|------|------|
| 代码格式 | `cargo fmt --check` | 无错误 |
| Clippy | `cargo clippy` | 无警告 |
| 编译 | `cargo build --release` | 成功 |
| 单元测试 | `cargo test --lib` | 100% 通过 |
| 集成测试 | `cargo test --test integration_test` | 100% 通过 |
| 回归测试 | `cargo test --test regression_test` | 1 test (全部通过) |

### 5.2 建议通过 (发布前)

| 检查项 | 命令 | 标准 |
|--------|------|------|
| TPC-H 基准 | `cargo test --test tpch_benchmark` | 11 tests |
| TPC-H 完整 | `cargo test --test tpch_full_test` | 28 tests |
| 向量化测试 | `cargo test --test vectorization_test` | 10 tests |
| 列式存储测试 | `cargo test --test columnar_storage_test` | 12 tests |
| 分布式事务 | `cargo test --test distributed_transaction_test` | 31 tests |
| 性能基准 | `cargo bench` | 无回归 |

---

## 六、文档

| 文档 | 描述 |
|------|------|
| `v2.1.0-test-report-summary.md` | 测试报告汇总 |
| `v2.1.0-tpch-benchmark-report.md` | TPC-H 基准测试报告 |
| `v2.0/BENCHMARK_FRAMEWORK.md` | 性能测试框架设计 |
| `RELEASE_GATES_COMPREHENSIVE.md` | 门禁检查清单 |

---

*最后更新: 2026-04-01*
