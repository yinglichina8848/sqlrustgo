//! Test Harness
//!
//! Provides test harness utilities for executor tests

use crate::executor::executor::{Executor, RecordBatch};
use crate::types::SqlResult;

/// Test harness for running executor tests
pub struct TestHarness {
    name: String,
}

impl TestHarness {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
    
    /// Run an executor and collect all results
    pub fn collect_all(&mut self, executor: &mut Box<dyn Executor>) -> SqlResult<Vec<RecordBatch>> {
        executor.init()?;
        let mut batches = vec![];
        while let Some(batch) = executor.next()? {
            batches.push(batch);
        }
        Ok(batches)
    }
    
    /// Verify that executor produces expected number of rows
    pub fn verify_row_count(&mut self, executor: &mut Box<dyn Executor>, expected: usize) -> SqlResult<bool> {
        let batches = self.collect_all(executor)?;
        let total: usize = batches.iter().map(|b| b.num_rows).sum();
        Ok(total == expected)
    }
}

/// Test fixture for setting up test data
pub struct TestFixture {
    data: std::collections::HashMap<String, Vec<Vec<crate::types::Value>>>,
}

impl TestFixture {
    pub fn new() -> Self {
        Self { data: std::collections::HashMap::new() }
    }
    
    pub fn add_table(mut self, name: &str, rows: Vec<Vec<crate::types::Value>>) -> Self {
        self.data.insert(name.to_string(), rows);
        self
    }
    
    pub fn get_table(&self, name: &str) -> Option<&Vec<Vec<crate::types::Value>>> {
        self.data.get(name)
    }
}

/// Assertion utilities
pub mod assertions {
    use crate::executor::executor::RecordBatch;
    use crate::types::Value;
    
    pub fn assert_batch_equals(batch: &RecordBatch, expected_columns: &[&str], expected_rows: &[&[Value]]) {
        assert_eq!(batch.columns, expected_columns.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        assert_eq!(batch.rows.len(), expected_rows.len());
        for (i, row) in expected_rows.iter().enumerate() {
            assert_eq!(&batch.rows[i], *row);
        }
    }
    
    pub fn assert_row_count(batch: &RecordBatch, expected: usize) {
        assert_eq!(batch.num_rows, expected);
    }
}
