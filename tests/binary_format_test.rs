// Binary Format Tests
use sqlrustgo_storage::binary_format::{helpers, BinaryFormatError};
use std::f64::consts::PI;

#[test]
fn test_helpers_write_f64() {
    let bytes = helpers::write_f64(PI);
    assert_eq!(bytes.len(), 8);
}

#[test]
fn test_helpers_read_f64() {
    let bytes = helpers::write_f64(PI);
    let result = helpers::read_f64(&bytes);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PI);
}

#[test]
fn test_binary_format_error_debug() {
    let err = BinaryFormatError::InsufficientData;
    assert!(format!("{:?}", err).contains("InsufficientData"));
}

#[test]
fn test_helpers_write_u64() {
    let bytes = helpers::write_u64(42);
    assert_eq!(bytes.len(), 8);
}

#[test]
fn test_helpers_read_u64() {
    let bytes = helpers::write_u64(42);
    let result = helpers::read_u64(&bytes);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_helpers_read_u64_insufficient_data() {
    let result = helpers::read_u64(&[1, 2, 3]);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        BinaryFormatError::InsufficientData
    ));
}

#[test]
fn test_helpers_write_i64() {
    let bytes = helpers::write_i64(-42);
    assert_eq!(bytes.len(), 8);
}

#[test]
fn test_helpers_read_i64() {
    let bytes = helpers::write_i64(-42);
    let result = helpers::read_i64(&bytes);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), -42);
}

#[test]
fn test_helpers_read_i64_insufficient_data() {
    let result = helpers::read_i64(&[1, 2, 3]);
    assert!(result.is_err());
}

#[test]
fn test_helpers_write_string() {
    let bytes = helpers::write_string("hello");
    assert!(!bytes.is_empty());
}

#[test]
fn test_helpers_read_string() {
    let bytes = helpers::write_string("hello");
    let result = helpers::read_string(&bytes);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "hello");
}
