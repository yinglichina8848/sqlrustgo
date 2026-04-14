# Subquery Execution Phase 3 Design

## Execution Engine Semantics Upgrade Specification

---

## Background

### Current State (After Phase 1 & Phase 2)

**Supported Capabilities:**

| Capability | Status |
|------------|--------|
| EXISTS parser | ✅ |
| IN parser | ✅ |
| ANY / ALL parser | ✅ |
| Planner Exists/AnyAll conversion | ✅ |
| VolcanoExecutor subquery executors | ✅ |
| Filter executor integration | ✅ |
| execute_plan integration framework | ✅ |
| TPC-H subquery test suite | ✅ |

**Known Limitations:**

| Limitation | Impact |
|------------|--------|
| Correlated subquery not supported | Cannot execute queries with outer row references |
| Outer-row context not propagated | InSubquery/AnyAll return placeholder `false` |
| execute_plan returns placeholder results | Subquery predicates evaluated incorrectly |
| Non-correlated subquery repeatedly executed | Performance degradation |

---

## Problem Statement

### The Core Issue

The Volcano execution model uses a `next()` method that does **not** accept external row context:

```rust
pub trait VolcanoExecutor {
    fn next(&mut self) -> SqlResult<Option<Vec<Value>>>;
}
```

However, correlated subqueries **require** outer-row propagation:

```sql
SELECT *
FROM orders o
WHERE o.custkey IN (
    SELECT c.custkey
    FROM customer c
    WHERE c.nationkey = o.nationkey  -- o.nationkey is outer reference
)
```

Here, `o.nationkey` must be passed as `outer_row` to the subquery executor.

### Current System Behavior

| Subquery Type | Current Behavior | Expected Behavior |
|---------------|-----------------|-------------------|
| `EXISTS` | Executes but ignores correlation | Should pass outer_row when correlated |
| `IN` | Returns placeholder `false` | Should use outer_row for evaluation |
| `ANY/ALL` | Returns placeholder `false` | Should use outer_row for evaluation |

---

## Design Goals

Phase 3 introduces:

1. **Outer row propagation** - Unified mechanism for passing external row context
2. **Correlated subquery detection** - Planner capability to identify outer references
3. **Executor integration** - Filter executor orchestrates subquery evaluation
4. **Subquery caching** - Avoid repeated execution for non-correlated subqueries

### Target Capabilities (After Phase 3)

| Capability | Status |
|------------|--------|
| Correlated EXISTS | ✅ |
| Correlated IN | ✅ |
| Correlated ANY | ✅ |
| Correlated ALL | ✅ |

**TPC-H Queries Enabled:**

- Q17 - Large Volume Order Tracking
- Q21 - Suppliers Who Kept Orders Waiting
- Q22 - Global Sales Opportunity

---

## Design Architecture

### Execution Strategy Selection

```
FilterExecutor
    │
    ├── has_subquery?
    │       │
    │       └── yes → is_correlated?
    │                   │
    │                   ├── yes → Per-row VolcanoExecutor + set_outer_row()
    │                   │
    │                   └── no  → Execute once + cache result
    │
    └── no → Standard filter evaluation
```

---

## Section 1 — OuterRowAwareExecutor Trait

### Motivation

Currently, `InSubqueryVolcanoExecutor` and `AnyAllVolcanoExecutor` have ad-hoc `set_outer_row()` methods:

```rust
// Current (ad-hoc)
impl InSubqueryVolcanoExecutor {
    pub fn set_outer_row(&mut self, row: Option<Vec<Value>>) { ... }
}
```

We introduce a **unified trait** for consistency:

```rust
/// Trait for executors that require external row context
/// Used for correlated subquery evaluation
pub trait OuterRowAwareExecutor: Send {
    /// Set the outer row context for correlated subquery evaluation
    fn set_outer_row(&mut self, row: Option<&[Value]>);

    /// Check if outer row has been set
    fn has_outer_row(&self) -> bool;
}
```

### Applies To

| Executor | Requires Outer Row | Reason |
|----------|-------------------|--------|
| `InSubqueryVolcanoExecutor` | Yes | Must evaluate `expr IN (subquery)` with outer value |
| `AnyAllVolcanoExecutor` | Yes | Must evaluate `expr OP (subquery)` with outer value |
| `ExistsVolcanoExecutor` | No | Only checks if subquery returns any rows |
| `FilterVolcanoExecutor` | Yes (proxy) | Passes outer row to subquery predicates |

