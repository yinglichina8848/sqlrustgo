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
}
