# SQLRustGo 功能补全实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**目标:** 补全 SQLRustGo 缺失的核心功能，实现完整的 DML 操作和数据存储

**架构:** 在现有 ExecutionEngine 基础上添加行存储和表达式求值，采用内存 HashMap 存储表数据

**技术栈:** Rust 2024 Edition, HashMap 存储, 表达式求值

---

## 阶段一：数据存储基础

### Task 1: 添加行存储结构

**文件:**
- Modify: `src/executor/mod.rs:17-30` - 添加 rows 字段

**Step 1: 修改 ExecutionEngine 添加行存储**

```rust
// 在 ExecutionEngine 结构体中，将 tables 改为存储实际数据
pub struct ExecutionEngine {
    buffer_pool: BufferPool,
    // 存储表名 -> 表数据(列定义 + 行数据)
    tables: std::collections::HashMap<String, TableData>,
}

pub struct TableData {
    pub info: TableInfo,
    pub rows: Vec<Vec<Value>>,  // 实际行数据
}
```

**Step 2: 运行测试验证**

Run: `cargo test --lib`
Expected: PASS (现有测试应该仍然通过)

**Step 3: 提交**

```bash
git add src/executor/mod.rs
git commit -m "feat: 添加行存储结构到 ExecutionEngine"
```

---

### Task 2: 实现表达式求值

**文件:**
- Modify: `src/executor/mod.rs` - 添加 evaluate_expression 函数

**Step 1: 添加表达式求值函数**

```rust
/// Evaluate an expression to a Value
fn evaluate_expression(&self, expr: &Expression) -> SqlResult<Value> {
    match expr {
        Expression::Literal(s) => {
            // 尝试解析为数字或布尔值
            if let Ok(n) = s.parse::<i64>() {
                Ok(Value::Integer(n))
            } else if let Ok(f) = s.parse::<f64>() {
                Ok(Value::Float(f))
            } else if s.to_lowercase() == "true" {
                Ok(Value::Boolean(true))
            } else if s.to_lowercase() == "false" {
                Ok(Value::Boolean(false))
            } else {
                Ok(Value::Text(s.clone()))
            }
        }
        Expression::Identifier(name) => {
            // 标识符求值需要在具体行的上下文中进行
            Err(SqlError::ExecutionError(format!("Cannot evaluate identifier: {}", name)))
        }
        Expression::BinaryOp(left, op, right) => {
            let left_val = self.evaluate_expression(left)?;
            let right_val = self.evaluate_expression(right)?;
            self.eval_binary_op(&left_val, op, &right_val)
        }
    }
}

fn eval_binary_op(&self, left: &Value, op: &str, right: &Value) -> SqlResult<Value> {
    match op {
        "=" => Ok(Value::Boolean(left == right)),
        "!=" | "<>" => Ok(Value::Boolean(left != right)),
        ">" => {
            if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                Ok(Value::Boolean(l > r))
            } else {
                Err(SqlError::TypeMismatch("Cannot compare".to_string()))
            }
        }
        "<" => {
            if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                Ok(Value::Boolean(l < r))
            } else {
                Err(SqlError::TypeMismatch("Cannot compare".to_string()))
            }
        }
        ">=" => {
            if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                Ok(Value::Boolean(l >= r))
            } else {
                Err(SqlError::TypeMismatch("Cannot compare".to_string()))
            }
        }
        "<=" => {
            if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                Ok(Value::Boolean(l <= r))
            } else {
                Err(SqlError::TypeMismatch("Cannot compare".to_string()))
            }
        }
        _ => Err(SqlError::ExecutionError(format!("Unknown operator: {}", op))),
    }
}
```

**Step 2: 添加测试**

```rust
#[test]
fn test_evaluate_literal_integer() {
    let engine = ExecutionEngine::new();
    let expr = Expression::Literal("42".to_string());
    let result = engine.evaluate_expression(&expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(42));
}

#[test]
fn test_evaluate_literal_text() {
    let engine = ExecutionEngine::new();
    let expr = Expression::Literal("hello".to_string());
    let result = engine.evaluate_expression(&expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Text("hello".to_string()));
}

#[test]
fn test_evaluate_binary_op() {
    let engine = ExecutionEngine::new();
    let expr = Expression::BinaryOp(
        Box::new(Expression::Literal("10".to_string())),
        ">".to_string(),
        Box::new(Expression::Literal("5".to_string())),
    );
    let result = engine.evaluate_expression(&expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}
```

