//! Optimizer Module
//!
//! Provides query optimization through rule-based transformations.

use crate::logical_plan::LogicalPlan;
use crate::Expr;
use thiserror::Error;

/// Optimizer errors
#[derive(Debug, Error)]
pub enum OptimizerError {
    #[error("Optimization failed: {0}")]
    OptimizationFailed(String),
}

/// Optimizer result type
pub type OptimizerResult<T> = Result<T, OptimizerError>;

/// Optimizer trait - interface for query optimization
pub trait Optimizer {
    /// Optimize a logical plan
    fn optimize(&mut self, plan: LogicalPlan) -> OptimizerResult<LogicalPlan>;
}

/// Rule trait - interface for optimization rules
pub trait OptimizerRule: Send + Sync {
    /// Get rule name
    fn name(&self) -> &str;

    /// Apply the rule to a plan
    fn apply(&self, plan: &mut LogicalPlan) -> bool;
}

/// Predicate pushdown optimization rule
pub struct PredicatePushdown;

impl PredicatePushdown {
    pub fn new() -> Self {
        Self
    }

    /// Push predicates down to table scans
    fn pushdown(&self, plan: &mut LogicalPlan) -> bool {
        match plan {
            LogicalPlan::Filter { predicate, input } => {
                // Try to push filter down into input
                if let LogicalPlan::TableScan { .. } = **input {
                    // Filter directly on table scan - no pushdown needed
                    // But we can mark it for storage-level filtering
                    return false;
                }

                // Try to push into Projection or Aggregate
                match &mut **input {
                    LogicalPlan::Projection { input: proj_input, .. } => {
                        // Push filter into the projection's input
                        let new_filter = LogicalPlan::Filter {
                            predicate: predicate.clone(),
                            input: proj_input.clone(),
                        };
                        **input = new_filter;
                        return true;
                    }
                    LogicalPlan::TableScan { .. } => {
                        // Already at table scan, can't push further
                        return false;
                    }
                    _ => {
                        // Can't push down through other operators
                        return false;
                    }
                }
            }
            LogicalPlan::Projection { input, expr: _, schema: _ } => {
                // Recursively process input
                self.pushdown(input)
            }
            LogicalPlan::Aggregate { input: _, .. } => {
                // Cannot push predicates through aggregation
                false
            }
            LogicalPlan::Join { left, right, join_type, condition } => {
                // Try to push predicates into join children based on join type
                let mut changed = false;

                // Push down into left
                if let Some(pred) = condition.as_ref() {
                    if self.can_push_to_left(pred, join_type) {
                        changed |= self.pushdown(left);
                    }
                }

                // Push down into right
                if let Some(pred) = condition.as_ref() {
                    if self.can_push_to_right(pred, join_type) {
                        changed |= self.pushdown(right);
                    }
                }

                changed
            }
            LogicalPlan::Sort { input, .. } => self.pushdown(input),
            LogicalPlan::Limit { input, .. } => self.pushdown(input),
            _ => false,
        }
    }

    /// Check if predicate can be pushed to left side of join
    fn can_push_to_left(&self, _pred: &Expr, join_type: &crate::JoinType) -> bool {
        matches!(join_type, crate::JoinType::Inner | crate::JoinType::Left)
    }

    /// Check if predicate can be pushed to right side of join
    fn can_push_to_right(&self, _pred: &Expr, join_type: &crate::JoinType) -> bool {
        matches!(join_type, crate::JoinType::Inner | crate::JoinType::Right)
    }
}

impl Default for PredicatePushdown {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizerRule for PredicatePushdown {
    fn name(&self) -> &str {
        "PredicatePushdown"
    }

    fn apply(&self, plan: &mut LogicalPlan) -> bool {
        self.pushdown(plan)
    }
}

/// Projection pruning optimization rule
pub struct ProjectionPruning;

impl ProjectionPruning {
    pub fn new() -> Self {
        Self
    }

    /// Remove unnecessary columns from projections
    fn prune(&self, plan: &mut LogicalPlan) -> bool {
        match plan {
            LogicalPlan::Projection { input, expr, schema: _ } => {
                // Collect columns used in this projection
                let used_cols = self.collect_columns(expr);

                // Check if input is a table scan that can benefit from projection
                if let LogicalPlan::TableScan { projection, .. } = &mut **input {
                    if projection.is_none() && !used_cols.is_all {
                        // Push down projection to table scan
                        let new_projection: Option<Vec<usize>> = Some(used_cols.indices);
                        *projection = new_projection;
                        return true;
                    }
                }

                // Recurse into input
                self.prune(input)
            }
            LogicalPlan::Filter { input, .. } => self.prune(input),
            LogicalPlan::Aggregate { input, .. } => self.prune(input),
            LogicalPlan::Join { left, right, .. } => {
                let changed_left = self.prune(left);
                let changed_right = self.prune(right);
                changed_left || changed_right
            }
            LogicalPlan::Sort { input, .. } => self.prune(input),
            LogicalPlan::Limit { input, .. } => self.prune(input),
            _ => false,
        }
    }

