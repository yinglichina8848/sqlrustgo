//! TableScan Executor
use crate::executor::executor::{Executor, RecordBatch};
use crate::storage::FileStorage;
use crate::types::SqlResult;
use std::sync::Arc;

pub struct TableScanExecutor {
    table_name: String,
    columns: Vec<String>,
    storage: Arc<FileStorage>,
    current_row: usize,
    batch_size: usize,
    table_data: Option<crate::executor::TableData>,
}

impl TableScanExecutor {
    pub fn new(table_name: String, storage: Arc<FileStorage>) -> Self {
        Self { table_name, columns: vec![], storage, current_row: 0, batch_size: 1024, table_data: None }
    }
    pub fn with_columns(mut self, columns: Vec<String>) -> Self { self.columns = columns; self }
    pub fn with_batch_size(mut self, batch_size: usize) -> Self { self.batch_size = batch_size; self }
    
    fn load_table(&mut self) -> SqlResult<()> {
        if self.table_data.is_none() {
            self.table_data = self.storage.get_table(&self.table_name).cloned();
            if self.table_data.is_none() {
                return Err(crate::types::SqlError::TableNotFound(self.table_name.clone()));
            }
        }
        Ok(())
    }
    fn output_columns(&self) -> Vec<String> {
        if let Some(ref data) = self.table_data {
            if self.columns.is_empty() { data.info.columns.iter().map(|c| c.name.clone()).collect() }
            else { self.columns.clone() }
        } else { vec![] }
    }
    fn column_indices(&self) -> Vec<usize> {
        if let Some(ref data) = self.table_data {
            let table_cols: Vec<String> = data.info.columns.iter().map(|c| c.name.clone()).collect();
            if self.columns.is_empty() { (0..table_cols.len()).collect() }
            else { self.columns.iter().filter_map(|c| table_cols.iter().position(|tc| tc == c)).collect() }
        } else { vec![] }
    }
}

impl Executor for TableScanExecutor {
    fn next(&mut self) -> SqlResult<Option<RecordBatch>> {
        self.load_table()?;
        let data = match &self.table_data { Some(d) => d, None => return Ok(None) };
        if self.current_row >= data.rows.len() { return Ok(None); }
        let indices = self.column_indices();
        let columns = self.output_columns();
        let mut batch_rows = Vec::with_capacity(self.batch_size);
        while self.current_row < data.rows.len() && batch_rows.len() < self.batch_size {
            let row = &data.rows[self.current_row];
            let projected: Vec<crate::types::Value> = indices.iter().filter_map(|&idx| row.get(idx).cloned()).collect();
            batch_rows.push(projected);
            self.current_row += 1;
        }
        Ok(Some(RecordBatch::new(columns, batch_rows)))
    }
    fn schema(&self) -> &[String] { &[] }
    fn init(&mut self) -> SqlResult<()> { self.load_table()?; self.current_row = 0; Ok(()) }
}
