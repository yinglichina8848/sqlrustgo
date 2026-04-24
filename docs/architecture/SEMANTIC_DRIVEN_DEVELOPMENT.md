# 语义驱动开发：从功能验证到语义不变量

> SQLRustGo v2.8.0 架构方法论总结

## 概述

本文档记录了一次从**功能驱动开发**到**语义驱动开发**的方法论转变过程。通过修复 SQL NULL 语义的 bug，我们建立了一套以**语义不变量**为核心的开发范式。

**核心转变**：从"这个功能能工作吗？"到"这个语义正确吗？"

---

## 第一部分：WHAT — 发生了什么

### 1.1 问题描述

### 1.2 解决过程

### 1.3 技术成果

---

## 第二部分：WHY — 为什么重要

### 2.1 功能驱动 vs 语义驱动

### 2.2 语义不变量的价值

### 2.3 这种方法论为什么有效

---

## 第三部分：HOW — 如何应用

### 3.1 识别语义边界

### 3.2 编写语义测试

### 3.3 建立单一语义入口

### 3.4 持续验证

---

## 附录：关键代码结构

---

## 第一部分：WHAT — 发生了什么

### 1.1 问题描述

#### 初始 Bug

```
SQL: SELECT * FROM t1 LEFT JOIN t2 ON t1.id = t2.id WHERE t2.id IS NULL
期望: [1, 3]  (t1.id=1 和 t1.id=3 的 t2.id 为 NULL)
实际: [1, 2, 3]  (所有行都被返回)
```

**根因分析**：`execute_select_with_join` 函数在计算 JOIN 结果后，**没有应用 WHERE 子句过滤**，直接返回了所有行。

#### 深层问题

这不是单一的 bug，而是**语义散射**问题：

| 位置 | 问题 |
|------|------|
| `execute_select_with_join` | JOIN 结果不经过 WHERE 过滤 |
| `find_column_index` | 不支持带限定符的列名（`t2.id`） |
| 多个位置 | NULL 处理逻辑重复且不一致 |

### 1.2 解决过程

#### 阶段 1：发现与隔离

```
原始行为: NULL = NULL 在 HashJoin 中错误匹配
        ↓
原因: 使用 format!("{:?}", value) 产生字符串 "Null"
        ↓
结果: NULL 键被放入 HashMap，错误地与另一个 NULL 匹配
```

#### 阶段 2：测试先行

我们没有直接修复 bug，而是先编写**语义测试**来定义正确行为：

```rust
// semantic_guard: join_null_filter
#[test]
fn test_join_with_is_null_filter() {
    // 数据: t1=[1,2,3], t2=[2]
    // SQL: SELECT t1.id FROM t1 LEFT JOIN t2 ON t1.id = t2.id WHERE t2.id IS NULL
    // 期望: [1, 3]
    // 锁定的语义: JOIN 不匹配 → NULL 填充 → WHERE IS NULL 过滤
}
```

#### 阶段 3：语义集中化

我们没有在各个位置打补丁，而是创建了**单一语义入口**：

```rust
// eval_predicate() - 谓词评估的单一入口
fn eval_predicate(expr: &Expression, row: &[Value], table_info: &TableInfo) -> bool {
    match expr {
        // IS NULL
        Expression::IsNull(inner) => {
            matches!(evaluate_expression(inner, row, table_info), Ok(Value::Null))
        }
        // 所有比较操作符走 sql_compare
        Expression::BinaryOp(left, op, right) => {
            sql_compare(op, &left_val, &right_val)
        }
        // ...
    }
}

// sql_compare() - NULL 语义的统一处理
fn sql_compare(op: &str, left: &Value, right: &Value) -> bool {
    if matches!(left, Value::Null) || matches!(right, Value::Null) {
        return false;  // Phase 1: UNKNOWN → FALSE
    }
    // ...
}
```

#### 阶段 4：修复漏掉的环节

即使有了 `eval_predicate`，WHERE 过滤仍然没有生效：

```rust
// execute_select_with_join 原始代码
let matched_results = match join_type { /* JOIN 计算 */ };
let row_count = matched_results.len();
Ok(ExecutorResult::new(matched_results, row_count))  // 缺少 WHERE 过滤！
```

修复：

```rust
let mut matched_results = match join_type { /* JOIN 计算 */ };

// 添加 WHERE 过滤
if let Some(ref where_expr) = select.where_clause {
    let combined_table_info = build_combined_schema(...);
    matched_results.retain(|row| eval_predicate(where_expr, row, &combined_table_info));
}
```

