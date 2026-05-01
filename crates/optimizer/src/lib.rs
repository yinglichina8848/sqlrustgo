//! Optimizer Module
//!
//! This module provides query optimization interfaces for the SQL engine.

#![allow(clippy::type_complexity)]

pub mod cost;
pub mod graph_cost;
pub mod index_selector;
pub mod network_cost;
pub mod path_selector;
pub mod plan;
pub mod query_planner;
pub mod rules;
pub mod stats;
pub mod unified_cost;
pub mod unified_plan;
pub mod vector_cost;

pub use cost::SimpleCostModel;
pub use network_cost::{NetworkCost, NetworkCostEstimator, SimpleNetworkCostEstimator};
pub use plan::{OptimizerError, OptimizerResult};
pub use rules::{ConstantFolding, PredicatePushdown, ProjectionPruning};
pub use stats::{
    ColumnStats, DefaultStatsCollector, InMemoryStatisticsProvider, StatisticsProvider,
    StatsCollector, StatsError, StatsResult, TableStats,
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
#[allow(clippy::type_complexity)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_optimizer_optimize() {
        let mut optimizer = NoOpOptimizer;
        let mut plan: Box<dyn std::any::Any> = Box::new(42i32);
        let result = optimizer.optimize(&mut plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_noop_optimizer_trait_object() {
        let mut optimizer: Box<dyn Optimizer> = Box::new(NoOpOptimizer);
        let mut plan: Box<dyn std::any::Any> = Box::new("test".to_string());
        assert!(optimizer.optimize(&mut plan).is_ok());
    }

    #[test]
    fn test_rule_set_new() {
        let mut rule_set = RuleSet::new();
        assert!(rule_set.apply(&mut Box::new(0i32) as &mut dyn std::any::Any) == false);
    }

    #[test]
    fn test_rule_set_default() {
        let mut rule_set = RuleSet::default();
        assert!(rule_set.apply(&mut Box::new(0i32) as &mut dyn std::any::Any) == false);
    }

    #[test]
    fn test_rule_set_add_rule() {
        let mut rule_set = RuleSet::new();
        rule_set.add_rule(|_| true);
        assert!(rule_set.apply(&mut Box::new(0i32) as &mut dyn std::any::Any) == true);
    }

    #[test]
    fn test_rule_set_multiple_rules() {
        let mut rule_set = RuleSet::new();
        rule_set.add_rule(|_| false);
        rule_set.add_rule(|_| true);
        rule_set.add_rule(|_| false);
        assert!(rule_set.apply(&mut Box::new(0i32) as &mut dyn std::any::Any) == true);
    }

    #[test]
    fn test_rule_set_no_changes() {
        let mut rule_set = RuleSet::new();
        rule_set.add_rule(|_| false);
        rule_set.add_rule(|_| false);
        assert!(rule_set.apply(&mut Box::new(0i32) as &mut dyn std::any::Any) == false);
    }

    #[test]
    fn test_rule_trait() {
        struct TestRule;
        impl Rule<i32> for TestRule {
            fn name(&self) -> &str {
                "TestRule"
            }
            fn apply(&self, plan: &mut i32) -> bool {
                *plan += 1;
                true
            }
        }
        let rule = TestRule;
        assert_eq!(rule.name(), "TestRule");
        let mut plan = 10i32;
        assert!(rule.apply(&mut plan));
        assert_eq!(plan, 11);
    }

    #[test]
    fn test_cost_model_trait() {
        struct TestCostModel;
        impl CostModel for TestCostModel {
            fn estimate_cost(&self, _plan: &dyn std::any::Any) -> f64 {
                42.0
            }
        }
        let model = TestCostModel;
        let cost = model.estimate_cost(&0i32);
        assert_eq!(cost, 42.0);
    }
}
