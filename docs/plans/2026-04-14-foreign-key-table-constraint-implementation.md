# FOREIGN KEY 表级约束实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 SQL 标准表级 FOREIGN KEY (...) REFERENCES 语法，支持 multi-column FK

**Architecture:** 在 Parser 层新增 TableConstraint 枚举，扩展 CreateTableStatement 支持 constraints 字段。解析时将表级约束与列级约束分开处理，最终都转换为 Catalog 层 ForeignKeyRef 存储。

**Tech Stack:** Rust, sqlrustgo-parser crate

---

## 任务 1: 新增 TableConstraint 枚举

**Files:**
- Modify: `crates/parser/src/parser.rs:570-580` (ForeignKeyAction 之后)

**Step 1: 添加 TableConstraint 枚举**

在 `ForeignKeyAction` 枚举后添加：

```rust
/// Table-level constraints for CREATE TABLE
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TableConstraint {
    /// PRIMARY KEY (col1, col2, ...)
    PrimaryKey {
        columns: Vec<String>,
    },
    /// FOREIGN KEY (col1, col2, ...) REFERENCES table(col1, col2, ...)
    ForeignKey {
        columns: Vec<String>,
        reference_table: String,
        reference_columns: Vec<String>,
        on_delete: Option<ForeignKeyAction>,
        on_update: Option<ForeignKeyAction>,
    },
    /// UNIQUE (col1, col2, ...)
    Unique {
        columns: Vec<String>,
    },
}
```

**Step 2: 验证代码**

Run: `cargo build -p sqlrustgo-parser 2>&1 | head -20`
Expected: 无编译错误

**Step 3: Commit**

```bash
git add crates/parser/src/parser.rs
git commit -m "feat(parser): add TableConstraint enum for table-level constraints"
```

---

## 任务 2: 扩展 CreateTableStatement

**Files:**
- Modify: `crates/parser/src/parser.rs:533-537` (CreateTableStatement struct)

**Step 1: 修改 CreateTableStatement**

将：
```rust
pub struct CreateTableStatement {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    pub if_not_exists: bool,
}
```

修改为：
```rust
pub struct CreateTableStatement {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    pub if_not_exists: bool,
    pub constraints: Vec<TableConstraint>,  // 新增
}
```

**Step 2: 验证代码**

Run: `cargo build -p sqlrustgo-parser 2>&1 | head -20`
Expected: 编译错误 - 因为 parse_create_table 还没返回 constraints 字段

**Step 3: Commit**

```bash
git add crates/parser/src/parser.rs
git commit -m "feat(parser): extend CreateTableStatement with constraints field"
```

---

## 任务 3: 修改 parse_create_table 返回 constraints

**Files:**
- Modify: `crates/parser/src/parser.rs:2823-2827` (parse_create_table 返回值)

**Step 1: 修改返回值**

将：
```rust
Ok(Statement::CreateTable(CreateTableStatement {
    name,
    columns,
    if_not_exists,
}))
```

修改为：
```rust
Ok(Statement::CreateTable(CreateTableStatement {
    name,
    columns,
    if_not_exists,
    constraints: Vec::new(),  // 暂时为空，后续任务填充
}))
```

**Step 2: 验证编译**

Run: `cargo build -p sqlrustgo-parser 2>&1 | tail -10`
Expected: 编译成功

**Step 3: Commit**

```bash
git add crates/parser/src/parser.rs
git commit -m "feat(parser): return empty constraints in CreateTableStatement"
```

---

## 任务 4: 实现 parse_table_constraint 方法

**Files:**
- Modify: `crates/parser/src/parser.rs` (新增方法)

**Step 1: 在适当位置添加 parse_table_constraint 方法**

在 `parse_create_table` 方法之后（约 line 2830）添加：

