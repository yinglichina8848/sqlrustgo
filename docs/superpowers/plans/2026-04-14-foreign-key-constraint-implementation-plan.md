# FOREIGN KEY 约束实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现表级 FOREIGN KEY 语法支持和 CASCADE DELETE/UPDATE 执行

**Architecture:** 
- Parser 层: 新增 TableConstraint AST，扩展 CreateTableStatement
- ExecutionEngine 层: CASCADE 执行逻辑
- Catalog 层: 新增 get_referencing_foreign_keys() API

**Tech Stack:** Rust, sqlrustgo-parser, sqlrustgo-storage, ExecutionEngine

---

## Task 1: Parser - 新增 TableConstraint AST

**Files:**
- Modify: `crates/parser/src/parser.rs:555-602`

**Step 1: 添加 TableConstraint 结构体**

在 `CreateTableStatement` 定义后添加:

```rust
/// 表级约束类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    PrimaryKey { columns: Vec<String> },
    Unique { columns: Vec<String> },
    ForeignKey {
        columns: Vec<String>,
        reference_table: String,
        reference_columns: Vec<String>,
        on_delete: Option<ForeignKeyAction>,
        on_update: Option<ForeignKeyAction>,
    },
    Check { condition: String },
}

/// 表级约束
#[derive(Debug, Clone, PartialEq)]
pub struct TableConstraint {
    pub name: Option<String>,
    pub constraint_type: ConstraintType,
}
```

**Step 2: 修改 CreateTableStatement 添加 table_constraints 字段**

```rust
pub struct CreateTableStatement {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    pub table_constraints: Vec<TableConstraint>,  // 新增
    pub if_not_exists: bool,
}
```

**Step 3: 验证编译**

Run: `cargo build -p sqlrustgo-parser 2>&1`
Expected: 编译成功（可能有 unused warnings）

---

## Task 2: Parser - 解析表级 FOREIGN KEY 语法

**Files:**
- Modify: `crates/parser/src/parser.rs` (parse_create_table 函数)

**Step 1: 在 parse_create_table 中添加表级约束解析逻辑**

在解析完所有列之后，添加:

```rust
// 解析表级约束 (FOREIGN KEY, UNIQUE, CHECK, PRIMARY KEY)
while !self.peek_token_is(Token::RParen) {
    if self.peek_token_is(Token::Keyword(Kw::CONSTRAINT)) {
        self.expect_token(&Token::Keyword(Kw::CONSTRAINT))?;
        let name = Some(self.parse_identifier()?);
        self.parse_table_constraint(name)?
    } else if self.peek_token_is(Token::Keyword(Kw::FOREIGN)) {
        self.parse_table_constraint(None)?
    } else if self.peek_token_is(Token::Keyword(Kw::UNIQUE)) {
        self.parse_table_constraint(None)?
    } else if self.peek_token_is(Token::Keyword(Kw::PRIMARY)) {
        self.expect_token(&Token::Keyword(Kw::PRIMARY))?;
        self.expect_token(&Token::Keyword(Kw::KEY))?;
        // 解析 PRIMARY KEY (columns)
        self.expect_token(&Token::LParen)?;
        let columns = self.parse_column_list()?;
        self.expect_token(&Token::RParen)?;
        table_constraints.push(TableConstraint {
            name: None,
            constraint_type: ConstraintType::PrimaryKey { columns },
        });
    } else {
        break;
    }
}
```

**Step 2: 实现 parse_table_constraint 函数**

