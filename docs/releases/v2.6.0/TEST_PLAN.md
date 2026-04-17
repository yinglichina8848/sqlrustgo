# v2.6.0 测试计划

> **版本**: v2.6.0
> **创建日期**: 2026-04-17
> **维护人**: yinglichina8848

---

## 一、测试目标

| 指标 | v2.5.0 | v2.6.0 目标 |
|------|---------|--------------|
| 测试覆盖率 | 49% | ≥70% |
| SQL-92 测试通过率 | 20.3% | ≥90% |
| 并发压力测试 | - | 100% 通过 |
| MVCC SSI 测试 | - | 100% 通过 |

---

## 二、测试策略

### 2.1 覆盖率提升计划

| 阶段 | 目标覆盖率 | 周期 | PR |
|------|-----------|------|-----|
| Phase 1 | 49% → 55% | 1-2 周 | - |
| Phase 2 | 55% → 62% | 2-3 周 | - |
| Phase 3 | 62% → 70% | 3-4 周 | - |

### 2.2 SQL Regression 测试

| 测试套件 | 目标用例数 | 目标通过率 |
|----------|-----------|-----------|
| SELECT | 1500+ | ≥90% |
| INSERT | 500+ | ≥90% |
| UPDATE | 500+ | ≥90% |
| DELETE | 500+ | ≥90% |
| JOIN | 500+ | ≥90% |
| 聚合函数 | 500+ | ≥90% |
| 事务 | 200+ | ≥90% |

---

## 三、测试套件

### 3.1 单元测试

```bash
# 运行所有单元测试
cargo test --lib

# 运行特定模块测试
cargo test -p sqlrustgo-planner --lib
cargo test -p sqlrustgo-executor --lib
cargo test -p sqlrustgo-storage --lib
cargo test -p sqlrustgo-parser --lib
```

### 3.2 集成测试

```bash
# 运行所有集成测试
cargo test --test '*'

# 运行特定集成测试
cargo test --test executor_integration
cargo test --test storage_integration
cargo test --test mvcc_integration
```

### 3.3 压力测试

```bash
# 并发压力测试
cargo test --test concurrency_stress_test

# TPC-H 基准测试
cargo test --test tpch_full_test
cargo test --test tpch_sf1_benchmark

# 崩溃恢复测试
cargo test --test crash_recovery_test
```

### 3.4 MVCC SSI 测试

```bash
# MVCC 快照隔离测试
cargo test mvcc_snapshot_isolation

# SSI 可串行化测试
cargo test mvcc_serializable

# MVCC 索引测试
cargo test mvcc_index
```

---

## 四、SQL Regression Corpus

### 4.1 目录结构

```
sql_corpus/
├── DML/
│   ├── SELECT/      # 1500+ 用例
│   ├── INSERT/      # 500+ 用例
│   ├── UPDATE/      # 500+ 用例
│   └── DELETE/      # 500+ 用例
├── DDL/
│   ├── CREATE_TABLE/
│   ├── ALTER_TABLE/
│   ├── FOREIGN_KEY/
│   └── INDEX/
├── Transactions/
│   └── isolation_levels.sql
└── Special/
    └── NULL_semantics.sql
```

### 4.2 执行测试

```bash
# 运行所有 SQL 测试
cargo test sql_corpus

# 运行特定类别
cargo test sql_corpus -- SELECT
cargo test sql_corpus -- JOIN
cargo test sql_corpus -- AGGREGATE
```

---

## 五、性能测试

### 5.1 OLTP 性能测试

```bash
# 点查性能
cargo bench -- point_select

# 索引扫描性能
cargo bench -- index_scan

# 插入性能
cargo bench -- insert
```

### 5.2 TPC-H 性能测试

```bash
# SF=1 完整测试
cargo test --test tpch_full_test -- --sf 1

# SF=10 性能测试
cargo test --test tpch_sf10_benchmark
```

---

## 六、测试时间线

| 阶段 | 日期 | 目标 |
|------|------|------|
| Alpha | 2026-04-21 | P0 功能单元测试通过 |
| Beta | 2026-04-28 | P1 功能集成测试通过 |
| RC | 2026-05-05 | 所有测试通过，覆盖率 ≥70% |
| GA | 2026-05-12 | 性能测试通过，CI 全绿 |

---

## 七、验收标准

### 7.1 P0 验收

```bash
# CBO 优化器测试
cargo test -p sqlrustgo-planner --lib  # 通过

# 存储过程/触发器测试
cargo test -p sqlrustgo-executor --lib  # 通过

# 索引扫描测试
cargo test -p sqlrustgo-storage --lib  # 通过

# MVCC SSI 测试
cargo test mvcc_serializable  # 通过
cargo test mvcc_index  # 通过
```

### 7.2 P1 验收

```bash
# DELETE 语句测试
cargo test sql_corpus -- DELETE  # 90%+ 通过

# FULL OUTER JOIN 测试
cargo test sql_corpus -- FULL_OUTER_JOIN  # 100% 通过

# 外键约束测试
cargo test fk_constraint  # 通过
```

### 7.3 P2 验收

```bash
# CREATE INDEX 测试
cargo test sql_corpus -- CREATE_INDEX  # 90%+ 通过

# 覆盖率
cargo tarpaulin  # ≥ 70%
```

---

## 八、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-04-17 | 初始版本 |
