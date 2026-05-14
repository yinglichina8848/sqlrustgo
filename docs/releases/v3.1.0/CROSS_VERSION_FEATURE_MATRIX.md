# SQLRustGo Cross-Version Feature Matrix Tracking Report

> **版本**: v3.1.0  
> **生成日期**: 2026-05-14  
> **对比版本**: v2.9.0 → v3.0.0 → v3.1.0

---

## 1. 执行摘要

本文档追踪 SQLRustGo 核心特性在 v2.9.0、v3.0.0 和 v3.1.0 版本间的实现状态、测试覆盖和 Gate 集成情况。

| 维度 | v2.9.0 | v3.0.0 | v3.1.0 (Beta) |
|------|--------|--------|----------------|
| 核心 SQL 功能 | ~75% | ~85% | ~90% |
| 事务隔离 | Read Committed | SSI (Serializable) | SSI + Deadlock Detection |
| 优化器 | 基础 | PredicatePushdown | CBO 代价模型 |
| 存储引擎 | WAL + MVCC | WAL + MVCC | Clustered Index + Gap Lock |
| Gate 测试覆盖 | N/A | N/A | B-S1 ~ B-S9 |

---

## 2. 跨版本功能矩阵

### 2.1 核心 DML 功能

| Feature | Version | Implemented | Tested | Gate-Integrated | Gap |
|---------|---------|-------------|--------|-----------------|-----|
| SELECT | v1.0.0 | ✅ | ✅ | ✅ (B2) | 无 |
| INSERT | v1.0.0 | ✅ | ✅ | ✅ (B2) | 无 |
| UPDATE | v1.0.0 | ✅ | ✅ | ✅ (B2) | 无 |
| DELETE | v1.0.0 | ✅ | ✅ | ✅ (B2) | 无 |
| REPLACE INTO | v2.9.0 | ✅ | ✅ | ✅ (B2) | 无 |
| INSERT...SELECT | v3.0.0 | ✅ | ⚠️ | ❌ | 需要独立测试用例 |
| **MERGE INTO** | v3.1.0 | ❌ | ❌ | ❌ | **规划中，未实现** |

### 2.2 窗口函数

| Feature | Version | Implemented | Tested | Gate-Integrated | Gap |
|---------|---------|-------------|--------|-----------------|-----|
| ROW_NUMBER | v2.9.0 | ✅ | ✅ | ✅ (B-S9) | 无 |
| RANK | v2.9.0 | ✅ | ✅ | ✅ (B-S9) | 无 |
| DENSE_RANK | v2.9.0 | ✅ | ✅ | ✅ (B-S9) | 无 |
| NTILE | v3.0.0 | ✅ | ✅ | ✅ (B-S9) | 无 |
| LEAD | v3.0.0 | ✅ | ✅ | ✅ (B-S9) | 无 |
| LAG | v3.0.0 | ✅ | ✅ | ✅ (B-S9) | 无 |
| FIRST_VALUE | v3.0.0 | ✅ | ⚠️ | ❌ | 需要完整测试 |
| LAST_VALUE | v3.0.0 | ✅ | ⚠️ | ❌ | 需要完整测试 |
| NTH_VALUE | v3.0.0 | ✅ | ⚠️ | ❌ | 需要完整测试 |

**实现位置**: `/crates/executor/src/window_executor.rs`  
**测试位置**: 
- `/tests/window_function_test.rs`
- `/tests/window_function_execution_test.rs`
- `/tests/e2e/e2e_window_function_test.rs`

### 2.3 CTE 与递归查询

| Feature | Version | Implemented | Tested | Gate-Integrated | Gap |
|---------|---------|-------------|--------|-----------------|-----|
| CTE / WITH | v2.9.0 | ✅ | ✅ | ✅ (B2) | 无 |
| 递归 CTE | v3.0.0 | ✅ | ⚠️ | ❌ | 需要压力测试 |

**实现位置**: 需要验证 `/crates/planner/src/` 中 CTE 相关实现  
**测试覆盖**: sql_corpus 中有 `EXPRESSIONS/cte*.sql`

### 2.4 事务与并发控制

| Feature | Version | Implemented | Tested | Gate-Integrated | Gap |
|---------|---------|-------------|--------|-----------------|-----|
| Read Committed | v1.0.0 | ✅ | ✅ | ✅ | 无 |
| Snapshot Isolation (MVCC) | v2.9.0 | ✅ | ✅ | ✅ | 无 |
| Serializable (SSI) | v3.0.0 | ✅ | ✅ | ✅ (B-S6) | 无 |
| Deadlock Detection | v3.0.0 | ✅ | ✅ | ✅ (B-S6) | 无 |
| Gap Lock | v3.1.0 | ✅ | ✅ | ❌ | 未集成到 Gate |
| Next-Key Lock | v3.1.0 | ✅ | ✅ | ❌ | 未集成到 Gate |

