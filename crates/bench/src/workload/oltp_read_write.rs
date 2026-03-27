//! OLTP Read Write workload
//!
//! Sysbench-compatible read-write workload

use async_trait::async_trait;

use crate::db::Database;

/// Read-write workload - mixed read and write queries
pub struct OltpReadWrite;

impl OltpReadWrite {
    pub fn new() -> Self {
        Self
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
}
