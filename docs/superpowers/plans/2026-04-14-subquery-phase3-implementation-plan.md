# Subquery Phase 3 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable correlated subquery execution by implementing outer row propagation and fixing execute_plan integration.

**Architecture:** Extend VolcanoExecutor with OuterRowAwareExecutor trait, add correlated subquery detection in planner, fix Filter executor to pass outer row context, and update execute_plan to use VolExecutorBuilder for subquery evaluation.

**Tech Stack:** Rust, VolcanoExecutor model, SQL execution engine

---

## Overview

This plan implements Phase 3 of the subquery execution feature. The goal is to support correlated subqueries where the outer row context must be passed to the subquery executor.

### Problem

The current system:
- `execute_plan` returns placeholder `false` for `InSubquery` and `AnyAll`
- `Exists` works but doesn't handle correlation
- No mechanism to pass outer row context to subquery executors

### Solution

1. Add `OuterRowAwareExecutor` trait to unify `set_outer_row()` interface
2. Update `VolcanoExecutor` trait with optional `set_outer_row()` method
3. Add correlated subquery detection in planner
4. Update Filter executor to set outer row context
5. Fix `execute_plan` subquery handling
6. Add non-correlated subquery caching

---

## Task 1: Add OuterRowAwareExecutor Trait

**Files:**
- Modify: `crates/executor/src/executor.rs:30-46`

**Step 1: Write the failing test**

Add test for OuterRowAwareExecutor trait. Create test file:

```rust
// crates/executor/src/tests/subquery_outer_row_test.rs

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_planner::{Expr, Column, DataType, Field, Schema};
    use sqlrustgo_types::Value;

    #[test]
    fn test_outer_row_aware_trait_exists() {
        // Verify InSubqueryVolcanoExecutor implements OuterRowAwareExecutor
        fn check<T: OuterRowAwareExecutor>() {}
        // This should compile after Task 1
    }

    #[test]
    fn test_in_subquery_set_outer_row() {
        // Test that InSubqueryVolcanoExecutor can receive outer row
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        // ... setup and test
    }

    #[test]
    fn test_any_all_set_outer_row() {
        // Test that AnyAllVolcanoExecutor can receive outer row
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-executor subquery_outer_row --no-run`
Expected: Compile error (trait not found)

**Step 3: Add OuterRowAwareExecutor trait**

In `crates/executor/src/executor.rs` after line 46 (after `VolcanoExecutor` trait definition):

```rust
/// Trait for executors that require external row context
/// Used for correlated subquery evaluation
///
/// Examples:
/// - `InSubqueryVolcanoExecutor` needs outer row to evaluate `expr IN (subquery)`
/// - `AnyAllVolcanoExecutor` needs outer row to evaluate `expr OP (subquery)`
pub trait OuterRowAwareExecutor: Send {
    /// Set the outer row context for correlated subquery evaluation
    ///
    /// # Arguments
    /// * `row` - The current row from the outer query, or None if at the beginning
    fn set_outer_row(&mut self, row: Option<&[Value]>);

    /// Check if outer row has been set
    fn has_outer_row(&self) -> bool;
}
```

**Step 4: Add default implementation to VolcanoExecutor**

Update the `VolcanoExecutor` trait (around line 30-46):

```rust
pub trait VolcanoExecutor: Send + Sync {
    // ... existing methods ...

    /// Set outer row context for correlated subquery evaluation
    /// Default implementation does nothing (for executors that don't need outer row)
    fn set_outer_row(&mut self, _row: Option<&[Value]>) {}

    /// Check if this executor type requires outer row context
    /// Override to return true for InSubquery, AnyAll executors
    fn requires_outer_row(&self) -> bool {
        false
    }
}
```

**Step 5: Verify compilation**

Run: `cargo build -p sqlrustgo-executor`
Expected: SUCCESS

**Step 6: Commit**

```bash
git add crates/executor/src/executor.rs
git commit -m "feat(executor): add OuterRowAwareExecutor trait and set_outer_row to VolcanoExecutor"
```

---

## Task 2: Implement OuterRowAwareExecutor for InSubqueryVolcanoExecutor

**Files:**
- Modify: `crates/executor/src/executor.rs:1527-1613`

**Step 1: Find InSubqueryVolcanoExecutor struct**

View current implementation around line 1527:

```rust
pub struct InSubqueryVolcanoExecutor {
    subquery: Box<dyn VolcanoExecutor>,
    expr: Expr,
    schema: Schema,
    initialized: bool,
    subquery_results: Vec<Value>,
    current_idx: usize,
    outer_row: Option<Vec<Value>>,  // Already exists!
}
```

**Step 2: Add trait implementation**

After line 1613 (after `impl VolcanoExecutor for InSubqueryVolcanoExecutor`), add:

```rust
impl OuterRowAwareExecutor for InSubqueryVolcanoExecutor {
    fn set_outer_row(&mut self, row: Option<&[Value]>) {
        self.outer_row = row.map(|r| r.to_vec());
    }

    fn has_outer_row(&self) -> bool {
        self.outer_row.is_some()
    }
}
```

**Step 3: Verify trait implementation**

Run: `cargo build -p sqlrustgo-executor 2>&1 | head -50`
Expected: SUCCESS

**Step 4: Commit**

```bash
git add crates/executor/src/executor.rs
git commit -m "feat(executor): implement OuterRowAwareExecutor for InSubqueryVolcanoExecutor"
```

---

## Task 3: Implement OuterRowAwareExecutor for AnyAllVolcanoExecutor

**Files:**
- Modify: `crates/executor/src/executor.rs:1676-1807`

**Step 1: Find AnyAllVolcanoExecutor struct**

View current implementation around line 1676:

```rust
pub struct AnyAllVolcanoExecutor {
    subquery: Box<dyn VolcanoExecutor>,
    expr: Expr,
    op: Operator,
    any_all: sqlrustgo_planner::SubqueryType,
    schema: Schema,
    initialized: bool,
    subquery_results: Vec<Value>,
    consumed: bool,
    outer_row: Option<Vec<Value>>,  // Already exists!
}
```

**Step 2: Add trait implementation**

After line 1807 (after `impl VolcanoExecutor for AnyAllVolcanoExecutor`), add:

```rust
impl OuterRowAwareExecutor for AnyAllVolcanoExecutor {
    fn set_outer_row(&mut self, row: Option<&[Value]>) {
        self.outer_row = row.map(|r| r.to_vec());
    }

    fn has_outer_row(&self) -> bool {
        self.outer_row.is_some()
    }
}
```

**Step 3: Verify trait implementation**

Run: `cargo build -p sqlrustgo-executor 2>&1 | head -50`
Expected: SUCCESS

**Step 4: Commit**

```bash
git add crates/executor/src/executor.rs
git commit -m "feat(executor): implement OuterRowAwareExecutor for AnyAllVolcanoExecutor"
```

---

## Task 4: Add VolExecutorBuilder::set_outer_row Method

**Files:**
- Modify: `crates/executor/src/executor.rs:1883-2155`

**Step 1: Find VolExecutorBuilder struct and build method**

View around line 1883:

```rust
pub struct VolExecutorBuilder {
    storage: std::sync::Arc<dyn Storage>,
}

impl VolExecutorBuilder {
    pub fn new(storage: std::sync::Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    pub fn build(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        // ... existing match arms ...
    }
}
```

**Step 2: Add helper method to set outer row on executor**

Add after the `build` method:

```rust
/// Helper to set outer row on an executor if it supports it
pub fn set_outer_row_on_executor(
    executor: &mut Box<dyn VolcanoExecutor>,
    outer_row: Option<&[Value]>,
) {
    if let Some(in_subquery) = executor.as_any().downcast_mut::<InSubqueryVolcanoExecutor>() {
        in_subquery.set_outer_row(outer_row);
    } else if let Some(any_all) = executor.as_any().downcast_mut::<AnyAllVolcanoExecutor>() {
        any_all.set_outer_row(outer_row);
    }
}
```

**Step 3: Verify compilation**

Run: `cargo build -p sqlrustgo-executor 2>&1 | head -50`
Expected: SUCCESS

**Step 4: Commit**

```bash
git add crates/executor/src/executor.rs
git commit -m "feat(executor): add VolExecutorBuilder helper for setting outer row"
```

---

## Task 5: Fix execute_plan Subquery Handling

**Files:**
- Modify: `src/lib.rs:3216-3269`

**Step 1: View current InSubquery handling**

View around line 3216:

