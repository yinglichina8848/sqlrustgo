//! OLTP Workload (YCSB-like)
//!
//! Operation mix:
//! - 50% Read
//! - 30% Update
//! - 10% Insert
//! - 10% Scan

use crate::db::Database;
use crate::workload::Workload;
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};

/// OLTP Workload - uses a simple atomic counter for randomness
pub struct OltpWorkload {
    scale: usize,
    counter: AtomicU64,
}

impl OltpWorkload {
    /// Create a new OLTP workload
    pub fn new(scale: usize) -> Self {
        Self {
            scale,
            counter: AtomicU64::new(0),
        }
    }

    /// Generate a pseudo-random number using atomic counter
    fn next_random(&self) -> u64 {
        // Simple linear congruential generator
        let x = self.counter.fetch_add(1, Ordering::Relaxed);
        // Constants from Numerical Recipes
        ((x.wrapping_mul(1664525)).wrapping_add(1013904223)) % 1000000
    }
}

#[async_trait]
impl Workload for OltpWorkload {
    /// Execute one operation according to the YCSB mix
    async fn execute(&self, _db: &dyn Database) -> anyhow::Result<()> {
        // Generate pseudo-random values
        let key = (self.next_random() as usize) % self.scale;
        let op = (self.next_random() as usize) % 100;

        if op < 50 {
            // 50% Read
            _db.read(key).await?;
        } else if op < 80 {
            // 30% Update
            _db.update(key).await?;
        } else if op < 90 {
            // 10% Insert
            _db.insert(key).await?;
        } else {
            // 10% Scan (range query)
            _db.scan(key, key + 10).await?;
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "oltp"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oltp_workload() {
        let workload = OltpWorkload::new(1000);
        assert_eq!(workload.name(), "oltp");
    }
}
