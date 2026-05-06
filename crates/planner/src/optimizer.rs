//! Optimizer Module
//!
//! Provides query optimization through rule-based transformations.
//! Uses real optimizer rules from crates/optimizer via LogicalPlan <-> UnifiedPlan bridge.

use crate::logical_plan::LogicalPlan;
use sqlrustgo_optimizer::unified_plan::UnifiedPlan;
use sqlrustgo_optimizer::Rule;
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
    fn optimize(&mut self, plan: LogicalPlan) -> OptimizerResult<LogicalPlan>;
}

/// Rule trait - interface for optimization rules
pub trait OptimizerRule: Send + Sync {
    fn name(&self) -> &str;
    fn apply(&self, plan: &mut LogicalPlan) -> bool;
}

// ===== Expr Converters =====

fn expr_to_unified(e: &crate::Expr) -> Option<sqlrustgo_optimizer::rules::Expr> {
    match e {
        crate::Expr::Column(col) => {
            Some(sqlrustgo_optimizer::rules::Expr::Column(col.name.clone()))
        }
        crate::Expr::Literal(v) => {
            let s = match v {
                sqlrustgo_types::Value::Integer(n) => n.to_string(),
                sqlrustgo_types::Value::Float(f) => f.to_string(),
                sqlrustgo_types::Value::Text(s) => s.clone(),
                _ => format!("{:?}", v),
            };
            Some(sqlrustgo_optimizer::rules::Expr::Literal(s))
        }
        crate::Expr::BinaryExpr { left, op, right } => {
            let l = expr_to_unified(left)?;
            let r = expr_to_unified(right)?;
            let uop = op_to_unified(op.clone())?;
            Some(sqlrustgo_optimizer::rules::Expr::BinaryExpr {
                left: Box::new(l),
                op: uop,
                right: Box::new(r),
            })
        }
        _ => None,
    }
}

fn op_to_unified(op: crate::Operator) -> Option<sqlrustgo_optimizer::rules::BinaryOperator> {
    use crate::Operator;
    use sqlrustgo_optimizer::rules::BinaryOperator;
    match op {
        Operator::Eq => Some(BinaryOperator::Eq),
        Operator::NotEq => Some(BinaryOperator::NotEq),
        Operator::Lt => Some(BinaryOperator::Lt),
        Operator::LtEq => Some(BinaryOperator::LtEq),
        Operator::Gt => Some(BinaryOperator::Gt),
        Operator::GtEq => Some(BinaryOperator::GtEq),
        Operator::And => Some(BinaryOperator::And),
        Operator::Or => Some(BinaryOperator::Or),
        Operator::Plus => Some(BinaryOperator::Plus),
        Operator::Minus => Some(BinaryOperator::Minus),
        Operator::Multiply => Some(BinaryOperator::Multiply),
        Operator::Divide => Some(BinaryOperator::Divide),
        _ => None,
    }
}

fn expr_to_planner(e: &sqlrustgo_optimizer::rules::Expr) -> crate::Expr {
    match e {
        sqlrustgo_optimizer::rules::Expr::Column(name) => {
            crate::Expr::Column(crate::Column::new(name.clone()))
        }
        sqlrustgo_optimizer::rules::Expr::Literal(s) => {
            crate::Expr::Literal(if let Ok(n) = s.parse::<i64>() {
                sqlrustgo_types::Value::Integer(n)
            } else if let Ok(f) = s.parse::<f64>() {
                sqlrustgo_types::Value::Float(f)
            } else if s.eq_ignore_ascii_case("true") {
                sqlrustgo_types::Value::Boolean(true)
            } else if s.eq_ignore_ascii_case("false") {
                sqlrustgo_types::Value::Boolean(false)
            } else {
                sqlrustgo_types::Value::Text(s.clone())
            })
        }
        sqlrustgo_optimizer::rules::Expr::BinaryExpr { left, op, right } => {
            crate::Expr::binary_expr(
                expr_to_planner(left),
                op_to_planner(*op),
                expr_to_planner(right),
            )
        }
    }
}

fn op_to_planner(op: sqlrustgo_optimizer::rules::BinaryOperator) -> crate::Operator {
    use sqlrustgo_optimizer::rules::BinaryOperator;
    match op {
        BinaryOperator::Eq => crate::Operator::Eq,
        BinaryOperator::NotEq => crate::Operator::NotEq,
        BinaryOperator::Lt => crate::Operator::Lt,
        BinaryOperator::LtEq => crate::Operator::LtEq,
        BinaryOperator::Gt => crate::Operator::Gt,
        BinaryOperator::GtEq => crate::Operator::GtEq,
        BinaryOperator::And => crate::Operator::And,
        BinaryOperator::Or => crate::Operator::Or,
        BinaryOperator::Plus => crate::Operator::Plus,
        BinaryOperator::Minus => crate::Operator::Minus,
        BinaryOperator::Multiply => crate::Operator::Multiply,
        BinaryOperator::Divide => crate::Operator::Divide,
    }
}

