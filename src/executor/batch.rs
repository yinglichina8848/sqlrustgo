//! Vectorized Execution: RecordBatch and Array
//!
//! # What (是什么)
//! RecordBatch 是列式内存布局的数据结构，用于向量化执行
//! Array 是列数据的抽象，支持批量处理
//!
//! # Why (为什么)
//! 行式处理每次只处理一行，效率较低
//! 列式处理一次处理多行，利用 CPU SIMD 指令，大幅提升性能
//!
//! # How (如何实现)
//! - RecordBatch 包含 Schema 和多个 Array
//! - Array trait 定义列操作的统一接口
//! - 每种数据类型实现对应的 Array

use crate::planner::Schema;
use crate::types::Value;
use std::sync::Arc;

/// Array reference type
pub type ArrayRef = Arc<dyn Array>;

/// Array trait for columnar data
///
/// # What
/// Array 是列数据的抽象接口，支持批量读取和操作
///
/// # Why
/// 统一不同数据类型的列操作接口，便于向量化执行
///
/// # How
/// - 实现 len() 返回列数据长度
/// - 实现 is_null() 判断空值
/// - 实现 get_value() 获取单个值
pub trait Array: Send + Sync {
    /// Get the length of the array
    fn len(&self) -> usize;

    /// Check if the array is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if value at index is null
    fn is_null(&self, index: usize) -> bool;

    /// Get value at index
    fn get_value(&self, index: usize) -> Option<Value>;

    /// Get the data type of this array
    fn get_data_type(&self) -> &str;
}

/// RecordBatch - Columnar memory layout for vectorized execution
///
/// # What
/// RecordBatch 是列式内存布局的数据结构，包含 Schema 和多个列数组
///
/// # Why
/// - 向量化执行：一次处理多行，利用 CPU SIMD
/// - 内存效率：同类型数据连续存储，压缩率高
/// - 缓存友好：顺序访问提高缓存命中率
///
/// # How
/// - schema: 表结构定义
/// - columns: 列数据数组
/// - row_count: 行数（所有列长度相同）
#[derive(Clone)]
pub struct RecordBatch {
    schema: Arc<Schema>,
    columns: Vec<ArrayRef>,
    row_count: usize,
}

impl RecordBatch {
    /// Create a new RecordBatch
    ///
    /// # Arguments
    /// * `schema` - Table schema
    /// * `columns` - Column arrays
    ///
    /// # Returns
    /// * `SqlResult<RecordBatch>` - RecordBatch or error
    ///
    /// # Errors
    /// Returns error if columns have different lengths
    pub fn new(schema: Arc<Schema>, columns: Vec<ArrayRef>) -> crate::types::SqlResult<Self> {
        if columns.is_empty() {
            return Ok(Self {
                schema,
                columns,
                row_count: 0,
            });
        }

        let row_count = columns[0].len();
        for (i, col) in columns.iter().enumerate() {
            if col.len() != row_count {
                return Err(crate::types::SqlError::ExecutionError(format!(
                    "Column {} has length {} but expected {}",
                    i,
                    col.len(),
                    row_count
                )));
            }
        }

        Ok(Self {
            schema,
            columns,
            row_count,
        })
    }

