//! GMP Audit Chain WAL Integration
//!
//! Provides persistent storage for audit chains via Write-Ahead Log.
//! Ensures audit entries are durable before transaction commits.

use crate::audit_chain::{AuditChain, AuditChainEntry, AuditChainError, AuditChainState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

/// WAL magic number for audit chain
const AUDIT_CHAIN_WAL_MAGIC: u32 = 0x41444301; // "ACD" + version 1
/// WAL version
const AUDIT_CHAIN_WAL_VERSION: u16 = 1;

/// Audit chain WAL entry types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AuditChainWalEntryType {
    /// Add audit entry to chain
    Append = 1,
    /// Checkpoint - marks state snapshot
    Checkpoint = 2,
    /// Truncate - remove entries before a sequence number
    Truncate = 3,
}

/// Audit chain WAL entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditChainWalEntry {
    /// Entry type
    pub entry_type: AuditChainWalEntryType,
    /// Sequence number (for Append)
    pub seq: u64,
    /// Serialized audit entry (for Append)
    pub entry_data: Option<Vec<u8>>,
    /// State snapshot (for Checkpoint)
    pub state_data: Option<AuditChainState>,
    /// Truncate before seq (for Truncate)
    pub truncate_before_seq: Option<u64>,
    /// LSN (Log Sequence Number)
    pub lsn: u64,
    /// Timestamp
    pub timestamp: u64,
}

impl AuditChainWalEntry {
    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Magic (4 bytes)
        bytes.extend_from_slice(&AUDIT_CHAIN_WAL_MAGIC.to_le_bytes());
        // Version (2 bytes)
        bytes.extend_from_slice(&AUDIT_CHAIN_WAL_VERSION.to_le_bytes());
        // Entry type (1 byte)
        bytes.push(self.entry_type as u8);
        // LSN (8 bytes)
        bytes.extend_from_slice(&self.lsn.to_le_bytes());
        // Timestamp (8 bytes)
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        // Seq (8 bytes)
        bytes.extend_from_slice(&self.seq.to_le_bytes());

        // Entry data
        match &self.entry_data {
            Some(data) => {
                bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
                bytes.extend_from_slice(data);
            }
            None => {
                bytes.extend_from_slice(&0u32.to_le_bytes());
            }
        }

        // State data
        match &self.state_data {
            Some(state) => {
                let state_bytes = serde_json::to_vec(state).unwrap_or_default();
                bytes.extend_from_slice(&(state_bytes.len() as u32).to_le_bytes());
                bytes.extend_from_slice(&state_bytes);
            }
            None => {
                bytes.extend_from_slice(&0u32.to_le_bytes());
            }
        }

        // Truncate before seq
        match self.truncate_before_seq {
            Some(seq) => {
                bytes.push(1u8);
                bytes.extend_from_slice(&seq.to_le_bytes());
            }
            None => {
                bytes.push(0u8);
            }
        }

        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 35 {
            return None;
        }

        let mut offset = 0;

        // Magic
        let magic = u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?);
        if magic != AUDIT_CHAIN_WAL_MAGIC {
            return None;
        }
        offset += 4;

        // Version
        let _version = u16::from_le_bytes(bytes[offset..offset + 2].try_into().ok()?);
        offset += 2;

        // Entry type
        let entry_type = match bytes[offset] {
            1 => AuditChainWalEntryType::Append,
            2 => AuditChainWalEntryType::Checkpoint,
            3 => AuditChainWalEntryType::Truncate,
            _ => return None,
        };
        offset += 1;

        // LSN
        let lsn = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        offset += 8;

        // Timestamp
        let timestamp = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        offset += 8;

        // Seq
        let seq = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        offset += 8;

        // Entry data
        let entry_data_len =
            u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?) as usize;
        offset += 4;
        let entry_data = if entry_data_len > 0 {
            Some(bytes[offset..offset + entry_data_len].to_vec())
        } else {
            None
        };
        if entry_data_len > 0 {
            offset += entry_data_len;
        }

        // State data
        if offset + 4 > bytes.len() {
            return None;
        }
        let state_data_len =
            u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?) as usize;
        offset += 4;
        let state_data = if state_data_len > 0 {
            if offset + state_data_len > bytes.len() {
                return None;
            }
            let state_json = std::str::from_utf8(&bytes[offset..offset + state_data_len]).ok()?;
            serde_json::from_str(state_json).ok()
        } else {
            None
        };
        if state_data_len > 0 {
            offset += state_data_len;
        }

        // Truncate before seq
        if offset + 1 > bytes.len() {
            return None;
        }
        let has_truncate = bytes[offset];
        offset += 1;
        let truncate_before_seq = if has_truncate == 1 {
            if offset + 8 > bytes.len() {
                return None;
            }
            Some(u64::from_le_bytes(
                bytes[offset..offset + 8].try_into().ok()?,
            ))
        } else {
            None
        };

        Some(Self {
            entry_type,
            seq,
            entry_data,
            state_data,
            truncate_before_seq,
            lsn,
            timestamp,
        })
    }
}

