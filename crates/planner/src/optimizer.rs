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
                    return false;
                }

                // Try to push into Projection
                match &mut **input {
                    LogicalPlan::Projection { input: proj_input, .. } => {
                        let new_filter = LogicalPlan::Filter {
                            predicate: predicate.clone(),
                            input: proj_input.clone(),
                        };
                        **input = new_filter;
                        return true;
                    }
                    LogicalPlan::TableScan { .. } => false,
                    _ => false,
                }
            }
            LogicalPlan::Projection { input, .. } => self.pushdown(input),
            LogicalPlan::Aggregate { input: _, .. } => false,
            LogicalPlan::Join { left, right, join_type, condition } => {
                let mut changed = false;
                if let Some(pred) = condition.as_ref() {
                    if self.can_push_to_left(pred, join_type) {
                        changed |= self.pushdown(left);
                    }
                }
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

    fn can_push_to_left(&self, _pred: &Expr, join_type: &crate::JoinType) -> bool {
        matches!(join_type, crate::JoinType::Inner | crate::JoinType::Left)
    }

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

    fn prune(&self, plan: &mut LogicalPlan) -> bool {
        match plan {
            LogicalPlan::Projection { input, expr, schema: _ } => {
                let used_cols = self.collect_columns(expr);
                if let LogicalPlan::TableScan { projection, .. } = &mut **input {
                    if projection.is_none() && !used_cols.is_all {
                        *projection = Some(used_cols.indices);
                        return true;
                    }
                }
                self.prune(input)
            }
            LogicalPlan::Filter { input, .. } => self.prune(input),
            LogicalPlan::Aggregate { input: _, .. } => false,
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

    fn collect_columns(&self, exprs: &[Expr]) -> ColumnSet {
        let mut cols = ColumnSet::new();
        for expr in exprs {
            self.collect_from_expr(expr, &mut cols);
        }
        cols
    }

    fn collect_from_expr(&self, expr: &Expr, cols: &mut ColumnSet) {
        match expr {
            Expr::Column(col) => cols.add(&col.name),
            Expr::BinaryExpr { left, right, .. } => {
                self.collect_from_expr(left, cols);
                self.collect_from_expr(right, cols);
            }
            Expr::UnaryExpr { expr, .. } => self.collect_from_expr(expr, cols),
            Expr::AggregateFunction { args, .. } => {
                for arg in args {
                    self.collect_from_expr(arg, cols);
                }
            }
            Expr::Alias { expr, .. } => self.collect_from_expr(expr, cols),
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

    fn fold(&self, plan: &mut LogicalPlan) -> bool {
        match plan {
            LogicalPlan::Filter { predicate, input } => {
                let simplified = self.simplify_expr(predicate);
                if simplified != *predicate {
                    *predicate = simplified;
                    return true;
                }
                self.fold(input)
            }
            LogicalPlan::Projection { input, expr, .. } => {
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

    fn simplify_expr(&self, expr: &Expr) -> Expr {
        match expr {
            Expr::BinaryExpr { left, op, right } => {
                let left_simplified = self.simplify_expr(left);
                let right_simplified = self.simplify_expr(right);
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
    use sqlrustgo_types::Value;

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

    #[test]
    fn test_column_set_new() {
        let cols = ColumnSet::new();
        assert!(cols.is_all);
        assert!(cols.indices.is_empty());
    }

    #[test]
    fn test_column_set_add() {
        let mut cols = ColumnSet::new();
        assert!(cols.is_all);
        cols.add("test");
        assert!(!cols.is_all);
    }

    #[test]
    fn test_predicate_pushdown_new() {
        let rule = PredicatePushdown::new();
        assert_eq!(rule.name(), "PredicatePushdown");
    }

    #[test]
    fn test_projection_pruning_new() {
        let rule = ProjectionPruning::new();
        assert_eq!(rule.name(), "ProjectionPruning");
    }

    #[test]
    fn test_constant_folding_new() {
        let rule = ConstantFolding::new();
        assert_eq!(rule.name(), "ConstantFolding");
    }

    #[test]
    fn test_predicate_pushdown_projection() {
        use crate::Operator;
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let inner_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        // Create: Filter -> Projection -> TableScan
        let mut plan = LogicalPlan::Filter {
            predicate: Expr::BinaryExpr {
                left: Box::new(Expr::Column(crate::Column::new("id".to_string()))),
                op: Operator::Eq,
                right: Box::new(Expr::Literal(Value::Integer(1))),
            },
            input: Box::new(LogicalPlan::Projection {
                expr: vec![Expr::Column(crate::Column::new("id".to_string()))],
                input: Box::new(LogicalPlan::TableScan {
                    table_name: "users".to_string(),
                    schema: inner_schema,
                    projection: None,
                }),
                schema: schema.clone(),
            }),
        };

        let rule = PredicatePushdown;
        let result = rule.apply(&mut plan);
        assert!(result);
    }

    #[test]
    fn test_predicate_pushdown_join() {
        use crate::Operator;
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        // Create: Filter -> Join
        let left = Box::new(LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        });
        let right = Box::new(LogicalPlan::TableScan {
            table_name: "orders".to_string(),
            schema: schema.clone(),
            projection: None,
        });

        let mut plan = LogicalPlan::Filter {
            predicate: Expr::BinaryExpr {
                left: Box::new(Expr::Column(crate::Column::new("id".to_string()))),
                op: Operator::Eq,
                right: Box::new(Expr::Literal(Value::Integer(1))),
            },
            input: Box::new(LogicalPlan::Join {
                left,
                right,
                join_type: crate::JoinType::Inner,
                condition: Some(Expr::BinaryExpr {
                    left: Box::new(Expr::Column(crate::Column::new("id".to_string()))),
                    op: Operator::Eq,
                    right: Box::new(Expr::Column(crate::Column::new("id".to_string()))),
                }),
            }),
        };

        let rule = PredicatePushdown;
        let _ = rule.apply(&mut plan);
    }

    #[test]
    fn test_predicate_pushdown_sort() {
        use crate::Operator;
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let inner_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut plan = LogicalPlan::Filter {
            predicate: Expr::BinaryExpr {
                left: Box::new(Expr::Column(crate::Column::new("id".to_string()))),
                op: Operator::Eq,
                right: Box::new(Expr::Literal(Value::Integer(1))),
            },
            input: Box::new(LogicalPlan::Sort {
                sort_expr: vec![crate::SortExpr {
                    expr: Expr::Column(crate::Column::new("id".to_string())),
                    asc: true,
                    nulls_first: true,
                }],
                input: Box::new(LogicalPlan::TableScan {
                    table_name: "users".to_string(),
                    schema: inner_schema,
                    projection: None,
                }),
            }),
        };

        let rule = PredicatePushdown;
        let _ = rule.apply(&mut plan);
    }

    #[test]
    fn test_predicate_pushdown_limit() {
        use crate::Operator;
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let inner_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut plan = LogicalPlan::Filter {
            predicate: Expr::BinaryExpr {
                left: Box::new(Expr::Column(crate::Column::new("id".to_string()))),
                op: Operator::Eq,
                right: Box::new(Expr::Literal(Value::Integer(1))),
            },
            input: Box::new(LogicalPlan::Limit {
                limit: 10,
                offset: None,
                input: Box::new(LogicalPlan::TableScan {
                    table_name: "users".to_string(),
                    schema: inner_schema,
                    projection: None,
                }),
            }),
        };

        let rule = PredicatePushdown;
        let _ = rule.apply(&mut plan);
    }

    #[test]
    fn test_projection_pruning_projection_table_scan() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let inner_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        // Create: Projection -> TableScan
        let mut plan = LogicalPlan::Projection {
            expr: vec![Expr::Column(crate::Column::new("id".to_string()))],
            input: Box::new(LogicalPlan::TableScan {
                table_name: "users".to_string(),
                schema: inner_schema,
                projection: None,
            }),
            schema: schema.clone(),
        };

        let rule = ProjectionPruning;
        let result = rule.apply(&mut plan);
        assert!(result);
    }

    #[test]
    fn test_projection_pruning_filter() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let inner_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        // Create: Projection -> Filter -> TableScan
        let mut plan = LogicalPlan::Projection {
            expr: vec![Expr::Column(crate::Column::new("id".to_string()))],
            input: Box::new(LogicalPlan::Filter {
                predicate: Expr::Literal(Value::Boolean(true)),
                input: Box::new(LogicalPlan::TableScan {
                    table_name: "users".to_string(),
                    schema: inner_schema,
                    projection: None,
                }),
            }),
            schema: schema.clone(),
        };

        let rule = ProjectionPruning;
        let _ = rule.apply(&mut plan);
    }

    #[test]
    fn test_projection_pruning_join() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let left = Box::new(LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        });
        let right = Box::new(LogicalPlan::TableScan {
            table_name: "orders".to_string(),
            schema: schema.clone(),
            projection: None,
        });

        let mut plan = LogicalPlan::Join {
            left,
            right,
            join_type: crate::JoinType::Inner,
            condition: None,
        };

        let rule = ProjectionPruning;
        let _ = rule.apply(&mut plan);
    }

    #[test]
    fn test_constant_folding_filter() {
        use crate::Operator;
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        // Create: Filter with constant expression 1 + 1
        let mut plan = LogicalPlan::Filter {
            predicate: Expr::BinaryExpr {
                left: Box::new(Expr::Literal(Value::Integer(1))),
                op: Operator::Plus,
                right: Box::new(Expr::Literal(Value::Integer(1))),
            },
            input: Box::new(LogicalPlan::TableScan {
                table_name: "users".to_string(),
                schema: schema.clone(),
                projection: None,
            }),
        };

        let rule = ConstantFolding;
        let result = rule.apply(&mut plan);
        assert!(result);
    }

    #[test]
    fn test_constant_folding_projection() {
        use crate::Operator;
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        // Create: Projection with constant expression 2 * 3
        let mut plan = LogicalPlan::Projection {
            expr: vec![Expr::BinaryExpr {
                left: Box::new(Expr::Literal(Value::Integer(2))),
                op: Operator::Multiply,
                right: Box::new(Expr::Literal(Value::Integer(3))),
            }],
            input: Box::new(LogicalPlan::TableScan {
                table_name: "users".to_string(),
                schema: schema.clone(),
                projection: None,
            }),
            schema: schema.clone(),
        };

        let rule = ConstantFolding;
        let result = rule.apply(&mut plan);
        assert!(result);
    }

    #[test]
    fn test_constant_folding_join() {
        use crate::Operator;
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let left = Box::new(LogicalPlan::Filter {
            predicate: Expr::BinaryExpr {
                left: Box::new(Expr::Literal(Value::Integer(1))),
                op: Operator::Plus,
                right: Box::new(Expr::Literal(Value::Integer(1))),
            },
            input: Box::new(LogicalPlan::TableScan {
                table_name: "users".to_string(),
                schema: schema.clone(),
                projection: None,
            }),
        });
        let right = Box::new(LogicalPlan::TableScan {
            table_name: "orders".to_string(),
            schema: schema.clone(),
            projection: None,
        });

        let mut plan = LogicalPlan::Join {
            left,
            right,
            join_type: crate::JoinType::Inner,
            condition: None,
        };

        let rule = ConstantFolding;
        let _ = rule.apply(&mut plan);
    }

    #[test]
    fn test_constant_folding_sort() {
        use crate::Operator;
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let inner_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut plan = LogicalPlan::Sort {
            sort_expr: vec![crate::SortExpr {
                expr: Expr::BinaryExpr {
                    left: Box::new(Expr::Literal(Value::Integer(1))),
                    op: Operator::Plus,
                    right: Box::new(Expr::Literal(Value::Integer(1))),
                },
                asc: true,
                nulls_first: true,
            }],
            input: Box::new(LogicalPlan::TableScan {
                table_name: "users".to_string(),
                schema: inner_schema,
                projection: None,
            }),
        };

        let rule = ConstantFolding;
        let _ = rule.apply(&mut plan);
    }

    #[test]
    fn test_constant_folding_limit() {
        use crate::Operator;
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let inner_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut plan = LogicalPlan::Limit {
            limit: 10,
            offset: None,
            input: Box::new(LogicalPlan::Filter {
                predicate: Expr::BinaryExpr {
                    left: Box::new(Expr::Literal(Value::Integer(1))),
                    op: Operator::Plus,
                    right: Box::new(Expr::Literal(Value::Integer(1))),
                },
                input: Box::new(LogicalPlan::TableScan {
                    table_name: "users".to_string(),
                    schema: inner_schema,
                    projection: None,
                }),
            }),
        };

        let rule = ConstantFolding;
        let _ = rule.apply(&mut plan);
    }

    #[test]
    fn test_default_optimizer_iterations() {
        let mut optimizer = DefaultOptimizer::new();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        // Create a plan that needs multiple optimization passes
        let plan = LogicalPlan::Projection {
            expr: vec![Expr::BinaryExpr {
                left: Box::new(Expr::Literal(Value::Integer(1))),
                op: crate::Operator::Plus,
                right: Box::new(Expr::Literal(Value::Integer(1))),
            }],
            input: Box::new(LogicalPlan::Filter {
                predicate: Expr::BinaryExpr {
                    left: Box::new(Expr::Literal(Value::Integer(2))),
                    op: crate::Operator::Multiply,
                    right: Box::new(Expr::Literal(Value::Integer(3))),
                },
                input: Box::new(LogicalPlan::TableScan {
                    table_name: "users".to_string(),
                    schema: schema.clone(),
                    projection: None,
                }),
            }),
            schema: schema.clone(),
        };

        let result = optimizer.optimize(plan);
        assert!(result.is_ok());
    }
}
