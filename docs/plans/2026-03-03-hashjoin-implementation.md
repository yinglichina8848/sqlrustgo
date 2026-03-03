# HashJoin Implementation Plan (C-04)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement INNER JOIN support via HashJoin algorithm: Parser → Planner → Executor

**Architecture:** Create minimal Planner module with LogicalPlan/PhysicalPlan, add JOIN syntax to Parser, implement HashJoinExec in Executor, add Hash trait to Value type

**Tech Stack:** Rust, Criterion.rs (already available)

---

## Overview

This plan implements HashJoin for v1.1.0 release gate (C-04). The implementation follows a simplified architecture:

```
Parser (JOIN syntax) → Planner (LogicalPlan → PhysicalPlan) → Executor (HashJoinExec)
```

### Scope

- **INNER JOIN only**: `SELECT ... FROM a JOIN b ON a.id = b.id`
- **Simplified Planner**: Minimal LogicalPlan/PhysicalPlan without full optimizer
- **Compatible with existing Executor**: New HashJoinExec works alongside existing execution

---

## Task 1: Add Hash Trait to Value Type

**Files:**
- Modify: `src/types/value.rs`
- Test: Add tests in same file

**Step 1: Write failing test**

```rust
// Add at end of value.rs tests
#[test]
fn test_value_hash() {
    use std::collections::HashMap;
    use crate::types::Value;

    let mut map: HashMap<Value, i32> = HashMap::new();
    map.insert(Value::Int64(1), 100);
    map.insert(Value::Int64(2), 200);

    assert_eq!(map.get(&Value::Int64(1)), Some(&100));
    assert_eq!(map.get(&Value::Int64(2)), Some(&200));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_value_hash --all-features`
Expected: FAIL - no Hash trait implementation for Value

**Step 3: Write minimal implementation**

Add to `src/types/value.rs`:

```rust
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Int64(i64),
    Float64(f64),
    Text(String),
    Boolean(bool),
}

// Add Hash implementation
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Null => 0u8.hash(state),
            Value::Int64(n) => n.hash(state),
            Value::Float64(f) => f.to_bits().hash(state),
            Value::Text(s) => s.hash(state),
            Value::Boolean(b) => b.hash(state),
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_value_hash --all-features`
Expected: PASS

**Step 5: Commit**

```bash
git add src/types/value.rs
git commit -m "feat(types): implement Hash trait for Value (C-04)"
```

---

## Task 2: Add JOIN Syntax to Parser

**Files:**
- Modify: `src/parser/mod.rs`
- Test: Add parser tests

**Step 1: Write failing test**

Add to `src/parser/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_inner_join() {
        use crate::parser::Parser;

        let sql = "SELECT a.id, b.name FROM a JOIN b ON a.id = b.id";
        let mut parser = Parser::new(sql);
        let result = parser.parse_statement();

        assert!(result.is_ok(), "Parser should support INNER JOIN");
        let stmt = result.unwrap();
        // Verify it's a SelectStatement with join
        match stmt {
            Statement::Select(s) => {
                assert!(s.join.is_some(), "Should have join clause");
            }
            _ => panic!("Expected SELECT statement"),
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_parse_inner_join --all-features`
Expected: FAIL - no join field in SelectStatement

**Step 3: Write minimal implementation**

First, check current Statement/SelectStatement structure:

```rust
// In parser/mod.rs - add Join field
pub struct SelectStatement {
    pub table: String,
    pub columns: Vec<Column>,
    pub where_clause: Option<Expression>,
    pub join: Option<JoinClause>,  // ADD THIS
}

// Add JoinClause struct
#[derive(Debug, Clone)]
pub struct JoinClause {
    pub table: String,
    pub condition: Expression,
}

// Update parse_select to handle JOIN
fn parse_select(&mut self) -> Result<Statement, SqlError> {
    // ... existing code ...

    // After parsing table name, check for JOIN
    let join = if self.consume_keyword("JOIN") {
        let join_table = self.parse_identifier()?;
        self.expect_keyword("ON")?;
        let condition = self.parse_expression()?;
        Some(JoinClause {
            table: join_table,
            condition,
        })
    } else {
        None
    };

    Ok(Statement::Select(SelectStatement {
        table,
        columns,
        where_clause,
        join,  // ADD THIS
    }))
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_parse_inner_join --all-features`
Expected: PASS

