use sqlrustgo_planner::PhysicalPlan;
use sqlrustgo_types::SqlResult;

use super::executor::{Executor, ExecutorResult};

pub struct LocalExecutor;

impl LocalExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LocalExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor for LocalExecutor {
    fn execute(&self, _plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        Ok(ExecutorResult::empty())
    }

    fn name(&self) -> &str {
        "local"
    }

    fn is_ready(&self) -> bool {
        true
    }
}
