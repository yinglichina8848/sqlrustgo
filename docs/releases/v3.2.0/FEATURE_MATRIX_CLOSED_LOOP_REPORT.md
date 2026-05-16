# SQLRustGo 功能矩阵闭环检查报告

> **版本**: v1.0
> **日期**: 2026-05-17
> **检查范围**: v1.0.0 → v3.2.0 (全部 27 个版本)
> **当前分支**: develop/v3.2.0

---

## 一、执行摘要

### 1.1 版本覆盖概览

| 维度 | 数值 |
|------|------|
| 历史版本总数 | 27 个 (v1.0.0 ~ v3.3.0) |
| 有 CHANGELOG 的版本 | 16 个 |
| 有 FEATURE_MATRIX 的版本 | 8 个 (从 v1.9.0 起) |
| 有 DEVELOPMENT_PLAN 的版本 | 13 个 |
| 有 TEST_PLAN 的版本 | 9 个 |
| 有跨版本追踪的版本 | 仅 v3.1.0 |

### 1.2 v3.2.0 当前状态

| 维度 | 状态 | 目标 |
|------|------|------|
| 代码实现 | 27/33 功能已实现 (82%) | — |
| 有测试的功能 | 20/33 功能有测试 (61%) | — |
| 门禁集成 | 18/33 功能已集成 (55%) | — |
| Beta Gate 覆盖率 | 46.63% | ≥75% |
| GA Gate 覆盖率 | 0% (未执行) | ≥85% |
| DEVELOPMENT_PLAN 章节 | 4/9 完整 | 9/9 |

### 1.3 关键问题

| 优先级 | 问题 | 影响版本 |
|--------|------|----------|
| 🔴 P0 | 13 个功能声称有测试但测试文件不存在 | v3.2.0 |
| 🔴 P0 | Beta 覆盖率 46.63% 远低于 75% 要求 | v3.2.0 |
| 🔴 P0 | GA Gate 引用 `audit_trail_test.rs` 但文件不存在 | v3.2.0 |
| 🟡 P1 | DEVELOPMENT_PLAN 缺少 5 个必需章节 | v3.2.0 |
| 🟡 P1 | CHANGELOG 与实际开发状态不同步 | v3.2.0 |
| 🟡 P1 | 早期版本 (v1.x) 文档严重缺失 | v1.0.0~v1.8.0 |

---

## 二、版本文档完整性总表

| 版本 | CHANGELOG | FEATURE_MATRIX | DEVELOPMENT_PLAN | TEST_PLAN | 文档质量 |
|------|-----------|----------------|-----------------|-----------|----------|
| v1.0.0 | ❌ | ❌ | ❌ | ❌ | 缺失 |
| v1.0.0-rc1 | ❌ | ❌ | ❌ | ❌ | 缺失 |
| v1.1.0 | ❌ | ❌ | ❌ | ❌ | 缺失 |
| v1.2.0 | ❌ | ❌ | ❌ | ✅ | 仅 TEST_PLAN |
| v1.3.0 | ✅ | ❌ | ✅ | ✅ | 较完整 |
| v1.4.0 | ✅ | ❌ | ✅ | ❌ | 基础 |
| v1.5.0 | ✅ | ❌ | ✅ | ❌ | 基础 |
| v1.6.0 | ✅ | ❌ | ✅ | ❌ | 基础 |
| v1.6.1 | ❌ | ❌ | ❌ | ❌ | 缺失 |
| v1.7.0 | ❌ | ❌ | ✅ (GOALS) | ❌ | 仅规划 |
| v1.8.0 | ✅ | ❌ | ❌ | ❌ | 仅 CHANGELOG |
| v1.9.0 | ✅ | ✅ | ❌ | ✅ | 首个 FEATURE_MATRIX |
| v2.0 | ❌ | ❌ | ✅ | ❌ | 仅 DEV_PLAN |
| v2.0.0 | ✅ | ❌ | ✅ | ❌ | 基础 |
| v2.1.0 | ✅ | ❌ | ✅ | ✅ | 较完整 |
| v2.2.0 | ❌ | ❌ | ✅ | ❌ | 仅 DEV_PLAN |
| v2.3.0 | ❌ | ❌ | ✅ | ❌ | 仅 DEV_PLAN |
| v2.4.0 | ✅ | ❌ | ❌ | ❌ | 仅 CHANGELOG |
| v2.5.0 | ⚠️ symlink | ❌ | ❌ | ❌ | 仅链接 |
| v2.6.0 | ✅ | ✅ | ✅ | ✅ | 规范化起点 |
| v2.7.0 | ✅ | ✅ | ✅ | ✅ | 规范化 |
| v2.8.0 | ✅ | ✅ | ✅ | ✅ | 规范化 |
| v2.9.0 | ✅ | ✅ | ✅ | ✅ | 规范化 |
| v3.0.0 | ✅ | ✅ | ✅ | ✅ | GA 达成 |
| v3.1.0 | ✅ | ✅ | ✅ | ✅ | ★ 含 CROSS_VERSION |
| v3.2.0 | ✅ | ✅ | ✅ | ✅ | 规范但章节缺失 |
| v3.3.0 | ⚠️ 进行中 | ❌ | ❌ | ❌ | 刚创建 |

