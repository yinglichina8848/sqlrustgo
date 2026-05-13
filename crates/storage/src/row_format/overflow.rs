//! Overflow page chain management for Compact Row v1 format.
//!
//! Overflow pages store large variable-length data (> 128 bytes).
//! This module provides writer/reader for managing overflow chains.

use crate::row_format::types::OverflowPage;
use std::collections::HashSet;

/// Maximum data size per overflow page (page_size - HEADER_SIZE)
pub const OVERFLOW_PAGE_DATA_SIZE: usize = 8192 - 4;

/// Overflow chain writer for append-only chain building.
pub struct OverflowChainWriter {
    pages: Vec<OverflowPage>,
}

impl OverflowChainWriter {
    /// Create a new empty overflow chain writer.
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    /// Add a new page to the end of the chain.
    /// The page will be linked to the previous last page.
    pub fn push_page(&mut self, data: Vec<u8>) {
        let next_page = None;
        let page = OverflowPage::new(data, next_page);

        // Link previous last page to this new page
        let prev_idx = self.pages.len();
        if let Some(prev) = self.pages.last_mut() {
            prev.next_page = Some(prev_idx as u32);
        }

        self.pages.push(page);
    }

    /// Get first page data (without the next_page link).
    pub fn first_page_data(&self) -> Option<&[u8]> {
        self.pages.first().map(|p| p.data.as_slice())
    }

    /// Get the number of pages in the chain.
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// Get the pages in the chain.
    pub fn pages(&self) -> &[OverflowPage] {
        &self.pages
    }

    /// Get pages into the chain (consumes self).
    pub fn into_pages(self) -> Vec<OverflowPage> {
        self.pages
    }
}

impl Default for OverflowChainWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Overflow chain reader for sequential overflow access.
pub struct OverflowChainReader {
    pages: Vec<OverflowPage>,
}

impl OverflowChainReader {
    /// Create a new reader from pages.
    pub fn new(pages: Vec<OverflowPage>) -> Self {
        Self { pages }
    }

    /// Read all data from the chain, reassembling in order.
    pub fn read_all(&self) -> Vec<u8> {
        self.pages.iter().flat_map(|p| p.data.clone()).collect()
    }

    /// Get the number of pages in the chain.
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// Get the pages in the chain.
    pub fn pages(&self) -> &[OverflowPage] {
        &self.pages
    }

    /// Get iterator over pages.
    pub fn iter(&self) -> impl Iterator<Item = &OverflowPage> {
        self.pages.iter()
    }
}

/// Split data into overflow pages.
/// Returns pages with None as next_page (chain broken - caller must link).
pub fn split_into_overflow_pages(data: &[u8]) -> Vec<OverflowPage> {
    if data.is_empty() {
        return Vec::new();
    }

    let mut pages = Vec::new();
    let mut offset = 0;

    while offset < data.len() {
        let end = (offset + OVERFLOW_PAGE_DATA_SIZE).min(data.len());
        let page_data = data[offset..end].to_vec();
        pages.push(OverflowPage::new(page_data, None));
        offset = end;
    }

    pages
}

/// Reassemble data from overflow pages (ignores next_page links, uses order).
pub fn reassemble_from_overflow_pages(pages: &[OverflowPage]) -> Vec<u8> {
    pages.iter().flat_map(|p| p.data.clone()).collect()
}

/// Verify overflow chain is acyclic (no loops).
/// Returns Ok(()) if acyclic, Err(cycle_start) if cycle found.
pub fn verify_chain_acyclic(pages: &[OverflowPage]) -> Result<(), usize> {
    if pages.is_empty() {
        return Ok(());
    }

    let mut seen: HashSet<usize> = HashSet::new();
    let mut current_idx: Option<usize> = Some(0);

    while let Some(idx) = current_idx {
        // Check for cycle
        if seen.contains(&idx) {
            return Err(idx);
        }
        seen.insert(idx);

        // Check if we have more pages to visit
        if idx >= pages.len() {
            return Err(idx);
        }

        // Move to next page
        current_idx = pages[idx].next_page.map(|n| n as usize);
    }

    Ok(())
}