```rust
/// Parse table-level constraints: FOREIGN KEY, PRIMARY KEY, UNIQUE
fn parse_table_constraint(&mut self) -> Result<TableConstraint, String> {
    match self.current() {
        Some(Token::Foreign) => {
            self.next(); // consume FOREIGN
            self.expect(Token::Key)?;
            // Parse: FOREIGN KEY (col1, col2, ...) REFERENCES table(col1, col2, ...)
            // Parse column list
            self.expect(Token::LParen)?;
            let columns = self.parse_identifier_list()?;
            self.expect(Token::RParen)?;
            self.expect(Token::References)?;

            // Parse reference table
            let reference_table = match self.current() {
                Some(Token::Identifier(s)) => {
                    let t = s.clone();
                    self.next();
                    t
                }
                _ => return Err("Expected reference table name".to_string()),
            };

            // Parse reference column list
            let reference_columns = if matches!(self.current(), Some(Token::LParen)) {
                self.next();
                let cols = self.parse_identifier_list()?;
                self.expect(Token::RParen)?;
                cols
            } else {
                vec!["id".to_string()] // 默认主键列
            };

            // Parse ON DELETE/UPDATE
            let on_delete = self.parse_fk_action(Token::Delete)?;
            let on_update = self.parse_fk_action(Token::Update)?;

            Ok(TableConstraint::ForeignKey {
                columns,
                reference_table,
                reference_columns,
                on_delete,
                on_update,
            })
        }
        Some(Token::Primary) => {
            self.next(); // consume PRIMARY
            self.expect(Token::Key)?;
            self.expect(Token::LParen)?;
            let columns = self.parse_identifier_list()?;
            self.expect(Token::RParen)?;
            Ok(TableConstraint::PrimaryKey { columns })
        }
        Some(Token::Unique) => {
            self.next();
            self.expect(Token::LParen)?;
            let columns = self.parse_identifier_list()?;
            self.expect(Token::RParen)?;
            Ok(TableConstraint::Unique { columns })
        }
        _ => Err("Expected FOREIGN KEY, PRIMARY KEY, or UNIQUE".to_string()),
    }
}

/// Parse a list of identifiers separated by commas
fn parse_identifier_list(&mut self) -> Result<Vec<String>, String> {
    let mut identifiers = Vec::new();
    loop {
        match self.current() {
            Some(Token::Identifier(s)) => {
                identifiers.push(s.clone());
                self.next();
            }
            _ => return Err("Expected identifier".to_string()),
        }
        match self.current() {
            Some(Token::Comma) => {
                self.next();
                continue;
            }
            _ => break,
        }
    }
    Ok(identifiers)
}

/// Parse FK action: ON DELETE/UPDATE CASCADE|SET NULL|RESTRICT
fn parse_fk_action(&mut self, action_type: Token) -> Result<Option<ForeignKeyAction>, String> {
    if matches!(self.current(), Some(Token::On)) {
        self.next();
        if matches!(self.current(), Some(tok) if tok.keyword_eq("DELETE")) {
            self.next();
            return self.parse_fk_action_value();
        }
        if matches!(self.current(), Some(tok) if tok.keyword_eq("UPDATE")) {
            self.next();
            return self.parse_fk_action_value();
        }
    }
    // 检查是否直接跟着 action_type (如 CASCADE, SET NULL, RESTRICT)
    match self.current() {
        Some(Token::Identifier(s)) => {
            let upper = s.to_uppercase();
            match upper.as_str() {
                "CASCADE" => {
                    self.next();
                    Ok(Some(ForeignKeyAction::Cascade))
                }
                "RESTRICT" => {
                    self.next();
                    Ok(Some(ForeignKeyAction::Restrict))
                }
                "SET" => {
                    self.next();
                    if let Some(Token::Identifier(null_check)) = self.current() {
                        if null_check.to_uppercase() == "NULL" {
                            self.next();
                            Ok(Some(ForeignKeyAction::SetNull))
                        } else {
                            Ok(None)
                        }
                    } else {
                        Ok(None)
                    }
                }
                _ => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

fn parse_fk_action_value(&mut self) -> Result<Option<ForeignKeyAction>, String> {
    match self.current() {
        Some(Token::Identifier(s)) => {
            let upper = s.to_uppercase();
            match upper.as_str() {
                "CASCADE" => {
                    self.next();
                    Ok(Some(ForeignKeyAction::Cascade))
                }
                "RESTRICT" => {
                    self.next();
                    Ok(Some(ForeignKeyAction::Restrict))
                }
                "SET" => {
                    self.next();
                    if let Some(Token::Identifier(null_check)) = self.current() {
                        if null_check.to_uppercase() == "NULL" {
                            self.next();
                            Ok(Some(ForeignKeyAction::SetNull))
                        } else {
                            Ok(None)
                        }
                    } else {
                        Ok(None)
                    }
                }
                _ => Ok(None),
            }
        }
        _ => Ok(None),
    }
}
```

