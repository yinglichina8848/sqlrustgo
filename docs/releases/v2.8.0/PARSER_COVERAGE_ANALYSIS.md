# parser 模块覆盖率分析

> 日期: 2026-04-30
> 基线: Line 54.70%, Region 52.01%, Func 70.41%
> 目标: Line 85%

---

## 1. 测试现状

### 1.1 已有测试覆盖

| 功能 | 测试函数 | 状态 |
|------|---------|------|
| INNER JOIN | `test_parse_inner_join` | ✅ |
| LEFT JOIN | `test_parse_left_join` | ✅ |
| RIGHT JOIN | `test_parse_right_join` | ✅ |
| FULL OUTER JOIN | `test_parse_full_join` | ✅ |
| INSERT | `test_parse_insert*` | ✅ |
| UPDATE | `test_parse_update*` | ✅ |
| DELETE | `test_parse_delete*` | ✅ |
| REPLACE INTO | `test_parse_replace_into*` | ✅ |
| BEGIN/COMMIT/ROLLBACK | `test_parse_begin/commit/rollback` | ✅ |
| 聚合函数 | `test_parse_aggregate_*` | ✅ |
| 二元表达式 | `test_parse_binary_expression_*` | ✅ |
| 比较表达式 | `test_parse_comparison_expression` | ✅ |

### 1.2 未覆盖区域

| 功能 | 缺失测试 | 优先级 |
|------|---------|--------|
| **CROSS JOIN** | `test_parse_cross_join` | P0 |
| **NATURAL JOIN** | `test_parse_natural_join` | P0 |
| **JOIN USING** | `test_parse_join_using` | P0 |
| **JOIN ON 多条件** | `test_parse_join_on_multiple` | P1 |
| **TRUNCATE TABLE** | `test_parse_truncate` | P0 |
| **IS NULL/IS NOT NULL** | `test_parse_is_null_expression` | P0 |
| **三值逻辑 AND/OR/NOT** | `test_parse_three_valued_logic` | P0 |
| **表达式优先级** | `test_parse_expression_precedence` | P1 |
| **子查询 (IN/EXISTS)** | `test_parse_subquery` | P1 |
| **语法错误恢复** | `test_parse_error_recovery` | P1 |

---

## 2. 未覆盖的关键函数

### 2.1 parse_join_clause (行 ~1194)

```rust
fn parse_join_clause(&mut self) -> Result<JoinClause, String>
```

分支覆盖分析:
- `CROSS` keyword → CROSS JOIN 分支
- `NATURAL` keyword → NATURAL JOIN 分支
- `JOIN` with `USING` → USING clause 分支
- `ON` keyword → ON expression 分支

### 2.2 parse_expression (行 ~1506)

```rust
fn parse_expression(&mut self) -> Result<Expression, String>
```

分支覆盖分析:
- `IS NULL` → IsNull expression
- `IS NOT NULL` → IsNotNull expression
- `AND` → 三值逻辑 AND
- `OR` → 三值逻辑 OR
- `NOT` → 三值逻辑 NOT

### 2.3 parse_truncate (行 ~2098)

```rust
fn parse_truncate(&mut self) -> Result<Statement, String>
```

这个函数完全没有测试！

---

## 3. 测试策略

### 3.1 P0 测试 (必须添加)

1. **三值逻辑测试** (6个)
   - `test_parse_is_null_with_null`
   - `test_parse_is_not_null_with_null`
   - `test_parse_and_with_null`
   - `test_parse_or_with_null`
   - `test_parse_not_with_null`

2. **JOIN 类型补全** (4个)
   - `test_parse_cross_join`
   - `test_parse_natural_join`
   - `test_parse_join_using`
   - `test_parse_join_on_multiple_conditions`

3. **TRUNCATE 测试** (2个)
   - `test_parse_truncate_table`
   - `test_parse_truncate_basic`

### 3.2 P1 测试 (尽量添加)

4. **表达式优先级** (4个)
   - `test_parse_expression_precedence_and_or`
   - `test_parse_expression_precedence_arithmetic`
   - `test_parse_expression_parentheses`

5. **错误处理** (4个)
   - `test_parse_unterminated_string`
   - `test_parse_invalid_syntax`
   - `test_parse_missing_paren`

---

## 4. 预期覆盖率提升

| 添加测试 | 预期 Line% 提升 |
|---------|----------------|
| 三值逻辑 (6个) | +8% |
| JOIN 补全 (4个) | +5% |
| TRUNCATE (2个) | +3% |
| 表达式优先级 (4个) | +4% |
| 错误处理 (4个) | +5% |
| **合计** | **+25% → ~80%** |

---

*分析完成: 2026-04-30*
