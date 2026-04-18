//! OLTP Range Scan workload
//!
//! Sysbench-compatible range scan workload on indexed column

use async_trait::async_trait;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::db::Database;

/// Range scan workload - range scan by indexed column 'k'
pub struct OltpRangeScan {
    max_id: u64,
    range_size: u64,
    statements_per_tx: usize,
}

impl OltpRangeScan {
    pub fn new() -> Self {
        Self {
            max_id: 1_000_000,
            range_size: 100,
            statements_per_tx: 10,
        }
    }

    #[allow(dead_code)]
    fn with_max_id(mut self, max_id: u64) -> Self {
        self.max_id = max_id;
        self
    }

    #[allow(dead_code)]
    fn with_range_size(mut self, size: u64) -> Self {
        self.range_size = size;
        self
    }
}

impl Default for OltpRangeScan {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::workload::Workload for OltpRangeScan {
    async fn execute(&self, db: &dyn Database) -> anyhow::Result<()> {
        let sql = self.generate_sql(&mut SmallRng::from_entropy());
        db.execute(&sql).await
    }

    fn name(&self) -> &str {
        "oltp_range_scan"
    }

    fn generate_sql(&self, rng: &mut SmallRng) -> String {
        let start = rng.gen_range(1..self.max_id);
        let end = start.saturating_add(self.range_size);
        format!("SELECT * FROM sbtest WHERE k BETWEEN {} AND {}", start, end)
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
