//! Workload generators for benchmark tests

pub mod oltp_point_select;
pub mod oltp_read_only;
pub mod oltp_read_write;

use async_trait::async_trait;
use std::sync::Arc;

use crate::db::Database;

/// Workload trait - defines a benchmark workload
#[async_trait]
pub trait Workload: Send + Sync {
    /// Execute one iteration of the workload
    async fn execute(&self, db: &dyn Database) -> anyhow::Result<()>;

    /// Get workload name
    fn name(&self) -> &str;
}

/// Create a workload by name
pub fn create_workload(name: &str, _scale: usize) -> Arc<dyn Workload> {
    match name.to_lowercase().as_str() {
        "oltp_point_select" => Arc::new(oltp_point_select::OltpPointSelect::new()),
        "oltp_read_only" => Arc::new(oltp_read_only::OltpReadOnly::new()),
        "oltp_read_write" => Arc::new(oltp_read_write::OltpReadWrite::new()),
        _ => panic!("Unknown workload: {}", name),
    }
}
