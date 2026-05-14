//! Page Checksum Module
//!
//! Provides CRC32-based page integrity verification for recovery.

use serde::{Deserialize, Serialize};

/// Page checksum entry for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageChecksum {
    pub page_id: u32,
    pub crc32: u32,
    pub page_offset: u64,
}

impl PageChecksum {
    pub fn new(page_id: u32, crc32: u32, page_offset: u64) -> Self {
        Self {
            page_id,
            crc32,
            page_offset,
        }
    }
}

/// Page checksum store for tracking all page CRCs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PageChecksumStore {
    pub checksums: Vec<PageChecksum>,
    pub xor_summary: u32,
}

impl PageChecksumStore {
    pub fn new() -> Self {
        Self {
            checksums: Vec::new(),
            xor_summary: 0,
        }
    }

    pub fn add(&mut self, checksum: PageChecksum) {
        self.xor_summary ^= checksum.crc32;
        self.checksums.push(checksum);
    }

    pub fn verify(&self, page_id: u32, computed_crc: u32) -> bool {
        if let Some(stored) = self.checksums.iter().find(|c| c.page_id == page_id) {
            stored.crc32 == computed_crc
        } else {
            false
        }
    }
}

pub fn compute_crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_crc32() {
        let data = b"test data";
        let crc = compute_crc32(data);
        assert_ne!(crc, 0);
    }

    #[test]
    fn test_page_checksum_store() {
        let mut store = PageChecksumStore::new();
        store.add(PageChecksum::new(1, compute_crc32(b"page1"), 0));
        store.add(PageChecksum::new(2, compute_crc32(b"page2"), 4096));

        assert!(store.verify(1, compute_crc32(b"page1")));
        assert!(!store.verify(1, compute_crc32(b"tampered")));
        assert!(!store.verify(999, 0));
    }
}
