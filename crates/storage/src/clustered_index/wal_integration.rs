//! WAL integration for clustered index.
//!
//! This module provides WAL integration for ClusteredLeafPage operations.
//! All modifications are logged to WAL before being applied for crash recovery.

use crate::row_format::types::ClusterKey;
use crate::wal::{WalEntry, WalEntryType, WalManager};
use sqlrustgo_types::Value;
use std::io::{Error, ErrorKind};

/// Clustered index WAL entry for recovery.
#[derive(Debug, Clone)]
pub struct ClusteredWalEntry {
    /// The WAL entry
    pub entry: WalEntry,
    /// Decoded cluster key (if available)
    pub cluster_key: Option<ClusterKey>,
    /// Decoded fixed columns (if available)
    pub fixed_columns: Option<Vec<Value>>,
    /// Decoded varlen columns (if available)
    pub varlen_columns: Option<Vec<Option<Vec<u8>>>>,
    /// Decoded null bitmap (if available)
    pub null_bitmap: Option<Vec<bool>>,
}

impl ClusteredWalEntry {
    /// Create from a generic WAL entry by decoding the data.
    pub fn from_wal_entry(entry: WalEntry) -> Self {
        let (fixed_columns, varlen_columns, null_bitmap) = if let Some(ref data) = entry.data {
            decode_row_components(data).unwrap_or((None, None, None))
        } else {
            (None, None, None)
        };

        let cluster_key = if let Some(ref key) = entry.key {
            decode_cluster_key(key).ok()
        } else {
            None
        };

        Self {
            entry,
            cluster_key,
            fixed_columns,
            varlen_columns,
            null_bitmap,
        }
    }

    /// Check if this is an insert operation.
    pub fn is_insert(&self) -> bool {
        self.entry.entry_type == WalEntryType::Insert
    }

    /// Check if this is a delete operation.
    pub fn is_delete(&self) -> bool {
        self.entry.entry_type == WalEntryType::Delete
    }

    /// Check if this is an update operation.
    pub fn is_update(&self) -> bool {
        self.entry.entry_type == WalEntryType::Update
    }
}

/// Decode cluster key from bytes.
fn decode_cluster_key(data: &[u8]) -> std::io::Result<ClusterKey> {
    if data.is_empty() {
        return Err(Error::new(ErrorKind::InvalidData, "Empty cluster key data"));
    }

    let tag = data[0];
    match tag {
        0 => {
            // PrimaryKey - type marker is at data[1]
            if data.len() < 2 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Invalid PrimaryKey data",
                ));
            }
            let value_type = data[1];
            match value_type {
                2 => {
                    // Integer
                    if data.len() < 10 {
                        return Err(Error::new(ErrorKind::InvalidData, "Invalid Integer key"));
                    }
                    let val = i64::from_le_bytes([
                        data[2], data[3], data[4], data[5], data[6], data[7], data[8], data[9],
                    ]);
                    Ok(ClusterKey::PrimaryKey(Value::Integer(val)))
                }
                4 => {
                    // Text
                    if data.len() < 6 {
                        return Err(Error::new(ErrorKind::InvalidData, "Invalid Text key"));
                    }
                    let len = u32::from_le_bytes([data[2], data[3], data[4], data[5]]) as usize;
                    if data.len() < 6 + len {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Invalid Text key length",
                        ));
                    }
                    let s = String::from_utf8(data[6..6 + len].to_vec())
                        .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8 in key"))?;
                    Ok(ClusterKey::PrimaryKey(Value::Text(s)))
                }
                _ => Err(Error::new(
                    ErrorKind::InvalidData,
                    "Unsupported value type for key",
                )),
            }
        }
        1 => {
            // HiddenRowId
            if data.len() < 9 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Invalid HiddenRowId data",
                ));
            }
            let id = u64::from_le_bytes([
                data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
            ]);
            Ok(ClusterKey::HiddenRowId(id))
        }
        _ => Err(Error::new(
            ErrorKind::InvalidData,
            "Invalid cluster key tag",
        )),
    }
}

/// Decode row components from encoded data.
#[allow(clippy::type_complexity)]
fn decode_row_components(
    data: &[u8],
) -> std::io::Result<(
    Option<Vec<Value>>,
    Option<Vec<Option<Vec<u8>>>>,
    Option<Vec<bool>>,
)> {
    use crate::row_format::decoder::decode_row;

    // For recovery, we need fixed_column_count and varlen_column_count
    // These would typically come from the table schema, which we don't have here
    // So we do a best-effort decode - just get the cluster key for now
    let _ = decode_row(data, 0, 0)?;

    // Full decode would require schema information
    Ok((None, None, None))
}

/// Encode cluster key to bytes for WAL storage.
pub fn encode_cluster_key_for_wal(key: &ClusterKey) -> Vec<u8> {
    let mut buf = Vec::new();
    crate::row_format::encoder::encode_cluster_key(&mut buf, key).unwrap();
    buf
}

/// Encode full row data for WAL storage.
pub fn encode_row_for_wal(
    key: &ClusterKey,
    fixed_columns: &[Value],
    varlen_columns: &[Option<Vec<u8>>],
    null_bitmap: &[bool],
) -> std::io::Result<Vec<u8>> {
    crate::row_format::encoder::encode_row(key, fixed_columns, varlen_columns, null_bitmap)
}