/// Audit chain WAL writer
pub struct AuditChainWalWriter {
    writer: BufWriter<File>,
    lsn: u64,
    path: PathBuf,
}

impl AuditChainWalWriter {
    /// Create new WAL writer
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(&path)?;

        let writer = BufWriter::new(file);

        Ok(Self {
            writer,
            lsn: 0,
            path,
        })
    }

    /// Append an audit entry to WAL
    pub fn append_entry(&mut self, entry: &AuditChainEntry) -> std::io::Result<u64> {
        let wal_entry = AuditChainWalEntry {
            entry_type: AuditChainWalEntryType::Append,
            seq: entry.seq,
            entry_data: Some(serde_json::to_vec(entry).unwrap_or_default()),
            state_data: None,
            truncate_before_seq: None,
            lsn: self.lsn,
            timestamp: entry.timestamp as u64,
        };

        let bytes = wal_entry.to_bytes();
        self.writer.write_all(&(bytes.len() as u32).to_le_bytes())?;
        self.writer.write_all(&bytes)?;
        self.writer.flush()?;

        let current_lsn = self.lsn;
        self.lsn += 1;
        Ok(current_lsn)
    }

    /// Write checkpoint with current state
    pub fn checkpoint(&mut self, state: &AuditChainState) -> std::io::Result<u64> {
        let wal_entry = AuditChainWalEntry {
            entry_type: AuditChainWalEntryType::Checkpoint,
            seq: 0,
            entry_data: None,
            state_data: Some(state.clone()),
            truncate_before_seq: None,
            lsn: self.lsn,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let bytes = wal_entry.to_bytes();
        self.writer.write_all(&(bytes.len() as u32).to_le_bytes())?;
        self.writer.write_all(&bytes)?;
        self.writer.flush()?;

        let current_lsn = self.lsn;
        self.lsn += 1;
        Ok(current_lsn)
    }

    /// Flush pending writes
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }

    /// Get current LSN
    pub fn current_lsn(&self) -> u64 {
        self.lsn
    }
}

/// Audit chain WAL reader
pub struct AuditChainWalReader {
    reader: BufReader<File>,
    path: PathBuf,
}

impl AuditChainWalReader {
    /// Create new WAL reader
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        let file = OpenOptions::new().read(true).open(&path)?;
        let reader = BufReader::new(file);

        Ok(Self { reader, path })
    }

    /// Read all entries from WAL
    pub fn read_all(&mut self) -> std::io::Result<Vec<AuditChainWalEntry>> {
        let mut entries = Vec::new();

        loop {
            // Read length prefix
            let mut len_bytes = [0u8; 4];
            match self.reader.read_exact(&mut len_bytes) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }

            let len = u32::from_le_bytes(len_bytes) as usize;

            // Read entry data
            let mut data = vec![0u8; len];
            match self.reader.read_exact(&mut data) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }

            // Deserialize entry
            if let Some(entry) = AuditChainWalEntry::from_bytes(&data) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }
}

/// Audit chain WAL manager for persistence
pub struct AuditChainWalManager {
    path: PathBuf,
}

impl AuditChainWalManager {
    /// Create new WAL manager
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Get WAL writer
    pub fn get_writer(&self) -> std::io::Result<AuditChainWalWriter> {
        AuditChainWalWriter::new(self.path.clone())
    }

    /// Get WAL reader
    pub fn get_reader(&self) -> std::io::Result<AuditChainWalReader> {
        AuditChainWalReader::new(self.path.clone())
    }

    /// Recover audit chain from WAL
    pub fn recover(&self) -> std::io::Result<(Vec<AuditChainEntry>, AuditChainState)> {
        let mut reader = self.get_reader()?;
        let entries = reader.read_all()?;

        let mut chain_entries = Vec::new();
        let mut state = AuditChainState::default();

        for entry in entries {
            match entry.entry_type {
                AuditChainWalEntryType::Append => {
                    if let Some(data) = entry.entry_data {
                        if let Ok(audit_entry) = serde_json::from_slice::<AuditChainEntry>(&data) {
                            chain_entries.push(audit_entry);
                        }
                    }
                }
                AuditChainWalEntryType::Checkpoint => {
                    if let Some(state_data) = entry.state_data {
                        state = state_data;
                    }
                }
                AuditChainWalEntryType::Truncate => {
                    if let Some(before_seq) = entry.truncate_before_seq {
                        chain_entries.retain(|e| e.seq >= before_seq);
                    }
                }
            }
        }

        Ok((chain_entries, state))
    }