**文档规范化从 v2.6.0 开始**，此前版本文档普遍不完整。

---

## 三、跨版本功能演进追踪

### 3.1 功能引入时间线

| 功能类别 | 功能名称 | v1.x | v2.0 | v2.5 | v2.6 | v2.7 | v2.8 | v2.9 | v3.0 | v3.1 | v3.2 |
|----------|---------|------|------|------|------|------|------|------|------|------|------|
| **DML** | SELECT/INSERT/UPDATE/DELETE | ✅ | — | — | — | — | — | — | — | — | — |
| | REPLACE INTO | — | — | — | — | — | — | ✅ | — | — | — |
| | INSERT...SELECT | — | — | — | — | — | — | — | ✅ | — | — |
| | Multi-Table UPDATE | — | — | — | — | — | — | — | — | — | ✅ |
| | MERGE INTO | — | — | — | — | — | — | — | — | ✅ | ✅ |
| **事务** | MVCC/Snapshot Isolation | — | — | — | — | — | — | ✅ | — | — | — |
| | SSI (Serializable) | — | — | — | — | — | — | — | ✅ | — | — |
| | Gap Lock / Next-Key Lock | — | — | — | — | — | — | — | — | ✅ | — |
| | Deadlock Detection | — | — | — | — | — | — | — | ✅ | ✅ | ✅ |
| **优化器** | PredicatePushdown | — | — | — | — | — | — | — | ✅ | — | — |
| | CBO Cost Model | — | — | — | — | — | — | — | — | ✅ | — |
| **窗口函数** | ROW_NUMBER/RANK | — | — | — | — | — | — | ✅ | — | — | — |
| | NTILE/LEAD/LAG | — | — | — | — | — | — | — | ✅ | — | — |
| | FIRST/LAST/NTH_VALUE | — | — | — | — | — | — | — | ✅ | — | — |
| **CTE** | Non-recursive CTE | — | — | — | — | — | — | ✅ | — | — | — |
| | Recursive CTE | — | — | — | — | — | — | — | ✅ | — | 🔄 |
| **存储** | WAL | — | — | — | ✅ | — | — | — | — | — | — |
| | Buffer Pool | — | — | ✅ | — | — | — | — | — | — | — |
| | Clustered Index | — | — | — | — | — | — | — | — | ✅ | — |
| | 冷存储分层 | — | — | — | — | — | — | — | — | — | ✅ |
| **GMP** | 数字签名审计链 | — | — | — | — | — | — | — | — | — | ✅ |
| | 电子签名 | — | — | — | — | — | — | — | — | — | ✅ |
| | Immutable Record | — | — | — | — | — | — | — | — | — | ✅ |
| | Correction Chain | — | — | — | — | — | — | — | — | — | ✅ |
| | Provenance Tracking | — | — | — | — | — | — | — | — | — | ✅ |
| | Trusted Timestamp | — | — | — | — | — | — | — | — | — | ✅ |
| | HSM/KMS 集成 | — | — | — | — | — | — | — | — | — | ✅ |
| | Workflow Engine | — | — | — | — | — | — | — | — | — | ✅ |
| **性能** | 连接池 200+ | — | — | — | — | — | — | — | — | — | ✅ |
| | TPC-H SF=10 | — | — | — | — | — | — | — | — | — | 🔄 |
| **分布式** | Semi-sync 复制 | — | — | — | — | — | ✅ | — | — | — | — |
| | XA 事务 | — | — | — | — | — | ✅ | — | — | — | — |
| | 读写分离 | — | — | — | — | — | ✅ | — | — | — | — |
| | 组复制 | — | — | — | — | — | — | — | — | — | 🔄 v3.3.0 |

