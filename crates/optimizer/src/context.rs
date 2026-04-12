// optimizer/src/context.rs

use crate::cost::CboOptimizer;
use std::sync::Arc;

/// Optimizer 全局上下文 - 持有 CboOptimizer 用于成本估算
#[derive(Clone)]
pub struct OptimizerContext {
    /// CboOptimizer 用于成本估算（包含 stats_provider）
    pub cbo: Arc<CboOptimizer>,
}

impl OptimizerContext {
    pub fn new(cbo: CboOptimizer) -> Self {
        Self { cbo: Arc::new(cbo) }
    }

    pub fn with_default_stats() -> Self {
        Self::new(CboOptimizer::new())
    }
}

impl Default for OptimizerContext {
    fn default() -> Self {
        Self::with_default_stats()
    }
}
