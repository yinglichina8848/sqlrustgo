//! Test Harness

use crate::executor::executor::{Executor, RecordBatch};
use crate::types::SqlResult;

pub struct TestHarness {
    name: String,
}

impl TestHarness {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
    
    pub fn collect_all(&mut self, executor: &mut Box<dyn Executor>) -> SqlResult<Vec<RecordBatch>> {
        executor.init()?;
        let mut batches = vec![];
        while let Some(batch) = executor.next()? {
            batches.push(batch);
        }
        Ok(batches)
    }
    
    pub fn verify_row_count(&mut self, executor: &mut Box<dyn Executor>, expected: usize) -> SqlResult<bool> {
        let batches = self.collect_all(executor)?;
        let total: usize = batches.iter().map(|b| b.num_rows).sum();
        Ok(total == expected)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

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

    pub fn table_names(&self) -> Vec<&String> {
        self.data.keys().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for TestFixture {
    fn default() -> Self { Self::new() }
}

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

    pub fn assert_columns(batch: &RecordBatch, expected: &[&str]) {
        assert_eq!(batch.columns, expected.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::executor::{NullExecutor, RecordBatch};
    use crate::types::Value;

    #[test]
    fn test_harness_new() {
        let harness = TestHarness::new("test");
        assert_eq!(harness.name(), "test");
    }

    #[test]
    fn test_harness_name() {
        let harness = TestHarness::new("my_test");
        assert_eq!(harness.name, "my_test");
    }

    #[test]
    fn test_test_fixture_new() {
        let fixture = TestFixture::new();
        assert!(fixture.is_empty());
    }

    #[test]
    fn test_test_fixture_add_table() {
        let fixture = TestFixture::new()
            .add_table("users", vec![vec![Value::Integer(1)]]);
        assert!(fixture.get_table("users").is_some());
    }

    #[test]
    fn test_test_fixture_table_names() {
        let fixture = TestFixture::new()
            .add_table("users", vec![])
            .add_table("orders", vec![]);
        let names = fixture.table_names();
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_assert_row_count() {
        let batch = RecordBatch::new(
            vec!["id".to_string()],
            vec![vec![Value::Integer(1)], vec![Value::Integer(2)]],
        );
        assertions::assert_row_count(&batch, 2);
    }

    #[test]
    fn test_assert_columns() {
        let batch = RecordBatch::new(
            vec!["id".to_string(), "name".to_string()],
            vec![],
        );
        assertions::assert_columns(&batch, &["id", "name"]);
    }
}
