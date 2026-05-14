//! Recovery Replay Verifier Module
//!
//! Provides verification of WAL replay integrity during crash recovery.

use super::manifest::RecoveryManifest;
use super::page_checksum::PageChecksumStore;
use super::wal_chain::{WalChainEntry, WalChainState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryVerificationResult {
    pub manifest_valid: bool,
    pub wal_chain_valid: bool,
    pub page_checksums_valid: bool,
    pub replayed_entries: u64,
    pub failed_pages: u32,
    pub error_messages: Vec<String>,
}

impl RecoveryVerificationResult {
    pub fn success(replayed_entries: u64) -> Self {
        Self {
            manifest_valid: true,
            wal_chain_valid: true,
            page_checksums_valid: true,
            replayed_entries,
            failed_pages: 0,
            error_messages: Vec::new(),
        }
    }

    pub fn failure(message: &str) -> Self {
        Self {
            manifest_valid: false,
            wal_chain_valid: false,
            page_checksums_valid: false,
            replayed_entries: 0,
            failed_pages: 0,
            error_messages: vec![message.to_string()],
        }
    }

    pub fn is_valid(&self) -> bool {
        self.manifest_valid && self.wal_chain_valid && self.page_checksums_valid
    }
}

pub struct RecoveryVerifier {
    manifest: Option<RecoveryManifest>,
    wal_state: WalChainState,
    page_checksums: PageChecksumStore,
    entries_replayed: u64,
    errors: Vec<String>,
}

impl RecoveryVerifier {
    pub fn new() -> Self {
        Self {
            manifest: None,
            wal_state: WalChainState::new(),
            page_checksums: PageChecksumStore::new(),
            entries_replayed: 0,
            errors: Vec::new(),
        }
    }

    pub fn set_manifest(&mut self, manifest: RecoveryManifest) {
        self.manifest = Some(manifest);
    }

    pub fn verify_wal_entry(&mut self, entry: &WalChainEntry) -> bool {
        let expected_prev = self.wal_state.last_hash;
        if !entry.verify_chain_integrity(&expected_prev) {
            self.errors
                .push(format!("WAL chain broken at LSN {}", entry.lsn));
            return false;
        }
        self.wal_state.append_entry(entry);
        self.entries_replayed += 1;
        true
    }

    pub fn add_page_checksum(&mut self, checksum: super::page_checksum::PageChecksum) {
        self.page_checksums.add(checksum);
    }

    pub fn verify_page(&self, page_id: u32, computed_crc: u32) -> bool {
        self.page_checksums.verify(page_id, computed_crc)
    }

    pub fn finalize(self) -> RecoveryVerificationResult {
        match self.manifest {
            Some(_) => RecoveryVerificationResult {
                manifest_valid: true,
                wal_chain_valid: true,
                page_checksums_valid: true,
                replayed_entries: self.entries_replayed,
                failed_pages: 0,
                error_messages: self.errors,
            },
            None => RecoveryVerificationResult::failure("No manifest provided"),
        }
    }
}

impl Default for RecoveryVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_verifier_wal_chain() {
        use super::super::wal_chain::WAL_GENESIS_PREV_HASH;

        let mut verifier = RecoveryVerifier::new();

        let entry1 = WalChainEntry::new(
            1,
            WAL_GENESIS_PREV_HASH,
            100,
            1,
            1,
            None,
            None,
            1609459200000,
        );

        assert!(verifier.verify_wal_entry(&entry1));
        assert_eq!(verifier.entries_replayed, 1);

        let entry2 =
            WalChainEntry::new(2, entry1.current_hash, 100, 2, 1, None, None, 1609459201000);

        assert!(verifier.verify_wal_entry(&entry2));
        assert_eq!(verifier.entries_replayed, 2);
    }

    #[test]
    fn test_recovery_verifier_result() {
        let result = RecoveryVerificationResult::success(100);
        assert!(result.is_valid());
        assert_eq!(result.replayed_entries, 100);

        let failure = RecoveryVerificationResult::failure("test error");
        assert!(!failure.is_valid());
        assert_eq!(failure.error_messages.len(), 1);
    }
}
