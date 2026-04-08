// optimizer/src/context.rs

use crate::cost::SimpleCostModel;
use crate::stats::{InMemoryStatisticsProvider, StatisticsProvider};
use std::sync::Arc;

/// Optimizer 全局上下文 - 持有成本模型和统计信息提供者
#[derive(Clone)]
pub struct OptimizerContext {
    /// 统计信息提供者
    pub stats_provider: Arc<dyn StatisticsProvider>,
    /// 成本模型
    pub cost_model: Arc<SimpleCostModel>,
}

impl OptimizerContext {
    pub fn new(stats_provider: Arc<dyn StatisticsProvider>, cost_model: SimpleCostModel) -> Self {
        Self {
            stats_provider,
            cost_model: Arc::new(cost_model),
        }
    }
}

impl Default for OptimizerContext {
    fn default() -> Self {
        Self {
            stats_provider: Arc::new(InMemoryStatisticsProvider::new()),
            cost_model: Arc::new(SimpleCostModel::default_model()),
        }
    }
}
