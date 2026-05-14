# v3.1.0 开发计划

> **版本**: v3.1.0  
> **日期**: 2026-05-11  
> **状态**: 🟡 规划中  
> **基于**: v3.0.0 GA (65440326)

---

## 一、v3.0.0 遗留问题继承

### 1.1 P0 阻塞项（Beta Gate 必须解决）

| 原 Issue | 任务 | v3.0.0 状态 | v3.1.0 目标 | 验收条件 |
|---------|------|------------|-------------|----------|
| #451 | SQL Operations 语法支持 | 20% (11/55) | ≥80% (44/55) | `test_sql_corpus_operations` 通过率 ≥80% |
| #392 | CBO 代价模型集成 | 未开始 | SimpleCostModel 接入 planner | EXPLAIN 能选择索引扫描而非全表 |
| — | TPC-H SF=1 无 OOM | 曾 OOM | 22/22 无 OOM，p99 < 5s | `check_tpch.sh sf=1` 22/22 全部通过 |
| #379 | 事务状态机压力测试 | 未开始 | crash_recovery_test 全部 PASS | B-S2 PASS，100 并发无状态泄漏 |

### 1.2 P1 阻塞项（RC Gate 必须解决）

| 原 Issue | 任务 | v3.0.0 状态 | v3.1.0 目标 | 验收条件 |
|---------|------|------------|-------------|----------|
| #380 | Optimizer 测试扩展 | 未开始 | 覆盖率 ≥75% | `cargo llvm-cov -p sqlrustgo-optimizer` ≥ 75% |
| #381 | Planner 测试扩展 | 未开始 | 覆盖率 ≥80% | `cargo llvm-cov -p sqlrustgo-planner` ≥ 80% |
| — | 连接池/缓存/Group Commit | 部分完成 | 并发压力测试通过 | 连接池泄漏检测、缓存 DML 失效、WAL 崩溃恢复全部 PASS |

### 1.3 GA Gate 遗留（GA 前必须解决）

| 遗留编号 | 任务 | 验收条件 |
|---------|------|----------|
| GA-GAP-02 | 实现 G7/G8/G9 QPS 实际测量 | `cargo bench -- point_select` ≥ 10,000 ops/s |
| GA-GAP-03 | 统一 SQL Corpus 阈值为 ≥98% | `test_sql_corpus_all` ≥ 98% |

### 1.4 v3.0.0 功能缺口（v3.1.0 新增目标）

| 类别 | 功能 | MySQL | v3.0.0 | v3.1.0 目标 |
|------|------|-------|--------|-------------|
| 事件调度器 | CREATE EVENT | ✅ | ❌ | P0 基础支持 |
| 全文索引 | FULLTEXT | ✅ | ❌ | P1 基础支持 |
| INFORMATION_SCHEMA | 完整 | ⚠️ ~30% | P0 ≥80% |
| performance_schema | 完整 | ❌ | P1 ≥50% |
| TLS/SSL | 完整 | ✅ | — | ✅ 已实现 |
| 聚簇索引 | 主键即数据 | ✅ | ❌ | P1 设计完成 |
| ~~窗口函数增强~~ | NTILE/LEAD/LAG | ✅ 6/6 | ✅ v3.0.0 已实现 | |
| 触发器 | BEFORE/AFTER | ⚠️ 30% | P1 ≥80% | |
| SERIALIZABLE | 完整 | ⚠️ 50% | P1 ≥90% | |

---

## 二、v3.1.0 里程碑计划

```
v3.1.0-alpha  ────────────────────────────────────────────────────────────── 2026-06-01
  ├── INFORMATION_SCHEMA 基础 (SCHEMATA/TABLES/COLUMNS)
  ├── SQL Operations ≥60%
  └── 基础功能测试通过

v3.1.0-beta  ─────────────────────────────────────────────────────────────── 2026-07-01
  ├── Performance Schema 基础 (5+ 表)
  ├── TLS 完整握手指纹
  ├── TPC-H SF=1 22/22 无 OOM
  ├── CBO 代价模型接入 planner
  ├── SQL Operations ≥80%
  └── 稳定性测试 B-S1~B-S5 ≥95%

v3.1.0-rc  ──────────────────────────────────────────────────────────────── 2026-08-01
  ├── 全文索引 基础支持
  ├── 事件调度器 基础支持
  ├── Optimizer 覆盖率 ≥75%
  ├── Planner 覆盖率 ≥80%
  ├── SQL Corpus ≥95%
  └── QPS Benchmark 全部通过

v3.1.0-ga  ──────────────────────────────────────────────────────────────── 2026-09-01
  ├── GA Gate 23/23 PASS
  ├── SQL Operations ≥80%
  ├── Formal proofs ≥30
  └── 综合评分 70/100
```