---

## 四、v3.2.0 功能闭环追踪矩阵（经验证）

### 4.1 GMP 合规功能

| ID | 功能名称 | 代码存在 | 测试文件 | 测试存在 | 门禁项 | 门禁集成 | 状态 |
|----|---------|---------|---------|---------|--------|---------|------|
| GMP-1 | 数字签名审计链 | ✅ | `gmp_signature_chain_test.rs` | ✅ | G-QA8, G-S11 | ✅ | ✅ 已完成 |
| GMP-2 | 电子签名 (21 CFR Part 11) | ✅ | `gmp_electronic_signature_test.rs` | ✅ | G-QA9 | ✅ | ✅ 已完成 |
| GMP-3 | Immutable Record / EBR | ✅ | `gmp_immutable_record_test.rs` | ❌ | — | ❌ | ⚠️ 测试缺失 |
| GMP-4 | Correction Chain | ✅ | `gmp_correction_chain_test.rs` | ❌ | — | ❌ | ⚠️ 测试缺失 |
| GMP-5 | Provenance Tracking | ✅ | `gmp_provenance_test.rs` | ❌ | — | ❌ | ⚠️ 测试缺失 |
| GMP-6 | Trusted Timestamp (RFC3161) | ✅ | `gmp_timestamp_test.rs` | ❌ | — | ❌ | ⚠️ 测试缺失 |
| GMP-7 | 审计链验证工具 | ✅ | `gmp_audit_chain_verify_test.rs` | ✅ | G-QA12 | ✅ | ✅ 已完成 |
| GMP-8 | HSM/KMS 集成 | ✅ | `gmp_hsm_test.rs` | ❌ | — | ❌ | ⚠️ 测试缺失 |
| GMP-9 | GMP Workflow Engine | ✅ | `gmp_workflow_test.rs` | ❌ | — | ❌ | ⚠️ 测试缺失 |
| GMP-10 | 移动端可信采集 | ✅ | `gmp_mobile_test.rs` | ✅ | — | — | ✅ 已完成 |
| GMP-11 | SOP/培训绑定 | ✅ | `gmp_sop_test.rs` | ✅ | G-QA13 | ✅ | ✅ 已完成 |
| GMP-12 | Device Calibration | ✅ | `gmp_calibration_test.rs` | ✅ | — | — | ✅ 已完成 |

**GMP 小计**: 代码 ✅ 12/12, 测试 ✅ 6/12, 门禁集成 ✅ 4/12

### 4.2 性能功能

| ID | 功能名称 | 代码存在 | 测试文件 | 测试存在 | 门禁项 | 门禁集成 | 状态 |
|----|---------|---------|---------|---------|--------|---------|------|
| PERF-1 | Point SELECT QPS ≥1M | ⚠️ 部分 | `qps_benchmark_test.rs` | ✅ | G9 | ✅ | 🔄 进行中 |
| PERF-2 | TPC-H SF=10 | ⚠️ 部分 | `tpch_test.rs` (SF=1) | ✅ | G8 | ✅ | 🔄 进行中 |
| PERF-3 | 并发连接 200+ | ✅ | `concurrency_stress_test.rs` | ✅ | G9, G-S1 | ✅ | ✅ 已完成 |
| PERF-4 | 死锁检测优化 <50ms | ✅ | `gap_locking_e2e_test.rs` | ✅ | G9, G-S6 | ✅ | ✅ 已完成 |
| PERF-5 | 内存优化 -15% | ⚠️ 部分 | `memory_footprint_test.rs` | ❌ | — | ❌ | 🔄 进行中 |

**性能小计**: 代码 ✅ 3/5, 测试 ✅ 3/5, 门禁集成 ✅ 3/5

### 4.3 SQL 功能

