# FULL OUTER JOIN 设计方案

> **日期**: 2026-04-20
> **状态**: Draft
> **目标**: 实现 FULL OUTER JOIN 支持，带去重优化

## 1. 概述

实现 SQL 标准 FULL OUTER JOIN，使用 HashJoin 算法，基于 3 Phase 去重策略避免结果重复。

### 1.1 目标

- Parser 支持 `FULL OUTER JOIN` 和 `FULL JOIN` 语法
- Executor 实现 FULL OUTER JOIN，带去重优化
- 通过 TPC-DS Q2.6 及相关测试

### 1.2 当前状态

| 组件 | 状态 |
|-------|------|
| Lexer (`Token::Full`) | ✅ 已有 |
| Parser (`JoinType::Full`) | ⚠️ 已有，但未处理 FULL 关键字 |
| Executor (`JoinType::Full`) | ❌ 未实现 |

## 2. 架构设计

### 2.1 执行流程

```
┌─────────────────────────────────────────────────────────────┐
│                        Parser                               │
│  parse_join_clause(): 处理 Token::Full → JoinType::Full    │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    HashJoinExec                            │
│  execute_hash_join() → match JoinType::Full                │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│              执行逻辑 (3 Phase 去重算法)                     │
│  Phase 1: LEFT JOIN 结果 (匹配 + 左表未匹配)                 │
│  Phase 2: 收集 Phase1 匹配右表的 key                        │
│  Phase 3: 右表中 key ∉ Phase2 → (NULL, right_row)          │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 3 Phase 去重算法

```
输入: 左表 L, 右表 R, 连接条件 key
输出: FULL OUTER JOIN 结果 (无重复)

Phase 1: 标准 LEFT JOIN
  - 遍历 L 的每一行
  - 在 R 中查找匹配键
  - 若匹配: emit (L.row, R.row) → 记录 R.row 的 key 到 matched_R_keys
  - 若不匹配: emit (L.row, NULL)

Phase 2: 右表 ANTI JOIN
  - 遍历 R 的每一行
  - 若其键值 ∉ matched_R_keys: emit (NULL, R.row)
  - 否则跳过 (已在 Phase 1 以匹配形式输出)

结果: 三部分输出互斥，无重复
  - L 匹配 R 的行
  - L 未匹配 R 的行 (右表列为 NULL)
  - R 未匹配 L 的行 (左表列为 NULL)