---

## 三、详细功能任务

### 3.1 P0-1: INFORMATION_SCHEMA 完善

**现状**: ~30% 覆盖率  
**目标**: ≥80%

| 阶段 | 任务 | 交付物 | 时间 |
|------|------|--------|------|
| 1 | 实现 SCHEMATA/TABLES/COLUMNS | 基础表 | 1 周 |
| 2 | 实现 STATISTICS/REFERENTIAL_CONSTRAINTS | 索引信息 | 0.5 周 |
| 3 | 实现 PRIVILEGES 表 | 权限信息 | 0.5 周 |
| 4 | 实现 CHARACTER_SETS/COLLATIONS | 字符集信息 | 0.5 周 |
| 5 | 测试与文档 | 完整测试用例 | 0.5 周 |

**核心文件**:
```
crates/catalog/src/system_tables/
├── information_schema.rs
├── schemata.rs
├── tables.rs
├── columns.rs
├── statistics.rs
└── privileges.rs
```

### 3.2 P0-2: SQL Operations ≥80%

**现状**: 11/55 (20%)  
**目标**: ≥44/55 (80%)

需实现的操作（来自 #451）:
- BACKUP/RESTORE
- SAVEPOINT/ROLLBACK TO SAVEPOINT
- SET TRANSACTION ISOLATION LEVEL
- LIMIT/OFFSET 优化
- TRUNCATE TABLE
- REPLACE INTO
- SHOW (完整变体)
- EXPLAIN ANALYZE
- TEMPORARY TABLE
- ALTER TABLE INPLACE
- BATCH INSERT (多行 VALUES)

**验证**: `cargo test -p sqlrustgo-sql-corpus` 通过率 ≥80%

### 3.3 P0-3: TPC-H SF=1 无 OOM

**现状**: 曾 OOM  
**目标**: 22/22 通过，p99 < 5s

**根因分析**: 内存分配策略 + 查询执行计划  
**解决方案**:
1. 增量构建 TPC-H 数据（SF=1 分批生成）
2. 优化 Hash Join 内存分配
3. 增加流式处理减少峰值内存

**验证**: `bash scripts/gate/check_tpch.sh sf=1` 22/22 PASS

### 3.4 P0-4: 事务状态机压力测试

**现状**: crash_recovery_test 未实现
**目标**: B-S2 PASS，100 并发无状态泄漏

**核心文件**:
```
tests/crash_recovery_test.rs
tests/wal_crash_recovery_test.rs  (新建)
```

### 3.5 P0-5: MERGE 语句实现

**现状**: 0% 未实现 (grep 无结果)
**目标**: 完整实现 MERGE INTO ... WHEN MATCHED/NOT MATCHED

**验收条件**:
- `MERGE INTO t USING s ON t.id = s.id WHEN MATCHED THEN UPDATE SET ...`
- `MERGE INTO t USING s ON t.id = s.id WHEN NOT MATCHED THEN INSERT (...) VALUES (...)`
- 正确处理多表合并

**核心文件**:
```
crates/parser/src/statement.rs      # MERGE 语法解析
crates/planner/src/logical_plan.rs  # MERGE 逻辑计划
crates/executor/src/merge_executor.rs # MERGE 执行器 (新建)
```

**风险**: 中 - 需要 planner 和 executor 协同实现

### 3.6 P1-1: Performance Schema 基础

**现状**: 完全缺失  
**目标**: ≥50% (10+ 表)

| 表 | 状态 |
|---|------|
| setup_actors | 新建 |
| setup_instruments | 新建 |
| events_statements_summary_by_digest | 新建 |
| events_statements_history | 新建 |
| events_waits_summary_by_thread | 新建 |

### 3.6 P1-2: CBO 代价模型集成

**现状**: 未开始  
**目标**: SimpleCostModel 接入 planner

**验收条件**:
- `EXPLAIN SELECT * FROM t WHERE idx = 1` 选择索引扫描
- 多表 JOIN 按代价排序（小表先驱动）

### 3.7 P1-3: 全文索引

**现状**: 完全缺失  
**目标**: 中英文分词 + MATCH...AGAINST