### 1.3 技术成果

#### 代码层面

| 指标 | 改进前 | 改进后 |
|------|--------|--------|
| NULL 处理位置 | 散落在 5+ 处 | 集中在 2 处 |
| IS NULL 支持 | 需要 BinaryOp 模拟 | 原生 Expression::IsNull |
| WHERE 过滤（JOIN） | 缺失 | 完整实现 |

#### 架构层面

```
旧架构:                          新架构:
evaluate_binary_comparison()      eval_predicate()
        ↓                               ↓
NULL 逻辑散落               →     sql_compare()
                                      ↓
                                统一 NULL 处理
```

#### 测试层面

```
旧:                               新:
功能测试                         语义护城河测试
  "JOIN 能跑"                      semantic_guard: join_null_filter
                                    ↓
                                  "NULL 不匹配 + JOIN 填充 + IS NULL 过滤"
```

---

## 第二部分：WHY — 为什么重要

### 2.1 功能驱动 vs 语义驱动

| 维度 | 功能驱动 | 语义驱动 |
|------|----------|----------|
| 核心问题 | "这个功能能用吗？" | "这个语义正确吗？" |
| 测试关注点 | 输入→输出正确 | 行为的不变性 |
| 失败模式 | 功能坏了才知道 | 语义漂移前就发现 |
| 可演进性 | 差（新功能破坏旧功能） | 好（语义边界清晰） |

### 2.2 语义不变量的价值

**语义不变量**是系统中永远不应该改变的行为：

```
✓ NULL = NULL 返回 UNKNOWN（不是 TRUE）
✓ LEFT JOIN 保留所有左表行
✓ WHERE 只保留 TRUE（不是 UNKNOWN）
✓ COUNT(*) 计算所有行包括 NULL
```

**为什么叫"不变量"（Invariant）？**

因为它们是**跨越版本保持不变**的承诺：

```rust
// 即使未来实现 TriBool (三值逻辑)
// 这些不变量仍然必须成立：
// - NULL = NULL ≠ TRUE
// - NULL = NULL ≠ FALSE
// - NULL = NULL IS UNKNOWN
```

### 2.3 这种方法论为什么有效

#### 1. 语义边界清晰

通过测试明确**什么行为是绝对不能改的**：

```rust
// 这个测试锁定了 JOIN + NULL + IS NULL 的交互语义
// 即使未来重构 JOIN 实现，这个测试也必须通过
#[test]
fn test_join_with_is_null_filter() { /* ... */ }
```

#### 2. 防止回归的"护城河"

语义测试像护城河一样保护核心语义：

```
            ┌─────────────────────────────┐
            │      新功能 / 重构           │
            └──────────────┬──────────────┘
                           ↓
                   语义护城河测试
                           ↓
              ┌────────────────────────────┐
              │  语义护城河 1: NULL 比较    │
              │  语义护城河 2: JOIN NULL    │
              │  语义护城河 3: IS NULL 过滤 │
              └────────────────────────────┘
                           ↓
                    核心语义不被破坏
```

#### 3. 从"修 bug"到"证语义"

**传统做法**：
```
用户报告 bug → 修复 → 测试修复有效 → 完成
```

**语义驱动做法**：
```
发现语义漏洞 → 编写语义测试（定义正确行为）→ 修复 → 验证测试通过 → 测试成为不变量
```

区别：传统修复是临时的，语义测试是永久的保护。

#### 4. 功能的可证明性

语义驱动回答了一个更根本的问题：

> "你怎么知道这个功能是对的？"

传统 TDD：测试驱动开发，测试"这个函数返回正确结果"
语义 TDD：语义驱动开发，测试"这个语义在任何情况下都成立"

```
功能测试：  assert_eq!(add(2, 3), 5);        // 5 是正确结果
语义测试：  assert!(NULL == NULL != TRUE);   // NULL 语义是永恒的
```

---

## 第三部分：HOW — 如何应用

### 3.1 识别语义边界

**问题**：什么样的行为应该成为语义不变量？

**判断标准**：

1. **SQL 标准规定的语义**
   - NULL 的比较行为
   - JOIN 的行保留语义
   - 聚合函数的 NULL 处理

2. **业务逻辑的核心假设**
   - 账户余额不能为负
   - 事务的 ACID 语义
   - 外键引用的完整性

3. **架构决策的约束**
   - eval_predicate 是谓词评估的唯一入口
   - 所有 NULL 逻辑走 sql_compare
   - WHERE 只保留 TRUE

