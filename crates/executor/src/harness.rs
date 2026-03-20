//! Test Harness for Executor Testing
//!
//! This module provides utilities and helpers for testing executor operations,
//! including test fixtures, assertions, and common test patterns.

use crate::{ExecutorResult, LocalExecutor};
use sqlrustgo_planner::{PhysicalPlan, Schema};
use sqlrustgo_storage::StorageEngine;
use sqlrustgo_types::{SqlResult, Value};

/// TestHarness - Main test harness for executor testing
pub struct TestHarness<S: StorageEngine> {
    storage: S,
}

impl<S: StorageEngine> TestHarness<S> {
    /// Create a new TestHarness with the given storage
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Execute a physical plan and return the result
    pub fn execute(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let executor = LocalExecutor::new(&self.storage);
        executor.execute(plan)
    }

    /// Get the storage engine
    pub fn storage(&self) -> &S {
        &self.storage
    }
}

/// ExecutorTestCase - Structure for defining test cases
pub struct ExecutorTestCase {
    pub name: String,
    pub plan: Box<dyn PhysicalPlan>,
    pub expected_rows: usize,
    pub expected_first_row: Option<Vec<Value>>,
}

impl ExecutorTestCase {
    /// Create a new test case
    pub fn new(name: &str, plan: Box<dyn PhysicalPlan>) -> Self {
        Self {
            name: name.to_string(),
            plan,
            expected_rows: 0,
            expected_first_row: None,
        }
    }

    /// Set expected row count
    pub fn expect_rows(mut self, count: usize) -> Self {
        self.expected_rows = count;
        self
    }

    /// Set expected first row
    pub fn expect_first_row(mut self, row: Vec<Value>) -> Self {
        self.expected_first_row = Some(row);
        self
    }

    /// Run the test case
    pub fn run<S: StorageEngine>(&self, harness: &TestHarness<S>) -> SqlResult<()> {
        let result = harness.execute(self.plan.as_ref())?;

        assert_eq!(
            result.rows.len(),
            self.expected_rows,
            "Test '{}' failed: expected {} rows, got {}",
            self.name,
            self.expected_rows,
            result.rows.len()
        );

        if let Some(ref expected) = self.expected_first_row {
            assert!(
                !result.rows.is_empty(),
                "Test '{}' failed: expected first row but result is empty",
                self.name
            );
            assert_eq!(
                result.rows[0], *expected,
                "Test '{}' failed: first row mismatch",
                self.name
            );
        }

        Ok(())
    }
}

/// Assertion helpers for executor tests
pub mod assertions {
    use super::*;

    /// Assert that an executor result has the expected number of rows
    pub fn assert_row_count(result: &ExecutorResult, expected: usize) {
        assert_eq!(
            result.rows.len(),
            expected,
            "Expected {} rows, got {}",
            expected,
            result.rows.len()
        );
    }

    /// Assert that an executor result has rows
    pub fn assert_has_rows(result: &ExecutorResult) {
        assert!(
            !result.rows.is_empty(),
            "Expected at least one row, but result is empty"
        );
    }

    /// Assert that an executor result is empty
    pub fn assert_no_rows(result: &ExecutorResult) {
        assert!(
            result.rows.is_empty(),
            "Expected no rows, but got {}",
            result.rows.len()
        );
    }

    /// Assert that the first row matches expected values
    pub fn assert_first_row_equals(result: &ExecutorResult, expected: &[Value]) {
        assert!(
            !result.rows.is_empty(),
            "Cannot assert first row: result is empty"
        );
        assert_eq!(result.rows[0], expected, "First row mismatch");
    }

    /// Assert that a specific row matches
    pub fn assert_row_equals(result: &ExecutorResult, index: usize, expected: &[Value]) {
        assert!(
            index < result.rows.len(),
            "Row index {} out of bounds (total rows: {})",
            index,
            result.rows.len()
        );
        assert_eq!(result.rows[index], expected, "Row {} mismatch", index);
    }

    /// Assert that affected_rows matches expected
    pub fn assert_affected_rows(result: &ExecutorResult, expected: usize) {
        assert_eq!(
            result.affected_rows, expected,
            "Expected {} affected rows, got {}",
            expected, result.affected_rows
        );
    }

    /// Assert that result contains a specific value in any row
    pub fn assert_contains_value(result: &ExecutorResult, value: &Value) {
        let found = result.rows.iter().any(|row| row.contains(value));
        assert!(
            found,
            "Expected to find value {:?} in result, but it was not found",
            value
        );
    }
}

/// Helper functions for creating test plans
pub mod helpers {
    use super::*;
    use sqlrustgo_planner::{Expr, Field, SeqScanExec};

    /// Create a simple SeqScan plan for testing
    pub fn create_seq_scan_plan(table_name: &str, schema: Schema) -> Box<dyn PhysicalPlan> {
        Box::new(SeqScanExec::new(table_name.to_string(), schema))
    }