| 阶段 | 任务 | 时间 |
|------|------|------|
| 1 | 分词器框架 Tokenizer trait | 1 周 |
| 2 | 倒排索引结构 InvertedIndex | 1 周 |
| 3 | MATCH...AGAINST 语法 + 执行 | 1 周 |
| 4 | 增量索引更新 DML 触发器 | 1 周 |

### 3.8 P1-4: 事件调度器

**现状**: 完全缺失  
**目标**: CREATE EVENT ... ON SCHEDULE

| 阶段 | 任务 | 时间 |
|------|------|------|
| 1 | 事件存储表 mysql.event | 0.5 周 |
| 2 | 调度器核心 EventScheduler | 1 周 |
| 3 | CREATE EVENT 语法 Parser | 0.5 周 |
| 4 | 测试与集成 E2E | 1 周 |

### 3.9 P1-5: 聚簇索引设计

**现状**: B+Tree 无聚簇概念
**目标**: 设计完成（GA 前实现）

**交付物**: ADR 决策记录 + 原型验证

### 3.10 P1-6: MERGE JOIN 实现

**现状**: 0% 未实现
**目标**: Sort-Merge Join 算法，支持等值 JOIN

**验收条件**:
- `SELECT * FROM t1 JOIN t2 ON t1.id = t2.id` 使用 MERGE JOIN
- 大表排序后合并，性能优于 Hash Join

**核心文件**:
```
crates/executor/src/merge_join_executor.rs  # 新建
crates/optimizer/src/join_ordering.rs        # 加入 MERGE JOIN 选择
```

**风险**: 中 - 需要排序算子支持

### 3.11 P1-7: BNL JOIN 实现

**现状**: 0% 未实现
**目标**: Block Nested Loop Join，支持非等值 JOIN

**验收条件**:
- `SELECT * FROM t1 JOIN t2 ON t1.id < t2.id` 使用 BNL JOIN
- 小表驱动大表扫描

**核心文件**:
```
crates/executor/src/bnl_join_executor.rs  # 新建
crates/optimizer/src/join_ordering.rs      # 加入 BNL JOIN 选择
```

**风险**: 低 - NestedLoop 扩展

---

## 四、测试任务

### 4.1 稳定性测试（B-S1~B-S5）

| 测试 | 路径 | v3.0.0 状态 | v3.1.0 目标 |
|------|------|------------|-------------|
| concurrency_stress_test | `tests/concurrency_stress_test.rs` | ✅ 9/9 | ✅ 9/9 |
| crash_recovery_test | `tests/crash_recovery_test.rs` | ✅ 8/8 | ✅ 9/9 (新增 1) |
| long_run_stability_test | `tests/long_run_stability_test.rs` | 🔴 10/10 #[ignore] | ✅ 10/10 |
| wal_integration_test | `tests/wal_integration_test.rs` | ✅ 16/16 | ✅ 16/16 |
| network_tcp_smoke_test | `tests/network_tcp_smoke_test.rs` | ✅ 6/6 | ✅ 8/8 (新增 2) |

### 4.2 压力测试（S-01~S-03）

| 测试 | 路径 | v3.0.0 状态 | v3.1.0 目标 |
|------|------|------------|-------------|
| connection_pool_stress_test | `tests/connection_pool_stress_test.rs` | ✅ 11/11 | ✅ 15/15 (新增 4) |
| query_cache_test | `tests/query_cache_test.rs` | 🔴 缺失 | ✅ 8/8 (新建) |
| wal_crash_recovery_test | `tests/wal_crash_recovery_test.rs` | 🔴 缺失 | ✅ 10/10 (新建) |

### 4.3 覆盖率目标

| crate | Beta 目标 | RC 目标 | GA 目标 |
|-------|-----------|---------|---------|
| sqlrustgo-parser | ≥65% | ≥70% | ≥75% |
| sqlrustgo-executor | ≥70% | ≥75% | ≥80% |
| sqlrustgo-planner | ≥60% | **≥80%** | ≥85% |
| sqlrustgo-optimizer | ≥50% | **≥75%** | ≥80% |
| sqlrustgo-storage | ≥60% | ≥70% | ≥75% |
| sqlrustgo-transaction | ≥65% | ≥70% | ≥75% |
| **总体** | **≥50%** | **≥60%** | **≥65%** |

---

## 五、门禁检查

### 5.1 Beta Gate (B1-B9 + B-S1~B-S5)

