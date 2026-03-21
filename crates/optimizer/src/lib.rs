//! Optimizer Module
//!
//! This module provides query optimization interfaces for the SQL engine.

#![allow(clippy::type_complexity)]

pub mod cost;
pub mod network_cost;
pub mod plan;
pub mod projection_pushdown;
pub mod rules;
pub mod stats;

pub use cost::SimpleCostModel;
pub use network_cost::{NetworkCost, NetworkCostEstimator, SimpleNetworkCostEstimator};
pub use plan::{OptimizerError, OptimizerResult};
pub use projection_pushdown::{
    ColumnPruner, ProjectionPushdownConfig, ProjectionPushdownOptimizer, ProjectionPushdownRule,
};
pub use rules::{
    ConstantFolding, Expr, ExpressionSimplification, IndexSelect, JoinReordering, JoinType,
    MatchResult, Operator, Plan, PlanPattern, PredicatePushdown, ProjectionPruning, RuleContext,
    RuleMeta, SimpleColumnSet, Value,
};
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
    fn test_noop_optimizer() {
        let mut optimizer = NoOpOptimizer;
        let mut plan = String::from("test plan");
        let result = optimizer.optimize(&mut plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ruleset_new() {
        let ruleset = RuleSet::new();
        assert_eq!(ruleset.rules.len(), 0);
    }

    #[test]
    fn test_ruleset_add_rule() {
        let mut ruleset = RuleSet::new();
        let called = std::cell::RefCell::new(false);

        let rule = move |_plan: &mut dyn std::any::Any| {
            *called.borrow_mut() = true;
            true
        };

        ruleset.add_rule(rule);
        assert_eq!(ruleset.rules.len(), 1);
    }

    #[test]
    fn test_ruleset_apply() {
        let mut ruleset = RuleSet::new();

        let rule1 = |_plan: &mut dyn std::any::Any| -> bool { true };
        let rule2 = |_plan: &mut dyn std::any::Any| -> bool { false };

        ruleset.add_rule(rule1);
        ruleset.add_rule(rule2);

        let mut plan = String::from("test");
        let changed = ruleset.apply(&mut plan);
        assert!(changed); // At least one rule returned true
    }

    #[test]
    fn test_ruleset_apply_no_changes() {
        let mut ruleset = RuleSet::new();

        let rule = |_plan: &mut dyn std::any::Any| -> bool { false };
        ruleset.add_rule(rule);

        let mut plan = String::from("test");
        let changed = ruleset.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_optimizer_trait_object() {
        struct TestOptimizer {
            called: std::cell::RefCell<bool>,
        }

        impl Optimizer for TestOptimizer {
            fn optimize(&mut self, _plan: &mut dyn std::any::Any) -> OptimizerResult<()> {
                *self.called.borrow_mut() = true;
                Ok(())
            }
        }

        let mut optimizer = TestOptimizer {
            called: std::cell::RefCell::new(false),
        };
        let mut plan = String::from("test");
        optimizer.optimize(&mut plan).unwrap();
        assert!(*optimizer.called.borrow());
    }

    #[test]
    fn test_rule_trait() {
        struct TestRule;

        impl Rule<String> for TestRule {
            fn name(&self) -> &str {
                "TestRule"
            }

            fn apply(&self, plan: &mut String) -> bool {
                plan.push_str("_modified");
                true
            }
        }

        let rule = TestRule;
        assert_eq!(rule.name(), "TestRule");

        let mut plan = String::from("original");
        let changed = rule.apply(&mut plan);
        assert!(changed);
        assert_eq!(plan, "original_modified");
    }

    #[test]
    fn test_cost_model_trait() {
        struct TestCostModel;

        impl CostModel for TestCostModel {
            fn estimate_cost(&self, plan: &dyn std::any::Any) -> f64 {
                if let Some(s) = plan.downcast_ref::<String>() {
                    s.len() as f64
                } else {
                    0.0
                }
            }
        }

        let cost_model = TestCostModel;
        let plan = String::from("test plan");
        let cost = cost_model.estimate_cost(&plan);
        assert_eq!(cost, 9.0);
    }

    #[test]
    fn test_ruleset_default() {
        let ruleset = RuleSet::default();
        assert_eq!(ruleset.rules.len(), 0);
    }
}
