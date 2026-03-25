//! Benchmark workload definitions

pub mod oltp;

use async_trait::async_trait;
use std::sync::Arc;

use crate::db::Database;

/// Workload trait - defines a benchmark workload
#[async_trait]
#[allow(dead_code)]
pub trait Workload: Send + Sync {
    /// Execute one iteration of the workload
    async fn execute(&self, db: &dyn Database) -> anyhow::Result<()>;

    /// Get workload name
    fn name(&self) -> &str;
}

/// Create a workload by name
pub fn create_workload(name: &str, scale: usize) -> Arc<dyn Workload> {
    match name.to_lowercase().as_str() {
        "oltp" => Arc::new(oltp::OltpWorkload::new(scale)),
        "tpch" => {
            // TODO: Implement TPC-H workload
            todo!("TPC-H workload not yet implemented")
        }
        _ => panic!("Unknown workload: {}", name),
    }
}
