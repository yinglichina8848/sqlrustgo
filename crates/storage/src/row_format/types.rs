//! Core type definitions for SQLRustGo Compact Row v1 Format ABI.
//!
//! This module provides the fundamental types used by clustered index row format.

use serde::{Deserialize, Serialize};
use sqlrustgo_types::Value;

/// Row header for Compact Row v1 format.
/// Always 19 bytes - this size is ABI-stable and will never change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct RowHeader {
    /// Format version (v1 = 1)
    pub format_version: u8,
    /// Flags for future use (reserved)
    pub flags: u16,
    /// Transaction ID (Phase A = 0)
    pub trx_id: u64,
    /// Undo pointer (Phase A = 0)
    pub undo_ptr: u64,
}

impl RowHeader {
    /// Current format version = 1
    pub const CURRENT_VERSION: u8 = 1;

    /// Fixed size of RowHeader in bytes = 19
    pub const SIZE: usize = 19;

    /// Create a new RowHeader with default Phase A values.
    pub fn new() -> Self {
        Self {
            format_version: Self::CURRENT_VERSION,
            flags: 0,
            trx_id: 0,
            undo_ptr: 0,
        }
    }
}

impl Default for RowHeader {
    fn default() -> Self {
        Self::new()
    }
}

/// Cluster key for identifying rows in a clustered index.
/// Can be either a primary key value or a hidden row ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClusterKey {
    /// Primary key value from sqlrustgo_types::Value
    PrimaryKey(Value),
    /// Hidden row ID - stable logical identity (used when no primary key defined)
    HiddenRowId(u64),
}

impl PartialOrd for ClusterKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ClusterKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            // HiddenRowId always comes before PrimaryKey (for btree ordering)
            (ClusterKey::HiddenRowId(lhs), ClusterKey::HiddenRowId(rhs)) => lhs.cmp(rhs),
            (ClusterKey::HiddenRowId(_), ClusterKey::PrimaryKey(_)) => std::cmp::Ordering::Less,
            (ClusterKey::PrimaryKey(_), ClusterKey::HiddenRowId(_)) => std::cmp::Ordering::Greater,
            (ClusterKey::PrimaryKey(lhs), ClusterKey::PrimaryKey(rhs)) => lhs.cmp(rhs),
        }
    }
}

/// Trait for generating unique row IDs.
pub trait RowIdGenerator {
    /// Generate the next unique row ID.
    fn next_id(&mut self) -> u64;
}

/// Default row ID generator using node_id + local counter.
/// Generates IDs in format: (node_id << 48) | local_counter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefaultRowIdGenerator {
    /// Node ID (upper 16 bits)
    pub node_id: u16,
    /// Local counter (lower 48 bits)
    pub local_counter: u64,
}

impl DefaultRowIdGenerator {
    /// Create a new generator with the given node ID.
    pub fn new(node_id: u16) -> Self {
        Self {
            node_id,
            local_counter: 0,
        }
    }
}

impl RowIdGenerator for DefaultRowIdGenerator {
    fn next_id(&mut self) -> u64 {
        let id = (self.node_id as u64) << 48 | self.local_counter;
        self.local_counter = self.local_counter.wrapping_add(1);
        id
    }
}

/// Variable-length slot for storing inline or overflow data.
/// Uses 128-byte inline threshold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VarLenSlot {
    /// Inline data length (0 = NULL)
    pub inline_len: u16,
    /// Inline data (only if <= 128 bytes)
    pub inline_data: Option<Vec<u8>>,
    /// Overflow page number (if data > 128 bytes)
    pub overflow_page: Option<u32>,
    /// Overflow data length
    pub overflow_len: Option<u32>,
}

impl VarLenSlot {
    /// Threshold for inline storage (128 bytes)
    pub const VARLEN_INLINE_THRESHOLD: usize = 128;

    /// Create a new slot from data.
    /// If data is > 128 bytes, overflow_page and overflow_len must be set separately.
    pub fn new(data: &[u8]) -> Self {
        if data.is_empty() {
            return Self {
                inline_len: 0,
                inline_data: None,
                overflow_page: None,
                overflow_len: None,
            };
        }

        if data.len() <= Self::VARLEN_INLINE_THRESHOLD {
            Self {
                inline_len: data.len() as u16,
                inline_data: Some(data.to_vec()),
                overflow_page: None,
                overflow_len: None,
            }
        } else {
            // For overflow data, caller must set overflow_page and overflow_len separately
            Self {
                inline_len: 0,
                inline_data: None,
                overflow_page: None,
                overflow_len: Some(data.len() as u32),
            }
        }
    }

    /// Check if this slot represents a NULL value.
    pub fn is_null(&self) -> bool {
        // A slot is null if it has no inline data AND no overflow (not just zero inline_len)
        self.inline_data.is_none() && self.overflow_page.is_none() && self.overflow_len.is_none()
    }
}

/// Clustered leaf record structure.
/// Contains all components of a row in a clustered index leaf page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusteredLeafRecord {
    /// Row header
    pub header: RowHeader,
    /// Cluster key (primary key or hidden row ID)
    pub cluster_key: ClusterKey,
    /// Fixed-length data
    pub fixed_data: Vec<u8>,
    /// Variable-length slots
    pub varlen_slots: Vec<VarLenSlot>,
    /// Null bitmap
    pub null_bitmap: Vec<u8>,
}

impl ClusteredLeafRecord {
    /// Create a new clustered leaf record.
    pub fn new(cluster_key: ClusterKey) -> Self {
        Self {
            header: RowHeader::new(),
            cluster_key,
            fixed_data: Vec::new(),
            varlen_slots: Vec::new(),
            null_bitmap: Vec::new(),
        }
    }
}

/// Overflow page for storing data that exceeds inline threshold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverflowPage {
    /// Next overflow page (None = last page)
    pub next_page: Option<u32>,
    /// Data on this page
    pub data: Vec<u8>,
}

impl OverflowPage {
    /// Header size = 4 bytes (for next_page)
    pub const HEADER_SIZE: usize = 4;

    /// Create a new overflow page.
    pub fn new(data: Vec<u8>, next_page: Option<u32>) -> Self {
        Self { next_page, data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_header_size() {
        assert_eq!(RowHeader::SIZE, 19);
    }

    #[test]
    fn test_row_header_default() {
        let header = RowHeader::default();
        assert_eq!(header.format_version, 1);
        assert_eq!(header.flags, 0);
        assert_eq!(header.trx_id, 0);
        assert_eq!(header.undo_ptr, 0);
    }

    #[test]
    fn test_varlen_slot_inline() {
        let data = vec![0u8; 50];
        let slot = VarLenSlot::new(&data);
        assert!(slot.inline_data.is_some());
        assert_eq!(slot.inline_len, 50);
        assert!(!slot.is_null());
    }

    #[test]
    fn test_varlen_slot_overflow() {
        let data = vec![0u8; 200];
        let slot = VarLenSlot::new(&data);
        assert!(slot.inline_data.is_none());
        assert_eq!(slot.inline_len, 0);
        assert!(!slot.is_null());
    }

    #[test]
    fn test_hidden_row_id_generator() {
        let mut gen = DefaultRowIdGenerator::new(1);
        let id1 = gen.next_id();
        let id2 = gen.next_id();
        assert_eq!(id2, id1 + 1);
    }
}
