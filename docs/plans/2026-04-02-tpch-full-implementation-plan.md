# TPC-H Q1-Q22 完整实现计划

**日期**: 2026-04-02
**状态**: 第一阶段准备开始
**目标**: 实现完整的 TPC-H Q1-Q22 标准查询测试，与 SQLite、MySQL、PostgreSQL 横向对比

---

## 背景

SQLRustGo 目前有简化的 TPC-H 测试，但使用的是 stub 查询而非 TPC-H 标准定义。目标是让 SQLRustGo 真实执行完整的 TPC-H Q1-Q22 标准查询。

---

## 总体实现方案

按 TPC-H 查询依赖顺序分三阶段实现：

| 阶段 | 特性 | 工期 | 启用查询数 | 累计 |
|------|------|------|-----------|------|
| Phase 1 | BETWEEN, DATE, IN value list | 4-7 天 | 15 | 15/22 |
| Phase 2 | COUNT(DISTINCT), CASE | 5-8 天 | +3 | 18/22 |
| Phase 3 | EXTRACT, SUBSTRING | 5-7 天 | +4 | 22/22 |

---

## 第一阶段详细设计

### 1.1 BETWEEN 操作符

**TPC-H 需求**:
- Q6: `l_discount BETWEEN 0.06 - 0.01 AND 0.06 + 0.01`
- Q19: `l_quantity BETWEEN 1 AND 11`

**实现方案**:
- Parser: 在 `parse_comparison_expression()` 中添加 BETWEEN 处理
- 转换: `expr BETWEEN a AND b` → `expr >= a AND expr <= b`
- Token `Token::Between` 已存在于 `token.rs`

**修改文件**: `crates/parser/src/parser.rs`

---

### 1.2 DATE 类型支持

**TPC-H 需求**:
- Q1: `WHERE l_shipdate <= DATE '1998-12-01'`
- Q3: `WHERE o_orderdate < '2024-03-01'`
- Q6: `l_shipdate >= DATE '1994-01-01'`

**实现方案**:
- Parser: 识别 `DATE 'yyyy-mm-dd'` 语法
- Storage: TPC-H schema 使用 TEXT 存储日期
- 比较: ISO 格式字符串比较正常工作

**修改文件**: `crates/parser/src/parser.rs`

---

### 1.3 IN value list

**TPC-H 需求**:
- Q12: `l_shipmode IN ('MAIL', 'SHIP')`
- Q16: `p_size IN (49, 14, 23, 45, 19, 3, 36, 9)`
- Q22: `SUBSTRING(c_phone FROM 1 FOR 2) IN ('13', '31', ...)`

**实现方案**:
- Parser: 检测 `IN (literal, literal, ...)` 模式
- 新增 `Expr::InList { expr, values }` 变体
- Executor: 实现 `values.contains()`

**修改文件**:
- `crates/parser/src/parser.rs`
- `crates/planner/src/lib.rs`
- `crates/executor/src/executor.rs`

---

## 第一阶段完成后可运行的查询

| 查询 | 状态 | 说明 |
|------|------|------|
| Q3 | ✅ | JOIN + GROUP BY + ORDER BY + LIMIT |
| Q4 | ✅ | EXISTS 子查询 |
| Q5 | ✅ | 多表 JOIN + GROUP BY |
| Q6 | 🔧 | 需 BETWEEN + DATE |
| Q7 | ✅ | JOIN + GROUP BY |
| Q10 | ✅ | JOIN + GROUP BY + ORDER BY |
| Q11 | ✅ | GROUP BY + HAVING + 子查询 |
| Q12 | 🔧 | 需 IN list + BETWEEN |
| Q13 | ✅ | GROUP BY + 子查询 |
| Q14 | 🔧 | 需 CASE |
| Q15 | ✅ | JOIN + GROUP BY |
| Q17 | ✅ | JOIN + AVG |
| Q18 | ✅ | JOIN + GROUP BY + HAVING |
| Q20 | ✅ | JOIN + ORDER BY |
| Q21 | ✅ | JOIN + GROUP BY + 子查询 |

---

## 第二阶段 (待定)

### 2.1 COUNT(DISTINCT)
- Q16: `COUNT(DISTINCT ps_suppkey)`
- 实现: HashSet 去重聚合

### 2.2 CASE 表达式
- Q1, Q14, Q19 需要
- 需要 Parser + Planner + Executor 支持

---

## 第三阶段 (待定)

### 3.1 EXTRACT 函数
- Q1, Q8, Q9 需要: `EXTRACT(year FROM date)`

### 3.2 SUBSTRING 函数
- Q2, Q22 需要: `SUBSTRING(col FROM 1 FOR 2)`

---

## 风险和依赖

| 风险 | 级别 | 缓解 |
|------|------|------|
| CASE 表达式复杂度高 | 中 | 第二阶段实现，有充足测试 |
| 日期处理跨存储类型 | 中 | 使用 ISO 字符串比较 |
| Parser 修改影响现有功能 | 低 | 增量修改，充分测试 |

---

## 测试策略

```
阶段测试:
├── cargo test --test tpch_test
├── cargo test --test tpch_benchmark  
└── cargo test --test tpch_full_test

数据库对比:
├── SQLite: 结果一致性验证
├── PostgreSQL: 结果一致性验证
└── MySQL: 可用时验证
```
