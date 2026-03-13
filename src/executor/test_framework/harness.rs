//! Test Harness for Executor Tests
//!
//! Provides utilities for setting up and managing test environments.

use std::path::PathBuf;
use crate::executor::{ExecutionEngine, ExecutionResult, TableData};
use crate::parser::parse;
use crate::types::{SqlError, SqlResult, Value};

/// A test harness for the executor
/// Provides utilities for setting up tests with various configurations
pub struct TestHarness {
    /// The execution engine
    pub engine: ExecutionEngine,
    /// Test-specific data directory
    data_dir: PathBuf,
    /// Whether to preserve data after test
    preserve: bool,
}

impl TestHarness {
    /// Create a new test harness with a temporary directory
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let random_part = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        
        let data_dir = std::env::temp_dir().join(format!(
            "sqlrustgo_test_{}_{}",
            std::process::id(),
            random_part
        ));
        
        // Create the directory if it doesn't exist
        std::fs::create_dir_all(&data_dir).ok();
        
        let engine = ExecutionEngine::with_data_dir(data_dir.clone());
        
        Self {
            engine,
            data_dir,
            preserve: false,
        }
    }

    /// Create a new test harness with a specific data directory
    pub fn with_dir(data_dir: PathBuf) -> Self {
        let engine = ExecutionEngine::with_data_dir(data_dir.clone());
        
        Self {
            engine,
            data_dir,
            preserve: false,
        }
    }

    /// Preserve the data directory after test (for debugging)
    pub fn preserve_data(mut self) -> Self {
        self.preserve = true;
        self
    }

    /// Execute a SQL statement and return the result
    pub fn execute(&mut self, sql: &str) -> SqlResult<ExecutionResult> {
        let statement = parse(sql)?;
        self.engine.execute(statement)
    }

    /// Execute a SQL statement and expect success
    pub fn execute_ok(&mut self, sql: &str) -> ExecutionResult {
        self.execute(sql).expect(&format!("SQL failed: {}", sql))
    }

    /// Execute a SQL statement and expect failure
    pub fn execute_err(&mut self, sql: &str) -> SqlError {
        self.execute(sql).expect_err(&format!("SQL should have failed: {}", sql))
    }

    /// Create a table using the harness
    pub fn create_table(&mut self, sql: &str) {
        self.execute_ok(sql);
    }

    /// Insert data using the harness
    pub fn insert(&mut self, sql: &str) -> ExecutionResult {
        self.execute_ok(sql)
    }

    /// Select data using the harness
    pub fn select(&mut self, sql: &str) -> ExecutionResult {
        self.execute_ok(sql)
    }

    /// Get a table from the engine
    pub fn get_table(&self, name: &str) -> Option<&TableData> {
        self.engine.get_table(name)
    }

    /// Check if a table exists
    pub fn table_exists(&self, name: &str) -> bool {
        self.get_table(name).is_some()
    }

    /// Get the row count for a table
    pub fn row_count(&self, table: &str) -> usize {
        self.get_table(table).map(|t| t.rows.len()).unwrap_or(0)
    }

    /// Verify the result matches expected values
    pub fn verify_result(&self, result: &ExecutionResult, expected_columns: &[&str], expected_rows: usize) {
        assert_eq!(
            result.columns,
            expected_columns,
            "Columns don't match"
        );
        assert_eq!(
            result.rows.len(),
            expected_rows,
            "Row count doesn't match"
        );
    }

    /// Verify a specific value in the result
    pub fn verify_value(&self, result: &ExecutionResult, row: usize, col: usize, expected: &Value) {
        assert_eq!(
            result.rows.get(row).and_then(|r| r.get(col)),
            Some(expected),
            "Value at ({}, {}) doesn't match",
            row, col
        );
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        if !self.preserve {
            let _ = std::fs::remove_dir_all(&self.data_dir);
        }
    }
}

/// A simple test case runner
#[macro_export]
macro_rules! test_case {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            let mut harness = $crate::executor::tests::TestHarness::new();
            $body(&mut harness);
        }
    };
}

/// A test case that expects an error
#[macro_export]
macro_rules! test_case_err {
    ($name:ident, $sql:expr, $error_type:pat) => {
        #[test]
        fn $name() {
            let mut harness = $crate::executor::tests::TestHarness::new();
            let result = harness.execute($sql);
            assert!(matches!(result, Err($error_type)));
        }
    };
}

/// A parameterized test case
#[macro_export]
macro_rules! parameterized_test {
    ($name:ident, $cases:expr, $body:expr) => {
        #[test]
        fn $name() {
            for (i, case) in $cases.iter().enumerate() {
                let mut harness = $crate::executor::tests::TestHarness::new();
                $body(&mut harness, case, i);
            }
        }
    };
}

/// Fixture for creating common test scenarios
pub struct TestFixture {
    /// Test harness
    pub harness: TestHarness,
}

impl TestFixture {
    /// Create a new fixture with basic tables
    pub fn basic() -> Self {
        let mut harness = TestHarness::new();
        
        // Create users table
        harness.execute_ok("CREATE TABLE users (id INTEGER, name TEXT, age INTEGER)");
        
        // Insert test data
        harness.execute_ok("INSERT INTO users VALUES (1, 'Alice', 30)");
        harness.execute_ok("INSERT INTO users VALUES (2, 'Bob', 25)");
        harness.execute_ok("INSERT INTO users VALUES (3, 'Charlie', 35)");
        
        Self { harness }
    }

