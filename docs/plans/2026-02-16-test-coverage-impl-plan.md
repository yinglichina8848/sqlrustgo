# Test Coverage Improvement Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Improve test coverage from 76.64% to 90% line coverage and 95% function coverage (excluding main.rs).

**Architecture:** Mixed approach - Network module tested separately (P0), Executor + B+Tree in parallel (P1).

**Tech Stack:** Rust, cargo test, cargo llvm-cov

---

## Phase 1: Network Module Tests (P0)

**Target:** 54.61% → 85%+ line coverage

### Task 1: Add Network Protocol Tests

**File:** `src/network/mod.rs`

**Step 1: Add test for HandshakeV10::to_bytes() with PLUGIN_AUTH**

```rust
#[test]
fn test_handshake_with_plugin_auth() {
    use capability::*;
    let mut handshake = HandshakeV10::new();
    handshake.capability |= PLUGIN_AUTH;
    handshake.auth_plugin_data = b"abcdefghijklmnopqrst".to_vec();

    let bytes = handshake.to_bytes();
    assert!(bytes.len() > 50);
}
```

**Step 2: Run test**
```bash
cargo test test_handshake_with_plugin_auth --all-features
```

**Step 3: Add test for OkPacket::to_bytes() with empty message**

```rust
#[test]
fn test_ok_packet_empty_message() {
    let packet = OkPacket::new(0, 0, None);
    let bytes = packet.to_bytes();
    assert!(!bytes.is_empty());
}
```

**Step 4: Run test**
```bash
cargo test test_ok_packet_empty_message --all-features
```

**Step 5: Add test for RowData::to_bytes() with Boolean and Blob**

```rust
#[test]
fn test_row_data_boolean_serialization() {
    let values = vec![Value::Boolean(true)];
    let row = RowData { values };
    let bytes = row.to_bytes();
    assert!(!bytes.is_empty());
}

#[test]
fn test_row_data_blob_serialization() {
    let values = vec![Value::Blob(b"binary data".to_vec())];
    let row = RowData { values };
    let bytes = row.to_bytes();
    assert!(!bytes.is_empty());
}
```

**Step 6: Run tests**
```bash
cargo test test_row_data_boolean --all-features
cargo test test_row_data_blob --all-features
```

**Step 7: Commit**
```bash
git add src/network/mod.rs
git commit -m "test: add network protocol tests for handshake, ok packet, row data"
```

### Task 2: Add NetworkHandler Tests

**File:** `src/network/mod.rs`

**Step 1: Add test for MySqlCommand serialization**

```rust
#[test]
fn test_mysql_command_all_variants() {
    for code in 0..0x20 {
        let _cmd = MySqlCommand::from(code);
    }
}
```

**Step 2: Run test**
```bash
cargo test test_mysql_command_all_variants --all-features
```

**Step 3: Add test for packet reading error handling**

```rust
#[test]
fn test_packet_incomplete_header() {
    let data = vec![0x01, 0x00]; // Only 2 bytes
    let result = parse_packet(&data);
    assert!(result.is_err());
}
```

**Step 4: Run test**
```bash
cargo test test_packet_incomplete_header --all-features
```

**Step 5: Commit**
```bash
git add src/network/mod.rs
git commit -m "test: add network handler tests for command variants and error handling"
```

### Task 3: Run Coverage Check for Network

**Step 1: Check coverage**
```bash
cargo llvm-cov --all-features -- src/network/mod.rs
```

**Step 2: If below 85%, identify gaps and add more tests**

---

## Phase 2: Executor Module Tests (P1)

**Target:** 77.31% → 85%+ line coverage

### Task 4: Add Executor Index Tests

**File:** `src/executor/mod.rs`

**Step 1: Add test for create_index**

```rust
#[test]
fn test_executor_create_index() {
    use crate::storage::file_storage::FileStorage;
    use std::path::PathBuf;
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();
    let mut storage = FileStorage::new(PathBuf::from(temp_dir.path())).unwrap();

    // Create table
    let table_data = TableData {
        info: TableInfo {
            name: "test".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            }],
        },
        rows: vec![vec![Value::Integer(1)]],
    };
    storage.insert_table("test".to_string(), table_data).unwrap();

    // Create index
    let engine = ExecutionEngine::new(storage);
    let result = engine.create_index("test", "id", 0);
    assert!(result.is_ok());
}
```

**Step 2: Run test**
```bash
cargo test test_executor_create_index --all-features
```

**Step 3: Add test for has_index**

```rust
#[test]
fn test_executor_has_index() {
    use crate::storage::file_storage::FileStorage;
    use std::path::PathBuf;
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();
    let mut storage = FileStorage::new(PathBuf::from(temp_dir.path())).unwrap();

    let table_data = TableData {
        info: TableInfo {
            name: "test".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            }],
        },
        rows: vec![],
    };
    storage.insert_table("test".to_string(), table_data).unwrap();
    storage.create_index("test", "id", 0).unwrap();

    let engine = ExecutionEngine::new(storage);
    let result = engine.has_index("test", "id");
    assert!(result);
}
```

**Step 4: Run test**
```bash
cargo test test_executor_has_index --all-features
```

