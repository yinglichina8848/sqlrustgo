//! Vectorized Sequential Scan Executor
//!
//! Executes table scans using vectorized (columnar) operations for improved performance.

use sqlrustgo_storage::engine::VectorStorage;
use sqlrustgo_types::{SqlError, Value};

use crate::executor::ExecutorResult;
use crate::vectorization::{ColumnArray, DataChunk};

/// Vectorized Sequential Scan Executor
/// Uses columnar storage and vectorized operations for efficient scanning
pub struct VectorizedSeqScanExecutor {
    /// Table name to scan
    table_name: String,
    /// Column indices to project (empty = all columns)
    column_indices: Vec<usize>,
    /// Batch size for vectorized processing
    batch_size: usize,
    /// Storage to scan from
    storage: Box<dyn VectorStorage>,
    /// Current batch offset
    current_offset: usize,
    /// Total rows in the table
    total_rows: usize,
    /// Schema (column names)
    schema: Vec<String>,
}

impl VectorizedSeqScanExecutor {
    /// Create a new vectorized sequential scan executor
    pub fn new(
        table_name: String,
        storage: Box<dyn VectorStorage>,
        column_indices: Vec<usize>,
        batch_size: usize,
    ) -> Self {
        Self {
            table_name,
            column_indices,
            batch_size,
            storage,
            current_offset: 0,
            total_rows: 0,
            schema: Vec::new(),
        }
    }

    /// Initialize the executor - must be called before fetching batches
    pub fn init(&mut self) -> Result<(), SqlError> {
        let columns = self.storage.scan_vectorized(&self.table_name)?;

        if columns.is_empty() {
            self.total_rows = 0;
            self.schema = Vec::new();
            return Ok(());
        }

        // Get total rows from first column
        self.total_rows = columns[0].len();

        // Build schema from column data
        self.schema = if self.column_indices.is_empty() {
            (0..columns.len()).map(|i| format!("col_{}", i)).collect()
        } else {
            self.column_indices
                .iter()
                .map(|&i| format!("col_{}", i))
                .collect()
        };

        self.current_offset = 0;
        Ok(())
    }

    /// Get the next batch of data as a DataChunk
    pub fn next_batch(&mut self) -> Result<Option<DataChunk>, SqlError> {
        if self.current_offset >= self.total_rows {
            return Ok(None);
        }

        let batch_limit = self.batch_size.min(self.total_rows - self.current_offset);

        // Scan batch from storage
        let (columns, _, _) = self.storage.scan_vectorized_batch(
            &self.table_name,
            self.current_offset,
            batch_limit,
        )?;

        // Convert StorageColumnArray to ColumnArray
        let column_arrays: Vec<ColumnArray> = columns.into_iter().map(ColumnArray::from).collect();

        // Apply projection if column_indices is specified
        let projected_columns: Vec<ColumnArray> = if self.column_indices.is_empty() {
            column_arrays
        } else {
            self.column_indices
                .iter()
                .filter_map(|&idx| column_arrays.get(idx).cloned())
                .collect()
        };

        let mut chunk = DataChunk::new(batch_limit).with_schema(self.schema.clone());
        for col in projected_columns {
            chunk.add_column(col);
        }

        self.current_offset += batch_limit;
        Ok(Some(chunk))
    }

    /// Reset the executor to the beginning
    pub fn reset(&mut self) {
        self.current_offset = 0;
    }

    /// Get the total number of rows
    pub fn total_rows(&self) -> usize {
        self.total_rows
    }

    /// Get the schema (column names)
    pub fn schema(&self) -> &[String] {
        &self.schema
    }
}

/// Convert VectorizedSeqScanExecutor to row-oriented results
impl VectorizedSeqScanExecutor {
    /// Execute the scan and collect all results as ExecutorResult
    pub fn execute_collect(&mut self) -> Result<ExecutorResult, SqlError> {
        self.init()?;

        let mut all_rows: Vec<Vec<Value>> = Vec::new();

        while let Some(chunk) = self.next_batch()? {
            let chunk_rows = chunk.to_rows();
            all_rows.extend(chunk_rows);
        }

        let row_count = all_rows.len();
        Ok(ExecutorResult::new(all_rows, row_count))
    }
}

/// Iterator adapter for VolcanoExecutor compatibility
/// Wraps vectorized batch iteration in row-by-row interface
#[allow(dead_code)]
pub struct VectorizedSeqScanIterator {
    executor: VectorizedSeqScanExecutor,
    current_batch: Option<DataChunk>,
    batch_index: usize,
}