    /// Create a fixture with products table
    pub fn products() -> Self {
        let mut harness = TestHarness::new();
        
        harness.execute_ok("CREATE TABLE products (id INTEGER, name TEXT, price REAL, stock INTEGER)");
        harness.execute_ok("INSERT INTO products VALUES (1, 'Apple', 1.50, 100)");
        harness.execute_ok("INSERT INTO products VALUES (2, 'Banana', 0.75, 200)");
        harness.execute_ok("INSERT INTO products VALUES (3, 'Orange', 2.00, 50)");
        
        Self { harness }
    }

    /// Create a fixture with orders table
    pub fn orders() -> Self {
        let mut harness = TestHarness::new();
        
        harness.execute_ok("CREATE TABLE orders (id INTEGER, user_id INTEGER, product_id INTEGER, quantity INTEGER, total REAL)");
        harness.execute_ok("INSERT INTO orders VALUES (1, 1, 1, 2, 3.00)");
        harness.execute_ok("INSERT INTO orders VALUES (2, 1, 2, 1, 0.75)");
        harness.execute_ok("INSERT INTO orders VALUES (3, 2, 1, 3, 4.50)");
        
        Self { harness }
    }

    /// Create a fixture with multiple tables (join testing)
    pub fn with_join() -> Self {
        let mut harness = TestHarness::new();
        
        // Users table
        harness.execute_ok("CREATE TABLE users (id INTEGER, name TEXT)");
        harness.execute_ok("INSERT INTO users VALUES (1, 'Alice')");
        harness.execute_ok("INSERT INTO users VALUES (2, 'Bob')");
        
        // Orders table
        harness.execute_ok("CREATE TABLE orders (id INTEGER, user_id INTEGER, amount REAL)");
        harness.execute_ok("INSERT INTO orders VALUES (1, 1, 100.00)");
        harness.execute_ok("INSERT INTO orders VALUES (2, 1, 50.00)");
        harness.execute_ok("INSERT INTO orders VALUES (3, 2, 75.00)");
        
        Self { harness }
    }

    /// Create an empty fixture
    pub fn empty() -> Self {
        Self {
            harness: TestHarness::new(),
        }
    }
}

/// Assertion helpers for test verification
pub mod assertions {
    use crate::executor::ExecutionResult;
    use crate::types::Value;

    /// Assert that a result has the expected number of rows
    pub fn assert_row_count(result: &ExecutionResult, expected: usize) {
        assert_eq!(
            result.rows.len(),
            expected,
            "Expected {} rows, got {}",
            expected,
            result.rows.len()
        );
    }

    /// Assert that a result has the expected columns
    pub fn assert_columns(result: &ExecutionResult, expected: &[&str]) {
        assert_eq!(
            result.columns,
            expected,
            "Columns don't match: expected {:?}, got {:?}",
            expected,
            result.columns
        );
    }

    /// Assert that a specific cell has the expected value
    pub fn assert_cell(result: &ExecutionResult, row: usize, col: usize, expected: &Value) {
        let actual = result
            .rows
            .get(row)
            .and_then(|r| r.get(col))
            .expect(&format!("Cell ({}, {}) not found", row, col));
        
        assert_eq!(
            actual, expected,
            "Cell ({}, {}) value mismatch: expected {:?}, got {:?}",
            row, col, expected, actual
        );
    }

    /// Assert that rows were affected
    pub fn assert_affected(result: &ExecutionResult, expected: u64) {
        assert_eq!(
            result.rows_affected, expected,
            "Expected {} rows affected, got {}",
            expected, result.rows_affected
        );
    }

    /// Assert result is empty
    pub fn assert_empty(result: &ExecutionResult) {
        assert_row_count(result, 0);
    }

    /// Assert result is not empty
    pub fn assert_not_empty(result: &ExecutionResult) {
        assert!(
            !result.rows.is_empty(),
            "Expected non-empty result"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_new() {
        let harness = TestHarness::new();
        assert!(harness.engine.get_table("users").is_none());
    }

    #[test]
    fn test_harness_execute() {
        let mut harness = TestHarness::new();
        let result = harness.execute("CREATE TABLE users (id INTEGER, name TEXT)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_harness_execute_ok() {
        let mut harness = TestHarness::new();
        let _ = harness.execute_ok("CREATE TABLE users (id INTEGER, name TEXT)");
        assert!(harness.table_exists("users"));
    }

    #[test]
    fn test_harness_execute_err() {
        let mut harness = TestHarness::new();
        let _err = harness.execute_err("SELECT * FROM nonexistent");
    }

    #[test]
    fn test_fixture_basic() {
        let fixture = TestFixture::basic();
        assert!(fixture.harness.table_exists("users"));
        assert_eq!(fixture.harness.row_count("users"), 3);
    }

    #[test]
    fn test_fixture_products() {
        let fixture = TestFixture::products();
        assert!(fixture.harness.table_exists("products"));
    }

    #[test]
    fn test_assertions() {
        use crate::parser::parse;
        
        let mut harness = TestHarness::new();
        harness.execute_ok("CREATE TABLE users (id INTEGER, name TEXT)");
        harness.execute_ok("INSERT INTO users VALUES (1, 'Alice')");
        
        let result = harness.execute_ok("SELECT * FROM users");
        
        assertions::assert_row_count(&result, 1);
        assertions::assert_columns(&result, &["id", "name"]);
        assertions::assert_cell(&result, 0, 0, &Value::Integer(1));
    }
}
