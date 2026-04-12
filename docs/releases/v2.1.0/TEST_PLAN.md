# SQLRustGo v2.1.0 测试计划

> **版本**: v2.1.0
> **更新日期**: 2026-04-02
> **状态**: ✅ 完成
> **覆盖率目标**: ≥ 80%

---

## 1. 测试概述

v2.1.0 是性能优化和新功能增强版本，主要测试目标：

1. **功能完整性**: 69 个测试文件全部通过
2. **覆盖率达标**: 整体覆盖率 ≥ 80%
3. **TPC-H Q1-Q22**: 完整 22 个查询覆盖
4. **新功能验证**: AgentSQL/KILL/备份/PITR/HA

---

## 2. 测试组织结构

### 2.1 测试分类

```
tests/
├── regression_test.rs           # 综合回归测试入口 (70+ 测试文件)
├── unit/                       # 单元测试 (16 files)
├── integration/                # 集成测试 (40+ files)
├── performance/               # 性能测试 (7 files)
├── anomaly/                    # 异常测试 (20+ files)
├── security/                   # 安全测试 (2 files)
└── tools/                      # 工具测试 (2 files)
```

### 2.2 测试文件清单

| 类别 | 测试文件数 | 测试数 | 覆盖率 |
|------|-----------|--------|--------|
| 单元测试 | 16 | 100+ | 85% |
| 集成测试 | 40+ | 300+ | 82% |
| 性能测试 | 7 | 100+ | 80% |
| 异常测试 | 20+ | 200+ | 78% |
| 安全测试 | 2 | 89 | 86% |
| **总计** | **70+** | **800+** | **80.2%** |

---

## 3. 功能测试矩阵

### 3.1 核心模块测试

| 模块 | 测试文件 | 测试数 | 覆盖率 | 状态 |
|------|----------|--------|--------|------|
| **Parser** | | | | |
| SQL 解析 | parser_token_test | - | 88% | ✅ |
| KILL 语句 | mysql_compatibility_test | 17 | 90% | ✅ |
| 存储过程 | stored_proc_test | 20 | 80% | ✅ |
| **Executor** | | | | |
| Volcano 模型 | executor_test | - | 82% | ✅ |
| 向量化 | vectorization_test | 10 | 78% | ✅ |
| 并行执行 | parallel_executor_test | 8 | 75% | ✅ |
| **Optimizer** | | | | |
| CBO 成本 | optimizer_cost_test | - | 80% | ✅ |
| 优化规则 | optimizer_rules_test | - | 85% | ✅ |
| **Storage** | | | | |
| B+Tree | bplus_tree_test | - | 92% | ✅ |
| 列式存储 | columnar_storage_test | 12 | 80% | ✅ |
| Parquet | parquet_test | 7 | 82% | ✅ |
| WAL | wal_integration_test | 16 | 88% | ✅ |
| **Transaction** | | | | |
| MVCC | mvcc_concurrency_test | - | 85% | ✅ |
| 快照隔离 | snapshot_isolation_test | 17 | 82% | ✅ |
| 分布式事务 | distributed_transaction_test | 31 | 78% | ✅ |

### 3.2 SQL 功能覆盖

| 功能 | 测试覆盖 | 状态 |
|------|----------|------|
| SELECT/INSERT/UPDATE/DELETE | ✅ | ✅ |
| JOIN (INNER/LEFT/RIGHT/FULL) | ✅ | ✅ |
| 聚合 (COUNT/SUM/AVG/MIN/MAX) | ✅ | ✅ |
| GROUP BY/HAVING | ✅ | ✅ |
| 窗口函数 (ROW_NUMBER/RANK/LEAD/LAG) | ✅ | ✅ |
| 子查询 (标量/IN/EXISTS) | ✅ | ✅ |
| 集合操作 (UNION/INTERSECT/EXCEPT) | ✅ | ✅ |
| 事务 (BEGIN/COMMIT/ROLLBACK/SAVEPOINT) | ✅ | ✅ |
| 外键约束 (CASCADE/SET NULL/RESTRICT) | ✅ | ✅ |
| 视图 (CREATE/DROP/SELECT) | ✅ | ✅ |

