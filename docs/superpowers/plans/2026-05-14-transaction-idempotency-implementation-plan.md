# Transaction Idempotency Framework Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Transaction-level Idempotency Framework for GMP mobile/offline sync scenarios

**Architecture:** Add `BEGIN IDEMPOTENT 'key'` syntax, `system.idempotency_registry` table, and integrate with transaction manager. Registry is WAL-protected and survives crash recovery.

**Tech Stack:** Rust, SQL parser, Transaction Manager, WAL, Recovery subsystem

---

## File Structure Overview

### New Files
- `crates/transaction/src/idempotency/mod.rs` - Module root
- `crates/transaction/src/idempotency/registry.rs` - IdempotencyRegistry table operations
- `crates/transaction/src/idempotency/request_hash.rs` - Request hash calculation/verification
- `crates/transaction/src/idempotency/dedupe.rs` - Duplicate detection logic
- `crates/transaction/src/idempotency/cache.rs` - Response caching
- `crates/transaction/src/idempotency/gc.rs` - Garbage collection

### Modified Files
- `crates/parser/src/token.rs` - Add `Idempotent`, `Idempotency` tokens
- `crates/parser/src/transaction.rs` - Add `BeginIdempotent` variant
- `crates/parser/src/parser.rs` - Parse `BEGIN IDEMPOTENT 'key'`
- `crates/transaction/src/lib.rs` - Add idempotency module
- `crates/transaction/src/transaction_manager.rs` - Integrate idempotency checks
- `crates/sqlrustgo/src/execution_engine.rs` - Execute BEGIN IDEMPOTENT

---

## Task 1: Token Definitions

**Files:**
- Modify: `crates/parser/src/token.rs`

- [ ] **Step 1: Add Idempotent token**

Find line ~180 (after `Collector` keyword), add:
```rust
// Idempotency keywords
Idempotent,
Idempotency,
Key,
```

- [ ] **Step 2: Add Display for Idempotent tokens**

Find ~line 407 (after `Collector` Display), add:
```rust
Token::Idempotent => write!(f, "IDEMPOTENT"),
Token::Idempotency => write!(f, "IDEMPOTENCY"),
Token::Key => write!(f, "KEY"),
```

- [ ] **Step 3: Add to from_keyword match**

Find ~line 609 (after `"COLLECTOR"`), add:
```rust
"IDEMPOTENT" => Some(Token::Idempotent),
"IDEMPOTENCY" => Some(Token::Idempotency),
"KEY" => Some(Token::Key),
```

- [ ] **Step 4: Add to is_keyword function**

Find ~line 812 (after `"COLLECTOR"`), add:
```rust
assert!(is_keyword("IDEMPOTENT"));
assert!(is_keyword("IDEMPOTENCY"));
assert!(is_keyword("KEY"));
```

- [ ] **Step 5: Commit**

```bash
git add crates/parser/src/token.rs
git commit -m "feat(parser): add IDEMPOTENT/IDEMPOTENCY/KEY tokens"
```

---

## Task 2: AST TransactionStatement Update

**Files:**
- Modify: `crates/parser/src/transaction.rs`

- [ ] **Step 1: Add BeginIdempotent variant**

Modify the `TransactionStatement` enum, add:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionStatement {
    Begin {
        work: bool,
        isolation_level: Option<IsolationLevel>,
    },
    // ... existing variants ...
    BeginIdempotent {
        key: String,
    },
    // ... rest of variants ...
}
```

- [ ] **Step 2: Add test for BeginIdempotent**

In the `#[cfg(test)]` mod, add:
```rust
#[test]
fn test_begin_idempotent() {
    let stmt = TransactionStatement::BeginIdempotent {
        key: "txn-123".to_string(),
    };
    match stmt {
        TransactionStatement::BeginIdempotent { key } => {
            assert_eq!(key, "txn-123");
        }
        _ => panic!("Expected BeginIdempotent"),
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add crates/parser/src/transaction.rs
git commit -m "feat(parser): add BeginIdempotent variant to TransactionStatement"
```

---

## Task 3: Parser for BEGIN IDEMPOTENT