    /// Create a simple projection plan
    pub fn create_projection_plan(
        child: Box<dyn PhysicalPlan>,
        exprs: Vec<Expr>,
        output_schema: Schema,
    ) -> Box<dyn PhysicalPlan> {
        use sqlrustgo_planner::ProjectionExec;
        Box::new(ProjectionExec::new(child, exprs, output_schema))
    }

    /// Create a simple filter plan
    pub fn create_filter_plan(
        child: Box<dyn PhysicalPlan>,
        predicate: Expr,
    ) -> Box<dyn PhysicalPlan> {
        use sqlrustgo_planner::FilterExec;
        Box::new(FilterExec::new(child, predicate))
    }

    /// Create a simple aggregate plan
    pub fn create_aggregate_plan(
        child: Box<dyn PhysicalPlan>,
        group_expr: Vec<Expr>,
        aggregate_expr: Vec<Expr>,
        output_schema: Schema,
    ) -> Box<dyn PhysicalPlan> {
        use sqlrustgo_planner::AggregateExec;
        Box::new(AggregateExec::new(
            child,
            group_expr,
            aggregate_expr,
            output_schema,
        ))
    }

    /// Create a schema for common test tables
    pub fn users_schema() -> Schema {
        Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ])
    }

    /// Create a schema for orders table
    pub fn orders_schema() -> Schema {
        Schema::new(vec![
            Field::new("order_id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("user_id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("amount".to_string(), sqlrustgo_planner::DataType::Integer),
        ])
    }

    /// Create a schema for products table
    pub fn products_schema() -> Schema {
        Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
            Field::new("price".to_string(), sqlrustgo_planner::DataType::Integer),
        ])
    }
}

/// Fixture utilities for setting up test data
pub mod fixtures {
    use super::*;
    use sqlrustgo_storage::{ColumnDefinition, TableInfo};

    /// Setup a test users table with data
    pub fn setup_users_table(storage: &mut dyn StorageEngine) -> SqlResult<()> {
        storage.create_table(&TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: false,
                    is_unique: false,
                },
            ],
        })?;

        storage.insert(
            "users",
            vec![
                vec![Value::Integer(1), Value::Text("Alice".to_string())],
                vec![Value::Integer(2), Value::Text("Bob".to_string())],
                vec![Value::Integer(3), Value::Text("Charlie".to_string())],
            ],
        )?;

        Ok(())
    }

    /// Setup a test customers table with data (for TPC-H Q10)
    pub fn setup_customers_table(storage: &mut dyn StorageEngine) -> SqlResult<()> {
        storage.create_table(&TableInfo {
            name: "customer".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "c_custkey".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                },
                ColumnDefinition {
                    name: "c_name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: false,
                    is_unique: false,
                },
            ],
        })?;

        storage.insert(
            "customer",
            vec![
                vec![
                    Value::Integer(1),
                    Value::Text("Customer#000000001".to_string()),
                ],
                vec![
                    Value::Integer(2),
                    Value::Text("Customer#000000002".to_string()),
                ],
                vec![
                    Value::Integer(3),
                    Value::Text("Customer#000000003".to_string()),
                ],
            ],
        )?;

        Ok(())
    }

    /// Setup a test orders table with data
    pub fn setup_orders_table(storage: &mut dyn StorageEngine) -> SqlResult<()> {
        storage.create_table(&TableInfo {
            name: "orders".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "order_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                },
                ColumnDefinition {
                    name: "user_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                },
                ColumnDefinition {
                    name: "amount".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                },
            ],
        })?;

        storage.insert(
            "orders",
            vec![
                vec![Value::Integer(1), Value::Integer(1), Value::Integer(100)],
                vec![Value::Integer(2), Value::Integer(1), Value::Integer(200)],
                vec![Value::Integer(3), Value::Integer(2), Value::Integer(300)],
            ],
        )?;

        Ok(())
    }

    #[test]
    fn test_compare_rows_match() {
        let actual = vec![Value::Integer(1), Value::Text("Alice".to_string())];
        let expected = vec![Value::Integer(1), Value::Text("Alice".to_string())];
        assert!(compare::rows_match(&actual, &expected));
    }

    #[test]
    fn test_compare_rows_with_null() {
        let actual = vec![Value::Integer(1), Value::Text("Alice".to_string())];
        let expected = vec![Value::Null, Value::Text("Alice".to_string())];
        assert!(compare::rows_match(&actual, &expected));
    }

    #[test]
    fn test_compare_find_matching_row() {
        let result = vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string())],
            vec![Value::Integer(2), Value::Text("Bob".to_string())],
        ];
        let expected = vec![Value::Null, Value::Text("Bob".to_string())];

        let found = compare::find_matching_row(&result, &expected);
        assert!(found.is_some());
    }
}
