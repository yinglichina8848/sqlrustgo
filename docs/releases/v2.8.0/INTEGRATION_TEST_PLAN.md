# v2.8.0 集成测试计划

> **版本**: v2.8.0 (GA)
> **创建日期**: 2026-05-02
> **基于**: TEST_PLAN.md + TEST_REPORT.md 实际结果
> **状态**: 已执行并验证 (2026-05-01)

---

## 一、测试总览

### 1.1 测试分层架构

```
┌─────────────────────────────────────────────────────────────┐
│               E2E 测试 (分布式场景)                          │
│        658 分布式集成测试 (100% PASS)                       │
├─────────────────────────────────────────────────────────────┤
│              集成测试 (模块间协作)                            │
│        CBO / WAL / E2E Query / E2E Observability            │
│        E2E Monitoring / Regression / Scheduler              │
│        216 单元+集成测试 (86.7%, 33 [ignore])               │
├─────────────────────────────────────────────────────────────┤
│              单元测试 (各模块)                                │
│        Parser / Executor / Storage / Transaction / Network   │
│        各 crate lib 测试                                     │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 v2.8.0 测试结果汇总

| 测试类别 | 总计 | 通过 | 失败 | 跳过 | 通过率 |
|----------|------|------|------|------|--------|
| **cargo test (单元+集成)** | 249 | 216 | 0 | 33 | **86.7%** |
| **分布式集成** | 658 | 658 | 0 | 0 | **100%** |
| **SQL Corpus 回归** | 426 | 174 | 252 | 0 | **40.8%** |
| **安全测试** | 81 | 81 | 0 | 0 | **100%** |
| **安全审计** | 1 | 0 漏洞 | - | - | **✅** |
| **GA 门禁** | 7 | 7 | 0 | 0 | **100%** |

---

## 二、单元测试计划

### 2.1 解析器测试 (parser)

| 测试组 | 用例数 | v2.8.0 状态 | 说明 |
|--------|--------|-------------|------|
| parser_token_test | 1 | ✅ PASS | Token 解析 |
| 基本 SELECT 解析 | 涵盖在回归测试 | ✅ | SELECT/WHERE/ORDER BY |
| INSERT/UPDATE/DELETE | 涵盖在回归测试 | ✅ | DML 解析 |
| DDL (CREATE/DROP/ALTER) | 涵盖在回归测试 | ✅ | DDL 解析 |
| FULL OUTER JOIN | 3 | ✅ PASS | 语法解析 (full_outer_join_test.rs) |
| TRUNCATE TABLE | 1 | ✅ PASS | TRUNCATE 解析 |
| REPLACE INTO | 2 | ✅ PASS | REPLACE 解析 |

### 2.2 执行器测试 (executor)

| 测试组 | 用例数 | v2.8.0 状态 | 说明 |
|--------|--------|-------------|------|
| aggregate_functions_test | 9 | ✅ PASS | SUM/AVG/COUNT/MIN/MAX |
| regression_test | 16 | ✅ PASS | 回归测试 |
| distinct_test | 6 | ✅ PASS | DISTINCT 查询 |
| expression_operators_test | 3 | ✅ PASS | 表达式运算符 |
| in_value_list_test | 4 | ⚠️ [ignore] | IN 值列表测试跳过 |
| limit_clause_test | 10 | ⚠️ [ignore] | LIMIT 子句测试跳过 |

### 2.3 存储测试 (storage)

| 测试组 | 用例数 | v2.8.0 状态 | 说明 |
|--------|--------|-------------|------|
| buffer_pool_test | 16 | ✅ PASS | BufferPool 页面置换/脏页 |
| buffer_pool_benchmark_test | 7 | ✅ PASS | BufferPool 性能基准 |
| page_io_benchmark_test | 10 | ⚠️ [ignore] | 需要真实 IO |
| data_loader | 1 | ✅ PASS | 数据加载器 |

### 2.4 事务测试 (transaction)

| 测试组 | 用例数 | v2.8.0 状态 | 说明 |
|--------|--------|-------------|------|
| mvcc_transaction_test | 4 | ✅ PASS | MVCC 快照隔离 |
| crash_recovery_test | 8 | ✅ PASS | WAL 崩溃恢复 |
| wal_integration_test | 0 | N/A | WAL 集成测试无用例 |
| long_run_stability_test | 8 | ✅ PASS | 长稳测试 |
| long_run_stability_72h_test | 6 | ✅ PASS | 72 小时长稳测试 |

### 2.5 网络与服务器测试

| 测试组 | 用例数 | v2.8.0 状态 | 说明 |
|--------|--------|-------------|------|
| scheduler_integration_test | 10 | ⚠️ 4/10 | 40% 实际通过 (6 [ignore]) |
| stored_proc_catalog_test | 16 | ✅ PASS | 存储过程目录 |
| stored_procedure_parser_test | 0 | N/A | 解析测试无用例 |

### 2.6 E2E 与可观测性测试

| 测试组 | 用例数 | v2.8.0 状态 | 说明 |
|--------|--------|-------------|------|
| e2e_query_test | 7 | ✅ PASS | 端到端查询流程 |
| e2e_observability_test | 7 | ✅ PASS | 可观测性测试 |
| e2e_monitoring_test | 8 | ✅ PASS | 监控测试 |
| cbo_integration_test | 12 | ✅ PASS | 成本优化器集成 |
| ci_test | 5 | ✅ PASS | CI 冒烟测试 |
| qps_benchmark_test | 22 | ✅ PASS | QPS 基准测试 |
| concurrency_stress_test | 9 | ✅ PASS | 并发压力测试 |

### 2.7 跳过的测试 ([ignore] 标记)

| 测试文件 | 跳过数 | 原因 | 处理计划 |
|---------|--------|------|---------|
| in_value_list_test | 4 | #[ignore] 标记 | v2.9.0 修复 |
| limit_clause_test | 10 | #[ignore] 标记 | v2.9.0 修复 |
| page_io_benchmark_test | 10 | 需要真实 IO 设备 | v2.9.0 恢复 |
| scheduler_integration_test | 6 | #[ignore] 标记 | v2.9.0 修复 |
| boundary_test | 3 | #[ignore] 标记 | v2.9.0 修复 |
| **合计** | **33** | | |

---

## 三、分布式集成测试计划

### 3.1 测试架构

分布式集成测试位于 `crates/distributed/tests/`，基于 Tokio 异步运行时。

| 测试文件 | 模块 | 用例数 | v2.8.0 状态 |
|----------|------|--------|-------------|
| `distributed_integration_test.rs` | 核心集成 | 314 | ✅ 全部 PASS |
| `distributed_e2e_test.rs` | 端到端 | 273 | ✅ 全部 PASS |
| `read_write_splitter_test.rs` | 读写分离 | 71 | ✅ 全部 PASS |
| **合计** | | **658** | **100% PASS** |

### 3.2 分布式功能模块

#### 3.2.1 分区表 (Partition)

| 测试项 | 用例数 | 状态 |
|--------|--------|------|
| Hash 分区均匀性 | 15 | ✅ |
| Range 分区 | 10 | ✅ |
| List 分区 | 10 | ✅ |
| 分区裁剪 | 15 | ✅ |
| 多分区查询 | 10 | ✅ |
| 分区键验证 | 15 | ✅ |
| **小计** | **75** | **✅ 100%** |

#### 3.2.2 主从复制 (Replication)

| 测试项 | 用例数 | 状态 |
|--------|--------|------|
| GTID 管理 (添加/查询/包含) | 15 | ✅ |
| GTID 集合操作 (并集/交集) | 10 | ✅ |
| Semi-sync 管理 (添加/ACK/移除) | 10 | ✅ |
| 数据一致性验证 | 20 | ✅ |
| GTID 主从同步 | 14 | ✅ |
| 延迟复制 | 10 | ✅ |
| **小计** | **79** | **✅ 100%** |

#### 3.2.3 故障转移 (Failover)

| 测试项 | 用例数 | 状态 |
|--------|--------|------|
| 主节点检测 | 10 | ✅ |
| 自动切换 | 10 | ✅ |
| 手动切换 | 8 | ✅ |
| 脑裂防护 | 7 | ✅ |
| 故障恢复 | 10 | ✅ |
| 仲裁机制 | 10 | ✅ |
| **小计** | **55** | **✅ 100%** |

#### 3.2.4 读写分离 (Read-Write Splitter)

| 测试项 | 用例数 | 状态 |
|--------|--------|------|
| SELECT 路由到从库 | 5 | ✅ |
| INSERT/UPDATE/DELETE 路由到主库 | 5 | ✅ |
| SHOW/DESCRIBE 路由到从库 | 4 | ✅ |
| START/BEGIN/COMMIT 特殊处理 | 5 | ✅ |
| 复杂查询分类 | 4 | ✅ |
| 事务内读写路由 | 4 | ✅ |
| **小计** | **27** | **✅ 100%** |

#### 3.2.5 其他分布式功能

| 测试项 | 用例数 | 状态 |
|--------|--------|------|
| 2PC 两阶段提交 | 50 | ✅ |
| 分片路由 (Shard Router) | 60 | ✅ |
| 分片管理 (Shard Manager) | 55 | ✅ |
| 负载均衡 | 40 | ✅ |
| 一致性协议 (Consensus/Raft) | 45 | ✅ |
| 跨分片查询 | 55 | ✅ |
| 分布式锁 | 40 | ✅ |
| gRPC 通信 | 35 | ✅ |
| 副本管理 (Replica Sync) | 42 | ✅ |
| **小计** | **422** | **✅ 100%** |

---

## 四、SQL 回归测试计划

详见 [SQL_REGRESSION_PLAN.md](./SQL_REGRESSION_PLAN.md)

| 指标 | v2.8.0 实际 | v2.9.0 目标 |
|------|-------------|-------------|
| SQL 文件数 | 103 | 103 |
| 测试用例 | 426 | 426 |
| 通过 | 174 | 328+ |
| 通过率 | **40.8%** | **≥ 77%** |
| 主要瓶颈 | 函数调用/CASE/子查询 | 修复解析器语法 |

---

## 五、安全测试计划

### 5.1 SQL 注入防护

| 测试组 | 用例数 | v2.8.0 状态 |
|--------|--------|-------------|
| UNION 注入 | 20 | ✅ PASS |
| OR 注入 | 20 | ✅ PASS |
| DROP 注入 | 10 | ✅ PASS |
| 注释注入 | 15 | ✅ PASS |
| EXEC 注入 | 16 | ✅ PASS |
| **合计** | **81** | **✅ 100%** |

### 5.2 安全审计

```bash
cargo audit: 0 vulnerabilities ✅
```

---

## 六、GA 门禁检查

| Gate | 要求 | v2.8.0 实际 | 状态 |
|------|------|-------------|------|
| cargo test (all) | 无失败 | 216 PASS / 0 FAIL / 33 IGNORE | ✅ |
| verification_engine | baseline_verified | VERIFIED (249 tests) | ✅ |
| self_audit | proof_match | TRUSTED | ✅ |
| run_hermes_gate | PASS | PASS | ✅ |
| cargo audit | 0 vulnerabilities | 0 ✅ | ✅ |
| cargo fmt | 格式正确 | ✅ | ✅ |
| cargo clippy | 0 warnings | ✅ | ✅ |
| SQL Corpus | 回归测试 | 174/426 (40.8%) | ⚠️ 已知限制 |

---

## 七、测试执行命令

### 7.1 全量测试

```bash
# 构建
cargo build --all-features

