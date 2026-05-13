//! Integration tests for row_format encode/decode roundtrip.
//!
//! These tests verify the full roundtrip functionality of the Compact Row v1 ABI.

use crate::row_format::encoder::encode_row;
use crate::row_format::decoder::decode_row;
use crate::row_format::types::{ClusterKey, RowHeader};
use sqlrustgo_types::Value;

/// Test roundtrip for integer value.
#[test]
fn test_roundtrip_integer() {
    let cluster_key = ClusterKey::HiddenRowId(1);
    let fixed_columns = vec![Value::Integer(42)];
    let varlen_columns: Vec<Option<Vec<u8>>> = vec![];
    let null_bitmap = vec![false];

    // Encode
    let encoded = encode_row(&cluster_key, &fixed_columns, &varlen_columns, &null_bitmap).unwrap();
    assert!(!encoded.is_empty());

    // Decode
    let (decoded_key, decoded_fixed, decoded_varlen, decoded_nulls) =
        decode_row(&encoded, 1, 0).unwrap();

    // Verify
    assert_eq!(decoded_key, cluster_key);
    assert_eq!(decoded_fixed[0], Value::Integer(42));
    assert_eq!(decoded_varlen.len(), 0);
    assert_eq!(decoded_nulls, null_bitmap);
}

/// Test roundtrip for all Value types.
#[test]
fn test_roundtrip_all_types() {
    let cluster_key = ClusterKey::HiddenRowId(1);

    // Test all Value types: Null, Boolean, Integer, Float, Text, Blob
    let fixed_columns = vec![
        Value::Null,
        Value::Boolean(true),
        Value::Integer(12345),
        Value::Float(3.14159),
        Value::Text("hello world".to_string()),
        Value::Blob(vec![0xDE, 0xAD, 0xBE, 0xEF]),
    ];
    let varlen_columns: Vec<Option<Vec<u8>>> = vec![];
    let null_bitmap = vec![false; 6]; // No NULLs

    // Encode
    let encoded = encode_row(&cluster_key, &fixed_columns, &varlen_columns, &null_bitmap).unwrap();
    assert!(!encoded.is_empty());

    // Decode
    let (decoded_key, decoded_fixed, decoded_varlen, _) =
        decode_row(&encoded, 6, 0).unwrap();

    // Verify all types
    assert_eq!(decoded_key, cluster_key);
    assert_eq!(decoded_fixed[0], Value::Null);
    assert_eq!(decoded_fixed[1], Value::Boolean(true));
    assert_eq!(decoded_fixed[2], Value::Integer(12345));
    assert_eq!(decoded_fixed[3], Value::Float(3.14159));
    assert_eq!(decoded_fixed[4], Value::Text("hello world".to_string()));
    assert_eq!(decoded_fixed[5], Value::Blob(vec![0xDE, 0xAD, 0xBE, 0xEF]));
}

/// Test roundtrip for varlen inline vs overflow threshold.
/// 100 bytes = inline (<= 128 threshold)
/// 200 bytes = overflow (> 128 threshold)
#[test]
fn test_roundtrip_varlen_inline_threshold() {
    let cluster_key = ClusterKey::HiddenRowId(1);
    let fixed_columns = vec![Value::Integer(1)];
    let null_bitmap = vec![false, false];

    // Test 100 bytes (inline)
    let data_100 = vec![0xAB; 100];
    let varlen_columns: Vec<Option<Vec<u8>>> = vec![Some(data_100.clone())];
    let encoded = encode_row(&cluster_key, &fixed_columns, &varlen_columns, &null_bitmap).unwrap();
    let (_, _, decoded_varlen, _) = decode_row(&encoded, 1, 1).unwrap();
    assert_eq!(decoded_varlen[0], Some(data_100));

    // Test 200 bytes (overflow - stores overflow_len only, inline data is None)
    let data_200 = vec![0xCD; 200];
    let varlen_columns: Vec<Option<Vec<u8>>> = vec![Some(data_200.clone())];
    let encoded = encode_row(&cluster_key, &fixed_columns, &varlen_columns, &null_bitmap).unwrap();
    let (_, _, decoded_varlen, _) = decode_row(&encoded, 1, 1).unwrap();
    // For overflow, decode returns None (inline_len = 0, no inline data)
    // The actual data would need to be fetched from overflow page
    assert_eq!(decoded_varlen[0], None);
}