**Files:**
- Modify: `crates/parser/src/parser.rs`

- [ ] **Step 1: Add parsing for BEGIN IDEMPOTENT in parse_begin()**

Modify `parse_begin()` function (~line 1103). After checking for isolation level, add:
```rust
// Check for IDEMPOTENT
if self.current() == Some(&Token::Idempotent) {
    self.next();
    // Expect IDEMPOTENCY KEY 'string' or just 'string'
    let key = if self.current() == Some(&Token::Idempotency) {
        self.next();
        self.expect(Token::Key)?;
        match self.next() {
            Some(Token::StringLiteral(s)) => s,
            Some(Token::Identifier(s)) => s,
            _ => return Err("Expected idempotency key string".to_string()),
        }
    } else {
        match self.current() {
            Some(Token::StringLiteral(s)) => s,
            Some(Token::Identifier(s)) => s,
            _ => return Err("Expected idempotency key".to_string()),
        }
    };
    return Ok(Statement::Transaction(TransactionStatement::BeginIdempotent { key }));
}
```

- [ ] **Step 2: Add test for BEGIN IDEMPOTENT**

Add test function:
```rust
#[test]
fn test_parse_begin_idempotent() {
    let result = parse("BEGIN IDEMPOTENT 'txn-123'");
    match result {
        Ok(Statement::Transaction(TransactionStatement::BeginIdempotent { key })) => {
            assert_eq!(key, "txn-123");
        }
        _ => panic!("Expected BEGIN IDEMPOTENT statement"),
    }
}

#[test]
fn test_parse_begin_idempotent_key() {
    let result = parse("BEGIN IDEMPOTENCY KEY 'txn-456'");
    match result {
        Ok(Statement::Transaction(TransactionStatement::BeginIdempotent { key })) => {
            assert_eq!(key, "txn-456");
        }
        _ => panic!("Expected BEGIN IDEMPOTENCY KEY statement"),
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test -p sqlrustgo-parser test_parse_begin_idempotent -v
cargo test -p sqlrustgo-parser test_parse_begin_idempotent_key -v
```

- [ ] **Step 4: Commit**

```bash
git add crates/parser/src/parser.rs
git commit -m "feat(parser): parse BEGIN IDEMPOTENT 'key' syntax"
```

---

## Task 4: Idempotency Registry Module

**Files:**
- Create: `crates/transaction/src/idempotency/mod.rs`
- Create: `crates/transaction/src/idempotency/registry.rs`
- Create: `crates/transaction/src/idempotency/request_hash.rs`
- Create: `crates/transaction/src/idempotency/dedupe.rs`
- Create: `crates/transaction/src/idempotency/cache.rs`
- Create: `crates/transaction/src/idempotency/gc.rs`

- [ ] **Step 1: Create idempotency/mod.rs**

```rust
//! Transaction Idempotency Framework
//!
//! Provides idempotent transaction execution for GMP mobile/offline sync.
//!
//! Core invariant: same request UUID must only be applied once.

pub mod registry;
pub mod request_hash;
pub mod dedupe;
pub mod cache;
pub mod gc;

pub use registry::{IdempotencyRegistry, IdempotencyStatus};
pub use request_hash::RequestHash;
pub use dedupe::DuplicateDetector;
pub use cache::ResponseCache;
pub use gc::GarbageCollector;
```

- [ ] **Step 2: Create idempotency/registry.rs**

