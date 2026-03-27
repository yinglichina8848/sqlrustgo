# SQLRustGo v1.9.0 数据库内核工程化强化计划

> **版本**: v1.9.0  
> **阶段**: Engineering Hardening Phase  
> **目标**: 单机数据库内核稳定化版本  
> **更新日期**: 2026-03-26

---

## 一、阶段定位

### 1.1 v1.9.x 核心目标

**不是**：继续加功能

**而是**：数据库内核工程化强化

```
目标：验证数据库正确性 + 稳定性 + 可部署能力
```

### 1.2 当前阶段

SQLRustGo 已进入 **L3.5 → L4** (教学数据库 → 研究级数据库内核)

v1.9.x 是 **向量化执行引擎 (v2.0) 之前的稳定化窗口期**

---

## 二、强化目标

### 2.1 核心指标

| 指标 | 当前状态 | 目标 |
|------|----------|------|
| SQL semantic coverage | 60-70% | 80%+ |
| Crash Safety Confidence | 中等 | 高 |
| QPS 验证 | 未测试 | ≥3k (单线程) |
| 并发连接 | 未测试 | ≥50 |
| long-run stability | 未验证 | 72h 无崩溃 |

### 2.2 成功标准

- [ ] Crash injection tests: 10+ 测试
- [ ] Catalog consistency tests: 10+ 测试
- [ ] MVCC anomaly tests: 10+ 测试
- [ ] QPS benchmark: ≥3k (单线程)
- [ ] 72h stability: 通过

---

## 三、任务清单

### 3.1 P0 级（发布前必须完成）

| ISSUE | 任务 | 测试数 | 状态 |
|-------|------|--------|------|
| #843 | Crash Injection Test Matrix | 10+ | ⬜ |
| #846 | Catalog Consistency Verification | 10+ | ⬜ |
| #845 | MVCC & Concurrency Anomalies | 10+ | ⬜ |
| #842 | QPS/并发性能目标验证 | 20+ | ⬜ |
| #847 | 72h Long-Run Stability Test | 5+ | ⬜ |

### 3.2 P1 级（推荐完成）

| ISSUE | 任务 | 状态 |
|-------|------|------|
| #849 | SQL Fuzz Testing (SQLancer) | ⬜ |
| #850 | Random Transaction Stress Test | ⬜ |
| #852 | 并发连接数测试 | ⬜ |

### 3.3 P2 级（提升工程成熟度）

| ISSUE | 任务 | 状态 |
|-------|------|------|
| #851 | EXPLAIN ANALYZE 增强 | ⬜ |
| #852 | INFORMATION_SCHEMA 支持 | ⬜ |
| #853 | 查询统计系统 | ⬜ |

---

## 四、最优推进顺序

### 第一阶段：核心正确性（1-2 周）

