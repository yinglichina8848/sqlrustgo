# SQLRustGo 历史遗留问题完整追踪报告

> **版本**: 1.0
> **日期**: 2026-05-07
> **状态**: 🟡 进行中
> **追踪范围**: v2.0.0 → v3.0.0 所有功能、集成和测试缺口
> **目标**: 闭环追踪所有历史遗留问题，不允许跳过

---

## 一、执行摘要

### 1.1 问题概述

| 维度 | 数量 | 说明 |
|------|------|------|
| **总功能缺口** | 47 | 从 v2.0.0 以来定义但未完全实现的功能 |
| **集成缺口** | 12 | 功能开发完成但未正确集成 |
| **测试缺口** | 18 | 测试未实现、被忽略或无效 |
| **P0 阻塞项** | 8 | 阻塞 Beta/RC/GA Gate |
| **历史累计问题** | 77 | 全部缺口总和 |

### 1.2 版本演进中的核心问题

```
v2.0.0 (2025-xx-xx) → v2.9.0 (2026-05-05) → v3.0.0 (TBD)
     │                      │                      │
   向量化执行            形式化验证卓越          完整分布式
     │                      │                      │
   定义 15 个功能         定义 25 个功能          定义 37 个功能
   实现 10 个             实现 18 个              实现 ?? 个
   遗留 5 个              遗留 7 个               遗留待统计
```

---

## 二、功能缺口总表（v2.0.0 → v3.0.0）

### 2.1 P0 功能缺口（阻塞版本发布）

| # | 功能 | 版本引入 | 当前状态 | 差距分析 | 优先级 | 负责人 | 截止 |
|---|------|----------|----------|----------|--------|--------|------|
| F-01 | 事件调度器 (CREATE EVENT) | v2.0.0 | ❌ 未实现 | 完全缺失，无 parser/executor 支持 | P0 | 待分配 | GA |
| F-02 | 全文索引 (FULLTEXT) | v2.0.0 | ❌ 未实现 | storage 无 FULLTEXT 实现 | P0 | 待分配 | GA |
| F-03 | 空间数据类型 (GIS) | v2.0.0 | ❌ 未实现 | GEOMETRY/POINT/LINESTRING + ST_* 完全缺失 | P0 | 待分配 | GA |
| F-04 | INSERT...SELECT | v2.6.0 | ✅ 已实现 | v3.0.0 Alpha 实现 | - | - | - |
| F-05 | 窗口函数 (NTILE/LEAD/LAG/FIRST_VALUE/LAST_VALUE/NTH_VALUE) | v2.8.0 | ✅ 已实现 | v3.0.0 Alpha 实现 | - | - | - |
| F-06 | CTE 执行 (WITH 递归) | v2.6.0 | ✅ 已实现 | v3.0.0 Alpha 实现 | - | - | - |
| F-07 | 查询缓存 DML 失效 | v2.7.0 | ⚠️ 有缺陷 | LRU 缓存存在但 DML 失效机制未测试 | P0 | 待分配 | RC |
| F-08 | 连接池并发压力 | v2.7.0 | ⚠️ 未测试 | 连接池存在但无压力测试 | P0 | 待分配 | Beta |
| F-09 | Group Commit WAL 崩溃恢复 | v2.7.0 | ⚠️ 未测试 | Group Commit 存在但无崩溃恢复测试 | P0 | 待分配 | Beta |
| F-10 | TPC-H SF=1 无 OOM | v2.8.0 | ⚠️ OOM | 曾 OOM，内存治理未完成 | P0 | 待分配 | RC |

### 2.2 P1 功能缺口（影响兼容性）

