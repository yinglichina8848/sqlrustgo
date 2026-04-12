# Columnar Storage Block Skip Optimization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add block-level min/max statistics to ColumnChunk for skip-based predicate evaluation during scans.

**Architecture:** Divide ColumnChunk into fixed-size blocks (1024 rows), each with independent min/max stats. During scan with predicate, check block stats first to skip blocks that cannot possibly match.

**Tech Stack:** Rust, SQLRustGo storage engine

---

## Task 1: Add Block-Level Statistics to ColumnChunk

**Files:**
- Modify: `crates/storage/src/columnar/chunk.rs`

**Step 1: Add block-level fields to ColumnChunk struct**

Find the ColumnChunk struct (around line 154) and add new fields:

```rust
const BLOCK_SIZE: usize = 1024;

pub struct ColumnChunk {
    data: Vec<Value>,
    null_bitmap: Option<Bitmap>,
    stats: ColumnStats,
    block_mins: Vec<Option<Value>>,  // NEW: per-block min values
    block_maxes: Vec<Option<Value>>, // NEW: per-block max values
}
```

**Step 2: Update ColumnChunk constructor and methods**

Modify `new()`, `with_capacity()`, `from_values()` to initialize block arrays:

```rust
pub fn new() -> Self {
    Self {
        data: Vec::new(),
        null_bitmap: None,
        stats: ColumnStats::new(),
        block_mins: Vec::new(),
        block_maxes: Vec::new(),
    }
}
```

**Step 3: Add block index helper method**

Add to ColumnChunk:

```rust
fn get_block_index(&self, row_index: usize) -> usize {
    row_index / BLOCK_SIZE
}

fn ensure_block_exists(&mut self, block_idx: usize) {
    while self.block_mins.len() <= block_idx {
        self.block_mins.push(None);
        self.block_maxes.push(None);
    }
}
```

**Step 4: Update push_value to track block stats**

Modify `push_value()` to update block-level min/max:

```rust
fn push_value(&mut self, value: Value, is_null: bool) {
    let index = self.data.len();
    let block_idx = self.get_block_index(index);
    
    self.ensure_block_exists(block_idx);
    
    // ... existing bitmap code ...
    
    self.data.push(value.clone());
    self.stats.update(&value, is_null);
    
    // NEW: Update block-level stats
    if !is_null {
        // Update block min
        if let Some(ref mut min) = self.block_mins[block_idx] {
            if value < *min {
                *min = value.clone();
            }
        } else {
            self.block_mins[block_idx] = Some(value.clone());
        }
        
        // Update block max
        if let Some(ref mut max) = self.block_maxes[block_idx] {
            if value > *max {
                *max = value.clone();
            }
        } else {
            self.block_maxes[block_idx] = Some(value.clone());
        }
    }
}
```

**Step 5: Add can_skip_block method**

Add to ColumnChunk:

```rust
pub fn can_skip_block(&self, block_idx: usize, predicate: &crate::predicate::Predicate, col_name: &str) -> bool {
    let col_idx = self.column_index; // Need to track this
    
    match predicate {
        crate::predicate::Predicate::Gt(_, const_val) => {
            if let Some(max) = &self.block_maxes[block_idx] {
                return max <= const_val;
            }
        }
        crate::predicate::Predicate::Lt(_, const_val) => {
            if let Some(min) = &self.block_mins[block_idx] {
                return min >= const_val;
            }
        }
        crate::predicate::Predicate::Gte(_, const_val) => {
            if let Some(max) = &self.block_maxes[block_idx] {
                return max < const_val;
            }
        }
        crate::predicate::Predicate::Lte(_, const_val) => {
            if let Some(min) = &self.block_mins[block_idx] {
                return min > const_val;
            }
        }
        crate::predicate::Predicate::Eq(_, const_val) => {
            if let Some(min) = &self.block_mins[block_idx] {
                if let Some(max) = &self.block_maxes[block_idx] {
                    return min > const_val || max < const_val;
                }
            }
        }
        // Not(..) and others - conservatively don't skip
    }
    false
}
```

**Step 6: Run tests to verify**

Run: `cargo test -p sqlrustgo-storage -- columnar`
Expected: Existing tests should still pass (may need minor updates for new struct fields)

---

## Task 2: Add Block Skip Logic to ColumnarStorage Scan

**Files:**
- Modify: `crates/storage/src/columnar/storage.rs`

**Step 1: Update scan_predicate_with_limit to use block skip**

Find `scan_predicate_with_limit` method (around line 428). Modify to use block-level skipping:

```rust
fn scan_predicate_with_limit(
    &self,
    table: &str,
    predicate: &crate::predicate::Predicate,
    limit: usize,
) -> crate::engine::SqlResult<Vec<Vec<Value>>> {
    let store = match self.tables.get(table) {
        Some(s) => s,
        None => return Ok(vec![]),
    };

    // Extract column name from predicate
    let col_name = extract_column_name_from_predicate(predicate);
    
    let mut filtered = Vec::new();
    let row_count = store.row_count();
    let num_blocks = (row_count + BLOCK_SIZE - 1) / BLOCK_SIZE;
    
    for block_idx in 0..num_blocks {
        if filtered.len() >= limit {
            break;
        }
        
        let block_start = block_idx * BLOCK_SIZE;
        let block_end = std::cmp::min(block_start + BLOCK_SIZE, row_count);
        
        // Try to skip block based on stats
        if let Some(ref col_name) = col_name {
            if let Some(chunk) = store.columns.get(&store.column_indices.get(col_name)) {
                if chunk.can_skip_block(block_idx, predicate, col_name) {
                    continue; // Skip entire block
                }
            }
        }
        
        // Scan block row by row
        for i in block_start..block_end {
            if filtered.len() >= limit {
                break;
            }
            if let Some(row) = store.get_row(i) {
                if self.eval_predicate_for_scan(table, &row, predicate) {
                    filtered.push(row);
                }
            }
        }
    }
    Ok(filtered)
}
```

**Step 2: Add helper to extract column name from predicate**

```rust
fn extract_column_name_from_predicate(predicate: &crate::predicate::Predicate) -> Option<String> {
    match predicate {
        crate::predicate::Predicate::Eq(expr, _) 
        | crate::predicate::Predicate::Lt(expr, _)
        | crate::predicate::Predicate::Lte(expr, _)
        | crate::predicate::Predicate::Gt(expr, _)
        | crate::predicate::Predicate::Gte(expr, _) => {
            match expr {
                crate::predicate::Expr::Column(name) => Some(name.clone()),
                _ => None,
            }
        }
        crate::predicate::Predicate::And(left, right) => {
            extract_column_name_from_predicate(left)
                .or_else(|| extract_column_name_from_predicate(right))
        }
        // For IsNull, In, etc - return None (can't skip easily)
        _ => None,
    }
}
```

**Step 3: Run tests**

Run: `cargo test -p sqlrustgo-storage -- columnar`
Expected: Tests pass

---

## Task 3: Add Block Skip Tests

**Files:**
- Create: `tests/integration/block_skip_test.rs`

**Step 1: Write block skip performance test**

```rust
#[test]
fn test_block_skip_optimization() {
    let mut storage = ColumnarStorage::new();
    
    // Insert 10000 rows where first half has id < 5000, second half >= 5000
    let info = TableInfo {
        name: "skip_test".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                // ... other fields ...
            },
        ],
    };
    storage.create_table(&info).unwrap();
    
    for i in 0..10000 {
        storage.insert("skip_test", vec![vec![Value::Integer(i as i64)]]).unwrap();
    }
    
    // Query: WHERE id > 5000 LIMIT 10
    // Without block skip: would scan all 10000 rows
    // With block skip: should skip first ~5 blocks (ids 0-5000)
    let start = Instant::now();
    let result = storage.scan_predicate_with_limit(
        "skip_test",
        &Predicate::Gt(Expr::Column("id".to_string()), Value::Integer(5000)),
        10
    ).unwrap();
    let elapsed = start.elapsed();
    
    assert_eq!(result.len(), 10);
    println!("Block skip query took: {:?}", elapsed);
}
```

**Step 2: Run test to verify**

Run: `cargo test --test block_skip_test -- --nocapture`
Expected: Test passes and shows timing improvement

---

## Task 4: Verify Full Test Suite

**Step 1: Run all tests**

Run: `cargo test --quiet 2>&1 | tail -30`
Expected: All tests pass

**Step 2: Commit changes**

```bash
git add -A
git commit -m "feat(storage): add block-level skip optimization for columnar storage

- Add block_mins/block_maxes arrays to ColumnChunk
- Implement can_skip_block() for predicate-based block elimination
- Update scan_predicate_with_limit to use block skip
- Add block skip integration test"
```

---

## Implementation Notes

1. **BLOCK_SIZE**: Currently set to 1024. This is a reasonable default - not too small (overhead) or too large (poor skip granularity).

2. **Conservative skipping**: For predicates we can't easily analyze (OR, NOT, complex expressions), we conservatively don't skip blocks.

3. **Value comparison**: The `<` and `>` operators on `Value` types must be implemented for this to work correctly. Verify `impl PartialOrd for Value` exists.

4. **Memory overhead**: block_mins and block_maxes add ~2 * num_blocks * size_of(Value) bytes per column. For 1M rows with BLOCK_SIZE=1024, that's ~2000 entries per column - minimal overhead.