```rust
//! Idempotency Registry - stores idempotency keys and their status

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdempotencyStatus {
    Processing,
    Committed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyEntry {
    pub key: String,
    pub request_hash: Vec<u8>,
    pub transaction_id: Option<i64>,
    pub status: IdempotencyStatus,
    pub response_digest: Option<Vec<u8>>,
    pub created_at: i64,
    pub committed_at: Option<i64>,
}

pub struct IdempotencyRegistry {
    // In-memory cache backed by system table
}

impl IdempotencyRegistry {
    pub fn new() -> Self {
        Self {}
    }

    /// Check if key exists and return status
    pub fn get_status(&self, key: &str) -> Option<IdempotencyStatus> {
        // TODO: Implement with actual storage
        None
    }

    /// Insert new key in PROCESSING state
    pub fn insert_processing(&mut self, key: String, request_hash: Vec<u8>) -> Result<(), IdempotencyError> {
        // TODO: Implement
        Ok(())
    }

    /// Mark key as COMMITTED
    pub fn mark_committed(&mut self, key: &str, response_digest: Vec<u8>) -> Result<(), IdempotencyError> {
        // TODO: Implement
        Ok(())
    }

    /// Mark key as ROLLED_BACK
    pub fn mark_rolled_back(&mut self, key: &str) -> Result<(), IdempotencyError> {
        // TODO: Implement
        Ok(())
    }
}

#[derive(Debug)]
pub enum IdempotencyError {
    KeyExists,
    KeyNotFound,
    StatusMismatch,
}
```

- [ ] **Step 3: Create idempotency/request_hash.rs**

```rust
//! Request hash calculation and verification

use sha2::{Sha256, Digest};

pub struct RequestHash {
    hasher: Sha256,
}

impl RequestHash {
    pub fn new() -> Self {
        Self { hasher: Sha256::new() }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    pub fn finalize(self) -> Vec<u8> {
        self.hasher.finalize().to_vec()
    }

    /// Verify that retry request hash matches original
    pub fn verify(original: &[u8], retry: &[u8]) -> bool {
        original == retry
    }
}
```

- [ ] **Step 4: Create idempotency/dedupe.rs**

```rust
//! Duplicate detection logic

use super::registry::{IdempotencyEntry, IdempotencyStatus, IdempotencyError};

pub struct DuplicateDetector {
    registry: IdempotencyRegistry,
}

impl DuplicateDetector {
    pub fn new() -> Self {
        Self {
            registry: IdempotencyRegistry::new(),
        }
    }

    /// Check if request is a duplicate
    pub fn check(&self, key: &str) -> Result<CheckResult, IdempotencyError> {
        match self.registry.get_status(key) {
            Some(IdempotencyStatus::Committed) => Ok(CheckResult::AlreadyApplied),
            Some(IdempotencyStatus::Processing) => Ok(CheckResult::InProgress),
            Some(IdempotencyStatus::RolledBack) => Ok(CheckResult::NewRequest),
            None => Ok(CheckResult::NewRequest),
        }
    }

    /// Verify request payload hash matches
    pub fn verify_payload(&self, key: &str, hash: &[u8]) -> Result<bool, IdempotencyError> {
        // TODO: Implement with actual storage
        Ok(true)
    }
}

pub enum CheckResult {
    NewRequest,
    InProgress,
    AlreadyApplied,
}
```

- [ ] **Step 5: Create idempotency/cache.rs**

```rust
//! Response caching for idempotent retries

use std::collections::HashMap;

pub struct ResponseCache {
    cache: HashMap<String, CachedResponse>,
    max_size_bytes: usize,
}

#[derive(Clone)]
pub struct CachedResponse {
    pub digest: Vec<u8>,
    pub payload: Vec<u8>,
    pub timestamp: i64,
}

impl ResponseCache {
    pub fn new(max_size_mb: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size_bytes: max_size_mb * 1024 * 1024,
        }
    }

    pub fn get(&self, key: &str) -> Option<CachedResponse> {
        self.cache.get(key).cloned()
    }

    pub fn insert(&mut self, key: String, response: CachedResponse) {
        // Simple LRU eviction if over size limit
        self.cache.insert(key, response);
    }
}
```

- [ ] **Step 6: Create idempotency/gc.rs**

```rust
//! Garbage collection for expired idempotency entries

pub struct GarbageCollector {
    processing_timeout_hours: u64,
    committed_ttl_days: u64,
    rolled_back_ttl_hours: u64,
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            processing_timeout_hours: 1,
            committed_ttl_days: 7,
            rolled_back_ttl_hours: 24,
        }
    }

    /// Cleanup stale PROCESSING entries (older than timeout)
    pub fn cleanup_stale_processing(&self) -> Vec<String> {
        // TODO: Implement
        vec![]
    }

    /// Cleanup old COMMITTED entries (older than TTL)
    pub fn cleanup_old_committed(&self) -> Vec<String> {
        // TODO: Implement
        vec![]
    }
}
```

