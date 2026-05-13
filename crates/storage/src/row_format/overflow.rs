//! Overflow page utilities for Compact Row v1 format.

use crate::row_format::types::OverflowPage;

/// Maximum data size per overflow page (page_size - HEADER_SIZE)
pub const OVERFLOW_PAGE_DATA_SIZE: usize = 8192 - 4;

/// Split data into overflow pages.
pub fn split_into_overflow_pages(data: &[u8]) -> Vec<OverflowPage> {
    // TODO: Implement overflow page splitting
    Vec::new()
}

/// Reassemble data from overflow pages.
pub fn reassemble_from_overflow_pages(pages: &[OverflowPage]) -> Vec<u8> {
    // TODO: Implement overflow page reassembly
    Vec::new()
}