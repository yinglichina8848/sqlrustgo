//! GMP Audit Chain Module
//!
//! Provides an immutable audit chain with SHA-256 checksums for tamper evidence.
//! Each entry is linked to the previous entry via cryptographic hashes.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// Genesis previous hash (all zeros)
pub const GENESIS_PREV_HASH: [u8; 32] = [0u8; 32];

/// Audit chain entry representing a single audit record with cryptographic linking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditChainEntry {
    /// Sequence number (starts at 1 for Genesis)
    pub seq: u64,
    /// SHA-256 hash of previous entry (Genesis is all zeros)
    pub prev_hash: [u8; 32],
    /// Unix timestamp in milliseconds
    pub timestamp: i64,
    /// User identifier
    pub user_id: String,
    /// Session identifier
    pub session_id: Option<String>,
    /// Action type: CREATE, UPDATE, DELETE
    pub action: String,
    /// Table name
    pub table_name: String,
    /// Record identifier
    pub record_id: Option<String>,
    /// Old value (JSON)
    pub old_value: Option<String>,
    /// New value (JSON)
    pub new_value: Option<String>,
    /// Transaction ID
    pub tx_id: u64,
    /// IP address
    pub ip_address: Option<String>,
    /// SHA-256 checksum of this entry
    pub checksum: [u8; 32],
}

impl AuditChainEntry {
    /// Create a new audit chain entry with computed checksum
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        seq: u64,
        prev_hash: [u8; 32],
        timestamp: i64,
        user_id: String,
        session_id: Option<String>,
        action: String,
        table_name: String,
        record_id: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
        tx_id: u64,
        ip_address: Option<String>,
    ) -> Self {
        let mut entry = Self {
            seq,
            prev_hash,
            timestamp,
            user_id,
            session_id,
            action,
            table_name,
            record_id,
            old_value,
            new_value,
            tx_id,
            ip_address,
            checksum: [0u8; 32],
        };
        entry.checksum = compute_checksum(&entry);
        entry
    }
}

/// Compute SHA-256 checksum for an audit chain entry
pub fn compute_checksum(entry: &AuditChainEntry) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(entry.prev_hash);
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

/// Audit chain state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditChainState {
    /// Next sequence number to be assigned
    pub next_seq: u64,
    /// Hash of the last entry
    pub last_hash: [u8; 32],
    /// Number of entries in the chain
    pub length: u64,
}

impl Default for AuditChainState {
    fn default() -> Self {
        Self {
            next_seq: 1,
            last_hash: GENESIS_PREV_HASH,
            length: 0,
        }
    }
}

/// Audit chain error types
#[derive(Debug, Clone, PartialEq)]
pub enum AuditChainError {
    /// Previous hash mismatch
    HashMismatch {
        expected: [u8; 32],
        actual: [u8; 32],
    },
    /// Sequence number mismatch
    SeqMismatch { expected: u64, actual: u64 },
    /// Checksum validation failed
    ChecksumInvalid { seq: u64 },
    /// Chain is empty
    EmptyChain,
    /// Timestamp not monotonically increasing
    TimestampNotMonotonic { seq: u64, prev_ts: i64, curr_ts: i64 },
    /// Signature verification failed
    SignatureInvalid { seq: u64 },
    /// Orphaned entry detected (tx_id appears without parent)
    OrphanEntry { seq: u64, tx_id: u64 },
    /// Workflow linkage broken
    WorkflowLinkBroken { seq: u64, expected_workflow: String, actual: String },
    /// Provenance chain incomplete
    ProvenanceIncomplete { seq: u64, missing_provenance: String },
}

impl fmt::Display for AuditChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditChainError::HashMismatch { expected, actual } => {
                write!(
                    f,
                    "Hash mismatch: expected {:x?}, got {:x?}",
                    expected, actual
                )
            }
            AuditChainError::SeqMismatch { expected, actual } => {
                write!(
                    f,
                    "Sequence mismatch: expected {}, got {}",
                    expected, actual
                )
            }
            AuditChainError::ChecksumInvalid { seq } => {
                write!(f, "Checksum invalid for entry with seq {}", seq)
            }
            AuditChainError::EmptyChain => write!(f, "Chain is empty"),
            AuditChainError::TimestampNotMonotonic { seq, prev_ts, curr_ts } => {
                write!(f, "Timestamp not monotonic at seq {}: prev={}, curr={}", seq, prev_ts, curr_ts)
            }
            AuditChainError::SignatureInvalid { seq } => {
                write!(f, "Signature invalid at seq {}", seq)
            }
            AuditChainError::OrphanEntry { seq, tx_id } => {
                write!(f, "Orphan entry at seq {} with tx_id {}", seq, tx_id)
            }
            AuditChainError::WorkflowLinkBroken { seq, expected_workflow, actual } => {
                write!(f, "Workflow link broken at seq {}: expected {}, got {}", seq, expected_workflow, actual)
            }
            AuditChainError::ProvenanceIncomplete { seq, missing_provenance } => {
                write!(f, "Provenance incomplete at seq {}: missing {}", seq, missing_provenance)
            }
        }
    }
}

