//! Projection Executor
//!
//! Implements projection operator for the Volcano model.

use crate::executor::executor::{Executor, RecordBatch};
use crate::types::{SqlResult, Value};

/// ProjectionExecutor performs column projection on input data
pub struct ProjectionExecutor {
    child: Box<dyn Executor>,
    column_indices: Vec<usize>,
    output_columns: Vec<String>,
}

impl ProjectionExecutor {
    pub fn new(child: Box<dyn Executor>, column_indices: Vec<usize>, output_columns: Vec<String>) -> Self {
        Self { child, column_indices, output_columns }
    }
}

impl Executor for ProjectionExecutor {
    fn next(&mut self) -> SqlResult<Option<RecordBatch>> {
        match self.child.next()? {
            Some(batch) => {
                let projected_rows: Vec<Vec<Value>> = batch.rows.iter().map(|row| {
                    self.column_indices.iter().filter_map(|&idx| row.get(idx).cloned()).collect()
                }).collect();
                Ok(Some(RecordBatch::new(self.output_columns.clone(), projected_rows)))
            }
            None => Ok(None),
        }
    }
    fn schema(&self) -> &[String] { &self.output_columns }
    fn init(&mut self) -> SqlResult<()> { self.child.init() }
}

/// FilterExecutor applies a filter condition to input data
pub struct FilterExecutor {
    child: Box<dyn Executor>,
    predicate: Box<dyn Fn(&[Value]) -> bool + Send>,
}

impl FilterExecutor {
    pub fn new(child: Box<dyn Executor>, predicate: Box<dyn Fn(&[Value]) -> bool + Send>) -> Self {
        Self { child, predicate }
    }
}

impl Executor for FilterExecutor {
    fn next(&mut self) -> SqlResult<Option<RecordBatch>> {
        Ok(None) // TODO: implement proper filtering
    }
    fn schema(&self) -> &[String] { self.child.schema() }
    fn init(&mut self) -> SqlResult<()> { self.child.init() }
}
