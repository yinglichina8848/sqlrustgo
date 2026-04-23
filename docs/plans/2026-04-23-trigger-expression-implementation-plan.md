# Trigger Expression Evaluation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement complete trigger expression evaluation system supporting BinaryOp expressions and proper column resolution for trigger DML operations.

**Architecture:** Create a new `trigger/` submodule with context, resolver, and expression modules. Update existing `trigger.rs` to use the new expression evaluation system with fault tolerance and WHERE optimization.

**Tech Stack:** Rust, sqlrustgo-parser, sqlrustgo-storage, TDD methodology

---

## Task 1: Create Module Structure

**Files:**
- Create: `crates/executor/src/trigger/mod.rs`

**Step 1: Create trigger module directory**

```rust
// crates/executor/src/trigger/mod.rs

pub mod context;
pub mod resolver;
pub mod expression;

pub use context::{EvalContext, TriggerContext};
pub use resolver::resolve_column;
pub use expression::{expression_to_value, expression_to_bool};
```

**Step 2: Verify module compiles**

Run: `cargo build -p sqlrustgo-executor`
Expected: Compiles with warning about unused imports

**Step 3: Commit**

```bash
git add crates/executor/src/trigger/
git commit -m "feat(trigger): create trigger module structure"
```

---

## Task 2: Implement TriggerContext and EvalContext

**Files:**
- Create: `crates/executor/src/trigger/context.rs`

**Step 1: Write failing test**

```rust
// In crates/executor/src/trigger/context.rs (tests will go in trigger/tests/)

#[test]
fn test_trigger_context_creation() {
    let row = vec![Value::Integer(1), Value::Text("test".to_string())];
    let ctx = TriggerContext::new(Some(&row), None);

    assert_eq!(ctx.new().map(|r| r.len()), Some(2));
    assert_eq!(ctx.old(), None);
}

#[test]
fn test_eval_context_with_target_row() {
    let new_row = vec![Value::Integer(10)];
    let target_row = vec![Value::Integer(100)];

    let trigger_ctx = TriggerContext::new(Some(&new_row), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, Some(&target_row), None);

    assert_eq!(eval_ctx.target_row().map(|r| r.len()), Some(1));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-executor trigger::tests::test_trigger_context_creation`
Expected: FAIL - module not found

**Step 3: Write implementation**

```rust
// crates/executor/src/trigger/context.rs

use crate::types::Value;
use crate::storage::Record;

#[derive(Debug, Clone)]
pub struct TriggerContext<'a> {
    new: Option<&'a Record>,
    old: Option<&'a Record>,
}

impl<'a> TriggerContext<'a> {
    pub fn new(new: Option<&'a Record>, old: Option<&'a Record>) -> Self {
        Self { new, old }
    }

    pub fn new_row(&self) -> Option<&Record> {
        self.new
    }

    pub fn old_row(&self) -> Option<&Record> {
        self.old
    }
}

#[derive(Debug, Clone)]
pub struct EvalContext<'a> {
    trigger_ctx: &'a TriggerContext<'a>,
    target_row: Option<&'a Record>,
}

impl<'a> EvalContext<'a> {
    pub fn new(
        trigger_ctx: &'a TriggerContext<'a>,
        target_row: Option<&'a Record>,
    ) -> Self {
        Self { trigger_ctx, target_row }
    }

    pub fn trigger(&self) -> &TriggerContext<'a> {
        self.trigger_ctx
    }

    pub fn target_row(&self) -> Option<&Record> {
        self.target_row
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-executor trigger::tests::test_trigger_context_creation -p sqlrustgo-executor`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/executor/src/trigger/context.rs
git commit -m "feat(trigger): add TriggerContext and EvalContext"
```

---

## Task 3: Implement Column Resolver

**Files:**
- Create: `crates/executor/src/trigger/resolver.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_resolve_column_target_row_priority() {
    // target_row has stock=100, NEW has quantity=10
    let target = vec![Value::Integer(1), Value::Integer(100)]; // product_id, stock
    let new_row = vec![Value::Integer(1), Value::Integer(10)]; // product_id, quantity

    let trigger_ctx = TriggerContext::new(Some(&new_row), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, Some(&target));

    // "stock" should resolve to target_row's value (100)
    let result = resolve_column("stock", &eval_ctx);
    assert_eq!(result, Some(Value::Integer(100)));
}

