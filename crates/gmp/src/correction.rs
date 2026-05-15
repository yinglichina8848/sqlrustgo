//! Correction Record Module for GMP
//!
//! Implements correction records for immutable data in GMP compliance.
//! When immutable records need correction, a CorrectionRecord is created
//! instead of modifying the original record.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Table name for correction records
pub const TABLE_CORRECTION_RECORDS: &str = "gmp_correction_records";

/// Correction record representing a data correction for immutable records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionRecord {
    /// Unique correction identifier
    pub id: String,
    /// Original record ID that was corrected
    pub original_id: String,
    /// ID of the new (corrected) record
    pub corrected_id: String,
    /// Reason for the correction
    pub correction_reason: String,
    /// User ID of the person who submitted the correction
    pub corrector_id: String,
    /// User ID of the person who authorized the correction
    pub authorized_by: String,
    /// Timestamp when correction was created
    pub timestamp: i64,
    /// Electronic signature associated with this correction
    pub signature: Option<String>,
    /// Policy ID used for approval
    pub policy_id: Option<String>,
    /// Checksum for integrity verification
    pub checksum: [u8; 32],
}

impl CorrectionRecord {
    /// Create a new correction record with computed checksum
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        original_id: String,
        corrected_id: String,
        correction_reason: String,
        corrector_id: String,
        authorized_by: String,
        timestamp: i64,
        signature: Option<String>,
        policy_id: Option<String>,
    ) -> Self {
        let id = uuid_simple();
        let mut record = Self {
            id: id.clone(),
            original_id,
            corrected_id,
            correction_reason,
            corrector_id,
            authorized_by,
            timestamp,
            signature,
            policy_id,
            checksum: [0u8; 32],
        };
        record.checksum = compute_checksum(&record);
        record
    }

    /// Verify the correction record integrity
    pub fn verify_integrity(&self) -> bool {
        self.checksum == compute_checksum(self)
    }
}

/// Compute SHA-256 checksum for a correction record
pub fn compute_checksum(record: &CorrectionRecord) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(record.id.as_bytes());
    hasher.update(record.original_id.as_bytes());
    hasher.update(record.corrected_id.as_bytes());
    hasher.update(record.correction_reason.as_bytes());
    hasher.update(record.corrector_id.as_bytes());
    hasher.update(record.authorized_by.as_bytes());
    hasher.update(record.timestamp.to_le_bytes());
    if let Some(ref sig) = record.signature {
        hasher.update(sig.as_bytes());
    }
    if let Some(ref pid) = record.policy_id {
        hasher.update(pid.as_bytes());
    }
    hasher.finalize().into()
}

/// Simple UUID generator
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("corr_{:x}", now)
}

/// SQL for creating the correction records table
pub const CREATE_CORRECTION_RECORDS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS gmp_correction_records (
    id TEXT PRIMARY KEY,
    original_id TEXT NOT NULL,
    corrected_id TEXT NOT NULL,
    correction_reason TEXT NOT NULL,
    corrector_id TEXT NOT NULL,
    authorized_by TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    signature TEXT,
    policy_id TEXT,
    checksum BLOB NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now'))
)
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correction_record_creation() {
        let record = CorrectionRecord::new(
            "record_123".to_string(),
            "record_456".to_string(),
            "Data entry error corrected".to_string(),
            "user_001".to_string(),
            "manager_001".to_string(),
            1234567890,
            None,
            None,
        );

        assert!(record.id.starts_with("corr_"));
        assert_eq!(record.original_id, "record_123");
        assert_eq!(record.corrected_id, "record_456");
        assert!(record.verify_integrity());
    }

    #[test]
    fn test_correction_record_integrity() {
        let mut record = CorrectionRecord::new(
            "record_123".to_string(),
            "record_456".to_string(),
            "Data entry error".to_string(),
            "user_001".to_string(),
            "manager_001".to_string(),
            1234567890,
            None,
            None,
        );

        assert!(record.verify_integrity());

        record.correction_reason = "Tampered reason".to_string();
        assert!(!record.verify_integrity());
    }
}