### Implementation

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

---

## Section 2 — Correlated Subquery Detection

### Planner Addition

```rust
/// Check if a subquery contains references to outer scope
pub fn is_correlated_subquery(subquery: &SubqueryPlan) -> bool {
    contains_outer_reference(subquery)
}

/// Check if expression contains references to outer relations
fn contains_outer_reference(expr: &Expr) -> bool {
    match expr {
        Expr::Column(col) => col.relation.is_some(),  // Has outer reference
        Expr::BinaryExpr { left, right, .. } => {
            contains_outer_reference(left) || contains_outer_reference(right)
        }
        // ... other expr types
        _ => false
    }
}
```

### Detection Logic

A subquery is **correlated** if:

1. It references columns from an outer query scope
2. The column's relation is **not** within the subquery's scope

### Example

```sql
-- Correlated (c.nationkey = o.nationkey)
SELECT * FROM orders o
WHERE o.custkey IN (
    SELECT c.custkey FROM customer c
    WHERE c.nationkey = o.nationkey  -- OUTER reference
)

-- Non-correlated (no outer references)
SELECT * FROM orders o
WHERE o.custkey IN (
    SELECT c.custkey FROM customer c  -- No outer refs
)
```

---

## Section 3 — Execution Strategy Selection

### Decision Logic

```rust
pub fn select_subquery_execution_strategy(
    subquery: &SubqueryPlan,
    outer_row: Option<&[Value]>
) -> SubqueryExecutionStrategy {
    if is_correlated_subquery(subquery) {
        // Per-row execution with outer context
        SubqueryExecutionStrategy::Correlated {
            requires_outer_row: true,
            cache_results: false,
        }
    } else {
        // Execute once, cache result
        SubqueryExecutionStrategy::NonCorrelated {
            cache_key: compute_cache_key(subquery),
        }
    }
}
```

### Strategy Comparison

| Strategy | Use Case | Performance |
|----------|----------|--------------|
| `Correlated` | Outer references present | O(N × subquery_rows) |
| `NonCorrelated` | No outer references | O(subquery_rows) + O(N) lookup |

---

## Section 4 — Filter Executor Integration

### Current Behavior (Bug)

In `execute_plan`, `InSubquery` returns placeholder:

```rust
"InSubquery" => {
    let _subquery_result = self.execute_plan(children[0])?;
    Ok(ExecutorResult::new(vec![vec![Value::Boolean(false)]], 0))
    // ❌ Always returns false, ignores outer_row
}
```

### Required Change

Filter executor must:

1. Detect subquery predicates during evaluation
2. For correlated subqueries:
   - Build `VolcanoExecutor` via `VolExecutorBuilder`
   - Set outer row via `set_outer_row()`
   - Evaluate subquery for each outer row
3. For non-correlated subqueries:
   - Execute subquery once
   - Cache result for subsequent lookups

### New Filter Executor Call Chain

```
FilterExecutor::next()
    │
    ├── evaluate predicate
    │
    ├── predicate.contains_subquery()?
    │       │
    │       └── yes
    │           │
    │           ├── is_correlated(predicate)?
    │           │       │
    │           │       └── yes
    │           │           │
    │           │           ├── vol_executor.set_outer_row(outer_row)
    │           │           ├── vol_executor.next()
    │           │           └── return result
    │           │
    │           └── no
    │               │
    │               ├── check cache
    │               │       │
    │               │       └── hit → return cached
    │               │
    │               ├── execute once
    │               ├── cache result
    │               └── return result
    │
    └── return standard filter result
```

---

## Section 5 — Non-Correlated Subquery Cache

### Motivation

For non-correlated subqueries, executing per-row is wasteful:

```sql
SELECT * FROM orders
WHERE custkey IN (
    SELECT custkey FROM customer  -- Constant, execute once
)
```

**Before:** Executes subquery for each order row (O(N × M))
**After:** Executes subquery once (O(M) + O(N) lookup)

### Cache Structure

