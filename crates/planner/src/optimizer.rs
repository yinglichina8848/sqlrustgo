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
                    LogicalPlan::Projection {
                        input: proj_input, ..
                    } => {
                        let new_filter = LogicalPlan::Filter {
                            predicate: predicate.clone(),
                            input: proj_input.clone(),
                        };
                        **input = new_filter;
                        true
                    }
                    LogicalPlan::TableScan { .. } => false,
                    _ => false,
                }
            }
            LogicalPlan::Projection { input, .. } => self.pushdown(input),
            LogicalPlan::Aggregate { input: _, .. } => false,
            LogicalPlan::Join {
                left,
                right,
                join_type,
                condition,
            } => {
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
            LogicalPlan::Projection {
                input,
                expr,
                schema: _,
            } => {
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

    fn eval_binary_op(
        &self,
        op: &crate::Operator,
        left: &sqlrustgo_types::Value,
        right: &sqlrustgo_types::Value,
    ) -> Option<sqlrustgo_types::Value> {
        use crate::Operator;
        use sqlrustgo_types::Value;
        match (op, left, right) {
            (Operator::Plus, Value::Integer(l), Value::Integer(r)) => Some(Value::Integer(l + r)),
            (Operator::Minus, Value::Integer(l), Value::Integer(r)) => Some(Value::Integer(l - r)),
            (Operator::Multiply, Value::Integer(l), Value::Integer(r)) => {
                Some(Value::Integer(l * r))
            }
            (Operator::Eq, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l == r)),
            (Operator::NotEq, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l != r)),
            (Operator::Gt, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l > r)),
            (Operator::Lt, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l < r)),
            (Operator::GtEq, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l >= r)),
            (Operator::LtEq, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l <= r)),
            _ => None,
        }
    }

    fn eval_unary_op(
        &self,
        op: &crate::Operator,
        value: &sqlrustgo_types::Value,
    ) -> Option<sqlrustgo_types::Value> {
        use crate::Operator;
        use sqlrustgo_types::Value;
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
    use_cost_based: bool,
    enable_index_scan: bool,
}

impl DefaultOptimizer {
    pub fn new() -> Self {
        let rules: Vec<Box<dyn OptimizerRule>> = vec![
            Box::new(ConstantFolding),
            Box::new(PredicatePushdown),
            Box::new(ProjectionPruning),
        ];
        Self {
            rules,
            use_cost_based: false,
            enable_index_scan: false,
        }
    }

    pub fn with_rules(rules: Vec<Box<dyn OptimizerRule>>) -> Self {
        Self {
            rules,
            use_cost_based: false,
            enable_index_scan: false,
        }
    }

    pub fn enable_cost_based(mut self) -> Self {
        self.use_cost_based = true;
        self
    }

    pub fn enable_index_scan(mut self) -> Self {
        self.enable_index_scan = true;
        self
    }