**Step 3: 运行测试**

Run: `cargo test --lib evaluate`
Expected: PASS

**Step 4: 提交**

```bash
git add src/executor/mod.rs
git commit -m "feat: 实现表达式求值功能"
```

---

### Task 3: 实现 WHERE 子句过滤

**文件:**
- Modify: `src/executor/mod.rs` - 添加 evaluate_where 函数

**Step 1: 添加 WHERE 条件过滤函数**

```rust
/// Filter rows based on WHERE clause
fn filter_by_where(&self, rows: &[Vec<Value>], where_clause: &Option<Expression>, columns: &[String]) -> SqlResult<Vec<Vec<Value>>> {
    match where_clause {
        None => Ok(rows.to_vec()),
        Some(expr) => {
            let mut result = Vec::new();
            for row in rows {
                if self.row_matches_where(row, expr, columns)? {
                    result.push(row.clone());
                }
            }
            Ok(result)
        }
    }
}

/// Check if a row matches the WHERE condition
fn row_matches_where(&self, row: &Vec<Value>, expr: &Expression, columns: &[String]) -> SqlResult<bool> {
    // 将列名映射到列索引
    let col_index = |name: &str| {
        columns.iter().position(|c| c == name)
    };

    // 创建上下文用于求值
    let eval_with_context = |expr: &Expression| -> SqlResult<Value> {
        match expr {
            Expression::Identifier(name) => {
                if let Some(idx) = col_index(name) {
                    Ok(row.get(idx).cloned().unwrap_or(Value::Null))
                } else {
                    Err(SqlError::ExecutionError(format!("Unknown column: {}", name)))
                }
            }
            Expression::Literal(s) => self.evaluate_expression(expr),
            Expression::BinaryOp(left, op, right) => {
                let left_val = eval_with_context(left)?;
                let right_val = eval_with_context(right)?;
                self.eval_binary_op(&left_val, op, &right_val)
            }
        }
    };

    let result = eval_with_context(expr)?;
    match result {
        Value::Boolean(b) => Ok(b),
        Value::Null => Ok(false),
        _ => Ok(true),  // 非空非布尔值视为 true
    }
}
```

**Step 2: 添加测试**

```rust
#[test]
fn test_filter_by_where() {
    let engine = ExecutionEngine::new();
    let rows = vec![
        vec![Value::Integer(1), Value::Text("Alice".to_string())],
        vec![Value::Integer(2), Value::Text("Bob".to_string())],
        vec![Value::Integer(3), Value::Text("Charlie".to_string())],
    ];
    let columns = vec!["id".to_string(), "name".to_string()];

    // WHERE id > 1
    let expr = Expression::BinaryOp(
        Box::new(Expression::Identifier("id".to_string())),
        ">".to_string(),
        Box::new(Expression::Literal("1".to_string())),
    );
    let result = engine.filter_by_where(&rows, &Some(expr), &columns);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
}
```

**Step 3: 运行测试**

Run: `cargo test --lib filter_by_where`
Expected: PASS

**Step 4: 提交**

```bash
git add src/executor/mod.rs
git commit -m "feat: 实现 WHERE 子句过滤"
```

---

## 阶段二：DML 执行

### Task 4: 完成 INSERT 执行

**文件:**
- Modify: `src/executor/mod.rs:59-72` - 重写 execute_insert

**Step 1: 修改 INSERT 执行逻辑**

