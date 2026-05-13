//! Row encoder for Compact Row v1 format.

use crate::row_format::types::ClusteredLeafRecord;

/// Encode a clustered leaf record to bytes.
pub fn encode_row(record: &ClusteredLeafRecord) -> Vec<u8> {
    // TODO: Implement encoder
    Vec::new()
}