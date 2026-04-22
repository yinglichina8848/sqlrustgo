# v2.7.0 SQL Regression 测试计划

> **版本**: v2.7.0
> **创建日期**: 2026-04-22
> **维护人**: yinglichina8848

---

## 一、概述

SQL Regression Corpus 是用于全面测试 SQL 语法的测试用例集合。v2.7.0 目标是将测试通过率从 90%+ 提升到 98%+，并扩展对高级 SQL 特性的覆盖。

---

## 二、当前状态

### 2.1 v2.6.0 测试结果

| 测试套件 | 用例数 | 通过数 | 通过率 |
|----------|--------|--------|--------|
| sql-corpus | 5123 | 4611 | 90.0% |

### 2.2 失败用例分布

| 类别 | 失败数 | 说明 |
|------|--------|------|
| 窗口函数 | 127 | ROW_NUMBER, RANK, LEAD/LAG |
| CTE (递归) | 89 | 递归 CTE 语法 |
| JSON 函数 | 156 | JSON_EXTRACT, JSON_OBJECT |
| 高级聚合 | 64 | GROUPING SETS, CUBE, ROLLUP |
| 复杂子查询 | 45 | 多层嵌套子查询 |
| 其他 | 31 | 边界情况和边缘语法 |

---

## 三、扩展计划

### 3.1 用例扩展目标

| 阶段 | 目标用例数 | 日期 |
|------|-----------|------|
| v2.7.0 | +3000 条 | 2026-05-18 |
| v2.7.1 | +1500 条 | TBD |
| v2.7.2 | +2000 条 | TBD |

### 3.2 目录结构

```
sql_corpus/
├── DML/
│   ├── SELECT/
│   │   ├── where_conditions.sql
│   │   ├── group_by_having.sql
│   │   ├── order_by.sql
│   │   ├── limit_offset.sql
│   │   ├── distinct_all.sql
│   │   ├── set_operations.sql
│   │   ├── inner_join.sql
│   │   ├── outer_join.sql
│   │   ├── subquery.sql
│   │   ├── correlated_subquery.sql
│   │   ├── cte_operations.sql
│   │   ├── cte_recursive.sql
│   │   ├── aggregate_functions.sql
│   │   └── window_functions.sql
│   ├── INSERT/
│   │   ├── insert_variations.sql
│   │   ├── batch_operations.sql
│   │   └── upsert_operations.sql
│   ├── UPDATE/
│   │   ├── update_variations.sql
│   │   └── update_complex.sql
│   └── DELETE/
│       ├── delete_variations.sql
│       └── delete_complex.sql
├── DDL/
│   ├── CREATE_TABLE/
│   ├── ALTER_TABLE/
│   ├── FOREIGN_KEY/
│   ├── INDEX/
│   └── VIEW/
├── Transactions/
│   ├── transactions.sql
│   └── transaction_isolation.sql
├── EXPRESSIONS/
│   ├── string_functions.sql
│   ├── numeric_functions.sql
│   ├── datetime_functions.sql
│   ├── case_expressions.sql
│   ├── window_functions.sql
│   └── json_functions.sql
└── Special/
    ├── null_semantics.sql
    └── full_text_search.sql
```

---

## 四、SQL-92/99 语法覆盖

### 4.1 窗口函数 (P0)

| 函数 | 用例数 | 状态 |
|------|--------|------|
| ROW_NUMBER | 30 | ⏳ |
| RANK / DENSE_RANK | 30 | ⏳ |
| LEAD / LAG | 25 | ⏳ |
| FIRST_VALUE / LAST_VALUE | 20 | ⏳ |
| PARTITION BY | 25 | ⏳ |
| 帧定义 (ROWS/RANGE) | 20 | ⏳ |

### 4.2 CTE 递归 (P0)

| 类型 | 用例数 | 状态 |
|------|--------|------|
| 简单递归 | 25 | ⏳ |
| 多层递归 | 20 | ⏳ |
| 递归终止条件 | 15 | ⏳ |
| 递归深度限制 | 10 | ⏳ |

### 4.3 高级聚合 (P1)

| 功能 | 用例数 | 状态 |
|------|--------|------|
| GROUPING SETS | 20 | ⏳ |
| CUBE | 15 | ⏳ |
| ROLLUP | 15 | ⏳ |
| GROUPING() 函数 | 10 | ⏳ |

### 4.4 JSON 函数 (P1)

| 函数 | 用例数 | 状态 |
|------|--------|------|
| JSON_EXTRACT | 30 | ⏳ |
| JSON_OBJECT | 25 | ⏳ |
| JSON_ARRAY | 20 | ⏳ |
| JSON_VALID | 15 | ⏳ |
| JSON_KEYS / JSON_VALUES | 20 | ⏳ |

---

## 五、执行测试

### 5.1 运行所有 SQL 测试

```bash
cargo test sql_corpus
```

### 5.2 运行特定类别

```bash
# 窗口函数测试
cargo test sql_corpus -- WINDOW

# CTE 测试
cargo test sql_corpus -- CTE

# 递归 CTE 测试
cargo test sql_corpus -- RECURSIVE

# GROUPING SETS 测试
cargo test sql_corpus -- GROUPING_SETS

# JSON 函数测试
cargo test sql_corpus -- JSON
```

### 5.3 生成测试报告

```bash
# 生成覆盖率报告
cargo tarpaulin --out Html

# 检查通过率
find sql_corpus -name '*.sql' | wc -l  # 用例总数
```

---

## 六、验收标准

| 测试套件 | 目标通过率 | 当前 |
|----------|-----------|------|
| SELECT | ≥98% | 90.0% |
| INSERT | ≥98% | 95.2% |
| UPDATE | ≥98% | 93.8% |
| DELETE | ≥98% | 91.5% |
| JOIN | ≥98% | 96.1% |
| 聚合函数 | ≥98% | 94.3% |
| 窗口函数 | ≥95% | 0.0% |
| CTE | ≥95% | 78.2% |
| JSON | ≥90% | 0.0% |

---

## 七、回归测试策略

### 7.1 自动化回归

- 每次 PR 必须运行 `cargo test sql_corpus`
- 通过率下降超过 2% 则阻止合并

### 7.2 新功能验证

- 新 SQL 语法支持必须附带测试用例
- 测试用例必须覆盖边界条件

### 7.3 性能基准

- 单条 SQL 执行时间 < 100ms (简单查询)
- 复杂查询 < 1s

---

## 八、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-04-17 | 初始版本 v2.6.0 |
| 2.0 | 2026-04-22 | v2.7.0 - 增加窗口函数、递归CTE、JSON函数测试 |
