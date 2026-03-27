//! OLTP Read Only workload
//!
//! Sysbench-compatible read-only workload

use async_trait::async_trait;

use crate::db::Database;

/// Read-only workload - multiple read queries
pub struct OltpReadOnly;

impl OltpReadOnly {
    pub fn new() -> Self {
        Self
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
}
