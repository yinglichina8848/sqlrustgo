# Trigger Expression Evaluation System Design

**Date**: 2026-04-23
**Author**: AI Assistant
**Status**: Approved

## 1. Overview

This document describes the design for a complete trigger expression evaluation system in SQLRustGo. The system enables triggers to execute complex DML statements (INSERT/UPDATE/DELETE) within trigger bodies, with proper expression evaluation including binary operations and column reference resolution.

## 2. Problem Statement

The current `expression_to_value` function in `trigger.rs` is a stub implementation that only handles:
- Literal values (strings, numbers)
- Two special identifiers: `NEW.id` and `NEW.col1`

This prevents triggers from executing DML like:
```sql
UPDATE inventory SET stock = stock - NEW.quantity WHERE product_id = NEW.product_id
```

## 3. Design Goals

1. **Complete Expression Evaluation**: Support BinaryOp expressions (`+`, `-`, `*`, `/`, `=`, `>`, `<`, etc.)
2. **Correct Column Resolution**: Priority order: target_row → NEW → OLD
3. **Fault Tolerance**: Any evaluation error returns `Value::Null`
4. **WHERE Optimization**: Extract indexable predicates, fallback to row-by-row evaluation
5. **NULL Propagation**: Any NULL operand results in NULL output

## 4. Architecture

### 4.1 Module Structure

```
crates/executor/src/
├── trigger.rs              # TriggerExecutor (modified)
└── trigger/
    ├── mod.rs             # Module exports
    ├── context.rs         # EvalContext, TriggerContext
    ├── resolver.rs         # Column name resolution
    └── expression.rs      # expression_to_value, expression_to_bool
```

### 4.2 Core Data Structures

```rust
/// Trigger execution context (NEW/OLD rows)
struct TriggerContext<'a> {
    pub new: Option<&'a Row>,
    pub old: Option<&'a Row>,
}

/// Evaluation context for expressions
struct EvalContext<'a> {
    pub trigger_ctx: &'a TriggerContext<'a>,
    pub target_row: Option<&'a Row>,
    pub target_schema: Option<&'a Schema>,
}

/// Constraint types for WHERE optimization
enum Constraint {
    Eq { col: String, value: Value },
    Range { col: String, lower: Option<Value>, upper: Option<Value> },
}

/// Predicate analysis result
struct PredicateAnalysis {
    pub indexable: Vec<IndexablePredicate>,
    pub residual: Vec<Expr>,
}
```

## 5. Expression Evaluation

### 5.1 Column Resolution Priority

```rust
fn resolve_column(name: &str, ctx: &EvalContext) -> Option<Value> {
    // 1. target_row (for UPDATE SET clause and WHERE)
    if let Some(row) = ctx.target_row {
        if let Some(v) = row.get(name) { return Some(v.clone()); }
    }
    // 2. NEW
    if let Some(new) = ctx.trigger_ctx.new {
        if let Some(v) = new.get(name) { return Some(v.clone()); }
    }
    // 3. OLD
    if let Some(old) = ctx.trigger_ctx.old {
        if let Some(v) = old.get(name) { return Some(v.clone()); }
    }
    None
}
```

### 5.2 expression_to_value Signature

```rust
fn expression_to_value(
    expr: &Expr,
    ctx: &EvalContext,
) -> Result<Value> {
    // Returns Value::Null on any error (fault tolerance)
}
```

### 5.3 Supported Expression Types

| Type | Behavior |
|------|----------|
| `Literal` | Parse string/number/boolean |
| `Identifier` | Resolve via priority (target → NEW → OLD) |
| `BinaryOp` | Evaluate left/right, apply operator |
| `FunctionCall` | Return Null (not supported in v1) |
| `Subquery` | Return Null (not supported in v1) |

### 5.4 BinaryOp Operators

| Operator | Type | Behavior |
|----------|------|----------|
| `+` | Arithmetic | Integer/Float addition, NULL propagation |
| `-` | Arithmetic | Integer/Float subtraction, NULL propagation |
| `*` | Arithmetic | Integer/Float multiplication, NULL propagation |
| `/` | Arithmetic | Integer/Float division, NULL on division by zero |
| `=` | Comparison | Boolean equality |
| `!=`, `<>` | Comparison | Boolean inequality |
| `>` | Comparison | Greater than |
| `<` | Comparison | Less than |
| `>=` | Comparison | Greater than or equal |
| `<=` | Comparison | Less than or equal |

### 5.5 NULL Propagation

```rust
match (left_val, right_val, op) {
    (Value::Null, _, _) => Value::Null,
    (_, Value::Null, _) => Value::Null,
    // ... operators
}
```

## 6. WHERE Optimization (Two-Phase)

### 6.1 Predicate Extraction

Extract indexable predicates from AND-connected WHERE conditions:

**Supported Patterns**:
```sql
col = literal
col = NEW.column
col = OLD.column
col > literal (and variants)
col < literal (and variants)
```

**Non-Indexable** (fallback to row-by-row):
```sql
stock - NEW.quantity > 0
A OR B
```

### 6.2 Extraction Algorithm

