//! PITR Snapshot Registry Module
//!
//! Provides registry for point-in-time recovery snapshots.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PITRSnapshot {
    pub id: String,
    pub timestamp: u64,
    pub lsn: u64,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub manifest_path: PathBuf,
}

impl PITRSnapshot {
    pub fn new(
        id: String,
        timestamp: u64,
        lsn: u64,
        path: PathBuf,
        size_bytes: u64,
        manifest_path: PathBuf,
    ) -> Self {
        Self {
            id,
            timestamp,
            lsn,
            path,
            size_bytes,
            manifest_path,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PITRRegistry {
    pub snapshots: Vec<PITRSnapshot>,
}

impl PITRRegistry {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }

    pub fn add_snapshot(&mut self, snapshot: PITRSnapshot) {
        self.snapshots.push(snapshot);
    }

    pub fn find_nearest(&self, target_lsn: u64) -> Option<&PITRSnapshot> {
        self.snapshots
            .iter()
            .filter(|s| s.lsn <= target_lsn)
            .max_by_key(|s| s.lsn)
    }

    pub fn cleanup_old(&mut self, keep: usize) {
        if self.snapshots.len() > keep {
            self.snapshots.sort_by_key(|s| s.timestamp);
            self.snapshots.truncate(keep);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitr_registry() {
        let mut registry = PITRRegistry::new();

        registry.add_snapshot(PITRSnapshot::new(
            "snap1".to_string(),
            1000,
            100,
            PathBuf::from("/tmp/snap1"),
            1024,
            PathBuf::from("/tmp/snap1.manifest"),
        ));

        registry.add_snapshot(PITRSnapshot::new(
            "snap2".to_string(),
            2000,
            200,
            PathBuf::from("/tmp/snap2"),
            2048,
            PathBuf::from("/tmp/snap2.manifest"),
        ));

        let nearest = registry.find_nearest(150);
        assert!(nearest.is_some());
        assert_eq!(nearest.unwrap().id, "snap1");

        let nearest2 = registry.find_nearest(250);
        assert!(nearest2.is_some());
        assert_eq!(nearest2.unwrap().id, "snap2");
    }
}
