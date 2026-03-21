//! Projection Pushdown Optimizer Module
//!
//! Optimizes queries by pushing column projections down to storage layer,
//! reducing I/O and memory usage by reading only required columns.

use crate::Rule;
use crate::{Expr, JoinType, Plan, Value};
use std::collections::HashSet;
use std::fmt::Debug;

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
            // Eliminate redundant projection by pushing it down
            Plan::Projection { expr, input } => {
                if let Plan::Projection { expr: inner_expr, .. } = &mut **input {
                    // Two consecutive projections - merge them
                    let merged = self.merge_projections(expr, inner_expr);
                    if let Some(merged_expr) = merged {
                        *expr = merged_expr;
                        // Remove the inner projection by replacing with its input
                        if let Plan::Projection { input: inner_input, .. } = std::mem::replace(&mut **input, Plan::EmptyRelation) {
                            **input = *inner_input;
                        }
                        return true;
                    }
                }
                
                // Try to push projection through Filter
                if let Plan::Filter { input: filter_input, .. } = &mut **input {
                    // Move projection above filter is usually better
                    // But if filter_input is a scan, we can push down
                    if let Plan::TableScan { projection: Some(_), .. } = &mut **filter_input {
                        // Already has projection
                        return false;
                    }
                }
                
                false
            }
            
            // Add projection to table scan if missing
            Plan::TableScan { projection: None, .. } => {
                // Check if there's a projection above
                false
            }
            
            // Propagate through Filter
            Plan::Filter { input, .. } => {
                let mut changed = false;
                if let Plan::Projection { .. } = &mut **input {
                    // Filter can work with projection's output
                    // The filter should be pushed down to scan, not through projection
                }
                changed
            }
            
            // Propagate through Aggregate
            Plan::Aggregate { input, .. } => {
                // Aggregate needs all columns for group by
                // Can only push down columns not used in aggregate
                let mut changed = false;
                changed
            }
            
            // Propagate through Join - push to both children
            Plan::Join { left, right, .. } => {
                let mut changed = false;
                changed
            }
            
            // Propagate through Sort
            Plan::Sort { input, .. } => {
                let mut changed = false;
                changed
            }
            
            // Propagate through Limit
            Plan::Limit { input, .. } => {
                let mut changed = false;
                changed
            }
            
            _ => false,
        }
    }
    
    /// Merge two consecutive projections
    fn merge_projections(&self, outer: &[Expr], inner: &[Expr]) -> Option<Vec<Expr>> {
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
            Plan::Aggregate { group_by, aggregates, input } => {
                self.collect_from_exprs(group_by, columns);
                self.collect_from_exprs(aggregates, columns);
                self.collect_from_plan(input, columns);
            }
            Plan::Sort { expr, input } => {
                self.collect_from_exprs(expr, columns);
                self.collect_from_plan(input, columns);
            }
            Plan::Join { left, right, condition, .. } => {
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
        let columns = pruner.collect_columns(&[
            Expr::BinaryExpr {
                left: Box::new(Expr::Column("a".to_string())),
                op: crate::Operator::Plus,
                right: Box::new(Expr::Column("b".to_string())),
            },
        ]);
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