```rust
fn execute_insert(&mut self, stmt: InsertStatement) -> SqlResult<ExecutionResult> {
    // 获取表数据
    let table_data = self.tables.get_mut(&stmt.table)
        .ok_or_else(|| SqlError::TableNotFound(stmt.table.clone()))?;

    // 如果没有提供列名，使用表的所有列
    let columns = if stmt.columns.is_empty() {
        table_data.info.columns.iter().map(|c| c.name.clone()).collect()
    } else {
        stmt.columns.clone()
    };

    // 将表达式值转换为实际行数据
    for expr in &stmt.values {
        let row = self.expression_to_row(expr, &columns)?;
        table_data.rows.push(row);
    }

    Ok(ExecutionResult {
        rows_affected: stmt.values.len() as u64,
        columns: Vec::new(),
        rows: Vec::new(),
    })
}

/// Convert expression to row values
fn expression_to_row(&self, expr: &Expression, columns: &[String]) -> SqlResult<Vec<Value>> {
    match expr {
        Expression::Literal(s) => {
            // 尝试自动类型转换
            if let Ok(n) = s.parse::<i64>() {
                Ok(vec![Value::Integer(n)])
            } else if let Ok(f) = s.parse::<f64>() {
                Ok(vec![Value::Float(f)])
            } else if s.to_lowercase() == "true" {
                Ok(vec![Value::Boolean(true)])
            } else if s.to_lowercase() == "false" {
                Ok(vec![Value::Boolean(false)])
            } else {
                Ok(vec![Value::Text(s.clone())])
            }
        }
        _ => Err(SqlError::ExecutionError("Unsupported expression in VALUES".to_string())),
    }
}
```

**Step 2: 修改 TableData 结构**

```rust
pub struct TableData {
    pub info: TableInfo,
    pub rows: Vec<Vec<Value>>,
}

// 修改 get_table 返回 TableData
pub fn get_table(&self, name: &str) -> Option<&TableData> {
    self.tables.get(name)
}
```

**Step 3: 添加测试**

```rust
#[test]
fn test_execute_insert_with_data() {
    let mut engine = ExecutionEngine::new();
    // Create table with columns
    let _ = engine.execute(crate::parser::parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap());

    // Insert data
    let result = engine.execute(crate::parser::parse("INSERT INTO users VALUES (1, 'Alice')").unwrap());
    assert_eq!(result.rows_affected, 1);

    // Verify data was stored
    let table = engine.get_table("users").unwrap();
    assert_eq!(table.rows.len(), 1);
    assert_eq!(table.rows[0][0], Value::Integer(1));
}
```

**Step 4: 运行测试**

Run: `cargo test --lib insert`
Expected: PASS

**Step 5: 提交**

```bash
git add src/executor/mod.rs
git commit -m "feat: 完成 INSERT 执行，存储实际行数据"
```

---

### Task 5: 实现 UPDATE 执行

**文件:**
- Modify: `src/executor/mod.rs:37` - 修改 execute 函数处理 Update
- Add: `src/executor/mod.rs` - 添加 execute_update 方法

**Step 1: 添加 UPDATE 执行方法**

```rust
fn execute_update(&mut self, stmt: UpdateStatement) -> SqlResult<ExecutionResult> {
    // 获取表数据
    let table_data = self.tables.get_mut(&stmt.table)
        .ok_or_else(|| SqlError::TableNotFound(stmt.table.clone()))?;

    let columns: Vec<String> = table_data.info.columns.iter().map(|c| c.name.clone()).collect();
    let mut rows_affected = 0;

    // 遍历所有行
    for row in &mut table_data.rows {
        // 检查 WHERE 条件
        if let Some(ref where_clause) = stmt.where_clause {
            if !self.row_matches_where(row, where_clause, &columns)? {
                continue;
            }
        }

        // 应用 SET 更新
        for (col_name, new_value_expr) in &stmt.set_clauses {
            if let Some(col_idx) = columns.iter().position(|c| c == col_name) {
                let new_value = self.evaluate_expression(new_value_expr)?;
                if col_idx < row.len() {
                    row[col_idx] = new_value;
                    rows_affected += 1;
                }
            }
        }
    }

    Ok(ExecutionResult {
        rows_affected,
        columns: Vec::new(),
        rows: Vec::new(),
    })
}
```

**Step 2: 更新 execute 函数**

```rust
pub fn execute(&mut self, statement: Statement) -> SqlResult<ExecutionResult> {
    match statement {
        Statement::Select(s) => self.execute_select(s),
        Statement::Insert(s) => self.execute_insert(s),
        Statement::Update(s) => self.execute_update(s),
        Statement::Delete(s) => self.execute_delete(s),
        // ...
    }
}
```

**Step 3: 添加测试**