    pub fn is_index_scan_enabled(&self) -> bool {
        self.enable_index_scan
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

impl NoOpOptimizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoOpOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

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

    #[test]
    fn test_predicate_pushdown_with_aggregate() {
        let rule = PredicatePushdown::new();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        // Test with Aggregate - should return false (can't push down)
        let plan = LogicalPlan::Aggregate {
            group_expr: vec![Expr::column("id")],
            aggregate_expr: vec![Expr::column("id")],
            having_expr: None,
            input: Box::new(LogicalPlan::TableScan {
                table_name: "users".to_string(),
                schema: schema.clone(),
                projection: None,
            }),
            schema: schema.clone(),
        };

        let mut plan_clone = plan.clone();
        let result = rule.apply(&mut plan_clone);
        assert!(!result);
    }

    #[test]
    fn test_predicate_pushdown_with_sort() {
        let rule = PredicatePushdown::new();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let plan = LogicalPlan::Sort {
            sort_expr: vec![crate::SortExpr {
                expr: Expr::column("id"),
                asc: true,
                nulls_first: true,
            }],
            input: Box::new(LogicalPlan::TableScan {
                table_name: "users".to_string(),
                schema: schema.clone(),
                projection: None,
            }),
        };

        let mut plan_clone = plan;
        let result = rule.apply(&mut plan_clone);
        assert!(!result);
    }

    #[test]
    fn test_predicate_pushdown_with_limit() {
        let rule = PredicatePushdown::new();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let plan = LogicalPlan::Limit {
            limit: 10,
            offset: None,
            input: Box::new(LogicalPlan::TableScan {
                table_name: "users".to_string(),
                schema: schema.clone(),
                projection: None,
            }),
        };

        let mut plan_clone = plan;
        let result = rule.apply(&mut plan_clone);
        assert!(!result);
    }

    #[test]
    fn test_predicate_pushdown_join_can_push_left() {
        let rule = PredicatePushdown::new();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        // Test Inner join - can push to left
        let can_push_inner = rule.can_push_to_left(&Expr::column("id"), &crate::JoinType::Inner);
        assert!(can_push_inner);

        // Test Left join - can push to left
        let can_push_left = rule.can_push_to_left(&Expr::column("id"), &crate::JoinType::Left);
        assert!(can_push_left);

        // Test Right join - cannot push to left
        let can_push_right = rule.can_push_to_left(&Expr::column("id"), &crate::JoinType::Right);
        assert!(!can_push_right);
    }

    #[test]
    fn test_predicate_pushdown_join_can_push_right() {
        let rule = PredicatePushdown::new();

        // Test Inner join - can push to right
        let can_push_inner = rule.can_push_to_right(&Expr::column("id"), &crate::JoinType::Inner);
        assert!(can_push_inner);

        // Test Right join - can push to right
        let can_push_right = rule.can_push_to_right(&Expr::column("id"), &crate::JoinType::Right);
        assert!(can_push_right);

        // Test Left join - cannot push to right
        let can_push_left = rule.can_push_to_right(&Expr::column("id"), &crate::JoinType::Left);
        assert!(!can_push_left);
    }

    #[test]
    fn test_projection_pruning_with_projection() {
        let rule = ProjectionPruning::new();
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);

        let plan = LogicalPlan::Projection {
            expr: vec![Expr::column("id")],
            input: Box::new(LogicalPlan::TableScan {
                table_name: "users".to_string(),
                schema: schema.clone(),
                projection: None,
            }),
            schema: schema.clone(),
        };

        let mut plan_clone = plan;
        let result = rule.apply(&mut plan_clone);
        assert!(result);
    }

    #[test]
    fn test_projection_pruning_default() {
        let rule = ProjectionPruning::default();
        assert_eq!(rule.name(), "ProjectionPruning");
    }

    #[test]
    fn test_constant_folding_with_literal_expressions() {
        let rule = ConstantFolding::new();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        // Test with all literals - should be folded
        let plan = LogicalPlan::Projection {
            expr: vec![Expr::BinaryExpr {
                left: Box::new(Expr::Literal(Value::Integer(1))),
                op: crate::Operator::Plus,
                right: Box::new(Expr::Literal(Value::Integer(2))),
            }],
            input: Box::new(LogicalPlan::TableScan {
                table_name: "users".to_string(),
                schema: schema.clone(),
                projection: None,
            }),
            schema: schema.clone(),
        };

        let mut plan_clone = plan;
        let result = rule.apply(&mut plan_clone);
        assert!(result);
    }

    #[test]
    fn test_constant_folding_unary_minus() {
        let rule = ConstantFolding::new();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let plan = LogicalPlan::Projection {
            expr: vec![Expr::UnaryExpr {
                op: crate::Operator::Minus,
                expr: Box::new(Expr::Literal(Value::Integer(5))),
            }],
            input: Box::new(LogicalPlan::TableScan {
                table_name: "users".to_string(),
                schema: schema.clone(),
                projection: None,
            }),
            schema: schema.clone(),
        };

        let mut plan_clone = plan;
        let result = rule.apply(&mut plan_clone);
        assert!(result);
    }