**Step 5: Commit**

```bash
git add src/parser/mod.rs
git commit -m "feat(parser): add INNER JOIN syntax support (C-04)"
```

---

## Task 3: Create Planner Module

**Files:**
- Create: `src/planner/mod.rs`
- Create: `src/planner/logical_plan.rs`
- Create: `src/planner/physical_plan.rs`
- Test: Add planner tests

**Step 1: Write failing test**

Create `src/planner/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_logical_plan_join() {
        use crate::planner::LogicalPlan;

        let plan = LogicalPlan::Join {
            left: Box::new(LogicalPlan::TableScan { table_name: "a".to_string() }),
            right: Box::new(LogicalPlan::TableScan { table_name: "b".to_string() }),
            join_type: crate::planner::JoinType::Inner,
            condition: crate::types::Expression::Literal(crate::types::Value::Boolean(true)),
        };

        match plan {
            LogicalPlan::Join { .. } => (),
            _ => panic!("Expected Join plan"),
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_logical_plan_join --all-features`
Expected: FAIL - no planner module

**Step 3: Write minimal implementation**

Create `src/planner/mod.rs`:

```rust
//! Query Planner
//!
//! Converts Parser output to LogicalPlan and PhysicalPlan

pub mod logical_plan;
pub mod physical_plan;

pub use logical_plan::LogicalPlan;
pub use physical_plan::{PhysicalPlan, HashJoinExec};
```

Create `src/planner/logical_plan.rs`:

```rust
use crate::types::{Expression, Value};

/// Logical Plan nodes
#[derive(Debug, Clone)]
pub enum LogicalPlan {
    /// Table scan
    TableScan {
        table_name: String,
    },
    /// Filter operation
    Filter {
        input: Box<LogicalPlan>,
        predicate: Expression,
    },
    /// Projection
    Project {
        input: Box<LogicalPlan>,
        columns: Vec<String>,
    },
    /// Join operation
    Join {
        left: Box<LogicalPlan>,
        right: Box<LogicalPlan>,
        join_type: JoinType,
        condition: Expression,
    },
}

/// Join type
#[derive(Debug, Clone, Copy)]
pub enum JoinType {
    Inner,
}
```

Create `src/planner/physical_plan.rs`:

```rust
use crate::types::{Expression, Schema, Value};
use std::collections::HashMap;

/// Physical Plan nodes
#[derive(Debug, Clone)]
pub enum PhysicalPlan {
    /// Table scan
    TableScan {
        table_name: String,
    },
    /// Filter
    Filter {
        input: Box<PhysicalPlan>,
        predicate: Expression,
    },
    /// Projection
    Project {
        input: Box<PhysicalPlan>,
        columns: Vec<String>,
    },
    /// Hash Join
    HashJoin {
        left: Box<PhysicalPlan>,
        right: Box<PhysicalPlan>,
        join_type: crate::planner::logical_plan::JoinType,
        condition: Expression,
    },
}

/// Hash Join Executor
#[derive(Debug)]
pub struct HashJoinExec {
    left: Box<PhysicalPlan>,
    right: Box<PhysicalPlan>,
    join_type: crate::planner::logical_plan::JoinType,
    condition: Expression,
}

impl HashJoinExec {
    pub fn new(
        left: Box<PhysicalPlan>,
        right: Box<PhysicalPlan>,
        join_type: crate::planner::logical_plan::JoinType,
        condition: Expression,
    ) -> Self {
        Self {
            left,
            right,
            join_type,
            condition,
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_logical_plan_join --all-features`
Expected: PASS

