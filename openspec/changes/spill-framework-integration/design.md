## Context

TPC-H SF=10 requires processing 600K lineitem rows. Current executor loads all data into `MemoryStorage` then processes with in-memory operators. Hash joins build entire hash tables in memory, causing:
- Q4: EXISTS subquery with correlated join
- Q13: LEFT JOIN with subquery aggregation
- Q15: Multiple nested subqueries with large GROUP BY
- Q17: Correlated subquery with AVG aggregation
- Q18: IN subquery with HAVING and multi-table join
- Q20: Nested IN subqueries with large intermediate results
- Q21: EXISTS/NOT EXISTS with 4-table join
- Q22: NOT EXISTS with correlated subquery

The `crates/spill/` module provides:
- `AdaptiveMemoryTracker`: CAS-based memory accounting
- `PartitionManager`: File-based partition storage
- `SortSpillOperator`, `HashJoinSpillOperator`, `AggregateSpillOperator`: Spill-capable operators

**Problem**: These operators exist but are NOT integrated into `local_executor.rs`.

## Goals / Non-Goals

**Goals:**
- Integrate spill framework into executor for memory-bounded hash join and aggregate
- Enable SF=10 TPC-H 22/22 queries to complete without OOM
- Soft memory limits with graceful degradation

**Non-Goals:**
- Not implementing new query operators (reuse existing)
- Not changing SQL semantics or planner output
- Not implementing distributed spilling

## Decisions

### Decision 1: Graceful Degradation Over Hard Limits

**Choice**: Use soft memory limits with adaptive spilling, not hard OOM prevention.

**Rationale**: Hard limits require precise memory accounting which is difficult in Rust. Soft limits with spill-on-threshold allows operators to continue with degraded performance.

**Alternative**: Hard memory limits via jemalloc/dlmalloc. Rejected - too invasive.

### Decision 2: Partition-Based Spilling

**Choice**: Partition-based spilling (graceful hash partitioning) over sort-based spilling.

**Rationale**: Hash join naturally partitions data by key. Partitions can be processed independently, enabling parallel spill readers.

**Alternative**: Sort-based spilling (external sort). Rejected - sort is O(n log n) vs partition's O(n).

### Decision 3: Binary Serialization for Spill Files

**Choice**: Use `bincode` for partition serialization.

**Rationale**: Simple, fast, works with existing serde derive on types.

**Alternative**: CSV/JSON. Rejected - larger files, slower parsing.

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| Spill I/O dominates query time | Enable only when memory pressure detected |
| Partition manager temp files accumulate | Cleanup on error/drop |
| Serialization overhead | Use bincode (fastest Rust serializer) |

## Open Questions

1. **Spill directory location**: Use system temp or configurable `spill_dir` in SessionConfig?
2. **Spill threshold**: Default 70% of memory_limit? Make configurable?
3. **Concurrency**: Multiple threads spilling simultaneously - need coordination?
