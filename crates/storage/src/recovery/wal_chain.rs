//! WAL Hash Chain Module
//!
//! Provides WAL entries with cryptographic hash chain (prev_hash, current_hash)
//! for tamper detection and recovery verification.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Genesis previous hash (all zeros for first entry)
pub const WAL_GENESIS_PREV_HASH: [u8; 32] = [0u8; 32];

/// WAL entry with hash chain for tamper detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalChainEntry {
    /// Log sequence number
    pub lsn: u64,
    /// Hash of previous entry (zero for first entry)
    pub prev_hash: [u8; 32],
    /// SHA-256 hash of this entry (excluding checksum field)
    pub current_hash: [u8; 32],
    /// Transaction ID
    pub tx_id: u64,
    /// Entry type (Begin/Insert/Update/Delete/Commit/Rollback/Checkpoint)
    pub entry_type: u8,
    /// Table ID
    pub table_id: u64,
    /// Row key (optional)
    pub key: Option<Vec<u8>>,
    /// Row data (optional)
    pub data: Option<Vec<u8>>,
    /// Timestamp (Unix epoch ms)
    pub timestamp: u64,
}

impl WalChainEntry {
    /// Create a new WAL chain entry with computed hash
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        lsn: u64,
        prev_hash: [u8; 32],
        tx_id: u64,
        entry_type: u8,
        table_id: u64,
        key: Option<Vec<u8>>,
        data: Option<Vec<u8>>,
        timestamp: u64,
    ) -> Self {
        let mut entry = Self {
            lsn,
            prev_hash,
            current_hash: [0u8; 32],
            tx_id,
            entry_type,
            table_id,
            key,
            data,
            timestamp,
        };
        entry.current_hash = entry.compute_hash();
        entry
    }

    /// Compute SHA-256 hash of this entry (excluding current_hash field)
    #[allow(clippy::needless_borrows_for_generic_args)]
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.lsn.to_le_bytes());
        hasher.update(self.prev_hash);
        hasher.update(self.tx_id.to_le_bytes());
        hasher.update(self.entry_type.to_le_bytes());
        hasher.update(self.table_id.to_le_bytes());
        hasher.update(self.timestamp.to_le_bytes());
        if let Some(ref k) = self.key {
            hasher.update((k.len() as u64).to_le_bytes());
            hasher.update(k);
        }
        if let Some(ref d) = self.data {
            hasher.update((d.len() as u64).to_le_bytes());
            hasher.update(d);
        }
        hasher.finalize().into()
    }

    /// Verify hash chain integrity
    pub fn verify_chain_integrity(&self, expected_prev_hash: &[u8; 32]) -> bool {
        if self.prev_hash != *expected_prev_hash {
            return false;
        }
        let computed = self.compute_hash();
        computed == self.current_hash
    }
}

/// WAL chain state for tracking chain continuity
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WalChainState {
    pub next_lsn: u64,
    pub last_hash: [u8; 32],
    pub entry_count: u64,
}

impl WalChainState {
    pub fn new() -> Self {
        Self {
            next_lsn: 1,
            last_hash: WAL_GENESIS_PREV_HASH,
            entry_count: 0,
        }
    }

    pub fn append_entry(&mut self, entry: &WalChainEntry) {
        self.last_hash = entry.current_hash;
        self.next_lsn = entry.lsn + 1;
        self.entry_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wal_chain_entry_creation() {
        let entry = WalChainEntry::new(
            1,
            WAL_GENESIS_PREV_HASH,
            100,
            1, // Begin
            1,
            None,
            None,
            1609459200000,
        );

        assert_eq!(entry.lsn, 1);
        assert_eq!(entry.prev_hash, WAL_GENESIS_PREV_HASH);
        assert_ne!(entry.current_hash, [0u8; 32]);
    }

    #[test]
    fn test_wal_chain_entry_hash_computation() {
        let entry = WalChainEntry::new(
            1,
            WAL_GENESIS_PREV_HASH,
            100,
            1,
            1,
            Some(vec![1, 2, 3]),
            Some(vec![4, 5, 6]),
            1609459200000,
        );

        let computed = entry.compute_hash();
        assert_eq!(computed, entry.current_hash);
    }

    #[test]
    fn test_wal_chain_entry_verify_integrity() {
        let entry = WalChainEntry::new(
            1,
            WAL_GENESIS_PREV_HASH,
            100,
            1,
            1,
            None,
            None,
            1609459200000,
        );

        assert!(entry.verify_chain_integrity(&WAL_GENESIS_PREV_HASH));
        assert!(!entry.verify_chain_integrity(&[1u8; 32]));
    }

    #[test]
    fn test_wal_chain_state() {
        let mut state = WalChainState::new();
        assert_eq!(state.next_lsn, 1);
        assert_eq!(state.entry_count, 0);

        let entry = WalChainEntry::new(
            1,
            WAL_GENESIS_PREV_HASH,
            100,
            1,
            1,
            None,
            None,
            1609459200000,
        );

        state.append_entry(&entry);
        assert_eq!(state.next_lsn, 2);
        assert_eq!(state.entry_count, 1);
        assert_eq!(state.last_hash, entry.current_hash);
    }
}