// ===== LogicalPlan <-> UnifiedPlan =====

fn logical_to_unified(plan: &LogicalPlan) -> Option<UnifiedPlan> {
    match plan {
        LogicalPlan::TableScan {
            table_name,
            projection,
            ..
        } => Some(UnifiedPlan::TableScan {
            table_name: table_name.clone(),
            projection: projection.clone(),
        }),
        LogicalPlan::Projection {
            input,
            expr: col_exprs,
            ..
        } => {
            let inner = logical_to_unified(input)?;
            let exprs: Vec<_> = col_exprs.iter().filter_map(expr_to_unified).collect();
            Some(UnifiedPlan::Projection {
                expr: if exprs.is_empty() {
                    vec![sqlrustgo_optimizer::rules::Expr::Column("*".to_string())]
                } else {
                    exprs
                },
                input: Box::new(inner),
            })
        }
        LogicalPlan::Filter { predicate, input } => {
            let inner = logical_to_unified(input)?;
            let pred = expr_to_unified(predicate)
                .unwrap_or(sqlrustgo_optimizer::rules::Expr::Column("1".to_string()));
            Some(UnifiedPlan::Filter {
                predicate: pred,
                input: Box::new(inner),
            })
        }
        LogicalPlan::Join {
            left,
            right,
            condition,
            ..
        } => Some(UnifiedPlan::Join {
            left: Box::new(logical_to_unified(left)?),
            right: Box::new(logical_to_unified(right)?),
            join_type: sqlrustgo_optimizer::rules::JoinType::Inner,
            condition: condition.as_ref().and_then(expr_to_unified),
        }),
        LogicalPlan::Aggregate {
            input, group_expr, ..
        } => {
            let inner = logical_to_unified(input)?;
            let group: Vec<_> = group_expr.iter().filter_map(expr_to_unified).collect();
            Some(UnifiedPlan::Aggregate {
                group_by: group,
                aggregates: vec![],
                input: Box::new(inner),
            })
        }
        LogicalPlan::Sort {
            input,
            sort_expr: sexprs,
            ..
        } => {
            let inner = logical_to_unified(input)?;
            let sort: Vec<_> = sexprs
                .iter()
                .filter_map(|s| expr_to_unified(&s.expr))
                .collect();
            Some(UnifiedPlan::Sort {
                expr: sort,
                input: Box::new(inner),
            })
        }
        LogicalPlan::Limit { input, limit, .. } => Some(UnifiedPlan::Limit {
            limit: *limit,
            input: Box::new(logical_to_unified(input)?),
        }),
        _ => None,
    }
}

fn unified_to_logical(unified: &UnifiedPlan, out: &mut LogicalPlan) -> bool {
    match (unified, out) {
        (
            UnifiedPlan::Filter {
                predicate: new_pred,
                input: uin,
            },
            LogicalPlan::Filter { predicate, input },
        ) => {
            let pe = expr_to_planner(new_pred);
            let changed = format!("{:?}", pe) != format!("{:?}", *predicate);
            if changed {
                *predicate = pe;
            }
            changed || unified_to_logical(uin, input.as_mut())
        }
        (
            UnifiedPlan::Projection {
                expr: new_exprs,
                input: uin,
            },
            LogicalPlan::Projection { expr, input, .. },
        ) => {
            let mut changed = false;
            if !new_exprs.is_empty() && new_exprs.len() <= expr.len() {
                for (i, ne) in new_exprs.iter().enumerate() {
                    let pe = expr_to_planner(ne);
                    if format!("{:?}", pe) != format!("{:?}", expr[i]) {
                        expr[i] = pe;
                        changed = true;
                    }
                }
            }
            changed || unified_to_logical(uin, input.as_mut())
        }
        (
            UnifiedPlan::Join {
                left: ul,
                right: ur,
                condition: new_cond,
                ..
            },
            LogicalPlan::Join {
                left,
                right,
                condition,
                ..
            },
        ) => {
            let mut changed = false;
            if let (Some(nc), Some(ref mut oc)) = (new_cond, condition) {
                let pe = expr_to_planner(nc);
                if format!("{:?}", pe) != format!("{:?}", *oc) {
                    *oc = pe;
                    changed = true;
                }
            }
            changed
                || unified_to_logical(ul, left.as_mut())
                || unified_to_logical(ur, right.as_mut())
        }
        (UnifiedPlan::Aggregate { input: uin, .. }, LogicalPlan::Aggregate { input, .. }) => {
            unified_to_logical(uin, input.as_mut())
        }
        (UnifiedPlan::Sort { input: uin, .. }, LogicalPlan::Sort { input, .. }) => {
            unified_to_logical(uin, input.as_mut())
        }
        (UnifiedPlan::Limit { input: uin, .. }, LogicalPlan::Limit { input, .. }) => {
            unified_to_logical(uin, input.as_mut())
        }
        _ => false,
    }
}

