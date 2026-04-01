//! OLTP Read Only workload
//!
//! Sysbench-compatible read-only workload

use async_trait::async_trait;
use rand::rngs::SmallRng;
use rand::Rng;

use crate::db::Database;

/// Read-only workload - multiple read queries
pub struct OltpReadOnly {
    max_id: u64,
    statements_per_tx: usize,
}

impl OltpReadOnly {
    pub fn new() -> Self {
        Self {
            max_id: 1_000_000,
            statements_per_tx: 10,
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
}

impl Default for OltpReadOnly {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::workload::Workload for OltpReadOnly {
    async fn execute(&self, _db: &dyn Database) -> anyhow::Result<()> {
        // TODO: Implement read only
        todo!("OLTP Read Only not yet implemented")
    }

    fn name(&self) -> &str {
        "oltp_read_only"
    }

    fn generate_sql(&self, rng: &mut SmallRng) -> String {
        // Randomly choose between different read operations
        let query_type = rng.gen_range(0..10);
        match query_type {
            // 0-4: Range query (40%)
            0..=4 => {
                let start = rng.gen_range(1..self.max_id);
                let end = start + rng.gen_range(1..100).min(self.max_id - start + 1);
                format!(
                    "SELECT c FROM sbtest WHERE id BETWEEN {} AND {}",
                    start, end
                )
            }
            // 5-7: Aggregation query (30%)
            5..=7 => {
                let k = rng.gen_range(0..1_000_000);
                format!("SELECT COUNT(*) FROM sbtest WHERE k = {}", k)
            }
            // 8-9: Sorted query (20%)
            _ => {
                let id = rng.gen_range(1..self.max_id);
                format!("SELECT * FROM sbtest WHERE id < {} ORDER BY k", id)
            }
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
        true
    }

    fn table_names(&self) -> Vec<String> {
        vec!["sbtest".to_string()]
    }
}