---

## 4. 新增功能测试 (v2.1.0)

### 4.1 AgentSQL Extension (Issue #1128)

| PR | 测试数 | 测试文件 | 状态 |
|-----|--------|----------|------|
| #1140 | 13 | lib.rs | ✅ |
| #1141 | 19 | memory.rs, nl2sql.rs | ✅ |
| **总计** | **32** | | ✅ |

### 4.2 KILL/PROCESSLIST (Issue #1135)

| PR | 测试数 | 测试文件 | 状态 |
|-----|--------|----------|------|
| #1142 | 10 | mysql_compatibility_test.rs | ✅ |
| #1150 | 13 | parser.rs | ✅ |
| **总计** | **23** | | ✅ |

### 4.3 Health Endpoints (Issue #1139)

| PR | 测试数 | 测试文件 | 状态 |
|-----|--------|----------|------|
| #1144 | 10 | server_health_test.rs | ✅ |
| **总计** | **10** | | ✅ |

### 4.4 Backup & PITR (Issue #1133)

| PR | 测试数 | 测试文件 | 状态 |
|-----|--------|----------|------|
| #1143 | 18 | backup.rs, pitr_recovery.rs | ✅ |
| #1151 | - | backup_storage.rs | ✅ |
| **总计** | **40+** | | ✅ |

### 4.5 HA Cluster (Issue #1133)

| PR | 测试数 | 测试文件 | 状态 |
|-----|--------|----------|------|
| #1145 | 3 | ha.rs | ✅ |
| #1152 | 21 | read_write_split.rs, failover_manager.rs | ✅ |
| **总计** | **24+** | | ✅ |

### 4.6 存储过程控制流

| PR | 测试数 | 测试文件 | 状态 |
|-----|--------|----------|------|
| #1167 | 15 | stored_proc_control_flow.rs | ✅ |
| #1168 | 15 | stored_proc_control_flow.rs | ✅ |
| #1174 | 15 | stored_proc_control_flow.rs | ✅ |
| **总计** | **45** | | ✅ |

---

## 5. TPC-H 测试计划

### 5.1 测试执行

```bash
# 基础 TPC-H 测试
cargo test --test tpch_test

# TPC-H 基准对比 (SQLite)
cargo test --test tpch_benchmark

# 完整 TPC-H Q1-Q22
cargo test --test tpch_full_test
```

### 5.2 Q1-Q22 覆盖

| 查询 | 描述 | 测试状态 |
|------|------|----------|
| Q1-Q6 | 报表和统计 | ✅ |
| Q7-Q12 | 运输和优先级 | ✅ |
| Q13-Q18 | 客户和订单分析 | ✅ |
| Q19-Q22 | 供应商和市场 | ✅ |

### 5.3 性能目标

| 操作 | 目标 QPS |
|------|----------|
| Insert | 500k rows/s |
| Scan | 1M rows/s |
| Join | 200k rows/s |
| Aggregate | 1M rows/s |

---

## 6. 覆盖率测试

### 6.1 覆盖率目标

| 模块 | 目标覆盖率 | 当前覆盖率 | 状态 |
|------|-----------|-----------|------|
| Parser | 85% | 88% | ✅ |
| Executor | 80% | 82% | ✅ |
| Optimizer | 80% | 80% | ✅ |
| Storage | 85% | 85% | ✅ |
| Transaction | 80% | 82% | ✅ |
| Server | 80% | 85% | ✅ |
| Tools | 75% | 80% | ✅ |
| **总计** | **80%** | **80.2%** | ✅ |

### 6.2 覆盖率命令

```bash
# 生成 HTML 报告
cargo tarpaulin --out Html --report-dir coverage/

# 门槛检查 (失败如果 < 80%)
cargo tarpaulin --fail-under 80

# 完整命令
cargo tarpaulin --workspace --ignore-panics --timeout 120 --fail-under 80
```