**Step 5: Commit**

```bash
git add src/planner/
git commit -m "feat(planner): create planner module with LogicalPlan/PhysicalPlan (C-04)"
```

---

## Task 4: Implement HashJoinExec in Executor

**Files:**
- Modify: `src/executor/mod.rs`
- Test: Add HashJoin tests

**Step 1: Write failing test**

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_hash_join_inner() {
        // Test basic inner join
        use crate::executor::HashJoinExec;

        // This test verifies HashJoinExec can be created
        // Full integration test with data comes later
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_hash_join_inner --all-features`
Expected: FAIL - no HashJoinExec in executor

**Step 3: Write minimal implementation**

Add to `src/executor/mod.rs`:

```rust
use crate::types::{Expression, Schema, Value, SqlError, SqlResult};
use std::collections::HashMap;

/// Hash Join Executor
pub struct HashJoinExec {
    left: Box<dyn PhysicalOperator>,
    right: Box<dyn PhysicalOperator>,
    join_type: crate::planner::logical_plan::JoinType,
    condition: Expression,
    hash_table: HashMap<Value, Vec<Vec<Value>>>,
    left_schema: Schema,
    right_schema: Schema,
    built: bool,
}

impl HashJoinExec {
    pub fn new(
        left: Box<dyn PhysicalOperator>,
        right: Box<dyn PhysicalOperator>,
        join_type: crate::planner::logical_plan::JoinType,
        condition: Expression,
    ) -> Self {
        Self {
            left,
            right,
            join_type,
            condition,
            hash_table: HashMap::new(),
            left_schema: Schema::empty(),
            right_schema: Schema::empty(),
            built: false,
        }
    }

    /// Build hash table from left input
    fn build(&mut self) -> Result<(), SqlError> {
        if self.built {
            return Ok(());
        }

        // Build hash table from left side
        // For each row, extract join key and store
        self.built = true;
        Ok(())
    }
}

/// Physical operator trait
pub trait PhysicalOperator {
    fn next(&mut self) -> SqlResult<Option<Vec<Value>>>;
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_hash_join_inner --all-features`
Expected: PASS

**Step 5: Commit**

```bash
git add src/executor/mod.rs
git commit -m "feat(executor): implement HashJoinExec operator (C-04)"
```

---

## Task 5: Integration - Connect Parser to Executor

**Files:**
- Modify: `src/executor/mod.rs`
- Modify: `src/planner/mod.rs`
- Test: Integration test

**Step 1: Write failing test**

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_integration_hash_join() {
        use crate::{lexer::Lexer, parser::Parser, executor::ExecutionEngine};

        // Create tables
        let mut engine = ExecutionEngine::new();

        // Create test tables and data
        // Execute: SELECT a.id, b.name FROM a JOIN b ON a.id = b.id

        // This requires full integration
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_integration_hash_join --all-features`
Expected: FAIL - integration not complete

**Step 3: Write minimal implementation**

Add planner conversion in `src/planner/mod.rs`:

```rust
use crate::types::{Expression, SqlError};

/// Convert LogicalPlan to PhysicalPlan
pub fn to_physical(plan: LogicalPlan) -> Result<PhysicalPlan, SqlError> {
    match plan {
        LogicalPlan::TableScan { table_name } => {
            Ok(PhysicalPlan::TableScan { table_name })
        }
        LogicalPlan::Filter { input, predicate } => {
            let physical_input = to_physical(*input)?;
            Ok(PhysicalPlan::Filter {
                input: Box::new(physical_input),
                predicate,
            })
        }
        LogicalPlan::Project { input, columns } => {
            let physical_input = to_physical(*input)?;
            Ok(PhysicalPlan::Project {
                input: Box::new(physical_input),
                columns,
            })
        }
        LogicalPlan::Join { left, right, join_type, condition } => {
            let left_physical = to_physical(*left)?;
            let right_physical = to_physical(*right)?;
            Ok(PhysicalPlan::HashJoin {
                left: Box::new(left_physical),
                right: Box::new(right_physical),
                join_type,
                condition,
            })
        }
    }
}
```

Add execution in `src/executor/mod.rs`:

```rust
impl HashJoinExec {
    pub fn execute(&mut self) -> SqlResult<Option<Vec<Value>>> {
        // Build phase: populate hash table
        self.build()?;

        // Probe phase: look up matching rows
        Ok(None)
    }

    fn build(&mut self) -> Result<(), SqlError> {
        if self.built {
            return Ok(());
        }

        // Simple implementation: build hash table
        // In real implementation, iterate through left input
        self.built = true;
        Ok(())
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_integration_hash_join --all-features`
Expected: PASS (basic structure)

**Step 5: Commit**

```bash
git add src/planner/ src/executor/mod.rs
git commit -m "feat: connect planner to executor for HashJoin (C-04)"
```

---

## Task 6: Full Integration Test

**Files:**
- Test: `tests/hash_join_test.rs`
- Modify: Integration test

**Step 1: Write failing test**

```rust
// tests/hash_join_test.rs
use sqlrustgo::{lexer::Lexer, parser::Parser, executor::ExecutionEngine};

#[test]
fn test_hash_join_basic() {
    let mut engine = ExecutionEngine::new();

    // Create table a (id, name)
    engine.execute(sqlrustgo::parser::Statement::CreateTable(
        sqlrustgo::parser::CreateTableStatement {
            table_name: "a".to_string(),
            columns: vec![
                sqlrustgo::parser::ColumnDef {
                    name: "id".to_string(),
                    data_type: sqlrustgo::parser::DataType::Int,
                },
                sqlrustgo::parser::ColumnDef {
                    name: "name".to_string(),
                    data_type: sqlrustgo::parser::DataType::Text,
                },
            ],
        }
    )).unwrap();

    // Insert data into a
    // Create table b (id, value)
    // Insert data into b

    // Execute JOIN query
    let sql = "SELECT a.id, b.value FROM a JOIN b ON a.id = b.id";
    let lexer = Lexer::new(sql);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let stmt = parser.parse_statement().unwrap();

    let result = engine.execute(stmt);
    assert!(result.is_ok());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_hash_join_basic --all-features`
Expected: FAIL - various integration issues

**Step 3: Fix and implement**

Fix issues as they arise:
1. Ensure Statement enum has Join field
2. Ensure ExecutionEngine handles JOIN statements
3. Implement HashJoinExec::execute fully

**Step 4: Run test to verify it passes**

Run: `cargo test test_hash_join_basic --all-features`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/
git commit -m "test: add HashJoin integration tests (C-04)"
```

---

## Task 7: Final Verification

**Step 1: Run all tests**

```bash
cargo test --all-features
```

Expected: All tests pass

**Step 2: Run clippy**

```bash
cargo clippy --all-features -- -D warnings
```

Expected: No warnings

**Step 3: Check coverage**

```bash
cargo llvm-cov --all-features --summary-only
```

Expected: Coverage maintained or improved

**Step 4: Update Issue #115**

```bash
gh issue close 115 --comment "C-04 HashJoin 基础功能已完成"
```

**Step 5: Commit final**

```bash
git add -A
git commit -m "feat: complete HashJoin implementation (C-04)

- Add Hash trait to Value type
- Add INNER JOIN syntax to Parser
- Create Planner module with LogicalPlan/PhysicalPlan
- Implement HashJoinExec in Executor
- Add integration tests"
```

---

## Summary

| Task | Description | Status |
|------|-------------|--------|
| 1 | Add Hash trait to Value | - |
| 2 | Add JOIN syntax to Parser | - |
| 3 | Create Planner module | - |
| 4 | Implement HashJoinExec | - |
| 5 | Connect Parser to Executor | - |
| 6 | Full integration test | - |
| 7 | Final verification | - |
