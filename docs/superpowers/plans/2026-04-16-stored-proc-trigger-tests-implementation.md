# 存储过程增强与触发器测试实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 CASE/REPEAT 存储过程语句并添加触发器集成测试

**Architecture:**
- 在 StoredProcStatement 添加 Case, CaseWhen, Repeat 变体
- 在 StoredProcExecutor 添加 execute_case, execute_case_when, execute_repeat 方法
- 创建触发器集成测试文件

**Tech Stack:** Rust, cargo test

---

## 文件结构

```
crates/
├── catalog/src/stored_proc.rs       # 添加 Case, CaseWhen, Repeat 变体
├── executor/src/stored_proc.rs     # 实现 execute_case 等方法
└── executor/tests/
    ├── test_stored_proc.rs         # 添加 CASE/REPEAT 测试
    └── test_trigger.rs              # 新建触发器测试
```

---

## Task 1: 添加 CASE 语句类型到 Catalog

**Files:**
- Modify: `crates/catalog/src/stored_proc.rs`

- [ ] **Step 1: 在 StoredProcStatement 枚举中添加 Case 和 CaseWhen 变体**

在 `crates/catalog/src/stored_proc.rs` 的 `StoredProcStatement` 枚举中添加:

```rust
/// CASE case_value WHEN value1 THEN result1 ... ELSE result END
Case {
    case_value: Option<String>,
    when_clauses: Vec<(String, String)>,
    else_result: Option<String>,
}

/// CASE WHEN condition1 THEN result1 ... ELSE result END
CaseWhen {
    when_clauses: Vec<(String, String)>,
    else_result: Option<String>,
}
```

找到现有枚举定义位置 (约第 31 行)，在 `Loop` 变体后添加:

```rust
    /// LOOP statements END LOOP (with optional LEAVE to exit)
    Loop { body: Vec<StoredProcStatement> },
    /// CASE case_value WHEN value THEN result ... ELSE result END
    Case {
        case_value: Option<String>,
        when_clauses: Vec<(String, String)>,
        else_result: Option<String>,
    },
    /// CASE WHEN condition THEN result ... ELSE result END
    CaseWhen {
        when_clauses: Vec<(String, String)>,
        else_result: Option<String>,
    },
```

- [ ] **Step 2: 验证编译**

Run: `cargo check --package sqlrustgo-catalog`
Expected: 编译成功

- [ ] **Step 3: 提交**

```bash
git add crates/catalog/src/stored_proc.rs
git commit -m "feat(catalog): add Case and CaseWhen statement variants"
```

---

## Task 2: 添加 REPEAT 语句类型到 Catalog

**Files:**
- Modify: `crates/catalog/src/stored_proc.rs`

- [ ] **Step 1: 在 StoredProcStatement 枚举中添加 Repeat 变体**

在 `StoredProcStatement` 枚举中添加:

```rust
/// REPEAT statements UNTIL condition END REPEAT
Repeat {
    body: Vec<StoredProcStatement>,
    condition: String,
},
```

- [ ] **Step 2: 验证编译**

Run: `cargo check --package sqlrustgo-catalog`
Expected: 编译成功

- [ ] **Step 3: 提交**

```bash
git add crates/catalog/src/stored_proc.rs
git commit -m "feat(catalog): add Repeat statement variant"
```

---

## Task 3: 实现 CASE 语句执行

**Files:**
- Modify: `crates/executor/src/stored_proc.rs`

- [ ] **Step 1: 添加 execute_case 方法**

在 `crates/executor/src/stored_proc.rs` 中找到现有的语句执行匹配代码 (约第 594 行 `StoredProcStatement::While` 附近)，在 `Loop` 分支后添加:

```rust
            StoredProcStatement::Case { case_value, when_clauses, else_result } => {
                // Execute CASE statement
                let case_val = if let Some(ref cv) = case_value {
                    self.evaluate_expression(cv, ctx)?
                } else {
                    Value::Null
                };

                // Check each WHEN clause
                for (when_val, result) in when_clauses {
                    let when_expr_val = self.evaluate_expression(&when_val, ctx)?;
                    if case_val == when_expr_val {
                        return self.evaluate_expression(&result, ctx).map(|v| {
                            ctx.set_return(v);
                            ExecutorResult::Ok(())
                        });
                    }
                }

                // Execute ELSE clause if present
                if let Some(else_val) = else_result {
                    return self.evaluate_expression(&else_val, ctx).map(|v| {
                        ctx.set_return(v);
                        ExecutorResult::Ok(())
                    });
                }

                Ok(ExecutorResult::Ok(()))
            }
```

