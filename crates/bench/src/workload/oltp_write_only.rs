//! OLTP Write Only workload
//!
//! Sysbench-compatible write-only workload
//! Operations: 50% UPDATE, 25% INSERT, 25% DELETE

use async_trait::async_trait;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;

use crate::db::Database;

/// Write-only workload - only write operations
pub struct OltpWriteOnly {
    max_id: u64,
    statements_per_tx: usize,
}

impl OltpWriteOnly {
    pub fn new() -> Self {
        Self {
            max_id: 1_000_000,
            statements_per_tx: 10,
        }
    }

    #[allow(dead_code)]
    fn with_max_id(mut self, max_id: u64) -> Self {
        self.max_id = max_id;
        self
    }

    fn generate_random_string(len: usize) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let chars: String = (0..len)
            .map(|_| {
                let idx = rng.gen_range(0..26);
                (b'a' + idx) as char
            })
            .collect();
        chars
    }

    fn generate_update(&self, rng: &mut SmallRng) -> String {
        let id = rng.gen_range(1..self.max_id);
        let k = rng.gen_range(1..100000);
        format!("UPDATE sbtest SET k = {} WHERE id = {}", k, id)
    }

    fn generate_insert(&self, rng: &mut SmallRng) -> String {
        let id = rng.gen_range(1..self.max_id);
        let k = rng.gen_range(1..100000);
        let c = Self::generate_random_string(120);
        let pad = Self::generate_random_string(60);
        format!(
            "INSERT INTO sbtest (id, k, c, pad) VALUES ({}, {}, '{}', '{}')",
            id, k, c, pad
        )
    }

    fn generate_delete(&self, rng: &mut SmallRng) -> String {
        let id = rng.gen_range(1..self.max_id);
        format!("DELETE FROM sbtest WHERE id = {}", id)
    }
}

impl Default for OltpWriteOnly {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::workload::Workload for OltpWriteOnly {
    async fn execute(&self, db: &dyn Database) -> anyhow::Result<()> {
        let sql = self.generate_sql(&mut SmallRng::from_entropy());
        db.execute(&sql).await
    }

    fn name(&self) -> &str {
        "oltp_write_only"
    }

    fn generate_sql(&self, rng: &mut SmallRng) -> String {
        let op = rng.gen_range(0..100);
        match op {
            0..50 => self.generate_update(rng),  // 50%
            50..75 => self.generate_insert(rng), // 25%
            _ => self.generate_delete(rng),      // 25%
        }
    }

    fn generate_transaction(&self, rng: &mut SmallRng) -> Vec<String> {
        (0..self.statements_per_tx)
            .map(|_| self.generate_sql(rng))
            .collect()
    }

    fn statements_per_tx(&self) -> usize {
        self.statements_per_tx
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn table_names(&self) -> Vec<String> {
        vec!["sbtest".to_string()]
    }
}