    /// Persist audit chain to WAL
    pub fn persist(&self, chain: &AuditChain) -> std::io::Result<()> {
        let mut writer = self.get_writer()?;

        // Write all entries
        for entry in chain.entries() {
            writer.append_entry(entry)?;
        }

        // Write checkpoint with final state
        writer.checkpoint(chain.get_state())?;

        Ok(())
    }
}

/// Compute SHA-256 checksum for an audit chain entry
pub fn compute_entry_checksum(entry: &AuditChainEntry) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(&entry.prev_hash);
    hasher.update(entry.seq.to_le_bytes());
    hasher.update(entry.timestamp.to_le_bytes());
    hasher.update(entry.user_id.as_bytes());
    if let Some(ref session_id) = entry.session_id {
        hasher.update(session_id.as_bytes());
    }
    hasher.update(entry.action.as_bytes());
    hasher.update(entry.table_name.as_bytes());
    if let Some(ref record_id) = entry.record_id {
        hasher.update(record_id.as_bytes());
    }
    if let Some(ref old_value) = entry.old_value {
        hasher.update(old_value.as_bytes());
    }
    if let Some(ref new_value) = entry.new_value {
        hasher.update(new_value.as_bytes());
    }
    hasher.update(entry.tx_id.to_le_bytes());
    if let Some(ref ip_address) = entry.ip_address {
        hasher.update(ip_address.as_bytes());
    }
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit_chain::GENESIS_PREV_HASH;
    use tempfile::tempdir;

    fn create_test_entry(seq: u64, prev_hash: [u8; 32]) -> AuditChainEntry {
        AuditChainEntry::new(
            seq,
            prev_hash,
            1000 + seq as i64,
            format!("user{}", seq),
            Some(format!("session{}", seq)),
            "CREATE".to_string(),
            "test_table".to_string(),
            Some(format!("record{}", seq)),
            None,
            Some(r#"{"data":"value"}"#.to_string()),
            seq,
            Some("192.168.1.1".to_string()),
        )
    }

    #[test]
    fn test_wal_entry_serialization() {
        let entry = create_test_entry(1, GENESIS_PREV_HASH);
        let wal_entry = AuditChainWalEntry {
            entry_type: AuditChainWalEntryType::Append,
            seq: 1,
            entry_data: Some(serde_json::to_vec(&entry).unwrap()),
            state_data: None,
            truncate_before_seq: None,
            lsn: 0,
            timestamp: 1000,
        };

        let bytes = wal_entry.to_bytes();
        let restored = AuditChainWalEntry::from_bytes(&bytes).unwrap();

        assert_eq!(restored.entry_type, AuditChainWalEntryType::Append);
        assert_eq!(restored.seq, 1);
        assert!(restored.entry_data.is_some());
    }

    #[test]
    fn test_wal_write_and_recover() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("audit_chain.wal");

        // Create and populate chain
        let mut chain = AuditChain::new();
        let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
        let hash1 = entry1.checksum;
        chain.append(entry1).unwrap();

        let entry2 = create_test_entry(2, hash1);
        chain.append(entry2).unwrap();

        // Persist to WAL
        let manager = AuditChainWalManager::new(wal_path);
        manager.persist(&chain).unwrap();

        // Recover from WAL
        let (recovered_entries, recovered_state) = manager.recover().unwrap();

        assert_eq!(recovered_entries.len(), 2);
        assert_eq!(recovered_state.length, 2);
        assert_eq!(recovered_state.next_seq, 3);
    }

    #[test]
    fn test_wal_checkpoint() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("checkpoint.wal");

        let mut writer = AuditChainWalWriter::new(wal_path).unwrap();

        let state = AuditChainState::default();
        let lsn = writer.checkpoint(&state).unwrap();

        assert_eq!(lsn, 0);
        assert_eq!(writer.current_lsn(), 1);
    }

    #[test]
    fn test_wal_append_entry() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("append.wal");

        let mut writer = AuditChainWalWriter::new(wal_path).unwrap();
        let entry = create_test_entry(1, GENESIS_PREV_HASH);

        let lsn = writer.append_entry(&entry).unwrap();

        assert_eq!(lsn, 0);
        assert_eq!(writer.current_lsn(), 1);
    }
}