**Step 5: Commit**
```bash
git add src/executor/mod.rs
git commit -m "test: add executor index tests"
```

### Task 5: Add WHERE Clause Operator Tests

**File:** `src/executor/mod.rs`

**Step 1: Add test for all comparison operators**

```rust
#[test]
fn test_execute_select_where_operators() {
    use crate::storage::file_storage::FileStorage;
    use std::path::PathBuf;
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();
    let mut storage = FileStorage::new(PathBuf::from(temp_dir.path())).unwrap();

    let table_data = TableData {
        info: TableInfo {
            name: "test".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            }],
        },
        rows: vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
        ],
    };
    storage.insert_table("test".to_string(), table_data).unwrap();

    let engine = ExecutionEngine::new(storage);

    // Test !=
    let result = engine.execute("SELECT * FROM test WHERE id != 1");
    assert!(result.is_ok());

    // Test >
    let result = engine.execute("SELECT * FROM test WHERE id > 1");
    assert!(result.is_ok());

    // Test <
    let result = engine.execute("SELECT * FROM test WHERE id < 3");
    assert!(result.is_ok());

    // Test >=
    let result = engine.execute("SELECT * FROM test WHERE id >= 2");
    assert!(result.is_ok());

    // Test <=
    let result = engine.execute("SELECT * FROM test WHERE id <= 2");
    assert!(result.is_ok());
}
```

**Step 2: Run test**
```bash
cargo test test_execute_select_where_operators --all-features
```

**Step 3: Add test for UPDATE without WHERE**

```rust
#[test]
fn test_execute_update_no_where() {
    use crate::storage::file_storage::FileStorage;
    use std::path::PathBuf;
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();
    let mut storage = FileStorage::new(PathBuf::from(temp_dir.path())).unwrap();

    let table_data = TableData {
        info: TableInfo {
            name: "test".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            }],
        },
        rows: vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
        ],
    };
    storage.insert_table("test".to_string(), table_data).unwrap();

    let engine = ExecutionEngine::new(storage);
    let result = engine.execute("UPDATE test SET id = 10");
    assert!(result.is_ok());
}
```

**Step 4: Run test**
```bash
cargo test test_execute_update_no_where --all-features
```

**Step 5: Add test for DELETE without WHERE**

```rust
#[test]
fn test_execute_delete_no_where() {
    use crate::storage::file_storage::FileStorage;
    use std::path::PathBuf;
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();
    let mut storage = FileStorage::new(PathBuf::from(temp_dir.path())).unwrap();

    let table_data = TableData {
        info: TableInfo {
            name: "test".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            }],
        },
        rows: vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
        ],
    };
    storage.insert_table("test".to_string(), table_data).unwrap();

    let engine = ExecutionEngine::new(storage);
    let result = engine.execute("DELETE FROM test");
    assert!(result.is_ok());
}
```

**Step 6: Run test**
```bash
cargo test test_execute_delete_no_where --all-features
```

**Step 7: Commit**
```bash
git add src/executor/mod.rs
git commit -m "test: add WHERE clause operator tests and UPDATE/DELETE without WHERE"
```

---

## Phase 3: B+ Tree Module Tests (P1)

**Target:** 79.43% → 85%+ line coverage

### Task 6: Add B+ Tree Split Tests

**File:** `src/storage/bplus_tree/tree.rs`

**Step 1: Add test for leaf node split**

```rust
#[test]
fn test_bplus_tree_leaf_split() {
    let mut tree = BPlusTree::new(4); // Small order to trigger split

    // Insert enough to cause split
    for i in 0..10 {
        tree.insert(i, i as u32);
    }

    assert_eq!(tree.len(), 10);
    // Verify all keys are searchable
    for i in 0..10 {
        assert_eq!(tree.search(i), Some(i as u32));
    }
}
```

**Step 2: Run test**
```bash
cargo test test_bplus_tree_leaf_split --all-features
```

**Step 3: Add test for internal node operations**

```rust
#[test]
fn test_bplus_tree_internal_node_traversal() {
    let mut tree = BPlusTree::new(2); // Very small order

    // Insert many to create internal nodes
    for i in 0..20 {
        tree.insert(i, i as u32);
    }

    // Test range query across internal nodes
    let results = tree.range_query(5, 15);
    assert!(!results.is_empty());
}
```

**Step 4: Run test**
```bash
cargo test test_bplus_tree_internal_node_traversal --all-features
```

**Step 5: Commit**
```bash
git add src/storage/bplus_tree/tree.rs
git commit -m "test: add B+ tree split and internal node tests"
```

---

## Phase 4: Verification

### Task 7: Final Coverage Check

**Step 1: Run full coverage**
```bash
cargo llvm-cov --all-features --exclude-files=main.rs
```

**Step 2: Verify targets met:**
- Line coverage ≥ 90%
- Function coverage ≥ 95%

**Step 3: Run all tests**
```bash
cargo test --all-features
```

**Step 4: Final commit**
```bash
git add .
git commit -m "test: improve coverage to 90% line, 95% function"
```

---

## Notes

- Each test should be isolated and not depend on external state
- Use tempfile crate for tests that need file storage
- Run `cargo llvm-cov` frequently to track progress
- If a test is hard to write, skip and focus on easier gaps first