```rust
#[test]
fn test_execute_update() {
    let mut engine = ExecutionEngine::new();
    let _ = engine.execute(crate::parser::parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap());
    let _ = engine.execute(crate::parser::parse("INSERT INTO users VALUES (1, 'Alice')").unwrap());
    let _ = engine.execute(crate::parser::parse("INSERT INTO users VALUES (2, 'Bob')").unwrap());

    // Update
    let result = engine.execute(
        crate::parser::parse("UPDATE users SET name = 'Charlie' WHERE id = 1").unwrap()
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().rows_affected, 1);

    // Verify
    let table = engine.get_table("users").unwrap();
    assert_eq!(table.rows[0][1], Value::Text("Charlie".to_string()));
}
```

**Step 4: 运行测试**

Run: `cargo test --lib update`
Expected: PASS

**Step 5: 提交**

```bash
git add src/executor/mod.rs
git commit -m "feat: 实现 UPDATE 执行"
```

---

### Task 6: 实现 DELETE 执行

**文件:**
- Modify: `src/executor/mod.rs` - 添加 execute_delete 方法

**Step 1: 添加 DELETE 执行方法**

```rust
fn execute_delete(&mut self, stmt: DeleteStatement) -> SqlResult<ExecutionResult> {
    // 获取表数据
    let table_data = self.tables.get_mut(&stmt.table)
        .ok_or_else(|| SqlError::TableNotFound(stmt.table.clone()))?;

    let columns: Vec<String> = table_data.info.columns.iter().map(|c| c.name.clone()).collect();

    // 如果没有 WHERE 子句，删除所有行
    if stmt.where_clause.is_none() {
        let count = table_data.rows.len() as u64;
        table_data.rows.clear();
        return Ok(ExecutionResult {
            rows_affected: count,
            columns: Vec::new(),
            rows: Vec::new(),
        });
    }

    // 否则过滤删除
    let where_clause = stmt.where_clause.as_ref().unwrap();
    let original_len = table_data.rows.len();

    table_data.rows.retain(|row| {
        !self.row_matches_where(row, where_clause, &columns).unwrap_or(true)
    });

    let deleted = (original_len - table_data.rows.len()) as u64;

    Ok(ExecutionResult {
        rows_affected: deleted,
        columns: Vec::new(),
        rows: Vec::new(),
    })
}
```

**Step 2: 添加测试**

```rust
#[test]
fn test_execute_delete() {
    let mut engine = ExecutionEngine::new();
    let _ = engine.execute(crate::parser::parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap());
    let _ = engine.execute(crate::parser::parse("INSERT INTO users VALUES (1, 'Alice')").unwrap());
    let _ = engine.execute(crate::parser::parse("INSERT INTO users VALUES (2, 'Bob')").unwrap());

    // Delete
    let result = engine.execute(
        crate::parser::parse("DELETE FROM users WHERE id = 1").unwrap()
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().rows_affected, 1);

    // Verify
    let table = engine.get_table("users").unwrap();
    assert_eq!(table.rows.len(), 1);
}
```

**Step 3: 运行测试**

Run: `cargo test --lib delete`
Expected: PASS

**Step 4: 提交**

```bash
git add src/executor/mod.rs
git commit -m "feat: 实现 DELETE 执行"
```

---

## 阶段三：增强解析器

### Task 7: 解析 INSERT VALUES

**文件:**
- Modify: `src/parser/mod.rs:182-196` - 完善 parse_insert

**Step 1: 解析 VALUES 子句**

```rust
fn parse_insert(&mut self) -> Result<Statement, String> {
    self.expect(Token::Insert)?;
    self.expect(Token::Into)?;

    let table = match self.next() {
        Some(Token::Identifier(name)) => name,
        _ => return Err("Expected table name".to_string()),
    };

    // 解析列名列表 (可选)
    let mut columns = Vec::new();
    if let Some(Token::LParen) = self.current() {
        self.next(); // consume '('
        loop {
            match self.current() {
                Some(Token::Identifier(name)) => {
                    columns.push(name.clone());
                    self.next();
                    match self.current() {
                        Some(Token::Comma) => { self.next(); }
                        Some(Token::RParen) => { self.next(); break; }
                        _ => return Err("Expected , or )".to_string()),
                    }
                }
                _ => break,
            }
        }
    }

    // 解析 VALUES
    self.expect(Token::Values)?;
    self.expect(Token::LParen)?;

    let mut values = Vec::new();
    loop {
        match self.current() {
            Some(Token::Number(n)) => {
                values.push(Expression::Literal(n.clone()));
                self.next();
            }
            Some(Token::String(s)) => {
                values.push(Expression::Literal(s.clone()));
                self.next();
            }
            Some(Token::RParen) => {
                self.next();
                break;
            }
            Some(Token::Comma) => {
                self.next();
            }
            _ => return Err("Expected value".to_string()),
        }
    }

    Ok(Statement::Insert(InsertStatement {
        table,
        columns,
        values,
    }))
}
```