```rust
pub struct SubqueryCache {
    cache: HashMap<CacheKey, CachedResult>,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct CacheKey {
    subquery_id: u64,
    // Could include parameter binding for prepared statements
}

pub struct CachedResult {
    values: Vec<Value>,
    cached_at: Instant,
}
```

### Cache Invalidation

- **Statement-level**: Cache cleared after statement execution
- **Transaction-level** (future): Invalidated on table modifications

---

## Section 6 — execute_plan Integration Fix

### Current Implementation

```rust
"InSubquery" => {
    let _subquery_result = self.execute_plan(children[0])?;
    Ok(ExecutorResult::new(vec![vec![Value::Boolean(false)]], 0))
}
```

### Fixed Implementation

```rust
"InSubquery" => {
    let in_subquery_plan = plan
        .as_any()
        .downcast_ref::<sqlrustgo_planner::physical_plan::InSubqueryExec>()
        .ok_or_else(|| {
            SqlError::ExecutionError("Failed to downcast InSubqueryExec".to_string())
        })?;

    let children = in_subquery_plan.children();
    if children.is_empty() {
        return Err(SqlError::ExecutionError(
            "InSubquery has no children".to_string(),
        ));
    }

    // Build subquery executor
    let vol_builder = VolExecutorBuilder::new(self.storage.clone());
    let mut sub_exec = vol_builder.build(children[0])?;

    // For correlated: outer_row should be set by parent Filter
    // For non-correlated: execute once during init
    sub_exec.init()?;

    // If we have outer_row context (from Filter), use it
    if let Some(ref outer_row) = current_outer_row {
        if let Some(exec) = sub_exec.as_any().downcast_mut::<InSubqueryVolcanoExecutor>() {
            exec.set_outer_row(Some(outer_row));
        }
    }

    let result = sub_exec.next()?;
    Ok(ExecutorResult::new(result.map(|r| vec![r]).unwrap_or_default(), 0))
}
```

### Integration with Filter Executor

When `execute_plan` processes a `Filter` node that contains subquery predicates:

1. Build child executor (possibly another `Filter` or `SeqScan`)
2. For each row from child:
   - Set `outer_row` on subquery executors
   - Evaluate predicate
   - Return matching rows

---

## Section 7 — Executor Changes Summary

### Modified Files

| File | Changes |
|------|---------|
| `crates/executor/src/executor.rs` | Add `OuterRowAwareExecutor` trait, implement for `InSubqueryVolcanoExecutor` and `AnyAllVolcanoExecutor` |
| `crates/executor/src/filter.rs` | Add `set_outer_row()` method, integrate with subquery evaluation |
| `src/lib.rs` | Fix `execute_plan` subquery handling to use `VolExecutorBuilder` + `set_outer_row()` |

### New Files

| File | Purpose |
|------|---------|
| `crates/executor/src/outer_row_aware.rs` | `OuterRowAwareExecutor` trait definition |

### Trait Changes

```rust
// New method in VolcanoExecutor trait (optional default impl)
pub trait VolcanoExecutor: Send + Sync {
    // ... existing methods ...

    /// Set outer row context for correlated subquery evaluation
    /// Default implementation returns early
    fn set_outer_row(&mut self, _row: Option<&[Value]>) {}
}
```

---

## Section 8 — Planner Changes Summary

### New Functions

```rust
// In planner module
pub fn is_correlated_subquery(subquery: &SubqueryPlan) -> bool;
pub fn contains_outer_reference(expr: &Expr) -> bool;
pub fn compute_subquery_cache_key(subquery: &SubqueryPlan) -> CacheKey;
```

### Expression Changes

```rust
// New method in Expr
impl Expr {
    pub fn contains_outer_reference(&self) -> bool {
        // ... implementation
    }

    pub fn collect_outer_references(&self) -> Vec<ColumnRef> {
        // ... for more detailed analysis
    }
}
```

---

## Section 9 — Performance Impact

### Complexity Analysis

**Before Phase 3:**

| Query Type | Complexity |
|------------|------------|
| Non-correlated IN/EXISTS/ANY/ALL | O(subquery) once |
| Correlated (broken) | ❌ Not supported |

**After Phase 3:**

| Query Type | Complexity |
|------------|------------|
| Non-correlated IN/EXISTS/ANY/ALL | O(subquery) once + O(1) lookup |
| Correlated IN/EXISTS/ANY/ALL | O(N × subquery_rows) |
| Optimized correlated (future) | O(join) via decorrelation |

