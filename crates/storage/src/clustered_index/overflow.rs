//! Overflow page management for large field storage.
//!
//! Overflow pages are used when a variable-length field exceeds the 128-byte
//! inline threshold. Each overflow page can store up to PAGE_SIZE bytes.
//! Large fields chain multiple overflow pages together.

use crate::clustered_index::leaf::PAGE_SIZE;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

/// Overflow page header size (next_page pointer)
pub const OVERFLOW_HEADER_SIZE: usize = 4;

/// Maximum data size per overflow page
pub const OVERFLOW_DATA_SIZE: usize = PAGE_SIZE - OVERFLOW_HEADER_SIZE;

/// Overflow page manager for allocating and managing overflow chains.
///
/// In a real implementation, this would interface with the buffer pool
/// and page allocator. For Phase A, we simulate the management.
#[derive(Debug, Clone)]
pub struct OverflowManager {
    /// In-memory cache of overflow pages (page_id -> data)
    /// In production, this would be replaced by buffer pool access
    pages: HashMap<u32, Vec<u8>>,
    /// Next available page ID
    next_page_id: u32,
    /// Page allocation count
    allocation_count: usize,
}

impl OverflowManager {
    /// Create a new overflow manager.
    pub fn new() -> Self {
        Self {
            pages: HashMap::new(),
            next_page_id: 1, // 0 is reserved for "no page"
            allocation_count: 0,
        }
    }

    /// Allocate a new overflow page.
    /// Returns the page ID.
    pub fn allocate_page(&mut self, data: &[u8], next_page: Option<u32>) -> u32 {
        let page_id = self.next_page_id;
        self.next_page_id += 1;
        self.allocation_count += 1;

        // Store the page data with header (no padding)
        let mut page_data =
            Vec::with_capacity(OVERFLOW_HEADER_SIZE + data.len().min(OVERFLOW_DATA_SIZE));

        // Write next_page pointer (4 bytes)
        page_data.extend_from_slice(&(next_page.unwrap_or(0).to_le_bytes()));

        // Write data (up to OVERFLOW_DATA_SIZE)
        if data.len() <= OVERFLOW_DATA_SIZE {
            page_data.extend_from_slice(data);
        } else {
            page_data.extend_from_slice(&data[..OVERFLOW_DATA_SIZE]);
            // For multi-page chains, caller should handle chaining
        }

        self.pages.insert(page_id, page_data);
        page_id
    }

    /// Read data from an overflow page chain.
    pub fn read_chain(&self, first_page_id: u32) -> std::io::Result<Vec<u8>> {
        let mut result = Vec::new();
        let mut current_page = first_page_id;

        while current_page != 0 {
            let page_data = self.pages.get(&current_page).ok_or_else(|| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("Overflow page {} not found", current_page),
                )
            })?;

            // Read next page pointer
            let next_page =
                u32::from_le_bytes([page_data[0], page_data[1], page_data[2], page_data[3]]);

            // Read data (skip 4-byte header)
            let data_end = page_data.len().min(PAGE_SIZE);
            result.extend_from_slice(&page_data[OVERFLOW_HEADER_SIZE..data_end]);

