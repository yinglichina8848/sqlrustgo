// Types Value Tests
use sqlrustgo_types::Value;

#[test]
fn test_value_integer_as_integer() {
    let v = Value::Integer(42);
    assert_eq!(v.as_integer(), Some(42));

    let v = Value::Text("hello".to_string());
    assert_eq!(v.as_integer(), None);
}

#[test]
fn test_value_to_bool() {
    assert_eq!(Value::Boolean(true).to_bool(), true);
    assert_eq!(Value::Boolean(false).to_bool(), false);
    assert_eq!(Value::Integer(0).to_bool(), false);
    assert_eq!(Value::Integer(1).to_bool(), true);
    assert_eq!(Value::Integer(-1).to_bool(), true);
    assert_eq!(Value::Null.to_bool(), false);
    assert_eq!(Value::Text("hello".to_string()).to_bool(), false);
}

#[test]
fn test_value_to_sql_string() {
    assert_eq!(Value::Null.to_sql_string(), "NULL");
    assert_eq!(Value::Boolean(true).to_sql_string(), "true");
    assert_eq!(Value::Boolean(false).to_sql_string(), "false");
    assert_eq!(Value::Integer(42).to_sql_string(), "42");
    assert_eq!(Value::Float(3.14).to_sql_string(), "3.14");
    assert_eq!(Value::Text("hello".to_string()).to_sql_string(), "hello");
    assert_eq!(Value::Blob(vec![1, 2, 3]).to_sql_string(), "X'010203'");
    assert_eq!(Value::Date(0).to_sql_string(), "0");
    assert_eq!(Value::Timestamp(0).to_sql_string(), "0");
}

#[test]
fn test_value_type_name() {
    assert_eq!(Value::Null.type_name(), "NULL");
    assert_eq!(Value::Boolean(true).type_name(), "BOOLEAN");
    assert_eq!(Value::Integer(1).type_name(), "INTEGER");
    assert_eq!(Value::Float(1.0).type_name(), "FLOAT");
    assert_eq!(Value::Text("".to_string()).type_name(), "TEXT");
    assert_eq!(Value::Blob(vec![]).type_name(), "BLOB");
    assert_eq!(Value::Date(0).type_name(), "DATE");
    assert_eq!(Value::Timestamp(0).type_name(), "TIMESTAMP");
}

#[test]
fn test_value_to_index_key() {
    assert_eq!(Value::Integer(42).to_index_key(), Some(42));
    // Text returns a hash value, not None
    let text_key = Value::Text("hello".to_string()).to_index_key();
    assert!(text_key.is_some());
    assert_eq!(Value::Null.to_index_key(), None);
}

#[test]
fn test_value_estimate_memory_size() {
    assert_eq!(Value::Null.estimate_memory_size(), 0);
    assert_eq!(Value::Boolean(true).estimate_memory_size(), 1);
    assert_eq!(Value::Integer(42).estimate_memory_size(), 8);
    assert_eq!(Value::Float(3.14).estimate_memory_size(), 8);
    assert_eq!(Value::Text("hello".to_string()).estimate_memory_size(), 5);
    assert_eq!(Value::Blob(vec![1, 2, 3]).estimate_memory_size(), 3);
    assert_eq!(Value::Date(0).estimate_memory_size(), 4);
    assert_eq!(Value::Timestamp(0).estimate_memory_size(), 8);
}

#[test]
fn test_value_timestamp_creation() {
    let ts = Value::timestamp(1000000);
    assert_eq!(ts, Value::Timestamp(1000000));
    assert_eq!(ts.as_timestamp(), Some(1000000));
}

#[test]
fn test_value_timestamp_to_string() {
    let ts = Value::Timestamp(0);
    assert_eq!(
        ts.timestamp_to_string(),
        Some("1970-01-01 00:00:00".to_string())
    );

    let ts = Value::Timestamp(1000000);
    assert_eq!(
        ts.timestamp_to_string(),
        Some("1970-01-01 00:00:01".to_string())
    );

    let not_ts = Value::Integer(42);
    assert_eq!(not_ts.timestamp_to_string(), None);
}

#[test]
fn test_value_date_creation() {
    let d = Value::Date(0);
    assert_eq!(d, Value::Date(0));

    let d = Value::Date(1);
    assert_eq!(d, Value::Date(1));
}

#[test]
fn test_value_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(Value::Integer(1));
    set.insert(Value::Integer(1));
    set.insert(Value::Integer(2));
    assert_eq!(set.len(), 2);

    let mut set = HashSet::new();
    set.insert(Value::Text("hello".to_string()));
    set.insert(Value::Text("hello".to_string()));
    assert_eq!(set.len(), 1);
}

#[test]
fn test_value_equality() {
    assert_eq!(Value::Null, Value::Null);
    assert_eq!(Value::Boolean(true), Value::Boolean(true));
    assert_eq!(Value::Integer(42), Value::Integer(42));
    assert_eq!(Value::Float(3.14), Value::Float(3.14));
    assert_eq!(
        Value::Text("hello".to_string()),
        Value::Text("hello".to_string())
    );
    assert_eq!(Value::Blob(vec![1, 2, 3]), Value::Blob(vec![1, 2, 3]));
    assert_eq!(Value::Date(0), Value::Date(0));
    assert_eq!(Value::Timestamp(0), Value::Timestamp(0));

    assert_ne!(Value::Integer(1), Value::Integer(2));
    assert_ne!(Value::Text("a".to_string()), Value::Text("b".to_string()));
}

#[test]
fn test_value_clone() {
    let v1 = Value::Text("hello".to_string());
    let v2 = v1.clone();
    assert_eq!(v1, v2);
}

#[test]
fn test_value_debug() {
    let debug_str = format!("{:?}", Value::Integer(42));
    assert!(debug_str.contains("Integer"));
}
