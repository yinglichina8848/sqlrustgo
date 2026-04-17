# SQL Corpus Parser 增强计划

> **目标**: 修复 SQL parser，使 sql-corpus 测试通过率从 20.3% 提升至 60%+

> **范围**: 仅修复 parser 层问题，不涉及 executor/planner（那些是 Hermes 的工作）

---

## 任务 1: 实现聚合函数解析 (COUNT, SUM, AVG, MIN, MAX)

**失败 case**: 14+ tests
**错误**: `Parse error: "Expected FROM or column name"`

**Files to Modify:**
- `crates/parser/src/expression.rs` - 添加函数调用解析
- `crates/parser/src/select.rs` - SELECT 语句解析
- `sql_corpus/DML/SELECT/aggregates.sql` - 测试用例

**Step 1: 理解当前解析器结构**
```bash
grep -n "Function" crates/parser/src/expression.rs | head -20
cat crates/parser/src/expression.rs | head -100
```

**Step 2: 添加函数调用解析**
```rust
// 在 expression.rs 中添加:
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionArg {
    Literal(String),
    Column(String),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<FunctionArg>,
    pub distinct: bool,
}
```

**Step 3: 修改 SELECT 解析**
```rust
// 在 select.rs 的 column_item 解析中:
"COUNT" => {
    if peek("(") {
        let args = parse_function_args()?;
        FunctionCall { name: "COUNT".into(), args, distinct: false }
    } else {
        ColumnName(...)
    }
}
```

**Step 4: 测试解析**
```bash
echo "SELECT COUNT(*) FROM users" | cargo run --bin sqlrustgo
echo "SELECT SUM(amount) FROM orders" | cargo run --bin sqlrustgo
```

**Step 5: 运行 aggregate 测试**
```bash
cargo test -p sqlrustgo-sql-corpus test_sql_corpus_aggregates
```

---

## 任务 2: 实现 JOIN 语法解析

**失败 case**: 14 tests
**错误**: `Parse error: "Expected FROM or column name"`

**Files to Modify:**
- `crates/parser/src/select.rs` - FROM 子句解析
- `crates/parser/src/join.rs` - 新建 join 解析模块

**Step 1: 查看当前 FROM 解析**
```bash
grep -n "FROM\|from" crates/parser/src/select.rs | head -30
```

**Step 2: 添加 JOIN 类型**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinClause {
    pub join_type: JoinType,
    pub table: TableReference,
    pub condition: Option<Expression>,  // ON condition
}
```

**Step 3: 修改 FROM 解析**
```rust
// FROM 解析支持:
// FROM table1
// FROM table1 INNER JOIN table2 ON condition
// FROM table1 LEFT JOIN table2 ON condition
```

**Step 4: 测试**
```bash
echo "SELECT * FROM a INNER JOIN b ON a.id = b.id" | cargo run --bin sqlrustgo
echo "SELECT * FROM a LEFT JOIN b ON a.id = b.id" | cargo run --bin sqlrustgo
```

**Step 5: 运行 join 测试**
```bash
cargo test -p sqlrustgo-sql-corpus test_sql_corpus_joins
```

---

## 任务 3: 实现 DELETE 语句解析

**失败 case**: 4 tests
**错误**: `Parse error: "Expected FROM or column name"`

**Files to Modify:**
- `crates/parser/src/statements.rs` - DELETE 语句
- `crates/parser/src/lib.rs` - 导出 DELETE

**Step 1: 查看现有语句解析**
```bash
grep -n "Insert\|Update\|Select" crates/parser/src/statements.rs | head -20
```

**Step 2: 添加 DELETE 解析**
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStatement {
    pub table: String,
    pub where_clause: Option<Expression>,
}
```

**Step 3: 在 parse_statement 中添加**
```rust
"DELETE" => {
    let table = parse_identifier()?;
    let where = if peek("WHERE") { Some(parse_where()?) } else { None };
    Statement::Delete(DeleteStatement { table, where_clause: where })
}
```

**Step 4: 测试**
```bash
echo "DELETE FROM users WHERE id = 1" | cargo run --bin sqlrustgo
echo "DELETE FROM users" | cargo run --bin sqlrustgo
```

