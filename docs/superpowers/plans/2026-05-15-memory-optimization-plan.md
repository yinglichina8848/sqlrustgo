# PERF-5 Memory Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce memory footprint by ≥15% vs v3.1.0 through incremental optimizations.

**Architecture:** Phase 1 uses faster/smaller data structures (FxHashMap, SmallStr). Phase 2 uses Buffer Pool compression.

**Tech Stack:** rustc_hash::FxHashMap, smol_str::SmolStr, Vec optimization.

---

## File Structure

- **Modify:** `Cargo.toml` (add dependencies)
- **Modify:** `crates/storage/src/buffer_pool.rs` (FxHashMap)
- **Modify:** `crates/storage/src/page.rs` (SmallStr)
- **Modify:** `crates/transaction/src/deadlock.rs` (FxHashMap)

---

## Phase 1: Conservative Optimizations

### Task 1: Add Dependencies

**Files:**
- Modify: `Cargo.toml` (workspace root)

- [ ] **Step 1: Add rustc_hash and smol_str to workspace Cargo.toml**

Add to `[workspace.dependencies]`:
```toml
rustc_hash = "1.1"
smol_str = "0.1"
```

- [ ] **Step 2: Add dependencies to sqlrustgo-storage**

Add to `crates/storage/Cargo.toml`:
```toml
[dependencies]
rustc_hash = { workspace = true }
```

- [ ] **Step 3: Add dependencies to sqlrustgo-transaction**

Add to `crates/transaction/Cargo.toml`:
```toml
[dependencies]
rustc_hash = { workspace = true }
```

- [ ] **Step 4: Verify dependencies resolve**

Run: `cargo check --workspace 2>&1 | tail -5`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/storage/Cargo.toml crates/transaction/Cargo.toml
git commit -m "perf(memory): add rustc_hash and smol_str dependencies"
```

---

### Task 2: Replace HashMap with FxHashMap in buffer_pool.rs

**Files:**
- Modify: `crates/storage/src/buffer_pool.rs`

- [ ] **Step 1: Add import**

Add after existing imports:
```rust
use rustc_hash::FxHashMap;
```

- [ ] **Step 2: Replace HashMap<u32, Arc<Page>> with FxHashMap**

In the `BufferPool` struct, change:
```rust
// FROM:
pages: Mutex<HashMap<u32, Arc<Page>>>,

// TO:
pages: Mutex<FxHashMap<u32, Arc<Page>>>,
```

- [ ] **Step 3: Update new() method**

Change:
```rust
// FROM:
pages: Mutex::new(HashMap::new()),

// TO:
pages: Mutex::new(FxHashMap::default()),
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p sqlrustgo-storage -- buffer_pool --nocapture 2>&1 | tail -20`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/storage/src/buffer_pool.rs
git commit -m "perf(memory): use FxHashMap in buffer_pool"
```

---

### Task 3: Replace HashMap with FxHashMap in deadlock.rs

**Files:**
- Modify: `crates/transaction/src/deadlock.rs`

- [ ] **Step 1: Add import**

Add after existing imports:
```rust
use rustc_hash::FxHashMap;
```

- [ ] **Step 2: Replace HashMap with FxHashMap in Inner struct**

Change:
```rust
// FROM:
struct Inner {
    waits_for: HashMap<TxId, HashSet<TxId>>,
}

// TO:
struct Inner {
    waits_for: FxHashMap<TxId, HashSet<TxId>>,
}
```

- [ ] **Step 3: Update new_with_timeout or with_timeout**

Change:
```rust
// FROM:
inner: Mutex::new(Inner::default()),

// TO (no change needed - default() works):
```

Note: `FxHashMap` implements `Default`, so `Inner::default()` continues to work.

- [ ] **Step 4: Run tests**

Run: `cargo test -p sqlrustgo-transaction -- deadlock --nocapture 2>&1 | tail -20`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/transaction/src/deadlock.rs
git commit -m "perf(memory): use FxHashMap in deadlock detector"
```

---

### Task 4: Replace String with SmolStr in page.rs

**Files:**
- Modify: `crates/storage/src/page.rs`

- [ ] **Step 1: Add import**

Add after existing imports:
```rust
use smol_str::SmolStr;
```

- [ ] **Step 2: Identify String fields in Page struct**

Look for `key: String` or similar fields in the Page struct. Replace with:
```rust
// FROM:
key: String,

