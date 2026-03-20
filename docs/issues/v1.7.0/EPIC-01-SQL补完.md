# [Epic-01] SQL 补完

## 概述

补齐 SQL-92 核心语法，为 v1.8 MySQL 教学兼容打基础。

**资源占比**: 40%
**优先级**: P0

---

## Issues

### [SQL-01] UNION / UNION ALL 支持

**优先级**: P0
**工作量**: 150 行

**描述**: 实现 UNION 和 UNION ALL 集合运算

**Acceptance Criteria**:
- [ ] `SELECT 1 UNION ALL SELECT 2` 返回 2 行
- [ ] `SELECT 1 UNION SELECT 1` 返回 1 行 (DISTINCT)
- [ ] UNION 和 UNION ALL 语义正确

**Files**:
- `crates/parser/src/parser.rs`
- `crates/planner/src/logical_plan.rs`
- `crates/planner/src/physical_plan.rs`
- `crates/executor/src/executor.rs`

---

### [SQL-02] INTERSECT / EXCEPT

**优先级**: P1
**工作量**: 100 行

**描述**: 实现 INTERSECT 和 EXCEPT 集合运算

**Acceptance Criteria**:
- [ ] `SELECT 1 INTERSECT SELECT 1` 返回 1 行
- [ ] `SELECT 1 EXCEPT SELECT 2` 返回 1 行

---

### [SQL-03] VIEW 创建和查询

**优先级**: P0
**工作量**: 200 行

**描述**: 实现视图的创建、存储和查询重写

**Acceptance Criteria**:
- [ ] `CREATE VIEW v AS SELECT ...` 成功创建视图
- [ ] `SELECT * FROM v` 正确执行视图查询
- [ ] 视图定义存储在 Catalog 中

---

### [SQL-04] BOOLEAN 类型补齐

**优先级**: P0
**工作量**: 50 行

**描述**: 补齐 BOOLEAN 类型支持

**Acceptance Criteria**:
- [ ] `CREATE TABLE t (b BOOLEAN)` 正确解析
- [ ] `INSERT INTO t VALUES (TRUE)` 正确执行
- [ ] `WHERE b = TRUE` 正确过滤

---

### [SQL-05] BLOB 类型支持

**优先级**: P1
**工作量**: 80 行

**描述**: 实现 BLOB 二进制类型支持

**Acceptance Criteria**:
- [ ] `CREATE TABLE t (data BLOB)` 正确解析
- [ ] `INSERT INTO t VALUES (X'0123')` 正确执行
- [ ] BLOB 数据可正确存储和读取

---

## 实现步骤

1. **Parser 扩展**
   - 添加 UNION/INTERSECT/EXCEPT token
   - 添加 SetQuery AST 节点
   - 添加 BOOLEAN/BLOB 类型关键字

2. **Planner 扩展**
   - 添加 Union LogicalPlan
   - 添加视图存储和查询重写
   - 添加类型检查

3. **Executor 扩展**
   - 实现 UnionExec
   - 实现 BOOLEAN/BLOB 读写

---

## 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| 视图查询重写复杂度 | 中 | 先实现简单视图，再扩展 |
| 类型系统扩展 | 低 | 复用现有类型基础设施 |

---

**关联 Issue**: SQL-01, SQL-02, SQL-03, SQL-04, SQL-05
**总工作量**: ~580 行