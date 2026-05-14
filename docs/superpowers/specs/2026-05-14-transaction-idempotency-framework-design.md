# Transaction Idempotency Framework - Design Specification

**Date**: 2026-05-14
**Issue**: #796
**Status**: Draft
**Author**: SQLRustGo Team

---

## 1. Overview

### 1.1 Problem Statement

Mobile/GMP (Good Manufacturing Practice) systems require **business event deduplication** across:

| Scenario | Problem |
|----------|---------|
| Mobile offline retry | Duplicate submission |
| App timeout retry | Duplicate insert |
| Gateway retry | Double write |
| MQ atleast-once | Duplicate consumption |
| User double-click | Duplicate approval |
| Leader failover | Replay |

### 1.2 Solution

Implement a **Transaction-level Idempotency Framework** that guarantees:

1. `same request` → `must apply once`
2. `same key + diff payload` → `must fail`
3. `committed txn` → `must survive crash recovery`
4. `retry must return same result`

---

## 2. Core Invariants

| # | Invariant | Description |
|---|-----------|-------------|
| 1 | **Once-per-request** | A given request UUID must only be applied once |
| 2 | **Payload integrity** | Same idempotency key with different payload must be rejected |
| 3 | **Durable commitment** | Committed transactions must survive crash recovery |
| 4 | **Result consistency** | Retry must return the same response as original execution |

---

## 3. Architecture

### 3.1 System Components

```
Client Request
    ↓
BEGIN IDEMPOTENT 'request-uuid'
    ↓
Idempotency Registry
    ↓
Transaction Manager
    ↓
WAL (protected)
    ↓
Recovery (rebuild state)
    ↓
Response Cached
```

### 3.2 New Module Location

```
crates/transaction/src/idempotency/
├── mod.rs
├── registry.rs      -- idempotency_registry table
├── request_hash.rs  -- request digest verification
├── replay.rs        -- replay handling
├── dedupe.rs        -- duplicate detection
└── gc.rs           -- expired cleanup
```

### 3.3 New System Table

```sql
CREATE TABLE system.idempotency_registry (
    idempotency_key VARCHAR(128) PRIMARY KEY,
    request_hash BLOB NOT NULL,
    transaction_id BIGINT,
    status ENUM('PROCESSING', 'COMMITTED', 'ROLLED_BACK') NOT NULL,
    response_digest BLOB,
    created_at TIMESTAMP NOT NULL,
    committed_at TIMESTAMP
);
```

---

## 4. SQL Syntax

### 4.1 Recommended Syntax

```sql
BEGIN IDEMPOTENT 'request-uuid';
    INSERT INTO orders (...) VALUES (...);
    INSERT INTO audit_log (...) VALUES (...);
    INSERT INTO signature (...) VALUES (...);
COMMIT;
```

### 4.2 Alternative (Session-level)

```sql
SET SESSION idempotency_key = 'request-uuid';
INSERT INTO orders (...) VALUES (...);
-- All statements in session use same idempotency key
```

### 4.3 Parser Changes

- New token: `IDEMPOTENT`
- New token: `IDEMPOTENCY`
- New AST node: `BeginIdempotentStatement(key: String)`
- Extend `TransactionStatement` to support idempotent variant

---

## 5. Execution Flow

### 5.1 BEGIN IDEMPOTENT Processing

```
Step 1: Client sends BEGIN IDEMPOTENT 'xxx'

Step 2: Server checks idempotency_registry
    ├── NOT EXISTS → Insert PROCESSING, begin transaction
    ├── COMMITTED → Return cached response (idempotent)
    └── PROCESSING → Conflict error (request in-flight)
```

### 5.2 COMMIT Processing

```
Step 1: Client sends COMMIT

Step 2: Within COMMIT:
    a. Commit transaction to WAL
    b. Update idempotency_registry status = 'COMMITTED'
    c. Store response_digest
    d. Commit durable
```

### 5.3 ROLLBACK Processing

```
Step 1: Client sends ROLLBACK

Step 2: Within ROLLBACK:
    a. Update idempotency_registry status = 'ROLLED_BACK'
    b. Transaction continues rollback normally
```

---

## 6. Security Critical Rules

### 6.1 Request Hash Verification

**MUST** store `request_hash` (SHA-256 of full request payload) to prevent:

| Scenario | Behavior |
|----------|----------|
| Same key + Same hash | Allow (legitimate retry) |
| Same key + Different hash | **REJECT** (potential attack) |

### 6.2 Request Hash Calculation

```rust
fn calculate_request_hash(stmt: &Statement) -> Vec<u8> {
    let serialized = serialize_statement(stmt);
    sha256(&serialized)
}
```

---

## 7. Recovery Integration

### 7.1 WAL Protection

Idempotency registry **MUST** be WAL-protected:

