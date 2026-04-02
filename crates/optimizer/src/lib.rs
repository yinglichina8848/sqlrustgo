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

pub use cost::{CboOptimizer, SimpleCostModel};
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

pub struct DefaultOptimizer {
    rules: Vec<Box<dyn Rule<Plan>>>,
    disabled_rules: std::collections::HashSet<String>,
    use_cbo: bool,
    cbo_optimizer: Option<CboOptimizer>,
}

impl DefaultOptimizer {
    pub fn new() -> Self {
        let mut optimizer = Self {
            rules: Vec::new(),
            disabled_rules: std::collections::HashSet::new(),
            use_cbo: false,
            cbo_optimizer: None,
        };
        optimizer.add_default_rules();
        optimizer
    }

    fn add_default_rules(&mut self) {
        self.rules.push(Box::new(ConstantFolding::new()));
        self.rules.push(Box::new(PredicatePushdown::new()));
        self.rules.push(Box::new(ProjectionPruning::new()));
        self.rules.push(Box::new(ExpressionSimplification::new()));
        self.rules.push(Box::new(IndexSelect::new()));
        self.rules.push(Box::new(JoinReordering::new()));
    }

    pub fn with_cbo(mut self, cbo: CboOptimizer) -> Self {
        self.use_cbo = true;
        self.cbo_optimizer = Some(cbo);
        self
    }

    pub fn enable_rule(&mut self, rule_name: &str) {
        self.disabled_rules.remove(rule_name);
    }

    pub fn disable_rule(&mut self, rule_name: &str) {
        self.disabled_rules.insert(rule_name.to_string());
    }

    pub fn add_rule(&mut self, rule: Box<dyn Rule<Plan>>) {
        self.rules.push(rule);
    }
}

impl Optimizer for DefaultOptimizer {
    fn optimize(&mut self, plan: &mut dyn std::any::Any) -> OptimizerResult<()> {
        if let Some(plan) = plan.downcast_mut::<Plan>() {
            let mut changed = true;
            let mut iterations = 0;
            const MAX_ITERATIONS: usize = 100;
            while changed && iterations < MAX_ITERATIONS {
                changed = false;
                iterations += 1;
                for rule in &self.rules {
                    if !self.disabled_rules.contains(rule.name()) {
                        if rule.apply(plan) {
                            changed = true;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl Default for DefaultOptimizer {
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

    #[test]
    fn test_default_optimizer_new() {
        let mut optimizer = DefaultOptimizer::new();
        let mut plan = String::from("test");
        let result = optimizer.optimize(&mut plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_optimizer_with_cbo() {
        let cbo = CboOptimizer::new();
        let mut optimizer = DefaultOptimizer::new().with_cbo(cbo);
        let mut plan = String::from("test");
        let result = optimizer.optimize(&mut plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cbo_optimizer_new() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_scan_cost("test_table");
        assert!(cost >= 0.0);
    }

    #[test]
    fn test_cbo_optimizer_select_access_method() {
        let cbo = CboOptimizer::new();
        let method = cbo.select_access_method("test_table", "id", 0.1);
        assert!(method == "seq_scan" || method == "index_scan");
    }
}