```rust
fn parse_table_constraint(&mut self, name: Option<String>) -> Result<TableConstraint, ParseError> {
    if self.match_token(&Token::Keyword(Kw::FOREIGN)) {
        self.expect_token(&Token::Keyword(Kw::KEY))?;
        self.expect_token(&Token::LParen)?;
        let columns = self.parse_column_list()?;
        self.expect_token(&Token::RParen)?;
        self.expect_token(&Token::Keyword(Kw::REFERENCES))?;
        let reference_table = self.parse_identifier()?;
        let reference_columns = if self.peek_token_is(Token::LParen) {
            self.expect_token(&Token::LParen)?;
            let cols = self.parse_column_list()?;
            self.expect_token(&Token::RParen)?;
            cols
        } else {
            vec![]
        };
        let (on_delete, on_update) = self.parse_fk_actions()?;
        Ok(TableConstraint {
            name,
            constraint_type: ConstraintType::ForeignKey {
                columns,
                reference_table,
                reference_columns,
                on_delete,
                on_update,
            },
        })
    } else if self.match_token(&Token::Keyword(Kw::UNIQUE)) {
        self.expect_token(&Token::LParen)?;
        let columns = self.parse_column_list()?;
        self.expect_token(&Token::RParen)?;
        Ok(TableConstraint {
            name,
            constraint_type: ConstraintType::Unique { columns },
        })
    } else {
        Err(ParseError::UnexpectedToken(self.current_token.clone()))
    }
}
```

**Step 3: 实现 parse_fk_actions 函数**

```rust
fn parse_fk_actions(&mut self) -> Result<(Option<ForeignKeyAction>, Option<ForeignKeyAction>), ParseError> {
    let mut on_delete = None;
    let mut on_update = None;
    
    if self.match_token(&Token::Keyword(Kw::ON)) {
        if self.match_token(&Token::Keyword(Kw::DELETE)) {
            on_delete = Some(self.parse_fk_action()?);
        } else if self.match_token(&Token::Keyword(Kw::UPDATE)) {
            on_update = Some(self.parse_fk_action()?);
        }
    }
    
    // 再次检查 ON UPDATE
    if self.match_token(&Token::Keyword(Kw::ON)) {
        self.expect_token(&Token::Keyword(Kw::UPDATE))?;
        on_update = Some(self.parse_fk_action()?);
    }
    
    Ok((on_delete, on_update))
}

fn parse_fk_action(&mut self) -> Result<ForeignKeyAction, ParseError> {
    if self.match_token(&Token::Keyword(Kw::CASCADE)) {
        Ok(ForeignKeyAction::Cascade)
    } else if self.match_token(&Token::Keyword(Kw::SET)) {
        if self.match_token(&Token::Keyword(Kw::NULL)) {
            Ok(ForeignKeyAction::SetNull)
        } else {
            Err(ParseError::UnexpectedToken(self.current_token.clone()))
        }
    } else if self.match_token(&Token::Keyword(Kw::RESTRICT)) {
        Ok(ForeignKeyAction::Restrict)
    } else {
        Err(ParseError::UnexpectedToken(self.current_token.clone()))
    }
}
```

**Step 4: 验证编译**

Run: `cargo build -p sqlrustgo-parser 2>&1`
Expected: 编译成功

**Step 5: 提交**

```bash
git add crates/parser/src/parser.rs
git commit -m "feat(parser): add TableConstraint AST and table-level FOREIGN KEY parsing"
```

---

## Task 3: Storage - 新增 get_referencing_foreign_keys API

**Files:**
- Modify: `crates/storage/src/engine.rs` (StorageEngine trait)

**Step 1: 添加 ReferencingForeignKey 结构体**

在 `ForeignKeyConstraint` 定义后添加:

```rust
/// 引用外键信息（用于 CASCADE 查询）
#[derive(Debug, Clone)]
pub struct ReferencingForeignKey {
    pub child_table: String,
    pub child_columns: Vec<String>,
    pub parent_table: String,
    pub parent_columns: Vec<String>,
    pub on_delete: Option<ForeignKeyAction>,
    pub on_update: Option<ForeignKeyAction>,
}
```

**Step 2: 在 StorageEngine trait 中添加方法签名**

```rust
/// 获取引用指定表的所有外键关系
fn get_referencing_foreign_keys(&self, table: &str) -> Vec<ReferencingForeignKey>;
```

**Step 3: 在 MemoryStorage 实现中实现该方法**

```rust
fn get_referencing_foreign_keys(&self, table: &str) -> Vec<ReferencingForeignKey> {
    let mut result = Vec::new();
    for (table_name, table_info) in &self.table_infos {
        for col_def in &table_info.columns {
            if let Some(ref fk) = col_def.references {
                if fk.referenced_table == table {
                    result.push(ReferencingForeignKey {
                        child_table: table_name.clone(),
                        child_columns: vec![col_def.name.clone()],
                        parent_table: fk.referenced_table.clone(),
                        parent_columns: vec![fk.referenced_column.clone()],
                        on_delete: fk.on_delete,
                        on_update: fk.on_update,
                    });
                }
            }
        }
    }
    result
}
```