**练习**：问自己"如果这个行为改变了，什么会坏？"

- 如果 NULL = NULL 变成 TRUE → 所有 JOIN 查询结果错误
- 如果 LEFT JOIN 不保留左表行 → 数据丢失，违反 SQL 标准
- 如果 WHERE 把 UNKNOWN 当 TRUE → 查询返回错误行

### 3.2 编写语义测试

**命名规范**：

```rust
// semantic_guard: [语义领域]
// 描述被锁定的具体语义行为

#[test]
fn semantic_guard_join_null_filter() {
    // 清晰的 Given-When-Then 注释
    // 数据设置
    // 执行 SQL
    // 验证语义（不只是结果正确，而是语义正确）
}
```

**语义测试 vs 功能测试**：

```rust
// ❌ 功能测试 - 只验证结果
#[test]
fn test_left_join() {
    let result = execute("SELECT * FROM t1 LEFT JOIN t2 ON t1.id = t2.id");
    assert_eq!(result.len(), 3);
}

// ✅ 语义测试 - 验证语义不变量
#[test]
fn semantic_guard_left_join_null_key_handling() {
    // 插入包含 NULL 键的数据
    execute("INSERT INTO t1 VALUES (NULL), (1), (2)");
    execute("INSERT INTO t2 VALUES (1)");
    
    // 执行 LEFT JOIN
    let result = execute("SELECT * FROM t1 LEFT JOIN t2 ON t1.id = t2.id");
    
    // 验证语义不变量：
    // 1. NULL 键永远不匹配
    assert!(result.rows.iter().any(|r| r.t1_id == NULL && r.t2_id == NULL));
    // 2. 所有左表行都被保留
    assert_eq!(result.len(), 3);
    // 3. 匹配的行正确连接
    assert!(result.rows.iter().any(|r| r.t1_id == 1 && r.t2_id == 1));
}
```

### 3.3 建立单一语义入口

**原则**：每个语义概念有且只有一个入口点

**反模式**：语义散射

```rust
// ❌ 语义散射 - NULL 处理在多处重复
fn evaluate_binary_op(...) {
    if matches!(left, Value::Null) || matches!(right, Value::Null) {
        return Value::Null;  // 这里处理 NULL
    }
}

fn evaluate_where_clause(...) {
    match expr {
        // 又处理一次 NULL
        Value::Null => false,
        // ...
    }
}

fn execute_hash_join(...) {
    // 再处理一次 NULL
    if matches!(left_key, Value::Null) {
        skip;  // NULL 不匹配
    }
}
```

**正确模式**：单一入口

```rust
// ✅ 单一入口 - NULL 处理在一处
fn eval_predicate(expr: &Expression, row: &[Value], table_info: &TableInfo) -> bool {
    match expr {
        Expression::BinaryOp(left, op, right) => {
            let left_val = evaluate_expression(left, row, table_info)?;
            let right_val = evaluate_expression(right, row, table_info)?;
            sql_compare(op, &left_val, &right_val)  // NULL 处理在这里
        }
        // ...
    }
}

// sql_compare 是比较操作符的单一语义入口
fn sql_compare(op: &str, left: &Value, right: &Value) -> bool {
    if matches!(left, Value::Null) || matches!(right, Value::Null) {
        return false;  // Phase 1: UNKNOWN → FALSE
    }
    // ...
}
```

### 3.4 持续验证

**CI 集成**：

```bash
# 运行语义护城河测试（每次 PR 必须通过）
cargo test semantic_guard

# 运行完整测试套件
cargo test --all-features
```

**代码审查检查清单**：

- [ ] 新代码是否引入了新的语义概念？
- [ ] 如果是，这个语义是否有对应的语义测试？
- [ ] 新代码是否复用了已有的语义入口？
- [ ] 是否有去除语义散射的机会？

**文档化**：

```rust
/// 评估谓词表达式的单一入口
/// Phase 1: UNKNOWN 折叠为 FALSE
/// Phase 2: 实现 NOT 和 AND/OR 的三值逻辑
/// Phase 3: TriBool 支持完整三值逻辑
fn eval_predicate(expr: &Expression, row: &[Value], table_info: &TableInfo) -> bool {
    // ...
}
```

---

## 附录：关键代码结构

### A.1 eval_predicate 架构

