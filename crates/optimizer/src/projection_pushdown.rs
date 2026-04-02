//! Projection Pushdown Optimizer Module
//!
//! Optimizes queries by pushing column projections down to storage layer,
//! reducing I/O and memory usage by reading only required columns.

use crate::Rule;
use crate::{Expr, Plan};
use std::collections::HashSet;
use std::fmt::Debug;

/// Column set for tracking used columns during projection pushdown
#[derive(Debug, Clone)]
pub struct ColumnSet {
    /// Column indices (for projection pushdown)
    pub indices: Vec<usize>,
    /// Column names (for analysis)
    pub names: HashSet<String>,
    /// Whether all columns are needed
    pub is_all: bool,
}

impl ColumnSet {
    /// Create a new empty column set (meaning all columns needed)
    pub fn new() -> Self {
        Self {
            indices: Vec::new(),
            names: HashSet::new(),
            is_all: true,
        }
    }

    /// Add a column by name
    pub fn add(&mut self, name: &str) {
        self.is_all = false;
        self.names.insert(name.to_string());
    }
}

impl Default for ColumnSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Projection Pushdown Rule - pushes column projections down to table scans
/// This reduces the amount of data read from storage
pub struct ProjectionPushdownRule;

impl ProjectionPushdownRule {
    /// Create a new ProjectionPushdownRule
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProjectionPushdownRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule<Plan> for ProjectionPushdownRule {
    fn name(&self) -> &str {
        "ProjectionPushdown"
    }

    fn apply(&self, plan: &mut Plan) -> bool {
        self.optimize(plan)
    }
}

impl ProjectionPushdownRule {
    /// Optimize plan by pushing down projections
    fn optimize(&self, plan: &mut Plan) -> bool {
        match plan {
            // Push projection down to table scan
            Plan::Projection { expr, input } => {
                // Check if input is a table scan without projection
                if let Plan::TableScan {
                    projection: None, ..
                } = &mut **input
                {
                    // Collect columns used in the projection expression
                    let used_cols = self.collect_columns(expr);

                    // If not all columns are needed, set the projection on the scan
                    if !used_cols.is_all && !used_cols.indices.is_empty() {
                        let new_projection = Some(used_cols.indices);
                        // Projection is None here, we need to modify the input's projection
                        if let Plan::TableScan { projection, .. } = &mut **input {
                            *projection = new_projection;
                        }
                        return true;
                    }
                }

                // Try to push through Filter
                if let Plan::Filter {
                    input: filter_input,
                    ..
                } = &mut **input
                {
                    if let Plan::TableScan {
                        projection: None, ..
                    } = &mut **filter_input
                    {
                        // Collect columns needed by both filter and projection
                        let used_cols = self.collect_columns(expr);

                        if !used_cols.is_all && !used_cols.indices.is_empty() {
                            let new_projection = Some(used_cols.indices);
                            if let Plan::TableScan { projection, .. } = &mut **filter_input {
                                *projection = new_projection;
                            }
                            return true;
                        }
                    }
                }

                // Recurse into input
                self.pushdown(input)
            }

            // Propagate through Filter
            Plan::Filter { input, .. } => self.pushdown(input),

            // Propagate through Aggregate - aggregate needs all columns
            Plan::Aggregate { input: _, .. } => false,

            // Propagate through Join - push to both children
            Plan::Join { left, right, .. } => {
                let changed_left = self.pushdown(left);
                let changed_right = self.pushdown(right);
                changed_left || changed_right
            }

            // Propagate through Sort
            Plan::Sort { input, .. } => self.pushdown(input),

            // Propagate through Limit
            Plan::Limit { input, .. } => self.pushdown(input),

            _ => false,
        }
    }

    /// Push projection down into a plan node
    fn pushdown(&self, plan: &mut Plan) -> bool {
        match plan {
            Plan::Projection { expr, input } => {
                if let Plan::TableScan {
                    projection: None, ..
                } = &mut **input
                {
                    let used_cols = self.collect_columns(expr);
                    if !used_cols.is_all && !used_cols.indices.is_empty() {
                        let new_projection = Some(used_cols.indices);
                        if let Plan::TableScan { projection, .. } = &mut **input {
                            *projection = new_projection;
                        }
                        return true;
                    }
                }
                self.pushdown(input)
            }
            Plan::Filter { input, .. } => self.pushdown(input),
            Plan::Join { left, right, .. } => {
                let changed_left = self.pushdown(left);
                let changed_right = self.pushdown(right);
                changed_left || changed_right
            }
            Plan::Sort { input, .. } => self.pushdown(input),
            Plan::Limit { input, .. } => self.pushdown(input),
            Plan::Aggregate { .. } => false,
            _ => false,
        }
    }

