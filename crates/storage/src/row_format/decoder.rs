//! Row decoder for Compact Row v1 format.

use crate::row_format::null_bitmap::decode_null_bitmap;
use crate::row_format::types::ClusterKey;
use sqlrustgo_types::Value;
use std::io::{Error, ErrorKind};

/// Current format version = 1
const FORMAT_VERSION: u8 = 1;

/// Decode a row from bytes.
///
/// # Arguments
/// * `buf` - The encoded row bytes
/// * `fixed_column_count` - Number of fixed-length columns
/// * `varlen_column_count` - Number of variable-length columns
///
/// # Returns
/// Tuple of (cluster_key, fixed_values, varlen_columns, nulls)
#[allow(clippy::type_complexity)]
pub fn decode_row(
    buf: &[u8],
    fixed_column_count: usize,
    varlen_column_count: usize,
) -> std::io::Result<(ClusterKey, Vec<Value>, Vec<Option<Vec<u8>>>, Vec<bool>)> {
    let mut offset = 0;

    // 1. RowHeader (19 bytes)
    (offset, _) = decode_row_header(buf, offset)?;

    // 2. ClusterKey
    let (new_offset, cluster_key) = decode_cluster_key(buf, offset)?;
    offset = new_offset;

    // 3. Fixed-length columns
    let mut fixed_values = Vec::with_capacity(fixed_column_count);
    for _ in 0..fixed_column_count {
        let (new_offset, value) = decode_fixed_value(buf, offset)?;
        offset = new_offset;
        fixed_values.push(value);
    }

    // 4. Null bitmap
    let null_bitmap_size = (fixed_column_count + varlen_column_count).div_ceil(8);
    let null_bitmap_bytes = read_bytes(buf, offset, null_bitmap_size)?;
    offset += null_bitmap_size;
    let nulls = decode_null_bitmap(null_bitmap_bytes, fixed_column_count + varlen_column_count);

    // 5. VarLen slots
    let mut varlen_columns = Vec::with_capacity(varlen_column_count);
    for _ in 0..varlen_column_count {
        let (new_offset, slot) = decode_varlen_slot(buf, offset)?;
        offset = new_offset;
        varlen_columns.push(slot);
    }

    Ok((cluster_key, fixed_values, varlen_columns, nulls))
}

fn decode_row_header(buf: &[u8], offset: usize) -> std::io::Result<(usize, ())> {
    let format_version = read_u8(buf, offset)?;
    if format_version != FORMAT_VERSION {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "invalid format version: expected {}, got {}",
                FORMAT_VERSION, format_version
            ),
        ));
    }
    // Skip flags (2), trx_id (8), undo_ptr (8) - not needed for Phase A
    Ok((offset + 19, ()))
}

fn decode_cluster_key(buf: &[u8], offset: usize) -> std::io::Result<(usize, ClusterKey)> {
    let tag = read_u8(buf, offset)?;
    match tag {
        0 => {
            // PrimaryKey
            let (_new_offset, value) = decode_fixed_value(buf, offset + 1)?;
            Ok((
                offset + 1 + value_encoding_size(&value),
                ClusterKey::PrimaryKey(value),
            ))
        }
        1 => {
            // HiddenRowId
            let id = read_u64(buf, offset + 1)?;
            Ok((offset + 9, ClusterKey::HiddenRowId(id)))
        }
        _ => Err(Error::new(
            ErrorKind::InvalidData,
            format!("invalid cluster key tag: {}", tag),
        )),
    }
}

/// Estimate the encoding size for a Value (for cluster key offset calculation).
fn value_encoding_size(val: &Value) -> usize {
    match val {
        Value::Null => 1,
        Value::Boolean(_) => 2,
        Value::Integer(_) => 9,
        Value::Float(_) => 9,
        Value::Text(s) => 1 + 4 + s.len(),
        Value::Blob(b) => 1 + 4 + b.len(),
    }
}

