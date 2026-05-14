# SQLRustGo v3.0.0 历史遗留开发计划

> **版本**: 1.0
> **日期**: 2026-05-07
> **基于**: Issue #353, ALPHA_GAPS.md, WEAKNESS_ANALYSIS.md (v2.9.0)
> **状态**: 🔴 未启动

---

## 一、背景与目的

本文档整合了从 v2.9.0 到 v3.0.0 规划但尚未完成的全部功能和测试，识别 MySQL 兼容性差距，并为 Beta/RC/GA 门禁提供完整清单。

**数据来源**:
- Issue #353 (v3.0.0 开发总控)
- `docs/releases/v2.9.0/WEAKNESS_ANALYSIS.md` (MySQL 兼容性差距分析)
- `docs/releases/v3.0.0/ALPHA_GAPS.md` (Alpha 阶段整改清单)
- `docs/releases/v3.0.0/BETA_GATE_MASTER_CONTROL.md` (Beta 门禁任务总控)
- `scripts/gate/check_beta_v300.sh` / `check_rc_v300.sh` / `check_ga_v300.sh`

---

## 二、已完成任务清单 (v3.0.0)

| 任务 | 标题 | 状态 | 验证方式 |
|------|------|------|----------|
| F-01 | CBO 规则桥接 | ✅ | 86 测试通过 |
| F-02 | 查询缓存完善 (LRU + DML 失效) | ✅ | opencode |
| F-03 | 连接池 (Thread Pool) | ✅ | opencode |
| F-04 | Group Commit WAL 批量 | ✅ | opencode |
| F-05 | INSERT...SELECT | ✅ | INSERT...SELECT 测试 |
| F-06 | 窗口函数 (NTILE/LEAD/LAG/FIRST_VALUE/LAST_VALUE/NTH_VALUE) | ✅ | 6 函数测试 |
| F-07 | CTE 执行 (WITH 子句 + inlining) | ✅ | CTE 测试 |
| I-01 | INFORMATION_SCHEMA (SHOW TABLES/COLUMNS/DESCRIBE) | ✅ | SQL 测试 |
| I-02 | EXPLAIN ANALYZE | ✅ | EXPLAIN 测试 |
| I-03 | SSL/TLS (rustls + 自签名证书) | ✅ | TLS 连接测试 |
| I-04 | 慢查询日志 | ✅ | 慢查询测试 |
| I-05 | CI Gate 完善 (TPC-H + coverage-trend) | ✅ | CI pipeline |
| I-06 | SHOW VARIABLES (15 个变量) | ✅ | SHOW VARIABLES 测试 |
| I-07 | 运维手册 | ✅ | docs/OPERATIONS_MANUAL.md |
| I-08 | ADR (5 条架构决策) | ✅ | docs/ARCHITECTURE_DECISIONS.md |
| I-09 | COM_MULTI 多语句执行 | ✅ | PR #388 |
| I-10 | Prepared Statement 参数绑定修复 | ✅ | PR #388 |
| A-02 | API 版本化 + deprecated | ✅ | deprecated API 测试 |
| A-03 | v2.9→v3.0 迁移指南 | ✅ | docs/MIGRATION_GUIDE_v3.md |
| A-04 | 教学模式 | ✅ | SQLRUSTGO_TEACHING_MODE |
| A-05 | 在线 DDL (ADD/DROP/MODIFY/RENAME) | ✅ | ALTER TABLE 测试 |
| A-06 | mysqldump 导出 | ✅ | mysqldump 测试 |
| A-07 | 性能调优指南 | ✅ | docs/PERFORMANCE_TUNING.md |
| PP-04 | 内存治理 (512MB 限额) | ✅ | MemoryTracker |
| PP-06 | Group Commit | ✅ | WAL 批量 fsync |
| PROOF-026 | Write Skew/SSI (TLA+ 模型 + 7 测试) | ✅ | formal proof |
| T-01 | Sysbench OLTP 兼容性 | ✅ | 3 场景 QPS 验证 |
| SQL Corpus | 485/485 全部通过 | ✅ | 100% 覆盖率 |

---

## 三、未完成功能任务

### 3.1 P0 - 必须完成 (阻塞 Beta Gate)

#### T-02: 事务状态机压力测试
|| 属性 | 值 |
||------|-----|
|| **Issue** | #379 |
|| **优先级** | P0 |
|| **负责人** | claude |
|| **工时** | 2d |
|| **现状** | 🔄 进行中 |
|| **测试文件** | `tests/mvcc_transaction_test.rs` |
|| **验收条件** | 100 并发 BEGIN/COMMIT/ROLLBACK 无状态泄漏 |
|| **测试命令** | `cargo test --test mvcc_transaction_test` |
|| **门禁映射** | B-S2 (崩溃恢复) |