    /// Collect columns used in expressions and return as ColumnSet
    fn collect_columns(&self, exprs: &[Expr]) -> ColumnSet {
        let mut cols = ColumnSet::new();
        for expr in exprs {
            self.collect_from_expr(expr, &mut cols);
        }
        cols
    }

    fn collect_from_expr(&self, expr: &Expr, cols: &mut ColumnSet) {
        match expr {
            Expr::Column(name) => {
                cols.add(name);
            }
            Expr::BinaryExpr { left, right, .. } => {
                self.collect_from_expr(left, cols);
                self.collect_from_expr(right, cols);
            }
            Expr::UnaryExpr { expr: inner, .. } => {
                self.collect_from_expr(inner, cols);
            }
            _ => {}
        }
    }

    /// Merge two consecutive projections
    #[allow(dead_code)]
    fn merge_projections(&self, outer: &[Expr], _inner: &[Expr]) -> Option<Vec<Expr>> {
        // This is a simplified merge - in reality we'd need expression analysis
        let mut merged = Vec::new();
        for expr in outer {
            if let Expr::Column(_) = expr {
                merged.push(expr.clone());
            }
        }
        if merged.is_empty() {
            None
        } else {
            Some(merged)
        }
    }
}

/// Column pruner - identifies unused columns
pub struct ColumnPruner;

impl ColumnPruner {
    /// Create a new ColumnPruner
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    /// Collect all columns referenced in expressions
    pub fn collect_columns(&self, exprs: &[Expr]) -> HashSet<String> {
        let mut columns = HashSet::new();
        self.collect_from_exprs(exprs, &mut columns);
        columns
    }

    fn collect_from_exprs(&self, exprs: &[Expr], columns: &mut HashSet<String>) {
        for expr in exprs {
            self.collect_from_expr(expr, columns);
        }
    }

    fn collect_from_expr(&self, expr: &Expr, columns: &mut HashSet<String>) {
        match expr {
            Expr::Column(name) => {
                columns.insert(name.clone());
            }
            Expr::BinaryExpr { left, right, .. } => {
                self.collect_from_expr(left, columns);
                self.collect_from_expr(right, columns);
            }
            Expr::UnaryExpr { expr: inner, .. } => {
                self.collect_from_expr(inner, columns);
            }
            _ => {}
        }
    }

    /// Determine which columns are needed based on plan
    pub fn required_columns(&self, plan: &Plan) -> HashSet<String> {
        let mut columns = HashSet::new();
        self.collect_from_plan(plan, &mut columns);
        columns
    }

    fn collect_from_plan(&self, plan: &Plan, columns: &mut HashSet<String>) {
        match plan {
            Plan::Projection { expr, input } => {
                self.collect_from_exprs(expr, columns);
                self.collect_from_plan(input, columns);
            }
            Plan::Filter { predicate, input } => {
                self.collect_from_expr(predicate, columns);
                self.collect_from_plan(input, columns);
            }
            Plan::Aggregate {
                group_by,
                aggregates,
                input,
            } => {
                self.collect_from_exprs(group_by, columns);
                self.collect_from_exprs(aggregates, columns);
                self.collect_from_plan(input, columns);
            }
            Plan::Sort { expr, input } => {
                self.collect_from_exprs(expr, columns);
                self.collect_from_plan(input, columns);
            }
            Plan::Join {
                left,
                right,
                condition,
                ..
            } => {
                if let Some(cond) = condition {
                    self.collect_from_expr(cond, columns);
                }
                self.collect_from_plan(left, columns);
                self.collect_from_plan(right, columns);
            }
            _ => {}
        }
    }
}

/// Optimizer configuration for projection pushdown
#[derive(Debug, Clone)]
pub struct ProjectionPushdownConfig {
    /// Enable projection pushdown
    pub enabled: bool,
    /// Minimum columns needed to enable pushdown
    pub min_columns: usize,
    /// Enable columnar storage for pushdown
    pub use_columnar: bool,
}

impl Default for ProjectionPushdownConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_columns: 2,
            use_columnar: true,
        }
    }
}

/// Projection pushdown optimizer
pub struct ProjectionPushdownOptimizer {
    config: ProjectionPushdownConfig,
}