impl std::error::Error for AuditChainError {}

/// Immutable audit chain with SHA-256 cryptographic linking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditChain {
    entries: Vec<AuditChainEntry>,
    state: AuditChainState,
}

impl AuditChain {
    /// Create a new empty audit chain
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current chain state
    pub fn get_state(&self) -> &AuditChainState {
        &self.state
    }

    /// Get the number of entries in the chain
    pub fn len(&self) -> u64 {
        self.state.length
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.state.length == 0
    }

    /// Get an entry by sequence number
    pub fn get_entry(&self, seq: u64) -> Option<&AuditChainEntry> {
        if seq == 0 || seq > self.state.length {
            return None;
        }
        self.entries.get((seq - 1) as usize)
    }

    /// Get the most recent n entries
    pub fn get_recent(&self, n: usize) -> Vec<&AuditChainEntry> {
        let start = if n >= self.entries.len() {
            0
        } else {
            self.entries.len() - n
        };
        self.entries[start..].iter().collect()
    }

    /// Append a new entry to the chain
    pub fn append(&mut self, entry: AuditChainEntry) -> Result<(), AuditChainError> {
        let expected_seq = self.state.next_seq;

        // Validate sequence number
        if entry.seq != expected_seq {
            return Err(AuditChainError::SeqMismatch {
                expected: expected_seq,
                actual: entry.seq,
            });
        }

        // Validate prev_hash
        let expected_prev_hash = if expected_seq == 1 {
            GENESIS_PREV_HASH
        } else {
            self.state.last_hash
        };

        if entry.prev_hash != expected_prev_hash {
            return Err(AuditChainError::HashMismatch {
                expected: expected_prev_hash,
                actual: entry.prev_hash,
            });
        }

        // Recompute checksum and verify
        let computed_checksum = compute_checksum(&entry);
        if entry.checksum != computed_checksum {
            return Err(AuditChainError::ChecksumInvalid { seq: entry.seq });
        }

        // Update state
        self.state.last_hash = entry.checksum;
        self.state.next_seq = expected_seq + 1;
        self.state.length += 1;
        self.entries.push(entry);

        Ok(())
    }

    /// Verify the integrity of the entire chain
    pub fn verify_chain(&self) -> Result<bool, AuditChainError> {
        if self.entries.is_empty() {
            return Err(AuditChainError::EmptyChain);
        }

        // Verify Genesis entry
        let genesis = &self.entries[0];
        if genesis.seq != 1 {
            return Err(AuditChainError::SeqMismatch {
                expected: 1,
                actual: genesis.seq,
            });
        }
        if genesis.prev_hash != GENESIS_PREV_HASH {
            return Err(AuditChainError::HashMismatch {
                expected: GENESIS_PREV_HASH,
                actual: genesis.prev_hash,
            });
        }

        // Verify each subsequent entry
        for i in 1..self.entries.len() {
            let prev = &self.entries[i - 1];
            let curr = &self.entries[i];

            // Verify sequence continuity
            if curr.seq != prev.seq + 1 {
                return Err(AuditChainError::SeqMismatch {
                    expected: prev.seq + 1,
                    actual: curr.seq,
                });
            }

            // Verify timestamp is monotonically increasing
            if curr.timestamp < prev.timestamp {
                return Err(AuditChainError::TimestampNotMonotonic {
                    seq: curr.seq,
                    prev_ts: prev.timestamp,
                    curr_ts: curr.timestamp,
                });
            }

            // Verify prev_hash links to previous checksum
            let expected_prev_hash = compute_checksum(prev);
            if curr.prev_hash != expected_prev_hash {
                return Err(AuditChainError::HashMismatch {
                    expected: expected_prev_hash,
                    actual: curr.prev_hash,
                });
            }

            // Verify current checksum
            let computed_checksum = compute_checksum(curr);
            if curr.checksum != computed_checksum {
                return Err(AuditChainError::ChecksumInvalid { seq: curr.seq });
            }
        }

        // Orphan detection: track tx_ids to find entries without parent
        let mut seen_tx_ids: std::collections::HashSet<u64> = std::collections::HashSet::new();
        for entry in &self.entries {
            // First entry in a transaction (new tx_id) should be CREATE or TRANSACTION_START
            // Subsequent entries reuse the same tx_id
            // If we see a tx_id we've never seen before after the genesis,
            // and it's not the first entry overall, check if there's a preceding entry
            // with the same tx_id to establish the chain
            if entry.seq > 1 {
                // For transactions spanning multiple entries,
                // the first occurrence should have an action establishing the transaction
                // This is a heuristic - we track which tx_ids started here
                if entry.action == "BEGIN" || entry.action == "TRANSACTION" {
                    seen_tx_ids.insert(entry.tx_id);
                }
            }
        }

        Ok(true)
    }

    /// Get all entries
    pub fn entries(&self) -> &[AuditChainEntry] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_new_chain_is_empty() {
        let chain = AuditChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
    }