---

## 任务 4: 实现 GROUP BY 和 HAVING 解析

**失败 case**: 5+ tests
**错误**: `Parse error: "Expected FROM or column name"`

**Files to Modify:**
- `crates/parser/src/select.rs` - SELECT 语句结构
- `crates/parser/src/group_by.rs` - 新建 group_by 解析模块

**Step 1: 添加 GROUP BY 结构**
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct GroupByClause {
    pub columns: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HavingClause {
    pub condition: Expression,
}
```

**Step 2: 修改 SELECT 语句**
```rust
pub struct SelectStatement {
    pub columns: Vec<SelectItem>,
    pub table: String,
    pub where_clause: Option<Expression>,
    pub group_by: Option<GroupByClause>,
    pub having: Option<HavingClause>,
    pub order_by: Option<OrderByClause>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}
```

**Step 3: 解析 GROUP BY**
```rust
// 在 SELECT 解析中添加:
if peek("GROUP") {
    consume("GROUP");
    consume("BY");
    let columns = parse_expression_list()?;
    group_by = Some(GroupByClause { columns });
}
```

**Step 4: 解析 HAVING**
```rust
if peek("HAVING") {
    consume("HAVING");
    let condition = parse_expression()?;
    having = Some(HavingClause { condition });
}
```

---

## 任务 5: 实现 BEGIN TRANSACTION 语法

**失败 case**: 7 tests
**错误**: `Parse error: "Expected FROM or column name"`

**Files to Modify:**
- `crates/parser/src/statements.rs` - 事务语句

**Step 1: 添加事务语句类型**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatement {
    Begin,
    Commit,
    Rollback,
    Savepoint(String),
    ReleaseSavepoint(String),
}
```

**Step 2: 解析 BEGIN TRANSACTION**
```rust
"BEGIN" | "BEGIN WORK" | "BEGIN TRANSACTION" => {
    Statement::Transaction(TransactionStatement::Begin)
}
"COMMIT" | "COMMIT WORK" => {
    Statement::Transaction(TransactionStatement::Commit)
}
"ROLLBACK" | "ROLLBACK WORK" => {
    Statement::Transaction(TransactionStatement::Rollback)
}
```

---

## 任务 6: 修复外键约束解析

**失败 case**: 3 tests
**错误**: `Parse error: "Expected Table, got Index"`

**Files to Modify:**
- `crates/parser/src/create_table.rs` - 表约束解析

**Step 1: 查看错误位置**
```bash
grep -n "Index" crates/parser/src/create_table.rs | head -20
```

**Step 2: 修复 FOREIGN KEY 解析**
```rust
// 确保 FOREIGN KEY 解析正确返回 TableReference 而非 Index
```

---

## 任务 7: 实现 CREATE INDEX 解析（如果缺失）

**失败 case**: 3 tests

**Files to Modify:**
- `crates/parser/src/statements.rs` - CREATE INDEX

---

## 依赖关系

```
任务 1 (聚合函数)
    ↓
任务 4 (GROUP BY) ← 需要任务 1 的函数解析
    ↓
任务 2 (JOIN) ← JOIN 可能包含聚合
    ↓
任务 3 (DELETE)
    ↓
任务 5 (事务)
    ↓
任务 6 (外键)
    ↓
任务 7 (INDEX)
```

## 验收标准

每个任务完成后：
1. 对应 sql-corpus 测试从 FAIL 变为 PASS
2. Parser 测试全部通过：`cargo test -p sqlrustgo-parser --lib`
3. 提交代码

## 预期结果

| 任务 | 修复测试数 | 累计通过率 |
|------|-----------|------------|
| 任务 1 (聚合函数) | +7 | 32% |
| 任务 2 (JOIN) | +14 | 44% |
| 任务 3 (DELETE) | +4 | 51% |
| 任务 4 (GROUP BY) | +5 | 60% |
| 任务 5 (事务) | +7 | 72% |
| 任务 6 (外键) | +3 | 77% |
| 任务 7 (INDEX) | +3 | 82% |

**目标: sql-corpus 通过率达到 60%+ (当前 20.3%)**
