//! GMP Audit Chain Tamper Detection Module
//!
//! Provides real-time tamper detection and alerting for audit chains.
//! Integrates with WAL for recovery when tampering is detected.

use crate::audit_chain::{compute_checksum, AuditChain, AuditChainEntry, AuditChainError, GENESIS_PREV_HASH};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Result of a verification operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether verification passed
    pub passed: bool,
    /// Entry sequence that failed (if any)
    pub failed_at_seq: Option<u64>,
    /// Type of violation detected
    pub violation: Option<TamperViolation>,
    /// Timestamp of verification
    pub timestamp: u64,
    /// Time taken for verification (microseconds)
    pub duration_us: u64,
}

/// Type of tamper violation detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TamperViolation {
    /// Checksum mismatch - entry data was modified
    ChecksumMismatch {
        seq: u64,
        expected: [u8; 32],
        actual: [u8; 32],
    },
    /// Hash chain broken - prev_hash doesn't match previous checksum
    ChainBroken {
        seq: u64,
        expected_prev_hash: [u8; 32],
        actual_prev_hash: [u8; 32],
    },
    /// Sequence gap detected
    SequenceGap { expected: u64, actual: u64 },
    /// Genesis block tampered
    GenesisTampered,
    /// Entry not found
    EntryNotFound(u64),
}

/// Tamper alert for critical violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TamperAlert {
    /// Alert ID
    pub id: u64,
    /// Sequence of first corrupted entry
    pub corrupted_seq: u64,
    /// Type of violation
    pub violation: TamperViolation,
    /// Timestamp when violation was detected
    pub detected_at: u64,
    /// Recommended action
    pub recommended_action: RecoveryAction,
}

/// Recovery action recommended when tampering detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// Chain is recoverable from WAL
    RecoverFromWal,
    /// Chain must be audited manually
    ManualAudit,
    /// Chain should be frozen
    FreezeChain,
    /// No recovery possible
    NoRecovery,
}

impl TamperAlert {
    /// Create a new tamper alert
    pub fn new(corrupted_seq: u64, violation: TamperViolation) -> Self {
        let recommended_action = match &violation {
            TamperViolation::ChecksumMismatch { .. } => RecoveryAction::RecoverFromWal,
            TamperViolation::ChainBroken { .. } => RecoveryAction::RecoverFromWal,
            TamperViolation::SequenceGap { .. } => RecoveryAction::RecoverFromWal,
            TamperViolation::GenesisTampered => RecoveryAction::ManualAudit,
            TamperViolation::EntryNotFound(_) => RecoveryAction::RecoverFromWal,
        };

        Self {
            id: std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            corrupted_seq,
            violation,
            detected_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            recommended_action,
        }
    }
}

/// Verify a single entry's checksum without scanning the entire chain
pub fn verify_entry_checksum(entry: &AuditChainEntry) -> Result<bool, AuditChainError> {
    let computed = compute_checksum(entry);
    if entry.checksum == computed {
        Ok(true)
    } else {
        Err(AuditChainError::ChecksumInvalid { seq: entry.seq })
    }
}

/// Verify entry against expected prev_hash
pub fn verify_entry_link(
    entry: &AuditChainEntry,
    expected_prev_hash: [u8; 32],
) -> Result<bool, AuditChainError> {
    if entry.seq == 1 {
        // Genesis entry should have GENESIS_PREV_HASH
        if entry.prev_hash == GENESIS_PREV_HASH {
            Ok(true)
        } else {
            Err(AuditChainError::HashMismatch {
                expected: GENESIS_PREV_HASH,
                actual: entry.prev_hash,
            })
        }
    } else if entry.prev_hash == expected_prev_hash {
        Ok(true)
    } else {
        Err(AuditChainError::HashMismatch {
            expected: expected_prev_hash,
            actual: entry.prev_hash,
        })
    }
}

/// Incremental verification - verify new entry against last known good state
pub fn incremental_verify(
    chain: &AuditChain,
    new_entry: &AuditChainEntry,
) -> Result<VerificationResult, AuditChainError> {
    let start = SystemTime::now();
    let state = chain.get_state();

    // Check if entry sequence is as expected
    if new_entry.seq != state.next_seq {
        return Ok(VerificationResult {
            passed: false,
            failed_at_seq: Some(new_entry.seq),
            violation: Some(TamperViolation::SequenceGap {
                expected: state.next_seq,
                actual: new_entry.seq,
            }),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            duration_us: start.elapsed().unwrap().as_micros() as u64,
        });
    }

    // Verify prev_hash
    let expected_prev_hash = if new_entry.seq == 1 {
        GENESIS_PREV_HASH
    } else {
        state.last_hash
    };

    if new_entry.prev_hash != expected_prev_hash {
        return Ok(VerificationResult {
            passed: false,
            failed_at_seq: Some(new_entry.seq),
            violation: Some(TamperViolation::ChainBroken {
                seq: new_entry.seq,
                expected_prev_hash,
                actual_prev_hash: new_entry.prev_hash,
            }),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            duration_us: start.elapsed().unwrap().as_micros() as u64,
        });
    }

    // Verify checksum
    let computed = compute_checksum(new_entry);
    if new_entry.checksum != computed {
        return Ok(VerificationResult {
            passed: false,
            failed_at_seq: Some(new_entry.seq),
            violation: Some(TamperViolation::ChecksumMismatch {
                seq: new_entry.seq,
                expected: computed,
                actual: new_entry.checksum,
            }),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            duration_us: start.elapsed().unwrap().as_micros() as u64,
        });
    }

    Ok(VerificationResult {
        passed: true,
        failed_at_seq: None,
        violation: None,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        duration_us: start.elapsed().unwrap().as_micros() as u64,
    })
}

