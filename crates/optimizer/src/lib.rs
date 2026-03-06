//! Optimizer Module
//!
//! This module provides query optimization interfaces for the SQL engine.

pub mod cost;
pub mod network_cost;
pub mod plan;
pub mod rules;
pub mod stats;

pub use cost::SimpleCostModel;
pub use network_cost::{NetworkCost, NetworkCostEstimator, SimpleNetworkCostEstimator};
pub use plan::{OptimizerError, OptimizerResult};
pub use rules::{ConstantFolding, PredicatePushdown, ProjectionPruning};
pub use stats::{
    ColumnStats, DefaultStatsCollector, InMemoryStatisticsProvider, StatsCollector,
    StatsError, StatsResult, StatisticsProvider, TableStats,
};

/// Optimizer trait - interface for query optimization
pub trait Optimizer {
    /// Optimize a query plan
    fn optimize(&mut self, plan: &mut dyn std::any::Any) -> OptimizerResult<()>;
}

/// Rule trait - interface for optimization rules
pub trait Rule<Plan> {
    /// Get rule name
    fn name(&self) -> &str;

    /// Apply the rule to a plan
    fn apply(&self, plan: &mut Plan) -> bool;
}

/// CostModel trait - interface for cost estimation
pub trait CostModel {
    /// Estimate cost for a plan
    fn estimate_cost(&self, plan: &dyn std::any::Any) -> f64;
}

/// NoOpOptimizer - optimizer that does nothing
pub struct NoOpOptimizer;

impl Optimizer for NoOpOptimizer {
    fn optimize(&mut self, _plan: &mut dyn std::any::Any) -> OptimizerResult<()> {
        Ok(())
    }
}

/// RuleSet - collection of optimization rules
pub struct RuleSet {
    rules: Vec<Box<dyn Fn(&mut dyn std::any::Any) -> bool>>,
}

impl RuleSet {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule<F>(&mut self, rule: F)
    where
        F: Fn(&mut dyn std::any::Any) -> bool + 'static,
    {
        self.rules.push(Box::new(rule));
    }

    pub fn apply(&mut self, plan: &mut dyn std::any::Any) -> bool {
        let mut changed = false;
        for rule in &self.rules {
            if rule(plan) {
                changed = true;
            }
        }
        changed
    }
}

impl Default for RuleSet {
    fn default() -> Self {
        Self::new()
    }
}