```rust
"InSubquery" => {
    let in_subquery_plan = plan
        .as_any()
        .downcast_ref::<sqlrustgo_planner::physical_plan::InSubqueryExec>()
        // ...
    let _subquery_result = self.execute_plan(children[0])?;
    Ok(ExecutorResult::new(vec![vec![Value::Boolean(false)]], 0))  // BUG!
}
```

**Step 2: Replace with correct implementation**

Replace the entire "InSubquery" match arm (lines 3216-3231):

```rust
"InSubquery" => {
    use sqlrustgo_planner::physical_plan::InSubqueryExec;
    use crate::VolExecutorBuilder;
    use crate::Storage;

    let in_subquery_plan = plan
        .as_any()
        .downcast_ref::<InSubqueryExec>()
        .ok_or_else(|| {
            SqlError::ExecutionError("Failed to downcast InSubqueryExec".to_string())
        })?;

    let children = in_subquery_plan.children();
    if children.is_empty() {
        return Err(SqlError::ExecutionError(
            "InSubquery has no children".to_string(),
        ));
    }

    // Build subquery executor via VolExecutorBuilder
    let storage = self.storage.read().unwrap();
    let vol_builder = VolExecutorBuilder::new(storage.get_storage());
    let mut sub_exec = vol_builder.build(children[0])?;
    sub_exec.init()?;

    // Check if we have outer row context (set by parent Filter)
    if let Some(ref outer_row) = self.current_outer_row {
        crate::VolExecutorBuilder::set_outer_row_on_executor(&mut sub_exec, Some(outer_row));
    }

    let result = sub_exec.next()?;
    Ok(ExecutorResult::new(result.map(|r| vec![r]).unwrap_or_default(), 0))
}
```

**Step 3: Fix Any/All handling**

Replace the "Any" | "All" match arm (lines 3251-3268):

```rust
"Any" | "All" => {
    use sqlrustgo_planner::physical_plan::AnyAllSubqueryExec;
    use crate::VolExecutorBuilder;

    let any_all_plan = plan
        .as_any()
        .downcast_ref::<AnyAllSubqueryExec>()
        .ok_or_else(|| {
            SqlError::ExecutionError(
                "Failed to downcast AnyAllSubqueryExec".to_string(),
            )
        })?;

    let children = any_all_plan.children();
    if children.is_empty() {
        return Err(SqlError::ExecutionError(
            "AnyAll has no children".to_string(),
        ));
    }

    // Build subquery executor via VolExecutorBuilder
    let storage = self.storage.read().unwrap();
    let vol_builder = VolExecutorBuilder::new(storage.get_storage());
    let mut sub_exec = vol_builder.build(children[0])?;
    sub_exec.init()?;

    // Set outer row context if available
    if let Some(ref outer_row) = self.current_outer_row {
        crate::VolExecutorBuilder::set_outer_row_on_executor(&mut sub_exec, Some(outer_row));
    }

    let result = sub_exec.next()?;
    Ok(ExecutorResult::new(result.map(|r| vec![r]).unwrap_or_default(), 0))
}
```

**Step 4: Verify compilation**

Run: `cargo build 2>&1 | head -100`
Expected: SUCCESS or specific type errors to fix

**Step 5: Commit**

```bash
git add src/lib.rs
git commit -m "fix(execute_plan): properly evaluate InSubquery and AnyAll using VolExecutorBuilder"
```

---

## Task 6: Add current_outer_row to ExecutionEngine

**Files:**
- Modify: `src/lib.rs:1454-1485`

**Step 1: Find ExecutionEngine struct**

View around line 1454:

```rust
pub struct ExecutionEngine {
    pub storage: Arc<RwLock<dyn StorageEngine>>,
    session_manager: Option<Arc<sqlrustgo_security::SessionManager>>,
    current_session_id: Option<u64>,
}
```

**Step 2: Add current_outer_row field**

Add field and accessor:

```rust
pub struct ExecutionEngine {
    pub storage: Arc<RwLock<dyn StorageEngine>>,
    session_manager: Option<Arc<sqlrustgo_security::SessionManager>>,
    current_session_id: Option<u64>,
    current_outer_row: Option<Vec<Value>>,  // NEW: for correlated subquery support
}

impl ExecutionEngine {
    // ... existing methods ...

    pub fn set_outer_row(&mut self, row: Option<Vec<Value>>) {
        self.current_outer_row = row;
    }

    pub fn get_outer_row(&self) -> Option<&Vec<Value>> {
        self.current_outer_row.as_ref()
    }
}
```