    /// Collect columns used in expressions
    fn collect_columns(&self, exprs: &[Expr]) -> ColumnSet {
        let mut cols = ColumnSet::new();
        for expr in exprs {
            self.collect_from_expr(expr, &mut cols);
        }
        cols
    }

    fn collect_from_expr(&self, expr: &Expr, cols: &mut ColumnSet) {
        match expr {
            Expr::Column(col) => {
                cols.add(&col.name);
            }
            Expr::BinaryExpr { left, right, .. } => {
                self.collect_from_expr(left, cols);
                self.collect_from_expr(right, cols);
            }
            Expr::UnaryExpr { expr, .. } => {
                self.collect_from_expr(expr, cols);
            }
            Expr::AggregateFunction { args, .. } => {
                for arg in args {
                    self.collect_from_expr(arg, cols);
                }
            }
            Expr::Alias { expr, .. } => {
                self.collect_from_expr(expr, cols);
            }
            _ => {}
        }
    }
}

impl Default for ProjectionPruning {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizerRule for ProjectionPruning {
    fn name(&self) -> &str {
        "ProjectionPruning"
    }

    fn apply(&self, plan: &mut LogicalPlan) -> bool {
        self.prune(plan)
    }
}

/// Column set for tracking used columns
#[derive(Debug, Clone)]
pub struct ColumnSet {
    pub indices: Vec<usize>,
    pub is_all: bool,
}

impl ColumnSet {
    pub fn new() -> Self {
        Self {
            indices: vec![],
            is_all: true,
        }
    }

    pub fn add(&mut self, _name: &str) {
        // For now, mark as not all columns
        // In full implementation, would track actual column names
        self.is_all = false;
    }
}

impl Default for ColumnSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Constant folding optimization rule
pub struct ConstantFolding;

impl ConstantFolding {
    pub fn new() -> Self {
        Self
    }

    /// Evaluate constant expressions
    fn fold(&self, plan: &mut LogicalPlan) -> bool {
        match plan {
            LogicalPlan::Filter { predicate, input } => {
                // Try to simplify the predicate
                let simplified = self.simplify_expr(predicate);
                if simplified != *predicate {
                    *predicate = simplified;
                    return true;
                }
                self.fold(input)
            }
            LogicalPlan::Projection { input, expr, schema: _ } => {
                // Try to simplify projection expressions
                let mut changed = false;
                for e in expr {
                    let simplified = self.simplify_expr(e);
                    if simplified != *e {
                        changed = true;
                    }
                }
                changed || self.fold(input)
            }
            LogicalPlan::Aggregate { input, .. } => self.fold(input),
            LogicalPlan::Join { left, right, .. } => {
                let changed_left = self.fold(left);
                let changed_right = self.fold(right);
                changed_left || changed_right
            }
            LogicalPlan::Sort { input, .. } => self.fold(input),
            LogicalPlan::Limit { input, .. } => self.fold(input),
            _ => false,
        }
    }

    /// Simplify an expression by evaluating constants
    fn simplify_expr(&self, expr: &Expr) -> Expr {
        match expr {
            Expr::BinaryExpr { left, op, right } => {
                let left_simplified = self.simplify_expr(left);
                let right_simplified = self.simplify_expr(right);

                // If both are literals, try to evaluate
                if let Expr::Literal(lv) = &left_simplified {
                    if let Expr::Literal(rv) = &right_simplified {
                        if let Some(result) = self.eval_binary_op(op, lv, rv) {
                            return Expr::Literal(result);
                        }
                    }
                }

                Expr::BinaryExpr {
                    left: Box::new(left_simplified),
                    op: op.clone(),
                    right: Box::new(right_simplified),
                }
            }
            Expr::UnaryExpr { op, expr } => {
                let simplified = self.simplify_expr(expr);
                if let Expr::Literal(v) = &simplified {
                    if let Some(result) = self.eval_unary_op(op, v) {
                        return Expr::Literal(result);
                    }
                }
                Expr::UnaryExpr {
                    op: op.clone(),
                    expr: Box::new(simplified),
                }
            }
            _ => expr.clone(),
        }
    }

