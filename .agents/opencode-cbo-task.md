# Opencode Task: CBO Cost Model Integration

## Priority: P0 | Estimate: 5-7 days | Branch: `feat/opencode-v3-cbo`

## Objective

Integrate the existing `SimpleCostModel` into the planner and optimizer to enable cost-based index selection, join reordering, and physical plan optimization.

**Target metrics:**
- Point SELECT QPS: **7,312 → ≥20,000**
- TPC-H Q1 execution time: **reduce ≥50%**
- `EXPLAIN SELECT * FROM t WHERE id = 1` chooses **index scan over full table scan**

## Current State (base to build on)

```
crates/optimizer/src/cost.rs  — SimpleCostModel (5 cost functions, CboOptimizer wrapper)
crates/planner/src/planner.rs — DefaultPlanner (holds cost_model, select_scan compares seq vs index)
src/execution_engine.rs       — execute_statement dispatches PhysicalPlan
```

### Already implemented:
- `SimpleCostModel` with `seq_scan_cost`, `index_scan_cost`, `join_cost`, `agg_cost`, `sort_cost`
- `CboOptimizer` wrapper with defaults
- `CostModel` trait (but `estimate_cost` is a stub returning 100.0)
- `DefaultPlanner.select_scan` already compares seq vs index cost
- `DefaultPlanner.with_storage()` passes storage for statistics
- `StorageEngine` trait has `get_row_count`, `get_page_count`, `get_index_page_count`, `list_indexes`

### What's missing (your work):

## Task Breakdown

### Phase 1: CostModel Trait Rework (Day 1-2)

**Problem**: `CostModel::estimate_cost(&self, _plan: &dyn Any) -> f64` uses type-erased `Any` and always returns 100.0.

**Requirements:**
- Define a `CostNode` enum or trait that PhysicalPlan nodes implement
- Replace `&dyn Any` with `&CostNode` so each PhysicalPlan node can report its cost
- Implement `CostNode` for: `SeqScanExec`, `IndexScanExec`, `FilterExec`, `HashJoinExec`, `AggregateExec`, `SortExec`, `LimitExec`, `ProjectionExec`
- Each node should calculate cost using the appropriate SimpleCostModel method and its own statistics

**Files:**
- `crates/optimizer/src/cost.rs` — CostNode trait/enum
- `crates/optimizer/src/lib.rs` — re-exports

### Phase 2: Multi-Index Selection (Day 2-3)

**Problem**: `select_scan` picks `indexes[0]` regardless of which index best matches the query predicates.

**Requirements:**
- Parse WHERE predicates to identify which index columns are referenced
- Estimate selectivity for each index (how many rows it filters)
- Compute cost for each candidate index using `index_scan_cost` × selectivity
- Pick the **lowest cost** index, not the first one
- Fall back to seq scan only if all indexes cost more

**Files:**
- `crates/planner/src/planner.rs` — `select_scan` method
- `crates/planner/src/physical_plan.rs` — IndexScanExec may need selectivity field

### Phase 3: Join Ordering with Cost (Day 3-4)

**Problem**: `select_join` computes costs for hash/nested_loop/sort_merge but doesn't reorder join inputs.

**Requirements:**
- For multi-table joins, estimate left-deep vs right-deep join order cost
- Compare hash_join vs nested_loop vs sort_merge and pick cheapest
- Consider estimated row counts after predicate pushdown (using selectivity from Phase 2)
- Simple heuristic for star joins: fact table first, dimension tables by selectivity

**Files:**
- `crates/planner/src/planner.rs` — `select_join` or new `optimize_join_order` method

### Phase 4: EXPLAIN Cost Output (Day 4-5)

**Requirements:**
- `EXPLAIN` output shows estimated cost per plan node
- Shows why a particular index was chosen (cost comparison)
- Shows join method selection reason
- Human-readable cost breakdown

**Files:**
- `crates/planner/src/physical_plan.rs` — add `cost()` method to PhysicalPlan
- `src/execution_engine.rs` — EXPLAIN formatting

### Phase 5: Statistics Collection & Caching (Day 5-6)

**Requirements:**
- Column-level statistics: min/max, distinct count, NULL count
- Histogram for range predicate selectivity estimation
- Cache statistics with TTL (avoid re-computing on every query)
- ANALYZE TABLE command to trigger statistics refresh

**Files:**
- `crates/storage/src/` — statistics module
- `crates/planner/src/` — statistics integration

### Phase 6: Verification & Tuning (Day 6-7)

**Requirements:**
```bash
# 1. Cost-based index selection test
cargo test -p sqlrustgo-planner --test cbo_index_selection

# 2. TPC-H Q1 performance
cargo bench --bench tpch_q1 -- features/cbo

# 3. All regression tests pass
cargo test -p sqlrustgo-planner -p sqlrustgo-optimizer --all-features

# 4. Clippy + fmt
cargo clippy --all-features -- -D warnings
cargo fmt --check --all
```

**Acceptance criteria:**
- `EXPLAIN SELECT * FROM t WHERE id = 1` shows IndexScanExec with cost < SeqScanExec
- TPC-H Q1 execution time reduced by ≥50%
- Point SELECT QPS in benchmark ≥18,000 (interim) or ≥20,000 (final)
- All optimizer tests pass (existing 86 + new CBO tests)
- Zero clippy warnings, zero fmt violations

## Working Context

```bash
cd ~/workspace/dev/openheart/sqlrustgo
git checkout develop/v3.0.0
git pull origin develop/v3.0.0
git checkout -b feat/opencode-v3-cbo
```

**Key files:**
- `crates/optimizer/src/cost.rs` (339 lines) — CostModel + SimpleCostModel + CboOptimizer
- `crates/optimizer/src/lib.rs` (180 lines) — re-exports, CostModel trait
- `crates/planner/src/planner.rs` (968 lines) — DefaultPlanner with select_scan / select_join
- `crates/planner/src/physical_plan.rs` — all PhysicalPlan node types
- `crates/planner/src/lib.rs` (683 lines) — planner interface
- `src/execution_engine.rs` — executes PhysicalPlan, formats EXPLAIN

**Run commands:**
```bash
cargo check -p sqlrustgo-planner -p sqlrustgo-optimizer
cargo test -p sqlrustgo-planner -p sqlrustgo-optimizer --all-features
cargo clippy -p sqlrustgo-planner -p sqlrustgo-optimizer --all-features -- -D warnings
```

## Branch naming

```
feat/opencode-v3-cbo
```

PR target: `develop/v3.0.0`

## Related Issues

- Closes P0: CBO 代价模型集成
- Part of #353: v3.0.0 开发总控
