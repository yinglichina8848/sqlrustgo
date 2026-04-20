//! OLTP Read Write workload
//!
//! Sysbench-compatible read-write workload

use async_trait::async_trait;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

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

    #[allow(dead_code)]
    fn with_max_id(mut self, max_id: u64) -> Self {
        self.max_id = max_id;
        self
    }

    #[allow(dead_code)]
    fn with_statements(mut self, count: usize) -> Self {
        self.statements_per_tx = count;
        self
    }

    #[allow(dead_code)]
    fn with_read_ratio(mut self, ratio: f64) -> Self {
        self.read_ratio = ratio;
        self
    }

    #[allow(dead_code)]
    fn generate_read_sql(&self, rng: &mut SmallRng) -> String {
        let id = rng.gen_range(1..self.max_id);
        format!("SELECT * FROM sbtest1 WHERE id = {}", id)
    }

    #[allow(dead_code)]
    fn generate_write_sql(&self, rng: &mut SmallRng) -> String {
        let id = rng.gen_range(1..self.max_id);
        let k = rng.gen_range(1..100000);
        format!("UPDATE sbtest1 SET k = {} WHERE id = {}", k, id)
    }
}

impl Default for OltpReadWrite {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::workload::Workload for OltpReadWrite {
    async fn execute(&self, db: &dyn Database) -> anyhow::Result<()> {
        let mut rng = rand::rngs::SmallRng::from_entropy();
        let sql = self.generate_sql(&mut rng);
        db.execute(&sql).await
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