#### T-03: Optimizer 测试扩展
|| 属性 | 值 |
||------|-----|
|| **Issue** | #380 |
|| **优先级** | P1 |
|| **负责人** | claude |
|| **工时** | 2d |
|| **现状** | 🔄 进行中 |
|| **测试文件** | `tests/optimizer_cbo_accuracy_test.rs` + `tests/cbo_integration_test.rs` |
|| **验收条件** | optimizer 覆盖率 ≥70%, Predicate Pushdown + Projection Pruning |
|| **测试命令** | `cargo test -p sqlrustgo-optimizer` + `cargo llvm-cov` |
|| **门禁映射** | B5 (覆盖率 ≥75%), B12 (CBO Optimizer Tests) |

#### T-04: Planner 测试扩展
|| 属性 | 值 |
||------|-----|
|| **Issue** | #381 |
|| **优先级** | P1 |
|| **负责人** | claude |
|| **工时** | 2d |
|| **现状** | 🔄 进行中 |
|| **测试文件** | `tests/planner_multi_join_test.rs` + `tests/cbo_integration_test.rs` |
|| **验收条件** | planner 覆盖率 ≥80%, SELECT/INSERT/UPDATE/DELETE/JOIN 计划正确 |
|| **测试命令** | `cargo test -p sqlrustgo-planner` + `cargo llvm-cov` |
|| **门禁映射** | B5 (覆盖率 ≥75%), B13 (CBO Planner Tests) |

#### M-02: TPC-H SF=1 CI Gate
|| 属性 | 值 |
||------|-----|
|| **Issue** | #382 |
|| **优先级** | P1 |
|| **现状** | 🔄 进行中 |
|| **测试文件** | `tests/tpch_gate_test.rs` |
|| **验收条件** | `check_tpch.sh sf=1` 可运行，22/22 无 OOM |
|| **测试命令** | `bash scripts/gate/check_tpch.sh sf=1` |
|| **门禁映射** | B8 (TPC-H SF=0.1 22/22), R9 (TPC-H SF=1 22/22), GA-9 (TPC-H SF=1 22/22) |

### 3.2 P1 - 建议 Beta 前完成

#### D-01: CBO 代价模型集成
| 属性 | 值 |
|------|-----|
| **Issue** | #392 |
| **优先级** | P0 (阻塞性能目标) |
| **负责人** | opencode |
| **工时** | 5-7d |
| **现状** | 🔄 进行中 |
| **验收条件** | SimpleCostModel 接入 planner, EXPLAIN 能选择索引扫描而非全表扫描 |
| **目标** | Point SELECT QPS ≥20,000 (当前 ~7,312) |

#### S-01: 连接池并发压力测试
| 属性 | 值 |
|------|-----|
| **优先级** | P1 |
| **现状** | 标记完成但无压力测试 |
| **验收条件** | 8/32/100 线程并发无连接泄漏 |
| **测试命令** | `cargo test --test connection_pool_stress_test` |
| **问题** | `test_high_contention_stress` 存在 overflow bug |

#### S-02: 查询缓存 DML 失效测试
| 属性 | 值 |
|------|-----|
| **优先级** | P1 |
| **现状** | 标记完成但无验证测试 |
| **验收条件** | DML 后缓存正确失效 |
| **测试命令** | `cargo test --test query_cache_test` |

#### S-03: Group Commit WAL 崩溃恢复测试
| 属性 | 值 |
|------|-----|
| **优先级** | P1 |
| **现状** | 标记完成但无崩溃恢复验证 |
| **验收条件** | kill -9 后重启数据完整 |
| **测试命令** | `cargo test --test wal_crash_recovery_test` |

### 3.3 P2 - 建议 RC 前完成

#### M-01: 存储引擎增强
| 功能 | MySQL | v2.9.0 | v3.0.0 | 状态 |
|------|-------|--------|---------|------|
| 聚簇索引 (Clustered Index) | ✅ | ❌ | ❌ | P2 |
| 自适应哈希索引 (AHI) | ✅ | ❌ | ❌ | P2 |
| Change Buffer | ✅ | ⚠️ | ⚠️ | P2 |
| 双写缓冲 (Doublewrite) | ✅ | ❌ | ❌ | P2 |
| 表压缩 | ✅ | ❌ | ❌ | P2 |

