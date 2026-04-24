# 执行模型统一重构方案

> 目标：将 JOIN 纳入统一 SQL pipeline
> 当前 PR: #1839
> 重构目标 PR: 下一版本

## 一、当前问题

```
execute_select:
├── has_join = true  → execute_select_with_join (直接返回，绕过 AGG/HAVING) ❌
└── has_join = false → scan → WHERE → GROUP BY → AGG → HAVING ✅
```

## 二、目标执行模型

```
execute_select:
└── FROM/JOIN (返回 rows)
    └──统一 pipeline:
        ├── WHERE      (eval_predicate)
        ├── GROUP BY   (build_groups)
        ├── AGGREGATE (compute_aggregates)
        ├── HAVING     (eval_predicate)
        └── PROJECTION (evaluate_expression)
```

## 三、重构代码骨架

### 3.1 修改 execute_select 入口

```rust
fn execute_select(&self, select: &SelectStatement) -> SqlResult<ExecutorResult> {
    let storage = self.storage.read().unwrap();

    // 1. FROM/JOIN - 生成初始 rows
    let (rows, table_info) = if select.join_clause.is_some() {
        self.execute_from_with_join(select)?
    } else {
        let rows = storage.scan(&select.table)?;
        let table_info = storage.get_table_info(&select.table)?;
        (rows, table_info)
    };

    // 2. WHERE 过滤
    let rows = if let Some(ref where_expr) = select.where_clause {
        self.apply_where(rows, where_expr, &table_info)?
    } else {
        rows
    };

    // 3. GROUP BY + AGGREGATE
    let rows = if !select.aggregates.is_empty() {
        self.apply_group_by_and_aggregate(rows, select, &table_info)?
    } else {
        rows
    };

    // 4. HAVING 过滤
    let rows = if let Some(ref having_expr) = select.having {
        self.apply_having(rows, having_expr, select, &table_info)?
    } else {
        rows
    };

    let row_count = rows.len();
    Ok(ExecutorResult::new(rows, row_count))
}
```

### 3.2 修改 execute_from_with_join (原 execute_select_with_join)

```rust
/// 执行 FROM/JOIN，返回 (rows, combined_schema)
/// 注意：这个函数只负责生成 rows，不走后续 pipeline
fn execute_from_with_join(&self, select: &SelectStatement) -> SqlResult<(Vec<Vec<Value>>, TableInfo)> {
    let join_clause = select.join_clause.as_ref().unwrap();
    let left_table_name = select.table.clone();
    let right_table_name = join_clause.table.clone();

    let storage = self.storage.read().unwrap();

    // Scan both tables
    let left_rows = storage.scan(&left_table_name)?;
    let right_rows = storage.scan(&right_table_name)?;

    let left_table_info = storage.get_table_info(&left_table_name)?;
    let right_table_info = storage.get_table_info(&right_table_name)?;

    // Extract join key column indices
    let left_key_idx = self.find_join_key_index(&join_clause.on_clause, &left_table_info, &select.table)?;
    let right_key_idx = self.find_join_key_index(&join_clause.on_clause, &right_table_info, &right_table_name)?;

    let join_type = match join_clause.join_type {
        ParserJoinType::Inner => JoinType::Inner,
        ParserJoinType::Left => JoinType::Left,
        ParserJoinType::Right => JoinType::Right,
        ParserJoinType::Full => JoinType::Full,
        ParserJoinType::Cross => JoinType::Cross,
    };

    let left_col_count = left_table_info.columns.len();
    let right_col_count = right_table_info.columns.len();

    // 执行 JOIN，只返回 rows
    let rows = match join_type {
        JoinType::Inner | JoinType::Left | JoinType::Right | JoinType::Full => {
            self.hash_join(left_rows, right_rows, left_key_idx, right_key_idx, left_col_count, right_col_count, join_type)?
        }
        JoinType::Cross => {
            self.cross_join(&left_rows, &right_rows)?
        }
    };

    // 构建联合 schema
    let combined_schema = self.build_combined_schema(&left_table_info, &right_table_name, &right_table_info)?;

    Ok((rows, combined_schema))
}
```

### 3.3 新增 apply_where

```rust
/// 应用 WHERE 过滤
fn apply_where(
    &self,
    rows: Vec<Vec<Value>>,
    where_expr: &Expression,
    table_info: &TableInfo,
) -> SqlResult<Vec<Vec<Value>>> {
    let mut result = Vec::new();
    for row in rows {
        if eval_predicate(where_expr, &row, table_info) {
            result.push(row);
        }
    }
    Ok(result)
}
```

### 3.4 新增 apply_group_by_and_aggregate

```rust
/// 应用 GROUP BY + AGGREGATE
fn apply_group_by_and_aggregate(
    &self,
    rows: Vec<Vec<Value>>,
    select: &SelectStatement,
    table_info: &TableInfo,
) -> SqlResult<Vec<Vec<Value>>> {
    let group_exprs = &select.group_by;

    if group_exprs.is_empty() {
        // 无 GROUP BY：简单 aggregate
        let agg_values = self.compute_aggregates(&select.aggregates, &rows, table_info)?;
        return Ok(vec![agg_values]);
    }

    // 有 GROUP BY：分组 + aggregate
    let mut groups: HashMap<String, Vec<Vec<Value>>> = HashMap::new();

    // 构建分组 key（支持 NULL）
    for row in &rows {
        let key = group_exprs
            .iter()
            .map(|expr| evaluate_expr_to_string(expr, row, table_info))
            .collect::<Vec<_>>()
            .join("\x00");
        groups.entry(key).or_default().push(row.clone());
    }

    // 对每个分组计算 aggregate
    let mut result_rows = Vec::new();
    for (key, group_rows) in groups {
        let agg_values = self.compute_aggregates(&select.aggregates, &group_rows, table_info)?;

        // 构建分组 key 值
        let key_values: Vec<Value> = key
            .split('\x00')
            .map(|s| {
                if s == "NULL" {
                    Value::Null
                } else if let Ok(n) = s.parse::<i64>() {
                    Value::Integer(n)
                } else {
                    Value::Text(s.to_string())
                }
            })
            .collect();

        let mut combined = key_values;
        combined.extend(agg_values);
        result_rows.push(combined);
    }

    Ok(result_rows)
}
```