```

## 3. 实现设计

### 3.1 Parser 层变更

**文件**: `crates/parser/src/parser.rs`

**修改位置**: `parse_join_clause()` 函数 (约 line 1135)

```rust
fn parse_join_clause(&mut self) -> Result<JoinClause, String> {
    let join_type = match self.current() {
        Some(Token::Inner) => {
            self.next();
            JoinType::Inner
        }
        Some(Token::Left) => {
            self.next();
            // Check if followed by Token::Outer
            if matches!(self.current(), Some(Token::Outer)) {
                self.next();
            }
            JoinType::Left
        }
        Some(Token::Right) => {
            self.next();
            if matches!(self.current(), Some(Token::Outer)) {
                self.next();
            }
            JoinType::Right
        }
        Some(Token::Full) => {
            self.next();
            // FULL [OUTER] JOIN - consume optional OUTER
            if matches!(self.current(), Some(Token::Outer)) {
                self.next();
            }
            JoinType::Full
        }
        Some(Token::Cross) => {
            self.next();
            JoinType::Cross
        }
        Some(Token::Join) => {
            self.next();
            JoinType::Inner
        }
        _ => return Err("Expected JOIN type".to_string()),
    };
    // ... rest unchanged
}
```

### 3.2 Executor 层变更

**文件**: `crates/executor/src/local_executor.rs`

**修改位置**: `execute_hash_join()` 函数中的 `match join_type` (约 line 648)

```rust
JoinType::Full => {
    // Phase 1: Standard LEFT JOIN
    let matched = hash_inner_join(
        &left_result.rows,
        &right_result.rows,
        condition,
        left_schema,
        right_schema,
    );

    // Track matched right keys for deduplication
    let matched_right_keys: HashSet<Vec<Value>> = matched
        .iter()
        .skip(left_schema.fields.len())
        .cloned()
        .collect();

    // Phase 2: LEFT-only rows (same as LEFT JOIN)
    let left_only: Vec<Vec<Value>> = left_result
        .rows
        .iter()
        .filter(|lrow| {
            !matched.iter().any(|m| {
                m.iter().take(lrow.len()).cloned().collect::<Vec<_>>()
                    == lrow.iter().cloned().collect::<Vec<_>>()
            })
        })
        .map(|lrow| {
            let mut row = lrow.clone();
            row.extend(vec![Value::Null; right_schema.fields.len()]);
            row
        })
        .collect();

    // Phase 3: RIGHT-only rows (not in matched_right_keys)
    let right_only: Vec<Vec<Value>> = right_result
        .rows
        .iter()
        .filter(|rrow| {
            let key: Vec<Value> = rrow.iter().cloned().collect();
            !matched_right_keys.contains(&key)
        })
        .map(|rrow| {
            let mut row = vec![Value::Null; left_schema.fields.len()];
            row.extend(rrow.clone());
            row
        })
        .collect();

    // Combine all three parts
    let mut results = matched;
    results.extend(left_only);
    results.extend(right_only);

    Ok(ExecutorResult::new(results, 0))
}
```

### 3.3 Parallel Executor 变更

**文件**: `crates/executor/src/parallel_executor.rs`

**修改位置**: `execute_hash_join()` 中的 match (约 line 322)

需要添加 `JoinType::Full` 分支，处理方式与 local_executor 类似。

## 4. 测试计划

### 4.1 单元测试

| 测试用例 | 描述 |
|----------|------|
| `test_full_outer_join_basic` | 两表简单 FULL JOIN |
| `test_full_outer_join_no_match` | 两表无匹配行 |
| `test_full_outer_join_all_match` | 两表所有行都匹配 |
| `test_full_outer_join_partial_match` | 部分匹配场景 |
| `test_full_outer_join_multi_column` | 多列连接条件 |
| `test_full_outer_join_with_nulls` | 含 NULL 值的连接 |

### 4.2 集成测试

| 测试用例 | 描述 |
|----------|------|
| `test_tpcds_q2_6` | TPC-DS Q2.6 FULL OUTER JOIN 查询 |
| `test_full_join_read_write` | 含 FULL JOIN 的读写混合查询 |

### 4.3 边界测试

| 测试用例 | 描述 |
|----------|------|
| 空表 FULL JOIN | 左表或右表为空 |
| 单行表 FULL JOIN | 极端小数据量 |
| 大数据量 FULL JOIN | 验证内存使用 |

## 5. 验收标准

1. **语法**: `SELECT * FROM t1 FULL OUTER JOIN t2 ON t1.id = t2.id` 正确解析
2. **语义**: 结果包含左表全部行 + 右表全部行，未匹配的补 NULL
3. **去重**: 结果无重复行
4. **性能**: TPC-DS Q2.6 在 SF=1 下 < 100ms
5. **兼容性**: 与 PostgreSQL FULL OUTER JOIN 语义一致

## 6. 涉及文件

| 文件 | 修改类型 |
|------|----------|
| `crates/parser/src/parser.rs` | 修改 |
| `crates/executor/src/local_executor.rs` | 修改 |
| `crates/executor/src/parallel_executor.rs` | 修改 |
| `crates/executor/tests/` | 新增测试 |

## 7. 风险与备选方案

| 风险 | 概率 | 影响 | 备选方案 |
|------|------|------|----------|
| NULL 值导致 key 匹配问题 | 中 | 高 | 使用 COALESCE 或特殊 NULL 处理 |
| 内存占用过高 | 低 | 中 | 流式处理或外部排序 |
| 去重逻辑错误 | 中 | 高 | 严格测试覆盖 |

### NULL 值处理策略

FULL OUTER JOIN 中 NULL 值的比较需要特殊处理：
- `NULL = NULL` 在 SQL 中是 UNKNOWN，不应匹配
- 实现时需要显式处理 NULL 键值对