/// Decode a fixed-length value (Value type).
/// Value encoding: Null=0, Boolean=1, Integer=2, Float=3, Text=4, Blob=5
pub fn decode_fixed_value(buf: &[u8], offset: usize) -> std::io::Result<(usize, Value)> {
    let type_marker = read_u8(buf, offset)?;
    match type_marker {
        0 => Ok((offset + 1, Value::Null)),
        1 => {
            let b = read_u8(buf, offset + 1)?;
            Ok((offset + 2, Value::Boolean(b != 0)))
        }
        2 => {
            let i = read_i64(buf, offset + 1)?;
            Ok((offset + 9, Value::Integer(i)))
        }
        3 => {
            let bits = read_u64(buf, offset + 1)?;
            let f = f64::from_bits(bits);
            Ok((offset + 9, Value::Float(f)))
        }
        4 => {
            let len = read_u32(buf, offset + 1)? as usize;
            let data = read_bytes(buf, offset + 5, len)?;
            let s = String::from_utf8(data)
                .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid UTF-8 in Text value"))?;
            Ok((offset + 5 + len, Value::Text(s)))
        }
        5 => {
            let len = read_u32(buf, offset + 1)? as usize;
            let data = read_bytes(buf, offset + 5, len)?;
            Ok((offset + 5 + len, Value::Blob(data)))
        }
        _ => Err(Error::new(
            ErrorKind::InvalidData,
            format!("invalid value type marker: {}", type_marker),
        )),
    }
}

/// Decode a variable-length slot.
/// Format: u16 length + inline data (if length > 0 and <= 128)
fn decode_varlen_slot(buf: &[u8], offset: usize) -> std::io::Result<(usize, Option<Vec<u8>>)> {
    let inline_len = read_u16(buf, offset)?;
    if inline_len == 0 {
        return Ok((offset + 2, None));
    }
    let data = read_bytes(buf, offset + 2, inline_len as usize)?;
    Ok((offset + 2 + inline_len as usize, Some(data)))
}

// ============== Helper functions ==============

fn read_u8(buf: &[u8], offset: usize) -> std::io::Result<u8> {
    if offset >= buf.len() {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "unexpected end of data",
        ));
    }
    Ok(buf[offset])
}

fn read_u16(buf: &[u8], offset: usize) -> std::io::Result<u16> {
    if offset + 2 > buf.len() {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "unexpected end of data",
        ));
    }
    Ok(u16::from_le_bytes([buf[offset], buf[offset + 1]]))
}

fn read_u32(buf: &[u8], offset: usize) -> std::io::Result<u32> {
    if offset + 4 > buf.len() {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "unexpected end of data",
        ));
    }
    Ok(u32::from_le_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
    ]))
}

fn read_u64(buf: &[u8], offset: usize) -> std::io::Result<u64> {
    if offset + 8 > buf.len() {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "unexpected end of data",
        ));
    }
    Ok(u64::from_le_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
        buf[offset + 4],
        buf[offset + 5],
        buf[offset + 6],
        buf[offset + 7],
    ]))
}

fn read_i64(buf: &[u8], offset: usize) -> std::io::Result<i64> {
    if offset + 8 > buf.len() {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "unexpected end of data",
        ));
    }
    Ok(i64::from_le_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
        buf[offset + 4],
        buf[offset + 5],
        buf[offset + 6],
        buf[offset + 7],
    ]))
}