| ID | 检查项 | 通过标准 | 脚本 |
|----|--------|---------|------|
| B1 | Release Build | `cargo build --release --workspace` | check_beta_v310.sh |
| B2 | 测试 ≥90% | `cargo test --all-features` 通过率 ≥90% | check_beta_v310.sh |
| B3 | Clippy | `cargo clippy --all-features -- -D warnings` | check_beta_v310.sh |
| B4 | Format | `cargo fmt --all -- --check` | check_beta_v310.sh |
| B5 | 覆盖率 ≥50% | `cargo llvm-cov --all-features --lib` ≥50% | check_beta_v310.sh |
| B6 | 安全扫描 | `cargo audit` | check_beta_v310.sh |
| B7 | 文档链接 | `bash scripts/gate/check_docs_links.sh` | check_beta_v310.sh |
| B8 | TPC-H SF=0.1 | `bash scripts/gate/check_tpch.sh sf=0.1` 22/22 | check_beta_v310.sh |
| B9 | SQL Corpus ≥80% | `cargo test -p sqlrustgo-sql-corpus` ≥80% | check_beta_v310.sh |
| B-S1 | concurrency_stress_test | 9/9 PASS | check_beta_v300.sh |
| B-S2 | crash_recovery_test | 9/9 PASS | check_beta_v300.sh |
| B-S3 | long_run_stability_test | 10/10 PASS | check_beta_v300.sh |
| B-S4 | wal_integration_test | 16/16 PASS | check_beta_v300.sh |
| B-S5 | network_tcp_smoke_test | 8/8 PASS | check_beta_v300.sh |

### 5.2 RC Gate (R1-R12)

| ID | 检查项 | 通过标准 | 脚本 |
|----|--------|---------|------|
| R1 | Release Build | `cargo build --release --workspace` | check_rc_v300.sh |
| R2 | 测试 100% | `cargo test --all-features` 0 failures | check_rc_v300.sh |
| R3 | Clippy | `cargo clippy --all-features -- -D warnings` | check_rc_v300.sh |
| R4 | Format | `cargo fmt --all -- --check` | check_rc_v300.sh |
| R5 | 覆盖率 ≥60% | `cargo llvm-cov --all-features --lib` ≥60% | check_rc_v300.sh |
| R6 | 安全扫描 | `cargo audit` | check_rc_v300.sh |
| R7 | 文档完整性 | v3.1.0 docs 存在 | check_rc_v300.sh |
| R8 | SQL Corpus ≥95% | `cargo test -p sqlrustgo-sql-corpus` ≥95% | check_rc_v300.sh |
| R9 | TPC-H SF=1 | `bash scripts/gate/check_tpch.sh sf=1` 22/22 | check_rc_v300.sh |
| R10 | Performance Baseline | `bash scripts/gate/check_regression.sh` | check_rc_v300.sh |
| R11 | Sysbench Gate | `bash scripts/gate/check_sysbench.sh` | check_rc_v300.sh |
| R12 | Formal Proof | `docs/proof/*.json/*.dfy/*.tla` ≥10 | check_rc_v300.sh |

### 5.3 GA Gate (GA-1~GA-19)

| ID | 检查项 | 通过标准 | 脚本 |
|----|--------|---------|------|
| GA-1 | Release Build | `cargo build --release --workspace` | check_ga_v300.sh |
| GA-2 | 测试 100% | `cargo test --all-features` 0 failures | check_ga_v300.sh |
| GA-3 | Integration tests | `bash scripts/test/run_integration.sh --quick` | check_ga_v300.sh |
| GA-4 | Clippy | `cargo clippy --all-features -- -D warnings` | check_ga_v300.sh |
| GA-5 | Format | `cargo fmt --all -- --check` | check_ga_v300.sh |
| GA-6 | 覆盖率 ≥65% | `cargo llvm-cov --all-features --lib` ≥65% | check_ga_v300.sh |
| GA-7 | 安全扫描 | `cargo audit` | check_ga_v300.sh |
| GA-8 | 文档链接 | `bash scripts/gate/check_docs_links.sh` | check_ga_v300.sh |
| GA-9 | TPC-H SF=1 | `bash scripts/gate/check_tpch.sh sf=1` 22/22 | check_ga_v300.sh |
| GA-10 | Point SELECT QPS ≥10K | `cargo test --release --test qps_benchmark_test test_qps_simple_select -- --ignored --nocapture` | check_ga_v300.sh |
| GA-11 | Formal proofs ≥10 | `docs/proof/*.{json,dfy,tla,formalog,formulog}` ≥10 | check_ga_v300.sh |
| GA-12 | QPS Benchmark | 8/8 benchmarks within 5% of baseline | check_ga_v300.sh |
| GA-12b | B-S1 | concurrency_stress_test PASS | check_ga_v300.sh |
| GA-12c | B-S2 | crash_recovery_test PASS | check_ga_v300.sh |
| GA-12d | B-S3 | long_run_stability_test PASS | check_ga_v300.sh |
| GA-12e | B-S4 | wal_integration_test PASS | check_ga_v300.sh |
| GA-12f | B-S5 | network_tcp_smoke_test PASS | check_ga_v300.sh |
| GA-13 | Release docs | 8 份文档存在 | check_ga_v300.sh |
| GA-14 | SQL Corpus ≥98% | `cargo test -p sqlrustgo-sql-corpus` ≥98% | check_ga_v300.sh |
| GA-15 | Version consistency | cargo=3.1.0 + docs refs | check_ga_v300.sh |
| GA-16 | UPDATE QPS ≥5K | `cargo test --release --test qps_benchmark_test test_qps_update -- --ignored --nocapture` | check_ga_v300.sh |
| GA-17 | DELETE QPS ≥2K | `cargo test --release --test qps_benchmark_test test_qps_delete -- --ignored --nocapture` | check_ga_v300.sh |