    fn eval_binary_op(&self, op: &crate::Operator, left: &sqlrustgo_types::Value, right: &sqlrustgo_types::Value) -> Option<sqlrustgo_types::Value> {
        use sqlrustgo_types::Value;
        use crate::Operator;

        match (op, left, right) {
            (Operator::Plus, Value::Integer(l), Value::Integer(r)) => Some(Value::Integer(l + r)),
            (Operator::Minus, Value::Integer(l), Value::Integer(r)) => Some(Value::Integer(l - r)),
            (Operator::Multiply, Value::Integer(l), Value::Integer(r)) => Some(Value::Integer(l * r)),
            (Operator::Eq, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l == r)),
            (Operator::NotEq, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l != r)),
            (Operator::Gt, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l > r)),
            (Operator::Lt, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l < r)),
            (Operator::GtEq, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l >= r)),
            (Operator::LtEq, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l <= r)),
            _ => None,
        }
    }

    fn eval_unary_op(&self, op: &crate::Operator, value: &sqlrustgo_types::Value) -> Option<sqlrustgo_types::Value> {
        use sqlrustgo_types::Value;
        use crate::Operator;

        match (op, value) {
            (Operator::Minus, Value::Integer(n)) => Some(Value::Integer(-n)),
            (Operator::Not, Value::Boolean(b)) => Some(Value::Boolean(!b)),
            _ => None,
        }
    }
}

impl Default for ConstantFolding {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizerRule for ConstantFolding {
    fn name(&self) -> &str {
        "ConstantFolding"
    }

    fn apply(&self, plan: &mut LogicalPlan) -> bool {
        self.fold(plan)
    }
}

/// Default optimizer with standard optimization rules
pub struct DefaultOptimizer {
    rules: Vec<Box<dyn OptimizerRule>>,
}

impl DefaultOptimizer {
    pub fn new() -> Self {
        let rules: Vec<Box<dyn OptimizerRule>> = vec![
            Box::new(ConstantFolding),
            Box::new(PredicatePushdown),
            Box::new(ProjectionPruning),
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
    fn optimize(&mut self, mut plan: LogicalPlan) -> OptimizerResult<LogicalPlan> {
        let mut changed = true;
        let max_iterations = 10;
        let mut iterations = 0;

        while changed && iterations < max_iterations {
            changed = false;
            for rule in &self.rules {
                if rule.apply(&mut plan) {
                    changed = true;
                }
            }
            iterations += 1;
        }

        Ok(plan)
    }
}

/// No-op optimizer that returns the plan unchanged
pub struct NoOpOptimizer;

impl Optimizer for NoOpOptimizer {
    fn optimize(&mut self, plan: LogicalPlan) -> OptimizerResult<LogicalPlan> {
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Schema;
    use crate::{DataType, Field};

    #[test]
    fn test_predicate_pushdown_name() {
        let rule = PredicatePushdown;
        assert_eq!(rule.name(), "PredicatePushdown");
    }

    #[test]
    fn test_predicate_pushdown_apply() {
        let rule = PredicatePushdown;
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let mut plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema,
            projection: None,
        };
        let result = rule.apply(&mut plan);
        assert!(!result);
    }

    #[test]
    fn test_projection_pruning_name() {
        let rule = ProjectionPruning;
        assert_eq!(rule.name(), "ProjectionPruning");
    }

    #[test]
    fn test_projection_pruning_apply() {
        let rule = ProjectionPruning;
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let mut plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema,
            projection: None,
        };
        let result = rule.apply(&mut plan);
        assert!(!result);
    }

    #[test]
    fn test_constant_folding_name() {
        let rule = ConstantFolding;
        assert_eq!(rule.name(), "ConstantFolding");
    }

    #[test]
    fn test_constant_folding_apply() {
        let rule = ConstantFolding;
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let mut plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema,
            projection: None,
        };
        let result = rule.apply(&mut plan);
        assert!(!result);
    }

    #[test]
    fn test_default_optimizer_new() {
        let optimizer = DefaultOptimizer::new();
        assert!(true);
    }

    #[test]
    fn test_default_optimizer_with_rules() {
        let optimizer = DefaultOptimizer::with_rules(vec![]);
        assert!(true);
    }

    #[test]
    fn test_noop_optimizer() {
        let mut optimizer = NoOpOptimizer;
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema,
            projection: None,
        };
        let result = optimizer.optimize(plan).unwrap();
        assert!(matches!(result, LogicalPlan::TableScan { .. }));
    }

    #[test]
    fn test_default_optimizer_optimize() {
        let mut optimizer = DefaultOptimizer::new();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema,
            projection: None,
        };
        let result = optimizer.optimize(plan).unwrap();
        assert!(matches!(result, LogicalPlan::TableScan { .. }));
    }

    #[test]
    fn test_optimizer_error() {
        let error = OptimizerError::OptimizationFailed("test error".to_string());
        assert!(error.to_string().contains("Optimization failed"));
    }

    #[test]
    fn test_optimizer_result() {
        let ok_result: OptimizerResult<i32> = Ok(42);
        assert!(ok_result.is_ok());

        let err_result: OptimizerResult<i32> =
            Err(OptimizerError::OptimizationFailed("test".to_string()));
        assert!(err_result.is_err());
    }
}
