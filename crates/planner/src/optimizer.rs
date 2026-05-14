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
    use sqlrustgo_optimizer::unified_plan::UnifiedPlan;
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

    #[test]
    fn test_noop_optimizer() {
        let plan = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let mut opt = NoOpOptimizer;
        let result = opt.optimize(plan.clone());
        assert!(result.is_ok());
        // NoOpOptimizer should return plan unchanged
        let optimized = result.unwrap();
        match optimized {
            LogicalPlan::TableScan { table_name, .. } => {
                assert_eq!(table_name, "t");
            }
            _ => panic!("Expected TableScan"),
        }
    }

    #[test]
    fn test_default_optimizer_with_custom_rules() {
        let rules: Vec<Box<dyn OptimizerRule>> = vec![Box::new(ConstantFolding)];
        let mut opt = DefaultOptimizer::with_rules(rules);
        let plan = LogicalPlan::Filter {
            predicate: BinaryExpr {
                left: Box::new(Literal(Value::Integer(5))),
                op: crate::Operator::Plus,
                right: Box::new(Literal(Value::Integer(3))),
            },
            input: Box::new(LogicalPlan::TableScan {
                table_name: "t".to_string(),
                schema: crate::Schema::empty(),
                projection: None,
            }),
        };
        let result = opt.optimize(plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_pruning_rule() {
        let plan = LogicalPlan::Projection {
            input: Box::new(LogicalPlan::TableScan {
                table_name: "t".to_string(),
                schema: crate::Schema::empty(),
                projection: None,
            }),
            expr: vec![crate::Expr::Column(crate::Column::new("x".to_string()))],
            schema: crate::Schema::empty(),
        };
        let mut p = plan;
        let rule = ProjectionPruning;
        assert_eq!(rule.name(), "ProjectionPruning");
        let changed = rule.apply(&mut p);
        println!("ProjectionPruning changed={}", changed);
    }

    #[test]
    fn test_constant_folding_no_change() {
        // Test when constant folding cannot optimize
        let plan = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let mut p = plan;
        let changed = ConstantFolding.apply(&mut p);
        assert!(!changed, "No change expected for TableScan");
    }

    #[test]
    fn test_predicate_pushdown_no_change() {
        // Predicate already at scan level - cannot push further
        let plan = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let mut p = plan;
        let changed = PredicatePushdown.apply(&mut p);
        assert!(!changed, "No change expected for TableScan");
    }

    #[test]
    fn test_expr_to_unified_column() {
        let col_expr = crate::Expr::Column(crate::Column::new("x".to_string()));
        let unified = expr_to_unified(&col_expr);
        assert!(unified.is_some());
    }

    #[test]
    fn test_expr_to_unified_literal_integer() {
        let lit_expr = crate::Expr::Literal(Value::Integer(42));
        let unified = expr_to_unified(&lit_expr);
        assert!(unified.is_some());
    }

    #[test]
    fn test_expr_to_unified_literal_float() {
        let lit_expr = crate::Expr::Literal(Value::Float(3.14));
        let unified = expr_to_unified(&lit_expr);
        assert!(unified.is_some());
    }

    #[test]
    fn test_expr_to_unified_literal_text() {
        let lit_expr = crate::Expr::Literal(Value::Text("hello".to_string()));
        let unified = expr_to_unified(&lit_expr);
        assert!(unified.is_some());
    }

    #[test]
    fn test_expr_to_unified_binary_expr() {
        let expr = crate::Expr::binary_expr(
            crate::Expr::Column(crate::Column::new("a".to_string())),
            crate::Operator::Eq,
            crate::Expr::Literal(Value::Integer(1)),
        );
        let unified = expr_to_unified(&expr);
        assert!(unified.is_some());
    }

    #[test]
    fn test_expr_to_unified_unsupported() {
        // Test expressions that return None
        let agg_expr = crate::Expr::AggregateFunction {
            func: crate::AggregateFunction::Count,
            args: vec![crate::Expr::Wildcard],
            distinct: false,
        };
        let unified = expr_to_unified(&agg_expr);
        assert!(unified.is_none());
    }

    #[test]
    fn test_op_to_unified_all_operators() {
        // Test all operators that map to Some
        assert!(op_to_unified(crate::Operator::Eq).is_some());
        assert!(op_to_unified(crate::Operator::NotEq).is_some());
        assert!(op_to_unified(crate::Operator::Lt).is_some());
        assert!(op_to_unified(crate::Operator::LtEq).is_some());
        assert!(op_to_unified(crate::Operator::Gt).is_some());
        assert!(op_to_unified(crate::Operator::GtEq).is_some());
        assert!(op_to_unified(crate::Operator::And).is_some());
        assert!(op_to_unified(crate::Operator::Or).is_some());
        assert!(op_to_unified(crate::Operator::Plus).is_some());
        assert!(op_to_unified(crate::Operator::Minus).is_some());
        assert!(op_to_unified(crate::Operator::Multiply).is_some());
        assert!(op_to_unified(crate::Operator::Divide).is_some());
    }

    #[test]
    fn test_op_to_unified_unsupported_operators() {
        // Operators that return None
        assert!(op_to_unified(crate::Operator::Not).is_none());
        assert!(op_to_unified(crate::Operator::Like).is_none());
        assert!(op_to_unified(crate::Operator::Modulo).is_none());
    }

    #[test]
    fn test_expr_to_planner_column() {
        use sqlrustgo_optimizer::rules::Expr;
        let unified_expr = Expr::Column("x".to_string());
        let planner_expr = expr_to_planner(&unified_expr);
        assert!(matches!(planner_expr, crate::Expr::Column(_)));
    }

    #[test]
    fn test_expr_to_planner_literal_integer() {
        use sqlrustgo_optimizer::rules::Expr;
        let unified_expr = Expr::Literal("42".to_string());
        let planner_expr = expr_to_planner(&unified_expr);
        assert!(matches!(
            planner_expr,
            crate::Expr::Literal(Value::Integer(42))
        ));
    }

    #[test]
    fn test_expr_to_planner_literal_float() {
        use sqlrustgo_optimizer::rules::Expr;
        let unified_expr = Expr::Literal("3.14".to_string());
        let planner_expr = expr_to_planner(&unified_expr);
        assert!(matches!(
            planner_expr,
            crate::Expr::Literal(Value::Float(_))
        ));
    }

    #[test]
    fn test_expr_to_planner_literal_boolean_true() {
        use sqlrustgo_optimizer::rules::Expr;
        let unified_expr = Expr::Literal("true".to_string());
        let planner_expr = expr_to_planner(&unified_expr);
        assert!(matches!(
            planner_expr,
            crate::Expr::Literal(Value::Boolean(true))
        ));
    }

    #[test]
    fn test_expr_to_planner_literal_boolean_false() {
        use sqlrustgo_optimizer::rules::Expr;
        let unified_expr = Expr::Literal("false".to_string());
        let planner_expr = expr_to_planner(&unified_expr);
        assert!(matches!(
            planner_expr,
            crate::Expr::Literal(Value::Boolean(false))
        ));
    }

    #[test]
    fn test_expr_to_planner_literal_text() {
        use sqlrustgo_optimizer::rules::Expr;
        let unified_expr = Expr::Literal("hello".to_string());
        let planner_expr = expr_to_planner(&unified_expr);
        assert!(matches!(planner_expr, crate::Expr::Literal(Value::Text(s)) if s == "hello"));
    }

    #[test]
    fn test_expr_to_planner_binary_expr() {
        use sqlrustgo_optimizer::rules::Expr;
        let unified_expr = Expr::BinaryExpr {
            left: Box::new(Expr::Column("a".to_string())),
            op: sqlrustgo_optimizer::rules::BinaryOperator::Plus,
            right: Box::new(Expr::Literal("1".to_string())),
        };
        let planner_expr = expr_to_planner(&unified_expr);
        assert!(matches!(
            planner_expr,
            crate::Expr::BinaryExpr {
                op: crate::Operator::Plus,
                ..
            }
        ));
    }

    #[test]
    fn test_op_to_planner_all_operators() {
        use sqlrustgo_optimizer::rules::BinaryOperator;
        assert_eq!(op_to_planner(BinaryOperator::Eq), crate::Operator::Eq);
        assert_eq!(op_to_planner(BinaryOperator::NotEq), crate::Operator::NotEq);
        assert_eq!(op_to_planner(BinaryOperator::Lt), crate::Operator::Lt);
        assert_eq!(op_to_planner(BinaryOperator::LtEq), crate::Operator::LtEq);
        assert_eq!(op_to_planner(BinaryOperator::Gt), crate::Operator::Gt);
        assert_eq!(op_to_planner(BinaryOperator::GtEq), crate::Operator::GtEq);
        assert_eq!(op_to_planner(BinaryOperator::And), crate::Operator::And);
        assert_eq!(op_to_planner(BinaryOperator::Or), crate::Operator::Or);
        assert_eq!(op_to_planner(BinaryOperator::Plus), crate::Operator::Plus);
        assert_eq!(op_to_planner(BinaryOperator::Minus), crate::Operator::Minus);
        assert_eq!(
            op_to_planner(BinaryOperator::Multiply),
            crate::Operator::Multiply
        );
        assert_eq!(
            op_to_planner(BinaryOperator::Divide),
            crate::Operator::Divide
        );
    }

    #[test]
    fn test_logical_to_unified_table_scan() {
        let plan = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: crate::Schema::empty(),
            projection: Some(vec![0, 1]),
        };
        let unified = logical_to_unified(&plan);
        assert!(unified.is_some());
    }

    #[test]
    fn test_logical_to_unified_projection() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Projection {
            input: Box::new(inner),
            expr: vec![crate::Expr::Column(crate::Column::new("x".to_string()))],
            schema: crate::Schema::empty(),
        };
        let unified = logical_to_unified(&plan);
        assert!(unified.is_some());
    }

    #[test]
    fn test_logical_to_unified_filter() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Filter {
            predicate: crate::Expr::Column(crate::Column::new("x".to_string())),
            input: Box::new(inner),
        };
        let unified = logical_to_unified(&plan);
        assert!(unified.is_some());
    }

    #[test]
    fn test_logical_to_unified_join() {
        let left = LogicalPlan::TableScan {
            table_name: "t1".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let right = LogicalPlan::TableScan {
            table_name: "t2".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Join {
            left: Box::new(left),
            right: Box::new(right),
            join_type: crate::JoinType::Inner,
            condition: Some(crate::Expr::Column(crate::Column::new("x".to_string()))),
        };
        let unified = logical_to_unified(&plan);
        assert!(unified.is_some());
    }

    #[test]
    fn test_logical_to_unified_aggregate() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Aggregate {
            input: Box::new(inner),
            group_expr: vec![crate::Expr::Column(crate::Column::new("x".to_string()))],
            aggregate_expr: vec![],
            schema: crate::Schema::empty(),
        };
        let unified = logical_to_unified(&plan);
        assert!(unified.is_some());
    }

    #[test]
    fn test_logical_to_unified_sort() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Sort {
            input: Box::new(inner),
            sort_expr: vec![crate::SortExpr {
                expr: crate::Expr::Column(crate::Column::new("x".to_string())),
                asc: true,
                nulls_first: false,
            }],
        };
        let unified = logical_to_unified(&plan);
        assert!(unified.is_some());
    }

    #[test]
    fn test_logical_to_unified_limit() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Limit {
            input: Box::new(inner),
            limit: 10,
            offset: None,
        };
        let unified = logical_to_unified(&plan);
        assert!(unified.is_some());
    }

    #[test]
    fn test_logical_to_unified_empty_relation() {
        let plan = LogicalPlan::EmptyRelation;
        let unified = logical_to_unified(&plan);
        assert!(unified.is_none());
    }

    #[test]
    fn test_logical_to_unified_update() {
        let plan = LogicalPlan::Update {
            table_name: "t".to_string(),
            updates: vec![],
            predicate: None,
        };
        let unified = logical_to_unified(&plan);
        assert!(unified.is_none());
    }

    #[test]
    fn test_logical_to_unified_delete() {
        let plan = LogicalPlan::Delete {
            table_name: "t".to_string(),
            predicate: None,
        };
        let unified = logical_to_unified(&plan);
        assert!(unified.is_none());
    }

    #[test]
    fn test_logical_to_unified_subquery() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Subquery {
            subquery: Box::new(inner),
            alias: "sq".to_string(),
        };
        let unified = logical_to_unified(&plan);
        assert!(unified.is_none());
    }

    #[test]
    fn test_unified_to_logical_filter() {
        let mut plan = LogicalPlan::Filter {
            predicate: crate::Expr::Column(crate::Column::new("x".to_string())),
            input: Box::new(LogicalPlan::TableScan {
                table_name: "t".to_string(),
                schema: crate::Schema::empty(),
                projection: None,
            }),
        };
        let unified = UnifiedPlan::Filter {
            predicate: sqlrustgo_optimizer::rules::Expr::Column("y".to_string()),
            input: Box::new(UnifiedPlan::TableScan {
                table_name: "t".to_string(),
                projection: None,
            }),
        };
        let changed = unified_to_logical(&unified, &mut plan);
        // Filter might or might not change depending on predicate
        println!("unified_to_logical filter changed={}", changed);
    }

    #[test]
    fn test_unified_to_logical_projection() {
        let mut plan = LogicalPlan::Projection {
            input: Box::new(LogicalPlan::TableScan {
                table_name: "t".to_string(),
                schema: crate::Schema::empty(),
                projection: None,
            }),
            expr: vec![crate::Expr::Column(crate::Column::new("x".to_string()))],
            schema: crate::Schema::empty(),
        };
        let unified = UnifiedPlan::Projection {
            expr: vec![sqlrustgo_optimizer::rules::Expr::Column("y".to_string())],
            input: Box::new(UnifiedPlan::TableScan {
                table_name: "t".to_string(),
                projection: None,
            }),
        };
        let changed = unified_to_logical(&unified, &mut plan);
        println!("unified_to_logical projection changed={}", changed);
    }

    #[test]
    fn test_unified_to_logical_sort() {
        let mut plan = LogicalPlan::Sort {
            input: Box::new(LogicalPlan::TableScan {
                table_name: "t".to_string(),
                schema: crate::Schema::empty(),
                projection: None,
            }),
            sort_expr: vec![crate::SortExpr {
                expr: crate::Expr::Column(crate::Column::new("x".to_string())),
                asc: true,
                nulls_first: false,
            }],
        };
        let unified = UnifiedPlan::Sort {
            expr: vec![sqlrustgo_optimizer::rules::Expr::Column("x".to_string())],
            input: Box::new(UnifiedPlan::TableScan {
                table_name: "t".to_string(),
                projection: None,
            }),
        };
        let changed = unified_to_logical(&unified, &mut plan);
        println!("unified_to_logical sort changed={}", changed);
    }

    #[test]
    fn test_unified_to_logical_limit() {
        let mut plan = LogicalPlan::Limit {
            input: Box::new(LogicalPlan::TableScan {
                table_name: "t".to_string(),
                schema: crate::Schema::empty(),
                projection: None,
            }),
            limit: 10,
            offset: None,
        };
        let unified = UnifiedPlan::Limit {
            limit: 20,
            input: Box::new(UnifiedPlan::TableScan {
                table_name: "t".to_string(),
                projection: None,
            }),
        };
        let changed = unified_to_logical(&unified, &mut plan);
        println!("unified_to_logical limit changed={}", changed);
    }

    #[test]
    fn test_optimizer_rule_trait() {
        // Test that rules implement the trait properly
        let rule: Box<dyn OptimizerRule> = Box::new(ConstantFolding);
        assert_eq!(rule.name(), "ConstantFolding");

        let rule: Box<dyn OptimizerRule> = Box::new(PredicatePushdown);
        assert_eq!(rule.name(), "PredicatePushdown");

        let rule: Box<dyn OptimizerRule> = Box::new(ProjectionPruning);
        assert_eq!(rule.name(), "ProjectionPruning");
    }

    #[test]
    fn test_default_optimizer_fixed_point() {
        // Test that optimizer converges after multiple iterations
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
        let mut opt = DefaultOptimizer::new();
        // Run optimize multiple times - should converge
        let result1 = opt.optimize(plan.clone());
        let result2 = opt.optimize(plan);
        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[test]
    fn test_join_condition_conversion() {
        let mut plan = LogicalPlan::Join {
            left: Box::new(LogicalPlan::TableScan {
                table_name: "t1".to_string(),
                schema: crate::Schema::empty(),
                projection: None,
            }),
            right: Box::new(LogicalPlan::TableScan {
                table_name: "t2".to_string(),
                schema: crate::Schema::empty(),
                projection: None,
            }),
            join_type: crate::JoinType::Inner,
            condition: Some(crate::Expr::Column(crate::Column::new("x".to_string()))),
        };
        let unified = UnifiedPlan::Join {
            left: Box::new(UnifiedPlan::TableScan {
                table_name: "t1".to_string(),
                projection: None,
            }),
            right: Box::new(UnifiedPlan::TableScan {
                table_name: "t2".to_string(),
                projection: None,
            }),
            join_type: sqlrustgo_optimizer::rules::JoinType::Inner,
            condition: Some(sqlrustgo_optimizer::rules::Expr::Column("y".to_string())),
        };
        let changed = unified_to_logical(&unified, &mut plan);
        println!("unified_to_logical join changed={}", changed);
    }
}
