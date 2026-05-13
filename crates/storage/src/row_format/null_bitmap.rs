//! Null bitmap utilities for Compact Row v1 format.

/// Get the number of bytes needed for a null bitmap with n columns.
pub fn null_bitmap_size(num_columns: usize) -> usize {
    (num_columns + 7) / 8
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