**Step 2: 验证编译**

Run: `cargo build -p sqlrustgo-parser 2>&1 | grep -E "^error|warning: unused" | head -10`
Expected: 无错误，可能有 unused warnings（正常）

**Step 3: Commit**

```bash
git add crates/parser/src/parser.rs
git commit -m "feat(parser): implement parse_table_constraint and helper methods"
```

---

## 任务 5: 修改 parse_create_table 解析表级约束

**Files:**
- Modify: `crates/parser/src/parser.rs:2808-2827` (列解析循环之后)

**Step 1: 修改列解析后的逻辑**

找到约 line 2808-2827：

```rust
                    }
                    Some(Token::RParen) => {
                        self.next();
                        break;
                    }
                    Some(Token::Comma) => {
                        self.next();
                    }
                    _ => break,
                }
            }
        }
```

修改为：

```rust
                    }
                    Some(Token::RParen) => {
                        self.next();
                        break;
                    }
                    Some(Token::Comma) => {
                        self.next();
                    }
                    // 如果遇到 FOREIGN/PRIMARY/UNIQUE，说明是表级约束
                    Some(Token::Foreign) | Some(Token::Primary) | Some(Token::Unique) => {
                        break;  // 跳出列解析，进入表级约束解析
                    }
                    _ => break,
                }
            }

            // 解析表级约束 (如果存在)
            let mut constraints = Vec::new();
            loop {
                match self.current() {
                    Some(Token::Foreign) | Some(Token::Primary) | Some(Token::Unique) => {
                        let constraint = self.parse_table_constraint()?;
                        constraints.push(constraint);
                    }
                    Some(Token::Comma) => {
                        self.next(); // consume comma between constraints
                    }
                    _ => break,
                }
            }
        }
```

**Step 2: 修改返回值使用 constraints**

将：
```rust
Ok(Statement::CreateTable(CreateTableStatement {
    name,
    columns,
    if_not_exists,
    constraints: Vec::new(),
}))
```

修改为：
```rust
Ok(Statement::CreateTable(CreateTableStatement {
    name,
    columns,
    if_not_exists,
    constraints,
}))
```

**Step 3: 验证编译**

Run: `cargo build -p sqlrustgo-parser 2>&1 | grep "^error" | head -5`
Expected: 无错误

**Step 4: Commit**

```bash
git add crates/parser/src/parser.rs
git commit -m "feat(parser): integrate table constraint parsing in parse_create_table"
```

---

## 任务 6: 添加单元测试

**Files:**
- Modify: `crates/parser/src/parser.rs` (测试模块)

**Step 1: 在 parser.rs 底部测试模块添加测试**

找到测试模块（约 line 4990-5250），添加：