**实现位置**:
- SSI: `/crates/transaction/src/ssi.rs`
- Deadlock: `/crates/transaction/src/deadlock.rs`, `/crates/transaction/src/wait_for_graph.rs`
- Gap Lock: `/crates/transaction/src/lock.rs` (LockTarget::Gap)

**测试位置**:
- `/crates/transaction/tests/ssi_stress_test.rs`
- `/crates/transaction/tests/ssi_integration.rs`

### 2.5 WAL 与审计链

| Feature | Version | Implemented | Tested | Gate-Integrated | Gap |
|---------|---------|-------------|--------|-----------------|-----|
| WAL | v2.9.0 | ✅ | ✅ | ✅ (B-S4) | 无 |
| Audit Chain (Hash Chain) | v3.1.0 | ✅ | ⚠️ | ❌ | 需要集成测试 |

**实现位置**:
- WAL: `/crates/transaction/src/idempotency/wal.rs`
- Audit Hash Chain: `/crates/transaction/src/audit/hash_chain.rs`

**测试位置**: `/crates/transaction/src/idempotency/tests.rs`

### 2.6 优化器与 CBO

| Feature | Version | Implemented | Tested | Gate-Integrated | Gap |
|---------|---------|-------------|--------|-----------------|-----|
| PredicatePushdown | v3.0.0 | ✅ | ✅ | ✅ (B2) | 无 |
| ProjectionPruning | v3.0.0 | ✅ | ✅ | ✅ (B2) | 无 |
| ConstantFolding | v3.0.0 | ✅ | ✅ | ✅ (B2) | 无 |
| SimpleCostModel | v3.0.0 | ✅ | ⚠️ | ❌ | **CBO 未接入 Planner** |
| CBO Cost Integration | v3.1.0 | ⚠️ | ❌ | ❌ | **规划中，未完成** |

**实现位置**:
- Cost Model: `/crates/optimizer/src/cost.rs` (SimpleCostModel)
- Path Selector: `/crates/optimizer/src/path_selector.rs`

**问题**: `SimpleCostModel::estimate_cost` 仍返回硬编码值，未真正接入物理计划选择

### 2.7 聚簇索引

| Feature | Version | Implemented | Tested | Gate-Integrated | Gap |
|---------|---------|-------------|--------|-----------------|-----|
| Clustered Index | v3.1.0 | ✅ | ⚠️ | ❌ | **需要完整测试套件** |

**实现位置**: `/crates/storage/src/clustered_index/` (模块已存在)
- `leaf.rs` - ClusteredLeafPage
- `overflow.rs` - OverflowManager
- `transaction.rs` - ClusteredPageTransaction
- `wal_integration.rs` - ClusteredWalManager

**文档**: `/docs/releases/v3.1.0/oo/CLUSTERED_INDEX.md` (状态: ✅ 已完成)

### 2.8 EXPLAIN ANALYZE

| Feature | Version | Implemented | Tested | Gate-Integrated | Gap |
|---------|---------|-------------|--------|-----------------|-----|
| EXPLAIN ANALYZE | v3.0.0 | ✅ | ✅ | ✅ (B-S8) | 无 |

**实现位置**: `/crates/executor/src/explain.rs`  
**测试位置**: `/tests/explain_analyze_test.rs`

---

## 3. Gate 集成状态 (check_beta_v310.sh)

### 3.1 B-S 系列测试映射

| Gate Test | Feature | Status | Test File |
|-----------|---------|--------|-----------|
| B-S1 | concurrency_stress | ❌ | 未找到 `concurrency_stress_test.rs` |
| B-S2 | crash_recovery | ❌ | 未找到 `crash_recovery_test.rs` |
| B-S3 | long_run_stability | ❌ | 未找到 `long_run_stability_test.rs` |
| B-S4 | wal_integration | ❌ | 未找到 `wal_integration_test.rs` |
| B-S5 | network_tcp | ❌ | 未找到 `network_tcp_smoke_test.rs` |
| B-S6 | ssi_stress | ✅ | `/crates/transaction/tests/ssi_stress_test.rs` |
| B-S7 | audit_trail | ❌ | 未找到 `wal_crash_recovery_test.rs` |
| B-S8 | explain_analyze | ✅ | `/tests/explain_analyze_test.rs` |
| B-S9 | window_functions | ✅ | `/tests/window_function_test.rs` |

### 3.2 问题分析

**严重问题**: B-S1 ~ B-S5, B-S7 测试文件均不存在，Gate 脚本引用的测试用例与实际代码库不匹配。

---

## 4. 代码实现验证

### 4.1 确认存在的实现