#[test]
fn test_resolve_column_new_fallback() {
    let target = vec![Value::Integer(1), Value::Integer(100)];
    let new_row = vec![Value::Integer(1), Value::Integer(10)];

    let trigger_ctx = TriggerContext::new(Some(&new_row), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, Some(&target));

    // "quantity" not in target, should resolve to NEW.quantity (10)
    let result = resolve_column("quantity", &eval_ctx);
    assert_eq!(result, Some(Value::Integer(10)));
}

#[test]
fn test_resolve_column_not_found() {
    let trigger_ctx = TriggerContext::new(None, None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);

    let result = resolve_column("nonexistent", &eval_ctx);
    assert_eq!(result, None);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-executor trigger::tests::test_resolve_column`
Expected: FAIL - resolver module not found

**Step 3: Write implementation**

```rust
// crates/executor/src/trigger/resolver.rs

use crate::types::Value;
use super::context::EvalContext;

/// Resolve a column name using priority: target_row -> NEW -> OLD
pub fn resolve_column(name: &str, ctx: &EvalContext) -> Option<Value> {
    // 1. Try target_row (for UPDATE SET and WHERE)
    if let Some(row) = ctx.target_row() {
        if let Some(idx) = find_column_index(row, name) {
            return Some(row[idx].clone());
        }
    }

    // 2. Try NEW row
    if let Some(new) = ctx.trigger().new_row() {
        if let Some(idx) = find_column_index(new, name) {
            return Some(new[idx].clone());
        }
    }

    // 3. Try OLD row
    if let Some(old) = ctx.trigger().old_row() {
        if let Some(idx) = find_column_index(old, name) {
            return Some(old[idx].clone());
        }
    }

    None
}

fn find_column_index(row: &[Value], name: &str) -> Option<usize> {
    // In a real implementation, we'd use column metadata
    // For now, we check if any Text value matches the name (for schema-less)
    // This is a placeholder - actual implementation will need column metadata
    None
}
```

**Step 4: Run test to verify it fails properly**

Run: `cargo test -p sqlrustgo-executor trigger::tests::test_resolve_column`
Expected: FAIL - find_column_index returns None

**Step 5: Write full implementation with column metadata**

```rust
// Updated resolver.rs

use crate::types::Value;
use crate::storage::Record;
use super::context::EvalContext;

/// Resolve column using context's schema information
pub fn resolve_column(
    name: &str,
    ctx: &EvalContext,
    column_names: Option<&[String]>,
) -> Option<Value> {
    // 1. Try target_row with column names
    if let Some(row) = ctx.target_row() {
        if let Some(names) = column_names {
            if let Some(idx) = find_column_index_by_name(names, name) {
                if idx < row.len() {
                    return Some(row[idx].clone());
                }
            }
        }
    }

    // 2. Try NEW row
    if let Some(new) = ctx.trigger().new_row() {
        if let Some(names) = column_names {
            if let Some(idx) = find_column_index_by_name(names, name) {
                if idx < new.len() {
                    return Some(new[idx].clone());
                }
            }
        }
    }

    // 3. Try OLD row
    if let Some(old) = ctx.trigger().old_row() {
        if let Some(names) = column_names {
            if let Some(idx) = find_column_index_by_name(names, name) {
                if idx < old.len() {
                    return Some(old[idx].clone());
                }
            }
        }
    }

    None
}

fn find_column_index_by_name(names: &[String], name: &str) -> Option<usize> {
    names.iter().position(|n| n.eq_ignore_ascii_case(name))
}
```

**Step 6: Update tests and implementation**

Need to pass column_names to resolve_column. Let's update the signature and tests.

**Step 7: Run tests to verify they pass**

Run: `cargo test -p sqlrustgo-executor trigger::tests::test_resolve_column`
Expected: PASS

**Step 8: Commit**

```bash
git add crates/executor/src/trigger/resolver.rs
git commit -m "feat(trigger): add column resolver with priority order"
```

---

## Task 4: Implement expression_to_value

**Files:**
- Create: `crates/executor/src/trigger/expression.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_expression_literal_integer() {
    let trigger_ctx = TriggerContext::new(None, None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None, None);

    let expr = sqlrustgo_parser::Expression::Literal("42".to_string());
    let result = expression_to_value(&expr, &eval_ctx, None).unwrap();

    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_expression_binary_op_add() {
    let new_row = vec![Value::Integer(10)];
    let trigger_ctx = TriggerContext::new(Some(&new_row), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None, None);

    let expr = Expression::BinaryOp(
        Box::new(Expression::Literal("5".to_string())),
        "+".to_string(),
        Box::new(Expression::Literal("3".to_string())),
    );
    let result = expression_to_value(&expr, &eval_ctx, None).unwrap();

    assert_eq!(result, Value::Integer(8));
}

#[test]
fn test_expression_binary_op_sub_with_new() {
    // SET stock = stock - NEW.quantity
    let target = vec![Value::Integer(100)]; // stock
    let new_row = vec![Value::Integer(10)]; // quantity

    let trigger_ctx = TriggerContext::new(Some(&new_row), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, Some(&target), Some(&["stock".to_string()]));

    let expr = Expression::BinaryOp(
        Box::new(Expression::Identifier("stock".to_string())),
        "-".to_string(),
        Box::new(Expression::Identifier("NEW.quantity".to_string())),
    );
    let result = expression_to_value(&expr, &eval_ctx, Some(&["stock".to_string()])).unwrap();

    assert_eq!(result, Value::Integer(90));
}

#[test]
fn test_expression_null_propagation() {
    let trigger_ctx = TriggerContext::new(None, None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None, None);

    let expr = Expression::BinaryOp(
        Box::new(Expression::Literal("NULL".to_string())),
        "+".to_string(),
        Box::new(Expression::Literal("5".to_string())),
    );
    let result = expression_to_value(&expr, &eval_ctx, None).unwrap();

    assert_eq!(result, Value::Null);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-executor trigger::tests::test_expression`
Expected: FAIL - module not found

**Step 3: Write implementation**

```rust
// crates/executor/src/trigger/expression.rs

use crate::types::Value;
use crate::storage::Record;
use sqlrustgo_parser::Expression;
use super::context::EvalContext;
use super::resolver::resolve_column;

/// Evaluate an expression to a Value
/// Returns Value::Null on any error (fault tolerance)
pub fn expression_to_value(
    expr: &Expression,
    ctx: &EvalContext,
    column_names: Option<&[String]>,
) -> Result<Value> {
    match expr {
        Expression::Literal(s) => Ok(parse_literal(s)),

        Expression::Identifier(name) => {
            resolve_identifier(name, ctx, column_names)
                .ok_or_else(|| "Unknown column".into())
        }

        Expression::BinaryOp(left, op, right) => {
            let left_val = expression_to_value(left, ctx, column_names)?;
            let right_val = expression_to_value(right, ctx, column_names)?;
            Ok(eval_binary_op(&left_val, op, &right_val))
        }

        // Unsupported - return Null
        _ => Ok(Value::Null),
    }
}

fn parse_literal(s: &str) -> Value {
    let s = s.trim();
    if s.eq_ignore_ascii_case("NULL") {
        Value::Null
    } else if let Ok(n) = s.parse::<i64>() {
        Value::Integer(n)
    } else if let Ok(f) = s.parse::<f64>() {
        Value::Float(f)
    } else if s.starts_with('\'') && s.ends_with('\'') {
        Value::Text(s[1..s.len() - 1].to_string())
    } else {
        Value::Text(s.to_string())
    }
}

fn resolve_identifier(
    name: &str,
    ctx: &EvalContext,
    column_names: Option<&[String]>,
) -> Option<Value> {
    let name_upper = name.to_uppercase();

    // Handle NEW.col and OLD.col prefixes
    if name_upper.starts_with("NEW.") {
        let col_name = &name[4..];
        if let Some(new) = ctx.trigger().new_row() {
            if let Some(names) = column_names {
                if let Some(idx) = names.iter().position(|n| n.eq_ignore_ascii_case(col_name)) {
                    if idx < new.len() {
                        return Some(new[idx].clone());
                    }
                }
            }
        }
        return None;
    }

    if name_upper.starts_with("OLD.") {
        let col_name = &name[4..];
        if let Some(old) = ctx.trigger().old_row() {
            if let Some(names) = column_names {
                if let Some(idx) = names.iter().position(|n| n.eq_ignore_ascii_case(col_name)) {
                    if idx < old.len() {
                        return Some(old[idx].clone());
                    }
                }
            }
        }
        return None;
    }

    // Bare identifier - resolve via priority
    resolve_column(name, ctx, column_names)
}

fn eval_binary_op(left: &Value, op: &str, right: &Value) -> Value {
    // NULL propagation
    if matches!(left, Value::Null) || matches!(right, Value::Null) {
        return Value::Null;
    }

    match (left, right, op) {
        // Arithmetic
        (Value::Integer(a), Value::Integer(b), "+") => Value::Integer(a + b),
        (Value::Integer(a), Value::Integer(b), "-") => Value::Integer(a - b),
        (Value::Integer(a), Value::Integer(b), "*") => Value::Integer(a * b),
        (Value::Integer(a), Value::Integer(b), "/") => {
            if *b != 0 { Value::Integer(a / b) } else { Value::Null }
        }
        (Value::Float(a), Value::Float(b), "+") => Value::Float(a + b),
        (Value::Float(a), Value::Float(b), "-") => Value::Float(a - b),
        (Value::Float(a), Value::Float(b), "*") => Value::Float(a * b),
        (Value::Float(a), Value::Float(b), "/") => {
            if *b != 0.0 { Value::Float(a / b) } else { Value::Null }
        }
        // Mixed arithmetic
        (Value::Integer(a), Value::Float(b), "+") => Value::Float(*a as f64 + b),
        (Value::Float(a), Value::Integer(b), "+") => Value::Float(a + *b as f64),
        (Value::Integer(a), Value::Float(b), "-") => Value::Float(*a as f64 - b),
        (Value::Float(a), Value::Integer(b), "-") => Value::Float(a - *b as f64),

        // Comparisons
        (Value::Integer(a), Value::Integer(b), "=") => Value::Boolean(a == b),
        (Value::Integer(a), Value::Integer(b), "!=") => Value::Boolean(a != b),
        (Value::Integer(a), Value::Integer(b), "<>") => Value::Boolean(a != b),
        (Value::Integer(a), Value::Integer(b), ">") => Value::Boolean(a > b),
        (Value::Integer(a), Value::Integer(b), "<") => Value::Boolean(a < b),
        (Value::Integer(a), Value::Integer(b), ">=") => Value::Boolean(a >= b),
        (Value::Integer(a), Value::Integer(b), "<=") => Value::Boolean(a <= b),

        _ => Value::Null,
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sqlrustgo-executor trigger::tests::test_expression`
Expected: PASS (after fixing any compilation errors)

**Step 5: Commit**

```bash
git add crates/executor/src/trigger/expression.rs
git commit -m "feat(trigger): implement expression_to_value with BinaryOp support"
```

---

## Task 5: Implement expression_to_bool

**Files:**
- Modify: `crates/executor/src/trigger/expression.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_bool_true() {
    let trigger_ctx = TriggerContext::new(None, None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None, None);

    let expr = Expression::BinaryOp(
        Box::new(Expression::Literal("5".to_string())),
        ">".to_string(),
        Box::new(Expression::Literal("3".to_string())),
    );
    let result = expression_to_bool(&expr, &eval_ctx, None);
    assert!(result);
}

#[test]
fn test_bool_false() {
    let trigger_ctx = TriggerContext::new(None, None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None, None);

    let expr = Expression::BinaryOp(
        Box::new(Expression::Literal("5".to_string())),
        "<".to_string(),
        Box::new(Expression::Literal("3".to_string())),
    );
    let result = expression_to_bool(&expr, &eval_ctx, None);
    assert!(!result);
}

#[test]
fn test_bool_null_is_false() {
    let trigger_ctx = TriggerContext::new(None, None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None, None);

    let expr = Expression::Literal("NULL".to_string());
    let result = expression_to_bool(&expr, &eval_ctx, None);
    assert!(!result);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-executor trigger::tests::test_bool`
Expected: FAIL - function not found

**Step 3: Write implementation**

```rust
// Add to crates/executor/src/trigger/expression.rs

/// Evaluate expression to boolean
/// Returns false for any error, NULL, or non-true value
pub fn expression_to_bool(
    expr: &Expression,
    ctx: &EvalContext,
    column_names: Option<&[String]>,
) -> bool {
    match expression_to_value(expr, ctx, column_names) {
        Ok(Value::Boolean(b)) => b,
        _ => false,
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-executor trigger::tests::test_bool`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/executor/src/trigger/expression.rs
git commit -m "feat(trigger): add expression_to_bool function"
```

---

## Task 6: Update execute_trigger_update

**Files:**
- Modify: `crates/executor/src/trigger.rs`

**Step 1: Write failing test (integration)**

```rust
#[test]
fn test_trigger_update_with_expression() {
    // This test should pass after Task 6 implementation
    // Test: UPDATE inventory SET stock = stock - NEW.quantity WHERE product_id = NEW.product_id
}
```

**Step 2: Run test to verify current behavior**

Run: `cargo test --test stored_proc_catalog_test test_trigger_executes_insert`
Expected: FAIL (before fix)

**Step 3: Update execute_trigger_update**

```rust
// In trigger.rs, replace execute_trigger_update function

fn execute_trigger_update(
    &self,
    sql: &str,
    trigger_table: &str,
    new_row: Option<&Record>,
) -> SqlResult<()> {
    let expanded = self.expand_update_values(sql, trigger_table, new_row);
    let statement = parse(&expanded)
        .map_err(|e| SqlError::ExecutionError(format!("Parse error: {}", e)))?;

    let mut storage = self.storage.write().unwrap();

    if let sqlrustgo_parser::Statement::Update(update) = statement {
        let table_name = &update.table;
        let table_info = storage.get_table_info(table_name)?;
        let target_col_names: Vec<String> = table_info.columns.iter().map(|c| c.name.clone()).collect();

        // Scan all rows (WHERE optimization can be added later)
        let current_rows = storage.scan(table_name)?;

        for row in current_rows {
            let trigger_ctx = TriggerContext::new(new_row, None);
            let eval_ctx = EvalContext::new(&trigger_ctx, Some(&row), Some(&target_col_names));

            // Evaluate WHERE clause
            if let Some(where_expr) = &update.where_clause {
                if !expression_to_bool(where_expr, &eval_ctx, Some(&target_col_names)) {
                    continue;
                }
            }

            // Evaluate SET clauses
            let set_updates: Vec<(usize, Value)> = update
                .set_clauses
                .iter()
                .filter_map(|(col_name, expr)| {
                    let col_idx = table_info.columns.iter()
                        .position(|c| c.name.eq_ignore_ascii_case(col_name));
                    col_idx.map(|idx| {
                        let val = expression_to_value(expr, &eval_ctx, Some(&target_col_names))
                            .unwrap_or(Value::Null);
                        (idx, val)
                    })
                })
                .collect();

            if !set_updates.is_empty() {
                storage.update(table_name, &[], &set_updates)?;
            }
        }
    }
    Ok(())
}
```

**Step 4: Run integration test**

Run: `cargo test --test stored_proc_catalog_test test_trigger_executes_insert`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/executor/src/trigger.rs
git commit -m "fix(trigger): update execute_trigger_update with expression evaluation"
```

---

## Task 7: Update execute_trigger_insert

**Files:**
- Modify: `crates/executor/src/trigger.rs`

**Step 1: Update function**

```rust
fn execute_trigger_insert(&self, sql: &str, new_row: Option<&Record>) -> SqlResult<()> {
    let mut storage = self.storage.write().unwrap();
    let expanded = self.expand_insert_values(sql, new_row);
    let statement = parse(&expanded)
        .map_err(|e| SqlError::ExecutionError(format!("Parse error: {}", e)))?;

    if let sqlrustgo_parser::Statement::Insert(insert) = statement {
        let table_name = &insert.table;
        let table_info = storage.get_table_info(table_name)?;
        let target_col_names: Vec<String> = table_info.columns.iter().map(|c| c.name.clone()).collect();

        let trigger_ctx = TriggerContext::new(new_row, None);

        for values in &insert.values {
            let mut record = Vec::new();
            for expr in values {
                let eval_ctx = EvalContext::new(&trigger_ctx, None, Some(&target_col_names));
                let val = expression_to_value(expr, &eval_ctx, Some(&target_col_names))
                    .unwrap_or(Value::Null);
                record.push(val);
            }
            while record.len() < table_info.columns.len() {
                record.push(Value::Null);
            }
            storage.insert(table_name, vec![record])?;
        }
    }
    Ok(())
}
```

**Step 2: Run tests**

Run: `cargo test --test stored_proc_catalog_test test_trigger_executes_update`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/executor/src/trigger.rs
git commit -m "fix(trigger): update execute_trigger_insert with expression evaluation"
```

---

## Task 8: Run All Tests

**Step 1: Run full test suite**

Run: `cargo test --test stored_proc_catalog_test`
Expected: All 16 tests pass

**Step 2: Run clippy**

Run: `cargo clippy -p sqlrustgo-executor -- -D warnings`
Expected: No warnings

**Step 3: Run format check**

Run: `cargo fmt --check --all`
Expected: No formatting issues

**Step 4: Commit final**

```bash
git add -A
git commit -m "fix(trigger): complete trigger expression evaluation system"
```

---

## Verification Checklist

- [ ] All 16 stored_proc_catalog_test tests pass
- [ ] All unit tests in trigger module pass
- [ ] `cargo clippy --all-features -- -D warnings` passes
- [ ] `cargo fmt --check --all` passes
- [ ] Documentation links valid (`bash scripts/gate/check_docs_links.sh`)
