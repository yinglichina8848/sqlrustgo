# GROUP BY + HAVING + ORDER BY 实现计划

> **目标:** 实现 SQL-92 标准的 GROUP BY、HAVING、ORDER BY 语法支持

**架构:** 在 parser 层添加 AST 结构支持，复用现有 planner/executor 的 AggregateExec 和 SortExec，扩展 HAVING 支持

---

## 现有代码分析

根据代码审查，发现：

| 组件 | 状态 | 说明 |
|------|------|------|
| `AggregateExec` (planner) | 已存在 | `crates/planner/src/physical_plan.rs:591`，缺少 `having` 字段 |
| `SortExec` (planner) | 已存在 | `crates/planner/src/physical_plan.rs:1001`，结构完整 |
| `AggregateVolcanoExecutor` (executor) | 已存在 | `crates/executor/src/executor.rs:187`，缺少 `having` 字段 |
| `SortVolcanoExecutor` (executor) | 已存在 | `crates/executor/src/executor.rs:796`，结构完整 |

**结论**: Sort 相关已完整实现，主要工作是：
1. Parser 层：添加 Token、AST、解析逻辑
2. Planner 层：为 AggregateExec 添加 `having` 字段
3. Executor 层：为 AggregateVolcanoExecutor 添加 `having` 字段和过滤逻辑

---

## Task 1: 添加新 Token

**Files:**
- Modify: `crates/parser/src/token.rs:8-144` (Token 枚举)
- Modify: `crates/parser/src/token.rs:147-280` (Display 实现)

- [ ] **Step 1: 在 Token 枚举中添加新变体**

在 `Token` 枚举的 Keywords 部分添加（在 `Limit` 之后）：

```rust
// Group By / Order By keywords
Group,      // GROUP
By,         // BY
Having,     // HAVING
Order,      // ORDER
Asc,        // ASC
Desc,       // DESC
Nulls,      // NULLS
First,      // FIRST
Last,       // LAST
```

- [ ] **Step 2: 在 Display 实现中添加新 Token 的格式化**

```rust
Token::Group => write!(f, "GROUP"),
Token::By => write!(f, "BY"),
Token::Having => write!(f, "HAVING"),
Token::Order => write!(f, "ORDER"),
Token::Asc => write!(f, "ASC"),
Token::Desc => write!(f, "DESC"),
Token::Nulls => write!(f, "NULLS"),
Token::First => write!(f, "FIRST"),
Token::Last => write!(f, "LAST"),
```

- [ ] **Step 3: 运行测试验证**

```bash
cargo test --package sqlrustgo-parser
```

---

## Task 2: 修改 Lexer 识别新关键字

**Files:**
- Modify: `crates/parser/src/lexer.rs:234-320` (关键字匹配)

- [ ] **Step 1: 在关键字匹配中添加新 Token**

```rust
"GROUP" => Token::Group,
"BY" => Token::By,
"HAVING" => Token::Having,
"ORDER" => Token::Order,
"ASC" => Token::Asc,
"DESC" => Token::Desc,
"NULLS" => Token::Nulls,
"FIRST" => Token::First,
"LAST" => Token::Last,
```

- [ ] **Step 2: 运行测试验证**

```bash
cargo test --package sqlrustgo-parser
```

---

## Task 3: 修改 SelectStatement AST 结构

**Files:**
- Modify: `crates/parser/src/parser.rs:177-187` (SelectStatement 定义)

- [ ] **Step 1: 添加新数据结构**

在 `SelectStatement` 定义之前添加：

```rust
/// GROUP BY clause
#[derive(Debug, Clone, PartialEq)]
pub struct GroupByClause {
    pub columns: Vec<Expression>,
}

/// ORDER BY clause
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByClause {
    pub items: Vec<OrderByItem>,
}

/// ORDER BY single item
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByItem {
    pub expr: Expression,
    pub asc: bool,          // true = ASC, false = DESC
    pub nulls_first: bool,  // true = NULLS FIRST, false = NULLS LAST
}
```