#### TX-01: 事务增强
| 功能 | MySQL | v2.9.0 | v3.0.0 | 状态 |
|------|-------|--------|---------|------|
| SERIALIZABLE 隔离级别 | ✅ | ❌ | ⚠️ | P1 |
| Gap Locking | ✅ | ❌ | ❌ | P2 |
| XA 两阶段提交验证 | ✅ | ⚠️ | ⚠️ | P2 |

#### O-01: 运维功能增强
| 功能 | MySQL | v2.9.0 | v3.0.0 | 状态 |
|------|-------|--------|---------|------|
| performance_schema | ✅ | ❌ | ❌ | P2 |
| mysqladmin 等效工具 | ✅ | ❌ | ❌ | P2 |
| 在线 DDL (INPLACE) | ✅ | ❌ | ⚠️ | P1 |
| 在线添加索引 | ✅ | ❌ | ❌ | P2 |

---

## 四、稳定性测试缺口 (Beta Gate B-S1~B-S5)

### 4.1 当前测试文件状态

|| 测试文件 | 对应门禁 | 当前状态 | Issue | 问题 |
|---------|---------|---------|------|------|
|| `tests/concurrency_stress_test.rs` | B-S1 | ✅ 存在 | #394 | - |
|| `tests/crash_recovery_test.rs` | B-S2 | ✅ 存在 | #395 | 仅 in-memory，缺少真实 kill -9 |
|| `tests/long_run_stability_test.rs` | B-S3 | ✅ 存在 | #396 | 需验证 30min+ 稳定性 |
|| `tests/wal_integration_test.rs` | B-S4 | ✅ 存在 | #397 | 需验证崩溃后零数据丢失 |
|| `tests/network_tcp_smoke_test.rs` | B-S5 | ✅ 存在 | #398 | KILL 后连接泄漏问题 |
|| `tests/mvcc_transaction_test.rs` | T-02 / B-S2 | ✅ 存在 | #379 | 需补充 100 并发压测 |
|| `tests/connection_pool_stress_test.rs` | S-01 | ✅ 存在 | - | test_high_contention_stress overflow bug |
|| `tests/mvcc_snapshot_isolation_test.rs` | SSI | ✅ 存在 | PROOF-026 | - |
|| `tests/ssi_stress_test.rs` | SSI | ✅ 存在 | PROOF-026 | - |
|| `tests/query_cache_test.rs` | S-02 | ❌ 不存在 | - | 需新建 |
|| `tests/wal_crash_recovery_test.rs` | S-03 | ❌ 不存在 | - | 需新建 |

### 4.2 需修复的 Bug

#### Bug-01: connection_pool_stress_test overflow
```
test connection_pool_stress_test::test_high_contention_stress has a runtime error
thread 'main' panicked at 'attempt to multiply with overflow'
```
**位置**: `crates/server/tests/connection_pool_stress_test.rs`
**修复**: 使用 checked_mul 或 saturating_mul 替代乘法

---

## 五、门禁检查清单

### 5.1 Beta Gate (14 项)

| ID | 检查项 | 通过标准 | 当前状态 | 阻塞项 |
|----|--------|---------|---------|--------|
| B1 | Release Build | `cargo build --release --workspace` | ⏳ | |
| B2 | 全量测试 ≥90% | `cargo test --all-features` | ⏳ | |
| B3 | Clippy | `cargo clippy --all-features -- -D warnings` | ⏳ | |
| B4 | Format | `cargo fmt --all -- --check` | ⏳ | |
| B5 | 覆盖率 ≥75% | `cargo llvm-cov` | ⏳ | T-03, T-04 |
| B6 | 安全扫描 | `cargo audit` | ⏳ | |
| B7 | 文档链接 | `bash scripts/gate/check_docs_links.sh` | ⏳ | |
| B8 | TPC-H SF=0.1 22/22 | `check_tpch.sh sf=0.1` | ⏳ | |
| B9 | SQL Corpus ≥85% | `cargo test -p sqlrustgo-sql-corpus` | ✅ 100% | |
| B-S1 | concurrency_stress_test | 全部通过 | ⏳ | |
| B-S2 | crash_recovery_test | 8/8 PASS | ⏳ | T-02 |
| B-S3 | long_run_stability_test | 10/10 PASS | ⏳ | |
| B-S4 | wal_integration_test | 崩溃后零数据丢失 | ⏳ | |
| B-S5 | network_tcp_smoke_test | 无连接泄漏 | ⏳ | |

**Beta Gate 通过标准**: B1-B9 + B-S1~B-S5 全部 PASS，BLOCKERS = 0

### 5.2 RC Gate (12 项)