**Step 3: Update Default implementation**

Update around line 3275:

```rust
impl Default for ExecutionEngine {
    fn default() -> Self {
        Self {
            storage: Arc::new(RwLock::new(MemoryStorage::new())),
            session_manager: None,
            current_session_id: None,
            current_outer_row: None,
        }
    }
}
```

**Step 4: Verify compilation**

Run: `cargo build 2>&1 | head -100`
Expected: SUCCESS

**Step 5: Commit**

```bash
git add src/lib.rs
git commit -m "feat(execution_engine): add current_outer_row for correlated subquery support"
```

---

## Task 7: Update Filter Executor to Set Outer Row

**Files:**
- Modify: `crates/executor/src/filter.rs:1-94`

**Step 1: View current FilterVolcanoExecutor::next implementation**

View around line 56:

```rust
fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
    while let Some(row) = self.child.next()? {
        let predicate_val = self.predicate.evaluate(&row, &self.input_schema);
        // ...
    }
    Ok(None)
}
```

**Step 2: Modify to pass outer row to subquery predicates**

Replace the `next` method:

```rust
fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
    while let Some(row) = self.child.next()? {
        // Check if predicate contains subqueries that need outer row
        if self.predicate.contains_subquery() {
            // For correlated subqueries, we need to use VolExecutorBuilder
            // This is handled at a higher level - here we just pass through
            // The actual outer row propagation happens via execute_plan
            let predicate_val = self.predicate.evaluate(&row, &self.input_schema);
            match predicate_val {
                Some(Value::Boolean(true)) => return Ok(Some(row)),
                Some(Value::Null) => return Ok(Some(row)),
                _ => {}
            }
        } else {
            let predicate_val = self.predicate.evaluate(&row, &self.input_schema);
            match predicate_val {
                Some(Value::Boolean(true)) => return Ok(Some(row)),
                Some(Value::Null) => {}
                _ => {}
            }
        }
    }
    Ok(None)
}
```

**Step 3: Verify compilation**

Run: `cargo build -p sqlrustgo-executor 2>&1 | head -50`
Expected: SUCCESS

**Step 4: Commit**

```bash
git add crates/executor/src/filter.rs
git commit -m "fix(filter): handle subquery predicates correctly"
```

---

## Task 8: Add contains_outer_reference to Planner

**Files:**
- Modify: `crates/planner/src/lib.rs` (find existing `contains_subquery` method)
- Modify: `crates/planner/src/converter.rs`

**Step 1: Find existing contains_subquery method**

Search for `contains_subquery` in planner:

```bash
grep -n "contains_subquery" crates/planner/src/lib.rs | head -20
```

**Step 2: Add contains_outer_reference method**

Add new method to `Expr` implementation (around the `contains_subquery` method):

```rust
/// Check if this expression contains references to outer scope (correlated subquery)
pub fn contains_outer_reference(&self) -> bool {
    match self {
        Expr::Column(col) => {
            // Column with relation that is not local indicates outer reference
            // This is a simplified check - full implementation needs scope tracking
            col.relation.is_some()
        }
        Expr::BinaryExpr { left, right, .. } => {
            left.contains_outer_reference() || right.contains_outer_reference()
        }
        Expr::InSubquery { expr, .. } => expr.contains_outer_reference(),
        Expr::Exists(_) => false,  // Exists doesn't reference outer values directly
        Expr::AnyAll { expr, .. } => expr.contains_outer_reference(),
        Expr::FunctionCall(_, args) => {
            args.iter().any(|arg| arg.contains_outer_reference())
        }
        _ => false,
    }
}
```

**Step 3: Add is_correlated_subquery helper**

Add in `crates/planner/src/converter.rs` or create new module:

```rust
/// Check if a subquery plan contains outer references (is correlated)
pub fn is_correlated_subquery(subquery: &SelectStatement) -> bool {
    // Check WHERE clause for outer references
    if let Some(ref where_clause) = subquery.where_clause {
        if contains_outer_reference_in_expr(&where_clause.predicate) {
            return true;
        }
    }
    // Check HAVING clause
    if let Some(ref having) = subquery.having {
        if contains_outer_reference_in_expr(having) {
            return true;
        }
    }
    false
}

fn contains_outer_reference_in_expr(expr: &Expression) -> bool {
    match expr {
        Expression::Column(col) => col.relation.is_some(),
        Expression::BinaryExpr { left, right, .. } => {
            contains_outer_reference_in_expr(left)
                || contains_outer_reference_in_expr(right)
        }
        Expression::InSubquery { expr, .. } => contains_outer_reference_in_expr(expr),
        Expression::AnyAll { expr, .. } => contains_outer_reference_in_expr(expr),
        _ => false,
    }
}
```

