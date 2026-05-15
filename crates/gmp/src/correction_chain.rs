use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::correction::CorrectionRecord;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionChainEntry {
    pub seq: u64,
    pub prev_hash: [u8; 32],
    pub correction_id: String,
    pub original_id: String,
    pub corrected_id: String,
    pub timestamp: i64,
    pub checksum: [u8; 32],
}

impl CorrectionChainEntry {
    pub fn new(
        seq: u64,
        prev_hash: [u8; 32],
        correction_id: String,
        original_id: String,
        corrected_id: String,
        timestamp: i64,
    ) -> Self {
        let mut entry = Self {
            seq,
            prev_hash,
            correction_id,
            original_id,
            corrected_id,
            timestamp,
            checksum: [0u8; 32],
        };
        entry.checksum = compute_entry_checksum(&entry);
        entry
    }
}

pub fn compute_entry_checksum(entry: &CorrectionChainEntry) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(entry.prev_hash);
    hasher.update(entry.seq.to_le_bytes());
    hasher.update(entry.correction_id.as_bytes());
    hasher.update(entry.original_id.as_bytes());
    hasher.update(entry.corrected_id.as_bytes());
    hasher.update(entry.timestamp.to_le_bytes());
    hasher.finalize().into()
}

#[derive(Debug)]
pub struct CorrectionChain {
    entries: Vec<CorrectionChainEntry>,
}

impl CorrectionChain {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn append(&mut self, record: &CorrectionRecord) {
        let prev_hash = self.entries.last().map(|e| e.checksum).unwrap_or([0u8; 32]);
        let seq = self.entries.len() as u64 + 1;
        let entry = CorrectionChainEntry::new(
            seq,
            prev_hash,
            record.id.clone(),
            record.original_id.clone(),
            record.corrected_id.clone(),
            record.timestamp,
        );
        self.entries.push(entry);
    }

    pub fn verify_integrity(&self) -> bool {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.checksum != compute_entry_checksum(entry) {
                return false;
            }
            if i > 0 {
                if entry.prev_hash != self.entries[i - 1].checksum {
                    return false;
                }
            }
        }
        true
    }

    pub fn get_corrections_for(&self, original_id: &str) -> Vec<&CorrectionChainEntry> {
        self.entries
            .iter()
            .filter(|e| e.original_id == original_id)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for CorrectionChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correction_chain_append() {
        let mut chain = CorrectionChain::new();
        let record = CorrectionRecord::new(
            "rec_001".to_string(),
            "rec_002".to_string(),
            "Corrected".to_string(),
            "user1".to_string(),
            "auth1".to_string(),
            1000,
            None,
            None,
        );
        chain.append(&record);
        assert_eq!(chain.len(), 1);
        assert!(chain.verify_integrity());
    }

    #[test]
    fn test_correction_chain_verify() {
        let mut chain = CorrectionChain::new();
        for i in 0..5 {
            let record = CorrectionRecord::new(
                format!("rec_{}", i),
                format!("rec_corrected_{}", i),
                "Reason".to_string(),
                "user1".to_string(),
                "auth1".to_string(),
                1000 + i as i64,
                None,
                None,
            );
            chain.append(&record);
        }
        assert!(chain.verify_integrity());
        assert_eq!(chain.get_corrections_for("rec_2").len(), 1);
    }
}