/// Test that RowHeader format_version = 1 is preserved.
#[test]
fn test_row_header_preserved() {
    // Verify RowHeader constant
    assert_eq!(RowHeader::CURRENT_VERSION, 1);
    assert_eq!(RowHeader::SIZE, 19);

    // Create a row and verify through encoding that format version is 1
    let cluster_key = ClusterKey::HiddenRowId(1);
    let fixed_columns = vec![Value::Integer(42)];
    let varlen_columns: Vec<Option<Vec<u8>>> = vec![];
    let null_bitmap = vec![false];

    let encoded = encode_row(&cluster_key, &fixed_columns, &varlen_columns, &null_bitmap).unwrap();

    // First byte should be format_version = 1
    assert_eq!(encoded[0], 1, "First byte should be format_version = 1");

    // Decode and verify no error (which means version was valid)
    let (decoded_key, _, _, _) = decode_row(&encoded, 1, 0).unwrap();
    assert_eq!(decoded_key, cluster_key);
}

/// Property-based test: decode(encode(x)) == x
/// This verifies the core roundtrip property for any valid row.
#[test]
fn test_decode_encode_inverse() {
    // Test case 1: HiddenRowId with all nulls
    {
        let cluster_key = ClusterKey::HiddenRowId(999);
        let fixed_columns = vec![Value::Null, Value::Null, Value::Null];
        let varlen_columns: Vec<Option<Vec<u8>>> = vec![None, None];
        let null_bitmap = vec![true, true, true, true, true];

        let encoded = encode_row(&cluster_key, &fixed_columns, &varlen_columns, &null_bitmap).unwrap();
        let (decoded_key, decoded_fixed, decoded_varlen, decoded_nulls) =
            decode_row(&encoded, 3, 2).unwrap();

        assert_eq!(decoded_key, cluster_key);
        assert_eq!(decoded_fixed, fixed_columns);
        assert_eq!(decoded_varlen, varlen_columns);
        assert_eq!(decoded_nulls, null_bitmap);
    }

    // Test case 2: PrimaryKey with mixed values
    {
        let cluster_key = ClusterKey::PrimaryKey(Value::Integer(100));
        let fixed_columns = vec![
            Value::Boolean(false),
            Value::Integer(-999),
            Value::Float(-0.0001),
        ];
        let varlen_columns = vec![
            Some(b"test data".to_vec()),
            Some(vec![0x01, 0x02, 0x03, 0x04]),
        ];
        let null_bitmap = vec![false, false, false, false, false];

        let encoded = encode_row(&cluster_key, &fixed_columns, &varlen_columns, &null_bitmap).unwrap();
        let (decoded_key, decoded_fixed, decoded_varlen, decoded_nulls) =
            decode_row(&encoded, 3, 2).unwrap();

        assert_eq!(decoded_key, cluster_key);
        assert_eq!(decoded_fixed, fixed_columns);
        assert_eq!(decoded_varlen, varlen_columns);
        assert_eq!(decoded_nulls, null_bitmap);
    }

    // Test case 3: Complex text and blob
    {
        let cluster_key = ClusterKey::HiddenRowId(1);
        let long_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.".to_string();
        let fixed_columns = vec![Value::Text(long_text.clone())];
        let varlen_columns: Vec<Option<Vec<u8>>> = vec![];
        let null_bitmap = vec![false];

        let encoded = encode_row(&cluster_key, &fixed_columns, &varlen_columns, &null_bitmap).unwrap();
        let (_, decoded_fixed, _, _) = decode_row(&encoded, 1, 0).unwrap();

        assert_eq!(decoded_fixed[0], Value::Text(long_text));
    }
}