```rust
#[test]
fn test_create_table_with_foreign_key_constraint() {
    let sql = "CREATE TABLE orders (\
                id INTEGER, \
                user_id INTEGER, \
                FOREIGN KEY (user_id) REFERENCES users(id) \
               )";
    let result = parse(sql);
    assert!(result.is_ok());
    let stmt = result.unwrap();
    match stmt {
        Statement::CreateTable(ct) => {
            assert_eq!(ct.name, "orders");
            assert_eq!(ct.columns.len(), 2);
            assert_eq!(ct.constraints.len(), 1);
            match &ct.constraints[0] {
                TableConstraint::ForeignKey { columns, reference_table, reference_columns, .. } => {
                    assert_eq!(columns, &vec!["user_id"]);
                    assert_eq!(reference_table, "users");
                    assert_eq!(reference_columns, &vec!["id"]);
                }
                _ => panic!("Expected ForeignKey constraint"),
            }
        }
        _ => panic!("Expected CreateTable statement"),
    }
}

#[test]
fn test_create_table_with_multi_column_foreign_key() {
    let sql = "CREATE TABLE order_items (\
                order_id INTEGER, \
                product_id INTEGER, \
                quantity INTEGER, \
                FOREIGN KEY (order_id, product_id) REFERENCES order_products(order_id, product_id) \
               )";
    let result = parse(sql);
    assert!(result.is_ok());
    let stmt = result.unwrap();
    match stmt {
        Statement::CreateTable(ct) => {
            assert_eq!(ct.constraints.len(), 1);
            match &ct.constraints[0] {
                TableConstraint::ForeignKey { columns, reference_columns, .. } => {
                    assert_eq!(columns, &vec!["order_id", "product_id"]);
                    assert_eq!(reference_columns, &vec!["order_id", "product_id"]);
                }
                _ => panic!("Expected ForeignKey constraint"),
            }
        }
        _ => panic!("Expected CreateTable statement"),
    }
}

#[test]
fn test_create_table_with_fk_on_delete_cascade() {
    let sql = "CREATE TABLE orders (\
                id INTEGER, \
                user_id INTEGER, \
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE \
               )";
    let result = parse(sql);
    assert!(result.is_ok());
}

#[test]
fn test_create_table_with_fk_on_update_set_null() {
    let sql = "CREATE TABLE orders (\
                id INTEGER, \
                user_id INTEGER, \
                FOREIGN KEY (user_id) REFERENCES users(id) ON UPDATE SET NULL \
               )";
    let result = parse(sql);
    assert!(result.is_ok());
}

#[test]
fn test_create_table_with_primary_key_constraint() {
    let sql = "CREATE TABLE composite (a INTEGER, b INTEGER, PRIMARY KEY (a, b))";
    let result = parse(sql);
    assert!(result.is_ok());
    let stmt = result.unwrap();
    match stmt {
        Statement::CreateTable(ct) => {
            assert_eq!(ct.constraints.len(), 1);
            match &ct.constraints[0] {
                TableConstraint::PrimaryKey { columns } => {
                    assert_eq!(columns, &vec!["a", "b"]);
                }
                _ => panic!("Expected PrimaryKey constraint"),
            }
        }
        _ => panic!("Expected CreateTable statement"),
    }
}
```

**Step 2: 运行测试**

Run: `cargo test -p sqlrustgo-parser test_create_table_with_foreign_key_constraint -v 2>&1 | tail -20`
Expected: 测试通过

**Step 3: Commit**

```bash
git add crates/parser/src/parser.rs
git commit -m "test(parser): add TableConstraint parsing tests"
```

---

## 任务 7: 导出 TableConstraint 从 lib.rs

**Files:**
- Modify: `crates/parser/src/lib.rs`

**Step 1: 检查 lib.rs 导出**

查看 lib.rs（约 line 16），确保 TableConstraint 被导出：

```rust
pub use parser::{CreateTableStatement, TableConstraint /* ... */};
```

如果没有 TableConstraint，添加它。

**Step 2: 验证编译**

Run: `cargo build -p sqlrustgo-parser 2>&1 | grep "^error" | head -3`
Expected: 无错误

**Step 3: Commit**

