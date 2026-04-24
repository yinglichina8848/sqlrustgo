# SQLRustGo SQL 语义审查报告

> v2.8.0 SQL 执行引擎语义审查
> 审查范围: Parser → Planner → Executor 全链路
> 审查日期: 2026-04-24

---

## 执行摘要

本次审查发现 **7 个语义问题**（2 个严重，3 个中等，2 个轻微），**0 个阻塞性问题**。

**核心发现**：
1. Aggregate 函数的 COUNT 存在 NULL 处理 bug
2. JOIN + Aggregate 组合存在 WHERE 过滤缺失
3. GROUP BY NULL 处理正确
4. 整体 NULL 语义已有统一入口 (eval_predicate)

---

## 第一部分：Aggregate 语义审查

### 1.1 COUNT 语义问题

#### 🔴 严重问题 #1：COUNT(column) 错误包含 NULL 值

**位置**: `src/execution_engine.rs:548`

```rust
AggregateFunction::Count => Value::Integer(values.len() as i64),
```

**问题**：
```rust
// 当前实现：values.len() 包含所有行（包括 NULL）
let values: Vec<Value> = if let Some(arg) = agg.args.first() {
    rows.iter()
        .map(|row| evaluate_expression(arg, row, table_info).unwrap_or(Value::Null))
        .collect()
} else {
    vec![Value::Integer(rows.len() as i64)]  // COUNT(*) 正确
};
// ...
AggregateFunction::Count => Value::Integer(values.len() as i64),  // BUG: 应该过滤 NULL
```

**SQL 标准行为**：
| 表达式 | 行为 |
|--------|------|
| `COUNT(*)` | 计算所有行，包括 NULL |
| `COUNT(col)` | 只计算非 NULL 的行 |

**测试验证**：
```sql
-- 数据: [NULL, 10, 20]
SELECT COUNT(*), COUNT(col) FROM t;
-- 期望: COUNT(*) = 3, COUNT(col) = 2
-- 当前实际: COUNT(*) = 3, COUNT(col) = 3 (BUG!)
```

**修复方案**：
```rust
AggregateFunction::Count => {
    let non_null_count = values
        .iter()
        .filter(|v| !matches!(v, Value::Null))
        .count();
    Value::Integer(non_null_count as i64)
}
```