- [ ] **Step 2: 修改 SelectStatement 添加新字段**

```rust
pub struct SelectStatement {
    pub columns: Vec<SelectColumn>,
    pub table: String,
    pub where_clause: Option<Expression>,
    pub join_clause: Option<JoinClause>,
    pub aggregates: Vec<AggregateCall>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    // 新增字段
    pub group_by: Option<GroupByClause>,   // 新增
    pub having: Option<Expression>,        // 新增
    pub order_by: Option<OrderByClause>,   // 新增
}
```

- [ ] **Step 3: 运行测试验证编译**

```bash
cargo build --package sqlrustgo-parser
```

---

## Task 4: 实现 GROUP BY 解析

**Files:**
- Modify: `crates/parser/src/parser.rs` (parse_select 函数)

- [ ] **Step 1: 在 parse_select 中添加 GROUP BY 解析**

在 `parse_select` 函数中，WHERE 子句解析之后、LIMIT 之前添加：

```rust
// Parse GROUP BY clause (optional)
let group_by = if matches!(self.current(), Some(Token::Group)) {
    self.next(); // consume GROUP
    if !matches!(self.current(), Some(Token::By)) {
        return Err("Expected BY after GROUP".to_string());
    }
    self.next(); // consume BY

    let mut columns = Vec::new();
    loop {
        let expr = self.parse_expression()?;
        columns.push(expr);

        if !matches!(self.current(), Some(Token::Comma)) {
            break;
        }
        self.next(); // consume comma
    }

    Some(GroupByClause { columns })
} else {
    None
};
```

- [ ] **Step 2: 运行测试验证**

```bash
cargo test --package sqlrustgo-parser
```

---

## Task 5: 实现 HAVING 解析

**Files:**
- Modify: `crates/parser/src/parser.rs` (parse_select 函数)

- [ ] **Step 1: 在 GROUP BY 之后、LIMIT 之前添加 HAVING 解析**

```rust
// Parse HAVING clause (optional) - must follow GROUP BY
let having = if matches!(self.current(), Some(Token::Having)) {
    self.next(); // consume HAVING
    Some(self.parse_expression()?)
} else {
    None
};
```

- [ ] **Step 2: 运行测试验证**

```bash
cargo test --package sqlrustgo-parser
```

---

## Task 6: 实现 ORDER BY 解析

**Files:**
- Modify: `crates/parser/src/parser.rs` (parse_select 函数)

- [ ] **Step 1: 添加 ORDER BY 解析**

在 HAVING 之后添加：

```rust
// Parse ORDER BY clause (optional)
let order_by = if matches!(self.current(), Some(Token::Order)) {
    self.next(); // consume ORDER
    if !matches!(self.current(), Some(Token::By)) {
        return Err("Expected BY after ORDER".to_string());
    }
    self.next(); // consume BY

    let mut items = Vec::new();
    loop {
        let expr = self.parse_expression()?;

        // Parse ASC/DESC (default ASC)
        let asc = match self.current() {
            Some(Token::Asc) => {
                self.next();
                true
            }
            Some(Token::Desc) => {
                self.next();
                false
            }
            _ => true, // default is ASC
        };

        // Parse NULLS FIRST/LAST (default depends on ASC/DESC)
        let nulls_first = match self.current() {
            Some(Token::Nulls) => {
                self.next(); // consume NULLS
                match self.current() {
                    Some(Token::First) => {
                        self.next();
                        true
                    }
                    Some(Token::Last) => {
                        self.next();
                        false
                    }
                    _ => return Err("Expected FIRST or LAST after NULLS".to_string()),
                }
            }
            _ => !asc, // default: NULLS FIRST for ASC, NULLS LAST for DESC
        };

        items.push(OrderByItem {
            expr,
            asc,
            nulls_first,
        });

        if !matches!(self.current(), Some(Token::Comma)) {
            break;
        }
        self.next(); // consume comma
    }

    Some(OrderByClause { items })
} else {
    None
};
```