---

## 7. 回归测试框架

### 7.1 运行方式

```bash
# 完整回归测试
cargo test --test regression_test -- --nocapture

# 单线程运行
cargo test --test regression_test -- --nocapture --test-threads=1

# 类别测试
cargo test --test regression_test -- --nocapture --test-threads=1 <category>
```

### 7.2 测试类别

| 类别 | 测试文件数 | 描述 |
|------|-----------|------|
| 单元测试 | 16 | 底层组件测试 |
| 集成测试 - 核心 | 3 | 执行器、规划器、页面 |
| 集成测试 - SQL功能 | 7 | 外键、UPSERT、保存点 |
| 集成测试 - 存储 | 6 | 列式、Parquet、缓存 |
| 性能测试 | 7 | TPC-H、批量插入、索引 |
| 异常测试 - 并发 | 3 | MVCC、快照隔离 |
| 异常测试 - 隔离级别 | 2 | 事务隔离、超时 |
| 异常测试 - 数据处理 | 5 | 边界、NULL、错误处理 |
| 异常测试 - 查询 | 4 | JOIN、视图、窗口函数 |
| 异常测试 - 约束 | 2 | 外键约束 |
| 压力测试 | 6 | 混沌、崩溃、WAL |
| 异常测试 - 稳定性 | 2 | 长时间运行 |
| 异常测试 - 崩溃注入 | 1 | 崩溃注入 |
| CI 测试 | 1 | CI 环境检查 |
| 其他测试 | 3 | 二进制、WAL、分布式 |
| 安全测试 | 2 | RBAC、日志 |
| 教学场景测试 | 2 | 客户端/服务器 |
| SQL CLI 测试 | 1 | UPDATE/DELETE |
| 工具测试 | 1 | 物理备份 |

---

## 8. 门禁检查

### 8.1 必须通过 (PR 合并前)

| 检查项 | 命令 | 标准 |
|--------|------|------|
| 代码格式 | `cargo fmt --check` | 无错误 |
| Clippy | `cargo clippy -- -D warnings` | 无警告 |
| 编译 | `cargo build --release` | 成功 |
| 单元测试 | `cargo test --lib` | 100% 通过 |
| 集成测试 | `cargo test --test integration_test` | 100% 通过 |
| 回归测试 | `cargo test --test regression_test` | 全部通过 |
| 覆盖率 | `cargo tarpaulin --fail-under 80` | ≥ 80% |

### 8.2 建议通过 (发布前)

| 检查项 | 命令 | 标准 |
|--------|------|------|
| TPC-H 基准 | `cargo test --test tpch_benchmark` | 11 tests |
| TPC-H 完整 | `cargo test --test tpch_full_test` | 28 tests |
| 向量化测试 | `cargo test --test vectorization_test` | 10 tests |
| 列式存储测试 | `cargo test --test columnar_storage_test` | 12 tests |
| 分布式事务 | `cargo test --test distributed_transaction_test` | 31 tests |
| 性能基准 | `cargo bench` | 无回归 |

---

## 9. 文档索引

| 文档 | 描述 |
|------|------|
| `v2.1.0-TEST-MATRIX.md` | 功能矩阵和测试详情 |
| `v2.1.0-test-report-summary.md` | 测试报告汇总 |
| `v2.1.0-tpch-benchmark-report.md` | TPC-H 基准报告 |
| `TEST_CHECKLIST.md` | 测试清单 |
| `RELEASE_GATES_COMPREHENSIVE.md` | 门禁检查清单 |

---

## 10. 版本对比

| 指标 | v1.9.0 | v2.1.0 | 提升 |
|------|--------|--------|------|
| 测试文件数 | 50+ | 70+ | +40% |
| 覆盖率 | 65% | 80% | +15% |
| TPC-H Q1-Q22 | 部分 | 完整 | +22 queries |
| 新功能测试 | 基础 | 完整 | +174 tests |

---

*最后更新: 2026-04-02*
*版本: v2.1.0*
*状态: ✅ 完成*