/// Clustered index WAL manager for logging and recovery.
pub struct ClusteredWalManager {
    /// Inner WAL manager
    inner: WalManager,
    /// Table ID for this clustered index
    table_id: u64,
}

impl ClusteredWalManager {
    /// Create a new clustered WAL manager.
    pub fn new(wal_path: std::path::PathBuf, table_id: u64) -> Self {
        Self {
            inner: WalManager::new(wal_path),
            table_id,
        }
    }

    /// Get the table ID.
    pub fn table_id(&self) -> u64 {
        self.table_id
    }

    /// Log a begin transaction.
    pub fn log_begin(&self, tx_id: u64) -> std::io::Result<u64> {
        self.inner.log_begin(tx_id)
    }

    /// Log a commit.
    pub fn log_commit(&self, tx_id: u64) -> std::io::Result<u64> {
        self.inner.log_commit(tx_id)
    }

    /// Log an insert operation.
    pub fn log_insert(
        &self,
        tx_id: u64,
        cluster_key: &ClusterKey,
        fixed_columns: &[Value],
        varlen_columns: &[Option<Vec<u8>>],
        null_bitmap: &[bool],
    ) -> std::io::Result<u64> {
        let key = encode_cluster_key_for_wal(cluster_key);
        let data = encode_row_for_wal(cluster_key, fixed_columns, varlen_columns, null_bitmap)?;
        self.inner.log_insert(tx_id, self.table_id, key, data)
    }

    /// Log an update operation.
    pub fn log_update(
        &self,
        tx_id: u64,
        cluster_key: &ClusterKey,
        fixed_columns: &[Value],
        varlen_columns: &[Option<Vec<u8>>],
        null_bitmap: &[bool],
    ) -> std::io::Result<u64> {
        let key = encode_cluster_key_for_wal(cluster_key);
        let data = encode_row_for_wal(cluster_key, fixed_columns, varlen_columns, null_bitmap)?;
        self.inner.log_update(tx_id, self.table_id, key, data)
    }

    /// Log a delete operation.
    pub fn log_delete(&self, tx_id: u64, cluster_key: &ClusterKey) -> std::io::Result<u64> {
        let key = encode_cluster_key_for_wal(cluster_key);
        self.inner.log_delete(tx_id, self.table_id, key)
    }

    /// Recover all entries for this table.
    pub fn recover(&self) -> std::io::Result<Vec<ClusteredWalEntry>> {
        Ok(self
            .inner
            .recover()?
            .into_iter()
            .filter(|e| {
                e.table_id == self.table_id
                    || e.entry_type == WalEntryType::Begin
                    || e.entry_type == WalEntryType::Commit
            })
            .map(ClusteredWalEntry::from_wal_entry)
            .collect())
    }

    /// Recover to a specific timestamp (PITR).
    pub fn recover_to_timestamp(&self, timestamp: u64) -> std::io::Result<Vec<ClusteredWalEntry>> {
        let entries = self.inner.recover_to_timestamp(timestamp)?;
        Ok(entries
            .into_iter()
            .filter(|e| {
                e.table_id == self.table_id
                    || e.entry_type == WalEntryType::Begin
                    || e.entry_type == WalEntryType::Commit
            })
            .map(ClusteredWalEntry::from_wal_entry)
            .collect())
    }

    /// Get the inner WAL manager.
    pub fn inner(&self) -> &WalManager {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_cluster_key_hidden() {
        let key = ClusterKey::HiddenRowId(12345);
        let encoded = encode_cluster_key_for_wal(&key);
        let decoded = decode_cluster_key(&encoded).unwrap();

        assert_eq!(decoded, key);
    }

    #[test]
    fn test_encode_decode_cluster_key_pk_integer() {
        let key = ClusterKey::PrimaryKey(Value::Integer(999));
        let encoded = encode_cluster_key_for_wal(&key);
        let decoded = decode_cluster_key(&encoded).unwrap();

        assert_eq!(decoded, key);
    }

    #[test]
    fn test_clustered_wal_entry_from_wal_entry() {
        let entry = WalEntry {
            tx_id: 1,
            entry_type: WalEntryType::Insert,
            table_id: 100,
            key: Some(vec![1, 0, 0, 0, 0, 0, 0, 0, 0]), // HiddenRowId(0)
            data: Some(vec![]),
            lsn: 0,
            timestamp: 0,
        };

        let clustered = ClusteredWalEntry::from_wal_entry(entry.clone());

        assert!(clustered.is_insert());
        assert!(!clustered.is_delete());
        assert_eq!(clustered.cluster_key, Some(ClusterKey::HiddenRowId(0)));
    }

    #[test]
    fn test_clustered_wal_entry_is_delete() {
        let entry = WalEntry {
            tx_id: 1,
            entry_type: WalEntryType::Delete,
            table_id: 100,
            key: Some(vec![1, 42, 0, 0, 0, 0, 0, 0, 0]), // HiddenRowId(42)
            data: None,
            lsn: 0,
            timestamp: 0,
        };

        let clustered = ClusteredWalEntry::from_wal_entry(entry);

        assert!(clustered.is_delete());
        assert!(!clustered.is_insert());
        assert_eq!(clustered.cluster_key, Some(ClusterKey::HiddenRowId(42)));
    }
}
