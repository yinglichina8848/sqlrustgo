//! Optimizer Module
//!
//! # What (是什么)
//! 优化器模块，提供查询优化接口和规则引擎
//!
//! # Why (为什么)
//! 定义统一的优化器接口，支持可扩展的优化规则和成本模型
//!
//! # How (如何实现)
//! - Optimizer trait: 优化器主接口
//! - Rule trait: 优化规则接口
//! - CostModel trait: 成本模型接口

pub mod cost;
pub mod network_cost;
pub mod plan;
pub mod rules;
pub mod stats;

pub use network_cost::{NetworkCost, NetworkCostEstimator, SimpleNetworkCostEstimator};
pub use plan::{OptimizerError, OptimizerResult};
pub use stats::{
    ColumnStats, DefaultStatsCollector, InMemoryStatisticsProvider, StatisticsProvider,
    StatsCollector, StatsError, StatsResult, TableStats,
};

/// Optimizer trait - main interface for query optimization
///
/// # What
/// 优化器主接口，负责将逻辑计划转换为优化后的物理计划
///
/// # Why
/// 统一的优化器接口，便于实现不同的优化策略
///
/// # How
/// - optimize 方法接收逻辑计划
/// - 返回优化后的逻辑计划或错误
pub trait Optimizer<Plan> {
    /// Optimize a logical plan
    fn optimize(&self, plan: Plan) -> OptimizerResult<Plan>;
}

/// Rule trait - base interface for optimization rules
///
/// # What
/// 优化规则接口，每条规则负责特定的优化转换
///
/// # Why
/// 规则化设计便于扩展和维护优化规则
///
/// # How
/// - apply 方法尝试应用规则
/// - 返回是否发生了改变
pub trait Rule<Plan> {
    /// Get rule name
    fn name(&self) -> &str;

    /// Try to apply the rule to a plan
    fn apply(&self, plan: &mut Plan) -> bool;
}

/// CostModel trait - interface for cost estimation
///
/// # What
/// 成本模型接口，用于估算执行计划的成本
///
/// # Why
/// 为基于成本的优化 (CBO) 提供基础设施
///
/// # How
/// - estimate 方法计算计划的估算成本
pub trait CostModel<Plan> {
    /// Estimate the cost of a plan
    fn estimate(&self, plan: &Plan) -> f64;
}

/// NoOpOptimizer - a simple optimizer that returns the plan unchanged
#[derive(Debug, Clone, Default)]
pub struct NoOpOptimizer;

impl<Plan> Optimizer<Plan> for NoOpOptimizer {
    fn optimize(&self, plan: Plan) -> OptimizerResult<Plan> {
        Ok(plan)
    }
}

/// RuleSet - a collection of optimization rules
#[derive(Default)]
pub struct RuleSet<Plan> {
    rules: Vec<Box<dyn Rule<Plan>>>,
}

impl<Plan> RuleSet<Plan> {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: Box<dyn Rule<Plan>>) {
        self.rules.push(rule);
    }

    pub fn apply(&self, plan: &mut Plan) -> bool {
        let mut changed = false;
        for rule in &self.rules {
            if rule.apply(plan) {
                changed = true;
            }
        }
        changed
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_optimizer() {
        let optimizer = NoOpOptimizer;
        let plan = vec![1, 2, 3];
        let result = optimizer.optimize(plan);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_rule_set_empty() {
        let rules: RuleSet<Vec<i32>> = RuleSet::new();
        assert!(rules.is_empty());
    }

    #[test]
    fn test_rule_set_apply() {
        let mut rules: RuleSet<Vec<i32>> = RuleSet::new();

        struct NoOpRule;
        impl Rule<Vec<i32>> for NoOpRule {
            fn name(&self) -> &str {
                "NoOpRule"
            }
            fn apply(&self, _plan: &mut Vec<i32>) -> bool {
                false
            }
        }

        rules.add_rule(Box::new(NoOpRule));
        assert_eq!(rules.len(), 1);

        let mut plan = vec![1, 2, 3];
        let changed = rules.apply(&mut plan);
        assert!(!changed);
    }
}
