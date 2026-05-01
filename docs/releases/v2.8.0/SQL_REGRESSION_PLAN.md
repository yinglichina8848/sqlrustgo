# SQL Regression Test Plan v2.8.0

> **版本**: v2.8.0 (GA)
> **创建日期**: 2026-05-02
> **基于**: sql-corpus 实际测试结果 (174/426 PASS, 40.8%)
> **参考**: v2.7.0 版本回归基线

---

## 一、概述

本文档定义 SQLRustGo v2.8.0 的 SQL 回归测试计划。测试框架基于 `sqlrustgo-sql-corpus` crate，通过解析和执行 SQL 文件来验证 SQL 解析与执行正确性。

### 1.1 测试范围

- **测试框架**: `crates/sql-corpus` (SimpleExecutor + MemoryStorage)
- **SQL 语料库**: `sql_corpus/` 目录 (103 个 .sql 文件, 426 个测试用例)
- **测试命令**: `cargo test -p sqlrustgo-sql-corpus --all-features`
- **v2.8.0 实际通过率**: **40.8%** (174 PASS / 252 FAIL)

### 1.2 v2.7.0 回归基线对比

| 指标 | v2.7.0 (参考) | v2.8.0 (实际) | 变化 |
|------|---------------|---------------|------|
| 总用例数 | ~426 | 426 | 持平 |
| 通过数 | - | 174 | - |
| 通过率 | - | **40.8%** | - |
| 失败用例 | - | 252 | - |

> 注：v2.7.0 未完整运行 SQL Corpus 回归（v2.7.0 TEST_PLAN 中标记为 ⏳ 待执行）。v2.8.0 首次建立可量化的回归基线。

---

## 二、SQL 语料库架构

```
sql_corpus/
├── ADVANCED/
│   ├── AGGREGATES/        -- 聚合函数 (SUM/AVG/COUNT/MIN/MAX)
│   ├── GROUP_BY/          -- GROUP BY / HAVING
│   ├── JOINS/             -- JOIN 语句 (INNER/LEFT/RIGHT)
│   ├── LIMIT/             -- LIMIT / OFFSET
│   ├── ORDER_BY/          -- ORDER BY
│   ├── SUBQUERIES/        -- 子查询
│   └── UNION/             -- UNION / UNION ALL
├── DDL/
│   ├── ALTER_TABLE/       -- ALTER TABLE
│   ├── CONSTRAINT/        -- 约束操作
│   ├── CREATE_TABLE/      -- CREATE TABLE
│   ├── DROP_TABLE/        -- DROP TABLE
│   ├── FOREIGN_KEY/       -- 外键约束
│   ├── INDEX/             -- 索引操作
│   ├── METADATA/          -- 元数据查询
│   ├── MISC/              -- 杂项 DDL
│   ├── TEMPORARY/         -- 临时表
│   ├── TRIGGER/           -- 触发器
│   └── VIEW/              -- 视图
├── DML/
│   ├── DELETE/            -- DELETE 语句
│   ├── INSERT/            -- INSERT/REPLACE/UPSERT
│   ├── SELECT/            -- SELECT (基础/聚合/JOIN/子查询/CTE/集合操作)
│   └── UPDATE/            -- UPDATE 语句
├── EVENTS/                -- 事件调度
├── EXPRESSIONS/
│   ├── case_expressions.sql        -- CASE WHEN
│   ├── conditional_expressions.sql -- 条件表达式
│   ├── data_types.sql              -- 数据类型
│   ├── datetime_functions.sql      -- 日期时间函数
│   ├── expression_evaluation.sql   -- 表达式求值
│   ├── json_functions.sql          -- JSON 函数
│   ├── logical_operators.sql       -- 逻辑运算符
│   ├── math_functions.sql          -- 数学函数
│   ├── numeric_functions.sql       -- 数值函数
│   ├── string_functions.sql        -- 字符串函数
│   ├── type_conversion.sql         -- 类型转换
│   ├── window_functions.sql        -- 窗口函数 (已 SKIP)
│   └── window_functions_advanced.sql -- 高级窗口函数 (已 SKIP)
├── FUNCTIONS/
│   ├── CONDITIONALS/      -- 条件函数 (COALESCE/NULLIF/IFNULL)
│   ├── DATE_TIME/         -- 日期时间函数
│   ├── NUMERIC/           -- 数值函数
│   └── STRING/            -- 字符串函数
├── INDEXES/               -- 索引语句
├── PROCEDURES/            -- 存储过程
├── SPECIAL/               -- 特殊语义 (NULL/全文搜索)
├── TCL/                   -- 事务控制语言
├── TRANSACTION/           -- 事务测试
├── TRIGGERS/              -- 触发器语句
└── VIEWS/                 -- 视图语句
```

