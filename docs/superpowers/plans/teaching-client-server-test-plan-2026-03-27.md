# Teaching Client-Server Test Implementation Plan

## Date: 2026-03-27
## Status: In Progress

---

## Phase 1: Add POST /sql Endpoint

### Task 1.1: Extend TeachingHttpServer with SQL Execution

**File:** `crates/server/src/teaching_endpoints.rs`

**Changes:**
1. Add `ExecutionEngine` integration to `TeachingHttpServer`
2. Add `POST /sql` endpoint handler
3. Parse JSON request `{"sql": "..."}`
4. Execute SQL via engine
5. Return JSON response

**Request Format:**
```json
POST /sql
Content-Type: application/json

{"sql": "SELECT * FROM users WHERE id = 1"}
```

**Response Format:**
```json
HTTP/1.1 200 OK
Content-Type: application/json

{
  "columns": ["id", "name", "email"],
  "rows": [[1, "Alice", "alice@example.com"]],
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
  "error": "Table 'users' does not exist"
}
```

### Task 1.2: Update TeachingHttpServer Struct

**Current:**
```rust
pub struct TeachingHttpServer {
    host: String,
    port: u16,
    version: String,
    metrics_registry: Arc<RwLock<MetricsRegistry>>,
    teaching_endpoints: TeachingEndpoints,
}
```

**Updated:**
```rust
pub struct TeachingHttpServer {
    host: String,
    port: u16,
    version: String,
    metrics_registry: Arc<RwLock<MetricsRegistry>>,
    teaching_endpoints: TeachingEndpoints,
    storage: Arc<RwLock<dyn StorageEngine>>,  // NEW
}
```

**Builder Pattern:**
```rust
impl TeachingHttpServer {
    pub fn with_storage(mut self, storage: Arc<RwLock<dyn StorageEngine>>) -> Self {
        self.storage = storage;
        self
    }
}
```

---

## Phase 2: Create Client-Server Teaching Tests

### Task 2.1: Create Test File

**File:** `tests/integration/teaching_scenario_client_server_test.rs`

### Task 2.2: Implement Test Fixture

```rust
struct TestServer {
    handle: JoinHandle<()>,
    port: u16,
}

impl TestServer {
    fn new() -> Self {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let server = TeachingHttpServer::new("127.0.0.1", 0)
            .with_storage(storage);

        let port = server.bind_to_available_port();
        let handle = thread::spawn(move || {
            let _ = server.start();
        });

        thread::sleep(Duration::from_millis(100));

        Self { handle, port }
    }

    fn sql(&self, sql: &str) -> SqlResponse {
        let client = reqwest::blocking::Client::new();
        client.post(&format!("http://127.0.0.1:{}/sql", self.port))
            .json(&json!({"sql": sql}))
            .send()
            .unwrap()
            .json()
            .unwrap()
    }
}
```

### Task 2.3: Test Cases

| Test | SQL | Expected |
|------|-----|----------|
| `test_create_table` | `CREATE TABLE t (id INT, name TEXT)` | affected_rows=0, no error |
| `test_insert` | `INSERT INTO t VALUES (1, 'Alice')` | affected_rows=1 |
| `test_select` | `SELECT * FROM t` | rows contain inserted data |
| `test_update` | `UPDATE t SET name='Bob' WHERE id=1` | affected_rows=1 |
| `test_delete` | `DELETE FROM t WHERE id=1` | affected_rows=1 |
| `test_select_where` | `SELECT * FROM t WHERE id=1` | filtered rows |
| `test_explain` | `EXPLAIN SELECT * FROM t` | returns query plan |

---

## Phase 3: Integration with Teaching Pipeline

### Task 3.1: Test /teaching/pipeline After SQL Execution

```rust
#[test]
fn test_teaching_pipeline_after_query() {
    let server = TestServer::new();

    // Execute query (should populate trace)
    server.sql("SELECT * FROM users");

    // Get pipeline visualization
    let pipeline: Value = server.get("/teaching/pipeline/json");

    assert!(pipeline.is_array());
    assert!(!pipeline.as_array().unwrap().is_empty());
}
```

---

## Phase 4: Transaction Session Tests

### Task 4.1: Test Session Lifecycle

```rust
#[test]
fn test_transaction_session() {
    let server = TestServer::new();

    // Begin transaction
    server.sql("BEGIN");

    // Insert in transaction
    server.sql("INSERT INTO t VALUES (1, 'Alice')");

    // Rollback
    server.sql("ROLLBACK");

    // Verify data is gone
    let result = server.sql("SELECT * FROM t");
    assert!(result.rows.is_empty());
}
```

---

## Implementation Order

```
1. Task 1.1: Add POST /sql endpoint (crates/server/src/teaching_endpoints.rs)
2. Task 1.2: Update TeachingHttpServer struct
3. Task 2.1: Create test file structure
4. Task 2.2: Implement TestServer fixture
5. Task 2.3: Add SQL execution tests (CRUD)
6. Task 3.1: Add teaching pipeline tests
7. Task 4.1: Add transaction session tests
```

---

## Verification

After implementation:

```bash
# Run new client-server tests
cargo test --test teaching_scenario_client_server_test

# Run teaching integration tests
cargo test --test teaching_scenario_test

# Run full test suite
cargo test --all-features
```

---

## Expected Outcomes

| Metric | Before | After |
|--------|--------|-------|
| Teaching tests cover real client-server | ❌ | ✅ |
| POST /sql endpoint exists | ❌ | ✅ |
| Transaction lifecycle testable | ❌ | ✅ |
| Teaching pipeline testable | ⚠️ | ✅ |
| Client-server test count | 0 | ~15 |
