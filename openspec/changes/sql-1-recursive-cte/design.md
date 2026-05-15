## Context

SQLRustGo's parser (`parser.rs`) correctly handles `WITH RECURSIVE` syntax, producing a `WithSelect` with `WithClause { recursive: true, ctes: [...] }`. The planner creates `LogicalPlan::With` for CTEs.

**Current Problem**: The executor at `src/execution_engine.rs:723-725` handles `Statement::WithSelect(ref ws)` by directly calling `self.execute_select(&ws.select)`, completely ignoring `ws.with_clause`. This means:
1. Non-recursive CTE definitions are never materialized
2. Recursive CTEs fail at execution time since the CTE name is undefined

**Affected Files**:
- `src/execution_engine.rs` - Main execution engine
- `crates/parser/src/parser.rs` - Already handles parsing correctly
- `crates/planner/src/planner.rs` - CTE planning exists

## Goals / Non-Goals

**Goals:**
- Implement recursive CTE execution with proper iterative evaluation
- Support anchor member → recursive member cycle until termination
- Detect termination condition (no new rows) and cycle condition (max iterations)
- Maintain backward compatibility with non-recursive CTEs

**Non-Goals:**
- Support mutually recursive CTEs (CTEs referencing each other)
- Support nested recursive CTEs
- Performance optimization beyond basic iteration control

## Decisions

### Decision 1: Execution Model - Push-Based Iteration

**Option A: Push-Based (chosen)**
- Execute anchor member, collect rows
- For each iteration, execute recursive member with current rows as input
- UNION ALL results, repeat until no new rows
- Simple, easy to debug, sufficient for Beta Gate

**Option B: Pull-Based / Demand**
- Recursive member executes "on demand" as rows are consumed
- More complex, harder to implement correctly

**Decision**: Use Push-Based iteration for simplicity and correctness.

### Decision 2: CTE Materialization Scope

**Option A: Per-CTE materialization (chosen)**
- Each CTE definition is materialized into a temporary table/view
- Recursive CTE references these temporary tables
- Allows multiple CTEs in single WITH clause

**Option B: Single CTE at a time**
- Only support single recursive CTE, no multiple CTEs
- Limits functionality but simpler

**Decision**: Support multiple CTEs with per-CTE materialization.

### Decision 3: Cycle Detection

**Option A: Max iteration limit (chosen)**
- Set `max_recursive_iterations = 1000` (configurable)
- Prevents infinite loops
- Simple and predictable

**Option B: Explicit cycle detection via row fingerprinting**
- Track visited rows to detect actual cycles
- More accurate but complex

**Decision**: Use max iteration limit for Beta Gate.

### Decision 4: Where to Implement

**Option A: New `cteExecutor.rs` module (chosen)**
- Clean separation of concerns
- Easy to test in isolation
- Follows existing pattern of `filter.rs`, `join.rs`, etc.

**Option B: Inline in existing execution_engine.rs**
- Faster to implement
- Violates single responsibility

**Decision**: Create new module at `src/cte_executor.rs`.

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| Infinite loop in recursive CTE | High | Max iteration limit (1000 default) |
| Memory blow-up with large recursion | Medium | Stream rows instead of collecting all, limit clause |
| Non-recursive CTEs broken | High | Test both non-recursive and recursive CTEs |
| Performance with many iterations | Low | Allow early termination detection |

## Implementation Plan

### Phase 1: Fix Non-Recursive CTE (Foundation)
1. Modify `execute_statement` for `Statement::WithSelect` to:
   - First materialize each CTE definition into a temp table
   - Then execute the inner SELECT (which can reference those temp tables)
2. Create `CteContext` to track materialized CTEs during execution

### Phase 2: Implement Recursive CTE
1. Detect `WithClause.recursive == true`
2. For recursive CTE:
   - Execute anchor member (first SELECT before UNION ALL)
   - Iteratively execute recursive member (SELECT after UNION ALL that references CTE)
   - Each iteration: execute recursive member with current working set
   - UNION ALL results from all iterations
   - Stop when recursive member returns no rows or max iterations reached

### Phase 3: Testing
1. Add unit tests for non-recursive CTE
2. Add unit tests for recursive CTE (factorial, fibonacci, depth examples)
3. Add integration tests via sql_corpus

## Open Questions

1. **Should we support `RECURSIVE` keyword without `WITH`?** - Standard SQL requires `WITH RECURSIVE`, not just `RECURSIVE`. Parser handles this correctly.
2. **Should we support `SEARCH` or `CYCLE` clauses?** - Advanced SQL standard features, not needed for Beta Gate.
3. **How to handle column name mismatch between anchor and recursive member?** - Use anchor member column names as authoritative.
