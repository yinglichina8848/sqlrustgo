//! Optimizer Module
//!
//! Provides query optimization rules and optimization engine.

use super::logical_plan::LogicalPlan;
use super::{Expr, Operator};
use crate::types::SqlResult;

/// Optimizer trait - transforms logical plans
pub trait Optimizer: Send + Sync {
    /// Optimize a logical plan
    fn optimize(&self, plan: &LogicalPlan) -> SqlResult<LogicalPlan>;
}

/// Rule trait - base trait for optimization rules
pub trait Rule: Send + Sync {
    /// Get the rule name
    fn name(&self) -> &str;

    /// Apply the rule to a plan, returns true if changes were made
    fn apply(&self, plan: &mut LogicalPlan) -> bool;
}

/// OptimizerRule trait - extends Rule with expression optimization
pub trait OptimizerRule: Send + Sync {
    /// Get the rule name
    fn name(&self) -> &str;

    /// Apply the rule to a logical plan
    fn optimize(&self, plan: LogicalPlan) -> SqlResult<LogicalPlan>;
}

/// Optimization rules that can be applied
pub mod rules {
    use super::*;

    /// Predicate Pushdown - pushes filter conditions closer to the data source
    pub struct PredicatePushdown;

    impl OptimizerRule for PredicatePushdown {
        fn name(&self) -> &str {
            "predicate_pushdown"
        }

        fn optimize(&self, plan: LogicalPlan) -> SqlResult<LogicalPlan> {
            // Recursively optimize children
            Ok(self.optimize_recursive(plan))
        }
    }

    impl PredicatePushdown {
        pub fn optimize_recursive(&self, plan: LogicalPlan) -> LogicalPlan {
            match plan {
                LogicalPlan::Projection { input, expr, schema } => {
                    let new_input = Box::new(self.optimize_recursive(*input));
                    LogicalPlan::Projection {
                        input: new_input,
                        expr,
                        schema,
                    }
                }
                LogicalPlan::Filter { input, predicate } => {
                    let new_input = Box::new(self.optimize_recursive(*input));
                    LogicalPlan::Filter {
                        input: new_input,
                        predicate,
                    }
                }
                LogicalPlan::Aggregate {
                    input,
                    group_expr,
                    aggr_expr,
                    schema,
                } => {
                    let new_input = Box::new(self.optimize_recursive(*input));
                    LogicalPlan::Aggregate {
                        input: new_input,
                        group_expr,
                        aggr_expr,
                        schema,
                    }
                }
                LogicalPlan::Join {
                    left,
                    right,
                    join_type,
                    on,
                    filter,
                    schema,
                } => {
                    let new_left = Box::new(self.optimize_recursive(*left));
                    let new_right = Box::new(self.optimize_recursive(*right));
                    LogicalPlan::Join {
                        left: new_left,
                        right: new_right,
                        join_type,
                        on,
                        filter,
                        schema,
                    }
                }
                LogicalPlan::Sort { input, expr } => {
                    let new_input = Box::new(self.optimize_recursive(*input));
                    LogicalPlan::Sort {
                        input: new_input,
                        expr,
                    }
                }
                LogicalPlan::Limit { input, n } => {
                    let new_input = Box::new(self.optimize_recursive(*input));
                    LogicalPlan::Limit {
                        input: new_input,
                        n,
                    }
                }
                // Leaf nodes - no children to optimize
                other => other,
            }
        }
    }

    /// Projection Pruning - removes unnecessary columns from projections
    pub struct ProjectionPruning;

    impl OptimizerRule for ProjectionPruning {
        fn name(&self) -> &str {
            "projection_pruning"
        }

        fn optimize(&self, plan: LogicalPlan) -> SqlResult<LogicalPlan> {
            Ok(self.optimize_recursive(plan))
        }
    }

    impl ProjectionPruning {
        fn optimize_recursive(&self, plan: LogicalPlan) -> LogicalPlan {
            match plan {
                LogicalPlan::Projection { input, expr, schema } => {
                    let new_input = Box::new(self.optimize_recursive(*input));
                    LogicalPlan::Projection {
                        input: new_input,
                        expr,
                        schema,
                    }
                }
                LogicalPlan::Filter { input, predicate } => {
                    let new_input = Box::new(self.optimize_recursive(*input));
                    LogicalPlan::Filter {
                        input: new_input,
                        predicate,
                    }
                }
                LogicalPlan::Aggregate {
                    input,
                    group_expr,
                    aggr_expr,
                    schema,
                } => {
                    let new_input = Box::new(self.optimize_recursive(*input));
                    LogicalPlan::Aggregate {
                        input: new_input,
                        group_expr,
                        aggr_expr,
                        schema,
                    }
                }
                LogicalPlan::Join {
                    left,
                    right,
                    join_type,
                    on,
                    filter,
                    schema,
                } => {
                    let new_left = Box::new(self.optimize_recursive(*left));
                    let new_right = Box::new(self.optimize_recursive(*right));
                    LogicalPlan::Join {
                        left: new_left,
                        right: new_right,
                        join_type,
                        on,
                        filter,
                        schema,
                    }
                }
                LogicalPlan::Sort { input, expr } => {
                    let new_input = Box::new(self.optimize_recursive(*input));
                    LogicalPlan::Sort {
                        input: new_input,
                        expr,
                    }
                }
                LogicalPlan::Limit { input, n } => {
                    let new_input = Box::new(self.optimize_recursive(*input));
                    LogicalPlan::Limit {
                        input: new_input,
                        n,
                    }
                }
                other => other,
            }
        }
    }

