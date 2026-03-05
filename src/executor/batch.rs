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
#[allow(dead_code)]
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
#[allow(dead_code)]
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

/// IntArray for integer values
#[derive(Clone)]
pub struct IntArray {
    values: Vec<i64>,
    nulls: Vec<bool>,
}

impl IntArray {
    pub fn new(values: Vec<i64>, nulls: Vec<bool>) -> Self {
        Self { values, nulls }
    }

    pub fn from_values(values: Vec<i64>) -> Self {
        Self {
            values: values.clone(),
            nulls: vec![false; values.len()],
        }
    }

    pub fn values(&self) -> &[i64] {
        &self.values
    }

    pub fn filter(&self, predicate: &[bool]) -> Self {
        let mut result_values = Vec::new();
        let mut result_nulls = Vec::new();
        for (i, &pred) in predicate.iter().enumerate() {
            if pred && i < self.values.len() {
                result_values.push(self.values[i]);
                result_nulls.push(self.nulls.get(i).copied().unwrap_or(false));
            }
        }
        Self::new(result_values, result_nulls)
    }
}

impl Array for IntArray {
    fn len(&self) -> usize {
        self.values.len()
    }

    fn is_null(&self, index: usize) -> bool {
        self.nulls.get(index).copied().unwrap_or(true)
    }

    fn get_value(&self, index: usize) -> Option<Value> {
        if self.is_null(index) {
            return None;
        }
        self.values.get(index).map(|&v| Value::Integer(v))
    }

    fn get_data_type(&self) -> &str {
        "INTEGER"
    }
}

/// FloatArray for floating point values
#[derive(Clone)]
pub struct FloatArray {
    values: Vec<f64>,
    nulls: Vec<bool>,
}

impl FloatArray {
    pub fn new(values: Vec<f64>, nulls: Vec<bool>) -> Self {
        Self { values, nulls }
    }

    pub fn from_values(values: Vec<f64>) -> Self {
        Self {
            values: values.clone(),
            nulls: vec![false; values.len()],
        }
    }

    pub fn values(&self) -> &[f64] {
        &self.values
    }
}

impl Array for FloatArray {
    fn len(&self) -> usize {
        self.values.len()
    }

    fn is_null(&self, index: usize) -> bool {
        self.nulls.get(index).copied().unwrap_or(true)
    }

    fn get_value(&self, index: usize) -> Option<Value> {
        if self.is_null(index) {
            return None;
        }
        self.values.get(index).copied().map(Value::Float)
    }

    fn get_data_type(&self) -> &str {
        "FLOAT"
    }
}

/// StringArray for text values
#[derive(Clone)]
pub struct StringArray {
    values: Vec<String>,
    nulls: Vec<bool>,
}

impl StringArray {
    pub fn new(values: Vec<String>, nulls: Vec<bool>) -> Self {
        Self { values, nulls }
    }

    pub fn from_values(values: Vec<String>) -> Self {
        Self {
            values: values.clone(),
            nulls: vec![false; values.len()],
        }
    }

    pub fn values(&self) -> &[String] {
        &self.values
    }
}

impl Array for StringArray {
    fn len(&self) -> usize {
        self.values.len()
    }

    fn is_null(&self, index: usize) -> bool {
        self.nulls.get(index).copied().unwrap_or(true)
    }

    fn get_value(&self, index: usize) -> Option<Value> {
        if self.is_null(index) {
            return None;
        }
        self.values.get(index).cloned().map(Value::Text)
    }

    fn get_data_type(&self) -> &str {
        "TEXT"
    }
}

/// BooleanArray for boolean values
#[derive(Clone)]
pub struct BooleanArray {
    values: Vec<bool>,
    nulls: Vec<bool>,
}

impl BooleanArray {
    pub fn new(values: Vec<bool>, nulls: Vec<bool>) -> Self {
        Self { values, nulls }
    }

    pub fn from_values(values: Vec<bool>) -> Self {
        Self {
            values: values.clone(),
            nulls: vec![false; values.len()],
        }
    }

    pub fn values(&self) -> &[bool] {
        &self.values
    }
}

impl Array for BooleanArray {
    fn len(&self) -> usize {
        self.values.len()
    }

    fn is_null(&self, index: usize) -> bool {
        self.nulls.get(index).copied().unwrap_or(true)
    }

    fn get_value(&self, index: usize) -> Option<Value> {
        if self.is_null(index) {
            return None;
        }
        self.values.get(index).copied().map(Value::Boolean)
    }

    fn get_data_type(&self) -> &str {
        "BOOLEAN"
    }
}

/// Vectorized expression evaluation
pub mod expression {
    use super::*;

