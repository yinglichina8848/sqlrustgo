# v2.9.0 RC Phase 2 Coverage Improvement Plan

**Date**: 2026-05-05
**Version**: v2.9.0 RC Phase 2
**Target Modules**: mysql-server, tools, expr

---

## Current Status

### Critical Low Coverage Modules

| Module | Current | Target | Gap | Files |
|--------|---------|--------|-----|-------|
| **mysql-server** | 35.67% | 60% | +24.33% | lib.rs (36.19%) |
| **tools** | 58.03% | 75% | +16.97% | upgrade.rs (19.56%), backup_restore.rs (54.76%) |
| **expr** | 60.31% | 80% | +19.69% | expr.rs (39.73%), op.rs (0.00%) |

---

## Improvement Tasks

### Task 1: expr/op.rs (0% → 50%)

**Owner**: AI Agent
**Estimated Time**: 2-4 hours
**Files**: `crates/expr/src/op.rs`

#### Root Cause
- `op.rs` contains operator functions that are never called in tests
- No unit tests for arithmetic, comparison, or logical operators

#### Implementation Plan

```rust
// Add to crates/expr/src/op.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_operators() {
        // Test add, sub, mul, div, mod
        assert_eq!(op_add(Value::Integer(1), Value::Integer(2)), Value::Integer(3));
        assert_eq!(op_sub(Value::Integer(5), Value::Integer(3)), Value::Integer(2));
        // ...
    }

    #[test]
    fn test_comparison_operators() {
        // Test eq, ne, lt, le, gt, ge
        assert_eq!(op_eq(Value::Integer(1), Value::Integer(1)), Value::Boolean(true));
        // ...
    }

    #[test]
    fn test_logical_operators() {
        // Test and, or, not
        assert_eq!(op_and(Value::Boolean(true), Value::Boolean(false)), Value::Boolean(false));
        // ...
    }

    #[test]
    fn test_null_handling() {
        // Test NULL propagation in operators
        assert_eq!(op_add(Value::Null, Value::Integer(1)), Value::Null);
        // ...
    }
}
```

---

### Task 2: tools/backup_restore.rs (54% → 75%)

**Owner**: AI Agent
**Estimated Time**: 4-8 hours
**Files**: `crates/tools/src/backup_restore.rs`

#### Root Cause
- `backup_restore.rs` has complex file I/O that is not mocked
- Integration tests require actual file system access

#### Implementation Plan

1. Add mock storage interface for testing
2. Add tests for backup operations:
   - Full backup
   - Incremental backup
   - Backup with compression
3. Add tests for restore operations:
   - Full restore
   - Point-in-time restore
4. Add error handling tests

---

### Task 3: executor/stored_proc.rs (48% → 65%)

**Owner**: AI Agent
**Estimated Time**: 8-12 hours
**Files**: `crates/executor/src/stored_proc.rs`

#### Root Cause
- 3193 lines with complex control flow
- Cursor operations, exception handlers, nested blocks not fully tested
- `patch_stored_proc_coverage.rs` covers basic paths but not edge cases

#### Implementation Plan

```rust
// Add to crates/executor/tests/patch_stored_proc_coverage.rs

#[test]
fn test_cursor_edge_cases() {
    // Test cursor not found
    // Test cursor already open
    // Test fetch after close
    // Test multiple cursors
}

#[test]
fn test_exception_handlers() {
    // Test SQLSTATE matching
    // Test NOT FOUND handler
    // Test WARNING handler
    // Test custom condition handler
    // Test nested handlers
}

#[test]
fn test_nested_blocks() {
    // Test deep nesting
    // Test scope isolation
    // Test label propagation
}

#[test]
fn test_cte_subqueries() {
    // Test simple CTE
    // Test recursive CTE
    // Test multiple CTEs
}
```

---

### Task 4: mysql-server/lib.rs (35% → 60%)

**Owner**: AI Agent
**Estimated Time**: 12-16 hours
**Files**: `crates/mysql-server/src/lib.rs`

#### Root Cause
- MySQL protocol implementation is tightly coupled with socket I/O
- Hard to unit test without mocking the entire server stack
- 1238 lines of protocol handling code

#### Implementation Plan

1. **Phase 1**: Add mock socket tests for packet parsing
2. **Phase 2**: Add unit tests for individual packet handlers
3. **Phase 3**: Extract protocol logic for better testability

```rust
// Add to crates/mysql-server/tests/

#[test]
fn test_packet_parser() {
    // Test COM_QUIT packet
    // Test COM_INIT_DB packet
    // Test COM_QUERY packet
    // Test COM_STMT_PREPARE packet
    // Test COM_STMT_EXECUTE packet
}

#[test]
fn test_handshake_sequence() {
    // Test successful handshake
    // Test authentication failure
    // Test wrong password
}
```

---

### Task 5: tools/upgrade.rs (19% → 50%)

**Owner**: AI Agent
**Estimated Time**: 6-10 hours
**Files**: `crates/tools/src/upgrade.rs`

#### Root Cause
- Version migration code is complex
- Requires actual version comparison logic
- Schema evolution not well covered

#### Implementation Plan

1. Add mock version detector
2. Add tests for version comparison
3. Add tests for migration paths
4. Add tests for rollback scenarios

---

## PR Plan

| PR | Task | Target | ETA |
|----|------|--------|-----|
| #303 | expr/op.rs tests | +15% | Day 1 |
| #304 | tools/backup_restore tests | +10% | Day 2-3 |
| #305 | executor/stored_proc tests | +10% | Day 3-4 |
| #306 | mysql-server packet tests | +15% | Day 5-7 |
| #307 | tools/upgrade tests | +10% | Day 7-8 |

---

## Success Criteria

| Module | Before | After | Target |
|--------|--------|-------|--------|
| mysql-server | 35.67% | 50% | 60% |
| tools | 58.03% | 70% | 75% |
| expr | 60.31% | 75% | 80% |
| executor | 70.54% | 75% | 80% |

**Total Improvement**: +12-15% overall coverage

---

## Execution

```bash
# Run tests for specific module
cargo test -p sqlrustgo-expr --lib

# Generate coverage
cargo llvm-cov --package sqlrustgo-expr

# Check coverage report
open target/llvm-cov/html/sqlrustgo-expr/index.html
```

---

**Plan Created**: 2026-05-05
**Target Completion**: 2026-05-12