| ID | 功能名称 | 代码存在 | 测试文件 | 测试存在 | 门禁项 | 门禁集成 | 状态 |
|----|---------|---------|---------|---------|--------|---------|------|
| SQL-1 | RECURSIVE CTE | ⚠️ 部分 | `cte_tests.rs` | ✅ (部分) | G7 | ✅ | 🔄 进行中 |
| SQL-2 | Performance Schema | ⚠️ 部分 | `ps_setup_actors_test.rs` | ❌ | G7 | ✅ | 🔄 进行中 |
| SQL-3 | 冷存储集成 | ✅ | `cold_storage_test.rs` | ❌ | — | ❌ | 🔄 测试缺失 |
| SQL-4 | 组复制 | 🔄 规划 | — | — | — | — | 🔄 v3.3.0 |
| SQL-5 | 自动故障转移 | 🔄 规划 | — | — | — | — | 🔄 v3.3.0 |
| SQL-6 | 地理分布 | 🔄 规划 | — | — | — | — | 🔄 v3.3.0 |
| SQL-7 | DCL 权限链完善 | ✅ | `auth_rls_test.rs` | ✅ | G7 | ✅ | ✅ 已完成 |
| SQL-8 | FULLTEXT 完善 | ✅ | `fts_tests.rs` | ✅ | G-QA1 | ✅ | ✅ 已完成 |

**SQL 小计**: 代码 ✅ 4/8, 测试 ✅ 4/8, 门禁集成 ✅ 4/8

### 4.4 架构/OO 文档

| ID | 文档名称 | 路径 | 状态 |
|----|---------|------|------|
| OO-1 | 数字签名审计链设计 | `oo/GMP/DIGITAL_SIGNATURE_CHAIN.md` | ✅ |
| OO-2 | 电子签名设计 | `oo/GMP/ELECTRONIC_SIGNATURE.md` | ✅ |
| OO-3 | Immutable Record 设计 | `oo/GMP/IMMUTABLE_RECORD.md` | ✅ |
| OO-4 | Correction Chain 设计 | `oo/GMP/CORRECTION_CHAIN.md` | ✅ |
| OO-5 | Provenance Tracking 设计 | `oo/GMP/PROVENANCE_TRACKING.md` | ✅ |
| OO-6 | HSM/KMS 集成设计 | `oo/GMP/HSM_KMS_INTEGRATION.md` | ✅ |
| OO-7 | GMP Workflow Engine 设计 | `oo/GMP/GMP_WORKFLOW_ENGINE.md` | ✅ |
| OO-8 | Trusted Timestamp 设计 | `oo/GMP/TRUSTED_TIMESTAMP.md` | ✅ |

**文档小计**: ✅ 8/8 (100%)

---

## 五、门禁脚本与测试文件不匹配问题

### 5.1 GA Gate (check_ga_v320.sh) 问题

| 门禁项 | 测试命令 | 文件存在 | 问题 |
|--------|---------|---------|------|
| G-S7 | `audit_trail_test` | ❌ 缺失 | 门禁引用了不存在的测试 |
| G-S4 | `wal_integration_test` | ✅ 存在 | — |
| G-S3 | `long_run_stability_test` | ✅ 存在 | — |
| G-S2 | `crash_recovery_test` | ✅ 存在 | — |
| G-S1 | `concurrency_stress_test` | ✅ 存在 | — |

**严重问题**: GA Gate 脚本第 G-S7 项引用 `audit_trail_test.rs`，该文件在 `crates/executor/tests/` 下不存在，导致门禁检查会失败。

### 5.2 Beta Gate (check_beta_v320.sh) 问题

根据 v3.1.0 的 CROSS_VERSION 报告，Beta Gate 存在类似问题：

| 门禁项 | 测试命令 | 文件存在 | 问题 |
|--------|---------|---------|------|
| B-S1 | `concurrency_stress_test` | ✅ 存在 | — |
| B-S2 | `crash_recovery_test` | ✅ 存在 | — |
| B-S3 | `long_run_stability_test` | ✅ 存在 | — |
| B-S4 | `wal_integration_test` | ✅ 存在 | — |
| B-S5 | `network_tcp_smoke_test` | ✅ 存在 | — |
| B-S6 | `ssi_stress_test` | ✅ 存在 | — |
| B-S7 | `audit_trail_test` | ❌ 缺失 | 与 GA Gate 相同问题 |
| B-S8 | `explain_analyze_test` | ✅ 存在 | — |
| B-S9 | `window_function_test` | ✅ 存在 | — |

---

## 六、缺失测试文件清单

### 6.1 按功能分类