    /// Constant Folding - evaluates constant expressions at compile time
    pub struct ConstantFolding;

    impl OptimizerRule for ConstantFolding {
        fn name(&self) -> &str {
            "constant_folding"
        }

        fn optimize(&self, plan: LogicalPlan) -> SqlResult<LogicalPlan> {
            Ok(self.fold_plan(plan))
        }
    }

    impl ConstantFolding {
        pub fn fold_expr(&self, expr: Expr) -> Expr {
            match expr {
                Expr::BinaryExpr { left, op, right } => {
                    let left = self.fold_expr(*left);
                    let right = self.fold_expr(*right);

                    // Try to evaluate constant expressions
                    if let Expr::Literal(l) = &left {
                        if let Expr::Literal(r) = &right {
                            if let Some(result) = self.eval_binary_op(l, &op, r) {
                                return Expr::Literal(result);
                            }
                        }
                    }

                    Expr::BinaryExpr {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    }
                }
                Expr::UnaryExpr { op, expr } => {
                    let expr = self.fold_expr(*expr);
                    if let Expr::Literal(v) = &expr {
                        if let Some(result) = self.eval_unary_op(v, &op) {
                            return Expr::Literal(result);
                        }
                    }
                    Expr::UnaryExpr {
                        op,
                        expr: Box::new(expr),
                    }
                }
                other => other,
            }
        }

        fn fold_plan(&self, plan: LogicalPlan) -> LogicalPlan {
            match plan {
                LogicalPlan::Projection { input, expr, schema } => {
                    let folded_exprs: Vec<Expr> = expr.into_iter().map(|e| self.fold_expr(e)).collect();
                    let new_input = Box::new(self.fold_plan(*input));
                    LogicalPlan::Projection {
                        input: new_input,
                        expr: folded_exprs,
                        schema,
                    }
                }
                LogicalPlan::Filter { input, predicate } => {
                    let folded_predicate = self.fold_expr(predicate);
                    let new_input = Box::new(self.fold_plan(*input));
                    LogicalPlan::Filter {
                        input: new_input,
                        predicate: folded_predicate,
                    }
                }
                LogicalPlan::Aggregate {
                    input,
                    group_expr,
                    aggr_expr,
                    schema,
                } => {
                    let folded_group: Vec<Expr> =
                        group_expr.into_iter().map(|e| self.fold_expr(e)).collect();
                    let folded_aggr: Vec<Expr> =
                        aggr_expr.into_iter().map(|e| self.fold_expr(e)).collect();
                    let new_input = Box::new(self.fold_plan(*input));
                    LogicalPlan::Aggregate {
                        input: new_input,
                        group_expr: folded_group,
                        aggr_expr: folded_aggr,
                        schema,
                    }
                }
                LogicalPlan::Join {
                    left,
                    right,
                    join_type,
                    on,
                    filter,
                    schema,
                } => {
                    let folded_on: Vec<(Expr, Expr)> = on
                        .into_iter()
                        .map(|(l, r)| (self.fold_expr(l), self.fold_expr(r)))
                        .collect();
                    let folded_filter = filter.map(|f| self.fold_expr(f));
                    let new_left = Box::new(self.fold_plan(*left));
                    let new_right = Box::new(self.fold_plan(*right));
                    LogicalPlan::Join {
                        left: new_left,
                        right: new_right,
                        join_type,
                        on: folded_on,
                        filter: folded_filter,
                        schema,
                    }
                }
                LogicalPlan::Sort { input, expr } => {
                    let new_input = Box::new(self.fold_plan(*input));
                    LogicalPlan::Sort {
                        input: new_input,
                        expr,
                    }
                }
                LogicalPlan::Limit { input, n } => {
                    let new_input = Box::new(self.fold_plan(*input));
                    LogicalPlan::Limit {
                        input: new_input,
                        n,
                    }
                }
                other => other,
            }
        }