| # | 功能 | 版本引入 | MySQL | v2.9.0 | v3.0.0 | 完整度 | 行动 |
|---|------|----------|-------|---------|--------|--------|------|
| F-11 | 窗口函数完整 | v2.0.0 | 完整 | ⚠️ 部分 | ✅ 完整 | 100% | ✅ 已解决 |
| F-12 | 存储过程游标 | v2.0.0 | 完整 | ⚠️ 部分 | ⚠️ 部分 | 60% | 需继续完善 |
| F-13 | 触发器 | v2.0.0 | 完整 | ❌ 假实现 | ⚠️ 部分 | 30% | 需重写或移除假实现 |
| F-14 | CTE 递归 | v2.0.0 | 完整 | ❌ 无 | ✅ 已实现 | 100% | ✅ 已解决 |
| F-15 | SERIALIZABLE 隔离级别 | v2.0.0 | 完整 | ❌ 无 | ⚠️ 部分 | 50% | 需完善 |
| F-16 | Gap Locking | v2.0.0 | 完整 | ❌ 无 | ❌ 无 | 0% | GA 前 |
| F-17 | JSON 函数完整 | v2.5.0 | 完整 | ⚠️ 部分 | ⚠️ 部分 | 50% | GA 前 |
| F-18 | INFORMATION_SCHEMA | v2.0.0 | 完整 | ❌ 无 | ✅ 部分 | 70% | 需完善 |
| F-19 | SSL/TLS 加密 | v2.0.0 | 完整 | ❌ 无 | ✅ 已实现 | 100% | ✅ 已解决 |
| F-20 | 慢查询日志 | v2.0.0 | 完整 | ❌ 无 | ✅ 已实现 | 100% | ✅ 已解决 |
| F-21 | 在线 DDL | v2.0.0 | 完整 | ❌ 无 | ⚠️ 阻塞式 | 50% | RC 前 |
| F-22 | Prepared Statement 参数绑定 | v2.6.0 | 完整 | ⚠️ 有缺陷 | ✅ 已修复 | 100% | ✅ 已解决 |

### 2.3 P2 功能缺口（建议 GA 前完成）

| # | 功能 | 版本引入 | 当前状态 | 完成计划 |
|---|------|----------|----------|----------|
| F-23 | 聚簇索引 | v2.5.0 | ❌ 未实现 | v3.1.0 |
| F-24 | 自适应哈希索引 (AHI) | v2.5.0 | ❌ 未实现 | v3.1.0 |
| F-25 | Change Buffer | v2.5.0 | ⚠️ 部分 | v3.1.0 |
| F-26 | 双写缓冲 | v2.5.0 | ❌ 未实现 | v3.1.0 |
| F-27 | 表压缩 | v2.5.0 | ❌ 未实现 | v3.1.0 |
| F-28 | XA 两阶段提交验证 | v2.6.0 | ⚠️ 有但不完整 | v3.1.0 |
| F-29 | 行级安全 (RLS) | v2.0.0 | ❌ 未实现 | v3.2.0 |
| F-30 | CREATE SEQUENCE | v2.0.0 | ❌ 未实现 | v3.2.0 |
| F-31 | performance_schema | v2.0.0 | ❌ 未实现 | v3.2.0 |
| F-32 | mysqladmin 等效 | v2.0.0 | ❌ 未实现 | v3.2.0 |
| F-33 | mysqlbinlog | v2.0.0 | ❌ 未实现 | v3.2.0 |
| F-34 | AES-256 存储加密 | v2.8.0 | ❌ 未实现 | v3.2.0 |
| F-35 | 密码轮转 | v2.0.0 | ❌ 未实现 | v3.2.0 |
| F-36 | 列级权限 | v2.0.0 | ⚠️ 部分 | v3.2.0 |

---

## 三、功能集成缺口

> 功能开发完成但未正确集成为完整实现

| # | 功能 | 模块 | 现状 | 集成问题 | 修复方案 | 优先级 |
|---|------|------|------|----------|----------|--------|
| I-01 | 触发器 | storage | ⚠️ 有代码 | storage 层返回 "not supported"，是假实现 | 要么实现要么删除，不能假实现 | P0 |
| I-02 | 查询缓存 | executor | ⚠️ 有代码 | LRU 缓存存在但无 DML 失效测试 | 补充 DML 失效测试 | P0 |
| I-03 | MVCC SSI | transaction | ⚠️ 有代码 | TLA+ 注释提及 "Classic 3-cycle"，可能脆弱 | 补充 100 并发压力测试 | P1 |
| I-04 | CTE | planner | ⚠️ 部分 | Parser 有 WithClause，planner 无执行 | 已实现 | ✅ |
| I-05 | EXPLAIN | executor | ⚠️ 部分 | 存在但不完整，无 CBO 输出 | 完善 EXPLAIN ANALYZE | P1 |
| I-06 | mysqldump | tools | ⚠️ 有代码 | 存在但功能不明 | 验证功能或移除 | P1 |
| I-07 | 窗口函数 | executor | ✅ 已实现 | v3.0.0 Alpha 实现 | - | - |
| I-08 | INSERT...SELECT | executor | ✅ 已实现 | v3.0.0 Alpha 实现 | - | - |
| I-09 | 连接池 | network | ⚠️ 有代码 | 存在但无压力测试验证 | 补充并发压力测试 | P0 |
| I-10 | Group Commit | wal | ⚠️ 有代码 | 存在但无崩溃恢复验证 | 补充 kill -9 测试 | P0 |
| I-11 | CBO 代价模型 | optimizer | ⚠️ 部分 | SimpleCostModel 存在但未接入 planner | 集成到 planner | P0 |
| I-12 | 并行执行 | executor | ⚠️ 部分 | parallel_executor.rs 存在但仅简单并行 | 完善分区并行 | P1 |