            current_page = next_page;
        }

        Ok(result)
    }

    /// Get a single overflow page.
    pub fn get_page(&self, page_id: u32) -> Option<&Vec<u8>> {
        self.pages.get(&page_id)
    }

    /// Get the next page in chain from a page.
    pub fn get_next_page(page_data: &[u8]) -> Option<u32> {
        if page_data.len() < OVERFLOW_HEADER_SIZE {
            return None;
        }
        let next = u32::from_le_bytes([page_data[0], page_data[1], page_data[2], page_data[3]]);
        if next == 0 {
            None
        } else {
            Some(next)
        }
    }

    /// Deallocate an overflow chain.
    pub fn deallocate_chain(&mut self, first_page_id: u32) -> std::io::Result<usize> {
        let mut deallocated = 0;
        let mut current_page = first_page_id;

        while current_page != 0 {
            let page_data = self.pages.get(&current_page).ok_or_else(|| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("Overflow page {} not found", current_page),
                )
            })?;

            let next_page = Self::get_next_page(page_data);

            self.pages.remove(&current_page);
            deallocated += 1;

            current_page = next_page.unwrap_or(0);
        }

        Ok(deallocated)
    }

    /// Get allocation count.
    pub fn allocation_count(&self) -> usize {
        self.allocation_count
    }

    /// Get cached page count.
    pub fn cached_pages(&self) -> usize {
        self.pages.len()
    }

    /// Verify an overflow chain is acyclic.
    pub fn verify_chain_acyclic(&self, first_page_id: u32) -> bool {
        let mut visited = HashMap::new();
        let mut current_page = first_page_id;
        let mut count = 0;
        const MAX_PAGES: usize = 10000;

        while current_page != 0 {
            if count > MAX_PAGES {
                return false; // Likely a cycle
            }

            // Check for cycle via count
            if let Some(first_seen) = visited.get(&current_page) {
                if *first_seen == count {
                    return false; // Cycle detected
                }
            }
            visited.insert(current_page, count);

            let page_data = match self.pages.get(&current_page) {
                Some(d) => d,
                None => return false, // Broken chain
            };

            let next_page = Self::get_next_page(page_data);
            current_page = next_page.unwrap_or(0);
            count += 1;
        }

        true
    }

    /// Get the data portion from a page (excluding header).
    pub fn get_page_data(page_data: &[u8]) -> &[u8] {
        if page_data.len() <= OVERFLOW_HEADER_SIZE {
            &[]
        } else {
            &page_data[OVERFLOW_HEADER_SIZE..]
        }
    }
}

impl Default for OverflowManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Encode a large value into overflow pages.
/// Returns the first page ID and total pages allocated.
pub fn encode_overflow_chain(manager: &mut OverflowManager, data: &[u8]) -> (u32, usize) {
    if data.len() <= OVERFLOW_DATA_SIZE {
        // Single page is enough
        let page_id = manager.allocate_page(data, None);
        return (page_id, 1);
    }

    // Multi-page chain - build in reverse order to get correct linking
    // First, collect all page IDs
    let mut page_ids: Vec<u32> = Vec::new();
    let mut offset = 0;

    while offset < data.len() {
        let end = std::cmp::min(offset + OVERFLOW_DATA_SIZE, data.len());
        let chunk = &data[offset..end];
        let page_id = manager.allocate_page(chunk, None);
        page_ids.push(page_id);
        offset = end;
    }

    // Now fix up the links: page i points to page i+1
    let pages_allocated = page_ids.len();
    for i in 0..page_ids.len() {
        let next_page = if i + 1 < page_ids.len() {
            Some(page_ids[i + 1])
        } else {
            None
        };
        // Update the first 4 bytes (next_page pointer) of each page
        let page_data = manager.pages.get_mut(&page_ids[i]).unwrap();
        page_data[0..4].copy_from_slice(&next_page.unwrap_or(0).to_le_bytes());
    }

    let first_page_id = page_ids.first().copied().unwrap_or(0);
    (first_page_id, pages_allocated)
}