```text
Transaction commits
    ↓
WAL written (includes registry update)
    ↓
Registry durable
    ↓
Commit confirmed
```

### 7.2 Recovery Protocol

```
Recovery starts
    ↓
Rebuild idempotency_registry from WAL
    ↓
For each COMMITTED entry:
    └── Restore response cache
    ↓
System ready
```

### 7.3 Crash Scenarios

| Scenario | Handling |
|----------|----------|
| Commit before registry | Registry recovered from WAL |
| Registry before commit | Transaction rolled back in WAL replay |
| Crash between phases | Both protected by WAL atomicity |

---

## 8. Concurrency Handling

### 8.1 Race Condition: Concurrent Same Key

```
Thread 1: BEGIN IDEMPOTENT 'xxx' → PROCESSING
Thread 2: BEGIN IDEMPOTENT 'xxx' → CONFLICT (in-flight)

Resolution:
- Thread 2 gets: "Request in progress, retry later"
- Client should: exponential backoff + retry
```

### 8.2 Concurrent Limit

For 100 concurrent requests with same key:
- Only **1 succeeds**
- 99 get CONFLICT error
- Client retry logic handles failures

---

## 9. Response Caching

### 9.1 Cache Structure

```rust
struct IdempotentResponse {
    digest: Vec<u8>,
    payload: Vec<u8>,
    timestamp: i64,
}
```

### 9.2 Cache Behavior

- **On COMMITTED**: Store response with digest
- **On retry with COMMITTED key**: Return cached response
- **Cache size limit**: Configurable (default 10MB per key)
- **Cache TTL**: Configurable (default 7 days)

---

## 10. Garbage Collection

### 10.1 Cleanup Policy

- **PROCESSING entries older than 1 hour**: Auto-rollback + delete
- **COMMITTED entries older than 7 days**: Delete (if not referenced)
- **ROLLED_BACK entries older than 1 day**: Delete

### 10.2 GC Trigger

- Background task runs every hour
- Also triggered on startup for stale PROCESSING entries

---

## 11. Implementation Phases

### Phase 1: Core Infrastructure
- [ ] Add `IDEMPOTENT`/`IDEMPOTENCY` tokens
- [ ] Parse `BEGIN IDEMPOTENT 'key'`
- [ ] Create `system.idempotency_registry` table
- [ ] Basic registry CRUD

### Phase 2: Transaction Integration
- [ ] Integrate with transaction manager
- [ ] Add request hash calculation
- [ ] Implement duplicate detection
- [ ] Add response caching

### Phase 3: WAL Integration
- [ ] Protect registry with WAL
- [ ] Add registry entries to WAL
- [ ] Recovery rebuild protocol

### Phase 4: Error Handling
- [ ] CONFLICT error for in-flight requests
- [ ] Hash mismatch error for security
- [ ] Client retry guidance in error messages

### Phase 5: GC and Cleanup
- [ ] Implement garbage collection
- [ ] Add metrics/logging
- [ ] Admin views for monitoring

---

## 12. Testing Requirements

### 12.1 L1 Unit Tests

| Test | Expected Behavior |
|------|-------------------|
| duplicate_retry_same_hash | Same result returned |
| same_key_diff_payload | Rejected with error |
| rollback_retry | Retry allowed |
| processing_conflict | Conflict error returned |

### 12.2 L2 Crash Tests

| Test | Scenario |
|------|----------|
| commit_before_registry_crash | Recovery restores consistency |
| registry_before_wal_crash | WAL replay cleans up |
| crash_between_phases | WAL atomicity ensures consistency |

### 12.3 L3 Concurrency Tests

| Test | Expected Behavior |
|------|-------------------|
| 100_concurrent_same_key | Only 1 succeeds, 99 get conflict |

### 12.4 L4 Recovery Tests

| Test | Expected Behavior |
|------|-------------------|
| recovery_after_commit | Retry returns same response |
| recovery_after_rollback | Retry succeeds (new transaction) |

---

## 13. Configuration

```toml
[transaction.idempotency]
enabled = true
registry_table = "system.idempotency_registry"
max_cache_size_mb = 10
cache_ttl_days = 7
processing_timeout_minutes = 60
gc_interval_minutes = 60
```

---

## 14. Related Issues

- Issue #626: WAL + Audit Chain Integration
- Issue #630: SSI Deadlock Detection

---

## 15. Future Enhancements

- Cluster-wide idempotency (distributed)
- Read-your-writes consistency for idempotent reads
- Batch idempotency keys
- Idempotency key expiration extension

---

## 16. References

- Stripe idempotency: https://stripe.com/blog/idempotency
- AWS SQS MessageDeduplicationId
- Kafka exactly-once semantics

---

*Document Version: 1.0*
*Created: 2026-05-14*