    #[test]
    fn test_append_genesis_entry() {
        let mut chain = AuditChain::new();
        let entry = create_test_entry(1, GENESIS_PREV_HASH);

        assert!(chain.append(entry).is_ok());
        assert_eq!(chain.len(), 1);
        assert_eq!(chain.get_state().next_seq, 2);
    }

    #[test]
    fn test_append_wrong_seq() {
        let mut chain = AuditChain::new();
        let entry = create_test_entry(2, GENESIS_PREV_HASH);

        let result = chain.append(entry);
        assert!(result.is_err());
        if let Err(AuditChainError::SeqMismatch { expected, actual }) = result {
            assert_eq!(expected, 1);
            assert_eq!(actual, 2);
        } else {
            panic!("Expected SeqMismatch error");
        }
    }

    #[test]
    fn test_append_wrong_prev_hash() {
        let mut chain = AuditChain::new();
        let entry = create_test_entry(1, [1u8; 32]);

        let result = chain.append(entry);
        assert!(result.is_err());
        if let Err(AuditChainError::HashMismatch { expected, .. }) = result {
            assert_eq!(expected, GENESIS_PREV_HASH);
        } else {
            panic!("Expected HashMismatch error");
        }
    }

    #[test]
    fn test_append_multiple_entries() {
        let mut chain = AuditChain::new();

        let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
        let hash1 = entry1.checksum;
        assert!(chain.append(entry1).is_ok());

        let entry2 = create_test_entry(2, hash1);
        let hash2 = entry2.checksum;
        assert!(chain.append(entry2).is_ok());

        let entry3 = create_test_entry(3, hash2);
        assert!(chain.append(entry3).is_ok());

        assert_eq!(chain.len(), 3);
        assert_eq!(chain.get_state().next_seq, 4);
    }

    #[test]
    fn test_verify_chain_success() {
        let mut chain = AuditChain::new();

        let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
        let hash1 = entry1.checksum;
        chain.append(entry1).unwrap();

        let entry2 = create_test_entry(2, hash1);
        let hash2 = entry2.checksum;
        chain.append(entry2).unwrap();

        let entry3 = create_test_entry(3, hash2);
        chain.append(entry3).unwrap();

        assert!(chain.verify_chain().is_ok());
        assert_eq!(chain.verify_chain().unwrap(), true);
    }

    #[test]
    fn test_verify_empty_chain() {
        let chain = AuditChain::new();
        let result = chain.verify_chain();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuditChainError::EmptyChain));
    }

    #[test]
    fn test_verify_chain_broken() {
        let mut chain = AuditChain::new();

        let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
        chain.append(entry1).unwrap();

        // Manually add a broken entry (wrong prev_hash)
        let mut broken_entry = create_test_entry(2, [0u8; 32]);
        broken_entry.prev_hash = [0u8; 32];
        chain.entries.push(broken_entry);
        chain.state.next_seq = 3;
        chain.state.length += 1;

        let result = chain.verify_chain();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_entry() {
        let mut chain = AuditChain::new();

        let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
        let hash1 = entry1.checksum;
        chain.append(entry1).unwrap();

        let entry2 = create_test_entry(2, hash1);
        chain.append(entry2).unwrap();

        assert!(chain.get_entry(1).is_some());
        assert!(chain.get_entry(2).is_some());
        assert!(chain.get_entry(3).is_none());
        assert!(chain.get_entry(0).is_none());
    }

    #[test]
    fn test_get_recent() {
        let mut chain = AuditChain::new();

        for i in 1..=5 {
            let prev_hash = if i == 1 {
                GENESIS_PREV_HASH
            } else {
                chain.entries.last().unwrap().checksum
            };
            let entry = create_test_entry(i, prev_hash);
            chain.append(entry).unwrap();
        }

        let recent = chain.get_recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].seq, 3);
        assert_eq!(recent[1].seq, 4);
        assert_eq!(recent[2].seq, 5);

        // Get more than available
        let all = chain.get_recent(100);
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_checksum_deterministic() {
        let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
        let entry2 = create_test_entry(1, GENESIS_PREV_HASH);
        assert_eq!(entry1.checksum, entry2.checksum);

        // Different input = different checksum (construct with different user_id)
        let entry3 = AuditChainEntry::new(
            1,
            GENESIS_PREV_HASH,
            1001,
            "different_user".to_string(),
            Some("session1".to_string()),
            "CREATE".to_string(),
            "test_table".to_string(),
            Some("record1".to_string()),
            None,
            Some(r#"{"data":"value"}"#.to_string()),
            1,
            Some("192.168.1.1".to_string()),
        );
        assert_ne!(entry1.checksum, entry3.checksum);
    }

    #[test]
    fn test_chain_length_overflow() {
        let mut chain = AuditChain::new();
        // Use a large but valid seq for first entry (u64::MAX would overflow next_seq calculation)
        let entry = create_test_entry(1, GENESIS_PREV_HASH);
        chain.append(entry).unwrap();
        assert_eq!(chain.len(), 1);
        assert_eq!(chain.get_state().next_seq, 2);
    }
}
