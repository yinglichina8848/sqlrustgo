# SQL Beta Tasks Implementation Tasks

## SQL-1: RECURSIVE CTE

### Task 1.1: Fix evaluate_binary_op

**Files:**
- Modify: `crates/executor/src/stored_proc.rs`

**Steps:**
- [ ] Review current `evaluate_binary_op` implementation
- [ ] Add arithmetic operator support (+, -, *, /)
- [ ] Run existing CTE tests
- [ ] Commit

### Task 1.2: Fix UNION ALL in CTE

**Files:**
- Modify: `crates/executor/src/stored_proc.rs`

**Steps:**
- [ ] Review `execute_cte_subquery` for UNION ALL handling
- [ ] Fix result concatenation
- [ ] Verify recursive termination
- [ ] Run CTE tests
- [ ] Commit

### Task 1.3: Add CTE Integration Tests

**Files:**
- Create: `crates/executor/tests/cte_tests.rs`

**Steps:**
- [ ] Test simple recursive: `WITH RECURSIVE cte AS (SELECT 1 UNION ALL SELECT n+1 FROM cte WHERE n < 10) SELECT * FROM cte`
- [ ] Test multiple CTEs
- [ ] Test nested CTEs
- [ ] Commit

---

## SQL-2: Performance Schema

### Task 2.1: Add Setup Tables

**Files:**
- Modify: `crates/information-schema/src/performance_schema.rs`
- Modify: `crates/information-schema/src/lib.rs`

**Steps:**
- [ ] Add `setup_actors` struct and methods
- [ ] Add `setup_instruments` struct and methods
- [ ] Register in InformationSchema
- [ ] Run tests
- [ ] Commit

### Task 2.2: Extend Events Statements

**Files:**
- Modify: `crates/information-schema/src/performance_schema.rs`

**Steps:**
- [ ] Add SQL_TEXT, DIGEST fields to events_statements_current
- [ ] Add events_statements_summary_by_digest
- [ ] Run tests
- [ ] Commit

### Task 2.3: Add Events Waits Ring Buffer

**Files:**
- Modify: `crates/information-schema/src/performance_schema.rs`

**Steps:**
- [ ] Implement RingBuffer<T> struct
- [ ] Add events_waits_current/history with ring buffer
- [ ] Run tests
- [ ] Commit

### Task 2.4: Final Verification

**Steps:**
- [ ] Run all information-schema tests
- [ ] Verify Performance Schema coverage ≥60%
- [ ] Run clippy
- [ ] Commit