| ID | 检查项 | 通过标准 | 状态 |
|----|--------|---------|------|
| R1 | Release Build | `cargo build --release --workspace` | ⏳ |
| R2 | 全量测试 100% | `cargo test --all-features` | ⏳ |
| R3 | Clippy | `cargo clippy --all-features -- -D warnings` | ⏳ |
| R4 | Format | `cargo fmt --all -- --check` | ⏳ |
| R5 | 覆盖率 ≥85% | `cargo llvm-cov` | ⏳ |
| R6 | 安全扫描 | `cargo audit` | ⏳ |
| R7 | 文档完整性 | v3.0.0 docs exist | ⏳ |
| R8 | SQL Corpus ≥95% | `cargo test -p sqlrustgo-sql-corpus` | ⏳ |
| R9 | TPC-H SF=1 22/22 | `check_tpch.sh sf=1` | ⏳ |
| R10 | Performance Baseline | `check_regression.sh` | ⏳ |
| R11 | Sysbench Gate | `check_sysbench.sh` | ⏳ |
| R12 | Formal Proof | `check_proof.sh` | ⏳ |

**RC Gate 通过标准**: R1-R12 全部 PASS，BLOCKERS = 0

### 5.3 GA Gate (15 项)

| ID | 检查项 | 通过标准 | 状态 |
|----|--------|---------|------|
| GA-1 | Release Build | `cargo build --release --workspace` | ⏳ |
| GA-2 | 测试 100% | `cargo test --all-features` | ⏳ |
| GA-3 | Integration tests | `run_integration.sh --quick` | ⏳ |
| GA-4 | Clippy | `cargo clippy --all-features -- -D warnings` | ⏳ |
| GA-5 | Format | `cargo fmt --all -- --check` | ⏳ |
| GA-6 | 覆盖率 ≥85% | `cargo llvm-cov` | ⏳ |
| GA-7 | 安全扫描 | `cargo audit` | ⏳ |
| GA-8 | 文档链接 | `check_docs_links.sh` | ⏳ |
| GA-9 | TPC-H SF=1 22/22 | `check_tpch.sh sf=1` | ⏳ |
| GA-10 | 性能回归 (5%) | `check_regression.sh` | ⏳ |
| GA-11 | Formal proofs ≥10 | `docs/proof/*.json` | ⏳ |
| GA-12 | Sysbench Gate | `check_sysbench.sh` | ⏳ |
| GA-13 | 文档完整性 | 8 份文档存在 | ⏳ |
| GA-14 | SQL Corpus ≥95% | `cargo test -p sqlrustgo-sql-corpus` | ⏳ |
| GA-15 | 版本一致性 | cargo version + docs | ⏳ |

**GA Gate 通过标准**: GA-1~GA-15 全部 PASS，BLOCKERS = 0

---

## 六、MySQL 兼容性差距 (v3.0.0 vs MySQL 5.7)

### 6.1 功能差距

| 类别 | MySQL 5.7 | v3.0.0 | 完整度 | Gap |
|------|-----------|--------|--------|-----|
| 事件调度器 | CREATE EVENT | ❌ | 0% | P0 |
| 全文索引 | FULLTEXT | ❌ | 0% | P0 |
| GIS | GEOMETRY + ST_* | ❌ | 0% | P0 |
| 窗口函数 | 完整 | ✅ | 100% | - |
| CTE | WITH 递归 | ✅ | 100% | - |
| INSERT...SELECT | 完整 | ✅ | 100% | - |
| 存储过程游标 | DECLARE CURSOR | ⚠️ | 60% | P1 |
| 触发器 | BEFORE/AFTER | ⚠️ | 30% | P1 |
| SERIALIZABLE | 完整 | ⚠️ | 50% | P1 |
| Gap Locking | Next-Key Lock | ❌ | 0% | P2 |
| JSON 函数 | 完整 | ⚠️ | 50% | P2 |
| 行级安全 RLS | 完整 | ❌ | 0% | P2 |
| CREATE SEQUENCE | 完整 | ❌ | 0% | P2 |

### 6.2 存储引擎差距

| 功能 | MySQL (InnoDB) | v3.0.0 | 状态 |
|------|----------------|--------|------|
| 聚簇索引 | 主键即数据 | ❌ | P2 |
| 自适应哈希索引 | 热数据哈希加速 | ❌ | P2 |
| Change Buffer | 辅助索引缓存 | ⚠️ | P2 |
| 双写缓冲 | 防止部分写入 | ❌ | P2 |
| 表压缩 | PAGE_COMPRESSED | ❌ | P2 |

### 6.3 运维生态差距