---

## 四、测试缺口

### 4.1 测试文件缺口（文件不存在）

| # | 测试文件 | 对应功能 | 定义版本 | 当前状态 | 创建任务 |
|---|---------|----------|----------|----------|----------|
| T-01 | `query_cache_test.rs` | 查询缓存 DML 失效 | v2.7.0 | ❌ 不存在 | Issue #401 |
| T-02 | `wal_crash_recovery_test.rs` | Group Commit WAL 崩溃恢复 | v2.7.0 | ❌ 不存在 | Issue #402 |
| T-03 | `mvcc_transaction_test.rs` | 事务状态机压力 | v2.6.0 | ⚠️ 存在但测试不足 | Issue #379 |

### 4.2 被忽略的测试（#[ignore]）

| # | 测试文件 | 测试数 | 忽略数 | 忽略原因 | 修复方案 | 优先级 |
|---|---------|--------|--------|----------|----------|--------|
| T-04 | `long_run_stability_test.rs` | 10 | 10 | 未说明 | 实现真实 30min+ 稳定性测试 | P0 |
| T-05 | `crash_recovery_test.rs` | 8 | 0 | - | 文档说 9 实际 8 | 已修正 |

### 4.3 测试覆盖率不足

| # | 模块 | 当前覆盖率 | 目标 | 缺口 | 修复任务 |
|---|------|-----------|------|------|----------|
| T-06 | optimizer | ~55% | 70% | -15% | Issue #380 |
| T-07 | planner | ~65% | 80% | -15% | Issue #381 |
| T-08 | network | ~60% | 75% | -15% | 补充网络测试 |
| T-09 | executor | ~72% | 80% | -8% | 补充执行器测试 |
| T-10 | storage | ~78% | 85% | -7% | 补充存储测试 |

### 4.4 性能回归测试无效

| # | 脚本 | 问题 | 修复方案 | 优先级 |
|---|------|------|----------|--------|
| T-11 | `check_perf_baseline.sh` | 仅检查文件存在然后 SKIP | 实现实际性能比较 | P0 |
| T-12 | `check_regression.sh` | 存在但未集成 CI | 集成到 RC Gate | P1 |
| T-13 | TPC-H in CI | 不在 CI gate | 已在 B8 | ✅ |
| T-14 | Sysbench in CI | 不在 CI gate | 已在 check_sysbench.sh | ✅ |

### 4.5 混沌工程测试缺失

| # | 测试场景 | Gitea CI | GitHub Actions | 修复方案 |
|---|---------|----------|---------------|----------|
| T-15 | Deadlock injection | ❌ | ✅ | 迁移到 Gitea CI |
| T-16 | CPU 80% stress | ❌ | ✅ | 迁移到 Gitea CI |
| T-17 | Network 30% packet loss | ❌ | ✅ | 迁移到 Gitea CI |
| T-18 | Memory fault injection | ❌ | ❌ | 待实现 |
| T-19 | Disk I/O delay simulation | ❌ | ❌ | 待实现 |
| T-20 | Process kill -9 mid-transaction | ❌ | ❌ | 待实现 |

---

## 五、门禁检查缺口

### 5.1 Beta Gate (19 项)