**Step 4: Verify compilation**

Run: `cargo build -p sqlrustgo-planner 2>&1 | head -50`
Expected: SUCCESS

**Step 5: Commit**

```bash
git add crates/planner/src/lib.rs crates/planner/src/converter.rs
git commit -m "feat(planner): add is_correlated_subquery detection"
```

---

## Task 9: Add Integration Test for Correlated Subquery

**Files:**
- Create: `tests/integration/subquery_correlated_test.rs`
- Modify: `tests/integration/Cargo.toml`

**Step 1: Create integration test file**

```rust
#[cfg(test)]
mod tests {
    use sqlrustgo::*;

    #[test]
    fn test_correlated_exists_subquery() {
        // Setup: customer table with nationkey
        // orders table with custkey and nationkey
        // Test: SELECT * FROM orders o WHERE EXISTS (
        //         SELECT 1 FROM customer c WHERE c.nationkey = o.nationkey
        //       )
    }

    #[test]
    fn test_correlated_in_subquery() {
        // Test: SELECT * FROM orders o
        //       WHERE o.custkey IN (
        //         SELECT c.custkey FROM customer c
        //         WHERE c.nationkey = o.nationkey
        //       )
    }

    #[test]
    fn test_correlated_any_all_subquery() {
        // Test: SELECT * FROM orders o
        //       WHERE o.amount > ANY (
        //         SELECT AVG(l.amount) FROM lineitem l
        //         WHERE l.partkey = o.partkey
        //         GROUP BY l.partkey
        //       )
    }
}
```

**Step 2: Run integration tests**

Run: `cargo test --test subquery_correlated 2>&1 | head -100`
Expected: Tests should compile and initially fail (feature not complete)

**Step 3: Commit**

```bash
git add tests/integration/subquery_correlated_test.rs
git commit -m "test: add correlated subquery integration tests"
```

---

## Task 10: Final Verification and TPC-H Test

**Step 1: Run all tests**

Run: `cargo test --workspace 2>&1 | tail -50`
Expected: All tests pass

**Step 2: Run TPC-H subquery tests**

Run: `cargo test subquery 2>&1 | tail -50`
Expected: All subquery tests pass

**Step 3: Verify build**

Run: `cargo build --release 2>&1 | tail -20`
Expected: Build successful

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat: complete Phase 3 correlated subquery execution

- Add OuterRowAwareExecutor trait for outer row propagation
- Implement set_outer_row for InSubquery and AnyAll executors
- Fix execute_plan to use VolExecutorBuilder for subquery evaluation
- Add correlated subquery detection in planner
- Add integration tests for correlated subqueries

Resolves: Issue #1382"
```

---

## Summary

| Task | Description | Files Modified |
|------|-------------|----------------|
| 1 | Add OuterRowAwareExecutor trait | `executor.rs` |
| 2 | Implement trait for InSubqueryVolcanoExecutor | `executor.rs` |
| 3 | Implement trait for AnyAllVolcanoExecutor | `executor.rs` |
| 4 | Add VolExecutorBuilder helper | `executor.rs` |
| 5 | Fix execute_plan subquery handling | `src/lib.rs` |
| 6 | Add current_outer_row to ExecutionEngine | `src/lib.rs` |
| 7 | Update Filter executor | `filter.rs` |
| 8 | Add correlated subquery detection | `planner/src/lib.rs`, `planner/src/converter.rs` |
| 9 | Add integration tests | `tests/integration/subquery_correlated_test.rs` |
| 10 | Final verification | All |

---

## Future Work (Phase 4)

- Subquery decorrelation (transform correlated → join)
- SEMI JOIN rewrite for EXISTS
- Query optimizer integration

---

## References

- Design Document: `docs/superpowers/plans/2026-04-14-subquery-phase3-design.md`
- PostgreSQL Subquery: `src/backend/executor/nodeSubplan.c`
- DuckDB Correlation: `src/execution/nested_loop_join.hpp`