### Memory Impact

| Component | Memory |
|-----------|--------|
| Subquery cache | O(subquery_result_size) per cached subquery |
| Outer row buffer | O(1) per executor instance |

### Worst Case

For correlated subqueries with large outer relation:

```sql
SELECT * FROM large_table  -- 1M rows
WHERE col IN (SELECT sub_col FROM sub_table)
```

- Before: Fails (placeholder result)
- After: O(1M × subquery_size)

**Note:** Future Phase 4 (decorrelation) will optimize this to O(join).

---

## Section 10 — TPC-H Support Improvement

### Query Q17 - Large Volume Order Tracking

```sql
SELECT SUM(l.quantity)
FROM lineitem l
WHERE l.partkey IN (
    SELECT p.partkey
    FROM part p
    WHERE p.size = 3
    AND p.type LIKE '%BRASS'
)
AND l.extendedprice > 0.9 * (
    SELECT AVG(l2.extendedprice)
    FROM lineitem l2
    WHERE l2.partkey = l.partkey
);
```

**Status Before:** ❌ Broken (correlated AVG subquery)
**Status After:** ✅ Supported

### Query Q21 - Suppliers Who Kept Orders Waiting

```sql
SELECT s.name
FROM supplier s
WHERE s.suppkey IN (
    SELECT l.suppkey
    FROM lineitem l
    WHERE l.orderkey IN (
        SELECT o.orderkey
        FROM orders o
        WHERE o.orderstatus = 'F'
        AND l.receiptdate > l.commitdate
    )
    GROUP BY l.suppkey
    HAVING COUNT(*) > 1
);
```

**Status Before:** ❌ Broken (multi-level correlation)
**Status After:** ✅ Supported

### Query Q22 - Global Sales Opportunity

```sql
SELECT cntrycode, COUNT(*) AS numcust, SUM(c.acctbal) AS totacctbal
FROM customer c
WHERE c.acctbal > (
    SELECT AVG(c1.acctbal)
    FROM customer c1
    WHERE c1.acctbal > 0
    AND c1.cntrycode = c.cntrycode
)
AND c.cntrycode IN ('20', '40', '22', '30', '39', '42', '21')
GROUP BY cntrycode
ORDER BY cntrycode;
```

**Status Before:** ❌ Broken (correlated with GROUP BY)
**Status After:** ✅ Supported

---

## Future Work

### Phase 4: Subquery Decorrelation

Transform correlated subqueries into joins for better performance:

```sql
-- Correlated (Phase 3)
SELECT * FROM orders o
WHERE o.custkey IN (
    SELECT c.custkey FROM customer c
    WHERE c.nationkey = o.nationkey
)

-- Decorrelated (Phase 4)
SELECT DISTINCT o.*
FROM orders o
JOIN customer c ON o.custkey = c.custkey
WHERE c.nationkey = o.nationkey
```

**Benefits:**

- O(N × M) → O(N + M) complexity
- Standard hash join optimization applies
- Batch execution possible

**Implementation Path:**

1. Detect correlation patterns
2. Lift correlated expressions to join conditions
3. Remove subquery nodes from plan tree
4. Insert deduplication (DISTINCT) if needed

---

## Summary

| Phase | Capability | Status |
|-------|------------|--------|
| Phase 1 | Parser + Planner support | ✅ Complete |
| Phase 2 | VolcanoExecutor framework | ✅ Complete |
| **Phase 3** | **Correlated subquery execution** | **In Progress** |
| Phase 4 | Subquery decorrelation | Planned |

**Phase 3 Deliverables:**

1. `OuterRowAwareExecutor` trait
2. Filter executor outer row integration
3. execute_plan subquery fix
4. Correlated subquery detection
5. Non-correlated subquery cache
6. TPC-H Q17/Q21/Q22 support

---

## References

- PostgreSQL Subquery Evaluation: `src/backend/executor/nodeSubplan.c`
- DuckDB Subquery Correlation: `src/execution/nested_loop_join.hpp`
- SparkSQL Adaptive Query Execution: `sql/catalyst/src/main/scala/org/apache/spark/sql/execution/adaptive.scala`