| ID | 检查项 | 脚本实现 | 测试文件 | 状态 | 问题 |
|----|--------|---------|---------|------|------|
| B1 | Release Build | ✅ | - | ⏳ | |
| B2 | 全量测试 ≥90% | ✅ | - | ⏳ | |
| B3 | Clippy | ✅ | - | ⏳ | |
| B4 | Format | ✅ | - | ⏳ | |
| B5 | 覆盖率 ≥75% | ✅ | - | ⏳ | T-06, T-07 |
| B6 | 安全扫描 | ✅ | - | ⏳ | |
| B7 | 文档链接 | ✅ | - | ⏳ | |
| B8 | TPC-H SF=0.1 22/22 | ✅ | - | ⏳ | |
| B9 | SQL Corpus ≥85% | ✅ | - | ✅ 100% | |
| B10 | CBO Index Selection | ✅ | cbo_integration_test | ✅ | |
| B11 | CBO Join Cost | ✅ | cbo_integration_test | ✅ | |
| B12 | CBO Optimizer Tests | ✅ | - | ⏳ | T-06 |
| B13 | CBO Planner Tests | ✅ | - | ⏳ | T-07 |
| B-S1 | concurrency_stress_test | ✅ | concurrency_stress_test.rs | ✅ 9/9 | |
| B-S2 | crash_recovery_test | ✅ | crash_recovery_test.rs | ✅ 8/8 | |
| B-S3 | long_run_stability_test | ✅ | long_run_stability_test.rs | 🔴 10/10 #[ignore] | T-04 |
| B-S4 | wal_integration_test | ✅ | wal_integration_test.rs | ✅ 16/16 | |
| B-S5 | network_tcp_smoke_test | ✅ | network_tcp_smoke_test.rs | ✅ 6/6 | |

