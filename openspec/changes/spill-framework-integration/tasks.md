## 1. Integrate AdaptiveMemoryTracker into SessionConfig

- [x] 1.1 Add `memory_limit: usize` and `spill_threshold: f64` to SessionConfig
- [x] 1.2 Pass memory tracker to executor via ExecutionEngine::new()

## 2. Integrate PartitionManager into local_executor

- [x] 2.1 Add `Arc<PartitionManager>` to LocalExecutor
- [x] 2.2 Create PartitionManager during executor initialization
- [x] 2.3 Cleanup partitions on executor drop

## 3. Implement SpillingHashJoin operator

- [x] 3.1 Wrap hash table building with memory tracking
- [x] 3.2 When memory exceeded, partition and spill right side to disk
- [x] 3.3 Read partitions back during probe phase
- [x] 3.4 Implement merge of results from spilled partitions

## 4. Implement SpillingAggregate operator

- [x] 4.1 Wrap HashMap aggregation with memory tracking
- [x] 4.2 When memory exceeded, spill groups to disk
- [x] 4.3 Read and merge spilled groups during output phase

## 5. Add TPC-H Q4, Q13, Q17, Q18, Q20, Q21, Q22 tests

- [x] 5.1 Verify TPC-H Q17 (complex aggregate expression) parsing - `SUM(l_extendedprice * (1 - l_discount))`
- [x] 5.2 Fix LEFT OUTER / RIGHT OUTER JOIN parsing
- [ ] 5.3 Verify SF=0.1 queries complete within timeout
- [ ] 5.4 Verify SF=10 queries complete without OOM

## 6. Performance validation

- [ ] 6.1 Measure Q4/Q13/Q17/Q18/Q20/Q21/Q22 latency at SF=10
- [ ] 6.2 Verify no regression on simple queries (Q1, Q6)

## Completed (PR #1064)

- Parser fixes: `parse_aggregate_function` uses `parse_expression()` for complex args
- Parser fixes: `LEFT OUTER` / `RIGHT OUTER` JOIN parsing
- Executor: `GroupAccumulator` with `Serialize/Deserialize` derives
- Executor: `hash_inner_join_with_spill` using `HashJoinSpillOperator`
- Executor: Aggregate memory tracking and spill-to-disk

## Testing

- Parser tests: 29 passed, 0 failed
- Executor tests: all passed
- Clippy: zero warnings
- TPC-H Q17 (complex aggregate): parses successfully
