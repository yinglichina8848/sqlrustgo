# TPC-H Q1-Q22 完整开发计划

**日期**: 2026-04-02  
**版本**: v2.1.0  
**目标**: 实现完整的 TPC-H Q1-Q22 标准查询测试

---

## 概述

SQLRustGo 需要实现完整的 TPC-H Q1-Q22 标准查询。当前 Phase 1 已完成 BETWEEN、DATE、IN value list 支持。还需要 Phase 2 和 Phase 3 才能执行完整的 TPC-H 标准 SQL。

---

## 当前状态

| 阶段 | 功能 | 状态 | 启用查询数 |
|------|------|------|-----------|
| Phase 1 | BETWEEN, DATE, IN value list | ✅ 已完成 | 15 |
| Phase 2 | COUNT(DISTINCT), CASE WHEN | ❌ 待实现 | +3 |
| Phase 3 | EXTRACT, SUBSTRING | ❌ 待实现 | +4 |

---

## TPC-H Q1-Q22 完整语法需求分析

### Q1-Q11 详细需求

| Query | 需要的语法 | 阶段 |
|--------|----------|------|
| Q1 | CASE WHEN (聚合中), DATE | Phase 2 |
| Q2 | SUBSTRING (PARTITION BY), ORDER BY, LIMIT | Phase 3 |
| Q3 | SUBSTRING (日期提取), DATE 比较 | Phase 3 |
| Q4 | EXISTS, DATE 比较 | ✅ 已支持 |
| Q5 | DATE 比较, JOIN + GROUP BY | ✅ 已支持 |
| Q6 | BETWEEN, DATE, CASE | Phase 2 |
| Q7 | SUBSTRING, CASE, INNER JOIN | Phase 2, 3 |
| Q8 | CASE WHEN, EXTRACT, SUBSTRING | Phase 2, 3 |
| Q9 | EXTRACT, SUBSTRING, CASE, LIKE | Phase 2, 3 |
| Q10 | EXISTS, DATE 比较 | ✅ 已支持 |
| Q11 | HAVING 子查询 | ✅ 已支持 |

### Q12-Q22 详细需求

| Query | 需要的语法 | 阶段 |
|--------|----------|------|
| Q12 | CASE WHEN, IN, BETWEEN | Phase 2 |
| Q13 | LEFT OUTER JOIN, LIKE | ✅ 部分支持 |
| Q14 | CASE WHEN, LIKE, DATE | Phase 2 |
| Q15 | 子查询, ORDER BY, LIMIT | ✅ 部分支持 |
| Q16 | COUNT(DISTINCT), IN, NOT IN, LIKE | Phase 2 |
| Q17 | 子查询, CASE, AVG | Phase 2 |
| Q18 | HAVING, 子查询, ORDER BY, LIMIT | ✅ 部分支持 |
| Q19 | CASE WHEN, IN, BETWEEN, LIKE | Phase 2 |
| Q20 | 子查询, DISTINCT, LIKE | ✅ 部分支持 |
| Q21 | EXISTS, NOT EXISTS, 子查询 | ✅ 已支持 |
| Q22 | SUBSTRING, CASE WHEN, IN, NOT EXISTS, AVG | Phase 2, 3 |

---

## Phase 2: CASE WHEN 和 COUNT(DISTINCT)

### 2.1 CASE WHEN 表达式

**TPC-H 需求**:
- Q1: `CASE WHEN ... THEN ... ELSE ... END`
- Q8: `CASE WHEN nation = 'CHINA' THEN volume ELSE 0 END`
- Q12: `CASE WHEN o_orderpriority = '1-URGENT' THEN 1 ELSE 0 END`
- Q14: `CASE WHEN p_type LIKE 'PROMO%' THEN ... ELSE 0 END`

**实现方案**:
```
Parser: 解析 CASE WHEN ... THEN ... ELSE ... END
Planner: 转换为 Expr::CaseWhen 结构
Executor: 计算条件分支
```

**修改文件**:
- `crates/parser/src/parser.rs` - 添加 CASE 解析
- `crates/planner/src/lib.rs` - 添加 Expr::CaseWhen
- `crates/executor/src/executor.rs` - 实现 CASE 执行逻辑

### 2.2 COUNT(DISTINCT)

**TPC-H 需求**:
- Q16: `COUNT(DISTINCT ps_suppkey)`

**实现方案**:
```
聚合函数: COUNT(DISTINCT expr) 使用 HashSet 去重
```

**修改文件**:
- `crates/planner/src/lib.rs` - AggregateFunction 支持 distinct
- `crates/executor/src/executor.rs` - 实现 HashSet 去重

---

## Phase 3: EXTRACT 和 SUBSTRING

### 3.1 EXTRACT 函数

**TPC-H 需求**:
- Q8: `EXTRACT(YEAR FROM o_orderdate)`
- Q9: `EXTRACT(YEAR FROM o_orderdate)`

