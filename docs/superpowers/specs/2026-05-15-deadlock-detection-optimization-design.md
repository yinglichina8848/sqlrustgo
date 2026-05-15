# Deadlock Detection Performance Optimization Design

**Date**: 2026-05-15
**Issue**: #989 (PERF-4)
**Target**: Deadlock detection latency < 50ms (current: ~100ms)
**Strategy**: Hybrid (BFS + Incremental Detection)

---

## 1. Overview

Optimize deadlock detection in `crates/transaction/src/deadlock.rs` to achieve < 50ms latency. Current implementation uses recursive DFS which has high function call overhead and scans the entire graph on each detection.

## 2. Current Implementation Analysis

### 2.1 Performance Bottlenecks

| Location | Issue | Impact |
|----------|-------|--------|
| `would_create_cycle` | O(V+E) × \|holders\| per call | High |
| `dfs_reachable` | Recursive DFS with function call overhead | High |
| `detect_cycle` | Full graph DFS, no incremental check | Medium |

### 2.2 Data Structures

- `waits_for: HashMap<TxId, HashSet<TxId>>` - Adjacency list
- Recursive traversal via `dfs_reachable()`

## 3. Optimization Strategy

### Phase 1: BFS Replacement (Quick Win)

Replace recursive DFS with iterative BFS using `VecDeque`:

```rust
fn bfs_reachable(&self, start: TxId, target: TxId) -> bool {
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();

    queue.push_back(start);

    while let Some(current) = queue.pop_front() {
        if current == target {
            return true;
        }
        if visited.insert(current) {
            if let Some(holders) = self.waits_for.get(&current) {
                queue.extend(holders);
            }
        }
    }
    false
}
```

**Expected gain**: ~20-30% reduction in latency

### Phase 2: Incremental Detection

Add a cache for incremental detection:

```rust
struct IncrementalCache {
    last_checked_tx: TxId,
    affected_paths: Vec<TxId>,
    version: u64,
}

impl DeadlockDetector {
    pub fn detect_cycle_incremental(&self, tx_id: TxId) -> Option<Vec<TxId>> {
        // Only traverse paths relevant to tx_id, not full graph
        let inner = self.inner.lock().unwrap();
        self.bfs_cycle(&inner.waits_for, tx_id, &mut HashSet::new(), &mut Vec::new())
    }
}
```

**Expected gain**: Additional ~30-40% reduction

## 4. Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    DeadlockDetector                         │
├─────────────────────────────────────────────────────────────┤
│  inner: Mutex<Inner>                                        │
│    - waits_for: HashMap<TxId, HashSet<TxId>>                │
│    - incremental_cache: Option<IncrementalCache>            │
├─────────────────────────────────────────────────────────────┤
│  + try_wait_edge(blocked, holders) -> Result<(), LockError>│
│    [BFS-based would_create_cycle check]                     │
│  + detect_cycle(start) -> Option<Vec<TxId>>                │
│    [Full graph BFS traversal]                               │
│  + detect_cycle_incremental(tx_id) -> Option<Vec<TxId>>    │
│    [Optimized: only relevant paths]                         │
└─────────────────────────────────────────────────────────────┘
```

## 5. API Compatibility

| Method | Changes | Backward Compatible |
|--------|---------|---------------------|
| `try_wait_edge` | Internal BFS optimization | Yes |
| `detect_cycle` | Internal BFS optimization | Yes |
| `detect_cycle_incremental` | New method | Yes |

## 6. Performance Targets

| Stage | Implementation | Expected Latency |
|-------|----------------|------------------|
| Baseline | Current recursive DFS | ~100ms |
| Phase 1 | BFS replacement | ~70ms |
| Phase 2 | Incremental detection | ~40ms |

## 7. Testing Strategy

### 7.1 Unit Tests
- All existing tests in `deadlock.rs` must pass
- New tests for BFS vs DFS equivalence

### 7.2 Performance Tests
- Add benchmark: `deadlock_bench.rs`
- Verify < 50ms per detection

### 7.3 Regression Tests
- Verify cycle detection results are identical before/after optimization

## 8. Implementation Order

1. **Phase 1a**: Add `bfs_reachable` method
2. **Phase 1b**: Replace `dfs_reachable` calls with `bfs_reachable` in `would_create_cycle`
3. **Phase 1c**: Replace recursive DFS in `detect_cycle` with BFS
4. **Phase 2**: Add `IncrementalCache` and `detect_cycle_incremental`
5. **Phase 3**: Add benchmarks and verify < 50ms target

## 9. Files to Modify

- `crates/transaction/src/deadlock.rs` - Core optimization
- `crates/transaction/benches/deadlock_bench.rs` - New benchmark

## 10. Verification

```bash
# Run existing tests
cargo test -p sqlrustgo-transaction --all-features

# Run new benchmarks
cargo bench -p sqlrustgo-transaction -- deadlock

# Verify latency
# Target: < 50ms per detect_cycle call with 100 concurrent transactions
```
