use sqlrustgo_planner::{PhysicalPlan, Schema};
use sqlrustgo_storage::predicate::Predicate;
use sqlrustgo_types::{SqlResult, Value};

pub trait ScanExecutor: Send {
    fn init(&mut self) -> SqlResult<()>;
    fn next(&mut self) -> SqlResult<Option<Vec<Value>>>;
    fn schema(&self) -> &Schema;
    fn name(&self) -> &str;
    fn close(&mut self) -> SqlResult<()>;
}

#[derive(Debug, Clone)]
pub struct ScanStats {
    pub rows_scanned: u64,
    pub rows_returned: u64,
    pub used_index: bool,
}

impl Default for ScanStats {
    fn default() -> Self {
        Self {
            rows_scanned: 0,
            rows_returned: 0,
            used_index: false,
        }
    }
}

pub trait IndexScanable {
    fn can_use_index(&self, predicate: &Predicate) -> bool;
    fn estimate_index_cost(&self, predicate: &Predicate) -> f64;
    fn estimate_seq_cost(&self) -> f64;
}