/// Read a full value from an overflow chain.
pub fn decode_overflow_chain(
    manager: &OverflowManager,
    first_page_id: u32,
) -> std::io::Result<Vec<u8>> {
    manager.read_chain(first_page_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overflow_manager_new() {
        let manager = OverflowManager::new();
        assert_eq!(manager.allocation_count(), 0);
        assert_eq!(manager.cached_pages(), 0);
    }

    #[test]
    fn test_allocate_single_page() {
        let mut manager = OverflowManager::new();
        let data = vec![0u8; 100];

        let page_id = manager.allocate_page(&data, None);
        assert_eq!(page_id, 1);
        assert_eq!(manager.allocation_count(), 1);
        assert!(manager.pages.contains_key(&page_id));
    }

    #[test]
    fn test_allocate_chain() {
        let mut manager = OverflowManager::new();
        let data = vec![1u8; 200]; // > OVERFLOW_DATA_SIZE

        let page_id1 = manager.allocate_page(&data[..100], Some(2));
        let page_id2 = manager.allocate_page(&data[100..], None);

        assert_eq!(page_id1, 1);
        assert_eq!(page_id2, 2);
    }

    #[test]
    fn test_read_chain_single_page() {
        let mut manager = OverflowManager::new();
        let data = vec![0x42u8; 100];

        let page_id = manager.allocate_page(&data, None);
        let read = manager.read_chain(page_id).unwrap();

        assert_eq!(read.len(), 100);
        assert!(read.iter().all(|&b| b == 0x42));
    }

    #[test]
    fn test_read_chain_multi_page() {
        let mut manager = OverflowManager::new();
        let mut data = vec![0u8; OVERFLOW_DATA_SIZE * 2 + 100];
        for (i, b) in data.iter_mut().enumerate() {
            *b = (i % 256) as u8;
        }

        // Manually create chain
        let page_id1 = manager.allocate_page(&data[..OVERFLOW_DATA_SIZE], Some(2));
        let page_id2 =
            manager.allocate_page(&data[OVERFLOW_DATA_SIZE..OVERFLOW_DATA_SIZE + 100], None);

        let read = manager.read_chain(page_id1).unwrap();
        assert_eq!(read.len(), OVERFLOW_DATA_SIZE + 100);
    }

    #[test]
    fn test_deallocate_chain() {
        let mut manager = OverflowManager::new();
        let data = vec![0u8; 100];

        let page_id = manager.allocate_page(&data, None);
        assert!(manager.pages.contains_key(&page_id));

        manager.deallocate_chain(page_id).unwrap();
        assert!(!manager.pages.contains_key(&page_id));
    }

    #[test]
    fn test_verify_chain_acyclic() {
        let mut manager = OverflowManager::new();
        let data = vec![0u8; 100];

        let page_id = manager.allocate_page(&data, None);
        assert!(manager.verify_chain_acyclic(page_id));
    }

    #[test]
    fn test_verify_broken_chain() {
        let manager = OverflowManager::new();
        // Page 999 doesn't exist
        assert!(!manager.verify_chain_acyclic(999));
    }

    #[test]
    fn test_encode_overflow_chain_single_page() {
        let mut manager = OverflowManager::new();
        let data = vec![0xAB; 50];

        let (page_id, count) = encode_overflow_chain(&mut manager, &data);
        assert_eq!(count, 1);

        let read = decode_overflow_chain(&manager, page_id).unwrap();
        assert_eq!(read, data);
    }

    #[test]
    fn test_encode_overflow_chain_multi_page() {
        let mut manager = OverflowManager::new();
        let data = vec![0xCD; OVERFLOW_DATA_SIZE + 500];

        let (page_id, count) = encode_overflow_chain(&mut manager, &data);
        // Should be at least 2 pages
        assert!(count >= 2);

        let read = decode_overflow_chain(&manager, page_id).unwrap();
        assert_eq!(read.len(), data.len());
    }

    #[test]
    fn test_get_page_data() {
        let data = vec![0u8; PAGE_SIZE];
        let page_data = OverflowManager::get_page_data(&data);
        assert_eq!(page_data.len(), PAGE_SIZE - OVERFLOW_HEADER_SIZE);
    }

    #[test]
    fn test_get_next_page() {
        let mut page_data = vec![0u8; PAGE_SIZE];
        // Set next_page to 42
        page_data[0..4].copy_from_slice(&42u32.to_le_bytes());

        assert_eq!(OverflowManager::get_next_page(&page_data), Some(42));
    }

    #[test]
    fn test_get_next_page_none() {
        let page_data = vec![0u8; PAGE_SIZE];
        // next_page = 0 means None
        assert_eq!(OverflowManager::get_next_page(&page_data), None);
    }
}
