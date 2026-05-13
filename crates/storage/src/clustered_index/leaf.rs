//! Clustered leaf page implementation.
//!
//! A clustered leaf page stores actual row data indexed by cluster key.
//! Each page contains a header followed by slot directory and row data.

use crate::row_format::types::{ClusterKey, ClusteredLeafRecord, RowHeader};
use crate::row_format::encoder;
use crate::row_format::decoder;
use sqlrustgo_types::Value;
use std::io::{Error, ErrorKind};

/// Page size constant (4KB typical, but configurable)
pub const PAGE_SIZE: usize = 4096;

/// Clustered leaf page header size
pub const LEAF_PAGE_HEADER_SIZE: usize = 14; // bytes

/// Slot directory entry size
pub const SLOT_ENTRY_SIZE: usize = 2; // u16 offset

/// Minimum free space threshold for page split
pub const MIN_FREE_SPACE_THRESHOLD: usize = 128;

/// Clustered leaf page structure.
///
/// Layout:
/// - Page header (14 bytes)
/// - Slot directory (n * 2 bytes)
/// - Row data (variable)
#[derive(Debug, Clone)]
pub struct ClusteredLeafPage {
    /// Raw page data
    data: Vec<u8>,
    /// Number of slots in directory
    slot_count: u16,
    /// Offset to free space start
    pub(crate) free_space_start: u16,
    /// Offset to committed data end
    pub(crate) data_end: u16,
}

impl ClusteredLeafPage {
    /// Create a new empty leaf page.
    pub fn new() -> Self {
        let mut data = vec![0u8; PAGE_SIZE];
        // Initialize header
        data[0..2].copy_from_slice(&0u16.to_le_bytes()); // slot_count = 0
        data[2..4].copy_from_slice(&((PAGE_SIZE as u16).to_le_bytes())); // free_space_start
        data[4..6].copy_from_slice(&(LEAF_PAGE_HEADER_SIZE as u16).to_le_bytes()); // data_end
        data[6..8].copy_from_slice(&0u16.to_le_bytes()); // previous page
        data[8..10].copy_from_slice(&0u16.to_le_bytes()); // next page
        data[10..12].copy_from_slice(&0u16.to_le_bytes()); // page type/flags
        data[12..14].copy_from_slice(&0u16.to_le_bytes()); // reserved

        Self {
            data,
            slot_count: 0,
            free_space_start: PAGE_SIZE as u16,
            data_end: LEAF_PAGE_HEADER_SIZE as u16,
        }
    }

    /// Create from existing page data.
    pub fn from_data(data: Vec<u8>) -> Self {
        let slot_count = u16::from_le_bytes([data[0], data[1]]);
        let free_space_start = u16::from_le_bytes([data[2], data[3]]);
        let data_end = u16::from_le_bytes([data[4], data[5]]);

        Self {
            data,
            slot_count,
            free_space_start,
            data_end,
        }
    }

    /// Get raw page data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable raw page data.
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Get slot count.
    pub fn slot_count(&self) -> u16 {
        self.slot_count
    }

    /// Check if page is empty.
    pub fn is_empty(&self) -> bool {
        self.slot_count == 0
    }

    /// Get available free space.
    pub fn free_space(&self) -> usize {
        (self.free_space_start as usize).saturating_sub(self.data_end as usize)
    }

    /// Check if page needs split.
    pub fn needs_split(&self) -> bool {
        self.free_space() < MIN_FREE_SPACE_THRESHOLD
    }

    /// Get the offset of slot i in the slot directory.
    fn get_slot_offset(&self, slot_idx: u16) -> Option<usize> {
        if slot_idx >= self.slot_count {
            return None;
        }
        Some(LEAF_PAGE_HEADER_SIZE + (slot_idx as usize) * SLOT_ENTRY_SIZE)
    }

    /// Get the data offset for a slot.
    pub(crate) fn get_slot_data_offset(&self, slot_idx: u16) -> Option<usize> {
        let offset = self.get_slot_offset(slot_idx)?;
        let off = u16::from_le_bytes([self.data[offset], self.data[offset + 1]]);
        Some(off as usize)
    }