    /// Get the schema
    pub fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }

    /// Get the columns
    pub fn columns(&self) -> &[ArrayRef] {
        &self.columns
    }

    /// Get the number of rows
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    /// Get the number of columns
    pub fn num_columns(&self) -> usize {
        self.columns.len()
    }

    /// Get column by index
    pub fn column(&self, index: usize) -> Option<&ArrayRef> {
        self.columns.get(index)
    }

    /// Get column by name
    pub fn column_by_name(&self, name: &str) -> Option<&ArrayRef> {
        self.schema.field_index(name).and_then(|i| self.columns.get(i))
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.row_count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::{DataType, Field};
    use crate::types::Value;

    /// Helper to create a simple test array
    struct TestArray {
        data: Vec<Option<Value>>,
        data_type: String,
    }

    impl TestArray {
        fn new(data: Vec<Option<Value>>, data_type: &str) -> Self {
            Self {
                data,
                data_type: data_type.to_string(),
            }
        }
    }

    impl Array for TestArray {
        fn len(&self) -> usize {
            self.data.len()
        }

        fn is_null(&self, index: usize) -> bool {
            self.data.get(index).map_or(true, |v| v.is_none())
        }

        fn get_value(&self, index: usize) -> Option<Value> {
            self.data.get(index).and_then(|v| v.clone())
        }

        fn get_data_type(&self) -> &str {
            &self.data_type
        }
    }

    #[test]
    fn test_record_batch_creation() {
        let fields = vec![
            Field::new_not_null("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ];
        let schema = Arc::new(Schema::new(fields));

        let col1: ArrayRef = Arc::new(TestArray::new(
            vec![Some(Value::Integer(1)), Some(Value::Integer(2)), Some(Value::Integer(3))],
            "INTEGER",
        ));
        let col2: ArrayRef = Arc::new(TestArray::new(
            vec![
                Some(Value::Text("a".to_string())),
                Some(Value::Text("b".to_string())),
                Some(Value::Text("c".to_string())),
            ],
            "TEXT",
        ));

        let batch = RecordBatch::new(schema, vec![col1.clone(), col2.clone()]).unwrap();

        assert_eq!(batch.row_count(), 3);
        assert_eq!(batch.num_columns(), 2);
        assert_eq!(batch.column(0).unwrap().as_ref() as *const dyn Array, col1.as_ref() as *const dyn Array);
    }

    #[test]
    fn test_record_batch_empty() {
        let fields = vec![Field::new_not_null("id".to_string(), DataType::Integer)];
        let schema = Arc::new(Schema::new(fields));

        let batch = RecordBatch::new(schema, vec![]).unwrap();

        assert!(batch.is_empty());
        assert_eq!(batch.row_count(), 0);
        assert_eq!(batch.num_columns(), 0);
    }

    #[test]
    fn test_record_batch_column_mismatch() {
        let fields = vec![
            Field::new_not_null("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ];
        let schema = Arc::new(Schema::new(fields));

        let col1: ArrayRef = Arc::new(TestArray::new(
            vec![Some(Value::Integer(1)), Some(Value::Integer(2))],
            "INTEGER",
        ));
        let col2: ArrayRef = Arc::new(TestArray::new(
            vec![Some(Value::Text("a".to_string()))],
            "TEXT",
        ));

        let result = RecordBatch::new(schema, vec![col1, col2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_record_batch_column_by_name() {
        let fields = vec![
            Field::new_not_null("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ];
        let schema = Arc::new(Schema::new(fields));

        let col1: ArrayRef = Arc::new(TestArray::new(
            vec![Some(Value::Integer(1)), Some(Value::Integer(2))],
            "INTEGER",
        ));
        let col2: ArrayRef = Arc::new(TestArray::new(
            vec![
                Some(Value::Text("a".to_string())),
                Some(Value::Text("b".to_string())),
            ],
            "TEXT",
        ));

        let batch = RecordBatch::new(schema, vec![col1, col2]).unwrap();

        let id_col = batch.column_by_name("id").unwrap();
        assert_eq!(id_col.get_value(0), Some(Value::Integer(1)));

        let name_col = batch.column_by_name("name").unwrap();
        assert_eq!(name_col.get_value(0), Some(Value::Text("a".to_string())));
    }

    #[test]
    fn test_array_trait() {
        let arr: ArrayRef = Arc::new(TestArray::new(
            vec![
                Some(Value::Integer(1)),
                None,
                Some(Value::Integer(3)),
            ],
            "INTEGER",
        ));

        assert_eq!(arr.len(), 3);
        assert!(!arr.is_empty());
        assert!(!arr.is_null(0));
        assert!(arr.is_null(1));
        assert!(!arr.is_null(2));
        assert_eq!(arr.get_value(0), Some(Value::Integer(1)));
        assert_eq!(arr.get_value(1), None);
        assert_eq!(arr.get_data_type(), "INTEGER");
    }
}