```bash
git add crates/parser/src/lib.rs
git commit -m "feat(parser): export TableConstraint from lib.rs"
```

---

## 任务 8: 添加集成测试

**Files:**
- Create: `tests/integration/foreign_key_table_constraint_test.rs`
- Modify: `tests/regression_test.rs` (注册测试)

**Step 1: 创建集成测试文件**

```rust
//! Foreign Key Table Constraint Integration Tests
//!
//! Tests for Issue #1379: Table-level FOREIGN KEY constraint syntax

use sqlrustgo::parse;
use sqlrustgo::{ExecutionEngine, MemoryStorage};
use std::sync::{Arc, RwLock};

#[test]
fn test_fk_table_constraint_single_column() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    // Insert parent records
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    // Create child table with table-level FK
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id))")
                .unwrap(),
        )
        .unwrap();

    // Insert with valid FK reference - should succeed
    let result = engine.execute(parse("INSERT INTO orders VALUES (1, 1)").unwrap());
    assert!(result.is_ok(), "Should allow insert with valid FK reference: {:?}", result);

    // Insert with invalid FK reference - should fail
    let result = engine.execute(parse("INSERT INTO orders VALUES (2, 999)").unwrap());
    assert!(result.is_err(), "Should reject insert with invalid FK reference");
}

#[test]
fn test_fk_table_constraint_multi_column() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent table with composite PK
    engine
        .execute(
            parse("CREATE TABLE order_products (order_id INTEGER, product_id INTEGER, PRIMARY KEY (order_id, product_id))")
                .unwrap(),
        )
        .unwrap();

    // Insert parent record
    engine
        .execute(parse("INSERT INTO order_products VALUES (1, 100)").unwrap())
        .unwrap();

    // Create child table with multi-column FK
    engine
        .execute(
            parse("CREATE TABLE line_items (id INTEGER, order_id INTEGER, product_id INTEGER, FOREIGN KEY (order_id, product_id) REFERENCES order_products(order_id, product_id))")
                .unwrap(),
        )
        .unwrap();

    // Insert with valid composite FK - should succeed
    let result = engine
        .execute(parse("INSERT INTO line_items VALUES (1, 1, 100)").unwrap());
    assert!(result.is_ok(), "Should allow insert with valid composite FK: {:?}", result);

    // Insert with partial match - should fail
    let result = engine
        .execute(parse("INSERT INTO line_items VALUES (2, 1, 999)").unwrap());
    assert!(result.is_err(), "Should reject insert with partial composite FK match");
}

#[test]
fn test_fk_table_constraint_on_delete_cascade() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice')").unwrap())
        .unwrap();

    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE)")
                .unwrap(),
        )
        .unwrap();
    engine
        .execute(parse("INSERT INTO orders VALUES (1, 1)").unwrap())
        .unwrap();

    // Delete parent - should cascade delete child
    engine.execute(parse("DELETE FROM users WHERE id = 1").unwrap()).unwrap();

    // Verify child is also deleted
    let result = engine.execute(parse("SELECT * FROM orders").unwrap()).unwrap();
    assert_eq!(result.rows.len(), 0, "Child rows should be cascade deleted");
}

#[test]
fn test_fk_table_constraint_on_update_set_null() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id) ON UPDATE SET NULL)")
                .unwrap(),
        )
        .unwrap();
    engine
        .execute(parse("INSERT INTO orders VALUES (1, 1)").unwrap())
        .unwrap();

    // Update parent PK - should SET NULL on child
    engine.execute(parse("UPDATE users SET id = 100 WHERE id = 1").unwrap()).unwrap();

    // Verify child's FK is set to NULL
    let result = engine.execute(parse("SELECT user_id FROM orders").unwrap()).unwrap();
    assert_eq!(result.rows[0].values[0], sqlrustgo_types::Value::Null);
}

#[test]
fn test_primary_key_table_constraint() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(
            parse("CREATE TABLE composite (a INTEGER, b INTEGER, c TEXT, PRIMARY KEY (a, b))")
                .unwrap(),
        )
        .unwrap();

    // Insert valid record
    let result = engine.execute(parse("INSERT INTO composite VALUES (1, 2, 'test')").unwrap());
    assert!(result.is_ok());

    // Insert duplicate PK - should fail
    let result = engine.execute(parse("INSERT INTO composite VALUES (1, 2, 'dup')").unwrap());
    assert!(result.is_err(), "Should reject duplicate primary key");
}

#[test]
fn test_fk_and_column_level_fk_together() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    // Create parent tables
    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY)").unwrap())
        .unwrap();
    engine
        .execute(parse("CREATE TABLE products (id INTEGER PRIMARY KEY)").unwrap())
        .unwrap();

    engine
        .execute(parse("INSERT INTO users VALUES (1)").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO products VALUES (100)").unwrap())
        .unwrap();

    // Create table with both column-level and table-level FKs
    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id), product_id INTEGER, FOREIGN KEY (product_id) REFERENCES products(id))")
                .unwrap(),
        )
        .unwrap();

    // Insert with both FKs valid
    let result = engine
        .execute(parse("INSERT INTO orders VALUES (1, 1, 100)").unwrap());
    assert!(result.is_ok());

    // Insert with invalid column-level FK
    let result = engine
        .execute(parse("INSERT INTO orders VALUES (2, 999, 100)").unwrap());
    assert!(result.is_err());

    // Insert with invalid table-level FK
    let result = engine
        .execute(parse("INSERT INTO orders VALUES (3, 1, 999)").unwrap());
    assert!(result.is_err());
}
```

