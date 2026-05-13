# ARCH-1: Clustered Index & SQLRustGo Compact Row v1 Design

**Date**: 2026-05-13
**Status**: Approved
**Author**: SQLRustGo Team

---

## 1. Overview

### 1.1 Goals

Implement clustered index support for SQLRustGo with a stable, future-proof storage ABI.

### 1.2 Non-Goals

- MVCC (Phase B)
- Page defragmentation
- Prefix compression
- Page-level compression
- Overflow GC (Phase B+)

---

## 2. Core Abstraction: Logical ID vs Physical Pointer

### 2.1 Critical Invariant

```
HiddenRowId (logical identity) ≠ (page_id, slot_id) (physical location)
```

**Rationale**: Physical locations change during page splits, compaction, vacuum, etc. Using physical pointers as cluster keys would invalidate all secondary indexes on row relocation.

### 2.2 Types

```rust
/// Stable logical identity - never changes, not recycled in Phase A
pub type HiddenRowId = u64;

/// Physical pointer - internal to page, used for slot navigation only
pub struct RowPointer {
    pub page_id: u32,
    pub slot_id: u16,
}
```

### 2.3 ClusterKey

```rust
pub enum ClusterKey {
    PrimaryKey(KeyValue),
    HiddenRowId(u64),
}
```

---

## 3. Row Format ABI (SQLRustGo Compact Row v1)

### 3.1 RowHeader (Required)

```rust
#[repr(C)]
pub struct RowHeader {
    pub format_version: u8,   // v1 = 1
    pub flags: u16,          // future: deleted, locked, etc.
    pub trx_id: u64,         // future MVCC - set to 0 in Phase A
    pub undo_ptr: u64,       // future undo chain - set to 0 in Phase A
}
```

**Invariant**: `RowHeader` is always present, even if fields are unused.

### 3.2 ClusteredLeafRecord

```rust
pub struct ClusteredLeafRecord {
    pub header: RowHeader,
    pub cluster_key: ClusterKey,

    // Fixed-length columns (inline)
    pub fixed_data: Vec<u8>,

    // Variable-length column directory
    pub varlen_slots: Vec<VarLenSlot>,

    // NULL bitmap
    pub null_bitmap: Vec<u8>,
}
```

### 3.3 VarLenSlot

```rust
pub struct VarLenSlot {
    /// Inline length (0 = NULL)
    pub inline_len: u16,

    /// Small field inline data (≤ 128 bytes)
    pub inline_data: Option<Vec<u8>>,

    /// Overflow page reference for large fields (> 128 bytes)
    pub overflow_page: Option<PageId>,
    pub overflow_len: Option<u32>,
}
```

### 3.4 Threshold

| Field Size | Behavior |
|------------|----------|
| ≤ 128 bytes | Inline in `inline_data` |
| > 128 bytes | Overflow page |

### 3.5 OverflowPage

```rust
pub struct OverflowPage {
    pub next_page: Option<PageId>,  // Chain for fields > 1 page
    pub data: Vec<u8>,
}
```

**Invariant**: Updates create new overflow chains; old chains are left dangling for Phase B vacuum.

---

## 4. HiddenRowId Generator

### 4.1 Trait

```rust
pub trait RowIdGenerator {
    fn next_id(&mut self) -> HiddenRowId;
}
```

### 4.2 Default Implementation

```rust
pub struct DefaultRowIdGenerator {
    node_id: u16,
    local_counter: u64,
}

impl RowIdGenerator for DefaultRowIdGenerator {
    fn next_id(&mut self) -> HiddenRowId {
        let id = (self.node_id as u64) << 48 | self.local_counter;
        self.local_counter += 1;
        id
    }
}
```

### 4.3 Invariant

- `HiddenRowId` is **immutable** - never updated
- `HiddenRowId` is **never recycled** in Phase A

---

## 5. Directory Structure

```
crates/storage/src/
├── page/
├── bplus_tree/
│   └── index.rs              # Existing non-clustered B+Tree
├── row_format/                # NEW - Independent storage ABI
│   ├── mod.rs
│   ├── header.rs            # RowHeader, ClusterKey
│   ├── encoder.rs           # encode_row()
│   ├── decoder.rs           # decode_row()
│   ├── null_bitmap.rs       # NULL bitmap helpers
│   ├── overflow.rs          # OverflowPage, overflow chain
│   └── types.rs             # VarLenSlot, ClusteredLeafRecord
├── clustered_index/           # NEW - Clustered B+Tree engine
│   ├── mod.rs
│   ├── leaf.rs              # ClusteredLeafRecord
│   ├── overflow.rs          # OverflowPage management
│   ├── page_split.rs        # Page split logic
│   └── scan.rs              # Range scan
└── ...
```

**Key Principle**: `row_format/` is independent from `clustered_index/`. Future engines (heap, column store, LSM, undo segment) will reuse it.

---

## 6. Secondary Index Entry

```rust
pub struct SecondaryIndexEntry {
    pub secondary_key: KeyValue,
    pub cluster_key: ClusterKey,  // PrimaryKey or HiddenRowId
}
```

**Invariant**: Secondary indexes always reference `ClusterKey`, never `RowPointer`.

---

## 7. Page Split Invariants

1. **Record relocation is transparent** - `RowPointer` changes, `ClusterKey` does not
2. **Secondary indexes remain valid** - they reference `ClusterKey`, not physical location
3. **Split creates new page** - old page may be marked for future vacuum

---

## 8. Invariants to Maintain

### 8.1 Record Decode Safety
```
forall valid page: decode(encode(x)) == x
```

### 8.2 Overflow Chain Acyclic
```
forall overflow: overflow.next != self && no cycles
```

### 8.3 NULL Bitmap Correctness
```
column IS NULL => no payload access
```

### 8.4 Cluster Key Stability
```
ClusterKey is immutable after insert
```

### 8.5 HiddenRowId Immutability
```
UPDATE cannot change HiddenRowId
```

---

## 9. Implementation Phases

### Phase A (Current)

| Week | Deliverable |
|------|-------------|
| 1 | `row_format/` ABI - encoder, decoder, null_bitmap, types |
| 2 | Clustered leaf page, overflow page, HiddenRowId generator |
| 3 | Page split, scan, insert, delete |
| 4 | WAL integration, recovery validation |
| 5-6 | Benchmark, fuzz tests, invariant verification |

### Phase B (Future)

- RowHeader.trx_id population
- Snapshot visibility
- Undo chain (pointer only)

### Phase C (Future)

- Page defragmentation
- Hot/cold page split
- Prefix compression

---

## 10. Testing Requirements

### 10.1 Unit Tests

- `encode/decode roundtrip` - `decode(encode(x)) == x`
- `null_bitmap correctness`
- `overflow chain acyclic`
- `page split preserves cluster key`

### 10.2 Integration Tests

- Clustered index CRUD
- Secondary index with clustered key
- WAL replay after crash

### 10.3 Fuzz Tests

- Random row format encode/decode
- Random page split sequences

---

## 11. References

- InnoDB Row Format (COMPACT)
- PostgreSQL TID abstraction
- RocksDB internal key vs sequence number