| 优先级 | 测试文件 | 对应功能 | 当前状态 |
|--------|---------|---------|---------|
| 🔴 P0 | `gmp_immutable_record_test.rs` | GMP-3 Immutable Record | 代码有, 测试无 |
| 🔴 P0 | `gmp_correction_chain_test.rs` | GMP-4 Correction Chain | 代码有, 测试无 |
| 🔴 P0 | `gmp_provenance_test.rs` | GMP-5 Provenance Tracking | 代码有, 测试无 |
| 🔴 P0 | `gmp_timestamp_test.rs` | GMP-6 Trusted Timestamp | 代码有, 测试无 |
| 🔴 P0 | `gmp_hsm_test.rs` | GMP-8 HSM/KMS Integration | 代码有, 测试无 |
| 🔴 P0 | `gmp_workflow_test.rs` | GMP-9 Workflow Engine | 代码有, 测试无 |
| 🔴 P0 | `audit_trail_test.rs` | 审计链验证 (GA/Beta Gate) | **门禁会失败** |
| 🔴 P0 | `cold_storage_test.rs` | SQL-3 冷存储集成 | 代码有, 测试无 |
| 🟡 P1 | `recursive_cte_test.rs` | SQL-1 RECURSIVE CTE | 测试不完整 |
| 🟡 P1 | `memory_footprint_test.rs` | PERF-5 内存优化 | 测试无 |
| 🟡 P1 | `ps_setup_actors_test.rs` | SQL-2 Performance Schema | 测试无 |
| 🟡 P1 | `four_eyes_test.rs` | GMP-2 电子签名四眼原则 | 测试无 |

### 6.2 门禁必须修复项

以下测试文件的缺失会导致 GA Gate 检查失败，必须优先修复：

1. **`crates/executor/tests/audit_trail_test.rs`** — GA Gate G-S7 和 Beta Gate B-S7 均引用

---

## 七、历史版本遗漏问题追踪

### 7.1 v3.1.0 遗留到 v3.2.0 的未完成项

| 原问题 | v3.1.0 状态 | v3.2.0 状态 | 说明 |
|--------|------------|------------|------|
| MERGE 执行器 | ❌ 未实现 | ✅ 已实现 | PR #1021 |
| CBO 代价模型接入 Planner | ❌ 未完成 | ⚠️ 仍存在问题 | v3.1.0 CROSS_VERSION 记录 |
| Clustered Index 完整测试 | ⚠️ 测试不完整 | ⚠️ 仍有问题 | 需要独立测试套件 |
| FIRST/LAST/NTH_VALUE 测试 | ⚠️ 缺失 | ✅ 有测试 | `window_function_test.rs` 覆盖 |
| 递归 CTE 压力测试 | ⚠️ 缺失 | ⚠️ 仍缺失 | 仅有 `cte_tests.rs` 基础测试 |
| Audit Chain 集成测试 | ⚠️ 缺失 | ⚠️ 仍缺失 | 无独立 `audit_trail_test.rs` |
| Gap Lock Gate 集成 | ❌ 未集成 | ✅ 已集成 | `gap_locking_e2e_test.rs` |

### 7.2 长期存在的架构问题

| 问题 | 首次发现 | 当前状态 | 持续版本 |
|------|----------|----------|----------|
| CBO 未真正接入物理计划选择 | v3.0.0 | ⚠️ 仍为硬编码 | 3个版本 |
| 文档规范化 (v1.x 缺失) | v1.0.0 | ⚠️ 仍有6个版本无文档 | 持续 |
| 门禁脚本与测试文件不匹配 | v3.1.0 | ❌ GA Gate 仍有问题 | 2个版本 |

---

## 八、v3.2.0 闭环差距分析

### 8.1 代码到测试的闭环

| 分类 | 功能总数 | 有测试 | 无测试 | 测试率 |
|------|---------|--------|--------|--------|
| GMP | 12 | 6 | 6 | 50% |
| 性能 | 5 | 3 | 2 | 60% |
| SQL | 8 | 4 | 4 | 50% |
| 文档 | 8 | N/A | — | 100% |
| **合计** | **33** | **13** | **12** | **52%** |

### 8.2 测试到门禁的闭环

| 分类 | 有测试功能 | 已进门禁 | 未进门禁 | 门禁率 |
|------|-----------|---------|---------|--------|
| GMP | 6 | 4 | 2 | 67% |
| 性能 | 3 | 3 | 0 | 100% |
| SQL | 4 | 4 | 0 | 100% |
| **合计** | **13** | **11** | **2** | **85%** |

### 8.3 门禁到验证的闭环