**实现方案**:
```
Parser: 解析 EXTRACT(year FROM date)
Planner: 转换为 Expr::Extract { field, date }
Executor: 提取日期字段
```

**修改文件**:
- `crates/parser/src/parser.rs` - 添加 EXTRACT 解析
- `crates/planner/src/lib.rs` - 添加 Expr::Extract
- `crates/executor/src/functions.rs` - 实现 extract_date 函数

### 3.2 SUBSTRING 函数

**TPC-H 需求**:
- Q2: `SUBSTRING(p_name FROM 1 FOR 5)`
- Q7: `SUBSTRING(l_shipdate, 1, 4)` (MySQL 语法)
- Q22: `SUBSTRING(c_phone, 1, 2)`

**实现方案**:
```
Parser: 解析 SUBSTRING(str FROM start FOR len) 或 SUBSTRING(str, start, len)
Planner: 转换为 Expr::Substring { expr, start, length }
Executor: 实现字符串切片
```

**修改文件**:
- `crates/parser/src/parser.rs` - 添加 SUBSTRING 解析
- `crates/planner/src/lib.rs` - 添加 Expr::Substring
- `crates/executor/src/functions.rs` - 实现 substring 函数

---

## 数据库对比测试

### SQLite
- 内置支持，无需外部服务器
- 用于本地性能对比基准

### PostgreSQL
- 支持完整的 TPC-H Q1-Q22
- 通过 `PGHOST`, `PGUSER`, `PGPASSWORD` 环境变量配置
- 用于与 SQLRustGo 结果对比

### MySQL
- 支持完整的 TPC-H Q1-Q22
- 通过 `MYSQL_HOST`, `MYSQL_USER`, `MYSQL_PASSWORD` 环境变量配置
- 用于多数据库验证

---

## 测试文件结构

```
tests/integration/
├── tpch_test.rs          # SQLRustGo 基础测试 + BETWEEN/IN
├── tpch_full_test.rs     # Q1-Q22 完整测试 + 数据库对比
└── tpch_benchmark.rs     # 性能基准测试

测试数据:
- 8 张 TPC-H 表 (nation, region, part, supplier, partsupp, customer, orders, lineitem)
- 每表 3-10 条测试数据
- 用于验证查询语法正确性
```

---

## 开发任务清单

### Phase 2: CASE WHEN 和 COUNT(DISTINCT)

- [ ] Task 1: 分析现有 CASE 相关代码
- [ ] Task 2: 在 Parser 添加 CASE WHEN 解析
- [ ] Task 3: 在 Planner 添加 Expr::CaseWhen
- [ ] Task 4: 在 Executor 实现 CASE 执行
- [ ] Task 5: 在 Parser 添加 COUNT(DISTINCT) 解析
- [ ] Task 6: 在 Planner 实现 distinct 聚合
- [ ] Task 7: 编写测试验证
- [ ] Task 8: 更新 tpch_full_test.rs 使用完整 SQL

### Phase 3: EXTRACT 和 SUBSTRING

- [ ] Task 9: 分析现有函数解析逻辑
- [ ] Task 10: 在 Parser 添加 EXTRACT 解析
- [ ] Task 11: 在 Planner 添加 Expr::Extract
- [ ] Task 12: 在 Executor 实现 extract_date 函数
- [ ] Task 13: 在 Parser 添加 SUBSTRING 解析
- [ ] Task 14: 在 Planner 添加 Expr::Substring
- [ ] Task 15: 在 Executor 实现 substring 函数
- [ ] Task 16: 编写测试验证
- [ ] Task 17: 更新 tpch_full_test.rs 使用完整 SQL

---

## 预计工期

| 阶段 | 任务 | 工期 |
|------|------|------|
| Phase 2 | CASE WHEN | 3-4 天 |
| Phase 2 | COUNT(DISTINCT) | 2-3 天 |
| Phase 3 | EXTRACT | 2-3 天 |
| Phase 3 | SUBSTRING | 3-4 天 |
| 测试 | Q1-Q22 完整验证 | 2-3 天 |
| **总计** | | **12-17 天** |

---

## 风险和依赖

| 风险 | 级别 | 缓解 |
|------|------|------|
| CASE 表达式复杂度高 | 中 | 分阶段实现，充分测试 |
| SUBSTRING 语法多样性 | 中 | 先支持 SQL 标准语法 SUBSTRING(str FROM start FOR len) |
| Parser 修改影响现有功能 | 低 | 增量修改，每次修改后运行测试 |
| 日期/字符串函数性能 | 低 | 后续优化 |

---

## 下一步

1. 实现 Phase 2: CASE WHEN 和 COUNT(DISTINCT)
2. 实现 Phase 3: EXTRACT 和 SUBSTRING
3. 更新测试使用完整的 TPC-H Q1-Q22 SQL
4. 运行与 PostgreSQL、MySQL 的结果对比验证
