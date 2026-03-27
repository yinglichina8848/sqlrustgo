//! OLTP Point Select workload
//!
//! Sysbench-compatible point select workload

use async_trait::async_trait;

use crate::db::Database;

/// Point select workload - single row lookup by primary key
pub struct OltpPointSelect;

impl OltpPointSelect {
    pub fn new() -> Self {
        Self
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
}