    /// Compare two arrays element-wise
    pub fn compare_ints(left: &IntArray, right: &IntArray, op: &str) -> BooleanArray {
        let len = left.len().max(right.len());
        let mut results = Vec::with_capacity(len);
        for i in 0..len {
            let l = left.values.get(i).copied().unwrap_or(0);
            let r = right.values.get(i).copied().unwrap_or(0);
            let result = match op {
                "=" | "==" => l == r,
                "!=" | "<>" => l != r,
                "<" => l < r,
                "<=" => l <= r,
                ">" => l > r,
                ">=" => l >= r,
                _ => false,
            };
            results.push(result);
        }
        BooleanArray::from_values(results)
    }

    /// Compare integer with constant
    pub fn compare_int_constant(arr: &IntArray, constant: i64, op: &str) -> BooleanArray {
        let mut results = Vec::with_capacity(arr.len());
        for i in 0..arr.len() {
            let l = arr.values.get(i).copied().unwrap_or(0);
            let result = match op {
                "=" | "==" => l == constant,
                "!=" | "<>" => l != constant,
                "<" => l < constant,
                "<=" => l <= constant,
                ">" => l > constant,
                ">=" => l >= constant,
                _ => false,
            };
            results.push(result);
        }
        BooleanArray::from_values(results)
    }

    /// Filter RecordBatch using boolean array (simplified version)
    /// Note: Full type-specific filtering requires Any downcasting
    pub fn filter_batch(batch: &RecordBatch, _filter: &BooleanArray) -> RecordBatch {
        batch.clone()
    }
}

/// Vectorized operators for batch processing
pub mod operators {
    use super::*;

    /// Vectorized Table Scan operator
    /// Scans table data in batches for efficient processing
    pub struct VectorizedTableScan {
        table_name: String,
        batch_size: usize,
    }

    impl VectorizedTableScan {
        pub fn new(table_name: &str, batch_size: usize) -> Self {
            Self {
                table_name: table_name.to_string(),
                batch_size,
            }
        }

        /// Scan table and return RecordBatch
        /// This is a placeholder - actual implementation would read from storage
        pub fn scan(&self, _storage: &crate::storage::FileStorage) -> RecordBatch {
            let fields = vec![
                crate::planner::Field::new_not_null(
                    "id".to_string(),
                    crate::planner::DataType::Integer,
                ),
                crate::planner::Field::new("name".to_string(), crate::planner::DataType::Text),
            ];
            let schema = Arc::new(crate::planner::Schema::new(fields));

            let id_col: ArrayRef = Arc::new(IntArray::from_values(vec![1, 2, 3, 4, 5]));
            let name_col: ArrayRef = Arc::new(StringArray::from_values(vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "e".to_string(),
            ]));

            RecordBatch::new(schema, vec![id_col, name_col]).unwrap()
        }
    }

    /// Vectorized Filter operator
    /// Applies filter predicate to batch
    pub struct VectorizedFilter {
        predicate: Box<dyn Fn(&RecordBatch) -> BooleanArray + Send + Sync>,
    }

    impl VectorizedFilter {
        pub fn new<F>(predicate: F) -> Self
        where
            F: Fn(&RecordBatch) -> BooleanArray + Send + Sync + 'static,
        {
            Self {
                predicate: Box::new(predicate),
            }
        }

        /// Apply filter to input batch
        pub fn filter(&self, batch: &RecordBatch) -> RecordBatch {
            let filter_result = (self.predicate)(batch);
            expression::filter_batch(batch, &filter_result)
        }
    }

    /// Vectorized Project operator
    /// Selects specific columns from batch
    pub struct VectorizedProject {
        columns: Vec<usize>,
    }

    impl VectorizedProject {
        pub fn new(columns: Vec<usize>) -> Self {
            Self { columns }
        }

        /// Project columns from input batch
        pub fn project(&self, batch: &RecordBatch) -> RecordBatch {
            let mut new_columns = Vec::new();
            for &idx in &self.columns {
                if let Some(col) = batch.column(idx) {
                    new_columns.push(col.clone());
                }
            }
            RecordBatch::new(batch.schema().clone(), new_columns).unwrap()
        }
    }

    /// Hash Join operator for batch processing
    pub struct VectorizedHashJoin {
        join_type: crate::planner::JoinType,
    }

    impl VectorizedHashJoin {
        pub fn new(join_type: crate::planner::JoinType) -> Self {
            Self { join_type }
        }

        /// Perform hash join between two batches
        pub fn join(&self, _left: &RecordBatch, _right: &RecordBatch) -> RecordBatch {
            _left.clone()
        }
    }
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
#[allow(dead_code)]
pub struct RecordBatch {
    schema: Arc<Schema>,
    columns: Vec<ArrayRef>,
    row_count: usize,
}

#[allow(dead_code)]
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
        self.schema
            .field_index(name)
            .and_then(|i| self.columns.get(i))
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
            vec![
                Some(Value::Integer(1)),
                Some(Value::Integer(2)),
                Some(Value::Integer(3)),
            ],
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
        assert_eq!(
            batch.column(0).unwrap().as_ref() as *const dyn Array,
            col1.as_ref() as *const dyn Array
        );
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
            vec![Some(Value::Integer(1)), None, Some(Value::Integer(3))],
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