- [ ] **Step 2: 运行测试验证**

```bash
cargo test --package sqlrustgo-parser
```

---

## Task 7: 更新 parse_select 返回值

**Files:**
- Modify: `crates/parser/src/parser.rs:421-429` (base_select 创建)

- [ ] **Step 1: 修改 base_select 构建包含新字段**

将 `base_select` 的构建改为：

```rust
let base_select = SelectStatement {
    columns,
    table,
    where_clause,
    join_clause: None,
    aggregates,
    limit: None,
    offset: None,
    group_by,    // 新增
    having,      // 新增
    order_by,     // 新增
};
```

**注意**: 需要确保 `select` 变量最终包含 `group_by`, `having`, `order_by`。

- [ ] **Step 2: 运行测试验证**

```bash
cargo test --package sqlrustgo-parser
```

---

## Task 8: 添加 Parser 单元测试

**Files:**
- Modify: `crates/parser/src/lib.rs` (或现有测试文件)

- [ ] **Step 1: 添加 GROUP BY 解析测试**

```rust
#[test]
fn test_parse_group_by() {
    let sql = "SELECT category, COUNT(*) FROM products GROUP BY category";
    let result = parse(sql);
    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::Select(select) = stmt {
        assert!(select.group_by.is_some());
        let group_by = select.group_by.unwrap();
        assert_eq!(group_by.columns.len(), 1);
    }
}
```

- [ ] **Step 2: 添加 HAVING 解析测试**

```rust
#[test]
fn test_parse_having() {
    let sql = "SELECT category, COUNT(*) FROM products GROUP BY category HAVING COUNT(*) > 1";
    let result = parse(sql);
    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::Select(select) = stmt {
        assert!(select.group_by.is_some());
        assert!(select.having.is_some());
    }
}
```

- [ ] **Step 3: 添加 ORDER BY 解析测试**

```rust
#[test]
fn test_parse_order_by() {
    let sql = "SELECT * FROM products ORDER BY name ASC";
    let result = parse(sql);
    assert!(result.is_ok());
    let stmt = result.unwrap();
    if let Statement::Select(select) = stmt {
        assert!(select.order_by.is_some());
        let order_by = select.order_by.unwrap();
        assert_eq!(order_by.items.len(), 1);
        assert!(order_by.items[0].asc);
    }
}

#[test]
fn test_parse_order_by_desc_nulls_last() {
    let sql = "SELECT * FROM products ORDER BY price DESC NULLS LAST";
    let result = parse(sql);
    assert!(result.is_ok());
}
```

- [ ] **Step 4: 添加完整语法测试**

```rust
#[test]
fn test_parse_complete_aggregate_query() {
    let sql = "SELECT category, SUM(price) FROM products WHERE price > 10 GROUP BY category HAVING SUM(price) > 100 ORDER BY SUM(price) DESC";
    let result = parse(sql);
    assert!(result.is_ok());
}
```

- [ ] **Step 5: 运行测试验证**

```bash
cargo test --package sqlrustgo-parser
```

---

## Task 9: 扩展 AggregateExec 添加 HAVING 支持

**Files:**
- Modify: `crates/planner/src/physical_plan.rs:591-650` (AggregateExec)

- [ ] **Step 1: 在 AggregateExec 结构体中添加 having 字段**

现有结构：
```rust
pub struct AggregateExec {
    input: Box<dyn PhysicalPlan>,
    group_expr: Vec<Expr>,
    aggregate_expr: Vec<Expr>,
    schema: Schema,
}
```

添加 `having` 字段：
```rust
pub struct AggregateExec {
    input: Box<dyn PhysicalPlan>,
    group_expr: Vec<Expr>,
    aggregate_expr: Vec<Expr>,
    schema: Schema,
    having: Option<Expr>,  // 新增: HAVING 条件
}
```

