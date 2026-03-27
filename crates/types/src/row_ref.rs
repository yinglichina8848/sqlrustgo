//! RowRef - Zero-copy row reference for executor hot paths
//!
//! Provides efficient row data access without cloning the entire row.
//! RowRef borrows from underlying storage bytes.
//!
//! ## Memory Layout
//!
//! Encoded row format (using `encode_row`):
//! ```text
//! [num_cols: u32][offset0: u32][offset1: u32]...[col0_data...][col1_data...]...
//! ```
//!
//! ## Benefits
//!
//! - Avoids `Vec<Value>` clone on every row access
//! - O(1) column access via offset index
//! - Reduces GC pressure in hot paths
//! - Improves cache locality

use crate::Value;

/// A zero-copy row reference that borrows from underlying row bytes
///
/// RowRef provides efficient access to row data by borrowing from the
/// original byte buffer rather than cloning. Column access requires
/// a Schema to interpret the binary data correctly.
///
/// # Example
///
/// ```ignore
/// // Note: This requires encoded row data from `encode_row()`
/// let encoded_row: Vec<u8> = encode_row(&values);
/// let row_ref = RowRef::from_bytes(&encoded_row);
/// let value = row_ref.get_column(0, &schema);
/// ```
pub struct RowRef<'a> {
    /// Raw encoded row bytes
    data: &'a [u8],
}

impl<'a> RowRef<'a> {
    /// Create a RowRef from raw byte slice
    #[inline]
    pub fn from_bytes(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Get the raw byte slice
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.data
    }

    /// Get the number of columns from encoded header
    ///
    /// Returns None if the row data is too short to contain column count
    #[inline]
    pub fn num_cols(&self) -> Option<u32> {
        if self.data.len() < 4 {
            return None;
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.data[..4]);
        Some(u32::from_le_bytes(bytes))
    }

    /// Get the offset of a specific column
    ///
    /// Returns None if index is out of bounds or data is malformed
    #[inline]
    pub fn column_offset(&self, index: u32) -> Option<u32> {
        let num_cols = self.num_cols()?;

        // Offset array starts after 4 bytes (num_cols) + (num_cols * 4) for offsets
        let offset_array_len = num_cols as usize * 4;
        if self.data.len() < 4 + offset_array_len {
            return None;
        }

        if index >= num_cols {
            return None;
        }

        let mut bytes = [0u8; 4];
        let offset_idx = 4 + (index as usize) * 4;
        bytes.copy_from_slice(&self.data[offset_idx..offset_idx + 4]);
        Some(u32::from_le_bytes(bytes))
    }

    /// Get a column value by index
    ///
    /// Requires schema to determine the column type for proper deserialization.
    /// Returns the column value by parsing the binary data at the computed offset.
    ///
    /// # Performance
    ///
    /// This is O(1) due to embedded offset index in the encoded format.
    pub fn get_column(&self, index: u32, schema: &impl RowSchema) -> Option<Value> {
        let num_cols = self.num_cols()?;

        if index >= num_cols {
            return None;
        }

        let offset = self.column_offset(index)?;
        let col_type = schema.column_type(index)?;

        // Data section starts after: 4 (num_cols) + (num_cols * 4) (offsets)
        let data_start = 4 + (num_cols as usize * 4);
        let col_data_start = data_start + offset as usize;

        if col_data_start > self.data.len() {
            return None;
        }

        Some(Self::parse_value_at(&self.data[col_data_start..], col_type))
    }