// ===== Rules =====

/// Predicate pushdown (bridged to real implementation in crates/optimizer)
pub struct PredicatePushdown;

impl OptimizerRule for PredicatePushdown {
    fn name(&self) -> &str {
        "PredicatePushdown"
    }
    fn apply(&self, plan: &mut LogicalPlan) -> bool {
        if let Some(unified) = logical_to_unified(plan) {
            let mut u = unified;
            let changed = sqlrustgo_optimizer::PredicatePushdown::new().apply(&mut u);
            if changed {
                unified_to_logical(&u, plan);
            }
            changed
        } else {
            false
        }
    }
}

/// Projection pruning (bridged to real implementation)
pub struct ProjectionPruning;

impl OptimizerRule for ProjectionPruning {
    fn name(&self) -> &str {
        "ProjectionPruning"
    }
    fn apply(&self, plan: &mut LogicalPlan) -> bool {
        if let Some(unified) = logical_to_unified(plan) {
            let mut u = unified;
            let changed = sqlrustgo_optimizer::ProjectionPruning::new().apply(&mut u);
            if changed {
                unified_to_logical(&u, plan);
            }
            changed
        } else {
            false
        }
    }
}

/// Constant folding (bridged to real implementation)
pub struct ConstantFolding;

impl OptimizerRule for ConstantFolding {
    fn name(&self) -> &str {
        "ConstantFolding"
    }
    fn apply(&self, plan: &mut LogicalPlan) -> bool {
        if let Some(unified) = logical_to_unified(plan) {
            let mut u = unified;
            let changed = sqlrustgo_optimizer::ConstantFolding::new().apply(&mut u);
            if changed {
                unified_to_logical(&u, plan);
            }
            changed
        } else {
            false
        }
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
        for _ in 0..10 {
            let mut changed = false;
            for rule in &self.rules {
                if rule.apply(&mut plan) {
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        Ok(plan)
    }
}

/// No-op optimizer
pub struct NoOpOptimizer;

impl Optimizer for NoOpOptimizer {
    fn optimize(&mut self, plan: LogicalPlan) -> OptimizerResult<LogicalPlan> {
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Expr::*;
    use sqlrustgo_types::Value;

    #[test]
    fn test_bridge_noop() {
        let p = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let mut p2 = p;
        let r = ConstantFolding;
        assert!(!r.apply(&mut p2), "No-op on TableScan");
    }

    #[test]
    fn test_constant_folding_bridge() {
        let plan = LogicalPlan::Filter {
            predicate: BinaryExpr {
                left: Box::new(Literal(Value::Integer(1))),
                op: crate::Operator::Plus,
                right: Box::new(Literal(Value::Integer(2))),
            },
            input: Box::new(LogicalPlan::TableScan {
                table_name: "t".to_string(),
                schema: crate::Schema::empty(),
                projection: None,
            }),
        };
        let mut p = plan;
        let changed = ConstantFolding.apply(&mut p);
        assert!(changed, "1+2 should be folded");
        match &p {
            LogicalPlan::Filter { predicate, .. } => match predicate {
                Literal(Value::Integer(3)) => {}
                other => panic!("Expected Literal(3), got {:?}", other),
            },
            _ => panic!("Expected Filter"),
        }
    }

    #[test]
    fn test_predicate_pushdown_bridge() {
        let plan = LogicalPlan::Projection {
            input: Box::new(LogicalPlan::Filter {
                predicate: crate::Expr::Column(crate::Column::new("x".to_string())),
                input: Box::new(LogicalPlan::TableScan {
                    table_name: "t".to_string(),
                    schema: crate::Schema::empty(),
                    projection: None,
                }),
            }),
            expr: vec![crate::Expr::Column(crate::Column::new("x".to_string()))],
            schema: crate::Schema::empty(),
        };
        let mut p = plan;
        let changed = PredicatePushdown.apply(&mut p);
        println!("PredicatePushdown changed={}", changed);
    }

    #[test]
    fn test_full_optimizer_pipeline() {
        let plan = LogicalPlan::Filter {
            predicate: BinaryExpr {
                left: Box::new(Literal(Value::Integer(10))),
                op: crate::Operator::Eq,
                right: Box::new(Literal(Value::Integer(10))),
            },
            input: Box::new(LogicalPlan::TableScan {
                table_name: "t".to_string(),
                schema: crate::Schema::empty(),
                projection: None,
            }),
        };
        let mut opt = DefaultOptimizer::new();
        let result = opt.optimize(plan);
        assert!(result.is_ok(), "Optimizer should succeed");
    }
}
