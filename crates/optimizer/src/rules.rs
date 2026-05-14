//! Optimizer Rules Module

use crate::Rule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
    LeftSemi,
    LeftAnti,
    RightSemi,
    RightAnti,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Column(String),
    Literal(String),
    BinaryExpr {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
    Plus,
    Minus,
    Multiply,
    Divide,
}

impl Expr {
    pub fn fold_constants(&self) -> Expr {
        match self {
            Expr::BinaryExpr { left, op, right } => {
                let left_folded = left.fold_constants();
                let right_folded = right.fold_constants();
                if let (Expr::Literal(l), Expr::Literal(r)) = (&left_folded, &right_folded) {
                    if let Some(val) = eval_binary_op(l, r, *op) {
                        return Expr::Literal(val);
                    }
                }
                if *op == BinaryOperator::And {
                    if let Expr::Literal(l) = &left_folded {
                        if l == "true" || l == "1" {
                            return right_folded;
                        }
                        if l == "false" || l == "0" {
                            return Expr::Literal("false".to_string());
                        }
                    }
                    if let Expr::Literal(r) = &right_folded {
                        if r == "true" || r == "1" {
                            return left_folded;
                        }
                        if r == "false" || r == "0" {
                            return Expr::Literal("false".to_string());
                        }
                    }
                }
                if *op == BinaryOperator::Or {
                    if let Expr::Literal(l) = &left_folded {
                        if l == "false" || l == "0" {
                            return right_folded;
                        }
                        if l == "true" || l == "1" {
                            return Expr::Literal("true".to_string());
                        }
                    }
                    if let Expr::Literal(r) = &right_folded {
                        if r == "false" || r == "0" {
                            return left_folded;
                        }
                        if r == "true" || r == "1" {
                            return Expr::Literal("true".to_string());
                        }
                    }
                }
                if left_folded == *left.as_ref() && right_folded == *right.as_ref() {
                    self.clone()
                } else {
                    Expr::BinaryExpr {
                        left: Box::new(left_folded),
                        op: *op,
                        right: Box::new(right_folded),
                    }
                }
            }
            _ => self.clone(),
        }
    }

    pub fn references_columns(&self) -> Vec<&str> {
        match self {
            Expr::Column(name) => vec![name.as_str()],
            Expr::Literal(_) => vec![],
            Expr::BinaryExpr { left, right, .. } => {
                let mut cols = left.references_columns();
                cols.extend(right.references_columns());
                cols
            }
        }
    }
}

fn eval_binary_op(l: &str, r: &str, op: BinaryOperator) -> Option<String> {
    let lhs = l.parse::<f64>().ok()?;
    let rhs = r.parse::<f64>().ok()?;
    match op {
        BinaryOperator::Plus => Some((lhs + rhs).to_string()),
        BinaryOperator::Minus => Some((lhs - rhs).to_string()),
        BinaryOperator::Multiply => Some((lhs * rhs).to_string()),
        BinaryOperator::Divide => {
            if rhs != 0.0 {
                Some((lhs / rhs).to_string())
            } else {
                None
            }
        }
        BinaryOperator::Eq => Some((lhs == rhs).to_string()),
        BinaryOperator::NotEq => Some((lhs != rhs).to_string()),
        BinaryOperator::Lt => Some((lhs < rhs).to_string()),
        BinaryOperator::LtEq => Some((lhs <= rhs).to_string()),
        BinaryOperator::Gt => Some((lhs > rhs).to_string()),
        BinaryOperator::GtEq => Some((lhs >= rhs).to_string()),
        BinaryOperator::And => {
            let lv = !(l == "0" || l == "false" || l.is_empty());
            let rv = !(r == "0" || r == "false" || r.is_empty());
            Some((lv && rv).to_string())
        }
        BinaryOperator::Or => {
            let lv = !(l == "0" || l == "false" || l.is_empty());
            let rv = !(r == "0" || r == "false" || r.is_empty());
            Some((lv || rv).to_string())
        }
    }
}

use crate::unified_plan::UnifiedPlan;