---

## 三、测试用例分类与当前状态

### 3.1 已通过 (PASS) — 174 用例

以下类别在 v2.8.0 中完全通过或基本通过：

| 类别 | 状态 | 说明 |
|------|------|------|
| **SELECT 基础查询** | ✅ 通过 | WHERE、ORDER BY、LIMIT、DISTINCT |
| **INSERT** | ✅ 通过 | 单行/多行/批量插入 |
| **UPDATE** | ✅ 通过 | 单表/条件更新 |
| **DELETE** | ✅ 通过 | 单表/条件删除 |
| **DDL (CREATE/DROP/ALTER)** | ✅ 通过 | 表创建/删除/修改 |
| **聚合函数 (COUNT/SUM/AVG/MIN/MAX)** | ✅ 通过 | 基本聚合 |
| **JOIN (INNER/LEFT/RIGHT)** | ✅ 通过 | 基本 JOIN 操作 |
| **WHERE 条件** | ✅ 通过 | 比较/AND/OR/NOT/NULL |
| **ORDER BY** | ✅ 通过 | ASC/DESC 排序 |
| **LIMIT/OFFSET** | ✅ 通过 | 分页查询 |
| **基本表达式** | ✅ 通过 | 算术/比较/逻辑运算 |

### 3.2 已失败 (FAIL) — 252 用例

以下类别在 v2.8.0 中失败：

| 类别 | 失败数 | 根因 |
|------|--------|------|
| **函数调用 (COALESCE/NULLIF/IFNULL)** | ~30+ | 解析器不支持 `FunctionName(...)` 小写语法，需 `FUNCTION(...)` 或大写 |
| **CASE 表达式** | ~20+ | 解析器不支持 CASE WHEN ... THEN ... END |
| **子查询 (标量/表)** | ~30+ | 解析器不支持 `(SELECT ...)` 嵌套 |
| **GROUP BY 表达式** | ~15+ | 解析器不支持 GROUP BY 中的表达式引用 |
| **窗口函数** | ~20+ | 已标记 `-- === SKIP ===`，解析器不支持 ROW_NUMBER/RANK/DENSE_RANK OVER |
| **CTE (WITH ... AS)** | ~15+ | 解析器不支持公用表表达式 |
| **集合操作 (UNION/INTERSECT/EXCEPT)** | ~10+ | 解析器不支持 UNION 操作 |
| **类型转换 (CAST/CONVERT)** | ~10+ | 解析器不支持 CAST 语法 |
| **日期时间函数** | ~15+ | 函数调用语法不支持 + 日期函数名不识别 |
| **字符串函数** | ~15+ | 函数调用语法不支持 |
| **数学/数值函数** | ~15+ | 函数调用语法不支持 |
| **JSON 函数** | ~8+ | 函数调用语法不支持 |
| **外键/约束** | ~10+ | 解析器不支持完整 CONSTRAINT 语法 |
| **触发器** | ~8+ | 解析器不支持 CREATE TRIGGER |
| **存储过程** | ~10+ | 解析器不支持 CREATE PROCEDURE |
| **视图** | ~8+ | 解析器不支持 CREATE VIEW |
| **事务控制 (COMMIT/ROLLBACK/SAVEPOINT)** | ~8+ | 解析器不支持 TCL 语句 |

### 3.3 已跳过 (SKIP) — 标记为 `-- === SKIP ===`

以下文件直接标记跳过，不纳入计数：

| 文件 | 原因 |
|------|------|
| `EXPRESSIONS/window_functions.sql` | 窗口函数未实现 |
| `EXPRESSIONS/window_functions_advanced.sql` | 高级窗口函数未实现 |
| `PROCEDURES/stored_procedures.sql` | 存储过程未实现 |
| `TRIGGERS/trigger_statements.sql` | 触发器未实现 |
| `VIEWS/view_statements.sql` | 视图未实现 |
| `EVENTS/event_statements.sql` | 事件未实现 |

---

## 四、回归测试执行计划

### 4.1 测试执行矩阵

| 阶段 | 测试命令 | 目标通过率 | 时间估计 |
|------|----------|-----------|----------|
| **L0: 冒烟** | `cargo test -p sqlrustgo-sql-corpus --lib -- test_sql_corpus_all` | ≥ 40% (基线) | ~5 min |
| **L1: 分类回归** | 按类别单独执行 | 逐类验证 | ~15 min |
| **L2: 全量回归** | `cargo test -p sqlrustgo-sql-corpus --all-features` | ≥ 40% | ~30 min |