    /// Parse a Value from binary data at the given offset
    /// This uses the BinaryFormat::from_bytes pattern
    fn parse_value_at(data: &[u8], col_type: ColumnType) -> Value {
        if data.is_empty() {
            return Value::Null;
        }

        match col_type {
            ColumnType::Null => Value::Null,
            ColumnType::Integer => {
                if data.len() < 8 {
                    Value::Null
                } else {
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&data[..8]);
                    Value::Integer(i64::from_le_bytes(bytes))
                }
            }
            ColumnType::Float => {
                if data.len() < 8 {
                    Value::Null
                } else {
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&data[..8]);
                    Value::Float(f64::from_le_bytes(bytes))
                }
            }
            ColumnType::Text => {
                // Text format: [len: u64][string_data...]
                if data.len() < 8 {
                    return Value::Null;
                }
                let mut len_bytes = [0u8; 8];
                len_bytes.copy_from_slice(&data[..8]);
                let len = u64::from_le_bytes(len_bytes) as usize;
                if data.len() < 8 + len {
                    Value::Null
                } else {
                    let str_data = &data[8..8 + len];
                    String::from_utf8(str_data.to_vec())
                        .map(Value::Text)
                        .unwrap_or(Value::Null)
                }
            }
            ColumnType::Boolean => {
                if data.is_empty() {
                    Value::Null
                } else {
                    Value::Boolean(data[0] != 0)
                }
            }
            ColumnType::Blob => {
                // Blob format: [len: u64][blob_data...]
                if data.len() < 8 {
                    return Value::Null;
                }
                let mut len_bytes = [0u8; 8];
                len_bytes.copy_from_slice(&data[..8]);
                let len = u64::from_le_bytes(len_bytes) as usize;
                if data.len() < 8 + len {
                    Value::Null
                } else {
                    Value::Blob(data[8..8 + len].to_vec())
                }
            }
            ColumnType::Date => {
                if data.len() < 4 {
                    Value::Null
                } else {
                    let mut bytes = [0u8; 4];
                    bytes.copy_from_slice(&data[..4]);
                    Value::Date(i32::from_le_bytes(bytes))
                }
            }
            ColumnType::Timestamp => {
                if data.len() < 8 {
                    Value::Null
                } else {
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&data[..8]);
                    Value::Timestamp(i64::from_le_bytes(bytes))
                }
            }
        }
    }

    /// Convert RowRef to an owned Vec<Value> by cloning all columns
    ///
    /// This clones the data, but is still more efficient than the old pattern
    /// because it avoids the intermediate Vec<Vec<u8>> → Vec<Value> conversion.
    pub fn to_owned_row(&self, schema: &impl RowSchema) -> Vec<Value> {
        let num_cols = self.num_cols().unwrap_or(0) as usize;
        let mut result = Vec::with_capacity(num_cols);

        for i in 0..num_cols {
            if let Some(val) = self.get_column(i as u32, schema) {
                result.push(val);
            } else {
                result.push(Value::Null);
            }
        }

        result
    }
}

impl<'a> std::fmt::Debug for RowRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RowRef")
            .field("data_len", &self.data.len())
            .field("num_cols", &self.num_cols())
            .finish()
    }
}

/// Column type enumeration for RowRef parsing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnType {
    Null,
    Boolean,
    Integer,
    Float,
    Text,
    Blob,
    Date,
    Timestamp,
}

/// Trait for schema access needed by RowRef
///
/// This allows RowRef to work with different schema implementations
/// without depending on the catalog crate.
pub trait RowSchema {
    /// Get the number of columns
    fn num_columns(&self) -> usize;

    /// Get the column type at index
    fn column_type(&self, index: u32) -> Option<ColumnType>;
}

impl RowSchema for Vec<ColumnType> {
    fn num_columns(&self) -> usize {
        self.len()
    }

    fn column_type(&self, index: u32) -> Option<ColumnType> {
        self.get(index as usize).copied()
    }
}

/// Encode a row (Vec<Value>) to bytes with column offset index
///
/// Format:
/// ```text
/// [num_cols: u32][offset0: u32][offset1: u32]...[col0_data...][col1_data...]...
/// ```
///
/// This enables O(1) column access via the offset index.
pub fn encode_row(values: &[Value]) -> Vec<u8> {
    if values.is_empty() {
        return vec![0, 0, 0, 0]; // num_cols = 0
    }

    let num_cols = values.len() as u32;

    // First pass: calculate sizes and collect data
    let col_data: Vec<Vec<u8>> = values
        .iter()
        .map(|v| binary_encode_value(v))
        .collect();

    let total_data_len: usize = col_data.iter().map(|d| d.len()).sum();

    // Layout: [4 bytes num_cols][num_cols * 4 bytes offsets][data...]
    let offset_array_size = (num_cols as usize) * 4;
    let total_size = 4 + offset_array_size + total_data_len;

    let mut result = Vec::with_capacity(total_size);

    // Write num_cols (little-endian for easier parsing)
    result.extend_from_slice(&num_cols.to_le_bytes());

    // Write offset array and data section
    let mut current_offset = 0u32;
    for col_bytes in &col_data {
        result.extend_from_slice(&current_offset.to_le_bytes());
        current_offset += col_bytes.len() as u32;
    }

    // Write column data
    for col_bytes in &col_data {
        result.extend_from_slice(col_bytes);
    }

    result
}

