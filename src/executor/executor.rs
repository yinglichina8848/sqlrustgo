//! Volcano Model Executor Trait
use crate::types::{SqlResult, Value};

#[derive(Debug, Clone)]
pub struct RecordBatch {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub num_rows: usize,
}

impl RecordBatch {
    pub fn new(columns: Vec<String>, rows: Vec<Vec<Value>>) -> Self {
        let num_rows = rows.len();
        Self { columns, rows, num_rows }
    }
    pub fn empty() -> Self {
        Self { columns: vec![], rows: vec![], num_rows: 0 }
    }
}

pub trait Executor: Send {
    fn next(&mut self) -> SqlResult<Option<RecordBatch>>;
    fn schema(&self) -> &[String];
    fn init(&mut self) -> SqlResult<()> { Ok(()) }
    fn close(&mut self) -> SqlResult<()> { Ok(()) }
}

pub type BoxExecutor = Box<dyn Executor>;

pub struct NullExecutor { schema: Vec<String> }
impl NullExecutor {
    pub fn new(schema: Vec<String>) -> Self { Self { schema } }
}
impl Executor for NullExecutor {
    fn next(&mut self) -> SqlResult<Option<RecordBatch>> { Ok(None) }
    fn schema(&self) -> &[String] { &self.schema }
}

pub struct OnceExecutor { batch: Option<RecordBatch> }
impl OnceExecutor {
    pub fn new(batch: RecordBatch) -> Self { Self { batch: Some(batch) } }
}
impl Executor for OnceExecutor {
    fn next(&mut self) -> SqlResult<Option<RecordBatch>> { Ok(self.batch.take()) }
    fn schema(&self) -> &[String] { self.batch.as_ref().map(|b| b.columns.as_slice()).unwrap_or(&[]) }
}

pub use crate::executor::ExecutionEngine;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    #[test]
    fn test_record_batch_new() {
        let columns = vec!["id".to_string(), "name".to_string()];
        let rows = vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string())],
            vec![Value::Integer(2), Value::Text("Bob".to_string())],
        ];
        let batch = RecordBatch::new(columns.clone(), rows.clone());
        assert_eq!(batch.columns, columns);
        assert_eq!(batch.rows.len(), 2);
        assert_eq!(batch.num_rows, 2);
    }

    #[test]
    fn test_record_batch_empty() {
        let batch = RecordBatch::empty();
        assert!(batch.columns.is_empty());
        assert!(batch.rows.is_empty());
        assert_eq!(batch.num_rows, 0);
    }

    #[test]
    fn test_null_executor() {
        let mut executor = NullExecutor::new(vec!["col1".to_string()]);
        assert_eq!(executor.schema(), &["col1"]);
        let result = executor.next();
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_once_executor() {
        let batch = RecordBatch::new(
            vec!["id".to_string()],
            vec![vec![Value::Integer(1)]],
        );
        let mut executor = OnceExecutor::new(batch);
        let result = executor.next().unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().num_rows, 1);
        let result = executor.next().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_once_executor_schema() {
        let batch = RecordBatch::new(
            vec!["a".to_string(), "b".to_string()],
            vec![],
        );
        let executor = OnceExecutor::new(batch);
        assert_eq!(executor.schema(), &["a", "b"]);
    }
}
