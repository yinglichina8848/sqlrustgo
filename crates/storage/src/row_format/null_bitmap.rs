//! Null bitmap utilities for Compact Row v1 format.

/// Get the number of bytes needed for a null bitmap with n columns.
pub fn null_bitmap_size(num_columns: usize) -> usize {
    (num_columns + 7) / 8
}

/// Encode a null bitmap from a vector of booleans.
/// Each true value means NULL, false means NOT NULL.
pub fn encode_null_bitmap(nulls: &[bool]) -> Vec<u8> {
    let size = null_bitmap_size(nulls.len());
    let mut bitmap = vec![0u8; size];
    for (i, &is_null) in nulls.iter().enumerate() {
        if is_null {
            let byte_index = i / 8;
            let bit_index = i % 8;
            bitmap[byte_index] |= 1 << bit_index;
        }
    }
    bitmap
}

/// Check if a column is null given the null bitmap.
pub fn is_null(bitmap: &[u8], column_index: usize) -> bool {
    if column_index >= bitmap.len() * 8 {
        return false;
    }
    let byte_index = column_index / 8;
    let bit_index = column_index % 8;
    (bitmap[byte_index] & (1 << bit_index)) != 0
}

/// Decode a null bitmap to a vector of booleans.
/// Each true value means NULL, false means NOT NULL.
pub fn decode_null_bitmap(bitmap: Vec<u8>) -> Vec<bool> {
    let mut nulls = Vec::new();
    for byte in bitmap {
        for bit in 0..8 {
            nulls.push((byte & (1 << bit)) != 0);
        }
    }
    nulls
}