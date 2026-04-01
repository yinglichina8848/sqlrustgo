# TPC-H 第一阶段实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 BETWEEN、DATE、IN value list 三个 SQL 特性，启用 15 个 TPC-H 查询

**Architecture:** 在 Parser 层添加语法支持，将新语法转换为已有的表达式结构

**Tech Stack:** Rust, Cargo, SQLRustGo parser/executor/planner

---

## 概述

第一阶段实现三个 SQL 特性：
1. BETWEEN 操作符 (1-2 天)
2. DATE 字面量 (1-2 天)  
3. IN value list (2-3 天)

---

## 阶段 1: BETWEEN 操作符

### Task 1: 分析 BETWEEN 现状

**Files:**
- Examine: `crates/parser/src/parser.rs` - 查找 `parse_comparison_expression` 方法
- Examine: `crates/parser/src/token.rs` - 确认 `Token::Between` 存在

**Step 1: 查找 Between 解析位置**

Run: `rg "parse_comparison" crates/parser/src/parser.rs | head -20`
Expected: 显示 parse_comparison_expression 方法位置

**Step 2: 确认 Token 定义存在**

Run: `rg "Between" crates/parser/src/token.rs`
Expected: 显示 Token::Between 定义

---

### Task 2: 添加 BETWEEN 解析

**Files:**
- Modify: `crates/parser/src/parser.rs` - 在 `parse_comparison_expression` 方法中添加 BETWEEN 处理

**Step 1: 找到 parse_comparison_expression 方法**

Run: `rg -n "fn parse_comparison_expression" crates/parser/src/parser.rs`
Expected: 显示方法开始行号

**Step 2: 阅读方法结构，找到比较操作符处理位置**

Read: `crates/parser/src/parser.rs` offset 1200 limit 100 (具体行号待上一步确认)

**Step 3: 添加 BETWEEN 处理逻辑**

在比较操作符解析后添加:
```rust
// Handle BETWEEN: a BETWEEN x AND y -> a >= x AND a <= y
if self.parse_keyword(&["BETWEEN"]) {
    let low = self.parse_operator precedence_expression()?;
    self.expect_keyword("AND")?;
    let high = self.parse_operator precedence_expression()?;
    return Ok(Expr::BinaryOp {
        left: Box::new(expr),
        op: Operator::And,
        right: Box::new(Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(expr.clone()),
                op: Operator::Gte,
                right: Box::new(low),
            }),
            op: Operator::And,
            right: Box::new(Expr::BinaryOp {
                left: Box::new(expr),
                op: Operator::Lte,
                right: Box::new(high),
            }),
        }),
    });
}
```

**Step 4: 编写测试验证**

在 `tests/integration/tpch_test.rs` 或新建测试文件:
```rust
#[test]
fn test_between_operator() {
    let mut engine = create_engine();
    engine.execute(parse("CREATE TABLE t (a INT, b INT)").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO t VALUES (5, 10)").unwrap()).unwrap();
    
    let result = engine.execute(parse("SELECT * FROM t WHERE a BETWEEN 1 AND 10").unwrap()).unwrap();
    assert_eq!(result.rows.len(), 1);
    
    let result = engine.execute(parse("SELECT * FROM t WHERE a BETWEEN 6 AND 20").unwrap()).unwrap();
    assert_eq!(result.rows.len(), 0);
}
```

**Step 5: 运行测试**

Run: `cargo test test_between_operator -- --nocapture`
Expected: 测试通过

---

### Task 3: 验证 TPC-H Q6 使用 BETWEEN

**Step 1: 找到 Q6 测试**

Read: `tests/integration/tpch_full_test.rs` - 查找 Q6 相关测试

**Step 2: 尝试运行 Q6**

Run: `cargo test tpch_q6 -- --nocapture`
Expected: 显示 Q6 执行结果

**Step 3: 确认 BETWEEN 在 Q6 中工作**

如果测试失败，检查错误信息，修复问题。

---

## 阶段 2: DATE 字面量

### Task 4: 分析 DATE 解析现状

**Files:**
- Examine: `crates/parser/src/parser.rs` - 查找日期相关解析
- Examine: `crates/parser/src/token.rs` - 查找 DATE 相关 token

**Step 1: 搜索 DATE 相关代码**

Run: `rg "DATE|date" crates/parser/src/token.rs | head -20`
Expected: 显示 DATE 相关 token 定义

**Step 2: 查找 TPC-H 中的 DATE 用法**

Run: `rg "DATE '199" tests/integration/tpch_full_test.rs`
Expected: 显示 TPC-H Q1 日期字面量

---

### Task 5: 实现 DATE 'yyyy-mm-dd' 解析

**Files:**
- Modify: `crates/parser/src/parser.rs` - 添加 DATE 字面量解析

**Step 1: 找到字面量解析位置**

Run: `rg -n "fn parse_value" crates/parser/src/parser.rs`
Expected: 显示 parse_value 或类似方法

**Step 2: 添加 DATE 关键字检测**

在值解析中添加:
```rust
if self.parse_keyword(&["DATE"]) {
    self.expect_token(&Token::StringLit)?;
    let date_str = self.prev.token_str();
    return Ok(Expr::Value(Value::Text(date_str)));
}
```

**Step 3: 测试 DATE 解析**

```rust
#[test]
fn test_date_literal() {
    let mut engine = create_engine();
    engine.execute(parse("CREATE TABLE t (d TEXT)").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO t VALUES ('2024-01-01')").unwrap()).unwrap();
    
    // DATE '2024-01-01' 应该被解析为字符串 '2024-01-01'
    let result = engine.execute(parse("SELECT * FROM t WHERE d = DATE '2024-01-01'").unwrap()).unwrap();
    assert_eq!(result.rows.len(), 1);
}
```