// TO:
key: SmolStr,
```

- [ ] **Step 3: Update any String conversions**

Replace `String::from(...)` or `.to_string()` calls with `SmolStr::from(...)`.

- [ ] **Step 4: Run tests**

Run: `cargo test -p sqlrustgo-storage -- page --nocapture 2>&1 | tail -20`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/storage/src/page.rs
git commit -m "perf(memory): use SmolStr for page keys"
```

---

### Task 5: Phase 1 Verification

**Files:**
- No changes

- [ ] **Step 1: Run all storage tests**

Run: `cargo test -p sqlrustgo-storage --all-features 2>&1 | tail -10`
Expected: All tests pass

- [ ] **Step 2: Run all transaction tests**

Run: `cargo test -p sqlrustgo-transaction --all-features 2>&1 | tail -10`
Expected: All tests pass

- [ ] **Step 3: Run clippy**

Run: `cargo clippy --workspace --all-features -- -D warnings 2>&1 | tail -10`
Expected: Zero warnings

- [ ] **Step 4: Run format check**

Run: `cargo fmt --check --all 2>&1`
Expected: No formatting issues

- [ ] **Step 5: Commit Phase 1 completion**

```bash
git add -A
git commit -m "perf(memory): Phase 1 complete - FxHashMap + SmolStr"
```

---

## Phase 2: Aggressive Optimizations

### Task 6: Buffer Pool Vec Optimization

**Files:**
- Modify: `crates/storage/src/buffer_pool.rs`

- [ ] **Step 1: Analyze current BufferPool structure**

Read the full `buffer_pool.rs` to understand:
- How pages are stored and accessed
- The LRU eviction logic
- Memory overhead of current design

- [ ] **Step 2: Consider Option<Vec<Arc<Page>>> optimization**

If appropriate, change from `HashMap<u32, Arc<Page>>` to `Vec<Option<Arc<Page>>>`:
```rust
// Dense array, lower overhead per entry
pages: Vec<Option<Arc<Page>>>,
```

This only works if page_ids are dense (0 to N). If not dense, skip this optimization.

- [ ] **Step 3: Run tests and benchmark**

If optimization is applied:
Run: `cargo test -p sqlrustgo-storage -- buffer_pool 2>&1 | tail -10`
Run: `cargo bench -- buffer 2>&1 | tail -10`
Expected: Tests pass, memory usage reduced

- [ ] **Step 4: Commit**

```bash
git add crates/storage/src/buffer_pool.rs
git commit -m "perf(memory): use Vec for dense page storage"
```

---

### Task 7: Final Verification

**Files:**
- No changes

- [ ] **Step 1: Run full workspace tests**

Run: `cargo test --workspace --all-features 2>&1 | tail -10`
Expected: All tests pass

- [ ] **Step 2: Run full workspace clippy**

Run: `cargo clippy --workspace --all-features -- -D warnings 2>&1 | tail -10`
Expected: Zero warnings

- [ ] **Step 3: Run format check**

Run: `cargo fmt --check --all 2>&1`
Expected: No formatting issues

- [ ] **Step 4: Commit final state**

```bash
git add -A
git commit -m "perf(memory): Phase 2 complete - memory optimization finished"
```

---

## Spec Coverage Check

| Spec Requirement | Task | Status |
|-----------------|------|--------|
| HashMap → FxHashMap | Task 2, 3 | ✅ |
| String → SmolStr | Task 4 | ✅ |
| Buffer Pool optimization | Task 6 | ✅ |
| Memory reduction ≥15% | All tasks | Pending verification |
| Tests pass | Tasks 1-5, 7 | ✅ |
| Clippy zero warnings | Tasks 1-5, 7 | ✅ |

---

**Plan complete and saved to `docs/superpowers/plans/2026-05-15-memory-optimization-plan.md`**

Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