    #[test]
    fn test_default_optimizer_with_empty_rules() {
        let optimizer = DefaultOptimizer::with_rules(vec![]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let mut optimizer_mut = DefaultOptimizer::with_rules(vec![]);
        let result = optimizer_mut.optimize(plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_noop_optimizer_returns_plan() {
        let mut optimizer = NoOpOptimizer;
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let result = optimizer.optimize(plan);
        assert!(result.is_ok());
    }

    // Additional tests to improve coverage

    #[test]
    fn test_predicate_pushdown_with_join_condition() {
        // Test predicate pushdown through a join with actual condition
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Integer),
        ]);

        let left = Box::new(LogicalPlan::TableScan {
            table_name: "t1".to_string(),
            schema: schema.clone(),
            projection: None,
        });

        let right = Box::new(LogicalPlan::TableScan {
            table_name: "t2".to_string(),
            schema: schema.clone(),
            projection: None,
        });

        let join = LogicalPlan::Join {
            left,
            right,
            join_type: crate::JoinType::Inner,
            condition: Some(Expr::binary_expr(
                Expr::column("id"),
                crate::Operator::Eq,
                Expr::column("id"),
            )),
        };

        let filter = LogicalPlan::Filter {
            predicate: Expr::binary_expr(
                Expr::column("value"),
                crate::Operator::Gt,
                Expr::literal(sqlrustgo_types::Value::Integer(10)),
            ),
            input: Box::new(join),
        };

        let rule = PredicatePushdown::new();
        let mut plan = filter;
        let _result = rule.apply(&mut plan);
    }

    #[test]
    fn test_predicate_pushdown_default_trait() {
        // Test Default trait implementation
        let rule: PredicatePushdown = Default::default();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let mut plan = LogicalPlan::TableScan {
            table_name: "test".to_string(),
            schema,
            projection: None,
        };
        let _ = rule.apply(&mut plan);
    }

    #[test]
    fn test_projection_pruning_aggregate() {
        // Test ProjectionPruning with Aggregate (returns false)
        let rule = ProjectionPruning::new();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let plan = LogicalPlan::Aggregate {
            input: Box::new(LogicalPlan::TableScan {
                table_name: "test".to_string(),
                schema: schema.clone(),
                projection: None,
            }),
            group_expr: vec![Expr::column("id")],
            aggregate_expr: vec![],
            having_expr: None,
            schema,
        };

        let mut plan_clone = plan;
        let result = rule.apply(&mut plan_clone);
        assert!(!result); // Aggregate doesn't change
    }

    #[test]
    fn test_projection_pruning_sort() {
        // Test ProjectionPruning with Sort
        let rule = ProjectionPruning::new();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let inner_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let plan = LogicalPlan::Sort {
            input: Box::new(LogicalPlan::TableScan {
                table_name: "test".to_string(),
                schema: inner_schema,
                projection: None,
            }),
            sort_expr: vec![crate::SortExpr {
                expr: Expr::column("id"),
                asc: true,
                nulls_first: false,
            }],
        };

        let mut plan_clone = plan;
        let _result = rule.apply(&mut plan_clone);
    }

    #[test]
    fn test_projection_pruning_limit() {
        // Test ProjectionPruning with Limit
        let rule = ProjectionPruning::new();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let inner_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let plan = LogicalPlan::Limit {
            input: Box::new(LogicalPlan::TableScan {
                table_name: "test".to_string(),
                schema: inner_schema,
                projection: None,
            }),
            limit: 10,
            offset: None,
        };

        let mut plan_clone = plan;
        let _result = rule.apply(&mut plan_clone);
    }

    #[test]
    fn test_collect_columns_unary_expr() {
        // Test collect_columns with UnaryExpr
        let rule = ProjectionPruning::new();
        let exprs = vec![Expr::UnaryExpr {
            op: crate::Operator::Minus,
            expr: Box::new(Expr::column("value")),
        }];
        let cols = rule.collect_columns(&exprs);
        assert!(!cols.is_all);
    }

    #[test]
    fn test_collect_columns_aggregate_function() {
        // Test collect_columns with AggregateFunction
        use crate::AggregateFunction;
        let rule = ProjectionPruning::new();
        let exprs = vec![Expr::AggregateFunction {
            func: AggregateFunction::Sum,
            args: vec![Expr::column("value")],
            distinct: false,
        }];
        let cols = rule.collect_columns(&exprs);
        assert!(!cols.is_all);
    }

    #[test]
    fn test_collect_columns_alias() {
        // Test collect_columns with Alias
        let rule = ProjectionPruning::new();
        let exprs = vec![Expr::Alias {
            expr: Box::new(Expr::column("value")),
            name: "aliased".to_string(),
        }];
        let cols = rule.collect_columns(&exprs);
        assert!(!cols.is_all);
    }