/// Link overflow pages into a chain (from first to last).
pub fn link_overflow_pages(pages: &mut [OverflowPage]) {
    if pages.is_empty() {
        return;
    }

    for i in 0..pages.len() - 1 {
        pages[i].next_page = Some((i + 1) as u32);
    }

    // Last page has None
    if let Some(last) = pages.last_mut() {
        last.next_page = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer_basic() {
        let mut writer = OverflowChainWriter::new();
        assert!(writer.is_empty());
        assert_eq!(writer.len(), 0);

        writer.push_page(vec![1, 2, 3, 4]);
        assert_eq!(writer.len(), 1);
        assert!(!writer.is_empty());
        assert_eq!(writer.first_page_data(), Some(&[1, 2, 3, 4][..]));
    }

    #[test]
    fn test_writer_chain() {
        let mut writer = OverflowChainWriter::new();
        writer.push_page(vec![1, 2, 3]);
        writer.push_page(vec![4, 5, 6]);
        writer.push_page(vec![7, 8, 9]);

        assert_eq!(writer.len(), 3);

        // Check linking
        let pages = writer.pages();
        assert_eq!(pages[0].next_page, Some(1));
        assert_eq!(pages[1].next_page, Some(2));
        assert_eq!(pages[2].next_page, None);
    }

    #[test]
    fn test_reader() {
        let pages = vec![
            OverflowPage::new(vec![1, 2, 3], Some(1)),
            OverflowPage::new(vec![4, 5, 6], Some(2)),
            OverflowPage::new(vec![7, 8, 9], None),
        ];

        let reader = OverflowChainReader::new(pages);
        assert_eq!(reader.len(), 3);
        assert_eq!(reader.read_all(), vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_split_into_pages() {
        let data = vec![1u8; 20];
        let pages = split_into_overflow_pages(&data);

        // Should be split into pages of OVERFLOW_PAGE_DATA_SIZE each
        // 20 bytes should fit in one page
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].data.len(), 20);
    }

    #[test]
    fn test_split_large_data() {
        // Create data larger than one page
        let data: Vec<u8> = (0..15000).map(|i| i as u8).collect();
        let pages = split_into_overflow_pages(&data);

        // Should be split into 2 pages
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].data.len(), OVERFLOW_PAGE_DATA_SIZE);
        assert_eq!(pages[1].data.len(), 15000 - OVERFLOW_PAGE_DATA_SIZE);
    }

    #[test]
    fn test_reassemble() {
        let pages = vec![
            OverflowPage::new(vec![1, 2, 3], None),
            OverflowPage::new(vec![4, 5, 6], None),
        ];

        let data = reassemble_from_overflow_pages(&pages);
        assert_eq!(data, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_verify_acyclic() {
        let pages = vec![
            OverflowPage::new(vec![1, 2, 3], Some(1)),
            OverflowPage::new(vec![4, 5, 6], None),
        ];

        assert!(verify_chain_acyclic(&pages).is_ok());
    }

    #[test]
    fn test_verify_acyclic_empty() {
        let pages: Vec<OverflowPage> = vec![];
        assert!(verify_chain_acyclic(&pages).is_ok());
    }

    #[test]
    fn test_verify_cyclic() {
        // Create a cycle: 0 -> 1 -> 0
        let pages = vec![
            OverflowPage::new(vec![1, 2, 3], Some(1)),
            OverflowPage::new(vec![4, 5, 6], Some(0)),
        ];

        assert!(verify_chain_acyclic(&pages).is_err());
    }

    #[test]
    fn test_link_pages() {
        let mut pages = vec![
            OverflowPage::new(vec![1, 2, 3], None),
            OverflowPage::new(vec![4, 5, 6], None),
            OverflowPage::new(vec![7, 8, 9], None),
        ];

        link_overflow_pages(&mut pages);

        assert_eq!(pages[0].next_page, Some(1));
        assert_eq!(pages[1].next_page, Some(2));
        assert_eq!(pages[2].next_page, None);
    }
}