**Step 4: 运行测试**

Run: `cargo test test_date_literal -- --nocapture`
Expected: 测试通过

---

### Task 6: 验证 DATE 在 Q1 中工作

**Step 1: 找到 Q1 测试**

Read: `tests/integration/tpch_full_test.rs` - 查找 Q1 测试

**Step 2: 尝试运行 Q1**

Run: `cargo test tpch_q1 -- --nocapture`
Expected: 显示 Q1 执行结果

---

## 阶段 3: IN value list

### Task 7: 分析 IN 解析现状

**Files:**
- Examine: `crates/parser/src/parser.rs` - 查找 IN 相关解析
- Examine: `crates/planner/src/lib.rs` - 查找 Expr::InList

**Step 1: 搜索 IN 相关代码**

Run: `rg "InList|InSubquery" crates/parser/src/parser.rs crates/planner/src/lib.rs`
Expected: 显示 IN 相关代码位置

**Step 2: 确认 Expr 枚举中的 IN 变体**

Run: `rg "Expr::In" crates/planner/src/lib.rs | head -10`
Expected: 显示 Expr 枚举定义

---

### Task 8: 添加 IN value list 解析

**Files:**
- Modify: `crates/parser/src/parser.rs` - 在 `parse_in_expression` 或类似方法中添加
- Modify: `crates/planner/src/lib.rs` - 添加 `Expr::InList` 变体
- Modify: `crates/executor/src/executor.rs` - 实现 IN list 执行

**Step 1: 找到 IN 表达式解析位置**

Run: `rg -n "fn parse_in" crates/parser/src/parser.rs`
Expected: 显示 parse_in_expression 方法位置

**Step 2: 阅读现有 IN 解析逻辑**

Read: `crates/parser/src/parser.rs` offset 1400 limit 50 (具体行号待确认)

**Step 3: 添加 IN (value, value, ...) 解析**

在 IN 解析中添加:
```rust
// 如果不是子查询，则为值列表
let values = self.parse_comma_separated(Parser::parse_value)?;
return Ok(Expr::InList {
    expr: Box::new(expr),
    values,
});
```

**Step 4: 在 Expr 枚举中添加 InList**

Modify: `crates/planner/src/lib.rs`
```rust
pub enum Expr {
    // ... existing variants ...
    InList {
        expr: Box<Expr>,
        values: Vec<Value>,
    },
}
```

**Step 5: 在 executor 中实现 InList 执行**

Modify: `crates/executor/src/executor.rs`
```rust
// 在 evaluate_expression 中添加:
// Expr::InList { expr, values } => values.contains(&evaluate(expr)?)
```

**Step 6: 编写测试**

```rust
#[test]
fn test_in_list() {
    let mut engine = create_engine();
    engine.execute(parse("CREATE TABLE t (a INT)").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO t VALUES (1), (2), (3), (4)").unwrap()).unwrap();
    
    let result = engine.execute(parse("SELECT * FROM t WHERE a IN (1, 3)").unwrap()).unwrap();
    assert_eq!(result.rows.len(), 2);
}
```

**Step 7: 运行测试**

Run: `cargo test test_in_list -- --nocapture`
Expected: 测试通过

---

### Task 9: 验证 IN list 在 Q12/Q16/Q22 中工作

**Step 1: 找到相关测试**

Run: `rg "Q12|Q16|Q22" tests/integration/tpch_full_test.rs`
Expected: 显示各查询测试

**Step 2: 逐个运行测试**

Run: `cargo test tpch_q12 -- --nocapture`
Run: `cargo test tpch_q16 -- --nocapture`
Run: `cargo test tpch_q22 -- --nocapture`

---

## 集成测试

### Task 10: 运行完整 TPC-H 测试套件

**Step 1: 运行所有 TPC-H 测试**

Run: `cargo test --test tpch_test --test tpch_benchmark --test tpch_full_test 2>&1 | tail -50`
Expected: 所有测试通过

**Step 2: 验证性能基准**

Run: `cargo test --test tpch_benchmark -- --nocapture 2>&1 | grep -E "(Q[0-9]|Performance|SQLite)"`
Expected: 显示性能对比结果

**Step 3: 验证数据库对比**

Run: `cargo test --test tpch_full_test -- --nocapture 2>&1 | grep -E "(SQLite|PostgreSQL|MySQL)"`
Expected: 显示各数据库测试结果

---

## 提交和文档

### Task 11: 提交代码

**Step 1: 检查修改文件**

Run: `git status`
Expected: 显示修改的文件

**Step 2: 添加并提交**

```bash
git add crates/parser/src/parser.rs crates/planner/src/lib.rs crates/executor/src/executor.rs
git add tests/integration/tpch_*.rs
git commit -m "feat(tpch): implement BETWEEN, DATE, IN value list for TPC-H Q1-Q22"
```

**Step 3: 更新设计文档**

Modify: `docs/plans/2026-04-02-tpch-full-implementation-plan.md`
- 将 Phase 1 标记为完成
- 记录实际遇到的问题和解决方案

---

## 预计时间

- BETWEEN 实现: 1-2 天
- DATE 实现: 1-2 天
- IN list 实现: 2-3 天
- 集成测试: 0.5-1 天
- **总计: 4.5-8.5 天**

---

## 风险缓解

| 风险 | 缓解措施 |
|------|----------|
| Parser 修改破坏现有功能 | 每个任务后运行完整测试套件 |
| DATE 比较不工作 | 使用 ISO 字符串比较，验证日期排序 |
| IN list 性能问题 | 后续优化，当前先保证正确性 |

---

## 下一步

第一阶段完成后，进行第二阶段：
1. COUNT(DISTINCT) - Q16 需要
2. CASE 表达式 - Q1, Q14, Q19 需要