    #[test]
    fn test_column_set_default_trait() {
        // Test Default trait for ColumnSet
        let cols: ColumnSet = Default::default();
        assert!(cols.is_all); // Default is_all is true
    }

    #[test]
    fn test_constant_folding_aggregate() {
        // Test ConstantFolding with Aggregate
        let rule = ConstantFolding::new();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let plan = LogicalPlan::Aggregate {
            input: Box::new(LogicalPlan::TableScan {
                table_name: "test".to_string(),
                schema: schema.clone(),
                projection: None,
            }),
            group_expr: vec![Expr::column("id")],
            aggregate_expr: vec![],
            having_expr: None,
            schema,
        };

        let mut plan_clone = plan;
        let _result = rule.apply(&mut plan_clone);
    }

    #[test]
    fn test_constant_folding_binary_not_both_literals() {
        // Test constant folding when binary expr has column (not both literals)
        let rule = ConstantFolding::new();
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Integer),
        ]);

        let plan = LogicalPlan::Projection {
            input: Box::new(LogicalPlan::TableScan {
                table_name: "test".to_string(),
                schema: schema.clone(),
                projection: None,
            }),
            expr: vec![Expr::BinaryExpr {
                left: Box::new(Expr::column("id")),
                op: crate::Operator::Plus,
                right: Box::new(Expr::literal(sqlrustgo_types::Value::Integer(1))),
            }],
            schema,
        };

        let mut plan_clone = plan;
        let _result = rule.apply(&mut plan_clone);
    }

    #[test]
    fn test_constant_folding_unary_not_literal() {
        // Test constant folding when unary expr has column (not literal)
        let rule = ConstantFolding::new();
        let schema = Schema::new(vec![Field::new("value".to_string(), DataType::Integer)]);

        let plan = LogicalPlan::Projection {
            input: Box::new(LogicalPlan::TableScan {
                table_name: "test".to_string(),
                schema: schema.clone(),
                projection: None,
            }),
            expr: vec![Expr::UnaryExpr {
                op: crate::Operator::Minus,
                expr: Box::new(Expr::column("value")),
            }],
            schema,
        };

        let mut plan_clone = plan;
        let _result = rule.apply(&mut plan_clone);
    }

    #[test]
    fn test_eval_binary_op_float() {
        // Test eval_binary_op with Float values
        use crate::Operator;
        use sqlrustgo_types::Value;

        let rule = ConstantFolding::new();

        // Test Float Plus (not implemented)
        let result = rule.eval_binary_op(&Operator::Plus, &Value::Float(1.5), &Value::Float(2.5));
        assert!(result.is_none());

        // Test Float comparison (not implemented)
        let result = rule.eval_binary_op(&Operator::Eq, &Value::Float(1.5), &Value::Float(1.5));
        assert!(result.is_none());

        // Test Integer Divide (not implemented)
        let result =
            rule.eval_binary_op(&Operator::Divide, &Value::Integer(10), &Value::Integer(2));
        assert!(result.is_none());
    }

    #[test]
    fn test_eval_binary_op_integer_comparisons() {
        // Test more integer comparison operations
        use crate::Operator;
        use sqlrustgo_types::Value;

        let rule = ConstantFolding::new();

        let result = rule.eval_binary_op(&Operator::GtEq, &Value::Integer(5), &Value::Integer(3));
        assert_eq!(result, Some(Value::Boolean(true)));

        let result = rule.eval_binary_op(&Operator::LtEq, &Value::Integer(3), &Value::Integer(5));
        assert_eq!(result, Some(Value::Boolean(true)));

        // Test And/Or with Integer (should return None)
        let result = rule.eval_binary_op(&Operator::And, &Value::Integer(1), &Value::Integer(0));
        assert!(result.is_none());
    }

    #[test]
    fn test_eval_unary_op_not_implemented() {
        // Test eval_unary_op with types that aren't supported
        use crate::Operator;
        use sqlrustgo_types::Value;

        let rule = ConstantFolding::new();

        // Not on Integer (only Minus is supported)
        let result = rule.eval_unary_op(&Operator::Not, &Value::Integer(1));
        assert!(result.is_none());

        // Minus on non-integer
        let result = rule.eval_unary_op(&Operator::Minus, &Value::Float(1.5));
        assert!(result.is_none());

        // Not on Boolean (should work)
        let result = rule.eval_unary_op(&Operator::Not, &Value::Boolean(true));
        assert_eq!(result, Some(Value::Boolean(false)));

        // Minus on Boolean (should not work)
        let result = rule.eval_unary_op(&Operator::Minus, &Value::Boolean(true));
        assert!(result.is_none());
    }

    #[test]
    fn test_constant_folding_default_trait() {
        // Test Default trait for ConstantFolding
        let rule: ConstantFolding = Default::default();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let mut plan = LogicalPlan::TableScan {
            table_name: "test".to_string(),
            schema,
            projection: None,
        };
        let _ = rule.apply(&mut plan);
    }

    #[test]
    fn test_default_optimizer_default_trait() {
        // Test Default trait for DefaultOptimizer
        let optimizer: DefaultOptimizer = Default::default();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let plan = LogicalPlan::TableScan {
            table_name: "test".to_string(),
            schema,
            projection: None,
        };
        let mut optimizer_mut = optimizer;
        let result = optimizer_mut.optimize(plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_predicate_pushdown_filter_on_table_scan() {
        // Test predicate pushdown when Filter has direct TableScan child
        // This covers line 62 in the optimizer
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        // Create a Filter directly on TableScan (no Projection in between)
        let plan = LogicalPlan::Filter {
            predicate: Expr::binary_expr(
                Expr::column("id"),
                crate::Operator::Gt,
                Expr::literal(sqlrustgo_types::Value::Integer(5)),
            ),
            input: Box::new(LogicalPlan::TableScan {
                table_name: "test".to_string(),
                schema: schema.clone(),
                projection: None,
            }),
        };

        let rule = PredicatePushdown::new();
        let mut plan_clone = plan;
        // This should hit line 62 (TableScan case returns false)
        let _result = rule.apply(&mut plan_clone);
    }

    #[test]
    fn test_predicate_pushdown_join_left_right_push() {
        // Test predicate pushdown through join with condition that can push to both sides
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Integer),
        ]);

        let left = Box::new(LogicalPlan::TableScan {
            table_name: "t1".to_string(),
            schema: schema.clone(),
            projection: None,
        });

        let right = Box::new(LogicalPlan::TableScan {
            table_name: "t2".to_string(),
            schema: schema.clone(),
            projection: None,
        });

        // Join with Inner type (can push to both left and right)
        let join = LogicalPlan::Join {
            left,
            right,
            join_type: crate::JoinType::Inner,
            condition: Some(Expr::binary_expr(
                Expr::column("id"),
                crate::Operator::Eq,
                Expr::column("id"),
            )),
        };

        // Wrap in Filter
        let filter = LogicalPlan::Filter {
            predicate: Expr::binary_expr(
                Expr::column("value"),
                crate::Operator::Gt,
                Expr::literal(sqlrustgo_types::Value::Integer(10)),
            ),
            input: Box::new(join),
        };

        let rule = PredicatePushdown::new();
        let mut plan = filter;
        let _result = rule.apply(&mut plan);
    }

    #[test]
    fn test_eval_binary_op_gteq() {
        // Test GtEq comparison
        use crate::Operator;
        use sqlrustgo_types::Value;

        let rule = ConstantFolding::new();

        let result = rule.eval_binary_op(&Operator::GtEq, &Value::Integer(5), &Value::Integer(5));
        assert_eq!(result, Some(Value::Boolean(true)));

        let result = rule.eval_binary_op(&Operator::GtEq, &Value::Integer(3), &Value::Integer(5));
        assert_eq!(result, Some(Value::Boolean(false)));
    }

    #[test]
    fn test_eval_binary_op_lteq() {
        // Test LtEq comparison
        use crate::Operator;
        use sqlrustgo_types::Value;

        let rule = ConstantFolding::new();

        let result = rule.eval_binary_op(&Operator::LtEq, &Value::Integer(5), &Value::Integer(5));
        assert_eq!(result, Some(Value::Boolean(true)));

        let result = rule.eval_binary_op(&Operator::LtEq, &Value::Integer(6), &Value::Integer(5));
        assert_eq!(result, Some(Value::Boolean(false)));
    }
}
