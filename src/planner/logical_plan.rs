//! Logical Plan Definitions
//!
//! Defines the LogicalPlan enum representing the logical query execution plan.

use super::{Expr, JoinType, Schema, SortExpr};
use std::fmt;

#[derive(Debug, Clone)]
pub enum LogicalPlan {
    Projection {
        input: Box<LogicalPlan>,
        expr: Vec<Expr>,
        schema: Schema,
    },
    Filter {
        input: Box<LogicalPlan>,
        predicate: Expr,
    },
    Aggregate {
        input: Box<LogicalPlan>,
        group_expr: Vec<Expr>,
        aggr_expr: Vec<Expr>,
        schema: Schema,
    },
    Join {
        left: Box<LogicalPlan>,
        right: Box<LogicalPlan>,
        join_type: JoinType,
        on: Vec<(Expr, Expr)>,
        filter: Option<Expr>,
        schema: Schema,
    },
    TableScan {
        table_name: String,
        projection: Option<Vec<usize>>,
        filters: Vec<Expr>,
        limit: Option<usize>,
        schema: Schema,
    },
    Sort {
        input: Box<LogicalPlan>,
        expr: Vec<SortExpr>,
    },
    Limit {
        input: Box<LogicalPlan>,
        n: usize,
    },
    Values {
        values: Vec<Vec<Expr>>,
        schema: Schema,
    },
    EmptyRelation {
        produce_one_row: bool,
        schema: Schema,
    },
    Subquery {
        subquery: Box<LogicalPlan>,
        alias: Option<String>,
    },
    Union {
        inputs: Vec<LogicalPlan>,
        schema: Schema,
    },
}

impl LogicalPlan {
    pub fn schema(&self) -> &Schema {
        match self {
            LogicalPlan::Projection { schema, .. } => schema,
            LogicalPlan::Filter { input, .. } => input.schema(),
            LogicalPlan::Aggregate { schema, .. } => schema,
            LogicalPlan::Join { schema, .. } => schema,
            LogicalPlan::TableScan { schema, .. } => schema,
            LogicalPlan::Sort { input, .. } => input.schema(),
            LogicalPlan::Limit { input, .. } => input.schema(),
            LogicalPlan::Values { schema, .. } => schema,
            LogicalPlan::EmptyRelation { schema, .. } => schema,
            LogicalPlan::Subquery { subquery, .. } => subquery.schema(),
            LogicalPlan::Union { schema, .. } => schema,
        }
    }

    pub fn children(&self) -> Vec<&LogicalPlan> {
        match self {
            LogicalPlan::Projection { input, .. } => vec![input],
            LogicalPlan::Filter { input, .. } => vec![input],
            LogicalPlan::Aggregate { input, .. } => vec![input],
            LogicalPlan::Join { left, right, .. } => vec![left, right],
            LogicalPlan::TableScan { .. } => vec![],
            LogicalPlan::Sort { input, .. } => vec![input],
            LogicalPlan::Limit { input, .. } => vec![input],
            LogicalPlan::Values { .. } => vec![],
            LogicalPlan::EmptyRelation { .. } => vec![],
            LogicalPlan::Subquery { subquery, .. } => vec![subquery],
            LogicalPlan::Union { inputs, .. } => inputs.iter().collect(),
        }
    }
}

impl fmt::Display for LogicalPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalPlan::Projection { expr, .. } => {
                write!(
                    f,
                    "Projection: {}",
                    expr.iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            LogicalPlan::Filter { predicate, .. } => {
                write!(f, "Filter: {}", predicate)
            }
            LogicalPlan::Aggregate {
                group_expr,
                aggr_expr,
                ..
            } => {
                let group_str: Vec<String> = group_expr.iter().map(|e| e.to_string()).collect();
                let aggr_str: Vec<String> = aggr_expr.iter().map(|e| e.to_string()).collect();
                write!(
                    f,
                    "Aggregate: group=[{}] aggr=[{}]",
                    group_str.join(", "),
                    aggr_str.join(", ")
                )
            }
            LogicalPlan::Join { join_type, on, .. } => {
                let on_str: Vec<String> = on.iter().map(|(l, r)| format!("{}={}", l, r)).collect();
                write!(f, "Join: type={:?} on={}", join_type, on_str.join(", "))
            }
            LogicalPlan::TableScan { table_name, .. } => {
                write!(f, "TableScan: {}", table_name)
            }
            LogicalPlan::Sort { expr, .. } => {
                let expr_str: Vec<String> = expr.iter().map(|e| e.expr.to_string()).collect();
                write!(f, "Sort: {}", expr_str.join(", "))
            }
            LogicalPlan::Limit { n, .. } => {
                write!(f, "Limit: {}", n)
            }
            LogicalPlan::Values { values, .. } => {
                write!(f, "Values: {} rows", values.len())
            }
            LogicalPlan::EmptyRelation {
                produce_one_row, ..
            } => {
                write!(f, "EmptyRelation: produce_one_row={}", produce_one_row)
            }
            LogicalPlan::Subquery { alias, .. } => {
                write!(f, "Subquery: alias={:?}", alias)
            }
            LogicalPlan::Union { inputs, .. } => {
                write!(f, "Union: {} inputs", inputs.len())
            }
        }
    }
}