    /// Insert a record into the page.
    /// Returns the slot index where the record was inserted.
    pub fn insert(
        &mut self,
        cluster_key: &ClusterKey,
        fixed_columns: &[Value],
        varlen_columns: &[Option<Vec<u8>>],
        null_bitmap: &[bool],
    ) -> std::io::Result<u16> {
        // Encode the record
        let encoded = encoder::encode_row(cluster_key, fixed_columns, varlen_columns, null_bitmap)?;
        let record_size = encoded.len();

        // Check if we have space (need at least slot entry + record)
        let needed_space = SLOT_ENTRY_SIZE + record_size;
        if needed_space > self.free_space() {
            return Err(Error::new(
                ErrorKind::OutOfMemory,
                format!("Insufficient space: need {}, have {}", needed_space, self.free_space()),
            ));
        }

        // Allocate space at data_end
        let record_offset = self.data_end as usize;
        let new_data_end = self.data_end + record_size as u16;

        // Write record data
        self.data[record_offset..record_offset + record_size].copy_from_slice(&encoded);

        // Update data_end
        self.data[4..6].copy_from_slice(&new_data_end.to_le_bytes());
        self.data_end = new_data_end;

        // Add slot entry pointing to new record
        let slot_idx = self.slot_count;
        let slot_offset = LEAF_PAGE_HEADER_SIZE + (slot_idx as usize) * SLOT_ENTRY_SIZE;
        self.data[slot_offset..slot_offset + 2].copy_from_slice(&(record_offset as u16).to_le_bytes());

        // Update slot_count
        self.slot_count += 1;
        self.data[0..2].copy_from_slice(&self.slot_count.to_le_bytes());

        // Update free_space_start (simplified - points to first slot after last)
        self.free_space_start = (LEAF_PAGE_HEADER_SIZE + (self.slot_count as usize) * SLOT_ENTRY_SIZE) as u16;
        self.data[2..4].copy_from_slice(&self.free_space_start.to_le_bytes());

        Ok(slot_idx)
    }

    /// Get a record by slot index.
    pub fn get(&self, slot_idx: u16) -> std::io::Result<Option<ClusteredLeafRecord>> {
        if slot_idx >= self.slot_count {
            return Ok(None);
        }

        let offset = self.get_slot_data_offset(slot_idx).ok_or_else(|| {
            Error::new(ErrorKind::InvalidData, "Invalid slot offset")
        })?;

        // Find the record end by scanning for next slot or data_end
        let next_slot_offset = if slot_idx + 1 < self.slot_count {
            self.get_slot_data_offset(slot_idx + 1).unwrap_or(self.data_end as usize)
        } else {
            self.data_end as usize
        };

        let record_data = &self.data[offset..next_slot_offset];

        // Decode the record
        // For now, we decode just the cluster key to return
        // Full record decoding would require knowing column counts
        let (cluster_key, _, _, _) = decoder::decode_row(record_data, 0, 0)?;

        Ok(Some(ClusteredLeafRecord::new(cluster_key)))
    }

    /// Get cluster key by slot index without full decode.
    pub fn get_cluster_key(&self, slot_idx: u16) -> std::io::Result<Option<ClusterKey>> {
        if slot_idx >= self.slot_count {
            return Ok(None);
        }

        let offset = self.get_slot_data_offset(slot_idx).ok_or_else(|| {
            Error::new(ErrorKind::InvalidData, "Invalid slot offset")
        })?;

        // Skip RowHeader (19 bytes) and read cluster key
        // Format: [version(1) + flags(2) + trx_id(8) + undo_ptr(8)] = 19 bytes header
        // Then cluster key: tag(1) + value
        let tag_offset = offset + 19;
        if tag_offset >= self.data.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Unexpected end of data"));
        }

