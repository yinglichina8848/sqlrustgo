//! Projection Executor
use crate::executor::executor::{Executor, RecordBatch};
use crate::types::{SqlResult, Value};

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
    fn next(&mut self) -> SqlResult<Option<RecordBatch>> { Ok(None) }
    fn schema(&self) -> &[String] { self.child.schema() }
    fn init(&mut self) -> SqlResult<()> { self.child.init() }
}
