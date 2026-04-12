# SQLRustGo v1.7.0 发布计划

> **版本**: v1.7.0
> **代号**: SQL + 可观测性补完版
> **发布日期**: 2026-04-25
> **前置版本**: v1.6.1 (GA)
> **战略定位**: 教学数据库产品（Teaching DBMS）

---

## 一、版本目标

### 1.1 核心定位

v1.7.0 是 **SQL 补完 + 可观测性增强版**：

- 补齐 SQL-92 核心语法（UNION, VIEW, BOOLEAN, BLOB）
- 打造"可解释数据库"核心能力（EXPLAIN/EXPLAIN ANALYZE）
- 为 v1.8 MySQL 教学兼容打基础

### 1.2 版本路线（Teaching DBMS 战略）

```
v1.6 ✅    →    v1.7 (当前)    →    v1.8     →    v1.9     →    v2.0
Benchmark        SQL+可观测         MySQL兼容     稳定落地     高性能分析
 修复            补完            ⭐核心         ⭐⭐⭐
```

| 版本 | 代号 | 目标 |
|------|------|------|
| v1.6 | Benchmark 修复 | ✅ 已完成 |
| **v1.7** | **SQL+可观测补完** | **能讲清执行过程** |
| v1.8 | MySQL 教学兼容 | 可替代 MySQL 教学 |
| v1.9 | 稳定+教学落地 | 正式课程使用 |
| v2.0 | 高性能分析 | 超越 MySQL |

### 1.3 核心 Epic

| Epic | 名称 | 资源占比 | 优先级 | 核心目标 |
|------|------|----------|--------|----------|
| Epic-01 | SQL 补完 | 40% | P0 | UNION, VIEW, BOOLEAN, BLOB |
| Epic-02 | 可观测性 | 30% | P0 | EXPLAIN, EXPLAIN ANALYZE |
| Epic-03 | Benchmark 完善 | 20% | P1 | CLI 工具, JSON 输出 |
| Epic-04 | 错误系统 | 10% | P2 | MySQL 风格错误 |

---

## 二、Epic 详细规划

### Epic-01: SQL 补完 (40%)

**核心 Issue**:

| Issue | 标题 | 优先级 | 工作量 |
|-------|------|--------|--------|
| SQL-01 | UNION / UNION ALL 支持 | P0 | 150 行 |
| SQL-02 | INTERSECT / EXCEPT | P1 | 100 行 |
| SQL-03 | VIEW 创建和查询 | P0 | 200 行 |
| SQL-04 | BOOLEAN 类型补齐 | P0 | 50 行 |
| SQL-05 | BLOB 类型支持 | P1 | 80 行 |

**验收标准**:
- [ ] UNION/UNION ALL 查询正确
- [ ] VIEW 可创建和查询
- [ ] BOOLEAN 类型可用

---

### Epic-02: 可观测性 (30%) - 核心亮点

**核心 Issue**:

| Issue | 标题 | 优先级 | 工作量 |
|-------|------|--------|--------|
| OBS-01 | EXPLAIN 执行计划 | P0 | 200 行 |
| OBS-02 | EXPLAIN ANALYZE (耗时+行数) | P0 | 250 行 |
| OBS-03 | 算子级 Profiling | P1 | 150 行 |
| OBS-04 | 格式化输出 | P0 | 50 行 |

**EXPLAIN ANALYZE 输出示例**:

```sql
EXPLAIN ANALYZE SELECT * FROM orders WHERE o_orderdate >= '1993-10-01';

Output:
HashJoin (cost=1234.56 rows=1000)
├── SeqScan on orders (cost=123.45 rows=5000)
│   └── Filter: o_orderdate >= '1993-10-01'
└── SeqScan on customer (cost=100.00 rows=1000)
Actual: 15ms, 1000 rows
```

**验收标准**:
- [ ] EXPLAIN 输出执行计划树
- [ ] EXPLAIN ANALYZE 输出每个算子耗时
- [ ] 行数统计准确

---

### Epic-03: Benchmark 完善 (20%)

**核心 Issue**:

| Issue | 标题 | 优先级 | 工作量 |
|-------|------|--------|--------|
| BEN-01 | Benchmark CLI 完善 | P1 | 100 行 |
| BEN-02 | JSON 输出格式 | P0 | 50 行 |
| BEN-03 | PostgreSQL 对比测试 | P1 | 100 行 |

---

### Epic-04: 错误系统 (10%)

**核心 Issue**:

| Issue | 标题 | 优先级 | 工作量 |
|-------|------|--------|--------|
| ERR-01 | Unknown column 错误 | P1 | 30 行 |
| ERR-02 | Table not found 错误 | P1 | 30 行 |
| ERR-03 | Duplicate key 错误 | P2 | 40 行 |

**MySQL 风格错误示例**:

```
ERROR 1054 (42S22): Unknown column 'xxx' in 'field list'
ERROR 1146 (42S02): Table 'xxx' doesn't exist
ERROR 1062 (23000): Duplicate entry 'xxx' for key 'yyy'
```

---

## 三、里程碑

### 3.1 时间线

```
v1.7.0 开发计划
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Phase 1: SQL 补完 (03/21-04/07)
├── SQL-01 UNION 支持
├── SQL-03 VIEW
├── SQL-04 BOOLEAN
└── 里程碑: Beta 发布 (04/01)

Phase 2: 可观测性 + Benchmark (04/08-04/14)
├── OBS-01 EXPLAIN
├── OBS-02 EXPLAIN ANALYZE
├── BEN-02 JSON 输出
└── 里程碑: RC 发布 (04/15)

Phase 3: 收尾 & GA (04/15-04/25)
├── 错误系统完善
├── 文档
├── 测试
└── 里程碑: GA 发布 (04/25)
```

### 3.2 分支结构

```
main
├── release/v1.7.0          # GA 发布分支
├── rc/v1.7.0              # RC 候选分支
├── beta/v1.7.0            # Beta 分支
└── develop/v1.7.0         # 开发主分支
```

---

## 四、门禁检查清单

### 4.1 代码质量门禁

| 检查项 | 验收标准 |
|--------|----------|
| 编译 | 无错误 |
| 测试 | 100% 通过 |
| Clippy | 无警告 |
| 格式化 | 无错误 |

### 4.2 覆盖率门禁

| 阶段 | 目标覆盖率 |
|------|-----------|
| Alpha | ≥50% |
| Beta | ≥60% |
| RC | ≥75% |
| GA | ≥80% |

### 4.3 功能门禁

| # | 检查项 | 说明 |
|---|--------|------|
| 1 | UNION 支持 | UNION/UNION ALL 正确执行 |
| 2 | VIEW | 可创建和查询 VIEW |
| 3 | EXPLAIN | 输出执行计划树 |
| 4 | EXPLAIN ANALYZE | 输出算子耗时和行数 |
| 5 | JSON 输出 | Benchmark 结果 JSON 格式 |
| 6 | MySQL 风格错误 | 错误信息兼容 |

---

## 五、版本定位

### 5.1 v1.7 定位

```
✅ 能讲清数据库执行过程的版本

EXPLAIN ANALYZE 是核心亮点：
- 每个算子耗时可见
- 行数统计准确
- 执行计划树清晰

❌ 还不能替代 MySQL 教学
```

### 5.2 后续版本目标

| 版本 | 目标 |
|------|------|
| v1.8 | 可替代 MySQL 教学 (FOREIGN KEY, MySQL 语法, CLI, MySQL 协议) |
| v1.9 | 正式课程使用 (稳定性, ≥85% 覆盖, 教学文档) |
| v2.0 | 超越 MySQL (向量化, 列存, 高性能分析) |

---

## 六、相关文档

| 文档 | 说明 |
|------|------|
| `docs/releases/VERSION_ROADMAP.md` | 版本演化总览 |
| `docs/plans/2026-03-19-T-05-deadlock-detection-design.md` | 死锁检测设计 |
| `docs/plans/2026-03-19-T-06-savepoint-design.md` | SAVEPOINT 设计 |

---

*本文档由 AI 辅助分析生成*
*生成日期: 2026-03-21*
*版本: v1.7.0 Draft*
*战略定位: 教学数据库产品（Teaching DBMS）*