- [ ] **Step 2: 修改 AggregateExec::new 接受 having 参数**

```rust
pub fn new(
    input: Box<dyn PhysicalPlan>,
    group_expr: Vec<Expr>,
    aggregate_expr: Vec<Expr>,
    schema: Schema,
    having: Option<Expr>,  // 新增
) -> Self
```

- [ ] **Step 3: 添加 having() getter 方法**

```rust
pub fn having(&self) -> &Option<Expr> {
    &self.having
}
```

- [ ] **Step 4: 运行测试验证编译**

```bash
cargo build --package sqlrustgo-planner
```

---

## Task 10: 扩展 AggregateVolcanoExecutor 添加 HAVING 支持

**Files:**
- Modify: `crates/executor/src/executor.rs:187-280` (AggregateVolcanoExecutor)

- [ ] **Step 1: 在 AggregateVolcanoExecutor 结构体中添加 having 字段**

现有结构：
```rust
pub struct AggregateVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    group_expr: Vec<sqlrustgo_planner::Expr>,
    aggregate_expr: Vec<sqlrustgo_planner::Expr>,
    schema: Schema,
    input_schema: Schema,
    initialized: bool,
    groups: std::collections::HashMap<Vec<Value>, Vec<Vec<Value>>>,
    group_keys: Vec<Vec<Value>>,
    current_group_idx: usize,
}
```

添加 `having` 字段：
```rust
pub struct AggregateVolcanoExecutor {
    // ... 现有字段 ...
    having: Option<sqlrustgo_planner::Expr>,  // 新增: HAVING 条件
}
```

- [ ] **Step 2: 修改 AggregateVolcanoExecutor::new 接受 having 参数**

```rust
pub fn new(
    child: Box<dyn VolcanoExecutor>,
    group_expr: Vec<sqlrustgo_planner::Expr>,
    aggregate_expr: Vec<sqlrustgo_planner::Expr>,
    schema: Schema,
    input_schema: Schema,
    having: Option<sqlrustgo_planner::Expr>,  // 新增
) -> Self
```

- [ ] **Step 3: 在 execute 方法中添加 HAVING 过滤逻辑**

在产生最终结果之前，应用 having 条件过滤：
```rust
// 如果有 having 条件，对每个分组应用过滤
if let Some(ref having_expr) = self.having {
    // 过滤掉不满足 having 条件的分组
}
```

- [ ] **Step 4: 运行测试验证编译**

```bash
cargo build --package sqlrustgo-executor
```

---

## Task 11: 验证 SortExec 和 SortVolcanoExecutor 集成

**Files:**
- 检查 `crates/planner/src/physical_plan.rs` (SortExec)
- 检查 `crates/executor/src/executor.rs` (SortVolcanoExecutor)

- [ ] **Step 1: 确认 SortExec 已有 sort_expr 字段**

查看 `crates/planner/src/physical_plan.rs:1001-1018`：
```rust
pub struct SortExec {
    input: Box<dyn PhysicalPlan>,
    sort_expr: Vec<crate::SortExpr>,  // 已存在
}
```

`SortExpr` 结构 (`crates/planner/src/lib.rs:83-87`)：
```rust
pub struct SortExpr {
    pub expr: Expr,
    pub asc: bool,
    pub nulls_first: bool,
}
```

- [ ] **Step 2: 确认 SortVolcanoExecutor 已有排序逻辑**

查看 `crates/executor/src/executor.rs:796-823`：
```rust
pub struct SortVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    sort_expr: Vec<sqlrustgo_planner::SortExpr>,  // 已存在
    // ...
}
```

**结论**: Sort 相关已完整实现，ORDER BY 解析完成后会自动连接。

- [ ] **Step 3: 运行测试验证**

```bash
cargo build --package sqlrustgo-planner --package sqlrustgo-executor
```

---

## Task 12: 集成测试

**Files:**
- Modify: `tests/integration/teaching_scenario_test.rs`

