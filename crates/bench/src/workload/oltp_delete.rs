//! OLTP Delete workload
//!
//! Sysbench-compatible delete workload

use async_trait::async_trait;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::db::Database;

/// Delete workload - delete operations
pub struct OltpDelete {
    max_id: u64,
    statements_per_tx: usize,
}

impl OltpDelete {
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
}

impl Default for OltpDelete {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::workload::Workload for OltpDelete {
    async fn execute(&self, db: &dyn Database) -> anyhow::Result<()> {
        let sql = self.generate_sql(&mut SmallRng::from_entropy());
        db.execute(&sql).await
    }

    fn name(&self) -> &str {
        "oltp_delete"
    }

    fn generate_sql(&self, rng: &mut SmallRng) -> String {
        let id = rng.gen_range(1..self.max_id);
        format!("DELETE FROM sbtest WHERE id = {}", id)
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