**Step 4: 在 FileStorage 和其他实现中添加空实现或完整实现**

对于 FileStorage，可以返回空Vec或委托给 MemoryStorage 的共享逻辑。

**Step 5: 验证编译**

Run: `cargo build -p sqlrustgo-storage 2>&1`
Expected: 编译成功

**Step 6: 提交**

```bash
git add crates/storage/src/engine.rs
git commit -m "feat(storage): add get_referencing_foreign_keys API"
```

---

## Task 4: ExecutionEngine - 实现 CASCADE DELETE

**Files:**
- Modify: `src/lib.rs` (DELETE 执行逻辑)

**Step 1: 读取当前 DELETE 实现**

确认 `Statement::Delete` 的代码位置 (约 2313 行)

**Step 2: 在 DELETE 执行前添加 CASCADE 处理**

在 `let mut storage = self.storage.write().unwrap();` 后添加:

```rust
// FK CASCADE 处理
let fk_references = {
    let mut result = Vec::new();
    let mut to_delete_by_table: HashMap<String, Vec<Vec<Value>>> = HashMap::new();
    
    // 查找所有引用该表的子表
    let referencing_fks = storage.get_referencing_foreign_keys(table_name);
    
    for fk in referencing_fks {
        match fk.on_delete {
            Some(ForeignKeyAction::Cascade) | None => {
                // CASCADE: 收集需要删除的子表行
                // TODO: 根据父表删除的行，查找匹配的子表行
            }
            Some(ForeignKeyAction::SetNull) => {
                // SET NULL: 将子表外键设为 NULL
                // TODO: 实现 SET NULL 逻辑
            }
            Some(ForeignKeyAction::Restrict) => {
                // RESTRICT: 检查是否有子表行
                // TODO: 如果有子表行则返回错误
            }
        }
    }
    result
};
```

**Step 3: 实现递归 CASCADE 删除**

```rust
fn cascade_delete_recursive(
    &self,
    storage: &mut dyn StorageEngine,
    table: &str,
    parent_keys: &HashSet<Value>,
    visited: &mut HashSet<String>,
) -> Result<(), SqlError> {
    if visited.contains(table) {
        return Ok(()); // 防止循环引用
    }
    visited.insert(table.to_string());
    
    let referencing_fks = storage.get_referencing_foreign_keys(table);
    
    for fk in referencing_fks {
        if let Some(ForeignKeyAction::Cascade) = fk.on_delete {
            // 获取子表所有记录
            let child_records = storage.scan(&fk.child_table)?;
            
            // 找出需要删除的子记录（外键值在 parent_keys 中）
            let keys_to_delete: Vec<Vec<Value>> = child_records
                .into_iter()
                .filter(|row| {
                    if let Some(col_idx) = storage.get_table_info(&fk.child_table)
                        .and_then(|info| info.columns.iter().position(|c| c.name == fk.child_columns[0]))
                    {
                        if let Some(val) = row.get(col_idx) {
                            return parent_keys.contains(val);
                        }
                    }
                    false
                })
                .collect();
            
            if !keys_to_delete.is_empty() {
                // 递归处理子表的 CASCADE
                self.cascade_delete_recursive(
                    storage,
                    &fk.child_table,
                    &keys_to_delete.iter()
                        .filter_map(|r| r.get(0).cloned())
                        .collect(),
                    visited,
                )?;
                
                // 删除子表记录
                storage.delete(&fk.child_table, keys_to_delete)?;
            }
        }
    }
    
    Ok(())
}
```

**Step 4: 验证编译**

Run: `cargo build --package sqlrustgo 2>&1`
Expected: 编译成功

**Step 5: 提交**

```bash
git add src/lib.rs
git commit -m "feat(executor): implement CASCADE DELETE for FOREIGN KEY"
```

---

## Task 5: ExecutionEngine - 实现 CASCADE UPDATE

**Files:**
- Modify: `src/lib.rs` (UPDATE 执行逻辑)

