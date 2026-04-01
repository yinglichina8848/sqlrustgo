# Teaching Test Architecture Analysis & Plan

## Date: 2026-03-27
## Author: Claude Code
## Version: SQLRustGo 1.9 → 2.0 Transition

---

## 1. Current Problem

### 1.1 Teaching Scenario Tests Are NOT Teaching Tests

**Current Structure:**
```
teaching_scenario_test.rs
    ↓
ExecutionEngine.execute()
```

**Problem:** This is just a "core functionality unit test collection", NOT a "teaching-level system behavior verification".

### 1.2 What Real Teaching Should Look Like

```
Client
    ↓
Protocol Layer (HTTP / MySQL-like / CLI)
    ↓
Server Session
    ↓
ExecutionEngine
    ↓
Storage / Optimizer / Transaction
```

**Missing Components:**
- Session lifecycle
- Connection pool
- SQL parser server routing
- Protocol decode
- Error serialization
- Concurrent connection behavior
- Server transaction scope

---

## 2. Architecture Gap Analysis

### 2.1 TeachingHttpServer Limitations

| Feature | Status |
|---------|--------|
| `/health/live` | ✅ Implemented |
| `/health/ready` | ✅ Implemented |
| `/metrics` | ✅ Implemented |
| `/teaching/pipeline` | ✅ Visual only, no SQL execution |
| `/teaching/profile` | ✅ Visual only, no SQL execution |
| **POST /sql** | ❌ **MISSING - Critical Gap** |

### 2.2 Impact

Without `POST /sql` endpoint:
- Cannot test real client-server SQL execution
- Teaching tests remain "engine direct tests"
- Cannot verify real database system behavior
- Cannot support MySQL-compatible teaching scenarios

---

## 3. Recommended Test Structure

```
tests/
│
├── unit/
│   ├── optimizer_test.rs
│   ├── execution_engine_test.rs
│   └── storage_test.rs
│
├── integration/
│   ├── teaching_pipeline_test.rs
│   ├── teaching_transaction_test.rs
│   ├── teaching_join_test.rs
│   └── teaching_aggregate_test.rs
│
├── client_server/
│   ├── sql_execution_test.rs          ← NEW: POST /sql endpoint tests
│   ├── transaction_session_test.rs    ← NEW: Session lifecycle tests
│   └── concurrency_test.rs            ← NEW: Concurrent connection tests
│
└── protocol/
    └── mysql_compatibility_test.rs    ← NEW: Protocol layer tests
```

### 3.1 Test Type Responsibilities

| Type | Purpose | Required |
|------|---------|----------|
| Engine Direct Tests | Fast verification of correctness | ✅ |
| Client-Server Tests | Verify real teaching experience | ✅ |
| Protocol Tests | Verify protocol compatibility | Recommended |

**Rule:** `engine tests = correctness` | `server tests = realism`

---

## 4. Implementation Plan

### Phase 1: Add POST /sql Endpoint (Critical Path)

**File:** `crates/server/src/teaching_endpoints.rs`

**Interface:**
```json
// Request
POST /sql
{
  "sql": "SELECT * FROM t"
}

// Response
{
  "columns": ["id", "name"],
  "rows": [[1, "Alice"], [2, "Bob"]],
  "affected_rows": 0,
  "error": null
}
```

**Error Response:**
```json
{
  "columns": null,
  "rows": null,
  "affected_rows": 0,
  "error": "Parse error: ..."
}
```

### Phase 2: Create Client-Server Teaching Tests

**File:** `tests/integration/teaching_scenario_client_server_test.rs`

**Test Cases (Priority Order):**

1. **SQL Execution Path**
   - `test_client_server_create_table`
   - `test_client_server_insert`
   - `test_client_server_select`
   - `test_client_server_update`
   - `test_client_server_delete`

2. **Transaction Lifecycle**
   - `test_client_server_transaction_begin`
   - `test_client_server_transaction_commit`
   - `test_client_server_transaction_rollback`
   - `test_client_server_savepoint`

3. **Teaching Pipeline**
   - `test_client_server_explain`
   - `test_client_server_teaching_pipeline`

### Phase 3: Session Lifecycle Tests

**File:** `tests/client_server/transaction_session_test.rs`

- Session isolation
- Connection pool integration
- Server-side transaction scope

---

## 5. Success Criteria

After implementation:

| Capability | Before | After |
|------------|--------|-------|
| Real teaching system | ❌ | ✅ |
| Real database behavior simulation | ❌ | ✅ |
| Client-server course demo capability | ❌ | ✅ |
| MySQL alternative for teaching | ⚠️ | ✅ |
| Competition project credibility | Medium | High |

---

## 6. Key Insight

> This implementation is the critical step for SQLRustGo to transition from **"engine project"** to **"database system project"**.

Without this, teaching scenario tests remain unit tests with misleading names.
With this, SQLRustGo becomes a real database system suitable for teaching.