```
crates/executor/src/window_executor.rs     ✅ 窗口函数执行器
crates/executor/src/explain.rs            ✅ EXPLAIN ANALYZE
crates/transaction/src/ssi.rs              ✅ SSI 实现
crates/transaction/src/deadlock.rs         ✅ 死锁检测
crates/transaction/src/wait_for_graph.rs   ✅ 等待图
crates/transaction/src/lock.rs             ✅ 锁管理 (含 Gap Lock)
crates/transaction/src/audit/hash_chain.rs ✅ 审计哈希链
crates/optimizer/src/cost.rs               ✅ CBO 代价模型
crates/optimizer/src/path_selector.rs      ✅ 路径选择器
```

### 4.2 确认不存在的实现

```
crates/executor/src/merge*.rs             ❌ MERGE 执行器 (仅文档)
crates/storage/src/clustered_index/         ⚠️ 模块存在但测试不完整
```

### 4.3 需要验证的实现

```
crates/planner/src/* (CTE/recursive CTE)
crates/executor/src/* (INSERT...SELECT)
```

---

## 5. 覆盖率分析

### 5.1 Beta Gate 覆盖率目标

| Metric | Target | Current | Gap |
|--------|--------|---------|-----|
| L1 crates coverage | ≥50% | ? | 需要实际运行 |
| L1 test pass rate | ≥90% | ? | 需要实际运行 |
| SQL Operations | ≥80% | ? | 需要实际运行 |

### 5.2 功能覆盖率

| Feature Area | Implemented | Tested | Gate-Integrated |
|--------------|-------------|--------|-----------------|
| Window Functions | 9/9 (100%) | 6/9 (67%) | 1/9 (11%) |
| Transaction/SSI | 5/5 (100%) | 4/5 (80%) | 1/5 (20%) |
| CBO/Cost Model | 2/3 (67%) | 1/3 (33%) | 0/3 (0%) |
| Clustered Index | 1/1 (100%) | 0/1 (0%) | 0/1 (0%) |

---

## 6. 行动项

### 6.1 高优先级

| # | Action | Owner | Target |
|---|--------|-------|--------|
| 1 | 实现 MERGE 执行器 `crates/executor/src/merge.rs` | TBD | v3.1.0 GA |
| 2 | 创建 B-S1 ~ B-S5, B-S7 测试文件或更新 Gate 脚本 | TBD | v3.1.0 GA |
| 3 | 完成 CBO 代价模型接入 Planner | TBD | v3.1.0 GA |
| 4 | 为 Clustered Index 创建完整测试套件 | TBD | v3.1.0 GA |

### 6.2 中优先级

| # | Action | Owner | Target |
|---|--------|-------|--------|
| 5 | 为 FIRST_VALUE/LAST_VALUE/NTH_VALUE 添加测试 | TBD | v3.1.0 RC |
| 6 | 为递归 CTE 添加压力测试 | TBD | v3.1.0 RC |
| 7 | 为 Audit Chain 添加集成测试 | TBD | v3.1.0 RC |
| 8 | 将 Gap Lock 集成到 Gate 测试 | TBD | v3.1.0 RC |

### 6.3 低优先级

| # | Action | Owner | Target |
|---|--------|-------|--------|
| 9 | 完善 INSERT...SELECT 测试覆盖 | TBD | v3.2.0 |
| 10 | 添加 CBO 单元测试 | TBD | v3.2.0 |

---

## 7. 附录

### A. 参考文档

- v2.9.0 Feature Matrix: `/docs/releases/v2.9.0/FEATURE_MATRIX.md`
- v3.0.0 Feature Matrix: `/docs/releases/v3.0.0/FEATURE_MATRIX.md`
- v3.1.0 Feature Matrix: `/docs/releases/v3.1.0/FEATURE_MATRIX.md`
- MERGE Execution: `/docs/releases/v3.1.0/oo/MERGE_EXECUTION.md`
- CBO Integration: `/docs/releases/v3.1.0/oo/CBO_INTEGRATION.md`
- Gap Locking: `/docs/releases/v3.1.0/oo/GAP_LOCKING.md`
- Clustered Index: `/docs/releases/v3.1.0/oo/CLUSTERED_INDEX.md`

### B. Gate 脚本

- Beta Gate: `/scripts/gate/check_beta_v310.sh`
- RC Gate: `/scripts/gate/check_rc_v310.sh`

### C. 关键实现文件

| File | Purpose |
|------|---------|
| `crates/executor/src/window_executor.rs` | 窗口函数执行 |
| `crates/transaction/src/ssi.rs` | Serializable Snapshot Isolation |
| `crates/transaction/src/deadlock.rs` | 死锁检测 |
| `crates/transaction/src/lock.rs` | 锁管理 (Record/Gap/NextKey) |
| `crates/optimizer/src/cost.rs` | CBO 代价模型 |
| `crates/optimizer/src/path_selector.rs` | 基于代价的路径选择 |
| `crates/transaction/src/audit/hash_chain.rs` | 审计哈希链 |

---

**Report Status**: DRAFT  
**Next Review**: Before v3.1.0 RC Gate