**Step 2: 添加测试**

```rust
#[test]
fn test_parse_insert_values() {
    let result = parse("INSERT INTO users (id, name) VALUES (1, 'Alice')");
    assert!(result.is_ok());
    match result.unwrap() {
        Statement::Insert(i) => {
            assert_eq!(i.table, "users");
            assert_eq!(i.columns.len(), 2);
            assert_eq!(i.values.len(), 2);
        }
        _ => panic!("Expected INSERT"),
    }
}
```

**Step 3: 运行测试**

Run: `cargo test --lib parse_insert`
Expected: PASS

**Step 4: 提交**

```bash
git add src/parser/mod.rs
git commit -m "feat: 解析 INSERT VALUES 子句"
```

---

### Task 8: 解析 UPDATE SET

**文件:**
- Modify: `src/parser/mod.rs:198-210` - 完善 parse_update

**Step 1: 解析 SET 子句**

```rust
fn parse_update(&mut self) -> Result<Statement, String> {
    self.expect(Token::Update)?;
    let table = match self.next() {
        Some(Token::Identifier(name)) => name,
        _ => return Err("Expected table name".to_string()),
    };

    self.expect(Token::Set)?;

    // 解析 column = value 对
    let mut set_clauses = Vec::new();
    loop {
        match self.current() {
            Some(Token::Identifier(col_name)) => {
                let col = col_name.clone();
                self.next();
                // 跳过 =
                if let Some(Token::Equal) = self.current() {
                    self.next();
                }
                // 解析值
                let value = match self.current() {
                    Some(Token::Number(n)) => Expression::Literal(n.clone()),
                    Some(Token::String(s)) => Expression::Literal(s.clone()),
                    Some(Token::Identifier(id)) => Expression::Identifier(id.clone()),
                    _ => return Err("Expected value".to_string()),
                };
                set_clauses.push((col, value));
                self.next();

                // 检查是否有更多 SET 子句或 WHERE
                match self.current() {
                    Some(Token::Comma) => { self.next(); continue; }
                    Some(Token::Where) => { break; }
                    Some(Token::Eof) => { break; }
                    _ => { break; }
                }
            }
            _ => break,
        }
    }

    // 解析 WHERE 子句 (可选)
    let where_clause = if self.match_token(Token::Where) {
        Some(self.parse_where_clause()?)
    } else {
        None
    };

    Ok(Statement::Update(UpdateStatement {
        table,
        set_clauses,
        where_clause,
    }))
}

fn parse_where_clause(&mut self) -> Result<Expression, String> {
    // 简单解析: column op value
    let left = match self.current() {
        Some(Token::Identifier(name)) => {
            let n = name.clone();
            self.next();
            Expression::Identifier(n)
        }
        _ => return Err("Expected column name".to_string()),
    };

    let op = match self.current() {
        Some(Token::Equal) => "=".to_string(),
        Some(Token::NotEqual) => "!=".to_string(),
        Some(Token::Greater) => ">".to_string(),
        Some(Token::Less) => "<".to_string(),
        _ => return Err("Expected operator".to_string()),
    };
    self.next();

    let right = match self.current() {
        Some(Token::Number(n)) => Expression::Literal(n.clone()),
        Some(Token::String(s)) => Expression::Literal(s.clone()),
        _ => return Err("Expected value".to_string()),
    };
    self.next();

    Ok(Expression::BinaryOp(Box::new(left), op, Box::new(right)))
}
```

**Step 2: 添加 Token 变体**

需要在 `lexer/token.rs` 中添加:
- NotEqual
- Greater
- Less

**Step 3: 添加测试**

```rust
#[test]
fn test_parse_update_set() {
    let result = parse("UPDATE users SET name = 'Bob' WHERE id = 1");
    assert!(result.is_ok());
    match result.unwrap() {
        Statement::Update(u) => {
            assert_eq!(u.table, "users");
            assert_eq!(u.set_clauses.len(), 1);
            assert!(u.where_clause.is_some());
        }
        _ => panic!("Expected UPDATE"),
    }
}
```