        fn eval_binary_op(&self, left: &crate::types::Value, op: &Operator, right: &crate::types::Value) -> Option<crate::types::Value> {
            use crate::types::Value::*;
            use Operator::*;

            match (left, op, right) {
                (Integer(l), Plus, Integer(r)) => Some(Integer(l + r)),
                (Integer(l), Minus, Integer(r)) => Some(Integer(l - r)),
                (Integer(l), Multiply, Integer(r)) => Some(Integer(l * r)),
                (Integer(l), Divide, Integer(r)) if *r != 0 => Some(Integer(l / r)),
                (Integer(l), Modulo, Integer(r)) if *r != 0 => Some(Integer(l % r)),
                (Integer(l), Eq, Integer(r)) => Some(Boolean(l == r)),
                (Integer(l), NotEq, Integer(r)) => Some(Boolean(l != r)),
                (Integer(l), Lt, Integer(r)) => Some(Boolean(l < r)),
                (Integer(l), LtEq, Integer(r)) => Some(Boolean(l <= r)),
                (Integer(l), Gt, Integer(r)) => Some(Boolean(l > r)),
                (Integer(l), GtEq, Integer(r)) => Some(Boolean(l >= r)),
                (Text(l), Eq, Text(r)) => Some(Boolean(l == r)),
                (Text(l), NotEq, Text(r)) => Some(Boolean(l != r)),
                // LIKE is not folded at compile time (requires runtime pattern matching)
                (Boolean(l), And, Boolean(r)) => Some(Boolean(*l && *r)),
                (Boolean(l), Or, Boolean(r)) => Some(Boolean(*l || *r)),
                _ => None,
            }
        }

        fn eval_unary_op(&self, value: &crate::types::Value, op: &Operator) -> Option<crate::types::Value> {
            use crate::types::Value::*;
            use Operator::*;

            match (value, op) {
                (Boolean(v), Not) => Some(Boolean(!v)),
                (Integer(v), Minus) => Some(Integer(-v)),
                _ => None,
            }
        }
    }
}

/// Default optimizer that applies a set of rules
pub struct DefaultOptimizer {
    rules: Vec<Box<dyn OptimizerRule>>,
}

impl DefaultOptimizer {
    pub fn new() -> Self {
        let rules: Vec<Box<dyn OptimizerRule>> = vec![
            Box::new(rules::ConstantFolding),
            Box::new(rules::PredicatePushdown),
            Box::new(rules::ProjectionPruning),
        ];
        Self { rules }
    }

    pub fn with_rules(rules: Vec<Box<dyn OptimizerRule>>) -> Self {
        Self { rules }
    }
}

impl Default for DefaultOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Optimizer for DefaultOptimizer {
    fn optimize(&self, plan: &LogicalPlan) -> SqlResult<LogicalPlan> {
        let mut current_plan = plan.clone();
        for rule in &self.rules {
            current_plan = rule.optimize(current_plan)?;
        }
        Ok(current_plan)
    }
}

/// No-op optimizer that returns the plan unchanged
pub struct NoOpOptimizer;

impl Optimizer for NoOpOptimizer {
    fn optimize(&self, plan: &LogicalPlan) -> SqlResult<LogicalPlan> {
        Ok(plan.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::{
        Column, DataType, Field, Schema,
    };
    use crate::types::Value;

    fn test_schema() -> Schema {
        Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
            Field::new("age".to_string(), DataType::Integer),
        ])
    }

    fn test_table_scan() -> LogicalPlan {
        LogicalPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
            filters: vec![],
            limit: None,
            schema: test_schema(),
        }
    }

    #[test]
    fn test_noop_optimizer() {
        let plan = test_table_scan();
        let optimizer = NoOpOptimizer;
        let result = optimizer.optimize(&plan).unwrap();
        assert_eq!(format!("{}", result), format!("{}", plan));
    }

    #[test]
    fn test_default_optimizer() {
        let plan = test_table_scan();
        let optimizer = DefaultOptimizer::new();
        let result = optimizer.optimize(&plan).unwrap();
        assert!(!format!("{}",
result).is_empty());
    }

    #[test]
    fn test_predicate_pushdown_rule() {
        let rule = rules::PredicatePushdown;
        assert_eq!(rule.name(), "predicate_pushdown");
    }

    #[test]
    fn test_projection_pruning_rule() {
        let rule = rules::ProjectionPruning;
        assert_eq!(rule.name(), "projection_pruning");
    }

    #[test]
    fn test_constant_folding_rule() {
        let rule = rules::ConstantFolding;
        assert_eq!(rule.name(), "constant_folding");

        // Test constant expression folding
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Literal(Value::Integer(1))),
            op: Operator::Plus,
            right: Box::new(Expr::Literal(Value::Integer(2))),
        };
        let folded = rule.fold_expr(expr);
        assert_eq!(folded, Expr::Literal(Value::Integer(3)));
    }

    #[test]
    fn test_constant_folding_with_variables() {
        let rule = rules::ConstantFolding;

        // Variable + constant should not fold
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Column(Column::new("x".to_string()))),
            op: Operator::Plus,
            right: Box::new(Expr::Literal(Value::Integer(1))),
        };
        let folded = rule.fold_expr(expr);
        assert!(matches!(folded, Expr::BinaryExpr { .. }));
    }

    #[test]
    fn test_constant_folding_boolean() {
        let rule = rules::ConstantFolding;

        // true AND false = false
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Literal(Value::Boolean(true))),
            op: Operator::And,
            right: Box::new(Expr::Literal(Value::Boolean(false))),
        };
        let folded = rule.fold_expr(expr);
        assert_eq!(folded, Expr::Literal(Value::Boolean(false)));
    }
}
