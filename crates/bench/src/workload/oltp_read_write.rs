//! OLTP Read Write workload
//!
//! Sysbench-compatible read-write workload

use async_trait::async_trait;
use rand::rngs::SmallRng;
use rand::Rng;

use crate::db::Database;

/// Read-write workload - mixed read and write queries
pub struct OltpReadWrite {
    max_id: u64,
    statements_per_tx: usize,
    read_ratio: f64, // Ratio of read operations in a transaction
}

impl OltpReadWrite {
    pub fn new() -> Self {
        Self {
            max_id: 1_000_000,
            statements_per_tx: 10,
            read_ratio: 0.5, // 50% reads, 50% writes
        }
    }

    fn with_max_id(mut self, max_id: u64) -> Self {
        self.max_id = max_id;
        self
    }

    fn with_statements(mut self, count: usize) -> Self {
        self.statements_per_tx = count;
        self
    }

    fn with_read_ratio(mut self, ratio: f64) -> Self {
        self.read_ratio = ratio;
        self
    }
}

impl Default for OltpReadWrite {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::workload::Workload for OltpReadWrite {
    async fn execute(&self, _db: &dyn Database) -> anyhow::Result<()> {
        // TODO: Implement read write
        todo!("OLTP Read Write not yet implemented")
    }

    fn name(&self) -> &str {
        "oltp_read_write"
    }

    fn generate_sql(&self, rng: &mut SmallRng) -> String {
        // Decide whether this is a read or write operation
        if rng.gen::<f64>() < self.read_ratio {
            self.generate_read_sql(rng)
        } else {
            self.generate_write_sql(rng)
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

impl OltpReadWrite {
    fn generate_read_sql(&self, rng: &mut SmallRng) -> String {
        // Read operations: point select
        let id = rng.gen_range(1..=self.max_id);
        format!("SELECT c FROM sbtest WHERE id = {}", id)
    }

    fn generate_write_sql(&self, rng: &mut SmallRng) -> String {
        // Write operations: randomly choose between UPDATE, DELETE, INSERT
        let write_type = rng.gen_range(0..3);
        match write_type {
            // 0: UPDATE (33%)
            0 => {
                let id = rng.gen_range(1..=self.max_id);
                let c_value = format!("'{:x}'", rng.gen::<u32>());
                format!("UPDATE sbtest SET c = {} WHERE id = {}", c_value, id)
            }
            // 1: DELETE (33%)
            1 => {
                let id = rng.gen_range(1..=self.max_id);
                format!("DELETE FROM sbtest WHERE id = {}", id)
            }
            // 2: INSERT (34%)
            _ => {
                let id = rng.gen_range(1..=self.max_id);
                let k = rng.gen_range(0..1_000_000);
                let c = format!("'c{:x}'", rng.gen::<u32>());
                let pad = format!("'pad{:x}'", rng.gen::<u32>());
                format!(
                    "INSERT INTO sbtest (id, k, c, pad) VALUES ({}, {}, {}, {})",
                    id, k, c, pad
                )
            }
        }
    }
}