**Step 4: 运行测试**

Run: `cargo test --lib parse_update`
Expected: PASS

**Step 5: 提交**

```bash
git add src/parser/mod.rs src/lexer/token.rs
git commit -m "feat: 解析 UPDATE SET 和 WHERE 子句"
```

---

### Task 9: 解析 WHERE 子句

**文件:**
- Modify: `src/parser/mod.rs:138-180` - 完善 SELECT 解析

**Step 1: 解析 SELECT WHERE**

```rust
fn parse_select(&mut self) -> Result<Statement, String> {
    self.expect(Token::Select)?;

    // ... existing column parsing ...

    self.expect(Token::From)?;

    let table = match self.next() {
        Some(Token::Identifier(name)) => name,
        Some(t) => return Err(format!("Expected table name, got {:?}", t)),
        None => return Err("Expected table name".to_string()),
    };

    // 解析 WHERE 子句
    let where_clause = if self.match_token(Token::Where) {
        Some(self.parse_where_clause()?)
    } else {
        None
    };

    Ok(Statement::Select(SelectStatement {
        columns,
        table,
        where_clause,
    }))
}
```

**Step 2: 解析 DELETE WHERE**

```rust
fn parse_delete(&mut self) -> Result<Statement, String> {
    self.expect(Token::Delete)?;
    self.expect(Token::From)?;
    let table = match self.next() {
        Some(Token::Identifier(name)) => name,
        _ => return Err("Expected table name".to_string()),
    };

    // 解析 WHERE 子句
    let where_clause = if self.match_token(Token::Where) {
        Some(self.parse_where_clause()?)
    } else {
        None
    };

    Ok(Statement::Delete(DeleteStatement {
        table,
        where_clause,
    }))
}
```

**Step 3: 添加测试**

```rust
#[test]
fn test_parse_select_where() {
    let result = parse("SELECT id, name FROM users WHERE id > 10");
    assert!(result.is_ok());
    match result.unwrap() {
        Statement::Select(s) => {
            assert_eq!(s.table, "users");
            assert!(s.where_clause.is_some());
        }
        _ => panic!("Expected SELECT"),
    }
}

#[test]
fn test_parse_delete_where() {
    let result = parse("DELETE FROM users WHERE id = 1");
    assert!(result.is_ok());
    match result.unwrap() {
        Statement::Delete(d) => {
            assert_eq!(d.table, "users");
            assert!(d.where_clause.is_some());
        }
        _ => panic!("Expected DELETE"),
    }
}
```

**Step 4: 运行测试**

Run: `cargo test --lib where`
Expected: PASS

**Step 5: 提交**

```bash
git add src/parser/mod.rs
git commit -m "feat: 解析 SELECT/DELETE WHERE 子句"
```

---

## 阶段四：SELECT 返回实际数据

### Task 10: SELECT 返回行数据

**文件:**
- Modify: `src/executor/mod.rs:44-57` - 重写 execute_select

**Step 1: 修改 SELECT 执行**

```rust
fn execute_select(&mut self, stmt: SelectStatement) -> SqlResult<ExecutionResult> {
    // 获取表数据
    let table_data = self.tables.get(&stmt.table)
        .ok_or_else(|| SqlError::TableNotFound(stmt.table.clone()))?;

    let table_columns: Vec<String> = table_data.info.columns.iter().map(|c| c.name.clone()).collect();

    // 获取请求的列索引
    let mut result_columns: Vec<String> = Vec::new();
    let mut col_indices: Vec<usize> = Vec::new();

    for col in &stmt.columns {
        if col.name == "*" {
            // SELECT * - 返回所有列
            result_columns = table_columns.clone();
            col_indices = (0..table_columns.len()).collect();
            break;
        } else {
            if let Some(idx) = table_columns.iter().position(|c| c == &col.name) {
                result_columns.push(col.name.clone());
                col_indices.push(idx);
            }
        }
    }

    // 过滤行 (WHERE)
    let filtered_rows = self.filter_by_where(&table_data.rows, &stmt.where_clause, &table_columns)?;

    // 投影到请求的列
    let result_rows: Vec<Vec<Value>> = filtered_rows.iter()
        .map(|row| {
            col_indices.iter().map(|&idx| row.get(idx).cloned().unwrap_or(Value::Null)).collect()
        })
        .collect();

    Ok(ExecutionResult {
        rows_affected: result_rows.len() as u64,
        columns: result_columns,
        rows: result_rows,
    }))
}
```