### 4.2 分类验证命令

```bash
# 基础 SELECT
cargo test -p sqlrustgo-sql-corpus --lib -- test_sql_corpus_all

# JOIN 验证
cargo test -p sqlrustgo-sql-corpus --lib -- test_sql_corpus_joins

# 子查询验证 (预期失败)
cargo test -p sqlrustgo-sql-corpus --lib -- test_sql_corpus_subqueries

# 聚合验证
cargo test -p sqlrustgo-sql-corpus --lib -- test_sql_corpus_aggregates
```

### 4.3 回归门禁标准

| 门禁 | 要求 | 当前状态 |
|------|------|---------|
| P0: 基础 SQL 无退化 | 基本 SELECT/INSERT/UPDATE/DELETE/DDL 通过率 100% | ✅ 基线建立 |
| P1: 聚合/JOIN 无退化 | 聚合函数和 JOIN 通过率 ≥ 90% | ✅ 基线建立 |
| P2: 表达式通过率 | 表达式类别通过率 ≥ 50% | ⚠️ 40.8% |
| P3: 无新增回归 | 新提交不降低已有通过率 | ⚠️ 需监控 |

---

## 五、回归条目详细清单

### 5.1 按 SQL 文件统计 (已通过文件)

| 文件路径 | 用例数 | 通过 | 失败 |
|---------|--------|------|------|
| `DML/SELECT/basic_select.sql` | ~20 | ~20 | 0 |
| `DML/SELECT/where_conditions.sql` | ~15 | ~15 | 0 |
| `DML/SELECT/order_by.sql` | ~10 | ~10 | 0 |
| `DML/SELECT/limit_offset.sql` | ~8 | ~8 | 0 |
| `DML/SELECT/aggregates.sql` | ~6 | ~6 | 0 |
| `DML/INSERT/insert_statements.sql` | ~12 | ~12 | 0 |
| `DML/UPDATE/update_statements.sql` | ~8 | ~8 | 0 |
| `DML/DELETE/delete_statements.sql` | ~8 | ~8 | 0 |
| `DDL/CREATE_TABLE/create_table.sql` | ~10 | ~10 | 0 |
| `DDL/DROP_TABLE/drop_table.sql` | ~5 | ~5 | 0 |
| ... (其他已通过文件) |

### 5.2 按 SQL 文件统计 (已失败文件)

| 文件路径 | 用例数 | 通过 | 失败 | 根因 |
|---------|--------|------|------|------|
| `EXPRESSIONS/case_expressions.sql` | ~15 | 0 | ~15 | CASE 语法不支持 |
| `EXPRESSIONS/conditional_expressions.sql` | ~10 | 0 | ~10 | IF/COALESCE 函数调用 |
| `EXPRESSIONS/type_conversion.sql` | ~8 | 0 | ~8 | CAST 语法不支持 |
| `FUNCTIONS/CONDITIONALS/conditional_functions.sql` | ~15 | 0 | ~15 | COALESCE/NULLIF/IFNULL |
| `FUNCTIONS/STRING/string_functions.sql` | ~12 | 0 | ~12 | 函数调用语法 |
| `FUNCTIONS/DATE_TIME/date_time_functions.sql` | ~10 | 0 | ~10 | 函数调用语法 |
| `FUNCTIONS/NUMERIC/numeric_functions.sql` | ~10 | 0 | ~10 | 函数调用语法 |
| `DML/SELECT/subqueries.sql` | ~12 | 0 | ~12 | 子查询语法 |
| `DML/SELECT/cte_operations.sql` | ~8 | 0 | ~8 | CTE 语法 |
| `DML/SELECT/union_operations.sql` | ~8 | 0 | ~8 | UNION 语法 |
| `ADVANCED/SUBQUERIES/subquery_statements.sql` | ~15 | 0 | ~15 | 子查询语法 |
| `ADVANCED/GROUP_BY/group_by_statements.sql` | ~10 | 0 | ~10 | GROUP BY 表达式 |
| `DDL/FOREIGN_KEY/cascade.sql` | ~5 | 0 | ~5 | 外键语法 |
| `TCL/tcl_statements.sql` | ~5 | 0 | ~5 | TCL 语法 |
| ... (其他失败文件) |

---

## 六、v2.8.0 回归基线 vs v2.9.0 目标

### 6.1 当前基线 (v2.8.0 GA)