### 3.5 新增 apply_having

```rust
/// 应用 HAVING 过滤
fn apply_having(
    &self,
    rows: Vec<Vec<Value>>,
    having_expr: &Expression,
    select: &SelectStatement,
    _table_info: &TableInfo,
) -> SqlResult<Vec<Vec<Value>>> {
    // 构建 aggregate schema（用于 HAVING 中的 aggregate 引用）
    let having_schema = self.build_aggregate_schema(&select.group_by, &select.aggregates)?;

    let mut result = Vec::new();
    for row in rows {
        if eval_predicate(having_expr, &row, &having_schema) {
            result.push(row);
        }
    }
    Ok(result)
}
```

### 3.6 新增辅助函数

```rust
/// 构建联合 schema（JOIN 结果用）
fn build_combined_schema(
    &self,
    left_info: &TableInfo,
    right_info: &TableInfo,
) -> SqlResult<TableInfo> {
    let mut columns = Vec::new();

    for c in &left_info.columns {
        columns.push(ColumnDefinition {
            name: format!("{}.{}", left_info.name, c.name),
            data_type: c.data_type.clone(),
            nullable: c.nullable,
            primary_key: c.primary_key,
        });
    }

    for c in &right_info.columns {
        columns.push(ColumnDefinition {
            name: format!("{}.{}", right_info.name, c.name),
            data_type: c.data_type.clone(),
            nullable: c.nullable,
            primary_key: c.primary_key,
        });
    }

    Ok(TableInfo {
        name: format!("{}_join_{}", left_info.name, right_info.name),
        columns,
        foreign_keys: vec![],
        unique_constraints: vec![],
        check_constraints: vec![],
        partition_info: None,
    })
}

/// 构建 aggregate schema（用于 HAVING）
fn build_aggregate_schema(
    &self,
    group_by: &[Expression],
    aggregates: &[AggregateCall],
) -> SqlResult<TableInfo> {
    let mut columns = Vec::new();

    // GROUP BY 列
    for expr in group_by {
        columns.push(ColumnDefinition {
            name: expression_to_string(expr),
            data_type: "INTEGER".to_string(),
            nullable: false,
            primary_key: false,
        });
    }

    // Aggregate 列
    for agg in aggregates {
        columns.push(ColumnDefinition {
            name: expression_to_string(&Expression::Aggregate(agg.clone())),
            data_type: "INTEGER".to_string(),
            nullable: false,
            primary_key: false,
        });
    }

    Ok(TableInfo {
        name: "aggregate".to_string(),
        columns,
        foreign_keys: vec![],
        unique_constraints: vec![],
        check_constraints: vec![],
        partition_info: None,
    })
}
```

## 四、重构影响分析

| 现有函数 | 修改方式 | 原因 |
|----------|----------|------|
| `execute_select_with_join` | **重构为** `execute_from_with_join` | 只返回 rows，不走 pipeline |
| `execute_select` | **扩展** pipeline 逻辑 | 处理 JOIN rows 继续 pipeline |
| `compute_aggregates` | **保持不变** | 已在 HAVING 中使用 |
| `eval_predicate` | **保持不变** | WHERE/HAVING 共用 |
| `evaluate_expression` | **保持不变** | 已支持 Aggregate |

## 五、必须通过的测试

| 测试 | SQL | 验证点 |
|------|-----|--------|
| 1 | `SELECT COUNT(t2.id) FROM t1 LEFT JOIN t2 ON t1.id = t2.id WHERE t2.id IS NOT NULL` | JOIN + WHERE + AGG |
| 2 | `SELECT t1.id, COUNT(t2.id) FROM t1 LEFT JOIN t2 ON t1.id = t2.id GROUP BY t1.id HAVING COUNT(t2.id) > 0` | JOIN + GROUP BY + AGG + HAVING |
| 3 | `SELECT SUM(col) FROM t` (all NULL) | 全 NULL aggregate |

## 六、风险评估

| 风险 | 缓解措施 |
|------|----------|
| 影响现有 JOIN 功能 | 先备份现有测试，逐步验证 |
| schema 构建复杂 | 复用已有 `build_combined_schema` 逻辑 |
| GROUP BY key 构建 | 已有 `evaluate_expr_to_string` 可用 |

## 七、推荐实施步骤

1. **阶段 1**：提取 `execute_from_with_join`，返回 `(rows, schema)`
2. **阶段 2**：在 `execute_select` 中调用 `execute_from_with_join`，然后走 pipeline
3. **阶段 3**：删除原 `execute_select_with_join` 中的 pipeline 逻辑
4. **阶段 4**：添加 `apply_*` 辅助函数
5. **阶段 5**：运行完整测试验证

## 八、预期收益

- ✅ 单一执行模型，无语义分叉
- ✅ JOIN 可复用 WHERE/AGG/HAVING
- ✅ 易于扩展（Window / Subquery）
- ✅ 易于优化（vectorized / parallel）
- ✅ 可预测的执行语义