| 功能 | MySQL | v3.0.0 | 状态 |
|------|-------|--------|------|
| INFORMATION_SCHEMA | 完整 | ⚠️ | P1 |
| performance_schema | 完整 | ❌ | P2 |
| 慢查询日志 | 完整 | ✅ | - |
| mysqladmin | 完整 | ❌ | P2 |
| mysqlbinlog | 完整 | ❌ | P2 |
| 在线 DDL | INPLACE | ⚠️ | P1 |

### 6.4 安全差距

| 功能 | MySQL 5.7 | v3.0.0 | 状态 |
|------|-----------|--------|------|
| TLS 加密连接 | ✅ | ✅ | - |
| SSL/TLS | ✅ | ✅ | - |
| AES-256 存储加密 | ✅ | ❌ | P2 |
| 行级安全 RLS | ✅ | ❌ | P2 |
| 列级权限 | ✅ | ⚠️ | P2 |
| 密码轮转 | ✅ | ❌ | P2 |

---

## 七、行动清单

### 7.1 Beta Gate 阻塞项 (按优先级)

|| 优先级 | 任务 | Issue | 负责人 | 截止 | 测试命令 |
|--------|------|-------|--------|------|------|
| P0 | 事务状态机压力测试 | #379 | claude | Beta 前 | `cargo test --test mvcc_transaction_test` |
| P0 | TPC-H SF=1 CI Gate | #382 | - | Beta 前 | `bash scripts/gate/check_tpch.sh sf=1` |
| P1 | Optimizer 测试扩展 | #380 | claude | Beta 前 | `cargo test -p sqlrustgo-optimizer` |
| P1 | Planner 测试扩展 | #381 | claude | Beta 前 | `cargo test -p sqlrustgo-planner` |
| P1 | 连接池并发压力测试 | #394 | - | Beta 前 | `cargo test --test connection_pool_stress_test` |
| P1 | 查询缓存 DML 失效测试 | S-02 | - | Beta 前 | `cargo test --test query_cache_test` (需新建) |
| P1 | Group Commit WAL 崩溃恢复测试 | S-03 | - | Beta 前 | `cargo test --test wal_crash_recovery_test` (需新建) |
| P1 | Bug: connection_pool_stress_test overflow | - | - | Beta 前 | - |
| P1 | concurrency_stress_test 验证 | #394 | - | Beta 前 | `cargo test --test concurrency_stress_test` |
| P1 | crash_recovery_test 验证 | #395 | - | Beta 前 | `cargo test --test crash_recovery_test` |
| P1 | long_run_stability_test 验证 | #396 | - | Beta 前 | `cargo test --test long_run_stability_test` |
| P1 | wal_integration_test 验证 | #397 | - | Beta 前 | `cargo test --test wal_integration_test` |
| P1 | network_tcp_smoke_test 验证 | #398 | - | Beta 前 | `cargo test --test network_tcp_smoke_test` |

### 7.2 RC Gate 阻塞项

| 优先级 | 任务 | 现状 | 说明 |
|--------|------|------|------|
| P0 | CBO 代价模型集成 | 🔄 | SimpleCostModel 接入 planner |
| P0 | TPC-H SF=1 22/22 p99<5s | 🔄 | 当前 ~10.9s |
| P1 | 覆盖率 ≥85% | ⏳ | 当前 84.18% |
| P1 | Performance Baseline 建立 | ⏳ | check_regression.sh stub |

### 7.3 GA Gate 阻塞项

| 优先级 | 任务 | 现状 | 说明 |
|--------|------|------|------|
| P0 | 所有 RC Gate 项 | ⏳ | RC 通过是 GA 前提 |
| P1 | 聚簇索引实现 | ❌ | 性能关键 |
| P1 | 在线 DDL INPLACE | ⚠️ | 当前阻塞式 |
| P2 | performance_schema | ❌ | 监控完善 |
| P2 | mysqladmin 等效工具 | ❌ | 运维工具 |

---

## 八、相关文档

| 文档 | 说明 |
|------|------|
| Issue #353 | v3.0.0 开发总控 |
| `ALPHA_GAPS.md` | Alpha 阶段整改清单 |
| `BETA_GATE_MASTER_CONTROL.md` | Beta 门禁任务总控 |
| `../v2.9.0/WEAKNESS_ANALYSIS.md` | MySQL 兼容性差距分析 |
| `scripts/gate/check_beta_v300.sh` | Beta Gate 检查脚本 |
| `scripts/gate/check_rc_v300.sh` | RC Gate 检查脚本 |
| `scripts/gate/check_ga_v300.sh` | GA Gate 检查脚本 |

---

*本文档由 claude 生成，用于追踪 v3.0.0 全部规划但未完成的功能和测试。*
*每次 Beta/RC/GA gate 通过后更新状态。*
