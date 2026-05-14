//! Recovery Manifest Module
//!
//! Provides RecoveryManifest structure that extends CheckpointMetadata with
//! additional integrity fields required for GMP compliance:
//! - schema_version: Ensures recovery uses compatible schema
//! - catalog_hash: Validates catalog integrity at checkpoint time
//! - page_crc_summary: CRC summary of all pages at checkpoint

use crate::checkpoint::CheckpointMetadata;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Recovery manifest containing checkpoint metadata plus integrity fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryManifest {
    /// Basic checkpoint metadata
    pub checkpoint: CheckpointMetadata,
    /// Schema version at checkpoint time
    pub schema_version: u64,
    /// SHA-256 hash of catalog state
    pub catalog_hash: [u8; 32],
    /// CRC32 summary of all pages (XOR of individual page CRCs)
    pub page_crc_summary: u32,
    /// List of active transactions at checkpoint
    pub active_transactions: Vec<u64>,
    /// WAL LSN range covered by this checkpoint
    pub wal_range: (u64, u64), // (wal_start_lsn, wal_end_lsn)
    /// Human-readable timestamp (ISO 8601)
    pub timestamp_iso: String,
}

impl RecoveryManifest {
    /// Create a new recovery manifest
    pub fn new(
        checkpoint: CheckpointMetadata,
        schema_version: u64,
        catalog_hash: [u8; 32],
        page_crc_summary: u32,
        active_transactions: Vec<u64>,
        wal_range: (u64, u64),
    ) -> Self {
        let timestamp_iso = now_iso();
        Self {
            checkpoint,
            schema_version,
            catalog_hash,
            page_crc_summary,
            active_transactions,
            wal_range,
            timestamp_iso,
        }
    }

    /// Compute SHA-256 hash of the catalog
    pub fn compute_catalog_hash(catalog_data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(catalog_data);
        hasher.finalize().into()
    }

    /// Compute XOR-based page CRC summary
    pub fn compute_page_crc_summary(page_crcs: &[u32]) -> u32 {
        page_crcs.iter().fold(0u32, |acc, &crc| acc ^ crc)
    }

    /// Verify manifest integrity (excluding page CRC which requires full page scan)
    pub fn verify_integrity(&self, expected_catalog_hash: &[u8; 32]) -> bool {
        // Check catalog hash
        if &self.catalog_hash != expected_catalog_hash {
            return false;
        }
        // Additional integrity checks can be added here
        true
    }
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    format!("{}.{:03}Z", secs, millis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_recovery_manifest_creation() {
        let checkpoint = CheckpointMetadata {
            lsn: 1000,
            timestamp: 1609459200000,
            tx_count: 50,
            dirty_pages: 10,
            file_path: PathBuf::from("/tmp/checkpoint.1"),
        };

        let catalog_hash = RecoveryManifest::compute_catalog_hash(b"test catalog data");
        let page_crc_summary = RecoveryManifest::compute_page_crc_summary(&[1, 2, 3, 4]);

        let manifest = RecoveryManifest::new(
            checkpoint,
            1,
            catalog_hash,
            page_crc_summary,
            vec![1, 2, 3],
            (100, 200),
        );

        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.page_crc_summary, page_crc_summary);
        assert_eq!(manifest.wal_range, (100, 200));
    }

    #[test]
    fn test_catalog_hash_computation() {
        let hash1 = RecoveryManifest::compute_catalog_hash(b"data1");
        let hash2 = RecoveryManifest::compute_catalog_hash(b"data1");
        let hash3 = RecoveryManifest::compute_catalog_hash(b"data2");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_page_crc_summary() {
        let summary =
            RecoveryManifest::compute_page_crc_summary(&[0x12345678, 0x87654321, 0xABCDEF01]);
        // XOR of all CRCs
        let expected = 0x12345678 ^ 0x87654321 ^ 0xABCDEF01;
        assert_eq!(summary, expected);
    }

    #[test]
    fn test_verify_integrity() {
        let checkpoint = CheckpointMetadata {
            lsn: 1000,
            timestamp: 1609459200000,
            tx_count: 50,
            dirty_pages: 10,
            file_path: PathBuf::from("/tmp/checkpoint.1"),
        };

        let catalog_data = b"test catalog";
        let catalog_hash = RecoveryManifest::compute_catalog_hash(catalog_data);

        let manifest = RecoveryManifest::new(checkpoint, 1, catalog_hash, 0, vec![], (0, 1000));

        assert!(manifest.verify_integrity(&catalog_hash));
        assert!(!manifest.verify_integrity(&[0u8; 32]));
    }
}