impl VectorizedSeqScanIterator {
    pub fn new(executor: VectorizedSeqScanExecutor) -> Self {
        Self {
            executor,
            current_batch: None,
            batch_index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_storage::engine::{MemoryStorage, StorageEngine};

    #[test]
    fn test_vectorized_seq_scan_empty_table() {
        let storage = Box::new(MemoryStorage::new());
        let mut executor =
            VectorizedSeqScanExecutor::new("empty_table".to_string(), storage, vec![], 1024);

        executor.init().unwrap();

        let batch = executor.next_batch().unwrap();
        assert!(batch.is_none());
    }

    #[test]
    fn test_vectorized_seq_scan_with_data() {
        let mut storage = MemoryStorage::new();

        // Insert test data
        let records = vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string())],
            vec![Value::Integer(2), Value::Text("Bob".to_string())],
            vec![Value::Integer(3), Value::Text("Charlie".to_string())],
        ];
        storage.insert("users", records).unwrap();

        let storage = Box::new(storage);
        let mut executor =
            VectorizedSeqScanExecutor::new("users".to_string(), storage, vec![], 1024);

        let result = executor.execute_collect().unwrap();
        assert_eq!(result.rows.len(), 3);
        assert_eq!(result.rows[0][0], Value::Integer(1));
    }

    #[test]
    fn test_vectorized_seq_scan_projection() {
        let mut storage = MemoryStorage::new();

        let records = vec![
            vec![
                Value::Integer(1),
                Value::Text("Alice".to_string()),
                Value::Integer(25),
            ],
            vec![
                Value::Integer(2),
                Value::Text("Bob".to_string()),
                Value::Integer(30),
            ],
        ];
        storage.insert("users", records).unwrap();

        let storage = Box::new(storage);
        let mut executor = VectorizedSeqScanExecutor::new(
            "users".to_string(),
            storage,
            vec![0, 2], // Project only id and age columns
            1024,
        );

        let result = executor.execute_collect().unwrap();
        assert_eq!(result.rows.len(), 2);
        // Each row should have 2 columns (id and age, not name)
        assert_eq!(result.rows[0].len(), 2);
    }

    #[test]
    fn test_vectorized_seq_scan_batch_processing() {
        let mut storage = MemoryStorage::new();

        // Create 100 rows
        let records: Vec<Vec<Value>> = (0..100).map(|i| vec![Value::Integer(i as i64)]).collect();
        storage.insert("numbers", records).unwrap();

        let storage = Box::new(storage);
        let mut executor = VectorizedSeqScanExecutor::new(
            "numbers".to_string(),
            storage,
            vec![],
            30, // Batch size of 30
        );

        executor.init().unwrap();
        assert_eq!(executor.total_rows(), 100);

        // First batch
        let batch1 = executor.next_batch().unwrap();
        assert!(batch1.is_some());
        let batch1 = batch1.unwrap();
        assert_eq!(batch1.num_rows(), 30);

        // Second batch
        let batch2 = executor.next_batch().unwrap();
        assert!(batch2.is_some());
        let batch2 = batch2.unwrap();
        assert_eq!(batch2.num_rows(), 30);

        // Third batch (should have 30 rows to complete 90)
        let batch3 = executor.next_batch().unwrap();
        assert!(batch3.is_some());
        let batch3 = batch3.unwrap();
        assert_eq!(batch3.num_rows(), 30);

        // Fourth batch (should have 10 rows to complete 100)
        let batch4 = executor.next_batch().unwrap();
        assert!(batch4.is_some());
        let batch4 = batch4.unwrap();
        assert_eq!(batch4.num_rows(), 10);

        // Fifth batch should be None
        let batch5 = executor.next_batch().unwrap();
        assert!(batch5.is_none());
    }

    #[test]
    fn test_vectorized_seq_scan_reset() {
        let mut storage = MemoryStorage::new();

        let records = vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
        ];
        storage.insert("numbers", records).unwrap();

        let storage = Box::new(storage);
        let mut executor =
            VectorizedSeqScanExecutor::new("numbers".to_string(), storage, vec![], 2);

        executor.init().unwrap();

        // First batch
        let batch1 = executor.next_batch().unwrap().unwrap();
        assert_eq!(batch1.num_rows(), 2);

        // Reset
        executor.reset();

        // After reset, should get first batch again
        let batch2 = executor.next_batch().unwrap().unwrap();
        assert_eq!(batch2.num_rows(), 2);
    }
}
