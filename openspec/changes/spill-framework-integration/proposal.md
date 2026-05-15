## Why

TPC-H SF=10 queries (Q4, Q13, Q15, Q17, Q18, Q20, Q21, Q22) timeout or OOM because the execution engine loads entire tables into memory. The existing spill framework in `crates/spill/` is not integrated into the executor, leaving large hash joins and aggregates without memory management.

## What Changes

- **Integrate spill framework into execution engine**: Connect `AdaptiveMemoryTracker`, `PartitionManager`, and spill operators (Sort, HashJoin, Aggregate) to the executor's memory management
- **Add memory-aware hash join**: When building hash tables exceeds memory threshold, partition and spill to disk
- **Add memory-aware aggregation**: When group state exceeds memory, spill groups to disk
- **Implement streaming merge**: Read spilled partitions back and merge results
- **Add configuration**: Session-level `memory_limit` and `spill_threshold` settings

## Capabilities

### New Capabilities

- `spill-to-disk`: Core spill-to-disk infrastructure for memory-bounded operators
- `streaming-hash-join`: Hash join that spills partitions when memory exceeded
- `streaming-aggregate`: Group-by aggregate that spills groups when memory exceeded
- `adaptive-memory`: Adaptive memory tracking with soft/hard limits

### Modified Capabilities

- (none - this is a new performance optimization)

## Impact

- **Affected crates**: `executor`, `planner`, `storage`
- **New dependency**: `spill` crate integration
- **Config changes**: `SessionConfig` adds `memory_limit`, `spill_dir`
- **No API changes**: Backward compatible optimization
