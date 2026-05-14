//! Overflow page utilities for Compact Row v1 format.

use crate::row_format::types::OverflowPage;

/// Maximum data size per overflow page (page_size - HEADER_SIZE)
pub const OVERFLOW_PAGE_DATA_SIZE: usize = 8192 - 4;

/// Split data into overflow pages.
pub fn split_into_overflow_pages(_data: &[u8]) -> Vec<OverflowPage> {
    Vec::new()
}

/// Reassemble data from overflow pages.
pub fn reassemble_from_overflow_pages(_pages: &[OverflowPage]) -> Vec<u8> {
    Vec::new()
}