**Step 2: 注册到 regression_test.rs**

查看 `tests/regression_test.rs` 的 `get_test_categories()` 函数，添加新的测试类别：

```rust
TestCategory {
    name: "foreign_key_table_constraint",
    description: "Table-level FOREIGN KEY constraint tests (Issue #1379)",
    test_file: "integration/foreign_key_table_constraint_test.rs",
},
```

**Step 3: 运行测试**

Run: `cargo test -p sqlrustgo --test foreign_key_table_constraint_test 2>&1 | tail -30`
Expected: 所有测试通过

**Step 4: Commit**

```bash
git add tests/integration/foreign_key_table_constraint_test.rs tests/regression_test.rs
git commit -m "test: add foreign key table constraint integration tests (Issue #1379)"
```

---

## 任务 9: 最终验证

**Step 1: 运行所有 FK 相关测试**

Run: `cargo test -p sqlrustgo foreign_key 2>&1 | tail -30`
Expected: 所有 foreign_key 相关测试通过

**Step 2: 检查是否有编译警告**

Run: `cargo build -p sqlrustgo-parser 2>&1 | grep -E "^warning" | head -10`
Expected: 无新增警告

**Step 3: Commit**

```bash
git add -A
git commit -m "feat: complete table-level FOREIGN KEY constraint support (Issue #1379)"
```

---

## 任务 10: 创建 PR

**Step 1: Push 分支**

```bash
git push -u origin feature/foreign-key-table-constraint
```

**Step 2: 创建 PR**

使用 `gh pr create` 或 GitHub UI 创建 PR。

---

## 验证清单

- [ ] `cargo test -p sqlrustgo-parser test_create_table_with_foreign_key` 通过
- [ ] `cargo test -p sqlrustgo foreign_key` 全部通过
- [ ] Parser 支持 `FOREIGN KEY (col) REFERENCES table(col)`
- [ ] Parser 支持 `FOREIGN KEY (a,b) REFERENCES table(x,y)` (multi-column)
- [ ] Parser 支持 `ON DELETE CASCADE`
- [ ] Parser 支持 `ON UPDATE SET NULL`
- [ ] Parser 支持 `PRIMARY KEY (a,b)` 表级语法
- [ ] 集成测试全部通过
- [ ] PR 创建并通过 CI