/// Binary encode a single Value (without type indicator for embedded use)
fn binary_encode_value(value: &Value) -> Vec<u8> {
    match value {
        Value::Null => vec![],
        Value::Boolean(b) => vec![if *b { 1 } else { 0 }],
        Value::Integer(n) => n.to_le_bytes().to_vec(),
        Value::Float(f) => f.to_le_bytes().to_vec(),
        Value::Text(s) => {
            let mut result = Vec::with_capacity(8 + s.len());
            result.extend_from_slice(&(s.len() as u64).to_le_bytes());
            result.extend_from_slice(s.as_bytes());
            result
        }
        Value::Blob(b) => {
            let mut result = Vec::with_capacity(8 + b.len());
            result.extend_from_slice(&(b.len() as u64).to_le_bytes());
            result.extend_from_slice(b);
            result
        }
        Value::Date(d) => d.to_le_bytes().to_vec(),
        Value::Timestamp(ts) => ts.to_le_bytes().to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let values = vec![
            Value::Integer(42),
            Value::Text("hello".to_string()),
            Value::Float(3.14),
            Value::Boolean(true),
        ];
        let schema = vec![
            ColumnType::Integer,
            ColumnType::Text,
            ColumnType::Float,
            ColumnType::Boolean,
        ];

        let encoded = encode_row(&values);
        let row_ref = RowRef::from_bytes(&encoded);

        assert_eq!(row_ref.num_cols(), Some(4));

        assert_eq!(row_ref.get_column(0, &schema), Some(Value::Integer(42)));
        assert_eq!(row_ref.get_column(1, &schema), Some(Value::Text("hello".to_string())));
        assert_eq!(row_ref.get_column(2, &schema), Some(Value::Float(3.14)));
        assert_eq!(row_ref.get_column(3, &schema), Some(Value::Boolean(true)));
    }

    #[test]
    fn test_row_ref_empty_row() {
        let encoded = encode_row(&[]);
        let row_ref = RowRef::from_bytes(&encoded);
        assert_eq!(row_ref.num_cols(), Some(0));
    }

    #[test]
    fn test_row_ref_null_values() {
        let values = vec![Value::Null, Value::Integer(100), Value::Null];
        let schema = vec![ColumnType::Null, ColumnType::Integer, ColumnType::Null];

        let encoded = encode_row(&values);
        let row_ref = RowRef::from_bytes(&encoded);

        assert_eq!(row_ref.get_column(0, &schema), Some(Value::Null));
        assert_eq!(row_ref.get_column(1, &schema), Some(Value::Integer(100)));
        assert_eq!(row_ref.get_column(2, &schema), Some(Value::Null));
    }

    #[test]
    fn test_row_ref_to_owned_row() {
        let values = vec![
            Value::Integer(1),
            Value::Text("test".to_string()),
        ];
        let schema = vec![ColumnType::Integer, ColumnType::Text];

        let encoded = encode_row(&values);
        let row_ref = RowRef::from_bytes(&encoded);
        let owned = row_ref.to_owned_row(&schema);

        assert_eq!(owned, values);
    }

    #[test]
    fn test_row_ref_out_of_bounds() {
        let values = vec![Value::Integer(42)];
        let schema = vec![ColumnType::Integer];

        let encoded = encode_row(&values);
        let row_ref = RowRef::from_bytes(&encoded);

        assert!(row_ref.get_column(5, &schema).is_none());
    }

    #[test]
    fn test_binary_encode_blob() {
        let blob_data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let values = vec![Value::Blob(blob_data.clone())];
        let schema = vec![ColumnType::Blob];

        let encoded = encode_row(&values);
        let row_ref = RowRef::from_bytes(&encoded);

        assert_eq!(row_ref.get_column(0, &schema), Some(Value::Blob(blob_data)));
    }

    #[test]
    fn test_binary_encode_timestamp() {
        let ts = 1_000_000i64;
        let values = vec![Value::Timestamp(ts)];
        let schema = vec![ColumnType::Timestamp];

        let encoded = encode_row(&values);
        let row_ref = RowRef::from_bytes(&encoded);

        assert_eq!(row_ref.get_column(0, &schema), Some(Value::Timestamp(ts)));
    }

    #[test]
    fn test_binary_encode_date() {
        let date = 19780i32; // Days since epoch
        let values = vec![Value::Date(date)];
        let schema = vec![ColumnType::Date];

        let encoded = encode_row(&values);
        let row_ref = RowRef::from_bytes(&encoded);

        assert_eq!(row_ref.get_column(0, &schema), Some(Value::Date(date)));
    }

    #[test]
    fn test_row_ref_debug() {
        let values = vec![Value::Integer(42)];
        let encoded = encode_row(&values);
        let row_ref = RowRef::from_bytes(&encoded);
        let debug = format!("{:?}", row_ref);
        assert!(debug.contains("RowRef"));
    }
}