fn read_bytes(buf: &[u8], offset: usize, len: usize) -> std::io::Result<Vec<u8>> {
    if offset + len > buf.len() {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "unexpected end of data",
        ));
    }
    Ok(buf[offset..offset + len].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_row_header() {
        // Create a minimal buffer with valid header
        let mut buf = vec![1u8; 19]; // format_version = 1
        buf.extend_from_slice(&0u16.to_le_bytes()); // flags = 0
        buf.extend_from_slice(&0u64.to_le_bytes()); // trx_id = 0
        buf.extend_from_slice(&0u64.to_le_bytes()); // undo_ptr = 0

        let result = decode_row_header(&buf, 0).unwrap();
        assert_eq!(result.0, 19);
    }

    #[test]
    fn test_decode_invalid_version() {
        let mut buf = vec![99u8; 19]; // invalid version
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&0u64.to_le_bytes());
        buf.extend_from_slice(&0u64.to_le_bytes());

        let result = decode_row_header(&buf, 0);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("invalid format version"));
    }

    #[test]
    fn test_decode_cluster_key_hidden() {
        let mut buf = vec![1u8]; // tag = HiddenRowId
        buf.extend_from_slice(&100u64.to_le_bytes()); // id = 100

        let result = decode_cluster_key(&buf, 0).unwrap();
        assert_eq!(result.1, ClusterKey::HiddenRowId(100));
    }

    #[test]
    fn test_decode_cluster_key_pk() {
        let mut buf = vec![0u8]; // tag = PrimaryKey
        buf.push(2); // type = Integer
        buf.extend_from_slice(&42i64.to_le_bytes());

        let result = decode_cluster_key(&buf, 0).unwrap();
        assert_eq!(result.1, ClusterKey::PrimaryKey(Value::Integer(42)));
    }

    #[test]
    fn test_decode_fixed_value_integer() {
        let mut buf = vec![2u8]; // type = Integer
        buf.extend_from_slice(&42i64.to_le_bytes());

        let result = decode_fixed_value(&buf, 0).unwrap();
        assert_eq!(result.1, Value::Integer(42));
    }

    #[test]
    fn test_decode_fixed_value_text() {
        let mut buf = vec![4u8]; // type = Text
        buf.extend_from_slice(&5u32.to_le_bytes()); // len = 5
        buf.extend_from_slice(b"hello");

        let result = decode_fixed_value(&buf, 0).unwrap();
        assert_eq!(result.1, Value::Text("hello".to_string()));
    }

    #[test]
    fn test_decode_fixed_value_null() {
        let buf = vec![0u8]; // type = Null

        let result = decode_fixed_value(&buf, 0).unwrap();
        assert_eq!(result.1, Value::Null);
    }

    #[test]
    fn test_decode_varlen_slot_with_data() {
        let mut buf = vec![0u8; 2]; // padding
        buf.extend_from_slice(&5u16.to_le_bytes()); // len = 5
        buf.extend_from_slice(b"hello");

        let result = decode_varlen_slot(&buf, 2).unwrap();
        assert_eq!(result.1, Some(b"hello".to_vec()));
    }

    #[test]
    fn test_decode_varlen_null() {
        let buf = vec![0u8; 2]; // inline_len = 0

        let result = decode_varlen_slot(&buf, 0).unwrap();
        assert!(result.1.is_none());
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        // Use encoder to create test data
        use crate::row_format::encoder::encode_row;
        use crate::row_format::types::ClusterKey;

        let cluster_key = ClusterKey::HiddenRowId(100);
        let fixed = vec![Value::Integer(42), Value::Text("hello".to_string())];
        let varlen: Vec<Option<Vec<u8>>> = vec![Some(b"world".to_vec())];
        let nulls = vec![false, false, false];

        // Encode
        let encoded = encode_row(&cluster_key, &fixed, &varlen, &nulls).unwrap();
        assert!(!encoded.is_empty());

        // Decode
        let (decoded_key, decoded_fixed, decoded_varlen, decoded_nulls) =
            decode_row(&encoded, 2, 1).unwrap();

        // Verify
        assert_eq!(decoded_key, cluster_key);
        assert_eq!(decoded_fixed[0], Value::Integer(42));
        assert_eq!(decoded_fixed[1], Value::Text("hello".to_string()));
        assert_eq!(decoded_varlen[0], Some(b"world".to_vec()));
        assert_eq!(decoded_nulls, nulls);
    }
}