**Step 2: 修改 ExecutionResult 打印**

需要在 main.rs 中添加打印结果行的逻辑。

**Step 3: 添加测试**

```rust
#[test]
fn test_select_returns_data() {
    let mut engine = ExecutionEngine::new();
    let _ = engine.execute(crate::parser::parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap());
    let _ = engine.execute(crate::parser::parse("INSERT INTO users VALUES (1, 'Alice')").unwrap());
    let _ = engine.execute(crate::parser::parse("INSERT INTO users VALUES (2, 'Bob')").unwrap());

    let result = engine.execute(crate::parser::parse("SELECT * FROM users").unwrap());
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0][0], Value::Integer(1));
}

#[test]
fn test_select_with_where() {
    let mut engine = ExecutionEngine::new();
    let _ = engine.execute(crate::parser::parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap());
    let _ = engine.execute(crate::parser::parse("INSERT INTO users VALUES (1, 'Alice')").unwrap());
    let _ = engine.execute(crate::parser::parse("INSERT INTO users VALUES (2, 'Bob')").unwrap());

    let result = engine.execute(crate::parser::parse("SELECT * FROM users WHERE id > 1").unwrap());
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][1], Value::Text("Bob".to_string()));
}
```

**Step 4: 运行测试**

Run: `cargo test --lib select`
Expected: PASS

**Step 5: 提交**

```bash
git add src/executor/mod.rs
git commit -m "feat: SELECT 返回实际行数据"
```

---

## 阶段五：集成测试

### Task 11: 端到端功能测试

**文件:**
- Modify: `tests/integration_test.rs`

**Step 1: 添加完整流程测试**

```rust
#[test]
fn test_full_dml_workflow() {
    let mut engine = ExecutionEngine::new();

    // CREATE TABLE
    let result = engine.execute(parser::parse("CREATE TABLE users (id INTEGER, name TEXT, age INTEGER)").unwrap());
    assert!(result.is_ok());

    // INSERT multiple rows
    let result = engine.execute(parser::parse("INSERT INTO users VALUES (1, 'Alice', 25)").unwrap());
    assert_eq!(result.unwrap().rows_affected, 1);

    let result = engine.execute(parser::parse("INSERT INTO users VALUES (2, 'Bob', 30)").unwrap());
    assert_eq!(result.unwrap().rows_affected, 1);

    let result = engine.execute(parser::parse("INSERT INTO users VALUES (3, 'Charlie', 35)").unwrap());
    assert_eq!(result.unwrap().rows_affected, 1);

    // SELECT all
    let result = engine.execute(parser::parse("SELECT * FROM users").unwrap());
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.rows.len(), 3);

    // SELECT with WHERE
    let result = engine.execute(parser::parse("SELECT name FROM users WHERE age > 28").unwrap());
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.rows.len(), 2);

    // UPDATE
    let result = engine.execute(parser::parse("UPDATE users SET age = 26 WHERE id = 1").unwrap());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().rows_affected, 1);

    // Verify update
    let result = engine.execute(parser::parse("SELECT age FROM users WHERE id = 1").unwrap());
    let result = result.unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(26));

    // DELETE
    let result = engine.execute(parser::parse("DELETE FROM users WHERE id = 3").unwrap());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().rows_affected, 1);

    // Verify delete
    let result = engine.execute(parser::parse("SELECT * FROM users").unwrap());
    assert_eq!(result.unwrap().rows.len(), 2);
}
```

**Step 2: 运行集成测试**

Run: `cargo test --test integration_test`
Expected: PASS

**Step 3: 提交**

```bash
git add tests/integration_test.rs
git commit -m "test: 添加完整 DML 工作流集成测试"
```

---

## 计划完成

**计划完成，保存于 `docs/plans/2026-02-14-sqlrustgo-feature-completion.md`**

---

## 执行选项

**1. Subagent-Driven（本会话）** - 我在本会话中分派子代理逐个任务执行，中间进行代码审查

**2. Parallel Session（新会话）** - 在新会话中使用 executing-plans，分批执行并设置检查点

选择哪种方式？
