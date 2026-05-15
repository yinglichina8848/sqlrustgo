# Deadlock Detection Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Optimize deadlock detection latency from ~100ms to < 50ms by replacing recursive DFS with iterative BFS and adding incremental detection.

**Architecture:** BFS-based graph traversal in `crates/transaction/src/deadlock.rs` replacing recursive DFS. Incremental cache for targeted cycle detection.

**Tech Stack:** Rust, `VecDeque` for BFS, `HashMap`/`HashSet` for adjacency list.

---

## File Structure

- **Modify:** `crates/transaction/src/deadlock.rs` - Core optimization (lines 1-354)
- **Create:** `crates/transaction/benches/deadlock_bench.rs` - Performance benchmark
- **Verify:** Existing tests in `deadlock.rs` (lines 207-354) must pass

---

## Task 1: Add BFS Helper Methods to Inner

**Files:**
- Modify: `crates/transaction/src/deadlock.rs:11-65`

- [ ] **Step 1: Add VecDeque import**

Modify line 2:
```rust
use std::collections::{HashMap, HashSet, VecDeque};
```

- [ ] **Step 2: Add bfs_reachable method to Inner (after line 64)**

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

- [ ] **Step 3: Add bfs_cycle method to DeadlockDetector (after line 198)**

```rust
    fn bfs_cycle(
        graph: &HashMap<TxId, HashSet<TxId>>,
        start: TxId,
        visited: &mut HashSet<TxId>,
        path: &mut Vec<TxId>,
    ) -> Option<Vec<TxId>> {
        let mut queue = VecDeque::new();
        queue.push_back((start, vec![start]));

        while let Some((current, current_path)) = queue.pop_front() {
            if path.contains(&current) {
                let idx = path.iter().position(|x| *x == current).unwrap();
                return Some(path[idx..].to_vec());
            }
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            if let Some(holders) = graph.get(&current) {
                for &holder in holders {
                    let mut new_path = current_path.clone();
                    new_path.push(holder);
                    queue.push_back((holder, new_path));
                }
            }
        }
        None
    }
```

- [ ] **Step 4: Run tests to verify no regressions**

Run: `cargo test -p sqlrustgo-transaction -- deadlock --nocapture`
Expected: All existing tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/transaction/src/deadlock.rs
git commit -m "perf(deadlock): add BFS helper methods"
```

---

## Task 2: Replace DFS with BFS in would_create_cycle

**Files:**
- Modify: `crates/transaction/src/deadlock.rs:28-35`

- [ ] **Step 1: Replace dfs_reachable call with bfs_reachable in would_create_cycle**

Modify lines 28-35:
```rust
    fn would_create_cycle(&self, from: TxId, to_set: &HashSet<TxId>) -> bool {
        for &to in to_set {
            if self.bfs_reachable(to, from) {
                return true;
            }
        }
        false
    }
```

- [ ] **Step 2: Remove now-unused dfs_reachable method**

Delete lines 49-64 (the entire `dfs_reachable` method).

- [ ] **Step 3: Run tests to verify**

Run: `cargo test -p sqlrustgo-transaction -- deadlock --nocapture`
Expected: All tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/transaction/src/deadlock.rs
git commit -m "perf(deadlock): replace DFS with BFS in would_create_cycle"
```

---

## Task 3: Replace DFS with BFS in detect_cycle

**Files:**
- Modify: `crates/transaction/src/deadlock.rs:146-154`

- [ ] **Step 1: Replace dfs_cycle call with bfs_cycle in detect_cycle**

Modify lines 146-154:
```rust
    pub fn detect_cycle(&self, start: TxId) -> Option<Vec<TxId>> {
        let inner = self.inner.lock().unwrap();
        Self::bfs_cycle(
            &inner.waits_for,
            start,
            &mut HashSet::new(),
            &mut Vec::new(),
        )
    }
```

- [ ] **Step 2: Remove now-unused dfs_cycle method**

Delete lines 174-198 (the entire `dfs_cycle` method).

- [ ] **Step 3: Run tests to verify**

Run: `cargo test -p sqlrustgo-transaction -- deadlock --nocapture`
Expected: All tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/transaction/src/deadlock.rs
git commit -m "perf(deadlock): replace DFS with BFS in detect_cycle"
```

---

## Task 4: Add Incremental Detection (Phase 2)

**Files:**
- Modify: `crates/transaction/src/deadlock.rs:77-81`

- [ ] **Step 1: Add IncrementalCache struct**

Add after line 76:
```rust
#[derive(Debug, Default)]
struct IncrementalCache {
    last_checked_tx: Option<TxId>,
    affected_paths: Vec<TxId>,
    version: u64,
}
```

- [ ] **Step 2: Add incremental_cache field to DeadlockDetector**

Modify lines 77-81:
```rust
#[derive(Debug)]
pub struct DeadlockDetector {
    inner: Mutex<Inner>,
    lock_wait_timeout: Duration,
    incremental_cache: IncrementalCache,
}
```

- [ ] **Step 3: Update with_timeout to initialize incremental_cache**

Modify lines 88-93:
```rust
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            inner: Mutex::new(Inner::default()),
            lock_wait_timeout: timeout,
            incremental_cache: IncrementalCache::default(),
        }
    }