**Beta Gate 阻塞项**: B-S3 (long_run_stability_test 全部 #[ignore])

### 5.2 RC Gate (12 项)

| ID | 检查项 | 脚本 | 状态 | 阻塞项 |
|----|--------|------|------|--------|
| R1 | Release Build | check_rc_v300.sh | ✅ | |
| R2 | 全量测试 100% | - | ⏳ | |
| R3 | Clippy | - | ⏳ | |
| R4 | Format | - | ⏳ | |
| R5 | 覆盖率 ≥85% | - | ⏳ | T-06~T-10 |
| R6 | 安全扫描 | - | ⏳ | |
| R7 | 文档完整性 | - | ⏳ | |
| R8 | SQL Corpus ≥95% | - | ⏳ | |
| R9 | TPC-H SF=1 22/22 | - | ⏳ | F-10 |
| R10 | Performance Baseline | check_regression.sh | 🔴 stub | T-11 |
| R11 | Sysbench Gate | check_sysbench.sh | ⏳ | |
| R12 | Formal Proof | check_proof.sh | ⏳ | |

**RC Gate 阻塞项**: R10 (Performance Baseline stub)

### 5.3 GA Gate (15 项)

| ID | 检查项 | 脚本 | 状态 | 阻塞项 |
|----|--------|------|------|--------|
| GA-1 | Release Build | check_ga_v300.sh | ✅ | |
| GA-2 | 测试 100% | - | ⏳ | |
| GA-3 | Integration tests | - | ⏳ | |
| GA-4 | Clippy | - | ⏳ | |
| GA-5 | Format | - | ⏳ | |
| GA-6 | 覆盖率 ≥85% | - | ⏳ | |
| GA-7 | 安全扫描 | - | ⏳ | |
| GA-8 | 文档链接 | - | ⏳ | |
| GA-9 | TPC-H SF=1 22/22 | - | ⏳ | |
| GA-10 | 性能回归 (5%) | - | 🔴 stub | T-11 |
| GA-11 | Formal proofs ≥10 | - | ⏳ | |
| GA-12 | Sysbench Gate | - | ⏳ | |
| GA-13 | 文档完整性 | - | ⏳ | |
| GA-14 | SQL Corpus ≥95% | - | ⏳ | |
| GA-15 | 版本一致性 | - | ⏳ | |

---

## 六、问题闭环追踪

### 6.1 Issue 映射表

| Issue | 标题 | 类型 | 优先级 | 状态 | 对应缺口 |
|-------|------|------|--------|------|----------|
| #353 | v3.0.0 开发总控 | 追踪 | P0 | 🟡 进行中 | 全部 |
| #376 | Sysbench OLTP 适配 | 开发 | P0 | 🟡 进行中 | F-08 |
| #377 | COM_MULTI 多语句执行 | 开发 | P0 | ✅ 完成 | I-09 |
| #378 | Prepared Statement 参数绑定修复 | 开发 | P0 | ✅ 完成 | F-22 |
| #379 | 事务状态机压力测试 | 测试 | P0 | 🟡 进行中 | T-03 |
| #380 | Optimizer 测试扩展 | 测试 | P1 | 🟡 进行中 | T-06 |
| #381 | Planner 测试扩展 | 测试 | P1 | 🟡 进行中 | T-07 |
| #382 | TPC-H SF=1 CI Gate | 开发 | P1 | 🟡 进行中 | F-10 |
| #394 | concurrency_stress_test | 测试 | P0 | ✅ 完成 | T-11 |
| #395 | crash_recovery_test | 测试 | P0 | ✅ 完成 | T-05 |
| #396 | long_run_stability_test | 测试 | P0 | 🔴 阻塞 | T-04 |
| #397 | wal_integration_test | 测试 | P0 | ✅ 完成 | T-12 |
| #398 | network_tcp_smoke_test | 测试 | P0 | ✅ 完成 | T-13 |
| **#401** | **query_cache_test 创建** | **测试** | **P0** | **🔴 待创建** | **T-01** |
| **#402** | **wal_crash_recovery_test 创建** | **测试** | **P0** | **🔴 待创建** | **T-02** |
| **#403** | **check_perf_baseline.sh 实现** | **开发** | **P0** | **🔴 待实现** | **T-11** |
| **#404** | **long_run_stability_test #[ignore] 修复** | **测试** | **P0** | **🔴 待修复** | **T-04** |

### 6.2 行动项总表

#### 🔴 P0 - 立即行动（本周）

| # | 行动项 | 负责 | 验证方式 | 截止 |
|---|--------|------|----------|------|
| A-01 | 创建 query_cache_test.rs | 待分配 | cargo test --test query_cache_test | 2026-05-14 |
| A-02 | 创建 wal_crash_recovery_test.rs | 待分配 | cargo test --test wal_crash_recovery_test | 2026-05-14 |
| A-03 | 修复 long_run_stability_test 全部 #[ignore] | 待分配 | cargo test --test long_run_stability_test -- --ignored | 2026-05-14 |
| A-04 | 实现 check_perf_baseline.sh 实际比较 | 待分配 | check_perf_baseline.sh 输出真实比较结果 | 2026-05-14 |

#### 🟠 P1 - Beta Gate 前完成

| # | 行动项 | 负责 | 验证方式 | 截止 |
|---|--------|------|----------|------|
| A-05 | TPC-H SF=1 无 OOM | 待分配 | check_tpch.sh sf=1 22/22 | 2026-05-21 |
| A-06 | optimizer 覆盖率 ≥70% | claude | cargo llvm-cov | 2026-05-21 |
| A-07 | planner 覆盖率 ≥80% | claude | cargo llvm-cov | 2026-05-21 |
| A-08 | 连接池并发压力测试 | 待分配 | cargo test --test connection_pool_stress_test | 2026-05-21 |

#### 🟡 P2 - RC Gate 前完成

| # | 行动项 | 负责 | 验证方式 | 截止 |
|---|--------|------|----------|------|
| A-09 | CBO 代价模型集成到 planner | 待分配 | EXPLAIN 选择索引扫描 | 2026-06-01 |
| A-10 | Group Commit WAL 崩溃恢复验证 | 待分配 | kill -9 后数据完整 | 2026-06-01 |
| A-11 | 覆盖率整体 ≥85% | 待分配 | cargo llvm-cov | 2026-06-01 |

---

## 七、跨版本任务（不阻塞当前版本）

以下任务在后续版本完成，但必须有计划和状态追踪：

### v3.1.0

| # | 功能 | 状态 | 计划 |
|---|------|------|------|
| V-01 | 聚簇索引 | 未开始 | v3.1.0 开发 |
| V-02 | 自适应哈希索引 | 未开始 | v3.1.0 开发 |
| V-03 | Change Buffer 完善 | 未开始 | v3.1.0 开发 |
| V-04 | 双写缓冲 | 未开始 | v3.1.0 开发 |
| V-05 | Gap Locking | 未开始 | v3.1.0 开发 |

### v3.2.0

| # | 功能 | 状态 | 计划 |
|---|------|------|------|
| V-06 | performance_schema | 未开始 | v3.2.0 开发 |
| V-07 | mysqladmin 等效 | 未开始 | v3.2.0 开发 |
| V-08 | 事件调度器 | 未开始 | v3.2.0 开发 |
| V-09 | 全文索引 | 未开始 | v3.2.0 开发 |
| V-10 | GIS 支持 | 未开始 | v3.2.0 开发 |

---

## 八、报告更新机制

### 8.1 更新频率

| 阶段 | 更新频率 | 负责人 |
|------|----------|--------|
| Beta Gate 前 | 每日更新 | 版本负责人 |
| RC Gate 前 | 每周更新 | 版本负责人 |
| GA Gate 前 | 每日更新 | 版本负责人 |
| 发布后 | 每版本更新 | 版本负责人 |

### 8.2 状态定义

| 状态 | 定义 | 要求 |
|------|------|------|
| 🔴 未开始 | 完全没有实现 | 必须有完成计划 |
| 🔴 阻塞 | 有障碍阻止前进 | 必须有解决方案 |
| 🟠 进行中 | 正在开发 | 必须有进度报告 |
| 🟡 待验证 | 实现完成待测试 | 必须有测试验证 |
| ✅ 已完成 | 通过验证 | 必须有验证证据 |
| ⚠️ 故意跳过 | 有充分理由暂时不做 | 必须有文档说明原因和后续计划，且不能用于 Gate 豁免 |

### 8.3 禁止事项

- ❌ 禁止将问题标记为"已忽略"而不提供原因和后续计划
- ❌ 禁止在 Gate 检查中跳过任何已定义的检查项
- ❌ 禁止创建测试文件但不实现测试（假实现）
- ❌ 禁止功能开发完成但不补充对应测试

---

## 九、相关文档索引

| 文档 | 位置 | 用途 |
|------|------|------|
| 本报告 | docs/releases/v3.0.0/COMPLETE_LEGACY_TRACKING_REPORT.md | 完整历史追踪 |
| WEAKNESS_ANALYSIS (v2.9.0) | docs/releases/v2.9.0/WEAKNESS_ANALYSIS.md | v2.9.0 弱项分析 |
| ALPHA_GAPS | docs/releases/v3.0.0/ALPHA_GAPS.md | Alpha 阶段整改清单 |
| BETA_GATE_MASTER_CONTROL | docs/releases/v3.0.0/BETA_GATE_MASTER_CONTROL.md | Beta 门禁总控 |
| HISTORICAL_LEGACY | docs/releases/v3.0.0/HISTORICAL_LEGACY_DEVELOPMENT_PLAN.md | 历史开发计划 |
| Beta Gate 脚本 | scripts/gate/check_beta_v300.sh | Beta 门禁检查 |
| RC Gate 脚本 | scripts/gate/check_rc_v300.sh | RC 门禁检查 |
| GA Gate 脚本 | scripts/gate/check_ga_v300.sh | GA 门禁检查 |

---

## 十、闭环确认

本文档追踪的所有问题必须满足以下条件才能关闭：

1. **功能缺口**: 有对应的 Issue，且 Issue 已关闭（状态: 已完成）
2. **集成缺口**: 功能已集成并通过测试验证
3. **测试缺口**: 测试文件存在，测试通过，无 #[ignore]
4. **门禁缺口**: 对应的 Gate 检查项 PASS

---

*本文档由 Claude 生成并维护。*
*每次门禁检查后更新状态。*
*禁止将任何问题标记为"已忽略"而不提供原因和后续计划。*

---

## 十一、Issue 创建状态

### 11.1 Gitea Issue 创建结果

✅ **已成功创建所有 Issue**

| Issue # | 标题 | 优先级 | 门禁 | URL |
|---------|------|--------|------|------|
| #416 | query_cache_test.rs 创建 | P0 | S-02 | [#416](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/416) |
| #417 | wal_crash_recovery_test.rs 创建 | P0 | S-03 | [#417](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/417) |
| #418 | check_perf_baseline.sh 实现 | P0 | R10, GA-10 | [#418](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/418) |
| #419 | long_run_stability_test #[ignore] 修复 | P0 | B-S3 | [#419](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/419) |
| #420 | [总追踪] v2.0.0 → v3.0.0 历史遗留问题闭环追踪 | P0 | 全部 | [#420](http://192.168.0.252.3000/openclaw/sqlrustgo/issues/420) |

### 11.2 Issue #420（总追踪 Issue）

总追踪 Issue [#420](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/420) 已创建，关联所有子 Issue。
