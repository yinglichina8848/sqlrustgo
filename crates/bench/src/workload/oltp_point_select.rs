//! OLTP Point Select workload
//!
//! Sysbench-compatible point select workload

use async_trait::async_trait;
use rand::rngs::SmallRng;
use rand::Rng;

use crate::db::Database;

/// Point select workload - single row lookup by primary key
pub struct OltpPointSelect {
    max_id: u64,
}

impl OltpPointSelect {
    pub fn new() -> Self {
        Self { max_id: 1_000_000 }
    }

    fn with_max_id(mut self, max_id: u64) -> Self {
        self.max_id = max_id;
        self
    }
}

impl Default for OltpPointSelect {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::workload::Workload for OltpPointSelect {
    async fn execute(&self, _db: &dyn Database) -> anyhow::Result<()> {
        // TODO: Implement point select
        todo!("OLTP Point Select not yet implemented")
    }

    fn name(&self) -> &str {
        "oltp_point_select"
    }

    fn generate_sql(&self, rng: &mut SmallRng) -> String {
        let id = rng.gen_range(1..=self.max_id);
        format!("SELECT c FROM sbtest WHERE id = {}", id)
    }

    fn generate_transaction(&self, rng: &mut SmallRng) -> Vec<String> {
        // Point select workload: each transaction is a single SELECT
        vec![self.generate_sql(rng)]
    }

    fn statements_per_tx(&self) -> usize {
        1
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn table_names(&self) -> Vec<String> {
        vec!["sbtest".to_string()]
    }
}