impl ProjectionPushdownOptimizer {
    /// Create a new optimizer with default config
    pub fn new() -> Self {
        Self {
            config: ProjectionPushdownConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: ProjectionPushdownConfig) -> Self {
        Self { config }
    }

    /// Optimize a plan
    pub fn optimize(&self, plan: &Plan) -> Plan {
        if !self.config.enabled {
            return plan.clone();
        }

        let mut optimized = plan.clone();
        let rule = ProjectionPushdownRule::new();

        // Apply rule until fixed point
        let mut changed = true;
        let mut iterations = 0;
        while changed && iterations < 10 {
            changed = rule.apply(&mut optimized);
            iterations += 1;
        }

        optimized
    }

    /// Check if pushdown would be beneficial
    pub fn is_beneficial(&self, total_columns: usize, required_columns: usize) -> bool {
        if !self.config.enabled {
            return false;
        }
        if total_columns < self.config.min_columns {
            return false;
        }
        // Benefit if we're selecting less than half
        required_columns * 2 < total_columns
    }
}

impl Default for ProjectionPushdownOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;

    #[test]
    fn test_projection_pushdown_rule_name() {
        let rule = ProjectionPushdownRule::new();
        assert_eq!(rule.name(), "ProjectionPushdown");
    }

    #[test]
    fn test_column_pruner_collect() {
        let pruner = ColumnPruner::new();
        let columns = pruner.collect_columns(&[
            Expr::Column("id".to_string()),
            Expr::Column("name".to_string()),
        ]);
        assert_eq!(columns.len(), 2);
        assert!(columns.contains("id"));
        assert!(columns.contains("name"));
    }

    #[test]
    fn test_column_pruner_nested() {
        let pruner = ColumnPruner::new();
        let columns = pruner.collect_columns(&[Expr::BinaryExpr {
            left: Box::new(Expr::Column("a".to_string())),
            op: crate::Operator::Plus,
            right: Box::new(Expr::Column("b".to_string())),
        }]);
        assert_eq!(columns.len(), 2);
        assert!(columns.contains("a"));
        assert!(columns.contains("b"));
    }

    #[test]
    fn test_projection_pushdown_config_default() {
        let config = ProjectionPushdownConfig::default();
        assert!(config.enabled);
        assert_eq!(config.min_columns, 2);
        assert!(config.use_columnar);
    }

    #[test]
    fn test_projection_pushdown_optimizer_default() {
        let optimizer = ProjectionPushdownOptimizer::new();
        let plan = Plan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        };
        let optimized = optimizer.optimize(&plan);
        assert!(matches!(optimized, Plan::TableScan { .. }));
    }

    #[test]
    fn test_is_beneficial() {
        let optimizer = ProjectionPushdownOptimizer::new();
        // 10 columns total, need 3 - beneficial
        assert!(optimizer.is_beneficial(10, 3));
        // 10 columns total, need 6 - not beneficial
        assert!(!optimizer.is_beneficial(10, 6));
        // Only 2 columns total - not beneficial (below threshold)
        assert!(!optimizer.is_beneficial(2, 1));
    }

    #[test]
    fn test_merge_projections() {
        let pruner = ProjectionPushdownRule::new();
        let merged = pruner.merge_projections(
            &[Expr::Column("a".to_string()), Expr::Column("b".to_string())],
            &[Expr::Column("x".to_string()), Expr::Column("y".to_string())],
        );
        // Simple merge just copies outer expr
        assert!(merged.is_some());
    }

    #[test]
    fn test_required_columns_projection() {
        let pruner = ColumnPruner::new();
        let plan = Plan::Projection {
            expr: vec![
                Expr::Column("id".to_string()),
                Expr::Column("name".to_string()),
            ],
            input: Box::new(Plan::TableScan {
                table_name: "users".to_string(),
                projection: None,
            }),
        };
        let columns = pruner.required_columns(&plan);
        assert!(columns.contains("id"));
        assert!(columns.contains("name"));
    }

    #[test]
    fn test_required_columns_nested() {
        let pruner = ColumnPruner::new();
        let plan = Plan::Projection {
            expr: vec![Expr::Column("name".to_string())],
            input: Box::new(Plan::Filter {
                predicate: Expr::BinaryExpr {
                    left: Box::new(Expr::Column("id".to_string())),
                    op: crate::Operator::Gt,
                    right: Box::new(Expr::Literal(Value::Integer(10))),
                },
                input: Box::new(Plan::TableScan {
                    table_name: "users".to_string(),
                    projection: None,
                }),
            }),
        };
        let columns = pruner.required_columns(&plan);
        assert!(columns.contains("name"));
        assert!(columns.contains("id")); // From filter predicate
    }

    #[test]
    fn test_disabled_optimizer() {
        let config = ProjectionPushdownConfig {
            enabled: false,
            ..Default::default()
        };
        let optimizer = ProjectionPushdownOptimizer::with_config(config);
        let plan = Plan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        };
        let optimized = optimizer.optimize(&plan);
        // Should return same plan since disabled
        assert!(matches!(optimized, Plan::TableScan { .. }));
    }
}
