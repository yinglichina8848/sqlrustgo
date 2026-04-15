//! OLTP Update Non-Index workload
//!
//! Sysbench-compatible update on non-indexed column 'c'

use async_trait::async_trait;
use rand::rngs::SmallRng;
use rand::Rng;

use crate::db::Database;

/// Update non-index workload - update on non-indexed column 'c'
pub struct OltpUpdateNonIndex {
    max_id: u64,
    statements_per_tx: usize,
}

impl OltpUpdateNonIndex {
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
}

impl Default for OltpUpdateNonIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::workload::Workload for OltpUpdateNonIndex {
    async fn execute(&self, db: &dyn Database) -> anyhow::Result<()> {
        let sql = self.generate_sql(&mut SmallRng::from_entropy());
        db.execute(&sql).await
    }

    fn name(&self) -> &str {
        "oltp_update_non_index"
    }

    fn generate_sql(&self, rng: &mut SmallRng) -> String {
        let id = rng.gen_range(1..self.max_id);
        let c = Self::generate_random_string(120);
        format!("UPDATE sbtest SET c = '{}' WHERE id = {}", c, id)
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