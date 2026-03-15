//! Vectorization Module
//!
//! Provides SIMD infrastructure and batch processing for vectorized execution.

#[allow(unused_imports)]
use std::ops::{Add, Div, Mul, Sub};

/// Vector data type for SIMD operations
#[derive(Debug, Clone)]
pub struct Vector<T> {
    data: Vec<T>,
}

impl<T> Vector<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn from_vec(data: Vec<T>) -> Self {
        Self { data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }
}

impl<T: Clone> Vector<T> {
    pub fn resize(&mut self, new_len: usize, value: T) {
        self.data.resize(new_len, value);
    }
}

/// RecordBatch - a batch of records for vectorized processing
#[derive(Debug, Clone)]
pub struct RecordBatch {
    columns: Vec<Vector<u8>>,
    num_rows: usize,
    schema: Vec<String>,
}

impl RecordBatch {
    pub fn new(num_rows: usize) -> Self {
        Self {
            columns: Vec::new(),
            num_rows,
            schema: Vec::new(),
        }
    }

    pub fn with_schema(mut self, schema: Vec<String>) -> Self {
        self.schema = schema;
        self
    }

    pub fn add_column(&mut self, column: Vector<u8>) {
        self.columns.push(column);
    }

    pub fn num_columns(&self) -> usize {
        self.columns.len()
    }

    pub fn num_rows(&self) -> usize {
        self.num_rows
    }

    pub fn get_column(&self, index: usize) -> Option<&Vector<u8>> {
        self.columns.get(index)
    }

    pub fn schema(&self) -> &[String] {
        &self.schema
    }
}

/// BatchIterator - trait for iterating over batches
pub trait BatchIterator {
    type Item;

    fn next_batch(&mut self, batch_size: usize) -> Option<Self::Item>;

    fn reset(&mut self);
}

/// VectorizedExecutor - trait for vectorized execution
pub trait VectorizedExecutor {
    fn execute_batch(&mut self, batch: &mut RecordBatch) -> usize;

    fn batch_size(&self) -> usize;

    fn set_batch_size(&mut self, size: usize);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_new() {
        let v: Vector<i32> = Vector::new(10);
        assert!(v.is_empty());
    }

    #[test]
    fn test_vector_from_vec() {
        let v = Vector::from_vec(vec![1, 2, 3]);
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn test_vector_push() {
        let mut v = Vector::new(0);
        v.push(1);
        v.push(2);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn test_vector_get() {
        let v = Vector::from_vec(vec![10, 20, 30]);
        assert_eq!(v.get(0), Some(&10));
        assert_eq!(v.get(2), Some(&30));
        assert_eq!(v.get(3), None);
    }

    #[test]
    fn test_record_batch_new() {
        let batch = RecordBatch::new(100);
        assert_eq!(batch.num_rows(), 100);
        assert_eq!(batch.num_columns(), 0);
    }

    #[test]
    fn test_record_batch_with_schema() {
        let batch = RecordBatch::new(10).with_schema(vec!["id".to_string(), "name".to_string()]);
        assert_eq!(batch.schema().len(), 2);
    }

    #[test]
    fn test_record_batch_add_column() {
        let mut batch = RecordBatch::new(5);
        batch.add_column(Vector::from_vec(vec![1, 2, 3, 4, 5]));
        assert_eq!(batch.num_columns(), 1);
    }

    #[test]
    fn test_record_batch_get_column() {
        let mut batch = RecordBatch::new(3);
        batch.add_column(Vector::from_vec(vec![10, 20, 30]));
        let col = batch.get_column(0);
        assert!(col.is_some());
        assert_eq!(col.unwrap().len(), 3);
    }

    #[test]
    fn test_vector_resize() {
        let mut v = Vector::from_vec(vec![1, 2]);
        v.resize(5, 0);
        assert_eq!(v.len(), 5);
    }
}