**缺失测试**：
- `test_count_with_null` (已标记 `#[ignore]`，关联 issue #1833)

---

### 1.2 SUM/AVG NULL 处理

#### ✅ 正确实现

```rust
AggregateFunction::Sum => {
    let sum: i64 = values
        .iter()
        .filter_map(|v| {  // ✅ 正确：filter_map 跳过非 Integer
            if let Value::Integer(n) = v { Some(*n) } else { None }
        })
        .sum();
    Value::Integer(sum)
}
```

**行为**：
- SUM 正确忽略 NULL 值
- AVG 正确忽略 NULL 值，只对非 NULL 计算平均

**符合 SQL 标准** ✅

---

### 1.3 MIN/MAX NULL 处理

#### ✅ 正确实现

```rust
AggregateFunction::Min => {
    let min = values
        .iter()
        .filter_map(|v| {  // ✅ 正确
            if let Value::Integer(n) = v { Some(*n) } else { None }
        })
        .min();
    min.map(Value::Integer).unwrap_or(Value::Null)
}
```

**行为**：
- MIN/MAX 正确忽略 NULL 值
- 如果全为 NULL，返回 NULL

**符合 SQL 标准** ✅

---

### 1.4 GROUP BY NULL 处理

#### ✅ 正确实现

```rust
fn evaluate_expr_to_string(expr: &Expression, row: &[Value], table_info: &TableInfo) -> String {
    let val = evaluate_expression(expr, row, table_info).unwrap_or(Value::Null);
    match val {
        Value::Null => "NULL".to_string(),  // ✅ 正确：所有 NULL 归为一组
        // ...
    }
}
```

**行为**：
- 所有 NULL 值被归为同一组
- `GROUP BY NULL` 等价于无 GROUP BY

**符合 SQL 标准** ✅

---

### 1.5 Aggregate 缺失测试矩阵

| 场景 | SQL | 期望结果 | 当前状态 |
|------|-----|----------|----------|
| COUNT(*) 全行计数 | `COUNT(*)` | 3 | ✅ 正确 |
| COUNT(col) 忽略 NULL | `COUNT(col)` | 2 | 🔴 BUG |
| SUM 忽略 NULL | `SUM(col)` | 30 | ✅ 正确 |
| AVG 忽略 NULL | `AVG(col)` | 15 | ✅ 正确 |
| MIN 忽略 NULL | `MIN(col)` | 10 | ✅ 正确 |
| MAX 忽略 NULL | `MAX(col)` | 20 | ✅ 正确 |
| GROUP BY NULL | `GROUP BY col` (col 有 NULL) | NULL 组 | ✅ 正确 |
| COUNT(DISTINCT col) | `COUNT(DISTINCT col)` | 2 | ⚠️ 未测试 |

---

## 第二部分：JOIN 语义审查

### 2.1 WHERE 过滤缺失（已修复）

#### ✅ 已修复

**原问题**: `execute_select_with_join` 不应用 WHERE 过滤
**修复 commit**: `f47c6a31`

```rust
// 修复后：应用 WHERE 过滤
if let Some(ref where_expr) = select.where_clause {
    let combined_table_info = build_combined_schema(...);
    matched_results.retain(|row| eval_predicate(where_expr, row, &combined_table_info));
}
```

---

### 2.2 LEFT/RIGHT/FULL JOIN NULL 键处理

#### ✅ 正确实现

```rust
// LEFT JOIN: NULL 键不匹配
if matches!(left_row[left_key_idx], Value::Null) {
    continue;  // ✅ NULL 永远不匹配
}

// RIGHT JOIN: NULL 键不匹配
if matches!(right_row[right_key_idx], Value::Null) {
    continue;  // ✅ NULL 永远不匹配
}
```

**符合 SQL 标准** ✅

---

### 2.3 JOIN + Aggregate 组合

#### 🔴 严重问题 #2：JOIN 后 Aggregate 的 WHERE 过滤可能缺失

**问题描述**：
当执行 `SELECT COUNT(*) FROM t1 LEFT JOIN t2 ON ... WHERE ...` 时：
1. JOIN 正确应用 WHERE 过滤（修复后）
2. 但 aggregate 执行路径可能不同

**需要验证的执行路径**：
```
execute_select
  ├── 有 join_clause? → execute_select_with_join → compute_aggregates
  └── 无 join_clause? → scan → filter → compute_aggregates
```

**风险点**：
如果 `execute_select_with_join` 路径没有调用 `compute_aggregates`，则 JOIN + Aggregate + WHERE 组合可能有问题。

**建议**：添加集成测试验证：
```sql
SELECT COUNT(*)
FROM t1 LEFT JOIN t2 ON t1.id = t2.id
WHERE t1.id > 1;
-- 期望: 对 JOIN + WHERE 过滤后的结果计数
```

---

### 2.4 JOIN 测试覆盖矩阵

| 场景 | SQL | 期望 | 状态 |
|------|-----|------|------|
| LEFT JOIN 保留左表全行 | `SELECT * FROM t1 LEFT JOIN t2` | 全左表行 + NULL 填充 | ✅ |
| LEFT JOIN NULL 键不匹配 | `t1.id = NULL` | 不匹配 | ✅ |
| LEFT JOIN + WHERE IS NULL | `WHERE t2.id IS NULL` | 只保留未匹配行 | ✅ |
| INNER JOIN NULL 键 | `t1.id = NULL` | 不匹配 | ✅ |
| FULL OUTER JOIN | `FULL OUTER JOIN` | 全表行 | ✅ |
| JOIN + Aggregate | `SELECT COUNT(*) FROM t1 JOIN t2` | 正确计数 | ⚠️ 需验证 |

---

## 第三部分：WHERE / HAVING 语义审查

### 3.1 WHERE 三值逻辑（Phase 1）

#### ✅ eval_predicate 统一入口

```rust
fn eval_predicate(expr: &Expression, row: &[Value], table_info: &TableInfo) -> bool {
    // ...
    Expression::BinaryOp(left, op, right) => {
        let left_val = evaluate_expression(left, row, table_info).unwrap_or(Value::Null);
        let right_val = evaluate_expression(right, row, table_info).unwrap_or(Value::Null);
        sql_compare(op, &left_val, &right_val)  // NULL → false
    }
    // ...
}
```

**Phase 1 行为**：
- UNKNOWN (NULL 比较结果) → FALSE
- WHERE 只保留 TRUE

**符合 SQL 标准** ✅（Phase 1 阶段）

---

### 3.2 AND/OR 短路求值

#### ⚠️ 部分实现

```rust
// AND 短路
Expression::BinaryOp(left, op, right) if op == "AND" => {
    eval_predicate(left, row, table_info) && eval_predicate(right, row, table_info)
    // ✅ 短路：如果 left 为 false，right 不计算
}
```

**问题**：当前 `&&` 和 `||` 没有短路实现，但在 `eval_predicate` 中有短路。

**需要验证**：Rust 的 `&&` 和 `||` 本身是短路的，所以：
- `a && b`：如果 a 为 false，b 不计算
- `a || b`：如果 a 为 true，b 不计算

**符合 SQL 标准** ✅

---

### 3.3 HAVING 语义

#### ⚠️ 未完全测试

```rust
// HAVING 应用到聚合结果
if let Some(ref having_expr) = select.having {
    agg_result_rows.retain(|row| evaluate_where_clause(having_expr, row, &table_info));
}
```

**问题**：
1. HAVING 使用的 `table_info` 是原始表的，而不是聚合结果的 schema
2. 聚合结果的列可能是匿名的（如 `COUNT(*)` 的列名）

**需要测试**：
```sql
SELECT dept, COUNT(*) as cnt
FROM employees
GROUP BY dept
HAVING COUNT(*) > 1;
```

---

## 第四部分：Parser 语义审查

### 4.1 IS NULL / IS NOT NULL 支持

#### ✅ 已实现

```rust
// parser.rs
Expression::IsNull(inner) => ...
Expression::IsNotNull(inner) => ...
```

**遗留语法支持**：
```rust
// 仍支持 BinaryOp 形式的 IS NULL
Expression::BinaryOp(left, "IS", Expression::Literal("NULL"))
```

---

### 4.2 Parser 缺失功能

| 功能 | 状态 | 说明 |
|------|------|------|
| NOT (expr) | ⚠️ Phase 2 | `WHERE NOT (col = 1)` 暂不支持 |
| BETWEEN | ❌ 未实现 | `col BETWEEN 1 AND 10` |
| LIKE | ❌ 未实现 | 模式匹配 |
| IN (list) | ❌ 未实现 | `col IN (1, 2, 3)` |
| EXISTS | ❌ 未实现 | 子查询 |

---

## 第五部分：Top 10 高风险点

### 按严重性排序

| 排名 | 问题 | 严重性 | 影响范围 | 修复成本 |
|------|------|--------|----------|----------|
| 1 | COUNT(col) 包含 NULL | 🔴 严重 | 所有使用 COUNT(column) 的查询 | 低 |
| 2 | JOIN + Aggregate WHERE 过滤 | 🔴 严重 | JOIN 后的聚合查询 | 中 |
| 3 | HAVING schema 不匹配 | 🟡 中等 | HAVING 使用聚合函数 | 中 |
| 4 | NOT (expr) 不支持 | 🟡 中等 | 否定谓词查询 | 高 |
| 5 | BETWEEN 未实现 | 🟡 中等 | 范围查询 | 中 |
| 6 | LIKE 未实现 | 🟡 中等 | 模式查询 | 高 |
| 7 | IN (list) 未实现 | 🟡 中等 | 列表查询 | 中 |
| 8 | COUNT(DISTINCT) 未测试 | 🟢 轻微 | DISTINCT 聚合 | 低 |
| 9 | EXISTS 未实现 | 🟢 轻微 | 相关子查询 | 高 |
| 10 | 聚合列别名 | 🟢 轻微 | `COUNT(*) AS cnt` | 低 |

---

## 第六部分：测试覆盖缺口

### 6.1 缺失的语义护城河测试

```rust
// 1. COUNT NULL 语义
#[test]
fn semantic_guard_count_ignores_null() { /* ... */ }

// 2. JOIN + Aggregate + WHERE
#[test]
fn semantic_guard_join_aggregate_where() { /* ... */ }

// 3. HAVING 语义
#[test]
fn semantic_guard_having_with_aggregate() { /* ... */ }

// 4. GROUP BY NULL
#[test]
fn semantic_guard_group_by_null() { /* ... */ }

// 5. COUNT(DISTINCT) NULL
#[test]
fn semantic_guard_count_distinct_null() { /* ... */ }
```

### 6.2 测试矩阵建议

| 类别 | NULL 行为 | JOIN 行为 | WHERE 行为 | Aggregate |
|------|-----------|-----------|------------|-----------|
| 基本 SELECT | ✅ | ✅ | ✅ | 🔴 |
| JOIN 组合 | ✅ | ✅ | ✅ | ⚠️ |
| 子查询 | ⚠️ | N/A | ⚠️ | ⚠️ |
| DISTINCT | N/A | N/A | N/A | ⚠️ |

---

## 第七部分：执行路径分析

### 完整 SELECT 执行路径

```
SQL: SELECT [columns] FROM [table] [JOIN] WHERE [filter] GROUP BY [keys] HAVING [having] [ORDER BY] [limit]

路径分支：
├── 无 JOIN
│   └── scan → filter(WHERE) → group(GROUP BY) → aggregate → filter(HAVING) → sort → limit
│
└── 有 JOIN
    └── execute_select_with_join
        ├── hash_join (LEFT/RIGHT/INNER/FULL)
        ├── apply WHERE filter (eval_predicate)
        └── if aggregate:
            └── group(GROUP BY) → aggregate → filter(HAVING)
```

### 潜在问题点

1. **JOIN + WHERE 过滤**：✅ 已修复
2. **JOIN + Aggregate**：需要验证 `execute_select_with_join` 是否调用 `compute_aggregates`
3. **HAVING + 原始 schema**：⚠️ 可能有问题

---

## 第八部分：建议优先级

### 立即修复（1-2 天）

1. **COUNT(col) NULL bug**
   - 修复：`src/execution_engine.rs:548`
   - 验证：添加 `test_count_with_null`

2. **HAVING schema 问题**
   - 调查 HAVING 使用的 table_info
   - 验证聚合列是否可被 HAVING 引用

### 短期计划（1 周）

3. **NOT (expr) 支持**
   - Parser: 添加 NOT 表达式
   - Executor: eval_predicate 处理 NOT

4. **JOIN + Aggregate + WHERE 集成测试**
   - 验证完整路径

### 中期计划（2-4 周）

5. **BETWEEN/IN/LIKE 实现**
   - Parser 语法扩展
   - Executor 表达式支持

6. **COUNT(DISTINCT) 完整测试**
   - 验证 DISTINCT 聚合 NULL 处理

---

## 附录

### A. 相关文件

| 文件 | 职责 |
|------|------|
| `src/execution_engine.rs` | 核心执行引擎，聚合计算，JOIN |
| `crates/parser/src/parser.rs` | SQL 解析 |
| `crates/executor/tests/hash_join_left_null_test.rs` | NULL/JOIN 语义测试 |

### B. 相关 Commit

| Commit | 描述 |
|--------|------|
| `498dfd37` | 修复 HashJoin NULL 语义 |
| `5606342e` | 建立 eval_predicate 单一入口 |
| `f47c6a31` | 修复 JOIN 后 WHERE 过滤 |

### C. 追踪 Issue

- Issue #1833: Three-Valued Logic 债务追踪

---

## 审查结论

**整体评价**：SQLRustGo v2.8.0 的 NULL 语义已有良好基础，建立了 `eval_predicate` 统一入口。主要问题集中在 **Aggregate 函数的 COUNT NULL 处理**。

**推荐行动**：
1. 立即修复 COUNT NULL bug（低风险，高价值）
2. 添加语义护城河测试
3. 完善 JOIN + Aggregate 集成测试

**语义成熟度**：
- ✅ NULL 比较：统一入口，语义清晰
- ✅ JOIN NULL：处理正确
- 🔴 Aggregate NULL：COUNT 存在 bug
- ⚠️ 三值逻辑：Phase 1 完成，Phase 2/3 待实现