## Why

SQLRustGo's parser and planner support Common Table Expressions (CTE), including recursive syntax via `WITH RECURSIVE`. However, the executor does not implement recursive CTE execution, causing queries with recursive CTEs to fail at runtime. This blocks Beta Gate SQL-1 requirement and prevents users from querying hierarchical data structures (org charts, bill of materials, graph traversals).

## What Changes

- **New**: Recursive CTE execution engine implementation in `sqlrustgo-executor`
- **New**: `LogicalPlan::RecursiveCte` variant to represent recursive CTE plans
- **New**: Execution engine support for iterative CTE evaluation with termination detection
- **Modified**: Existing CTE execution path to detect and handle recursive vs non-recursive CTEs
- **New**: Unit tests for recursive CTE execution covering anchor members, recursive members, and termination conditions

## Capabilities

### New Capabilities

- `recursive-cte-execution`: Iterative execution of recursive CTEs with anchor member evaluation, recursive member evaluation with cycle detection, and termination when no new rows are produced

### Modified Capabilities

- `cte-execution`: Existing CTE execution path will be modified to distinguish between non-recursive (`WITH`/`WITH ... UNION ALL`) and recursive (`WITH RECURSIVE`) CTEs

## Impact

- **Executor crate**: New module `recursive_cte.rs` or extension to existing CTE execution
- **Planner crate**: May need `LogicalPlan::RecursiveCte` variant if current representation insufficient
- **Parser crate**: No changes (parsing already exists)
- **Tests**: New tests in `sqlrustgo-executor` for recursive CTE scenarios
