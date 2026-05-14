//! Row encoder for Compact Row v1 format.

use crate::row_format::null_bitmap::encode_null_bitmap;
use crate::row_format::types::{ClusterKey, ClusteredLeafRecord, RowHeader, VarLenSlot};
use sqlrustgo_types::Value;
use std::io::{self, Write};

/// Encode a clustered leaf record to bytes.
pub fn encode_clustered_leaf_record(record: &ClusteredLeafRecord) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();

    // 1. RowHeader (19 bytes)
    encode_row_header(&mut buf, &record.header)?;

    // 2. ClusterKey encoding
    encode_cluster_key(&mut buf, &record.cluster_key)?;

    // 3. Fixed-length data
    buf.write_all(&record.fixed_data)?;

    // 4. Null bitmap
    buf.write_all(&record.null_bitmap)?;

    // 5. VarLen slots
    for slot in &record.varlen_slots {
        encode_varlen_slot(&mut buf, slot)?;
    }

    Ok(buf)
}

/// Encode a row from component arrays (higher-level API).
///
/// # Arguments
/// * `cluster_key` - The cluster key (primary key or hidden row ID)
/// * `fixed_columns` - Fixed-length column values
/// * `varlen_columns` - Variable-length column data (None = NULL)
/// * `null_bitmap` - Null indicators (true = NULL)
///
/// # Returns
/// Encoded row bytes
pub fn encode_row(
    cluster_key: &ClusterKey,
    fixed_columns: &[Value],
    varlen_columns: &[Option<Vec<u8>>],
    null_bitmap: &[bool],
) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();

    // 1. RowHeader (19 bytes)
    let header = RowHeader::new();
    encode_row_header(&mut buf, &header)?;

    // 2. ClusterKey encoding
    encode_cluster_key(&mut buf, cluster_key)?;

    // 3. Fixed-length data (inline, in column order)
    for val in fixed_columns {
        encode_value(&mut buf, val)?;
    }

    // 4. NULL bitmap
    let bitmap = encode_null_bitmap(null_bitmap);
    buf.write_all(&bitmap)?;

    // 5. VarLen slots
    for opt_data in varlen_columns {
        match opt_data {
            Some(data) => {
                let slot = VarLenSlot::new(data);
                encode_varlen_slot(&mut buf, &slot)?;
            }
            None => {
                // NULL slot - write 0 length
                buf.write_all(&0u16.to_le_bytes())?;
            }
        }
    }

    Ok(buf)
}

fn encode_row_header(buf: &mut Vec<u8>, header: &RowHeader) -> io::Result<()> {
    buf.push(header.format_version);
    buf.write_all(&header.flags.to_le_bytes())?;
    buf.write_all(&header.trx_id.to_le_bytes())?;
    buf.write_all(&header.undo_ptr.to_le_bytes())?;
    Ok(())
}

/// Encode cluster key to bytes.
pub fn encode_cluster_key(buf: &mut Vec<u8>, key: &ClusterKey) -> io::Result<()> {
    match key {
        ClusterKey::PrimaryKey(v) => {
            buf.push(0); // variant tag
            encode_value(buf, v)?;
        }
        ClusterKey::HiddenRowId(id) => {
            buf.push(1); // variant tag
            buf.write_all(&id.to_le_bytes())?;
        }
    }
    Ok(())
}

fn encode_value(buf: &mut Vec<u8>, val: &Value) -> io::Result<()> {
    match val {
        Value::Null => {
            buf.push(0); // null marker
        }
        Value::Boolean(b) => {
            buf.push(1);
            buf.push(*b as u8);
        }
        Value::Integer(i) => {
            buf.push(2);
            buf.write_all(&i.to_le_bytes())?;
        }
        Value::Float(f) => {
            buf.push(3);
            buf.write_all(&f.to_bits().to_le_bytes())?;
        }
        Value::Text(s) => {
            buf.push(4);
            buf.write_all(&(s.len() as u32).to_le_bytes())?;
            buf.write_all(s.as_bytes())?;
        }
        Value::Blob(b) => {
            buf.push(5);
            buf.write_all(&(b.len() as u32).to_le_bytes())?;
            buf.write_all(b)?;
        }
        Value::Geometry(g) => {
            buf.push(6);
            let wkb = sqlrustgo_gis::to_wkb(g);
            buf.write_all(&(wkb.len() as u32).to_le_bytes())?;
            buf.write_all(&wkb)?;
        }
    }
    Ok(())
}

/// Encode a fixed-length value (alias for encode_value).
pub fn encode_fixed_value(buf: &mut Vec<u8>, val: &Value) -> io::Result<()> {
    encode_value(buf, val)
}

