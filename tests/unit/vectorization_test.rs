// Vectorization Tests
use sqlrustgo_executor::vectorization::{ColumnArray, DataChunk, RecordBatch, Vector};

#[test]
fn test_vector_new() {
    let v: Vector<i32> = Vector::new(10);
    assert!(v.is_empty());
    assert_eq!(v.len(), 0);
}

#[test]
fn test_vector_from_vec() {
    let v = Vector::from_vec(vec![1, 2, 3]);
    assert_eq!(v.len(), 3);
}

#[test]
fn test_vector_push() {
    let mut v: Vector<i32> = Vector::new(10);
    v.push(1);
    v.push(2);
    assert_eq!(v.len(), 2);
}

#[test]
fn test_vector_get() {
    let v = Vector::from_vec(vec![1, 2, 3]);
    assert_eq!(v.get(0), Some(&1));
    assert_eq!(v.get(5), None);
}

#[test]
fn test_vector_iter() {
    let v = Vector::from_vec(vec![1, 2, 3]);
    let collected: Vec<_> = v.iter().collect();
    assert_eq!(collected, vec![&1, &2, &3]);
}

#[test]
fn test_vector_resize() {
    let mut v = Vector::from_vec(vec![1, 2]);
    v.resize(4, 0);
    assert_eq!(v.len(), 4);
}

#[test]
fn test_vector_as_slice() {
    let v = Vector::from_vec(vec![1, 2, 3]);
    assert_eq!(v.as_slice(), &[1, 2, 3]);
}

#[test]
fn test_vector_as_mut_slice() {
    let mut v = Vector::from_vec(vec![1, 2, 3]);
    let slice = v.as_mut_slice();
    slice[0] = 10;
    assert_eq!(v.as_slice(), &[10, 2, 3]);
}

#[test]
fn test_vector_fill() {
    let mut v: Vector<i32> = Vector::new(3);
    v.resize(3, 0);
    v.fill(5);
    assert_eq!(v.as_slice(), &[5, 5, 5]);
}

#[test]
fn test_column_array_new_int64() {
    let arr = ColumnArray::new_int64(10);
    assert!(arr.is_empty());
    assert_eq!(arr.len(), 0);
}

#[test]
fn test_column_array_new_float64() {
    let arr = ColumnArray::new_float64(10);
    assert!(arr.is_empty());
}

#[test]
fn test_column_array_new_boolean() {
    let arr = ColumnArray::new_boolean(10);
    assert!(arr.is_empty());
}

#[test]
fn test_column_array_new_text() {
    let arr = ColumnArray::new_text(10);
    assert!(arr.is_empty());
}

#[test]
fn test_column_array_push_int64() {
    let mut arr = ColumnArray::new_int64(10);
    arr.push_int64(42);
    arr.push_int64(100);
    assert_eq!(arr.len(), 2);
}

#[test]
fn test_column_array_push_float64() {
    let mut arr = ColumnArray::new_float64(10);
    arr.push_float64(3.14);
    assert_eq!(arr.len(), 1);
}

#[test]
fn test_column_array_push_boolean() {
    let mut arr = ColumnArray::new_boolean(10);
    arr.push_boolean(true);
    arr.push_boolean(false);
    assert_eq!(arr.len(), 2);
}

#[test]
fn test_column_array_push_text() {
    let mut arr = ColumnArray::new_text(10);
    arr.push_text("hello".to_string());
    assert_eq!(arr.len(), 1);
}

#[test]
fn test_column_array_null() {
    let arr = ColumnArray::Null;
    assert!(arr.is_empty());
    assert_eq!(arr.len(), 0);
}

#[test]
fn test_record_batch_new() {
    let batch = RecordBatch::new(10);
    assert_eq!(batch.num_rows(), 10);
    assert_eq!(batch.num_columns(), 0);
}

#[test]
fn test_record_batch_with_schema() {
    let batch = RecordBatch::new(10).with_schema(vec!["col1".to_string(), "col2".to_string()]);
    assert_eq!(batch.schema(), &["col1", "col2"]);
}

#[test]
fn test_record_batch_add_column() {
    let mut batch = RecordBatch::new(5);
    let col = Vector::from_vec(vec![1, 2, 3, 4, 5]);
    batch.add_column(col);
    assert_eq!(batch.num_columns(), 1);
}

#[test]
fn test_record_batch_get_column() {
    let mut batch = RecordBatch::new(5);
    let col = Vector::from_vec(vec![1, 2, 3, 4, 5]);
    batch.add_column(col);
    assert!(batch.get_column(0).is_some());
    assert!(batch.get_column(1).is_none());
}

#[test]
fn test_record_batch_num_rows() {
    let batch = RecordBatch::new(100);
    assert_eq!(batch.num_rows(), 100);
}

#[test]
fn test_record_batch_debug() {
    let batch = RecordBatch::new(5);
    let debug_str = format!("{:?}", batch);
    assert!(debug_str.contains("RecordBatch"));
}

#[test]
fn test_data_chunk_new() {
    let chunk = DataChunk::new(10);
    assert_eq!(chunk.num_rows(), 10);
    assert_eq!(chunk.num_columns(), 0);
}

#[test]
fn test_data_chunk_with_capacity() {
    let chunk = DataChunk::new(5);
    assert_eq!(chunk.num_rows(), 5);
    assert_eq!(chunk.num_columns(), 0);
}

#[test]
fn test_data_chunk_add_column() {
    let mut chunk = DataChunk::new(5);
    let col = ColumnArray::new_int64(5);
    chunk.add_column(col);
    assert_eq!(chunk.num_columns(), 1);
}

#[test]
fn test_data_chunk_debug() {
    let chunk = DataChunk::new(5);
    let debug_str = format!("{:?}", chunk);
    assert!(debug_str.contains("DataChunk"));
}
