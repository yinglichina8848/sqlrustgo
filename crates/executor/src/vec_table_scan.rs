use crate::executor::VolcanoExecutor;
use crate::vectorization::{DataChunk, ColumnArray};
use sqlrustgo_planner::{Expr, Schema};
use sqlrustgo_types::{SqlResult, Value};

pub struct VecTableScanExecutor {
    data: Vec<DataChunk>,
    batch_size: usize,
    current_chunk_idx: usize,
    chunk_position: usize,
    predicate: Option<Expr>,
    opened: bool,
    schema: Schema,
}

impl VecTableScanExecutor {
    pub fn new(data: Vec<DataChunk>, schema: Schema, batch_size: usize) -> Self {
        Self {
            data,
            batch_size,
            current_chunk_idx: 0,
            chunk_position: 0,
            predicate: None,
            opened: false,
            schema,
        }
    }

    pub fn from_chunk(chunk: DataChunk, schema: Schema, batch_size: usize) -> Self {
        Self {
            data: vec![chunk],
            batch_size,
            current_chunk_idx: 0,
            chunk_position: 0,
            predicate: None,
            opened: false,
            schema,
        }
    }

    pub fn with_predicate(mut self, predicate: Expr) -> Self {
        self.predicate = Some(predicate);
        self
    }

    pub fn next_batch(&mut self) -> SqlResult<Option<DataChunk>> {
        if !self.opened {
            return Err(sqlrustgo_types::SqlError::ExecutionError(
                "Executor not opened".to_string(),
            ));
        }

        if self.current_chunk_idx >= self.data.len() {
            return Ok(None);
        }

        if self.batch_size == 0 {
            return Ok(None);
        }

        let chunk = &self.data[self.current_chunk_idx];

        if chunk.is_empty() {
            self.current_chunk_idx += 1;
            return self.next_batch();
        }

        let result = chunk.clone();
        self.current_chunk_idx += 1;

        Ok(Some(result))
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn name(&self) -> &str {
        "VecTableScan"
    }

    pub fn total_rows(&self) -> usize {
        self.data.iter().map(|c: &DataChunk| c.num_rows()).sum()
    }

    pub fn num_chunks(&self) -> usize {
        self.data.len()
    }

    pub fn has_predicate(&self) -> bool {
        self.predicate.is_some()
    }

    fn row_to_values(&self, chunk: &DataChunk, row_idx: usize) -> Vec<Value> {
        let mut row = Vec::with_capacity(chunk.num_columns());

        for col_idx in 0..chunk.num_columns() {
            if let Some(col) = chunk.get_column(col_idx) {
                let value = match col {
                    ColumnArray::Int64(v) => v.get(row_idx).map(|&x| Value::Integer(x)),
                    ColumnArray::Float64(v) => v.get(row_idx).map(|&x| Value::Float(x)),
                    ColumnArray::Boolean(v) => v.get(row_idx).map(|&x| Value::Boolean(x)),
                    ColumnArray::Text(v) => v.get(row_idx).cloned().map(Value::Text),
                    ColumnArray::Null => Some(Value::Null),
                };
                row.push(value.unwrap_or(Value::Null));
            }
        }

        row
    }
}

impl VolcanoExecutor for VecTableScanExecutor {
    fn open(&mut self) -> SqlResult<()> {
        self.current_chunk_idx = 0;
        self.chunk_position = 0;
        self.opened = true;
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.opened {
            return Err(sqlrustgo_types::SqlError::ExecutionError(
                "Executor not opened".to_string(),
            ));
        }

        if self.current_chunk_idx >= self.data.len() {
            return Ok(None);
        }

        let current_chunk = &self.data[self.current_chunk_idx];

        if self.chunk_position >= current_chunk.num_rows() {
            self.current_chunk_idx += 1;
            self.chunk_position = 0;

            if self.current_chunk_idx >= self.data.len() {
                return Ok(None);
            }
        }

        let chunk = &self.data[self.current_chunk_idx];
        let row_idx = self.chunk_position;
        self.chunk_position += 1;

        let row = self.row_to_values(chunk, row_idx);
        Ok(Some(row))
    }

    fn close(&mut self) -> SqlResult<()> {
        self.opened = false;
        self.current_chunk_idx = 0;
        self.chunk_position = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_planner::{DataType, Field};

    fn create_test_schema() -> Schema {
        Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
            Field::new("value".to_string(), DataType::Float),
        ])
    }

    fn create_test_chunk(num_rows: usize) -> DataChunk {
        let mut chunk = DataChunk::new(num_rows).with_schema(vec![
            "id".to_string(),
            "name".to_string(),
            "value".to_string(),
        ]);

        let ids: Vec<i64> = (0..num_rows as i64).collect();
        let names: Vec<String> = (0..num_rows)
            .map(|i| format!("name_{}", i))
            .collect();
        let values: Vec<f64> = (0..num_rows as i64)
            .map(|i| i as f64 * 1.5)
            .collect();

        chunk.add_column(ColumnArray::Int64(ids));
        chunk.add_column(ColumnArray::Text(names));
        chunk.add_column(ColumnArray::Float64(values));

        chunk
    }