# 全量单元+集成测试
cargo test --all-features

# 分布式测试
cargo test -p sqlrustgo-distributed --all-features

# SQL Corpus 回归
cargo test -p sqlrustgo-sql-corpus --all-features

# 安全测试
cargo test -p sqlrustgo-security --all-features

# 安全审计
cargo audit

# 格式检查
cargo fmt --check --all

# Lint
cargo clippy --all-features -- -D warnings
```

### 7.2 分类测试

```bash
# 核心功能
cargo test --test regression_test
cargo test --test cbo_integration_test
cargo test --test e2e_query_test

# 存储
cargo test --test buffer_pool_test
cargo test --test buffer_pool_benchmark_test
cargo test --test data_loader

# 事务
cargo test --test mvcc_transaction_test
cargo test --test crash_recovery_test

# 网络
cargo test -p sqlrustgo-server --test scheduler_integration_test

# 分布式模块
cargo test -p sqlrustgo-distributed --all-features -- test_partition
cargo test -p sqlrustgo-distributed --all-features -- test_replication
cargo test -p sqlrustgo-distributed --all-features -- test_failover
cargo test -p sqlrustgo-distributed --all-features -- test_read_write

# 长稳
cargo test --test long_run_stability_test
cargo test --test long_run_stability_72h_test
```

### 7.3 门禁脚本

```bash
# 一键门禁检查
bash scripts/gate/check_docs_links.sh
bash scripts/gate/check_coverage.sh
bash scripts/gate/check_security.sh
```

---

## 八、测试环境

| 项目 | 值 |
|------|-----|
| 测试机器 | HP Z6G4 Server / macOS |
| CPU | Intel Xeon (多核) |
| Rust 版本 | 1.94.1 |
| Cargo 版本 | 1.94.1 |
| 测试工具 | cargo test --all-features |
| 存储 | 内存存储 (MemoryStorage) + BufferPool |
| 网络 | Tokio 异步运行时 |

---

## 九、已知问题与风险

### 9.1 关键风险

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 33 个 [ignore] 测试 | 覆盖盲区 | v2.9.0 恢复 |
| SQL Corpus 40.8% | SQL 兼容性低 | 已记录为已知限制 |
| 未安装 go-tpc | TPC-H 基准无法执行 | 手动安装后补充 |
| 未安装 sysbench | OLTP 性能基准无法执行 | 手动安装后补充 |
| 未安装 mysqlslap | 并发基准无法执行 | 手动安装后补充 |
| 无 SIMD 性能对比 | 性能加速比未验证 | 需专用测试工具 |

### 9.2 未覆盖的测试场景

| 场景 | 原因 | 优先级 |
|------|------|--------|
| TPC-H SF=1 (Q1 < 2s) | go-tpc 未安装 | 高 |
| Sysbench OLTP (1000 QPS) | sysbench 未安装 | 高 |
| mysqlslap (500 并发) | mysqlslap 未安装 | 中 |
| SIMD 加速比 (≥ 2x) | 无专用测试 | 高 |
| Hash Join 并行化 (≥ 1.5x) | 无性能对比测试 | 中 |
| CPU 利用率 < 80% | 未监控 | 低 |
| 内存带宽 < 50 GB/s | 未监控 | 低 |

---

## 十、TEST_PLAN 对齐状态

### 10.1 已完成项目

| TEST_PLAN 条目 | 验证方式 | 状态 |
|----------------|---------|------|
| SELECT 解析 | 回归测试 + e2e_query_test | ✅ |
| INSERT/UPDATE/DELETE | parser_token_test | ✅ |
| FULL OUTER JOIN | full_outer_join_test (3 tests) | ✅ |
| TRUNCATE TABLE | parser_coverage_tests | ✅ |
| REPLACE INTO | parser_coverage_tests (2 tests) | ✅ |
| 聚合函数 | aggregate_functions_test (9) ✅ | ✅ |
| JOIN (INNER/LEFT/RIGHT) | e2e_query_test ✅ | ✅ |
| ORDER BY / GROUP BY | regression_test + aggregate | ✅ |
| 分区表 | distributed (75 PASS) ✅ | ✅ |
| GTID/主从复制 | distributed (79 PASS) ✅ | ✅ |
| 故障转移 | distributed (55 PASS) ✅ | ✅ |
| 读写分离 | distributed (27 PASS) ✅ | ✅ |
| MVCC 事务 | mvcc_transaction_test (4) ✅ | ✅ |
| WAL 恢复 | crash_recovery_test (8) ✅ | ✅ |
| BufferPool | buffer_pool_test (16) ✅ | ✅ |
| 存储过程 | stored_proc_catalog_test (16) ✅ | ✅ |
| SIMD 向量化 | SIMD tests (5/5) ✅ | ✅ |
| SQL 注入防护 | security (81 tests) ✅ | ✅ |

### 10.2 部分完成/待完成

| TEST_PLAN 条目 | 当前状态 | 差距 |
|----------------|---------|------|
| 窗口函数 | ⚠️ | 解析器支持，但 SQL Corpus 失败 |
| TPC-H 基准 | ❌ | go-tpc 未安装 |
| Sysbench OLTP | ❌ | sysbench 未安装 |
| mysqlslap 并发 | ❌ | 工具未安装 |
| SIMD 性能对比 | ❌ | 无专用性能对比测试 |
| Hash Join 并行化 | ❌ | 无性能对比测试 |
| 覆盖率达 80% | ⚠️ | 未用 tarpaulin 验证 |

---

## 十一、v2.9.0 测试改进计划

| 优先级 | 改进项 | 预期效果 |
|--------|--------|---------|
| P0 | 恢复 33 个 [ignore] 测试 | 通过率提升至 100% |
| P0 | 修复函数调用语法 (SQL Corpus) | 通过率从 40.8% → 77% |
| P0 | 支持 CASE/子查询/窗口函数 | SQL 兼容性大幅提升 |
| P1 | 安装 go-tpc 并执行 TPC-H SF=1 | 性能基准建立 |
| P1 | 安装 sysbench 并执行 OLTP | 性能基准建立 |
| P1 | 添加 SIMD 性能对比测试 | 验证 ≥ 2x 加速比 |
| P2 | 安装 cargo-tarpaulin 验证覆盖率 | 确认 ≥ 80% 覆盖率 |
| P2 | 添加 Hash Join 并行化性能测试 | 验证 ≥ 1.5x 加速比 |

---

## 十二、变更记录

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-05-02 | 基于 v2.8.0 GA 测试结果创建 |

---

*本文档由 Hermes Agent 自动生成*
*测试日期: 2026-05-01*
*GA Commit: 64ed7a5e*