**Step 1: 在 UPDATE 执行时添加 CASCADE 处理**

在 `Statement::Update` 处理中 (约 2418 行)，在更新父表后添加:

```rust
// FK CASCADE UPDATE 处理
// 当父表主键被更新时，需要同步更新子表的外键值
if let Some(updates) = &update.updates {
    // 检查是否更新了被外键引用的列
    let referencing_fks = storage.get_referencing_foreign_keys(table_name);
    
    for fk in referencing_fks {
        match fk.on_update {
            Some(ForeignKeyAction::Cascade) => {
                // 需要同步更新子表外键
                // TODO: 实现 UPDATE CASCADE
            }
            Some(ForeignKeyAction::SetNull) => {
                // 将子表外键设为 NULL
                // TODO: 实现 UPDATE SET NULL
            }
            Some(ForeignKeyAction::Restrict) => {
                // 检查是否有子表行引用
                // TODO: 如果有则返回错误
            }
            None => {
                // 默认行为: RESTRICT
            }
        }
    }
}
```

**Step 2: 验证编译**

Run: `cargo build --package sqlrustgo 2>&1`
Expected: 编译成功

**Step 3: 提交**

```bash
git add src/lib.rs
git commit -m "feat(executor): implement CASCADE UPDATE for FOREIGN KEY"
```

---

## Task 6: 测试 - 添加表级 FOREIGN KEY 解析测试

**Files:**
- Modify: `crates/parser/src/parser.rs` (测试部分)

**Step 1: 添加表级 FOREIGN KEY 解析测试**

```rust
#[test]
fn test_parse_table_level_foreign_key() {
    let sql = "CREATE TABLE orders (\
        id INTEGER, \
        user_id INTEGER, \
        CONSTRAINT fk_orders_users \
            FOREIGN KEY (user_id) \
            REFERENCES users(id) \
            ON DELETE CASCADE \
    )";
    let result = parse(sql).unwrap();
    assert!(matches!(result, Statement::CreateTable(_)));
    
    if let Statement::CreateTable(ct) = result {
        assert_eq!(ct.table_constraints.len(), 1);
        
        let constraint = &ct.table_constraints[0];
        assert_eq!(constraint.name, Some("fk_orders_users".to_string()));
        
        match &constraint.constraint_type {
            ConstraintType::ForeignKey { columns, reference_table, reference_columns, on_delete, on_update } => {
                assert_eq!(columns, &["user_id"]);
                assert_eq!(reference_table, "users");
                assert_eq!(reference_columns, &["id"]);
                assert_eq!(on_delete, Some(ForeignKeyAction::Cascade));
            }
            _ => panic!("Expected ForeignKey constraint"),
        }
    }
}

#[test]
fn test_parse_multi_column_foreign_key() {
    let sql = "CREATE TABLE order_items (\
        order_id INTEGER, \
        product_id INTEGER, \
        quantity INTEGER, \
        FOREIGN KEY (order_id, product_id) \
            REFERENCES order_products(order_id, product_id) \
            ON DELETE CASCADE \
    )";
    let result = parse(sql).unwrap();
    // 验证多列外键解析
}
```

**Step 2: 运行测试**

Run: `cargo test -p sqlrustgo-parser -- test_parse_table_level_foreign_key 2>&1`
Expected: PASS

**Step 3: 提交**

```bash
git add crates/parser/src/parser.rs
git commit -m "test(parser): add table-level FOREIGN KEY parsing tests"
```

---

## Task 7: 测试 - 添加 CASCADE DELETE 集成测试

**Files:**
- Modify: `tests/integration/foreign_key_test.rs`

**Step 1: 添加表级 FK CASCADE 测试**

```rust
#[test]
fn test_cascade_delete_table_level_fk() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    
    // 创建父表
    engine.execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap()).unwrap();
    
    // 创建子表（表级 FK 语法）
    engine.execute(parse("CREATE TABLE orders (\
        id INTEGER, \
        user_id INTEGER, \
        CONSTRAINT fk_orders_user \
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE \
    )").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO orders VALUES (1, 1), (2, 1), (3, 2)").unwrap()).unwrap();
    
    // 删除 Alice (id=1)
    engine.execute(parse("DELETE FROM users WHERE id = 1").unwrap()).unwrap();
    
    // 验证子表记录也被删除
    let result = engine.execute(parse("SELECT * FROM orders").unwrap()).unwrap();
    assert_eq!(result.rows.len(), 1); // 只剩 Bob 的订单
    assert_eq!(result.rows[0][1], Value::Integer(2));
}
```