```rust
/// SQL 谓词评估的单一语义入口
///
/// # 语义保证
/// - 所有 NULL 比较通过 sql_compare 统一处理
/// - IS NULL / IS NOT NULL 有专门处理路径
/// - AND / OR 短路求值
///
/// # 阶段演进
/// - Phase 1 (当前): UNKNOWN → FALSE
/// - Phase 2: Option<bool> 支持 NOT
/// - Phase 3: TriBool 完整三值逻辑
fn eval_predicate(expr: &Expression, row: &[Value], table_info: &TableInfo) -> bool {
    match expr {
        // AND 短路求值
        Expression::BinaryOp(left, op, right) if op == "AND" => {
            eval_predicate(left, row, table_info) && eval_predicate(right, row, table_info)
        }
        // OR 短路求值
        Expression::BinaryOp(left, op, right) if op == "OR" => {
            eval_predicate(left, row, table_info) || eval_predicate(right, row, table_info)
        }
        // IS NULL
        Expression::IsNull(inner) => {
            matches!(evaluate_expression(inner, row, table_info), Ok(Value::Null))
        }
        // IS NOT NULL
        Expression::IsNotNull(inner) => {
            !matches!(evaluate_expression(inner, row, table_info), Ok(Value::Null))
        }
        // 遗留 IS NULL 语法 (col IS NULL)
        Expression::BinaryOp(left, "IS", right @ Expression::Literal("NULL")) => {
            eval_predicate(&Expression::IsNull(left.clone()), row, table_info)
        }
        // 所有比较操作符
        Expression::BinaryOp(left, op, right) => {
            let l = evaluate_expression(left, row, table_info).unwrap_or(Value::Null);
            let r = evaluate_expression(right, row, table_info).unwrap_or(Value::Null);
            sql_compare(op, &l, &r)
        }
        // 其他表达式
        _ => matches!(evaluate_expression(expr, row, table_info), Ok(Value::Boolean(true))),
    }
}
```

### A.2 JOIN + WHERE 过滤架构

```rust
fn execute_select_with_join(select: &SelectStatement) -> SqlResult<ExecutorResult> {
    // 1. 执行 JOIN 获取结果
    let mut matched_results = match join_type {
        JoinType::Left | JoinType::Inner => {
            hash_join(left_rows, right_rows, condition, left_schema, right_schema)
        }
        // ...
    };

    // 2. 应用 WHERE 过滤（使用联合 schema）
    if let Some(ref where_expr) = select.where_clause {
        let combined_table_info = build_combined_schema(left_table_info, right_table_info);
        matched_results.retain(|row| eval_predicate(where_expr, row, &combined_table_info));
    }

    Ok(ExecutorResult::new(matched_results, row_count))
}
```

### A.3 语义护城河测试示例

```rust
// 文件: crates/executor/tests/hash_join_left_null_test.rs

// semantic_guard: join_null_filter
// 锁定: JOIN 不匹配 → NULL 填充 → IS NULL 过滤 的组合语义
#[test]
fn test_join_with_is_null_filter() {
    // Given: t1=[1,2,3], t2=[2]
    execute("INSERT INTO t1 VALUES (1), (2), (3)");
    execute("INSERT INTO t2 VALUES (2)");
    
    // When: LEFT JOIN + WHERE IS NULL
    let result = execute(
        "SELECT t1.id FROM t1 LEFT JOIN t2 ON t1.id = t2.id WHERE t2.id IS NULL"
    );
    
    // Then: 应该只有 t1.id=1 和 t1.id=3（它们的 t2.id 为 NULL）
    assert_eq!(result.rows.len(), 2);
    assert!(ids.contains(&1));
    assert!(ids.contains(&3));
}
```

---

## 结论

语义驱动开发不是要替代 TDD，而是 TDD 的**深化**：

| 层次 | 问题 | 方法 |
|------|------|------|
| 功能测试 | 这个功能正确吗？ | TDD |
| 语义测试 | 这个语义不变吗？ | 语义护城河 |
| 架构验证 | 这个设计合理吗？ | 架构评审 |

**核心洞见**：

> 功能的正确性可以被测试，但语义的不变性需要被守护。

通过建立语义不变量，我们从一个"不断修复 bug"的模式，进化到一个"语义被测试保护，安心演进"的模式。

---

## 参考文献

- SQL92 标准：NULL 比较语义
- SQLRustGo Issue #1833：Three-Valued Logic 债务追踪
- 本次改造相关 Commit：
  - `498dfd37`: 修复 HashJoin NULL 语义并添加测试
  - `5606342e`: 建立 eval_predicate 单一语义入口
  - `b180520d`: 添加 LEFT JOIN NULL 语义黄金测试