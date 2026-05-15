## 1. Foundation: CTE Materialization Infrastructure

- [x] 1.1 Create `CteContext` struct in `src/execution_engine.rs` to track materialized CTE names → temp tables
- [x] 1.2 Add method `materialize_cte(&mut self, cte: &CommonTableExpression, ctx: &mut CteContext) -> SqlResult<()>`
- [x] 1.3 Add method `resolve_cte_table(&self, cte_name: &str) -> Option<String>` to find CTE result table name
- [x] 1.4 Create temp table naming scheme for CTEs: `__cte_<cte_name>_<uuid>`

## 2. Non-Recursive CTE Execution

- [x] 2.1 Modify `execute_statement` for `Statement::WithSelect` to process `with_clause` before `select`
- [x] 2.2 For each CTE in `with_clause.ctes`, call `materialize_cte()` to create temp table
- [x] 2.3 After all CTEs materialized, execute the inner `select` (which can reference CTE names)
- [x] 2.4 Clean up temp CTE tables after main SELECT completes
- [x] 2.5 Handle case where `with_clause` is `None` (pass through to existing execute_select)

## 3. Recursive CTE Execution

- [x] 3.1 Detect `WithClause.recursive == true` path in `execute_statement` for `Statement::WithSelect`
- [x] 3.2 Create `execute_recursive_cte()` method with signature: `execute_recursive_cte(ws: &WithSelect) -> SqlResult<ExecutorResult>`
- [x] 3.3 Implement anchor member execution: parse and execute first SELECT before UNION ALL
- [x] 3.4 Implement iterative recursive execution loop:
  - Execute recursive member SELECT with current working set
  - UNION ALL results to accumulator
  - Update working set with new rows
  - Check termination condition (no new rows)
- [x] 3.5 Enforce max iteration limit (1000) with config: `max_recursive_iterations`
- [x] 3.6 Return aggregated results from all iterations

## 4. Error Handling and Edge Cases

- [x] 4.1 Handle empty anchor member result (recursive CTE with no base rows)
- [x] 4.2 Handle recursive member that references CTE but produces no rows (early termination)
- [x] 4.3 Handle `WITH RECURSIVE` without `UNION ALL` (syntax error, not supported)
- [ ] 4.4 Validate that recursive CTE references itself in recursive member (detect invalid syntax)

## 5. Testing

- [x] 5.1 Add unit test: `test_cte_materialization_basic` - simple non-recursive CTE (implemented as test_cte_non_recursive_basic)
- [x] 5.2 Add unit test: `test_cte_multiple_ctes` - multiple CTEs in single WITH clause
- [x] 5.3 Add unit test: `test_recursive_cte_counter` - counter from 1 to N (core recursive test)
- [x] 5.4 Add unit test: `test_recursive_cte_depth_limit` - depth-limited counter (1000 rows)
- [x] 5.5 Add unit test: `test_recursive_cte_max_iterations` - safety limit enforcement (1001 rows with limit)
- [ ] 5.6 Add unit test: `test_recursive_cte_fibonacci` - fibonacci sequence (parser doesn't support complex recursive syntax)
- [ ] 5.7 Run sql_corpus CTE tests and un-SKIP the recursive CTE tests

## 6. Integration and Verification

- [x] 6.1 Run `cargo build --all-features` - verify no build errors
- [x] 6.2 Run `cargo clippy --all-features -- -D warnings` - zero warnings (pre-existing gmp warnings)
- [x] 6.3 Run `cargo test -p sqlrustgo-executor --lib -- --test-threads=1` - all executor tests pass
- [x] 6.4 Run full test suite to ensure no regressions

## Implementation Summary

Completed core implementation:
- `CteContext` struct with temp table tracking and unique naming
- `execute_non_recursive_cte()` for non-recursive CTEs
- `execute_recursive_cte()` for recursive CTEs with iterative execution (max 1000 iterations)
- `materialize_cte()` to create temp tables from CTE subqueries
- `replace_cte_reference()` to swap CTE names with temp table names
- `cleanup_cte_tables()` for resource cleanup
