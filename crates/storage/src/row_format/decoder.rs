//! Row decoder for Compact Row v1 format.

use crate::row_format::types::ClusteredLeafRecord;
use crate::row_format::types::ClusterKey;
use crate::row_format::types::RowHeader;

/// Decode a clustered leaf record from bytes.
pub fn decode_row(data: &[u8]) -> Option<ClusteredLeafRecord> {
    // TODO: Implement decoder
    None
}