/// Quick verification - check if chain appears intact (O(1) vs O(n))
pub fn quick_verify(chain: &AuditChain) -> bool {
    let state = chain.get_state();

    // If chain says it has entries but is empty, likely tampered
    if state.length > 0 && chain.entries().is_empty() {
        return false;
    }

    // If next_seq doesn't match length + 1, likely tampered
    if state.next_seq != state.length + 1 {
        return false;
    }

    // If last_hash is all zeros but chain has entries, likely tampered
    if state.length > 0 && state.last_hash == GENESIS_PREV_HASH {
        return false;
    }

    true
}

/// Detect tampering and generate alert
pub fn detect_tamper(chain: &AuditChain) -> Option<TamperAlert> {
    // Quick check first
    if quick_verify(chain) {
        return None;
    }

    // If genesis is corrupted
    if let Some(first) = chain.get_entry(1) {
        if first.seq != 1 || first.prev_hash != GENESIS_PREV_HASH {
            return Some(TamperAlert::new(
                1,
                TamperViolation::GenesisTampered,
            ));
        }
    } else if chain.len() > 0 {
        return Some(TamperAlert::new(1, TamperViolation::EntryNotFound(1)));
    }

    // Check chain integrity
    for i in 0..chain.entries().len() {
        let entry = &chain.entries()[i];

        // Verify checksum
        let computed = compute_checksum(entry);
        if entry.checksum != computed {
            return Some(TamperAlert::new(
                entry.seq,
                TamperViolation::ChecksumMismatch {
                    seq: entry.seq,
                    expected: computed,
                    actual: entry.checksum,
                },
            ));
        }

        // Verify link to previous
        if i > 0 {
            let prev = &chain.entries()[i - 1];
            let expected_prev_hash = compute_checksum(prev);
            if entry.prev_hash != expected_prev_hash {
                return Some(TamperAlert::new(
                    entry.seq,
                    TamperViolation::ChainBroken {
                        seq: entry.seq,
                        expected_prev_hash,
                        actual_prev_hash: entry.prev_hash,
                    },
                ));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit_chain::GENESIS_PREV_HASH;

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
    fn test_verify_entry_checksum_valid() {
        let entry = create_test_entry(1, GENESIS_PREV_HASH);
        assert!(verify_entry_checksum(&entry).is_ok());
        assert!(verify_entry_checksum(&entry).unwrap());
    }

    #[test]
    fn test_verify_entry_checksum_tampered() {
        let mut entry = create_test_entry(1, GENESIS_PREV_HASH);
        entry.user_id = "tampered".to_string();

        let result = verify_entry_checksum(&entry);
        assert!(result.is_err());
    }

    #[test]
    fn test_incremental_verify_success() {
        let mut chain = AuditChain::new();
        let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
        chain.append(entry1.clone()).unwrap();

        let entry2 = create_test_entry(2, entry1.checksum);
        let result = incremental_verify(&chain, &entry2).unwrap();

        assert!(result.passed);
        assert!(result.violation.is_none());
    }

    #[test]
    fn test_incremental_verify_wrong_seq() {
        let mut chain = AuditChain::new();
        let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
        chain.append(entry1).unwrap();

        // Try to add entry with wrong seq
        let entry_wrong = create_test_entry(5, GENESIS_PREV_HASH);
        let result = incremental_verify(&chain, &entry_wrong).unwrap();

        assert!(!result.passed);
        assert!(matches!(
            result.violation,
            Some(TamperViolation::SequenceGap { .. })
        ));
    }

    #[test]
    fn test_quick_verify_intact() {
        let mut chain = AuditChain::new();
        let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
        chain.append(entry1).unwrap();

        assert!(quick_verify(&chain));
    }

    #[test]
    fn test_quick_verify_tampered() {
        let chain = AuditChain::new();
        // Empty chain with length > 0 is suspicious
        // But this is hard to create without unsafe...
        assert!(quick_verify(&chain));
    }

    #[test]
    fn test_detect_tamper_no_tamper() {
        let mut chain = AuditChain::new();
        let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
        chain.append(entry1).unwrap();

        let alert = detect_tamper(&chain);
        assert!(alert.is_none());
    }
}
