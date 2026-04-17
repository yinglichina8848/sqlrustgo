# v2.6.0 SQL Regression 测试计划

> **版本**: v2.6.0
> **创建日期**: 2026-04-17
> **维护人**: yinglichina8848

---

## 一、概述

SQL Regression Corpus 是用于全面测试 SQL 语法的测试用例集合。v2.6.0 目标是将测试通过率从 20.3% 提升到 90%+。

---

## 二、当前状态

### 2.1 v2.5.0 测试结果

| 测试套件 | 用例数 | 通过数 | 通过率 |
|----------|--------|--------|--------|
| sql-corpus | 59 | 12 | 20.3% |

### 2.2 失败用例分布

| 类别 | 失败数 | 说明 |
|------|--------|------|
| 聚合函数 | 11 | COUNT, SUM, AVG, MIN, MAX |
| JOIN 语法 | 14 | INNER/LEFT/RIGHT/FULL JOIN |
| GROUP BY/HAVING | 8 | 分组和过滤 |
| DELETE 语句 | 4 | DELETE 语法 |
| CREATE INDEX | 3 | 索引创建 |
| COUNT(DISTINCT) | 1 | 去重计数 |

---

## 三、扩展计划

### 3.1 用例扩展目标

| 阶段 | 目标用例数 | 日期 |
|------|-----------|------|
| v2.6.0 | +5000 条 | 2026-05-12 |
| v2.6.1 | +2000 条 | TBD |
| v2.6.2 | +5000 条 | TBD |

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
│   │   └── aggregate_functions.sql
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
│   └── window_functions.sql
└── Special/
    ├── null_semantics.sql
    └── full_text_search.sql
```

---

## 四、SQL-92 语法覆盖

### 4.1 聚合函数 (P0)

| 函数 | 用例数 | 状态 |
|------|--------|------|
| COUNT(*) | 20 | ⏳ |
| COUNT(column) | 20 | ⏳ |
| COUNT(DISTINCT) | 10 | ⏳ |
| SUM | 15 | ⏳ |
| AVG | 15 | ⏳ |
| MIN/MAX | 10 | ⏳ |

### 4.2 JOIN 语法 (P0)

| 类型 | 用例数 | 状态 |
|------|--------|------|
| INNER JOIN | 30 | ⏳ |
| LEFT JOIN | 30 | ⏳ |
| RIGHT JOIN | 20 | ⏳ |
| FULL OUTER JOIN | 20 | ⏳ |
| CROSS JOIN | 10 | ⏳ |
| Self JOIN | 10 | ⏳ |

### 4.3 GROUP BY / HAVING (P0)

| 用例 | 用例数 | 状态 |
|------|--------|------|
| GROUP BY 单列 | 20 | ⏳ |
| GROUP BY 多列 | 15 | ⏳ |
| HAVING 过滤 | 15 | ⏳ |
| GROUP BY + ORDER BY | 10 | ⏳ |

---

## 五、执行测试

### 5.1 运行所有 SQL 测试

```bash
cargo test sql_corpus
```

### 5.2 运行特定类别

```bash
# 聚合函数测试
cargo test sql_corpus -- AGGREGATE

# JOIN 测试
cargo test sql_corpus -- JOIN

# GROUP BY 测试
cargo test sql_corpus -- GROUP_BY
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
| SELECT | ≥90% | 20.3% |
| INSERT | ≥90% | - |
| UPDATE | ≥90% | - |
| DELETE | ≥90% | - |
| JOIN | ≥90% | - |
| 聚合函数 | ≥90% | - |

---

## 七、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-04-17 | 初始版本 |