| 维度 | 数值 |
|------|------|
| 总 SQL 文件 | 103 |
| 总测试用例 | 426 |
| 通过 | 174 |
| 失败 | 252 |
| **通过率** | **40.8%** |
| 已跳过文件 | 6 (窗口函数/存储过程/触发器/视图/事件) |

### 6.2 v2.9.0 回归目标

| 修复项 | 预期增量 | 目标通过率 |
|--------|---------|-----------|
| 函数调用语法 (FunctionName → FUNCTION) | +80 | 59.6% |
| CASE 表达式支持 | +20 | 64.3% |
| 子查询支持 | +30 | 71.4% |
| GROUP BY 表达式支持 | +15 | 74.9% |
| UNION 支持 | +10 | 77.2% |
| **v2.9.0 合计目标** | **+155** | **≥ 77%** |

---

## 七、回归测试流程

### 7.1 每日回归

```bash
# 全量回归（CI 自动执行）
cargo test -p sqlrustgo-sql-corpus --all-features 2>&1 | grep -E "Total:|Pass rate:|failures:"

# 快速回归（仅基础 SQL）
cargo test -p sqlrustgo-sql-corpus --lib -- test_sql_corpus_all 2>&1 | tail -10
```

### 7.2 提交前验证

开发人员在提交任何解析器或执行器变更前，应运行：

```bash
# 确保不引入回归
cargo test -p sqlrustgo-sql-corpus --all-features

# 验证通过率不低于基线（40.8%）
# 如通过率下降，必须修复后再提交
```

### 7.3 回归报告生成

```bash
# 生成详细回归报告
cargo test -p sqlrustgo-sql-corpus --all-features -- --nocapture 2>&1 | tee sql_corpus_report_$(date +%Y%m%d).log
```

---

## 八、已知限制与风险

### 8.1 解析器限制 (v2.8.0)

1. **函数调用语法**: 解析器不支持小写 `function_name()` 调用，需使用大写 `FUNCTION_NAME()` 或带 `FUNCTION` 关键字前缀
2. **CASE 表达式**: 不支持 `CASE WHEN ... THEN ... END` 语法
3. **子查询**: 不支持 `(SELECT ...)` 作为表达式或表
4. **窗口函数**: 不支持 `OVER` 子句
5. **CTE**: 不支持 `WITH ... AS (...)` 语法
6. **集合操作**: 不支持 `UNION`/`INTERSECT`/`EXCEPT`

### 8.2 执行器限制

1. **MemoryStorage 限制**: 回归测试使用内存存储，不涉及磁盘 I/O
2. **SimpleExecutor 简化**: 不支持 JOIN 执行、完整聚合执行、子查询执行
3. **类型系统简化**: 仅支持 Integer/Float/Text/Null 四种类型

### 8.3 回归测试覆盖盲区

1. **TPC-H 查询**: 未纳入 SQL Corpus（需 SF=1 数据）
2. **分布式 SQL**: 未纳入回归（分区/复制特定语法）
3. **性能回归**: SQL Corpus 不包含性能指标

---

## 九、附录

### 9.1 测试环境

```yaml
测试框架: sqlrustgo-sql-corpus v2.8.0
存储引擎: MemoryStorage
解析器: sqlrustgo-parser (当前限制: 无函数调用)
执行器: SimpleExecutor (有限功能子集)
运行模式: cargo test --lib
```

### 9.2 相关文档

- [TEST_REPORT.md](./TEST_REPORT.md) — v2.8.0 完整测试报告
- [TEST_PLAN.md](./TEST_PLAN.md) — v2.8.0 测试计划
- [PARSER_COVERAGE_ANALYSIS.md](./PARSER_COVERAGE_ANALYSIS.md) — 解析器覆盖分析
- `crates/sql-corpus/src/lib.rs` — SQL Corpus 框架源码
- `sql_corpus/` — SQL 语料库目录

### 9.3 命令速查

```bash
# 查看所有 SQL 语料库文件
find sql_corpus -name "*.sql" | sort | wc -l
# 预期: 103

# 统计测试用例总数
grep -r "\-\- === CASE:" sql_corpus/ | wc -l
# 预期: ~426

# 运行单一分类
cargo test -p sqlrustgo-sql-corpus --lib -- test_sql_corpus_joins

# 全量回归 + 详细输出
cargo test -p sqlrustgo-sql-corpus --all-features -- --nocapture
```

---

*本文档由 Hermes Agent 自动生成*
*创建日期: 2026-05-02*
*基于: v2.8.0 GA (commit 64ed7a5e)*