---

## 六、资源估算

### 6.1 人力估算

| 功能 | 工作量 | 优先级 | 负责人 |
|------|--------|--------|--------|
| INFORMATION_SCHEMA | 3 周 | P0 | 待分配 |
| SQL Operations ≥80% | 3 周 | P0 | 待分配 |
| TPC-H SF=1 OOM 修复 | 2 周 | P0 | 待分配 |
| 事务压力测试 | 1 周 | P0 | 待分配 |
| MERGE 语句 | 2 周 | P0 | 待分配 |
| Performance Schema | 2 周 | P1 | 待分配 |
| CBO 代价模型 | 2 周 | P1 | 待分配 |
| 全文索引 | 4 周 | P1 | 待分配 |
| 事件调度器 | 3 周 | P1 | 待分配 |
| 聚簇索引设计 | 2 周 | P1 | 待分配 |
| MERGE JOIN | 2 周 | P1 | 待分配 |
| BNL JOIN | 1 周 | P1 | 待分配 |
| ~~窗口函数增强~~ | N/A | ✅ | v3.0.0 已实现 |
| 触发器完善 | 1 周 | P1 | 待分配 |
| SERIALIZABLE 完善 | 1 周 | P1 | 待分配 |
| **总计** | **31 周** (含新增 MERGE/MERGE JOIN/BNL JOIN) | | |

---

## 七、风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| TPC-H SF=1 OOM 根因复杂 | 高 | 中 | 分阶段：先优化 Hash Join，再增加内存限制 |
| CBO 代价模型影响范围广 | 高 | 中 | 设计评审 + AB 测试对比 |
| 全文索引性能不达标 | 高 | 中 | 设计评审 + 基准测试先行 |
| 人力不足导致延期 | 高 | 中 | 优先级排序 + 迭代交付 |
| v3.0.0 遗留问题超出预期 | 中 | 低 | Beta 前缓冲 2 周 |

---

## 八、成功标准

### 8.1 功能标准

- [ ] INFORMATION_SCHEMA 覆盖率 ≥80%
- [ ] SQL Operations 通过率 ≥80%
- [ ] Performance Schema 基础表 ≥10 个
- [ ] TPC-H SF=1 22/22 无 OOM
- [ ] CBO 代价模型接入 planner
- [ ] 全文索引支持中英文分词
- [ ] 事件调度器支持定时任务
- [ ] MERGE 语句完整实现
- [ ] MERGE JOIN 算法实现
- [ ] BNL JOIN 算法实现
- [x] ✅ 窗口函数 6/6 v3.0.0 已全部实现

### 8.2 性能标准

- [ ] Point SELECT QPS ≥ 3,000,000 (较 v3.0.0 提升 ≥5%)
- [ ] UPDATE QPS ≥ 500,000
- [ ] DELETE QPS ≥ 700,000
- [ ] TPC-H SF=1 p99 < 5s
- [ ] 内存占用增加 < 15%

### 8.3 质量标准

- [ ] GA Gate 23/23 PASS
- [ ] Clippy 零警告
- [ ] Formal proofs ≥30 个
- [ ] 综合评分 70/100 (vs MySQL 8.3)

---

*本文档由 hermes agent 创建，基于 v3.0.0 GA 遗留问题分析和版本演进计划。*
*每次 Beta/RC/GA Gate 检查后更新状态。*