当前 GA Gate 引用了 20 个测试命令，其中：
- ✅ 19 个测试文件存在
- ❌ 1 个测试文件不存在 (`audit_trail_test.rs`)

---

## 九、修复行动计划

### Phase 1: 门禁阻塞项修复 (P0)

| 序号 | 行动 | 负责 | 目标 |
|------|------|------|------|
| 1 | 创建 `crates/executor/tests/audit_trail_test.rs` | TBD | 使 GA Gate G-S7 可执行 |
| 2 | 确认 `audit_trail_test.rs` 测试的是审计链验证功能 | TBD | 验证功能覆盖 |

### Phase 2: GMP 测试补充 (P0)

| 序号 | 测试文件 | 对应功能 | 优先级 |
|------|---------|---------|--------|
| 3 | `gmp_immutable_record_test.rs` | GMP-3 Immutable Record | 🔴 P0 |
| 4 | `gmp_correction_chain_test.rs` | GMP-4 Correction Chain | 🔴 P0 |
| 5 | `gmp_provenance_test.rs` | GMP-5 Provenance | 🔴 P0 |
| 6 | `gmp_timestamp_test.rs` | GMP-6 Trusted Timestamp | 🔴 P0 |
| 7 | `gmp_hsm_test.rs` | GMP-8 HSM/KMS | 🔴 P0 |
| 8 | `gmp_workflow_test.rs` | GMP-9 Workflow Engine | 🔴 P0 |

### Phase 3: SQL/性能测试补充 (P1)

| 序号 | 测试文件 | 对应功能 | 优先级 |
|------|---------|---------|--------|
| 9 | `cold_storage_test.rs` | SQL-3 冷存储 | 🟡 P1 |
| 10 | `recursive_cte_test.rs` | SQL-1 RECURSIVE CTE | 🟡 P1 |
| 11 | `memory_footprint_test.rs` | PERF-5 内存优化 | 🟡 P1 |
| 12 | `ps_setup_actors_test.rs` | SQL-2 Performance Schema | 🟡 P1 |
| 13 | `four_eyes_test.rs` | GMP-2 四眼原则 | 🟡 P1 |

### Phase 4: 文档修复 (P1)

| 序号 | 行动 | 目标 |
|------|------|------|
| 14 | 补全 DEVELOPMENT_PLAN.md 缺失的 5 个章节 | 架构决策、API约定、迁移指南、取消/延期项、延续任务 |
| 15 | 同步 CHANGELOG 与实际开发状态 | 补录 GMP-10~12, PERF-4 |
| 16 | 更新 FEATURE_TRACKING_MATRIX.md | 替换虚报数据为实际验证结果 |

### Phase 5: 覆盖率提升 (P0)

| 序号 | 目标 | 当前 | 目标 |
|------|------|------|------|
| 17 | Beta Gate 覆盖率 | 46.63% | ≥75% |
| 18 | GA Gate 覆盖率 | 0% (未执行) | ≥85% |

---

## 十、结论

### 10.1 整体评估

| 维度 | 评分 | 说明 |
|------|------|------|
| 文档覆盖 | 65% | v2.6+ 规范, v1.x 缺失 |
| 代码实现 | 82% | 27/33 功能已实现 |
| 测试覆盖 | 52% | 13/25 功能有测试 |
| 门禁集成 | 55% | 18/33 功能已集成 |
| 门禁准确性 | 95% | 仅 1/20 测试文件缺失 |

### 10.2 最大风险

1. **GA Gate 会因 `audit_trail_test.rs` 缺失而失败** — 必须优先修复
2. **Beta 覆盖率 46.63% 远低于 75% 要求** — 可能阻塞 RC Gate
3. **DEVELOPMENT_PLAN 缺少 5 个必需章节** — 不符合 governance 规范
4. **6 个 GMP 功能代码已完成但无测试** — 质量无保证

### 10.3 建议

1. 立即创建 `audit_trail_test.rs` 解除 GA Gate 阻塞
2. 优先补充 GMP-3~6, 8~9 的测试文件
3. 补全 DEVELOPMENT_PLAN 章节
4. 执行 RC Gate 检查实际覆盖率

---

*本文档由 hermes-agent 生成*
*数据核验日期: 2026-05-17*
*基于: v3.2.0 FEATURE_MATRIX.md, CHANGELOG.md, CROSS_VERSION_FEATURE_MATRIX.md (v3.1.0), GA Gate 脚本*