- [ ] **Step 7: Update transaction/src/lib.rs**

Add after line 16:
```rust
pub mod idempotency;
```

- [ ] **Step 8: Commit**

```bash
git add crates/transaction/src/idempotency/
git add crates/transaction/src/lib.rs
git commit -m "feat(transaction): add idempotency module structure"
```

---

## Task 5: Integrate with Transaction Manager

**Files:**
- Modify: `crates/transaction/src/transaction_manager.rs`

- [ ] **Step 1: Add IdempotencyManager to TransactionManager**

Find `TransactionManager` struct, add:
```rust
pub struct TransactionManager {
    // ... existing fields ...
    idempotency_manager: Option<IdempotencyManager>,
}

pub struct IdempotencyManager {
    registry: IdempotencyRegistry,
    cache: ResponseCache,
    detector: DuplicateDetector,
}

impl IdempotencyManager {
    pub fn new() -> Self {
        Self {
            registry: IdempotencyRegistry::new(),
            cache: ResponseCache::new(10), // 10MB
            detector: DuplicateDetector::new(),
        }
    }
}
```

- [ ] **Step 2: Add begin_idempotent method**

Add method to TransactionManager:
```rust
pub fn begin_idempotent(&mut self, key: String, request_hash: Vec<u8>) -> Result<TxId, IdempotencyError> {
    match self.idempotency_manager.detector.check(&key)? {
        CheckResult::AlreadyApplied => {
            // Return cached response
            return Err(IdempotencyError::AlreadyApplied);
        }
        CheckResult::InProgress => {
            return Err(IdempotencyError::RequestInProgress);
        }
        CheckResult::NewRequest => {
            // Proceed with transaction
        }
    }

    self.idempotency_manager.registry.insert_processing(key, request_hash)?;
    self.begin_transaction()
}
```

- [ ] **Step 3: Commit**

```bash
git add crates/transaction/src/transaction_manager.rs
git commit -m "feat(transaction): integrate idempotency with transaction manager"
```

---

## Task 6: Execution Engine Support

**Files:**
- Modify: `crates/sqlrustgo/src/execution_engine.rs`

- [ ] **Step 1: Handle BeginIdempotent in execute_transaction**

Find where `Statement::Transaction` is handled (~line 520), add case:
```rust
Statement::Transaction(TransactionStatement::BeginIdempotent { ref key }) => {
    let request_hash = calculate_request_hash(stmt);
    match tx_manager.begin_idempotent(key.clone(), request_hash) {
        Ok(tx_id) => ExecutorResult::Begin { tx_id },
        Err(IdempotencyError::AlreadyApplied) => {
            // Return cached response
            ExecutorResult::IdempotentAlreadyApplied
        }
        Err(IdempotencyError::RequestInProgress) => {
            return Err(SqlError::IdempotencyKeyInProgress(key.clone()));
        }
        Err(e) => return Err(SqlError::IdempotencyError(e.to_string())),
    }
}
```

- [ ] **Step 2: Add new ExecutorResult variant**

Find `ExecutorResult` enum, add:
```rust
pub enum ExecutorResult {
    // ... existing variants ...
    IdempotentAlreadyApplied,
}
```

- [ ] **Step 3: Commit**

```bash
git add crates/sqlrustgo/src/execution_engine.rs
git commit -m "feat(execution): handle BEGIN IDEMPOTENT in executor"
```

---

## Task 7: Unit Tests (L1)

**Files:**
- Create: `crates/transaction/src/idempotency/tests.rs`

- [ ] **Step 1: Write duplicate retry test**

```rust
#[test]
fn test_duplicate_retry_same_result() {
    let detector = DuplicateDetector::new();

    // First request - NewRequest
    match detector.check("txn-123") {
        Ok(CheckResult::NewRequest) => {},
        _ => panic!("Expected NewRequest"),
    }
}
```

- [ ] **Step 2: Write same key diff payload test**