```

- [ ] **Step 4: Add detect_cycle_incremental method**

Add after `detect_cycle` (after line 154):
```rust
    pub fn detect_cycle_incremental(&self, tx_id: TxId) -> Option<Vec<TxId>> {
        let inner = self.inner.lock().unwrap();

        if self.incremental_cache.last_checked_tx == Some(tx_id) {
            return None;
        }

        let result = Self::bfs_cycle(
            &inner.waits_for,
            tx_id,
            &mut HashSet::new(),
            &mut Vec::new(),
        );

        result
    }
```

- [ ] **Step 5: Run tests to verify**

Run: `cargo test -p sqlrustgo-transaction -- deadlock --nocapture`
Expected: All tests pass

- [ ] **Step 6: Commit**

```bash
git add crates/transaction/src/deadlock.rs
git commit -m "perf(deadlock): add incremental detection cache"
```

---

## Task 5: Add Performance Benchmark

**Files:**
- Create: `crates/transaction/benches/deadlock_bench.rs`

- [ ] **Step 1: Create benchmark file**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sqlrustgo_transaction::mvcc::TxId;
use sqlrustgo_transaction::deadlock::DeadlockDetector;

fn setup_detector(n_transactions: usize) -> DeadlockDetector {
    let detector = DeadlockDetector::new();

    for i in 0..n_transactions {
        let tx = TxId::new(i as u64);
        if i > 0 {
            let prev = TxId::new((i - 1) as u64);
            detector.add_edge_unsafe(tx, prev);
        }
    }

    detector
}

fn bench_detect_cycle(c: &mut Criterion) {
    let mut group = criterion_group("deadlock_detect_cycle");

    for n in [10, 50, 100, 200] {
        let detector = setup_detector(n);

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let tx_id = black_box(TxId::new(0));
                detector.detect_cycle(tx_id);
            });
        });
    }

    drop(group);
}

fn bench_try_wait_edge(c: &mut Criterion) {
    let mut group = criterion_group("deadlock_try_wait_edge");

    for n in [10, 50, 100] {
        let detector = Arc::new(setup_detector(n));

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &_| {
            b.iter(|| {
                let d = Arc::clone(&detector);
                d.try_wait_edge(
                    black_box(TxId::new(n as u64 + 1)),
                    black_box([TxId::new(0)].into_iter().collect()),
                );
            });
        });
    }

    drop(group);
}

criterion_group!(
    benches,
    bench_detect_cycle,
    bench_try_wait_edge
);
criterion_main!(benches);
```

- [ ] **Step 2: Run benchmark to verify setup**

Run: `cargo bench -p sqlrustgo-transaction -- deadlock --no-default-features`
Expected: Benchmark runs without errors

- [ ] **Step 3: Verify latency < 50ms target**

Check output for detect_cycle latency with 100 transactions.
Expected: < 50ms per call

- [ ] **Step 4: Commit**

```bash
git add crates/transaction/benches/deadlock_bench.rs
git commit -m "perf(deadlock): add benchmark for latency verification"
```

---

## Task 6: Final Verification

**Files:**
- No changes

- [ ] **Step 1: Run all transaction tests**

Run: `cargo test -p sqlrustgo-transaction --all-features`
Expected: All tests pass

- [ ] **Step 2: Run clippy**

Run: `cargo clippy -p sqlrustgo-transaction --all-features -- -D warnings`
Expected: No warnings

- [ ] **Step 3: Run format check**

Run: `cargo fmt --check -p sqlrustgo-transaction`
Expected: No formatting issues

- [ ] **Step 4: Run full benchmark**

Run: `cargo bench -p sqlrustgo-transaction -- --sample-size=100`
Expected: detect_cycle < 50ms for 100 concurrent transactions

- [ ] **Step 5: Commit final state**

```bash
git add -A
git commit -m "perf(deadlock): optimize detection to < 50ms latency"
```

---

## Spec Coverage Check

| Spec Requirement | Task | Status |
|-----------------|------|--------|
| BFS replacement | Task 1, 2, 3 | ✅ |
| Incremental detection | Task 4 | ✅ |
| Benchmark < 50ms | Task 5 | ✅ |
| Existing tests pass | All tasks | ✅ |
| API compatibility | All tasks | ✅ |

---

**Plan complete and saved to `docs/superpowers/plans/2026-05-15-deadlock-detection-optimization-plan.md`**

Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
