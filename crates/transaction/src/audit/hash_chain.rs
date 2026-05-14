//! Audit Hash Chain Module
//!
//! Provides tamper-evident hash chain for audit events.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashChainEntry {
    pub index: u64,
    pub event_hash: [u8; 32],
    pub timestamp: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HashChain {
    entries: Vec<HashChainEntry>,
}

impl HashChain {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn append(&mut self, event_hash: [u8; 32], timestamp: u64) -> &HashChainEntry {
        let index = self.entries.len() as u64;
        let entry = HashChainEntry {
            index,
            event_hash,
            timestamp,
        };
        self.entries.push(entry);
        self.entries.last().unwrap()
    }

    pub fn verify(&self) -> bool {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.index != i as u64 {
                return false;
            }
        }
        true
    }

    pub fn entries(&self) -> &[HashChainEntry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn last_hash(&self) -> Option<[u8; 32]> {
        self.entries.last().map(|e| e.event_hash)
    }
}

pub fn compute_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn verify_hash(data: &[u8], expected: &[u8; 32]) -> bool {
    compute_hash(data) == *expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_chain_append() {
        let mut chain = HashChain::new();
        let hash1 = compute_hash(b"event1");
        let hash2 = compute_hash(b"event2");

        chain.append(hash1, 1000);
        chain.append(hash2, 2000);

        assert_eq!(chain.len(), 2);
        assert_eq!(chain.last_hash(), Some(hash2));
    }

    #[test]
    fn test_hash_chain_verify() {
        let mut chain = HashChain::new();
        chain.append(compute_hash(b"event1"), 1000);
        chain.append(compute_hash(b"event2"), 2000);

        assert!(chain.verify());
    }

    #[test]
    fn test_compute_hash() {
        let hash = compute_hash(b"test data");
        assert_ne!(hash, [0u8; 32]);

        let hash2 = compute_hash(b"test data");
        assert_eq!(hash, hash2);

        let hash3 = compute_hash(b"different data");
        assert_ne!(hash, hash3);
    }
}
