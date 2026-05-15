# PERF-5 Memory Optimization Design

**Date**: 2026-05-15
**Issue**: #990 (PERF-5)
**Target**: Memory reduction ≥15% vs v3.1.0
**Strategy**: Two-phase incremental optimization

---

## 1. Overview

Reduce memory footprint by ≥15% compared to v3.1.0 through incremental optimizations.

### 1.1 Optimization Directions

1. **Memory Allocator** - Use faster/smaller hash maps
2. **Data Structure Compression** - Use stack-allocated types where possible
3. **Cache Memory** - Optimize buffer pool memory usage
4. **Zero-Copy** - Reduce unnecessary allocations

### 1.2 Two-Phase Strategy

| Phase | Changes | Expected Reduction |
|-------|---------|-------------------|
| Phase 1 | HashMap→FxHashMap, SmallString, inline optimizations | 5-10% |
| Phase 2 | Buffer Pool compression, Vec optimization | 5-10% (total ≥15%) |

---

## 2. Phase 1: Conservative Optimizations

### 2.1 HashMap → FxHashMap

Replace `std::collections::HashMap` with `rustc_hash::FxHashMap` in hot paths:

**Files to modify:**
- `crates/storage/src/buffer_pool.rs`
- `crates/storage/src/file_storage.rs`
- `crates/transaction/src/deadlock.rs`

**FxHashMap benefits:**
- Faster hashing (no DoS resistance needed in internal use)
- Lower memory overhead per entry

### 2.2 Small String Optimization

Replace `String` with `smol_str::SmolStr` for short keys:

**Files to modify:**
- `crates/storage/src/page.rs` (key fields)

**SmolStr benefits:**
- Stack-allocated for strings ≤ 23 bytes
- No heap allocation for typical keys

### 2.3 Inline small allocations

- Replace `Box<[u8]>` with `Vec<u8>` where appropriate
- Use `[u8; N]` stack allocation for fixed-size data

---

## 3. Phase 2: Aggressive Optimizations

### 3.1 Buffer Pool Compression

**Current state:**
```rust
pages: Mutex<HashMap<u32, Arc<Page>>>
```

**Optimized:**
```rust
pages: Vec<Option<Arc<Page>>>  // Dense array, lower overhead
```

### 3.2 Page Compression

- Use `#[repr(C)]` for page headers
- Reduce padding in `Page` struct
- Use `u32` instead of `u64` where safe

---

## 4. Memory Measurement

### 4.1 Benchmark

```bash
# Measure baseline
cargo run --bin sqlrustgo -- --memory-stats

# Target: ≥15% reduction
```

### 4.2 Key Metrics

| Metric | v3.1.0 | Target | Phase 1 | Phase 2 |
|--------|--------|--------|---------|---------|
| Idle memory | X MB | ≤0.85X | X-5% | X-15% |
| 10K connections | Y MB | ≤0.85Y | Y-5% | Y-15% |

---

## 5. Files to Modify

### Phase 1
- `crates/storage/src/buffer_pool.rs`
- `crates/storage/src/page.rs`
- `Cargo.toml` (add smol_str, rustc_hash)

### Phase 2
- `crates/storage/src/buffer_pool.rs` (major restructure)
- `crates/storage/src/row_format/`

---

## 6. Verification

```bash
# Phase 1
cargo test -p sqlrustgo-storage
cargo clippy -p sqlrustgo-storage -- -D warnings

# Phase 2
cargo bench -- memory
# Verify ≥15% reduction
```

---

## 7. Risks

| Risk | Mitigation |
|------|------------|
| FxHashMap not deterministic | Use only in non-critical paths first |
| Buffer Pool restructure breaks stability | Extensive testing after Phase 2 |
| Performance regression | Benchmark before/after each change |