```rust
fn try_extract_predicate(
    expr: &Expr,
    ctx: &EvalContext,
) -> Option<IndexablePredicate> {
    match expr {
        Expr::BinaryOp { left, op, right } => {
            match (left.as_ref(), right.as_ref()) {
                // col = value
                (Expr::Column(col), r) => {
                    if let Some(val) = try_eval_const(r, ctx) {
                        return Some(IndexablePredicate {
                            column: col.clone(),
                            op: *op,
                            value: val,
                        });
                    }
                }
                // value = col (commutative)
                (l, Expr::Column(col)) if is_commutative(op) => {
                    if let Some(val) = try_eval_const(l, ctx) {
                        return Some(IndexablePredicate {
                            column: col.clone(),
                            op: *op,
                            value: val,
                        });
                    }
                }
                _ => None
            }
        }
        _ => None
    }
}

fn try_eval_const(expr: &Expr, ctx: &EvalContext) -> Option<Value> {
    match expr {
        Expr::Literal(v) => Some(v.clone()),
        Expr::Identifier(name) => {
            // Only NEW/OLD allowed, not target_row
            resolve_trigger_column(name, ctx)
        }
        _ => None
    }
}
```

### 6.3 Execution Strategy

```rust
fn execute_update(stmt: &UpdateStmt, ctx: &TriggerContext) -> SqlResult<()> {
    // Phase 1: Analyze WHERE
    let analysis = analyze_predicate(stmt.where_expr, ctx);

    // Phase 2: Get candidate rows
    let rows = if !analysis.indexable.is_empty() {
        // Try index lookup first
        storage.scan_with_constraints(&analysis.indexable)?
    } else {
        // Fallback: full scan
        storage.scan(table_name)?
    };

    // Phase 3: Row-by-row evaluation
    for row in rows {
        let row_ctx = ctx.with_target_row(Some(&row));

        // Check residual conditions
        if !eval_all_bool(&analysis.residual, &row_ctx) {
            continue;
        }

        // Apply SET
        for (col_idx, expr) in stmt.set_clauses {
            let val = expression_to_value(expr, &row_ctx)?;
            row[col_idx] = val;
        }
        storage.update(&row)?;
    }
    Ok(())
}
```

## 7. Trigger DML Execution

### 7.1 INSERT Execution

```rust
fn execute_trigger_insert(stmt: &InsertStmt, ctx: &TriggerContext) -> SqlResult<()> {
    for values in stmt.values {
        let mut row = Vec::new();
        for expr in values {
            let val = expression_to_value(expr, &ctx.into_eval())?;
            row.push(val);
        }
        storage.insert(table_name, vec![row])?;
    }
    Ok(())
}
```

### 7.2 UPDATE Execution (as described in Section 6.3)

### 7.3 DELETE Execution

```rust
fn execute_trigger_delete(stmt: &DeleteStmt, ctx: &TriggerContext) -> SqlResult<()> {
    let analysis = analyze_predicate(stmt.where_expr, ctx);
    let rows = storage.scan_with_constraints(&analysis.indexable)?;

    for row in rows {
        let row_ctx = ctx.with_target_row(Some(&row));
        if !eval_all_bool(&analysis.residual, &row_ctx) {
            continue;
        }
        storage.delete(table_name, row.id())?;
    }
    Ok(())
}
```

## 8. Error Handling

### 8.1 Fault Tolerance Rules

| Error | Result |
|-------|--------|
| Unknown column | `Value::Null` |
| Type mismatch | `Value::Null` |
| Division by zero | `Value::Null` |
| Parse error | `Value::Null` |
| Storage error | Return `SqlResult::Err` |

### 8.2 Boolean Evaluation

```rust
fn expression_to_bool(expr: &Expr, ctx: &EvalContext) -> bool {
    match expression_to_value(expr, ctx) {
        Ok(Value::Bool(b)) => b,
        Ok(Value::Null) => false,
        Ok(_) => false,
        Err(_) => false,
    }
}
```

## 9. Limitations (V1)

1. **Max trigger depth = 1**: No trigger recursion
2. **No functions**: `FunctionCall` returns Null
3. **No subqueries**: Returns Null
4. **No IN operator**: Falls back to full scan
5. **No OR optimization**: Falls back to full scan
6. **Single column index only**: No composite indexes

## 10. Test Cases

### 10.1 Unit Tests

| Test | Description |
|------|-------------|
| `test_literal_integer` | Parse integer literals |
| `test_literal_float` | Parse float literals |
| `test_literal_string` | Parse string literals |
| `test_identifier_new` | Resolve NEW.column |
| `test_identifier_old` | Resolve OLD.column |
| `test_identifier_target` | Resolve target row column |
| `test_binary_op_add` | Integer addition |
| `test_binary_op_sub` | Integer subtraction |
| `test_binary_op_null` | NULL propagation |
| `test_bool_true` | Boolean TRUE |
| `test_bool_false` | Boolean FALSE |
| `test_bool_null` | NULL evaluates to false |

### 10.2 Integration Tests

| Test | SQL |
|------|-----|
| `test_trigger_executes_insert` | AFTER INSERT → UPDATE |
| `test_trigger_executes_update` | AFTER UPDATE → INSERT |
| `test_trigger_executes_delete` | AFTER DELETE → INSERT |

## 11. Implementation Order

1. Create `trigger/context.rs` - EvalContext, TriggerContext
2. Create `trigger/resolver.rs` - Column resolution logic
3. Create `trigger/expression.rs` - expression_to_value, expression_to_bool
4. Update `trigger.rs` - Use new expression system
5. Add unit tests
6. Verify integration tests pass

## 12. File Changes Summary

| File | Action |
|------|--------|
| `crates/executor/src/trigger.rs` | Modify execute_*_trigger functions |
| `crates/executor/src/trigger/mod.rs` | Create |
| `crates/executor/src/trigger/context.rs` | Create |
| `crates/executor/src/trigger/resolver.rs` | Create |
| `crates/executor/src/trigger/expression.rs` | Create |
| `tests/stored_proc_catalog_test.rs` | No change (existing tests) |
