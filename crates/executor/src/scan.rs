use sqlrustgo_planner::Schema;
use sqlrustgo_storage::predicate::Predicate;
use sqlrustgo_types::{SqlResult, Value};

pub trait ScanExecutor: Send {
    fn init(&mut self) -> SqlResult<()>;
    fn next(&mut self) -> SqlResult<Option<Vec<Value>>>;
    fn schema(&self) -> &Schema;
    fn name(&self) -> &str;
    fn close(&mut self) -> SqlResult<()>;
}

#[derive(Debug, Clone, Default)]
pub struct ScanStats {
    pub rows_scanned: u64,
    pub rows_returned: u64,
    pub used_index: bool,
}

pub trait IndexScanable {
    fn can_use_index(&self, predicate: &Predicate) -> bool;
    fn estimate_index_cost(&self, predicate: &Predicate) -> f64;
    fn estimate_seq_cost(&self) -> f64;
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_planner::Field;
    use sqlrustgo_types::Value;

    struct MockScanExecutor {
        data: Vec<Vec<Value>>,
        position: usize,
        schema: Schema,
        closed: bool,
    }

    impl MockScanExecutor {
        fn new(data: Vec<Vec<Value>>) -> Self {
            Self {
                data,
                position: 0,
                schema: Schema::empty(),
                closed: false,
            }
        }

        fn with_schema(mut self, schema: Schema) -> Self {
            self.schema = schema;
            self
        }
    }

    impl ScanExecutor for MockScanExecutor {
        fn init(&mut self) -> SqlResult<()> {
            self.position = 0;
            self.closed = false;
            Ok(())
        }

        fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
            if self.closed {
                return Err(sqlrustgo_types::SqlError::ExecutionError(
                    "Executor is closed".to_string(),
                ));
            }
            if self.position >= self.data.len() {
                return Ok(None);
            }
            let row = self.data[self.position].clone();
            self.position += 1;
            Ok(Some(row))
        }

        fn schema(&self) -> &Schema {
            &self.schema
        }

        fn name(&self) -> &str {
            "MockScan"
        }

        fn close(&mut self) -> SqlResult<()> {
            self.closed = true;
            self.position = 0;
            Ok(())
        }
    }

    #[test]
    fn test_scan_executor_empty_table() {
        let mut executor = MockScanExecutor::new(vec![]);
        executor.init().unwrap();
        let result = executor.next().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_scan_executor_single_row() {
        let data = vec![vec![Value::Integer(1)]];
        let mut executor = MockScanExecutor::new(data);
        executor.init().unwrap();
        let result = executor.next().unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap()[0], Value::Integer(1));
    }

    #[test]
    fn test_scan_executor_multiple_rows() {
        let data = vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
        ];
        let mut executor = MockScanExecutor::new(data);
        executor.init().unwrap();

        let r1 = executor.next().unwrap();
        let r2 = executor.next().unwrap();
        let r3 = executor.next().unwrap();
        let r4 = executor.next().unwrap();

        assert!(r1.is_some() && r2.is_some() && r3.is_some());
        assert!(r4.is_none());
    }

    #[test]
    fn test_scan_executor_double_next_stability() {
        // 验证稳定性: 消费完后再 next() 应返回 None
        let data = vec![vec![Value::Integer(1)]];
        let mut executor = MockScanExecutor::new(data);
        executor.init().unwrap();

        executor.next().unwrap();
        executor.next().unwrap();
        executor.next().unwrap();
        let result = executor.next().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_scan_stats_default() {
        let stats = ScanStats::default();
        assert_eq!(stats.rows_scanned, 0);
        assert_eq!(stats.rows_returned, 0);
        assert!(!stats.used_index);
    }

    #[test]
    fn test_scan_stats_clone() {
        let stats = ScanStats {
            rows_scanned: 100,
            rows_returned: 50,
            used_index: true,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.rows_scanned, 100);
        assert_eq!(cloned.rows_returned, 50);
        assert!(cloned.used_index);
    }

    #[test]
    fn test_scan_executor_with_null_values() {
        let data = vec![vec![Value::Null], vec![Value::Integer(1)]];
        let mut executor = MockScanExecutor::new(data);
        executor.init().unwrap();
        let r1 = executor.next().unwrap();
        assert!(matches!(r1.unwrap()[0], Value::Null));
    }

    #[test]
    fn test_scan_executor_with_text_values() {
        let data = vec![
            vec![Value::Text("hello".to_string())],
            vec![Value::Text("world".to_string())],
        ];
        let mut executor = MockScanExecutor::new(data);
        executor.init().unwrap();
        let result = executor.next().unwrap();
        assert!(matches!(result.unwrap()[0], Value::Text(_)));
    }

    #[test]
    fn test_scan_executor_with_float_values() {
        let data = vec![vec![Value::Float(3.14)], vec![Value::Float(-2.5)]];
        let mut executor = MockScanExecutor::new(data);
        executor.init().unwrap();
        let r1 = executor.next().unwrap();
        assert!(matches!(r1.unwrap()[0], Value::Float(_)));
    }

    #[test]
    fn test_scan_executor_with_blob_values() {
        let data = vec![vec![Value::Blob(vec![0xDE, 0xAD, 0xBE, 0xEF])]];
        let mut executor = MockScanExecutor::new(data);
        executor.init().unwrap();
        let result = executor.next().unwrap();
        assert!(matches!(result.unwrap()[0], Value::Blob(_)));
    }

    #[test]
    fn test_scan_executor_with_boolean_values() {
        let data = vec![vec![Value::Boolean(true)], vec![Value::Boolean(false)]];
        let mut executor = MockScanExecutor::new(data);
        executor.init().unwrap();
        let r1 = executor.next().unwrap();
        assert!(matches!(r1.unwrap()[0], Value::Boolean(true)));
    }

    #[test]
    fn test_scan_executor_close() {
        let data = vec![vec![Value::Integer(1)]];
        let mut executor = MockScanExecutor::new(data);
        executor.init().unwrap();
        executor.close().unwrap();
        // After close, next() should return error
        let result = executor.next();
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_executor_name() {
        let executor = MockScanExecutor::new(vec![]);
        assert_eq!(executor.name(), "MockScan");
    }

    #[test]
    fn test_scan_executor_schema() {
        let fields = vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ];
        let schema = Schema::new(fields);
        let executor = MockScanExecutor::new(vec![]).with_schema(schema);
        assert_eq!(executor.schema().fields.len(), 2);
    }

    #[test]
    fn test_scan_executor_mixed_types() {
        let data = vec![vec![
            Value::Integer(1),
            Value::Text("test".to_string()),
            Value::Float(3.14),
            Value::Boolean(true),
            Value::Null,
        ]];
        let mut executor = MockScanExecutor::new(data);
        executor.init().unwrap();
        let result = executor.next().unwrap().unwrap();
        assert_eq!(result.len(), 5);
        assert!(matches!(result[0], Value::Integer(1)));
        assert!(matches!(result[1], Value::Text(_)));
        assert!(matches!(result[2], Value::Float(_)));
        assert!(matches!(result[3], Value::Boolean(true)));
        assert!(matches!(result[4], Value::Null));
    }

    #[test]
    fn test_scan_stats_debug() {
        let stats = ScanStats {
            rows_scanned: 1000,
            rows_returned: 500,
            used_index: true,
        };
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("rows_scanned"));
    }
}