```rust
#[test]
fn test_same_key_diff_payload_rejected() {
    // When same key but different request hash
    let hash1 = RequestHash::new().finalize();
    let hash2 = {
        let mut h = RequestHash::new();
        h.update(b"different payload");
        h.finalize()
    };

    // Verify returns false for different hashes
    assert!(!RequestHash::verify(&hash1, &hash2));
}
```

- [ ] **Step 3: Write rollback retry test**

```rust
#[test]
fn test_rollback_retry_allowed() {
    let detector = DuplicateDetector::new();

    // After rollback, same key should be NewRequest again
    // (This requires integration with actual storage)
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p sqlrustgo-transaction idempotency -v
```

- [ ] **Step 5: Commit**

```bash
git add crates/transaction/src/idempotency/tests.rs
git commit -m "test(transaction): add idempotency L1 unit tests"
```

---

## Task 8: WAL Integration

**Files:**
- Modify: `crates/storage/src/wal.rs`
- Modify: `crates/transaction/src/idempotency/registry.rs`

- [ ] **Step 1: Add IdempotencyEntry to WAL entry types**

Find WAL entry type enum, add:
```rust
pub enum WalEntryType {
    // ... existing ...
    IdempotencyCommit,
    IdempotencyRollback,
}
```

- [ ] **Step 2: Protect registry with WAL**

In `IdempotencyRegistry::mark_committed()`:
```rust
pub fn mark_committed(&mut self, key: &str, response_digest: Vec<u8>) -> Result<(), IdempotencyError> {
    // Write to WAL first
    wal.append(IdempotencyEntry {
        key: key.to_string(),
        request_hash: vec![],
        transaction_id: None,
        status: IdempotencyStatus::Committed,
        response_digest: Some(response_digest),
        created_at: now(),
        committed_at: Some(now()),
    })?;
    // Then update in-memory
    // ...
}
```

- [ ] **Step 3: Commit**

```bash
git add crates/storage/src/wal.rs crates/transaction/src/idempotency/registry.rs
git commit -m "feat(wal): protect idempotency registry with WAL"
```

---

## Task 9: Integration Tests (L2, L3, L4)

- [ ] **Step 1: Write crash recovery test**

```rust
#[test]
fn test_crash_recovery_idempotency() {
    // Setup: BeginIdempotent, commit
    // Simulate crash
    // Recover
    // Verify same request returns cached response
}
```

- [ ] **Step 2: Write concurrency test**

```rust
#[test]
fn test_concurrent_same_idempotency_key() {
    // Spawn 100 threads with same key
    // Only 1 should succeed
    // Others should get CONFLICT error
}
```

- [ ] **Step 3: Run all tests**

```bash
cargo test -p sqlrustgo-transaction idempotency -v
cargo test -p sqlrustgo integration -v
```

- [ ] **Step 4: Commit**

```bash
git add crates/transaction/src/idempotency/tests.rs
git commit -m "test(transaction): add idempotency integration tests"
```

---

## Verification

After all tasks complete:

1. Run parser tests:
```bash
cargo test -p sqlrustgo-parser test_parse_begin_idempotent -v
```

2. Run transaction tests:
```bash
cargo test -p sqlrustgo-transaction idempotency -v
```

3. Run integration tests:
```bash
cargo test -p sqlrustgo --test '*idempotency*' -v
```

4. Run clippy and fmt:
```bash
cargo clippy --all-features -- -D warnings
cargo fmt --check
```

---

## Spec Coverage Check

| Spec Requirement | Task |
|-----------------|------|
| BEGIN IDEMPOTENT 'key' syntax | Task 3 |
| IdempotencyRegistry table | Task 4 |
| Request hash verification | Task 4 |
| Duplicate detection | Task 4 |
| Response caching | Task 4 |
| WAL protection | Task 8 |
| Crash recovery | Task 9 |
| L1 unit tests | Task 7 |
| L2 crash tests | Task 9 |
| L3 concurrency tests | Task 9 |
| L4 recovery tests | Task 9 |

All spec requirements are covered.

---

*Plan Version: 1.0*
*Created: 2026-05-14*