- [ ] **Step 2: 添加 execute_case_when 方法**

在同一位置添加:

```rust
            StoredProcStatement::CaseWhen { when_clauses, else_result } => {
                // Check each WHEN clause
                for (condition, result) in when_clauses {
                    if self.evaluate_condition(&condition, ctx)? {
                        return self.evaluate_expression(&result, ctx).map(|v| {
                            ctx.set_return(v);
                            ExecutorResult::Ok(())
                        });
                    }
                }

                // Execute ELSE clause if present
                if let Some(else_val) = else_result {
                    return self.evaluate_expression(&else_val, ctx).map(|v| {
                        ctx.set_return(v);
                        ExecutorResult::Ok(())
                    });
                }

                Ok(ExecutorResult::Ok(()))
            }
```

- [ ] **Step 3: 验证编译**

Run: `cargo check --package sqlrustgo-executor`
Expected: 编译成功

- [ ] **Step 4: 提交**

```bash
git add crates/executor/src/stored_proc.rs
git commit -m "feat(executor): implement CASE and CASE WHEN execution"
```

---

## Task 4: 实现 REPEAT 循环执行

**Files:**
- Modify: `crates/executor/src/stored_proc.rs`

- [ ] **Step 1: 添加 execute_repeat 方法**

在 `StoredProcStatement` 匹配代码中添加:

```rust
            StoredProcStatement::Repeat { body, condition } => {
                // REPEAT ... UNTIL condition END REPEAT
                // Execute body first, then check condition
                loop {
                    if ctx.should_leave() {
                        ctx.reset_leave();
                        break;
                    }
                    if ctx.get_return().is_some() {
                        break;
                    }

                    ctx.reset_iterate();
                    self.execute_body(body.clone(), ctx)?;

                    if ctx.should_iterate() {
                        ctx.reset_iterate();
                        continue;
                    }

                    // Check UNTIL condition
                    if self.evaluate_condition(&condition, ctx)? {
                        break;
                    }
                }
                Ok(ExecutorResult::Ok(()))
            }
```

- [ ] **Step 2: 验证编译**

Run: `cargo check --package sqlrustgo-executor`
Expected: 编译成功

- [ ] **Step 3: 提交**

```bash
git add crates/executor/src/stored_proc.rs
git commit -m "feat(executor): implement REPEAT loop execution"
```

---

## Task 5: 添加 CASE 和 REPEAT 测试

**Files:**
- Modify: `crates/executor/tests/test_stored_proc.rs`

- [ ] **Step 1: 添加 CASE 语句测试**

在 `crates/executor/tests/test_stored_proc.rs` 末尾添加:

```rust
// ============================================================================
// CASE Statement Tests
// ============================================================================

#[test]
fn test_case_simple_value() {
    let proc = StoredProcedure::new(
        "test_case".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "x".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("2".to_string()),
            },
            StoredProcStatement::Case {
                case_value: Some("@x".to_string()),
                when_clauses: vec![
                    ("1".to_string(), "'one'".to_string()),
                    ("2".to_string(), "'two'".to_string()),
                    ("3".to_string(), "'three'".to_string()),
                ],
                else_result: Some("'other'".to_string()),
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_case", vec![]).is_ok());
}

#[test]
fn test_case_when_conditional() {
    let proc = StoredProcedure::new(
        "test_case_when".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "x".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("15".to_string()),
            },
            StoredProcStatement::CaseWhen {
                when_clauses: vec![
                    ("@x > 10".to_string(), "'big'".to_string()),
                    ("@x > 5".to_string(), "'medium'".to_string()),
                ],
                else_result: Some("'small'".to_string()),
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_case_when", vec![]).is_ok());
}
```

- [ ] **Step 2: 添加 REPEAT 循环测试**

在 `crates/executor/tests/test_stored_proc.rs` 末尾添加:

```rust
// ============================================================================
// REPEAT Loop Tests
// ============================================================================

#[test]
fn test_repeat_until() {
    let proc = StoredProcedure::new(
        "test_repeat".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "counter".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("0".to_string()),
            },
            StoredProcStatement::Repeat {
                body: vec![
                    StoredProcStatement::Set {
                        variable: "counter".to_string(),
                        value: "@counter + 1".to_string(),
                    },
                ],
                condition: "@counter >= 10".to_string(),
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_repeat", vec![]).is_ok());
}

#[test]
fn test_repeat_with_leave() {
    let proc = StoredProcedure::new(
        "test_repeat_leave".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "x".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("0".to_string()),
            },
            StoredProcStatement::Repeat {
                body: vec![
                    StoredProcStatement::Set {
                        variable: "x".to_string(),
                        value: "@x + 1".to_string(),
                    },
                    StoredProcStatement::If {
                        condition: "@x >= 5".to_string(),
                        then_body: vec![StoredProcStatement::Leave { label: String::new() }],
                        elseif_body: vec![],
                        else_body: vec![],
                    },
                ],
                condition: "FALSE".to_string(), // Will exit via LEAVE
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_repeat_leave", vec![]).is_ok());
}
```

- [ ] **Step 3: 运行测试验证**

Run: `cargo test --package sqlrustgo-executor --test test_stored_proc case`
Expected: PASS

Run: `cargo test --package sqlrustgo-executor --test test_stored_proc repeat`
Expected: PASS

- [ ] **Step 4: 提交**

```bash
git add crates/executor/tests/test_stored_proc.rs
git commit -m "test(executor): add CASE and REPEAT tests"
```

---

## Task 6: 创建触发器集成测试

**Files:**
- Create: `crates/executor/tests/test_trigger.rs`
- Modify: `crates/executor/Cargo.toml` (如果需要)

- [ ] **Step 1: 创建触发器测试文件**

创建 `crates/executor/tests/test_trigger.rs`:

```rust
//! Integration tests for Trigger Executor

use sqlrustgo_executor::trigger::{TriggerExecutor, TriggerTiming, TriggerEvent};
use sqlrustgo_storage::{
    ColumnDefinition, MemoryStorage, StorageEngine, TableInfo, TriggerInfo as StorageTriggerInfo,
    Record, Value,
};

fn create_test_storage() -> MemoryStorage {
    let mut storage = MemoryStorage::new();

    let table_info = TableInfo {
        name: "orders".to_string(),
        columns: vec![
            ColumnDefinition::new("id", "INTEGER"),
            ColumnDefinition::new("price", "FLOAT"),
            ColumnDefinition::new("quantity", "INTEGER"),
            ColumnDefinition::new("total", "FLOAT"),
        ],
    };

    storage.create_table(table_info).unwrap();
    storage
}

// ============================================================================
// BEFORE Trigger Tests
// ============================================================================

#[test]
fn test_before_insert_trigger() {
    let storage = create_test_storage();
    let trigger_exec = TriggerExecutor::new(storage.clone());

    let trigger = StorageTriggerInfo {
        name: "before_insert".to_string(),
        table_name: "orders".to_string(),
        timing: sqlrustgo_storage::TriggerTiming::Before,
        event: sqlrustgo_storage::TriggerEvent::Insert,
        body: "SET NEW.total = NEW.price * NEW.quantity".to_string(),
        when_condition: None,
    };

    storage.create_trigger(trigger).unwrap();

    let new_row = Record(vec![
        Value::Integer(1),
        Value::Float(10.0),
        Value::Integer(5),
        Value::Null,
    ]);

    let result = trigger_exec.execute_before_insert("orders", &new_row);
    assert!(result.is_ok());
}

#[test]
fn test_before_update_trigger() {
    let storage = create_test_storage();
    let trigger_exec = TriggerExecutor::new(storage.clone());

    let trigger = StorageTriggerInfo {
        name: "before_update".to_string(),
        table_name: "orders".to_string(),
        timing: sqlrustgo_storage::TriggerTiming::Before,
        event: sqlrustgo_storage::TriggerEvent::Update,
        body: "SET NEW.total = NEW.price * NEW.quantity".to_string(),
        when_condition: None,
    };

    storage.create_trigger(trigger).unwrap();

    let old_row = Record(vec![
        Value::Integer(1),
        Value::Float(10.0),
        Value::Integer(5),
        Value::Null,
    ]);

    let new_row = Record(vec![
        Value::Integer(1),
        Value::Float(10.0),
        Value::Integer(10),
        Value::Null,
    ]);

    let result = trigger_exec.execute_before_update("orders", &old_row, &new_row);
    assert!(result.is_ok());
}

// ============================================================================
// AFTER Trigger Tests
// ============================================================================

#[test]
fn test_after_insert_trigger() {
    let storage = create_test_storage();
    let trigger_exec = TriggerExecutor::new(storage.clone());

    let trigger = StorageTriggerInfo {
        name: "after_insert".to_string(),
        table_name: "orders".to_string(),
        timing: sqlrustgo_storage::TriggerTiming::After,
        event: sqlrustgo_storage::TriggerEvent::Insert,
        body: "SELECT 1".to_string(),
        when_condition: None,
    };

    storage.create_trigger(trigger).unwrap();

    let new_row = Record(vec![
        Value::Integer(1),
        Value::Float(10.0),
        Value::Integer(5),
        Value::Float(50.0),
    ]);

    let result = trigger_exec.execute_after_insert("orders", &new_row);
    assert!(result.is_ok());
}

#[test]
fn test_after_delete_trigger() {
    let storage = create_test_storage();
    let trigger_exec = TriggerExecutor::new(storage.clone());

    let trigger = StorageTriggerInfo {
        name: "after_delete".to_string(),
        table_name: "orders".to_string(),
        timing: sqlrustgo_storage::TriggerTiming::After,
        event: sqlrustgo_storage::TriggerEvent::Delete,
        body: "SELECT 1".to_string(),
        when_condition: None,
    };

    storage.create_trigger(trigger).unwrap();

    let old_row = Record(vec![
        Value::Integer(1),
        Value::Float(10.0),
        Value::Integer(5),
        Value::Float(50.0),
    ]);

    let result = trigger_exec.execute_after_delete("orders", &old_row);
    assert!(result.is_ok());
}

// ============================================================================
// Multiple Triggers Tests
// ============================================================================

#[test]
fn test_multiple_triggers_order() {
    let storage = create_test_storage();

    let trigger1 = StorageTriggerInfo {
        name: "trigger_1".to_string(),
        table_name: "orders".to_string(),
        timing: sqlrustgo_storage::TriggerTiming::Before,
        event: sqlrustgo_storage::TriggerEvent::Insert,
        body: "SET NEW.total = 100".to_string(),
        when_condition: None,
    };

    let trigger2 = StorageTriggerInfo {
        name: "trigger_2".to_string(),
        table_name: "orders".to_string(),
        timing: sqlrustgo_storage::TriggerTiming::Before,
        event: sqlrustgo_storage::TriggerEvent::Insert,
        body: "SET NEW.total = NEW.total + 50".to_string(),
        when_condition: None,
    };

    storage.create_trigger(trigger1).unwrap();
    storage.create_trigger(trigger2).unwrap();

    let trigger_exec = TriggerExecutor::new(storage);

    let new_row = Record(vec![
        Value::Integer(1),
        Value::Float(10.0),
        Value::Integer(5),
        Value::Null,
    ]);

    let result = trigger_exec.execute_before_insert("orders", &new_row);
    assert!(result.is_ok());
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo check --package sqlrustgo-executor --test test_trigger`
Expected: 编译成功

- [ ] **Step 3: 运行测试**

Run: `cargo test --package sqlrustgo-executor --test test_trigger`
Expected: 所有触发器测试 PASS

- [ ] **Step 4: 提交**

```bash
git add crates/executor/tests/test_trigger.rs
git commit -m "test(executor): add trigger integration tests"
```

---

## Task 7: 集成到回归测试

- [ ] **Step 1: 验证回归测试包含新测试**

Run: `cargo test --test regression_test -- --nocapture 2>&1 | grep -E "stored_proc|trigger"`
Expected: 显示相关测试

- [ ] **Step 2: 提交最终状态**

```bash
git add -A
git commit -m "feat(executor): complete CASE/REPEAT and trigger tests for issue #1434"
```

---

## 验收标准

1. `cargo check --package sqlrustgo-catalog --lib` 编译成功
2. `cargo check --package sqlrustgo-executor --lib` 编译成功
3. `cargo test --package sqlrustgo-executor --test test_stored_proc case` PASS
4. `cargo test --package sqlrustgo-executor --test test_stored_proc repeat` PASS
5. `cargo test --package sqlrustgo-executor --test test_trigger` PASS

## 性能目标

无性能目标，仅功能验证。