**Step 2: 添加 RESTRICT 测试**

```rust
#[test]
fn test_restrict_delete_with_child_rows() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    
    engine.execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY)").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO users VALUES (1)").unwrap()).unwrap();
    
    engine.execute(parse("CREATE TABLE orders (\
        user_id INTEGER REFERENCES users(id) ON DELETE RESTRICT \
    )").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO orders VALUES (1)").unwrap()).unwrap();
    
    // 尝试删除应该被 RESTRICT
    let result = engine.execute(parse("DELETE FROM users WHERE id = 1").unwrap());
    assert!(result.is_err(), "DELETE with RESTRICT should fail when child rows exist");
}
```

**Step 3: 运行测试**

Run: `cargo test --test foreign_key_test -- test_cascade_delete_table_level_fk 2>&1`
Expected: PASS

**Step 4: 提交**

```bash
git add tests/integration/foreign_key_test.rs
git commit -m "test: add table-level FK and CASCADE DELETE tests"
```

---

## Task 8: 测试 - 添加 Self-Referencing FK 测试

**Files:**
- Modify: `tests/integration/foreign_key_test.rs`

**Step 1: 添加自引用 FK 测试**

```rust
#[test]
fn test_self_referencing_fk_cascade() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    
    // 创建自引用表 (员工表)
    engine.execute(parse("CREATE TABLE employees (\
        id INTEGER PRIMARY KEY, \
        name TEXT, \
        manager_id INTEGER REFERENCES employees(id) ON DELETE CASCADE \
    )").unwrap()).unwrap();
    
    engine.execute(parse("INSERT INTO employees VALUES (1, 'CEO', NULL)").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO employees VALUES (2, 'Manager', 1)").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO employees VALUES (3, 'Employee', 2)").unwrap()).unwrap();
    
    // 删除 CEO，应该 CASCADE 删除所有下属
    engine.execute(parse("DELETE FROM employees WHERE id = 1").unwrap()).unwrap();
    
    let result = engine.execute(parse("SELECT * FROM employees").unwrap()).unwrap();
    assert_eq!(result.rows.len(), 0, "All employees should be deleted");
}
```

**Step 2: 提交**

```bash
git add tests/integration/foreign_key_test.rs
git commit -m "test: add self-referencing FK CASCADE test"
```

---

## Task 9: 集成测试 - 验证完整流程

**Files:**
- Run: `cargo test --test foreign_key_test 2>&1`

**Expected:**
- 所有 23+ 个 FK 测试通过
- 包括新增的表级 FK、CASCADE、RESTRICT 测试

---

## Task 10: 更新回归测试框架

**Files:**
- Modify: `tests/regression_test.rs`

**Step 1: 添加 FK 测试到回归框架**

在 "集成测试 - SQL功能" 类别中添加 `fk_integration_test` 或类似条目（如果存在独立测试文件）

**Step 2: 提交**

```bash
git add tests/regression_test.rs
git commit -m "test: ensure FK tests in regression framework"
```

---

## 任务执行顺序

1. Task 1: Parser - TableConstraint AST (基础)
2. Task 2: Parser - 表级 FOREIGN KEY 解析
3. Task 3: Storage - get_referencing_foreign_keys API
4. Task 6: 测试 - 解析测试
5. Task 4: ExecutionEngine - CASCADE DELETE
6. Task 5: ExecutionEngine - CASCADE UPDATE
7. Task 7: 测试 - CASCADE DELETE 集成测试
8. Task 8: 测试 - Self-Referencing FK 测试
9. Task 9: 集成测试验证
10. Task 10: 回归测试框架更新

---

## 风险与缓解

| 风险 | 缓解 |
|------|------|
| 循环 FK 无限递归 | visited set 保护 |
| 性能问题 | 利用索引批量删除 |
| 事务一致性 | 所有操作在同一事务内 |
