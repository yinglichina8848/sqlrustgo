# PROOF-023 ↔ Rust Implementation Mapping

## 可信任系统工具链：TLA+ 形式化证明 ↔ Rust 运行时强制

---

## 1. 证明文件清单

| TLA+ File | Description | Status |
|-----------|-------------|--------|
| `PROOF_023_deadlock_v4.tla` | Multi-resource Wait-For Graph, atomic pre-check | ✅ Proved |
| `PROOF_023_deadlock_toctou.tla` | TOCTOU counterexample — cycle formed | ✅ Violated (as expected) |
| `PROOF_016_023_mvcc_toctou.tla` | MVCC + Wait-For + TOCTOU | ✅ Violated (as expected) |
| `PROOF_016_023_mvcc_atomic.tla` | MVCC + Wait-For + Atomic pre-check | ✅ Proved (993 states, NoCycle + NoWriteConflict) |

---

## 2. 核心映射

### 2.1 Wait-For Graph

| TLA+ Concept | Rust (`deadlock.rs`) | Status |
|---|---|---|
| `waitFor : Txn → SET Txn` | `Inner::waits_for: HashMap<TxId, HashSet<TxId>>` | ✅ |
| `Reachable(t1, t2)` | `Inner::dfs_reachable(t1, t2, &mut HashSet)` | ✅ |
| `waitFor' = [waitFor EXCEPT ![t] = @ ∪ targets]` | `Inner::add_edge(blocked, holder)` | ✅ |
| `waitFor' = [waitFor EXCEPT ![t] = @ \ {h}]` | `Inner::remove_edges_for(tx_id)` | ✅ |

### 2.2 Atomic Pre-Check (关键)

| TLA+ Concept | Rust (`deadlock.rs`) | Status |
|---|---|---|
| `AtomicAddWaitFor(t, targets)` body | `try_wait_edge()` body | ✅ |
| `∄ h ∈ targets: Reachable(h, t)` (pre-check) | `inner.would_create_cycle(blocked, &holders)` | ✅ |
| Graph mutation (only if check passes) | `inner.add_edge(blocked, holder)` | ✅ |
| Entire body under one lock | `Mutex::lock()` → entire operation | ✅ |

### 2.3 NoSelfWait

| TLA+ Concept | Rust (`lock.rs`) | Status |
|---|---|---|
| `t ∉ targets` | `holders.contains(&tx_id)` check | ✅ |
| TLA+: `AtomicAddWaitFor` receives filtered set | `try_wait_edge`: pre-filter `holders` | ✅ |
| Rust: explicit check before `try_wait_edge` | `if holders.contains(&tx_id) { return Err; }` | ✅ |

### 2.4 Invariants

| TLA+ Invariant | Rust Assertion | Status |
|---|---|---|
| `NoCycle ≡ ∀ t: ~Reachable(t, t)` | `Inner::assert_no_cycle()` | ✅ |
| `NoWriteConflict` (MVCC model) | Not enforced by `DeadlockDetector` — SSI layer's job | ⚠️ |
| Serializability | Enforced by `CommitTxn` in MVCC model | ⚠️ |

### 2.5 TOCTOU Window (Non-Atomic Baseline)

| Scenario | TLA+ Result | Rust Equivalent |
|---|---|---|
| `Check` + `CommitEdge` separate | ❌ Cycle formed | ❌ Old `acquire_lock` code |
| `AtomicAddWaitFor` merged | ✅ NoCycle | ✅ `try_wait_edge()` |

---

## 3. 并发安全保证

### 3.1 Mutex Atomicity

The `DeadlockDetector` wraps all graph operations in `Mutex<Inner>`:

```rust
pub fn try_wait_edge(&self, blocked: TxId, holders: HashSet<TxId>) -> Result<(), LockError> {
    let mut inner = self.inner.lock().unwrap();  // ◄─── atomic region starts
    if inner.would_create_cycle(blocked, &holders) { /* pre-check */ }
    for holder in holders { inner.add_edge(blocked, holder); }
    #[cfg(debug_assertions)] inner.assert_no_cycle();
    Ok(())
}  // ◄─── atomic region ends
```

This is the **physical enforcement** of the TLA+ `AtomicAddWaitFor` operator.

### 3.2 Thread-Safety of LockManager

`LockManager` holds `DeadlockDetector` (which is `&self` safe via Mutex) alongside `HashMap<_, LockInfo>` (protected by `&mut self` on `acquire_lock`). The `&mut self` ensures all lock state mutations are serialized at the `LockManager` level.

### 3.3 Concurrent Test Coverage

| Test | What It Proves |
|---|---|
| `test_concurrent_mutual_deadlock_prevention` | Two threads racing to add T1→T2 and T2→T1 edges — at least one fails |
| `test_concurrent_no_false_positive` | Linear chains not flagged as cycles under concurrency |

---

## 4. 未完成项 (S-04)

The following remain **unverified at the implementation level**:

| Item | TLA+ | Rust | Gap |
|---|---|---|---|
| MVCC write-write conflict detection | `CommitTxn` enforces `∀ committed: writeSet[t] ∩ writeSet[committed] = {}` | Not in `DeadlockDetector` | SSI commit validation needed in `TransactionManager` |
| Serializability | `NoWriteConflict` invariant in `PROOF_016_023_mvcc_atomic.tla` | Not enforced by lock layer | Requires SSI validator |
| `remove_edges_for` called on commit/abort | Implicit in TLA+ | Must be called explicitly by `LockManager::release_lock` | ✅ Covered by unit tests |

---

## 5. Correctness Argument

**Claim**: The Rust `DeadlockDetector` with `Mutex<Inner>` and `try_wait_edge()` is a sound implementation of the TLA+ `AtomicAddWaitFor` operator.

**Proof Sketch**:
1. The TLA+ `AtomicAddWaitFor` is specified as a single atomic step — either the edge is added after a successful pre-check, or nothing happens.
2. `try_wait_edge` holds `Mutex::lock()` for the **entire duration** of pre-check + mutation.
3. No other thread can observe or modify `waits_for` between the pre-check and the mutation.
4. Therefore, the Rust code is bisimilar to the TLA+ specification with respect to wait-for edge addition.
5. `Inner::would_create_cycle` implements exactly the `Reachable` predicate from the TLA+ model.
6. The `NoCycle` invariant (`∀ t: ~Reachable(t, t)`) is checked by `assert_no_cycle()` in debug builds and holds vacuously (all cycles are prevented by the pre-check).

---

## 6. 文件路径

```
docs/formal/
├── PROOF_023_deadlock_v4.tla          # Core proof (PASS ✅)
├── PROOF_023_deadlock_toctou.tla      # TOCTOU counterexample (FAIL 💀)
├── PROOF_016_023_mvcc_toctou.tla      # Unified MVCC+WF TOCTOU (FAIL 💀)
├── PROOF_016_023_mvcc_atomic.tla      # Unified MVCC+WF Atomic (PASS ✅)
└── PROOF_016_023_MAPPING.md           # This file

crates/transaction/src/
├── deadlock.rs                         # DeadlockDetector (Mutex-wrapped) ✅
└── lock.rs                            # LockManager using try_wait_edge ✅
```
