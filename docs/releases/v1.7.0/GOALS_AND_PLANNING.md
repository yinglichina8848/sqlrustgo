# SQLRustGo v1.7.0 开发规划

> **版本**: v1.7.0
> **目标**: 教学数据库核心能力
> **创建日期**: 2026-03-20

---

## 一、版本概述

v1.7.0 专注于打造"可解释数据库"核心能力，这是教学场景的核心亮点。

### 核心特性

- **SQL 补完**: UNION/VIEW/类型支持
- **可观测性**: EXPLAIN/ANALYZE
- **Benchmark 完善**: 完整基准测试
- **错误系统**: 统一错误处理

---

## 二、Epic 规划

### Epic-01: SQL 补完 (40%)

| Issue | 标题 | 优先级 | 工作量 |
|-------|------|--------|--------|
| SQL-01 | UNION / UNION ALL 支持 | P0 | 150 行 |
| SQL-02 | INTERSECT / EXCEPT | P1 | 100 行 |
| SQL-03 | VIEW 创建和查询 | P0 | 200 行 |
| SQL-04 | BOOLEAN 类型补齐 | P0 | 50 行 |
| SQL-05 | BLOB 类型支持 | P1 | 80 行 |

### Epic-02: 可观测性增强 (30%)

| Issue | 标题 | 优先级 | 工作量 |
|-------|------|--------|--------|
| OBS-01 | EXPLAIN 执行计划 | P0 | 200 行 |
| OBS-02 | EXPLAIN ANALYZE | P0 | 250 行 |
| OBS-03 | 算子级 Profiling | P1 | 150 行 |
| OBS-04 | 格式化输出 | P0 | 50 行 |

### Epic-03: Benchmark 完善 (15%)

| Issue | 标题 | 优先级 | 工作量 |
|-------|------|--------|--------|
| BM-01 | 完整 TPC-H 支持 | P0 | 300 行 |
| BM-02 | 性能回归检测 | P1 | 150 行 |
| BM-03 | 报告生成自动化 | P1 | 100 行 |

### Epic-04: 错误系统 (15%)

| Issue | 标题 | 优先级 | 工作量 |
|-------|------|--------|--------|
| ERR-01 | 统一错误码 | P0 | 100 行 |
| ERR-02 | 错误消息国际化 | P1 | 80 行 |
| ERR-03 | 错误恢复机制 | P1 | 120 行 |

---

## 三、资源分配

| Epic | 资源占比 | 优先级 |
|------|----------|--------|
| Epic-01 SQL 补完 | 40% | P0 |
| Epic-02 可观测性 | 30% | P0 |
| Epic-03 Benchmark | 15% | P1 |
| Epic-04 错误系统 | 15% | P1 |

---

## 四、验收标准

### Epic-01: SQL 补完

- [ ] `SELECT 1 UNION ALL SELECT 2` 返回 2 行
- [ ] `CREATE VIEW v AS SELECT ...` 成功创建视图
- [ ] `CREATE TABLE t (b BOOLEAN)` 正确解析

### Epic-02: 可观测性

- [ ] `EXPLAIN SELECT * FROM orders` 输出执行计划树
- [ ] `EXPLAIN ANALYZE` 输出每个算子耗时和行数

### Epic-03: Benchmark

- [ ] TPC-H 全部 22 个查询可执行
- [ ] 性能回归检测 CI 集成

### Epic-04: 错误系统

- [ ] 统一错误码体系
- [ ] 错误消息可追溯

---

## 五、开发里程碑

| 日期 | 里程碑 |
|------|--------|
| TBD | Alpha 发布 |
| TBD | Beta 发布 |
| TBD | GA 发布 |

---

## 六、相关链接

- Issue #701: [Epic-01] SQL 补完
- Issue #702: [Epic-02] 可观测性增强
- Issue #703: [Epic-03] Benchmark 完善
- Issue #704: [Epic-04] 错误系统

---

*创建日期: 2026-03-20*
