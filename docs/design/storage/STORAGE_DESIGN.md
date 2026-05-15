# Storage Design

This document describes the storage layer design for SqlRustGo, covering buffer pool management, B+ Tree indexes, columnar storage, and page layout.

## Page Layout

### Page Structure

Pages are the fundamental unit of storage (default 8KB). Each page contains:

- **Page Header (32 bytes)**: Page type, LSN, checksum, free space pointer
- **Body**: Actual data or index content
- **Special Space**: For variable-length data (TOAST, overflow)

### Page Types

| Type | Description |
|------|-------------|
| `HEAP_PAGE` | Row-store heap relation |
| `BTREE_PAGE` | B+ Tree internal/leaf page |
| `COLUMNAR_PAGE` | Columnar storage chunk |
| `INDEX_PAGE` | Secondary index page |
| `TOAST_PAGE` | Oversized attribute storage |
| `FSM_PAGE` | Free Space Map |

### Row-Store Page Layout

```
+------------------+
| PageHeader (32B) |
|------------------|
| ItemswitchPoint  |
|------------------|
| Item Pointers    |  <- grows down
|     ...          |
|------------------|
|     ...          |
| Actual Row Data  |  <- grows up
|     ...          |
|------------------+
| Special Space    |
+------------------+
```

### Columnar Page Layout

Columnar pages store column values contiguously:

```
+------------------+
| PageHeader (32B) |
|------------------|
| Column 0 values |
| Column 1 values |
| Column 2 values |
|     ...         |
|------------------+
| Statistics Area  |
| (min/max/nulls) |
+------------------+
```

## BufferPool

### Overview

The BufferPool manages pages in memory, providing caching and write coalescing between memory and disk.

### Buffer Pool Structure

```
BufferPool
├── Page Table (hash-based page lookup)
├── LRU Clock (eviction)
├── Flush List (dirty page tracking)
└── Buffer Frames (actual page data)
```

### Key Components

**Page Table**: Maps `(relfilenode, forkNum, blockNum)` to a buffer frame. Uses a hash table for O(1) lookup.

**Buffer Frame**: Each frame holds:
- `page`: Raw page data
- `page_id`: Physical page identifier
- `refcount`: Pin count (prevents eviction)
- `dirty`: Boolean indicating modifications
- `lsn`: Last modification LSN for WAL

**LRU Clock**: Clock-sweep algorithm for eviction with:
- Reference bit for second-chance behavior
- Usage history tracking
- Adaptive eviction threshold

### Buffer Pool Operations

**Pin/Unpin**: Increment/decrement reference count. Pinned pages cannot be evicted.

**Fetch**: Load page from disk (or return cached). Steps:
1. Hash lookup in page table
2. If found, pin and return
3. If not found, allocate frame, load from disk, pin, return

**Flush**: Write dirty page to disk. Background writer coalesces nearby dirty pages.

### Concurrency

Buffer pool uses fine-grained locking:
- Individual frame locks for pin/unpin
- Page table uses sharding (multiple hash tables) to reduce contention

## B+ Tree Index

### Overview

B+ Tree provides ordered index traversal with O(log n) lookup, range scans, and sequential I/O optimization.

### Tree Structure

```
Root
├── Internal Node (keys, child pointers)
│   ├── Internal Node
│   │   ├── Leaf Node (keys, heap row pointers)
│   │   └── Leaf Node
│   └── Internal Node
│       ├── Leaf Node
│       └── Leaf Node
```

### Page Layout (Internal Node)

```
+------------------+
| BTreePageHeader  |
|------------------|
| Parent Info      |
|------------------|
| Key[0]           |
| ChildPtro[0]     |
| Key[1]           |
| ChildPtro[1]     |
|     ...          |
+------------------+
```

### Page Layout (Leaf Node)

```
+------------------+
| BTreePageHeader  |
|------------------+
| Parent Info      |
|------------------|
| High Key         |
| Key[0] | RowPtr[0]|
| Key[1] | RowPtr[1]|
|     ...          |
| Next Leaf Pointer|
+------------------+
```

### B+ Tree Operations

**Search**: Descend tree following key comparisons. Leaf pages linked for range scans.

**Insert**: 
1. Find leaf where key belongs
2. Insert key/tuple pair
3. Split if overflow (50/50 split or 90/10 for right-heavy)

**Delete**:
1. Find leaf containing key
2. Remove entry
3. Coalesce with sibling if underflow (< 30% fill)

### Index Types

| Type | Description |
|------|-------------|
| `BTREE_INDEX` | Standard B+ Tree |
| `BTREE_UNIQUE` | Unique constraint enforcement |
| `BTREE_RANGE` | Range queries |
| `BTREE_HASH` | Hash-style exact match (no range) |

### Optimization

- **Bulk Insert**: Sort keys, build tree bottom-up
- **Compression**: Prefix key compression within pages
- **Covering Index**: Index-only scans when all needed columns in index

## Columnar Storage

### Overview

Columnar storage organizes data by column rather than row, enabling:
- Better compression (similar values together)
- Vectorized execution (batch column operations)
- Improved analytical query performance

### Columnar Page Structure

Each columnar page stores one or more columns:

```
ColumnarPage
├── Page Header
├── Column Data (value array)
├── Run-Length Encoding (if applicable)
├── Dictionary (if applicable)
└── Statistics (min, max, null count, distinct count)
```

### Value Encoding

| Encoding | Use Case |
|----------|----------|
| `PLAIN` | No compression, direct storage |
| `RLE` | Repeated consecutive values |
| `DICT` | Low cardinality strings |
| `BITPACK` | Integer delta encoding |
| `FOR` | Fixed-width offset for strings |
| `LZ4` | General-purpose compression |

### Statistics

Each columnar page maintains:
- `null_count`: Number of NULL values
- `min_value`: Minimum value (for pruning)
- `max_value`: Maximum value (for pruning)
- `distinct_count`: Approximate distinct values

### Row vs Columnar

| Aspect | Row Store | Columnar |
|--------|-----------|----------|
| Write Performance | Good | Moderate |
| Read Performance | Good (point) | Excellent (analytics) |
| Compression | Moderate | High |
| Tuple Overhead | Higher | Lower |

### Hybrid Storage

SqlRustGo supports hybrid storage where:
- OLTP workloads use row-store (HEAP)
- OLAP workloads use columnar (arrow format)
- Fresh data buffered in row-store, archived to columnar

## Page Version Management

### Overview

MVCC (Multi-Version Concurrency Control) page-level versioning for transaction isolation.

### Version Chain

Each row version points to newer/older versions:

```
Row Version N (in-progress)
  |
  v (next)
Row Version N-1 (visible to T1, invisible to T2)
  |
  v (next)
Row Version N-2 (committed)
```

### VACUUM

Background vacuum process:
1. Mark dead tuples for reuse
2. Reclaim unused page space
3. Update free space map
4. Freeze old tuple versions

## File Organization

### Relation Files

Each table/index stored as relation with multiple forks:

| Fork | Content |
|------|---------|
| `MAIN` | Primary data |
| `FSM` | Free Space Map |
| `VM` | Visibility Map |
| `INIT` | Initialization fork |

### Tablespace

Support for multiple tablespaces with per-tablespace:
- Location (directory or mount point)
- Access method selection
- Storage parameters