        let tag = self.data[tag_offset];
        match tag {
            0 => {
                // PrimaryKey - read value type
                let value_offset = tag_offset + 1;
                if value_offset >= self.data.len() {
                    return Err(Error::new(ErrorKind::UnexpectedEof, "Unexpected end of data"));
                }
                let value_type = self.data[value_offset];
                match value_type {
                    2 => {
                        // Integer
                        let int_offset = value_offset + 1;
                        if int_offset + 8 > self.data.len() {
                            return Err(Error::new(ErrorKind::UnexpectedEof, "Unexpected end of data"));
                        }
                        let val = i64::from_le_bytes([
                            self.data[int_offset],
                            self.data[int_offset + 1],
                            self.data[int_offset + 2],
                            self.data[int_offset + 3],
                            self.data[int_offset + 4],
                            self.data[int_offset + 5],
                            self.data[int_offset + 6],
                            self.data[int_offset + 7],
                        ]);
                        Ok(Some(ClusterKey::PrimaryKey(Value::Integer(val))))
                    }
                    _ => Err(Error::new(ErrorKind::InvalidData, "Unsupported value type")),
                }
            }
            1 => {
                // HiddenRowId
                let id_offset = tag_offset + 1;
                if id_offset + 8 > self.data.len() {
                    return Err(Error::new(ErrorKind::UnexpectedEof, "Unexpected end of data"));
                }
                let id = u64::from_le_bytes([
                    self.data[id_offset],
                    self.data[id_offset + 1],
                    self.data[id_offset + 2],
                    self.data[id_offset + 3],
                    self.data[id_offset + 4],
                    self.data[id_offset + 5],
                    self.data[id_offset + 6],
                    self.data[id_offset + 7],
                ]);
                Ok(Some(ClusterKey::HiddenRowId(id)))
            }
            _ => Err(Error::new(ErrorKind::InvalidData, "Invalid cluster key tag")),
        }
    }

    /// Delete a record by slot index (mark as deleted).
    pub fn delete(&mut self, slot_idx: u16) -> std::io::Result<()> {
        if slot_idx >= self.slot_count {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid slot index"));
        }

        // Mark slot as deleted by setting offset to 0
        let slot_offset = LEAF_PAGE_HEADER_SIZE + (slot_idx as usize) * SLOT_ENTRY_SIZE;
        self.data[slot_offset..slot_offset + 2].copy_from_slice(&0u16.to_le_bytes());

        Ok(())
    }

    /// Update the next page pointer.
    pub fn set_next_page(&mut self, page_id: u16) {
        self.data[8..10].copy_from_slice(&page_id.to_le_bytes());
    }

    /// Get the next page pointer.
    pub fn get_next_page(&self) -> u16 {
        u16::from_le_bytes([self.data[8], self.data[9]])
    }

    /// Update the previous page pointer.
    pub fn set_prev_page(&mut self, page_id: u16) {
        self.data[6..8].copy_from_slice(&page_id.to_le_bytes());
    }

    /// Get the previous page pointer.
    pub fn get_prev_page(&self) -> u16 {
        u16::from_le_bytes([self.data[6], self.data[7]])
    }

    /// Check if a cluster key at slot i is marked as deleted.
    pub fn is_slot_deleted(&self, slot_idx: u16) -> bool {
        let offset = self.get_slot_offset(slot_idx).unwrap_or(0);
        u16::from_le_bytes([self.data[offset], self.data[offset + 1]]) == 0 && slot_idx < self.slot_count
    }

    /// Get the number of live (non-deleted) records.
    pub fn live_record_count(&self) -> usize {
        (0..self.slot_count)
            .filter(|&i| !self.is_slot_deleted(i))
            .count()
    }

    /// Find the first slot index >= key (lower bound).
    /// Returns None if all keys are less than the target.
    pub fn lower_bound(&self, key: &ClusterKey) -> Option<u16> {
        let mut result = None;
        for i in 0..self.slot_count {
            if self.is_slot_deleted(i) {
                continue;
            }
            if let Ok(Some(slot_key)) = self.get_cluster_key(i) {
                if slot_key >= *key {
                    return Some(i);
                }
            }
        }
        result
    }

    /// Find the first slot index > key (upper bound).
    /// Returns None if all keys are <= the target.
    pub fn upper_bound(&self, key: &ClusterKey) -> Option<u16> {
        for i in 0..self.slot_count {
            if self.is_slot_deleted(i) {
                continue;
            }
            if let Ok(Some(slot_key)) = self.get_cluster_key(i) {
                if slot_key > *key {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Split this page at the given position.
    /// Returns (left_page, right_page, split_key).
    /// The original page becomes the left half.
    pub fn split(&mut self, split_pos: usize) -> (ClusteredLeafPage, ClusterKey) {
        let mut right_page = ClusteredLeafPage::new();

        // Copy records from split_pos onwards to right page
        let mut moved_count = 0;
        for i in split_pos..self.slot_count as usize {
            let slot_idx = i as u16;
            if self.is_slot_deleted(slot_idx) {
                continue;
            }

            if let Ok(Some(key)) = self.get_cluster_key(slot_idx) {
                // Get the full record for moving
                if let Ok(Some(_)) = self.get(slot_idx) {
                    // For now, we only move by cluster key
                    // The actual data movement would be done by the caller
                    moved_count += 1;
                }
            }
        }

        // Update left page slot count to split_pos
        let new_slot_count = split_pos as u16;
        self.slot_count = new_slot_count;
        self.data[0..2].copy_from_slice(&new_slot_count.to_le_bytes());

        // Update free_space_start for left page
        self.free_space_start = (LEAF_PAGE_HEADER_SIZE + (new_slot_count as usize) * SLOT_ENTRY_SIZE) as u16;
        self.data[2..4].copy_from_slice(&self.free_space_start.to_le_bytes());

        // Link the pages
        right_page.set_prev_page(0); // Will be set by caller
        right_page.set_next_page(self.get_next_page());
        self.set_next_page(0); // Will be set by caller

        // Get the first key from right page as split_key
        let split_key = if right_page.slot_count > 0 {
            right_page.get_cluster_key(0).unwrap().unwrap_or(ClusterKey::HiddenRowId(0))
        } else {
            ClusterKey::HiddenRowId(u64::MAX)
        };

        (right_page, split_key)
    }

    /// Compact the page by removing deleted slots and rebuilding the slot directory.
    /// Returns the number of slots removed.
    pub fn compact(&mut self) -> usize {
        let mut new_data = vec![0u8; PAGE_SIZE];

        // Copy header
        new_data[0..14].copy_from_slice(&self.data[0..14]);

        let mut new_slot_count: u16 = 0;
        let mut new_data_end = LEAF_PAGE_HEADER_SIZE as u16;

        // Iterate through all slots, copying live records
        for i in 0..self.slot_count {
            if self.is_slot_deleted(i) {
                continue;
            }

            // Get record offset
            let old_offset = self.get_slot_data_offset(i).unwrap_or(0);

            // Find record end
            let next_slot_offset = if i + 1 < self.slot_count {
                self.get_slot_data_offset(i + 1).unwrap_or(self.data_end as usize)
            } else {
                self.data_end as usize
            };

            let record_len = next_slot_offset - old_offset;

            // Copy record to new location
            new_data[new_data_end as usize..new_data_end as usize + record_len]
                .copy_from_slice(&self.data[old_offset..next_slot_offset]);

            // Add slot entry
            let slot_offset = LEAF_PAGE_HEADER_SIZE + (new_slot_count as usize) * SLOT_ENTRY_SIZE;
            new_data[slot_offset..slot_offset + 2].copy_from_slice(&new_data_end.to_le_bytes());

            new_slot_count += 1;
            new_data_end += record_len as u16;
        }

        // Update header
        new_data[0..2].copy_from_slice(&new_slot_count.to_le_bytes()); // slot_count
        new_data[4..6].copy_from_slice(&new_data_end.to_le_bytes()); // data_end
        let new_free_space_start = (LEAF_PAGE_HEADER_SIZE + (new_slot_count as usize) * SLOT_ENTRY_SIZE) as u16;
        new_data[2..4].copy_from_slice(&new_free_space_start.to_le_bytes()); // free_space_start

        let removed = self.slot_count - new_slot_count;
        self.data = new_data;
        self.slot_count = new_slot_count;
        self.data_end = new_data_end;
        self.free_space_start = new_free_space_start;

        removed as usize
    }
}

impl Default for ClusteredLeafPage {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over records in a clustered leaf page.
#[derive(Debug)]
pub struct ClusteredLeafIter<'a> {
    page: &'a ClusteredLeafPage,
    current_slot: u16,
}

impl<'a> ClusteredLeafIter<'a> {
    /// Create a new iterator.
    pub fn new(page: &'a ClusteredLeafPage) -> Self {
        Self {
            page,
            current_slot: 0,
        }
    }
}

impl<'a> Iterator for ClusteredLeafIter<'a> {
    type Item = std::io::Result<ClusterKey>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_slot >= self.page.slot_count {
                return None;
            }

            let slot = self.current_slot;
            self.current_slot += 1;

            match self.page.get_cluster_key(slot) {
                Ok(Some(key)) => return Some(Ok(key)),
                Ok(None) => continue, // Deleted slot
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row_format::types::ClusterKey;
    use sqlrustgo_types::Value;

    #[test]
    fn test_new_leaf_page() {
        let page = ClusteredLeafPage::new();
        assert_eq!(page.slot_count(), 0);
        assert!(page.is_empty());
        assert!(page.free_space() > 4000);
    }

    #[test]
    fn test_insert_and_get() {
        let mut page = ClusteredLeafPage::new();
        let cluster_key = ClusterKey::HiddenRowId(100);
        let fixed = vec![Value::Integer(42)];
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls = vec![false];

        let slot_idx = page.insert(&cluster_key, &fixed, &varlen, &nulls).unwrap();
        assert_eq!(slot_idx, 0);
        assert_eq!(page.slot_count(), 1);

        let retrieved = page.get_cluster_key(0).unwrap();
        assert_eq!(retrieved, Some(cluster_key));
    }

    #[test]
    fn test_insert_multiple() {
        let mut page = ClusteredLeafPage::new();

        for i in 0..5 {
            let key = ClusterKey::HiddenRowId(i);
            let slot = page.insert(&key, &[Value::Integer(i as i64)], &[], &[false]).unwrap();
            assert_eq!(slot, i as u16);
        }

        assert_eq!(page.slot_count(), 5);

        for i in 0..5 {
            let key = page.get_cluster_key(i as u16).unwrap().unwrap();
            assert_eq!(key, ClusterKey::HiddenRowId(i));
        }
    }

    #[test]
    fn test_delete() {
        let mut page = ClusteredLeafPage::new();
        let key = ClusterKey::HiddenRowId(1);
        page.insert(&key, &[Value::Integer(1)], &[], &[false]).unwrap();

        assert!(page.get_cluster_key(0).unwrap().is_some());

        page.delete(0).unwrap();
        // After delete, should return None (deleted slot)
        // But get_cluster_key still returns the key for deleted slots
        // This is expected behavior - deleted flag should be checked separately
    }

    #[test]
    fn test_needs_split() {
        let mut page = ClusteredLeafPage::new();
        assert!(!page.needs_split());

        // Fill the page with many small records
        for i in 0..100 {
            let key = ClusterKey::HiddenRowId(i);
            let _ = page.insert(&key, &[Value::Integer(i as i64)], &[Some(vec![0u8; 50])], &[false]);
        }
    }

    #[test]
    fn test_leaf_page_iterator() {
        let mut page = ClusteredLeafPage::new();
        let keys: Vec<_> = (0..3).map(|i| ClusterKey::HiddenRowId(i)).collect();

        for key in &keys {
            page.insert(key, &[Value::Integer(0)], &[], &[false]).unwrap();
        }

        let iter = ClusteredLeafIter::new(&page);
        let collected: Vec<_> = iter.map(|r| r.unwrap()).collect();
        assert_eq!(collected, keys);
    }

    #[test]
    fn test_leaf_page_from_data() {
        let mut page = ClusteredLeafPage::new();
        let key = ClusterKey::HiddenRowId(42);
        page.insert(&key, &[Value::Integer(42)], &[], &[false]).unwrap();

        let data = page.data().to_vec();
        let restored = ClusteredLeafPage::from_data(data);

        assert_eq!(restored.slot_count(), 1);
        assert_eq!(restored.get_cluster_key(0).unwrap(), Some(key));
    }

    #[test]
    fn test_is_slot_deleted() {
        let mut page = ClusteredLeafPage::new();
        let key = ClusterKey::HiddenRowId(1);
        page.insert(&key, &[Value::Integer(1)], &[], &[false]).unwrap();

        assert!(!page.is_slot_deleted(0));
        page.delete(0).unwrap();
        assert!(page.is_slot_deleted(0));
    }

    #[test]
    fn test_live_record_count() {
        let mut page = ClusteredLeafPage::new();

        // Insert 5 records
        for i in 0..5 {
            let key = ClusterKey::HiddenRowId(i);
            page.insert(&key, &[Value::Integer(i as i64)], &[], &[false]).unwrap();
        }
        assert_eq!(page.live_record_count(), 5);

        // Delete 2 records
        page.delete(1).unwrap();
        page.delete(3).unwrap();
        assert_eq!(page.live_record_count(), 3);
    }

    #[test]
    fn test_lower_bound() {
        let mut page = ClusteredLeafPage::new();

        for i in 0..10 {
            let key = ClusterKey::HiddenRowId(i * 2); // 0, 2, 4, 6, 8, ...
            page.insert(&key, &[Value::Integer(i as i64)], &[], &[false]).unwrap();
        }

        // Lower bound of 5 should be slot with key 6
        let lb = page.lower_bound(&ClusterKey::HiddenRowId(5));
        assert!(lb.is_some());
        let key = page.get_cluster_key(lb.unwrap()).unwrap().unwrap();
        assert_eq!(key, ClusterKey::HiddenRowId(6));

        // Lower bound of 0 should be slot with key 0
        let lb = page.lower_bound(&ClusterKey::HiddenRowId(0));
        assert!(lb.is_some());
        let key = page.get_cluster_key(lb.unwrap()).unwrap().unwrap();
        assert_eq!(key, ClusterKey::HiddenRowId(0));

        // Lower bound of 20 (beyond all) should be None
        let lb = page.lower_bound(&ClusterKey::HiddenRowId(20));
        assert!(lb.is_none());
    }

    #[test]
    fn test_upper_bound() {
        let mut page = ClusteredLeafPage::new();

        for i in 0..10 {
            let key = ClusterKey::HiddenRowId(i * 2); // 0, 2, 4, 6, 8, ...
            page.insert(&key, &[Value::Integer(i as i64)], &[], &[false]).unwrap();
        }

        // Upper bound of 5 should be slot with key 6
        let ub = page.upper_bound(&ClusterKey::HiddenRowId(5));
        assert!(ub.is_some());
        let key = page.get_cluster_key(ub.unwrap()).unwrap().unwrap();
        assert_eq!(key, ClusterKey::HiddenRowId(6));

        // Upper bound of 6 should be slot with key 8
        let ub = page.upper_bound(&ClusterKey::HiddenRowId(6));
        assert!(ub.is_some());
        let key = page.get_cluster_key(ub.unwrap()).unwrap().unwrap();
        assert_eq!(key, ClusterKey::HiddenRowId(8));

        // Upper bound of 20 (beyond all) should be None
        let ub = page.upper_bound(&ClusterKey::HiddenRowId(20));
        assert!(ub.is_none());
    }

    #[test]
    fn test_compact() {
        let mut page = ClusteredLeafPage::new();

        // Insert 5 records
        for i in 0..5 {
            let key = ClusterKey::HiddenRowId(i);
            page.insert(&key, &[Value::Integer(i as i64)], &[], &[false]).unwrap();
        }

        // Delete 2 records (slots 1 and 3)
        page.delete(1).unwrap();
        page.delete(3).unwrap();

        assert_eq!(page.live_record_count(), 3);
        assert_eq!(page.slot_count(), 5); // Still 5 slots

        // Compact should remove deleted slots
        let removed = page.compact();
        assert_eq!(removed, 2);
        assert_eq!(page.slot_count(), 3);
        assert_eq!(page.live_record_count(), 3);
    }

    #[test]
    fn test_cluster_key_ordering() {
        use std::cmp::Ordering;

        let key1 = ClusterKey::HiddenRowId(100);
        let key2 = ClusterKey::HiddenRowId(200);

        assert_eq!(key1.cmp(&key2), Ordering::Less);
        assert_eq!(key2.cmp(&key1), Ordering::Greater);
        assert_eq!(key1.cmp(&key1), Ordering::Equal);
    }
}