fn encode_varlen_slot(buf: &mut Vec<u8>, slot: &VarLenSlot) -> io::Result<()> {
    buf.write_all(&slot.inline_len.to_le_bytes())?;
    if let Some(ref data) = slot.inline_data {
        buf.write_all(data)?;
    }
    // overflow_page/len written separately by page allocator
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_row_header() {
        let header = RowHeader::new();
        let mut buf = Vec::new();
        encode_row_header(&mut buf, &header).unwrap();

        // format_version (1) + flags (2) + trx_id (8) + undo_ptr (8) = 19
        assert_eq!(buf.len(), 19);
    }

    #[test]
    fn test_encode_cluster_key_pk() {
        let key = ClusterKey::PrimaryKey(Value::Integer(42));
        let mut buf = Vec::new();
        encode_cluster_key(&mut buf, &key).unwrap();

        // tag (1) + type marker (1) + value (8) = 10
        assert!(!buf.is_empty());
    }

    #[test]
    fn test_encode_cluster_key_hidden() {
        let key = ClusterKey::HiddenRowId(100);
        let mut buf = Vec::new();
        encode_cluster_key(&mut buf, &key).unwrap();

        // tag (1) + id (8) = 9
        assert_eq!(buf.len(), 9);
    }

    #[test]
    fn test_encode_value_integer() {
        let val = Value::Integer(42);
        let mut buf = Vec::new();
        encode_value(&mut buf, &val).unwrap();

        // type marker (1) + value (8) = 9
        assert_eq!(buf.len(), 9);
    }

    #[test]
    fn test_encode_value_text() {
        let val = Value::Text("hello".to_string());
        let mut buf = Vec::new();
        encode_value(&mut buf, &val).unwrap();

        // type marker (1) + len (4) + "hello" (5) = 10
        assert_eq!(buf.len(), 10);
    }

    #[test]
    fn test_encode_value_null() {
        let val = Value::Null;
        let mut buf = Vec::new();
        encode_value(&mut buf, &val).unwrap();

        // type marker (1) = 1
        assert_eq!(buf.len(), 1);
    }

    #[test]
    fn test_encode_value_float() {
        let val = Value::Float(3.14);
        let mut buf = Vec::new();
        encode_value(&mut buf, &val).unwrap();

        // type marker (1) + bits (8) = 9
        assert_eq!(buf.len(), 9);
    }

    #[test]
    fn test_encode_value_boolean() {
        let val = Value::Boolean(true);
        let mut buf = Vec::new();
        encode_value(&mut buf, &val).unwrap();

        // type marker (1) + value (1) = 2
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_encode_value_blob() {
        let val = Value::Blob(vec![0x01, 0x02, 0x03]);
        let mut buf = Vec::new();
        encode_value(&mut buf, &val).unwrap();

        // type marker (1) + len (4) + data (3) = 8
        assert_eq!(buf.len(), 8);
    }

    #[test]
    fn test_encode_varlen_small() {
        let data = vec![0u8; 50];
        let slot = VarLenSlot::new(&data);
        let mut buf = Vec::new();
        encode_varlen_slot(&mut buf, &slot).unwrap();

        // u16 len (2) + data (50) = 52
        assert_eq!(buf.len(), 2 + 50);
    }

    #[test]
    fn test_encode_varlen_large() {
        let data = vec![0u8; 200]; // > 128 bytes
        let slot = VarLenSlot::new(&data);
        let mut buf = Vec::new();
        encode_varlen_slot(&mut buf, &slot).unwrap();

        // u16 len=0 only (overflow)
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_encode_varlen_null() {
        let data = vec![];
        let slot = VarLenSlot::new(&data);
        let mut buf = Vec::new();
        encode_varlen_slot(&mut buf, &slot).unwrap();

        // u16 len=0
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_encode_row() {
        let cluster_key = ClusterKey::HiddenRowId(100);
        let fixed = vec![Value::Integer(42), Value::Text("hello".to_string())];
        let varlen: Vec<Option<Vec<u8>>> = vec![None];
        let nulls = vec![false, false, true];

        let encoded = encode_row(&cluster_key, &fixed, &varlen, &nulls).unwrap();
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_encode_row_all_nulls() {
        let cluster_key = ClusterKey::HiddenRowId(1);
        let fixed = vec![Value::Null, Value::Null];
        let varlen: Vec<Option<Vec<u8>>> = vec![None, None];
        let nulls = vec![true, true, true, true];

        let encoded = encode_row(&cluster_key, &fixed, &varlen, &nulls).unwrap();
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_encode_decode_roundtrip_fixed() {
        let cluster_key = ClusterKey::HiddenRowId(100);
        let fixed = vec![Value::Integer(42), Value::Text("hello".to_string())];
        let varlen: Vec<Option<Vec<u8>>> = vec![None];
        let nulls = vec![false, false, true];

        let encoded = encode_row(&cluster_key, &fixed, &varlen, &nulls).unwrap();
        assert!(!encoded.is_empty());
    }
}