    #[test]
    fn test_vec_table_scan_new() {
        let chunk = create_test_chunk(10);
        let schema = create_test_schema();
        let executor = VecTableScanExecutor::from_chunk(chunk, schema, 10);

        assert_eq!(executor.name(), "VecTableScan");
        assert_eq!(executor.total_rows(), 10);
        assert_eq!(executor.num_chunks(), 1);
        assert!(!executor.has_predicate());
    }

    #[test]
    fn test_vec_table_scan_with_predicate() {
        let chunk = create_test_chunk(10);
        let schema = create_test_schema();
        let executor = VecTableScanExecutor::from_chunk(chunk, schema, 10)
            .with_predicate(Expr::Literal(Value::Boolean(true)));

        assert!(executor.has_predicate());
    }

    #[test]
    fn test_vec_table_scan_open_close() {
        let chunk = create_test_chunk(10);
        let schema = create_test_schema();
        let mut executor = VecTableScanExecutor::from_chunk(chunk, schema, 10);

        executor.open().unwrap();
        executor.close().unwrap();
    }

    #[test]
    fn test_vec_table_scan_next() {
        let chunk = create_test_chunk(5);
        let schema = create_test_schema();
        let mut executor = VecTableScanExecutor::from_chunk(chunk, schema, 5);

        executor.open().unwrap();

        let row1 = executor.next().unwrap();
        assert!(row1.is_some());
        let row1 = row1.unwrap();
        assert_eq!(row1.len(), 3);

        for _ in 0..4 {
            let row = executor.next().unwrap();
            assert!(row.is_some());
        }

        let row = executor.next().unwrap();
        assert!(row.is_none());

        executor.close().unwrap();
    }

    #[test]
    fn test_vec_table_scan_next_batch() {
        let chunk = create_test_chunk(10);
        let schema = create_test_schema();
        let mut executor = VecTableScanExecutor::from_chunk(chunk, schema, 10);

        executor.open().unwrap();

        let batch1 = executor.next_batch().unwrap();
        assert!(batch1.is_some());
        assert_eq!(batch1.unwrap().num_rows(), 10);

        let batch2 = executor.next_batch().unwrap();
        assert!(batch2.is_none());

        executor.close().unwrap();
    }

    #[test]
    fn test_vec_table_scan_multiple_chunks() {
        let chunk1 = create_test_chunk(5);
        let chunk2 = create_test_chunk(5);
        let schema = create_test_schema();
        let data = vec![chunk1, chunk2];

        let mut executor = VecTableScanExecutor::new(data, schema, 10);
        executor.open().unwrap();

        assert_eq!(executor.total_rows(), 10);
        assert_eq!(executor.num_chunks(), 2);

        let batch1 = executor.next_batch().unwrap();
        assert!(batch1.is_some());
        assert_eq!(batch1.unwrap().num_rows(), 5);

        let batch2 = executor.next_batch().unwrap();
        assert!(batch2.is_some());
        assert_eq!(batch2.unwrap().num_rows(), 5);

        let batch3 = executor.next_batch().unwrap();
        assert!(batch3.is_none());

        executor.close().unwrap();
    }

    #[test]
    fn test_vec_table_scan_row_values() {
        let chunk = create_test_chunk(3);
        let schema = create_test_schema();
        let mut executor = VecTableScanExecutor::from_chunk(chunk, schema, 3);

        executor.open().unwrap();

        let row = executor.next().unwrap().unwrap();
        assert_eq!(row.len(), 3);
        assert_eq!(row[0], Value::Integer(0));
        assert_eq!(row[1], Value::Text("name_0".to_string()));
        assert_eq!(row[2], Value::Float(0.0));

        let row = executor.next().unwrap().unwrap();
        assert_eq!(row[0], Value::Integer(1));
        assert_eq!(row[1], Value::Text("name_1".to_string()));
        assert_eq!(row[2], Value::Float(1.5));

        executor.close().unwrap();
    }

    #[test]
    fn test_vec_table_scan_empty() {
        let chunk = DataChunk::new(0).with_schema(vec![]);
        let schema = Schema::new(vec![]);
        let mut executor = VecTableScanExecutor::from_chunk(chunk, schema, 10);

        executor.open().unwrap();

        let batch = executor.next_batch().unwrap();
        assert!(batch.is_none());

        let row = executor.next().unwrap();
        assert!(row.is_none());

        executor.close().unwrap();
    }

    #[test]
    fn test_vec_table_scan_schema() {
        let chunk = create_test_chunk(10);
        let schema = create_test_schema();
        let executor = VecTableScanExecutor::from_chunk(chunk, schema, 10);

        let result_schema = executor.schema();
        assert_eq!(result_schema.fields.len(), 3);
        assert_eq!(result_schema.fields[0].name, "id");
        assert_eq!(result_schema.fields[1].name, "name");
        assert_eq!(result_schema.fields[2].name, "value");
    }
}