impl UnifiedPlan {
    pub fn predicate_pushdown(&mut self) -> bool {
        match self {
            UnifiedPlan::Filter { predicate, input } => {
                if input.predicate_pushdown() {
                    return true;
                }
                let new_pred = predicate.fold_constants();
                if new_pred != *predicate {
                    *predicate = new_pred;
                    return true;
                }
                false
            }
            UnifiedPlan::Projection { expr, input } => {
                if input.predicate_pushdown() {
                    return true;
                }
                let mut new_expr = Vec::new();
                let mut changed = false;
                for e in expr.iter() {
                    let folded = e.fold_constants();
                    if folded != *e {
                        changed = true;
                    }
                    new_expr.push(folded);
                }
                if changed {
                    *expr = new_expr;
                }
                changed
            }
            UnifiedPlan::Join {
                left,
                right,
                condition,
                ..
            } => {
                let mut changed = false;
                if let Some(cond) = condition {
                    let folded = cond.fold_constants();
                    if folded != *cond {
                        *condition = Some(folded);
                        changed = true;
                    }
                }
                if left.predicate_pushdown() {
                    changed = true;
                }
                if right.predicate_pushdown() {
                    changed = true;
                }
                changed
            }
            UnifiedPlan::Aggregate {
                group_by,
                aggregates,
                input,
            } => {
                let mut new_group = Vec::new();
                let mut changed = false;
                for g in group_by.iter() {
                    let folded = g.fold_constants();
                    if folded != *g {
                        changed = true;
                    }
                    new_group.push(folded);
                }
                if changed {
                    *group_by = new_group;
                }
                let mut new_agg = Vec::new();
                let mut changed_agg = false;
                for a in aggregates.iter() {
                    let folded = a.fold_constants();
                    if folded != *a {
                        changed_agg = true;
                    }
                    new_agg.push(folded);
                }
                if changed_agg {
                    *aggregates = new_agg;
                }
                changed || changed_agg || input.predicate_pushdown()
            }
            UnifiedPlan::Sort { expr, input } => {
                if input.predicate_pushdown() {
                    return true;
                }
                let mut new_expr = Vec::new();
                let mut changed = false;
                for e in expr.iter() {
                    let folded = e.fold_constants();
                    if folded != *e {
                        changed = true;
                    }
                    new_expr.push(folded);
                }
                if changed {
                    *expr = new_expr;
                }
                changed
            }
            UnifiedPlan::IndexScan { predicate, .. } => {
                if let Some(pred) = predicate {
                    let folded = pred.fold_constants();
                    if folded != *pred {
                        *predicate = Some(folded);
                        return true;
                    }
                }
                false
            }
            UnifiedPlan::HybridVectorScan { sql_filter, .. } => {
                if let Some(filter) = sql_filter {
                    let folded = filter.fold_constants();
                    if folded != *filter {
                        *sql_filter = Some(folded);
                        return true;
                    }
                }
                false
            }
            UnifiedPlan::HybridGraphScan { sql_filter, .. } => {
                if let Some(filter) = sql_filter {
                    let folded = filter.fold_constants();
                    if folded != *filter {
                        *sql_filter = Some(folded);
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub fn prune_projections(&mut self) -> bool {
        match self {
            UnifiedPlan::Projection { expr, input } => {
                let used_cols: Vec<_> = expr.iter().flat_map(|e| e.references_columns()).collect();
                if used_cols.is_empty() {
                    return false;
                }
                if let UnifiedPlan::TableScan {
                    projection: Some(proj),
                    ..
                } = input.as_mut()
                {
                    let needed: Vec<usize> = used_cols
                        .iter()
                        .filter_map(|c| c.parse::<usize>().ok())
                        .collect();
                    if !needed.is_empty() && needed.len() < proj.len() {
                        *proj = needed;
                        return true;
                    }
                }
                input.prune_projections()
            }
            UnifiedPlan::Join { left, right, .. } => {
                let l = left.prune_projections();
                let r = right.prune_projections();
                l || r
            }
            UnifiedPlan::Filter { predicate, input } => {
                let new_pred = predicate.fold_constants();
                let changed = new_pred != *predicate;
                if changed {
                    *predicate = new_pred;
                }
                changed || input.prune_projections()
            }
            UnifiedPlan::Aggregate {
                group_by,
                aggregates,
                input,
            } => {
                let mut new_group = Vec::new();
                let mut changed = false;
                for g in group_by.iter() {
                    let folded = g.fold_constants();
                    if folded != *g {
                        changed = true;
                    }
                    new_group.push(folded);
                }
                if changed {
                    *group_by = new_group;
                }
                let mut new_agg = Vec::new();
                let mut changed_agg = false;
                for a in aggregates.iter() {
                    let folded = a.fold_constants();
                    if folded != *a {
                        changed_agg = true;
                    }
                    new_agg.push(folded);
                }
                if changed_agg {
                    *aggregates = new_agg;
                }
                changed || changed_agg || input.prune_projections()
            }
            _ => false,
        }
    }
}

/// PredicatePushdown rule - pushes filter conditions down to the source
pub struct PredicatePushdown;

impl PredicatePushdown {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PredicatePushdown {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule<UnifiedPlan> for PredicatePushdown {
    fn name(&self) -> &str {
        "PredicatePushdown"
    }

    fn apply(&self, plan: &mut UnifiedPlan) -> bool {
        plan.predicate_pushdown()
    }
}

/// ProjectionPruning rule - removes unnecessary columns
pub struct ProjectionPruning;

impl ProjectionPruning {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProjectionPruning {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule<UnifiedPlan> for ProjectionPruning {
    fn name(&self) -> &str {
        "ProjectionPruning"
    }

    fn apply(&self, plan: &mut UnifiedPlan) -> bool {
        plan.prune_projections()
    }
}

/// ConstantFolding rule - evaluates constant expressions at compile time
pub struct ConstantFolding;

impl ConstantFolding {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConstantFolding {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule<UnifiedPlan> for ConstantFolding {
    fn name(&self) -> &str {
        "ConstantFolding"
    }

    fn apply(&self, plan: &mut UnifiedPlan) -> bool {
        plan.fold_constants()
    }
}

impl UnifiedPlan {
    fn fold_constants(&mut self) -> bool {
        match self {
            UnifiedPlan::Filter { predicate, input } => {
                let new_pred = predicate.fold_constants();
                let changed = new_pred != *predicate;
                if changed {
                    *predicate = new_pred;
                }
                changed || input.fold_constants()
            }
            UnifiedPlan::Projection { expr, input } => {
                let mut new_expr = Vec::new();
                let mut changed = false;
                for e in expr.iter() {
                    let folded = e.fold_constants();
                    if folded != *e {
                        changed = true;
                    }
                    new_expr.push(folded);
                }
                if changed {
                    *expr = new_expr;
                }
                changed || input.fold_constants()
            }
            UnifiedPlan::Join {
                left,
                right,
                condition,
                ..
            } => {
                let mut changed = false;
                if let Some(cond) = condition {
                    let folded = cond.fold_constants();
                    if folded != *cond {
                        *condition = Some(folded);
                        changed = true;
                    }
                }
                changed || left.fold_constants() || right.fold_constants()
            }
            UnifiedPlan::Aggregate {
                group_by,
                aggregates,
                input,
            } => {
                let mut new_group = Vec::new();
                let mut changed = false;
                for g in group_by.iter() {
                    let folded = g.fold_constants();
                    if folded != *g {
                        changed = true;
                    }
                    new_group.push(folded);
                }
                if changed {
                    *group_by = new_group;
                }
                let mut new_agg = Vec::new();
                let mut changed_agg = false;
                for a in aggregates.iter() {
                    let folded = a.fold_constants();
                    if folded != *a {
                        changed_agg = true;
                    }
                    new_agg.push(folded);
                }
                if changed_agg {
                    *aggregates = new_agg;
                }
                (changed || changed_agg) || input.fold_constants()
            }
            UnifiedPlan::Sort { expr, input } => {
                let mut new_expr = Vec::new();
                let mut changed = false;
                for e in expr.iter() {
                    let folded = e.fold_constants();
                    if folded != *e {
                        changed = true;
                    }
                    new_expr.push(folded);
                }
                if changed {
                    *expr = new_expr;
                }
                changed || input.fold_constants()
            }
            UnifiedPlan::IndexScan { predicate, .. } => {
                if let Some(pred) = predicate {
                    let folded = pred.fold_constants();
                    if folded != *pred {
                        *predicate = Some(folded);
                        return true;
                    }
                }
                false
            }
            UnifiedPlan::HybridVectorScan { sql_filter, .. } => {
                if let Some(filter) = sql_filter {
                    let folded = filter.fold_constants();
                    if folded != *filter {
                        *sql_filter = Some(folded);
                        return true;
                    }
                }
                false
            }
            UnifiedPlan::HybridGraphScan { sql_filter, .. } => {
                if let Some(filter) = sql_filter {
                    let folded = filter.fold_constants();
                    if folded != *filter {
                        *sql_filter = Some(folded);
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_folding_binary_arithmetic() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Literal("1".to_string())),
            op: BinaryOperator::Plus,
            right: Box::new(Expr::Literal("2".to_string())),
        };
        let folded = expr.fold_constants();
        assert_eq!(folded, Expr::Literal("3".to_string()));
    }

    #[test]
    fn test_constant_folding_binary_comparison() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Literal("5".to_string())),
            op: BinaryOperator::Gt,
            right: Box::new(Expr::Literal("3".to_string())),
        };
        let folded = expr.fold_constants();
        assert_eq!(folded, Expr::Literal("true".to_string()));
    }

    #[test]
    fn test_constant_folding_and_simplify() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Literal("true".to_string())),
            op: BinaryOperator::And,
            right: Box::new(Expr::Column("x".to_string())),
        };
        let folded = expr.fold_constants();
        assert_eq!(folded, Expr::Column("x".to_string()));
    }

    #[test]
    fn test_constant_folding_or_simplify() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Literal("false".to_string())),
            op: BinaryOperator::Or,
            right: Box::new(Expr::Column("y".to_string())),
        };
        let folded = expr.fold_constants();
        assert_eq!(folded, Expr::Column("y".to_string()));
    }

    #[test]
    fn test_constant_folding_with_column() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Column("a".to_string())),
            op: BinaryOperator::Plus,
            right: Box::new(Expr::Literal("1".to_string())),
        };
        let folded = expr.fold_constants();
        assert!(matches!(folded, Expr::BinaryExpr { .. }));
    }

    #[test]
    fn test_expr_references_columns() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Column("a".to_string())),
            op: BinaryOperator::Plus,
            right: Box::new(Expr::Column("b".to_string())),
        };
        let cols = expr.references_columns();
        assert!(cols.contains(&"a"));
        assert!(cols.contains(&"b"));
    }

    #[test]
    fn test_constant_folding_division_by_zero() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Literal("1".to_string())),
            op: BinaryOperator::Divide,
            right: Box::new(Expr::Literal("0".to_string())),
        };
        let folded = expr.fold_constants();
        assert!(matches!(folded, Expr::BinaryExpr { .. }));
    }

    #[test]
    fn test_constant_folding_nested() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::BinaryExpr {
                left: Box::new(Expr::Literal("1".to_string())),
                op: BinaryOperator::Plus,
                right: Box::new(Expr::Literal("2".to_string())),
            }),
            op: BinaryOperator::Multiply,
            right: Box::new(Expr::Literal("3".to_string())),
        };
        let folded = expr.fold_constants();
        assert_eq!(folded, Expr::Literal("9".to_string()));
    }

    #[test]
    fn test_predicate_pushdown_name() {
        let rule = PredicatePushdown::new();
        assert_eq!(Rule::<UnifiedPlan>::name(&rule), "PredicatePushdown");
    }

    #[test]
    fn test_projection_pruning_name() {
        let rule = ProjectionPruning::new();
        assert_eq!(Rule::<UnifiedPlan>::name(&rule), "ProjectionPruning");
    }

    #[test]
    fn test_constant_folding_name() {
        let rule = ConstantFolding::new();
        assert_eq!(Rule::<UnifiedPlan>::name(&rule), "ConstantFolding");
    }

    #[test]
    fn test_constant_folding_on_filter() {
        let mut plan = UnifiedPlan::Filter {
            predicate: Expr::BinaryExpr {
                left: Box::new(Expr::Literal("1".to_string())),
                op: BinaryOperator::Plus,
                right: Box::new(Expr::Literal("2".to_string())),
            },
            input: Box::new(UnifiedPlan::TableScan {
                table_name: "t".to_string(),
                projection: None,
            }),
        };
        let rule = ConstantFolding::new();
        let changed = rule.apply(&mut plan);
        assert!(changed);
        if let UnifiedPlan::Filter { predicate, .. } = plan {
            assert_eq!(predicate, Expr::Literal("3".to_string()));
        }
    }

    #[test]
    fn test_predicate_pushdown_no_change_on_table_scan() {
        let mut plan = UnifiedPlan::TableScan {
            table_name: "t".to_string(),
            projection: None,
        };
        let rule = PredicatePushdown::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_projection_pruning_no_change() {
        let mut plan = UnifiedPlan::Projection {
            expr: vec![Expr::Column("a".to_string())],
            input: Box::new(UnifiedPlan::TableScan {
                table_name: "t".to_string(),
                projection: None,
            }),
        };
        let rule = ProjectionPruning::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_eval_binary_op_minus() {
        let result = eval_binary_op("5", "3", BinaryOperator::Minus);
        assert_eq!(result, Some("2".to_string()));
    }

    #[test]
    fn test_eval_binary_op_multiply() {
        let result = eval_binary_op("4", "3", BinaryOperator::Multiply);
        assert_eq!(result, Some("12".to_string()));
    }

    #[test]
    fn test_eval_binary_op_divide() {
        let result = eval_binary_op("10", "2", BinaryOperator::Divide);
        assert_eq!(result, Some("5".to_string()));
    }

    #[test]
    fn test_eval_binary_op_divide_by_zero() {
        let result = eval_binary_op("10", "0", BinaryOperator::Divide);
        assert_eq!(result, None);
    }

    #[test]
    fn test_eval_binary_op_not_eq() {
        let result = eval_binary_op("5", "3", BinaryOperator::NotEq);
        assert_eq!(result, Some("true".to_string()));
        let result2 = eval_binary_op("5", "5", BinaryOperator::NotEq);
        assert_eq!(result2, Some("false".to_string()));
    }

    #[test]
    fn test_eval_binary_op_lt() {
        let result = eval_binary_op("3", "5", BinaryOperator::Lt);
        assert_eq!(result, Some("true".to_string()));
        let result2 = eval_binary_op("5", "3", BinaryOperator::Lt);
        assert_eq!(result2, Some("false".to_string()));
    }

    #[test]
    fn test_eval_binary_op_lt_eq() {
        let result = eval_binary_op("3", "3", BinaryOperator::LtEq);
        assert_eq!(result, Some("true".to_string()));
        let result2 = eval_binary_op("4", "3", BinaryOperator::LtEq);
        assert_eq!(result2, Some("false".to_string()));
    }

    #[test]
    fn test_eval_binary_op_gt_eq() {
        let result = eval_binary_op("5", "3", BinaryOperator::GtEq);
        assert_eq!(result, Some("true".to_string()));
        let result2 = eval_binary_op("3", "5", BinaryOperator::GtEq);
        assert_eq!(result2, Some("false".to_string()));
    }

    #[test]
    fn test_eval_binary_op_and_with_numeric() {
        let result = eval_binary_op("1", "1", BinaryOperator::And);
        assert_eq!(result, Some("true".to_string()));
        let result2 = eval_binary_op("1", "0", BinaryOperator::And);
        assert_eq!(result2, Some("false".to_string()));
    }

    #[test]
    fn test_eval_binary_op_or_with_numeric() {
        let result = eval_binary_op("0", "1", BinaryOperator::Or);
        assert_eq!(result, Some("true".to_string()));
        let result2 = eval_binary_op("0", "0", BinaryOperator::Or);
        assert_eq!(result2, Some("false".to_string()));
    }

    #[test]
    fn test_expr_references_columns_literal() {
        let expr = Expr::Literal("1".to_string());
        let cols = expr.references_columns();
        assert!(cols.is_empty());
    }

    #[test]
    fn test_expr_references_columns_column() {
        let expr = Expr::Column("name".to_string());
        let cols = expr.references_columns();
        assert_eq!(cols, vec!["name"]);
    }

    #[test]
    fn test_expr_references_columns_nested() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Column("a".to_string())),
            op: BinaryOperator::Plus,
            right: Box::new(Expr::BinaryExpr {
                left: Box::new(Expr::Column("b".to_string())),
                op: BinaryOperator::Multiply,
                right: Box::new(Expr::Literal("2".to_string())),
            }),
        };
        let cols = expr.references_columns();
        assert!(cols.contains(&"a"));
        assert!(cols.contains(&"b"));
        assert_eq!(cols.len(), 2);
    }

    #[test]
    fn test_constant_folding_or_simplify_left_true() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Literal("true".to_string())),
            op: BinaryOperator::Or,
            right: Box::new(Expr::Column("x".to_string())),
        };
        let folded = expr.fold_constants();
        assert_eq!(folded, Expr::Literal("true".to_string()));
    }

    #[test]
    fn test_constant_folding_and_simplify_left_false() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Literal("false".to_string())),
            op: BinaryOperator::And,
            right: Box::new(Expr::Column("x".to_string())),
        };
        let folded = expr.fold_constants();
        assert_eq!(folded, Expr::Literal("false".to_string()));
    }

    #[test]
    fn test_predicate_pushdown_on_projection() {
        let mut plan = UnifiedPlan::Projection {
            expr: vec![Expr::Column("a".to_string())],
            input: Box::new(UnifiedPlan::TableScan {
                table_name: "t".to_string(),
                projection: None,
            }),
        };
        let rule = PredicatePushdown::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_projection_pruning_multiple_columns() {
        let mut plan = UnifiedPlan::Projection {
            expr: vec![Expr::Column("a".to_string()), Expr::Column("b".to_string())],
            input: Box::new(UnifiedPlan::TableScan {
                table_name: "t".to_string(),
                projection: Some(vec![0, 1, 2]),
            }),
        };
        let rule = ProjectionPruning::new();
        let _changed = rule.apply(&mut plan);
    }

    // ===== Tests for predicate_pushdown on all UnifiedPlan variants =====

    #[test]
    fn test_predicate_pushdown_on_limit() {
        let mut plan = UnifiedPlan::Limit {
            limit: 100,
            input: Box::new(UnifiedPlan::TableScan {
                table_name: "t".to_string(),
                projection: None,
            }),
        };
        let rule = PredicatePushdown::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_predicate_pushdown_on_vector_scan() {
        use crate::unified_plan::VectorScanType;
        let mut plan = UnifiedPlan::VectorScan {
            vector_index: "vec_idx".to_string(),
            query_vector: vec![1.0, 2.0, 3.0],
            scan_type: VectorScanType::Knn { k: 5 },
            limit: Some(10),
        };
        let rule = PredicatePushdown::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_predicate_pushdown_on_graph_scan() {
        use crate::unified_plan::GraphScanType;
        let mut plan = UnifiedPlan::GraphScan {
            graph_name: "social".to_string(),
            scan_type: GraphScanType::Traversal { max_depth: 3 },
            start_node: Some("user1".to_string()),
        };
        let rule = PredicatePushdown::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_predicate_pushdown_on_empty_relation() {
        let mut plan = UnifiedPlan::EmptyRelation;
        let rule = PredicatePushdown::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_predicate_pushdown_on_hybrid_vector_scan() {
        use crate::unified_plan::VectorScanType;
        let mut plan = UnifiedPlan::HybridVectorScan {
            sql_filter: Some(Expr::BinaryExpr {
                left: Box::new(Expr::Literal("1".to_string())),
                op: BinaryOperator::Plus,
                right: Box::new(Expr::Literal("2".to_string())),
            }),
            vector_index: "vec_idx".to_string(),
            query_vector: vec![1.0, 2.0, 3.0],
            scan_type: VectorScanType::Ann { threshold: 0.8 },
            limit: Some(10),
        };
        let rule = PredicatePushdown::new();
        let changed = rule.apply(&mut plan);
        assert!(changed);
    }

    #[test]
    fn test_predicate_pushdown_on_hybrid_graph_scan() {
        use crate::unified_plan::GraphScanType;
        let mut plan = UnifiedPlan::HybridGraphScan {
            sql_filter: Some(Expr::BinaryExpr {
                left: Box::new(Expr::Literal("3".to_string())),
                op: BinaryOperator::Plus,
                right: Box::new(Expr::Literal("7".to_string())),
            }),
            graph_name: "social".to_string(),
            scan_type: GraphScanType::Reachability {
                target: "user2".to_string(),
            },
            start_node: Some("user1".to_string()),
        };
        let rule = PredicatePushdown::new();
        let changed = rule.apply(&mut plan);
        assert!(changed);
    }

    #[test]
    fn test_predicate_pushdown_on_join() {
        let mut plan = UnifiedPlan::Join {
            left: Box::new(UnifiedPlan::TableScan {
                table_name: "t1".to_string(),
                projection: None,
            }),
            right: Box::new(UnifiedPlan::TableScan {
                table_name: "t2".to_string(),
                projection: None,
            }),
            join_type: JoinType::Inner,
            condition: Some(Expr::BinaryExpr {
                left: Box::new(Expr::Column("a".to_string())),
                op: BinaryOperator::Eq,
                right: Box::new(Expr::Column("b".to_string())),
            }),
        };
        let rule = PredicatePushdown::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_predicate_pushdown_on_aggregate() {
        let mut plan = UnifiedPlan::Aggregate {
            group_by: vec![Expr::Column("dept".to_string())],
            aggregates: vec![Expr::Column("salary".to_string())],
            input: Box::new(UnifiedPlan::TableScan {
                table_name: "employees".to_string(),
                projection: None,
            }),
        };
        let rule = PredicatePushdown::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_predicate_pushdown_on_sort() {
        let mut plan = UnifiedPlan::Sort {
            expr: vec![Expr::Column("name".to_string())],
            input: Box::new(UnifiedPlan::TableScan {
                table_name: "t".to_string(),
                projection: None,
            }),
        };
        let rule = PredicatePushdown::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    // ===== Tests for prune_projections on all UnifiedPlan variants =====

    #[test]
    fn test_prune_projections_on_limit() {
        let mut plan = UnifiedPlan::Limit {
            limit: 100,
            input: Box::new(UnifiedPlan::TableScan {
                table_name: "t".to_string(),
                projection: None,
            }),
        };
        let rule = ProjectionPruning::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_prune_projections_on_vector_scan() {
        use crate::unified_plan::VectorScanType;
        let mut plan = UnifiedPlan::VectorScan {
            vector_index: "vec_idx".to_string(),
            query_vector: vec![1.0, 2.0, 3.0],
            scan_type: VectorScanType::Knn { k: 5 },
            limit: Some(10),
        };
        let rule = ProjectionPruning::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_prune_projections_on_graph_scan() {
        use crate::unified_plan::GraphScanType;
        let mut plan = UnifiedPlan::GraphScan {
            graph_name: "social".to_string(),
            scan_type: GraphScanType::Traversal { max_depth: 3 },
            start_node: Some("user1".to_string()),
        };
        let rule = ProjectionPruning::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_prune_projections_on_empty_relation() {
        let mut plan = UnifiedPlan::EmptyRelation;
        let rule = ProjectionPruning::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_prune_projections_on_hybrid_vector_scan() {
        use crate::unified_plan::VectorScanType;
        let mut plan = UnifiedPlan::HybridVectorScan {
            sql_filter: None,
            vector_index: "vec_idx".to_string(),
            query_vector: vec![1.0, 2.0, 3.0],
            scan_type: VectorScanType::Knn { k: 5 },
            limit: Some(10),
        };
        let rule = ProjectionPruning::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    // ===== Tests for fold_constants on all UnifiedPlan variants =====

    #[test]
    fn test_fold_constants_on_limit() {
        let mut plan = UnifiedPlan::Limit {
            limit: 100,
            input: Box::new(UnifiedPlan::TableScan {
                table_name: "t".to_string(),
                projection: None,
            }),
        };
        let rule = ConstantFolding::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_fold_constants_on_vector_scan() {
        use crate::unified_plan::VectorScanType;
        let mut plan = UnifiedPlan::VectorScan {
            vector_index: "vec_idx".to_string(),
            query_vector: vec![1.0, 2.0, 3.0],
            scan_type: VectorScanType::Knn { k: 5 },
            limit: Some(10),
        };
        let rule = ConstantFolding::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_fold_constants_on_graph_scan() {
        use crate::unified_plan::GraphScanType;
        let mut plan = UnifiedPlan::GraphScan {
            graph_name: "social".to_string(),
            scan_type: GraphScanType::Traversal { max_depth: 3 },
            start_node: Some("user1".to_string()),
        };
        let rule = ConstantFolding::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_fold_constants_on_empty_relation() {
        let mut plan = UnifiedPlan::EmptyRelation;
        let rule = ConstantFolding::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    #[test]
    fn test_fold_constants_on_hybrid_vector_scan() {
        use crate::unified_plan::VectorScanType;
        let mut plan = UnifiedPlan::HybridVectorScan {
            sql_filter: Some(Expr::BinaryExpr {
                left: Box::new(Expr::Literal("1".to_string())),
                op: BinaryOperator::Plus,
                right: Box::new(Expr::Literal("2".to_string())),
            }),
            vector_index: "vec_idx".to_string(),
            query_vector: vec![1.0, 2.0, 3.0],
            scan_type: VectorScanType::Knn { k: 5 },
            limit: Some(10),
        };
        let rule = ConstantFolding::new();
        let changed = rule.apply(&mut plan);
        assert!(changed);
    }

    #[test]
    fn test_fold_constants_on_hybrid_graph_scan() {
        use crate::unified_plan::GraphScanType;
        let mut plan = UnifiedPlan::HybridGraphScan {
            sql_filter: Some(Expr::BinaryExpr {
                left: Box::new(Expr::Literal("5".to_string())),
                op: BinaryOperator::Plus,
                right: Box::new(Expr::Literal("3".to_string())),
            }),
            graph_name: "social".to_string(),
            scan_type: GraphScanType::Traversal { max_depth: 3 },
            start_node: Some("user1".to_string()),
        };
        let rule = ConstantFolding::new();
        let changed = rule.apply(&mut plan);
        assert!(changed);
    }

    #[test]
    fn test_fold_constants_on_join() {
        let mut plan = UnifiedPlan::Join {
            left: Box::new(UnifiedPlan::TableScan {
                table_name: "t1".to_string(),
                projection: None,
            }),
            right: Box::new(UnifiedPlan::TableScan {
                table_name: "t2".to_string(),
                projection: None,
            }),
            join_type: JoinType::Inner,
            condition: Some(Expr::BinaryExpr {
                left: Box::new(Expr::Literal("1".to_string())),
                op: BinaryOperator::Eq,
                right: Box::new(Expr::Literal("1".to_string())),
            }),
        };
        let rule = ConstantFolding::new();
        let changed = rule.apply(&mut plan);
        assert!(changed);
    }

    #[test]
    fn test_fold_constants_on_aggregate() {
        let mut plan = UnifiedPlan::Aggregate {
            group_by: vec![Expr::BinaryExpr {
                left: Box::new(Expr::Literal("1".to_string())),
                op: BinaryOperator::Plus,
                right: Box::new(Expr::Literal("2".to_string())),
            }],
            aggregates: vec![Expr::Literal("count".to_string())],
            input: Box::new(UnifiedPlan::TableScan {
                table_name: "t".to_string(),
                projection: None,
            }),
        };
        let rule = ConstantFolding::new();
        let changed = rule.apply(&mut plan);
        assert!(changed);
    }

    #[test]
    fn test_fold_constants_on_sort() {
        let mut plan = UnifiedPlan::Sort {
            expr: vec![Expr::BinaryExpr {
                left: Box::new(Expr::Literal("3".to_string())),
                op: BinaryOperator::Plus,
                right: Box::new(Expr::Literal("1".to_string())),
            }],
            input: Box::new(UnifiedPlan::TableScan {
                table_name: "t".to_string(),
                projection: None,
            }),
        };
        let rule = ConstantFolding::new();
        let changed = rule.apply(&mut plan);
        assert!(changed);
    }

    // ===== Tests for JoinType enum =====

    #[test]
    fn test_join_type_inner() {
        assert_eq!(format!("{:?}", JoinType::Inner), "Inner");
    }

    #[test]
    fn test_join_type_left() {
        assert_eq!(format!("{:?}", JoinType::Left), "Left");
    }

    #[test]
    fn test_join_type_right() {
        assert_eq!(format!("{:?}", JoinType::Right), "Right");
    }

    #[test]
    fn test_join_type_full() {
        assert_eq!(format!("{:?}", JoinType::Full), "Full");
    }

    #[test]
    fn test_join_type_cross() {
        assert_eq!(format!("{:?}", JoinType::Cross), "Cross");
    }

    #[test]
    fn test_join_type_left_semi() {
        assert_eq!(format!("{:?}", JoinType::LeftSemi), "LeftSemi");
    }

    #[test]
    fn test_join_type_left_anti() {
        assert_eq!(format!("{:?}", JoinType::LeftAnti), "LeftAnti");
    }

    #[test]
    fn test_join_type_right_semi() {
        assert_eq!(format!("{:?}", JoinType::RightSemi), "RightSemi");
    }

    #[test]
    fn test_join_type_right_anti() {
        assert_eq!(format!("{:?}", JoinType::RightAnti), "RightAnti");
    }

    #[test]
    fn test_join_type_eq() {
        assert_eq!(JoinType::Inner, JoinType::Inner);
        assert_eq!(JoinType::Left, JoinType::Left);
        assert_ne!(JoinType::Inner, JoinType::Left);
    }

    // ===== Additional eval_binary_op edge case tests =====

    #[test]
    fn test_eval_binary_op_plus() {
        let result = eval_binary_op("1", "2", BinaryOperator::Plus);
        assert_eq!(result, Some("3".to_string()));
    }

    #[test]
    fn test_eval_binary_op_eq_true() {
        let result = eval_binary_op("5", "5", BinaryOperator::Eq);
        assert_eq!(result, Some("true".to_string()));
    }

    #[test]
    fn test_eval_binary_op_eq_false() {
        let result = eval_binary_op("5", "3", BinaryOperator::Eq);
        assert_eq!(result, Some("false".to_string()));
    }

    #[test]
    fn test_eval_binary_op_gt() {
        let result = eval_binary_op("5", "3", BinaryOperator::Gt);
        assert_eq!(result, Some("true".to_string()));
        let result2 = eval_binary_op("3", "5", BinaryOperator::Gt);
        assert_eq!(result2, Some("false".to_string()));
    }

    #[test]
    fn test_eval_binary_op_invalid_string() {
        let result = eval_binary_op("abc", "3", BinaryOperator::Plus);
        assert_eq!(result, None);
    }

    #[test]
    fn test_eval_binary_op_and_false_left() {
        let result = eval_binary_op("0", "1", BinaryOperator::And);
        assert_eq!(result, Some("false".to_string()));
    }

    #[test]
    fn test_eval_binary_op_and_false_right() {
        let result = eval_binary_op("1", "0", BinaryOperator::And);
        assert_eq!(result, Some("false".to_string()));
    }

    #[test]
    fn test_eval_binary_op_or_true_left() {
        let result = eval_binary_op("1", "0", BinaryOperator::Or);
        assert_eq!(result, Some("true".to_string()));
    }

    #[test]
    fn test_eval_binary_op_or_false_both() {
        let result = eval_binary_op("0", "0", BinaryOperator::Or);
        assert_eq!(result, Some("false".to_string()));
    }

    #[test]
    fn test_eval_binary_op_and_with_zero() {
        let result = eval_binary_op("0", "1", BinaryOperator::And);
        assert_eq!(result, Some("false".to_string()));
    }

    // ===== Additional fold_constants edge cases =====

    #[test]
    fn test_fold_constants_no_change_column() {
        let expr = Expr::Column("x".to_string());
        let folded = expr.fold_constants();
        assert!(matches!(folded, Expr::Column(_)));
    }

    #[test]
    fn test_fold_constants_binary_expr_no_folding() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Column("a".to_string())),
            op: BinaryOperator::Plus,
            right: Box::new(Expr::Column("b".to_string())),
        };
        let folded = expr.fold_constants();
        assert!(matches!(folded, Expr::BinaryExpr { .. }));
    }

    #[test]
    fn test_fold_constants_or_simplify_right_true() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Column("x".to_string())),
            op: BinaryOperator::Or,
            right: Box::new(Expr::Literal("true".to_string())),
        };
        let folded = expr.fold_constants();
        assert_eq!(folded, Expr::Literal("true".to_string()));
    }

    #[test]
    fn test_fold_constants_and_simplify_right_false() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Column("x".to_string())),
            op: BinaryOperator::And,
            right: Box::new(Expr::Literal("false".to_string())),
        };
        let folded = expr.fold_constants();
        assert_eq!(folded, Expr::Literal("false".to_string()));
    }

    #[test]
    fn test_fold_constants_and_simplify_right_one() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Column("x".to_string())),
            op: BinaryOperator::And,
            right: Box::new(Expr::Literal("1".to_string())),
        };
        let folded = expr.fold_constants();
        assert_eq!(folded, Expr::Column("x".to_string()));
    }

    #[test]
    fn test_fold_constants_or_simplify_left_zero() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Literal("0".to_string())),
            op: BinaryOperator::Or,
            right: Box::new(Expr::Column("y".to_string())),
        };
        let folded = expr.fold_constants();
        assert_eq!(folded, Expr::Column("y".to_string()));
    }

    // ===== Additional references_columns tests =====

    #[test]
    fn test_expr_references_columns_complex() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::BinaryExpr {
                left: Box::new(Expr::Column("a".to_string())),
                op: BinaryOperator::Plus,
                right: Box::new(Expr::Column("b".to_string())),
            }),
            op: BinaryOperator::Multiply,
            right: Box::new(Expr::Literal("2".to_string())),
        };
        let cols = expr.references_columns();
        assert!(cols.contains(&"a"));
        assert!(cols.contains(&"b"));
        assert_eq!(cols.len(), 2);
    }

    // ===== Test predicate_pushdown on IndexScan =====

    #[test]
    fn test_predicate_pushdown_on_index_scan_with_folding() {
        let mut plan = UnifiedPlan::IndexScan {
            table_name: "users".to_string(),
            index_name: "idx_email".to_string(),
            predicate: Some(Expr::BinaryExpr {
                left: Box::new(Expr::Literal("1".to_string())),
                op: BinaryOperator::Plus,
                right: Box::new(Expr::Literal("2".to_string())),
            }),
        };
        let rule = PredicatePushdown::new();
        let changed = rule.apply(&mut plan);
        assert!(changed);
    }

    #[test]
    fn test_predicate_pushdown_on_index_scan_no_change() {
        let mut plan = UnifiedPlan::IndexScan {
            table_name: "users".to_string(),
            index_name: "idx_email".to_string(),
            predicate: Some(Expr::Column("email".to_string())),
        };
        let rule = PredicatePushdown::new();
        let changed = rule.apply(&mut plan);
        assert!(!changed);
    }

    // ===== Test fold_constants on IndexScan =====

    #[test]
    fn test_fold_constants_on_index_scan() {
        let mut plan = UnifiedPlan::IndexScan {
            table_name: "users".to_string(),
            index_name: "idx_email".to_string(),
            predicate: Some(Expr::BinaryExpr {
                left: Box::new(Expr::Literal("5".to_string())),
                op: BinaryOperator::Plus,
                right: Box::new(Expr::Literal("3".to_string())),
            }),
        };
        let rule = ConstantFolding::new();
        let changed = rule.apply(&mut plan);
        assert!(changed);
    }
}
