// Vectorization Tests
use sqlrustgo_executor::vectorization::{RecordBatch, Vector};

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
fn test_record_batch_new() {
    let batch = RecordBatch::new(10);
    assert_eq!(batch.num_rows(), 10);
    assert_eq!(batch.num_columns(), 0);
}

#[test]
fn test_record_batch_with_schema() {
    let mut batch = RecordBatch::new(10);
    let col1 = Vector::from_vec(vec![1; 10]);
    let col2 = Vector::from_vec(vec![2; 10]);
    batch.add_column(col1);
    batch.add_column(col2);
    batch.schema = vec!["col1".to_string(), "col2".to_string()];
    assert_eq!(batch.num_columns(), 2);
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