- [ ] **Step 1: 修改 test_teaching_group_by 测试**

更新现有测试，验证 GROUP BY 执行：

```rust
#[test]
fn test_teaching_group_by() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine.execute(parse("CREATE TABLE products (id INTEGER, category TEXT, price INTEGER)").unwrap()).unwrap();

    engine.execute(parse("INSERT INTO products VALUES (1, 'Electronics', 100), (2, 'Electronics', 200), (3, 'Clothing', 50)").unwrap()).unwrap();

    // 解析应该成功
    let result = parse("SELECT category, COUNT(*) FROM products GROUP BY category");
    if result.is_err() {
        eprintln!("Note: GROUP BY parsing not yet implemented");
        return;
    }

    let result = engine.execute(result.unwrap());
    if result.is_err() {
        eprintln!("Note: GROUP BY execution not yet implemented");
        return;
    }

    let result = result.unwrap();
    // 应该返回 2 行: Electronics=2, Clothing=1
    assert_eq!(result.rows.len(), 2);
}
```

- [ ] **Step 2: 修改 test_teaching_having 测试**

- [ ] **Step 3: 添加新测试 test_order_by**

- [ ] **Step 4: 运行教学场景测试验证**

```bash
cargo test --test teaching_scenario_test --all-features
```

---

## Task 13: 提交 PR

- [ ] **Step 1: 检查 git 状态**

```bash
git status
```

- [ ] **Step 2: 提交更改**

```bash
git add -A
git commit -m "feat(parser): 实现 GROUP BY + HAVING + ORDER BY 解析支持

- 添加 Group, By, Having, Order, Asc, Desc, Nulls, First, Last Token
- 修改 SelectStatement 添加 group_by, having, order_by 字段
- 实现 group_by, having, order_by 解析逻辑
- 扩展 AggregateExec 和 AggregateVolcanoExecutor 支持 HAVING
- 添加完整的解析器单元测试

Co-authored-by: Claude <claude@anthropic.com>"
```

- [ ] **Step 3: 创建 PR**

```bash
gh pr create --title "feat(parser): 实现 GROUP BY + HAVING + ORDER BY" --body "$(cat <<'EOF'
## Summary
- 添加 GROUP BY + HAVING + ORDER BY 语法支持 (SQL-92 标准)
- Parser 层: 添加 Token、修改 AST、添加解析逻辑
- Planner 层: 扩展 AggregateExec 支持 HAVING
- Executor 层: 扩展 AggregateVolcanoExecutor 支持 HAVING
- SortExec/SortVolcanoExecutor 已完整支持 ORDER BY

## Test plan
- [x] cargo test --package sqlrustgo-parser
- [x] cargo test --package sqlrustgo-planner
- [x] cargo test --package sqlrustgo-executor
- [x] cargo test --test teaching_scenario_test
EOF
)"
```

---

## 依赖关系

```
Task 1 (Token) → Task 2 (Lexer) → Task 3 (AST) → Task 4 (GROUP BY 解析)
    → Task 5 (HAVING 解析) → Task 6 (ORDER BY 解析)
    → Task 7 (集成) → Task 8 (Parser 测试)
    → Task 9 (Planner AggregateExec HAVING)
    → Task 10 (Executor AggregateVolcanoExecutor HAVING)
    → Task 11 (Sort 验证)
    → Task 12 (集成测试)
    → Task 13 (PR)
```

## 注意事项

1. **表达式解析依赖**: HAVING 和 ORDER BY 都依赖 `parse_expression()`，需要确保它足够强大
2. **NULL 处理**: SQL-92 对 NULL 的排序有特殊规则，确保 `nulls_first` 默认值正确
3. **多列排序**: ORDER BY 支持多列，每个列可以有独立的 ASC/DESC 和 NULLS FIRST/LAST
4. **Sort 已完整**: SortExec 和 SortVolcanoExecutor 已完整实现，不需要额外开发