1. **Crash injection tests (#843)**
   - kill -9 during WAL write
   - kill -9 during commit
   - kill -9 during checkpoint
   - kill -9 during index insert
   - kill -9 during page split
   - kill -9 during buffer flush

2. **Catalog consistency tests (#846)**
   - CREATE/DROP/ALTER TABLE
   - CREATE/DROP INDEX
   - Schema metadata integrity
   - System tables correctness

3. **MVCC anomaly tests (#845)**
   - Dirty Read
   - Non-repeatable Read
   - Phantom Read
   - Lost Update

### 第二阶段：性能验证（1 周）

4. **QPS benchmark (#842)**
   - single-thread QPS ≥ 3k
   - 8-thread QPS ≥ 12k
   - mixed workload

5. **Concurrency benchmark (#850)**
   - 10 connections
   - 50 connections
   - 100 connections

6. **72h stability (#847)**
   - memory leak detection
   - WAL corruption detection
   - buffer pool stability

### 第三阶段：高级特性（1 周）

7. **SQLancer fuzz testing (#849)**
   - 自动生成 1000+ 随机 SQL
   - crash detection
   - incorrect result detection

8. **Random stress test (#850)**
   - 10+ concurrent workers
   - 10000+ random transactions
   - consistency verification

### 第四阶段：工程增强（1 周）

9. **EXPLAIN ANALYZE (#851)**
   - buffer/cpu/memory 统计
   - 毫秒级精度

10. **INFORMATION_SCHEMA (#852)**
    - tables/columns/constraints
    - 外键查询

11. **Query statistics (#853)**
    - Top N queries
    - Performance metrics

---

## 五、不建议做的事情

| 功能 | 原因 |
|------|------|
| 窗口函数 | SQL feature inflation |
| 列存储 | 属于 v2.0 |
| SIMD | 属于 v2.0 |
| 分布式 | 属于 v3.0 |
| Cascades optimizer | 属于 v2.0 |

---

## 六、评分权重（创业竞赛/省级一等奖）

| 指标 | 权重 | 说明 |
|------|------|------|
| 工程复杂度 | ⭐⭐⭐⭐⭐ | 核心引擎完整度 |
| 系统完整度 | ⭐⭐⭐⭐⭐ | SQL/Catalog/Transaction 完备 |
| 可靠性验证 | ⭐⭐⭐⭐⭐ | Crash/MVCC/Recovery 测试 |
| 创新点 | ⭐⭐⭐⭐ | Trait-based extensibility |
| 落地能力 | ⭐⭐⭐⭐ | 部署文档/性能验证 |

**最加分**:
- crash safety matrix
- mvcc anomaly matrix
- fuzz testing

**不是**:
- 窗口函数
- JSON 类型
- 更多 SQL 语法

---

## 七、测试目录结构

```
tests/
├── crash_injection/           # P0 - 新增
│   ├── crash_during_commit_test.rs
│   ├── crash_during_checkpoint_test.rs
│   ├── crash_during_index_insert_test.rs
│   └── crash_during_page_split_test.rs
│
├── catalog_consistency/       # P0 - 新增
│   ├── schema_metadata_integrity_test.rs
│   ├── table_crud_consistency_test.rs
│   ├── index_metadata_test.rs
│   └── foreign_key_validation_test.rs
│
├── mvcc_anomaly/              # P0 - 新增
│   ├── dirty_read_test.rs
│   ├── non_repeatable_read_test.rs
│   ├── phantom_read_test.rs
│   └── lost_update_test.rs
│
├── recovery_matrix/          # P0 - 新增
│   ├── partial_wal_recovery_test.rs
│   ├── double_restart_recovery_test.rs
│   └── checkpoint_recovery_test.rs
│
├── fuzz_sqlancer/            # P1 - 新增
│   └── sqlancer_test.rs
│
├── random_txn_stress/        # P1 - 新增
│   └── random_transaction_test.rs
│
├── long_run/                  # P1 - 新增
│   ├── 12h_stability_test.rs
│   └── memory_leak_detection_test.rs
│
└── qps_benchmark/            # P1 - 新增
    ├── single_thread_qps_test.rs
    ├── multi_thread_qps_test.rs
    └── mixed_workload_test.rs
```

---

## 八、验收检查

### 发布前检查 (P0)

- [ ] 所有 P0 测试通过 (50+ 测试)
- [ ] QPS ≥ 3k (单线程)
- [ ] 50+ 并发连接稳定
- [ ] 72h 稳定性测试通过

### 发布检查 (P1)

- [ ] SQLancer fuzz 测试通过
- [ ] 随机事务压力测试通过

### 完善检查 (P2)

- [ ] EXPLAIN ANALYZE 输出完整
- [ ] INFORMATION_SCHEMA 可用
- [ ] 查询统计系统可用

---

## 九、参考文档

- [全面评估报告](FINAL_ASSESSMENT_REPORT.md)
- [技术路线图](../v2.0/TECH_ROADMAP.md)
- [部署指南](DEPLOYMENT_GUIDE.md)
- [门禁清单](RELEASE_GATE_CHECKLIST.md)

---

## 十、版本标记

| 标记 | 条件 |
|------|------|
| RC for developer preview | 满足所有 P0 |
| 正式版 | 满足所有 P0 + P1 |

---

*计划制定: 2026-03-26*
*阶段: Engineering Hardening*
*版本: v1.9.0*
