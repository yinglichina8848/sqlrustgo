//! Physical Planner Module
//!
//! Converts LogicalPlan into PhysicalPlan (executable operators).

use super::logical_plan::LogicalPlan;
use super::physical_plan::{
    AggregateExec, FilterExec, HashJoinExec, LimitExec, PhysicalPlan, ProjectionExec, SeqScanExec,
    SortExec,
};
use crate::types::SqlResult;
use std::sync::Arc;

/// Planner trait - converts LogicalPlan to PhysicalPlan
pub trait Planner: Send + Sync {
    /// Create a physical plan from a logical plan
    fn create_physical_plan(&self, logical_plan: &LogicalPlan) -> SqlResult<Arc<dyn PhysicalPlan>>;
}

/// Default physical planner implementation
pub struct DefaultPlanner;

impl DefaultPlanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Planner for DefaultPlanner {
    fn create_physical_plan(&self, logical_plan: &LogicalPlan) -> SqlResult<Arc<dyn PhysicalPlan>> {
        self.plan_recursive(logical_plan)
    }
}

impl DefaultPlanner {
    fn plan_recursive(&self, logical_plan: &LogicalPlan) -> SqlResult<Arc<dyn PhysicalPlan>> {
        match logical_plan {
            LogicalPlan::TableScan {
                table_name,
                projection,
                filters,
                limit,
                schema,
            } => {
                // For now, return empty rows - in real implementation would read from storage
                let rows = vec![];
                Ok(Arc::new(SeqScanExec::new(
                    table_name.clone(),
                    schema.clone(),
                    rows,
                    projection.clone(),
                    filters.clone(),
                    *limit,
                )))
            }

            LogicalPlan::Projection {
                input,
                expr,
                schema,
            } => {
                let input_plan = self.plan_recursive(input)?;
                Ok(Arc::new(ProjectionExec::new(
                    input_plan,
                    expr.clone(),
                    schema.clone(),
                )))
            }

            LogicalPlan::Filter { input, predicate } => {
                let input_plan = self.plan_recursive(input)?;
                Ok(Arc::new(FilterExec::new(input_plan, predicate.clone())))
            }

            LogicalPlan::Aggregate {
                input,
                group_expr,
                aggr_expr,
                schema,
            } => {
                let input_plan = self.plan_recursive(input)?;
                // Convert aggregate expressions
                let physical_aggr_expr = self.convert_aggregate_exprs(aggr_expr)?;
                Ok(Arc::new(AggregateExec::new(
                    input_plan,
                    group_expr.clone(),
                    physical_aggr_expr,
                    schema.clone(),
                )))
            }

            LogicalPlan::Join {
                left,
                right,
                join_type,
                on,
                filter: _,
                schema,
            } => {
                let left_plan = self.plan_recursive(left)?;
                let right_plan = self.plan_recursive(right)?;

                // Convert join on expressions to Column pairs
                let on_columns = self.convert_join_on(on)?;

                Ok(Arc::new(HashJoinExec::new(
                    left_plan,
                    right_plan,
                    on_columns,
                    join_type.clone(),
                    schema.clone(),
                )))
            }

            LogicalPlan::Sort { input, expr } => {
                let input_plan = self.plan_recursive(input)?;
                let schema = input_plan.schema().clone();
                Ok(Arc::new(SortExec::new(input_plan, expr.clone(), schema)))
            }

            LogicalPlan::Limit { input, n } => {
                let input_plan = self.plan_recursive(input)?;
                Ok(Arc::new(LimitExec::new(input_plan, *n)))
            }

            LogicalPlan::Values { values, schema } => {
                // VALUES clause - evaluate expressions to create rows
                let mut rows = Vec::new();
                for row_exprs in values {
                    let mut row = Vec::new();
                    for expr in row_exprs {
                        let val = self.evaluate_expr(expr)?;
                        row.push(val);
                    }
                    rows.push(row);
                }
                Ok(Arc::new(SeqScanExec::new(
                    "values".to_string(),
                    schema.clone(),
                    rows,
                    None,
                    vec![],
                    None,
                )))
            }

            LogicalPlan::EmptyRelation {
                produce_one_row,
                schema,
            } => {
                let rows = if *produce_one_row {
                    vec![vec![]]
                } else {
                    vec![]
                };
                Ok(Arc::new(SeqScanExec::new(
                    "empty".to_string(),
                    schema.clone(),
                    rows,
                    None,
                    vec![],
                    None,
                )))
            }

            LogicalPlan::Subquery { subquery, .. } => self.plan_recursive(subquery),

            LogicalPlan::Union { inputs, schema } => {
                // Union - execute all inputs and concatenate results
                let mut all_rows = Vec::new();
                for input in inputs {
                    let plan = self.plan_recursive(input)?;
                    let rows = plan.execute()?;
                    all_rows.extend(rows);
                }
                Ok(Arc::new(SeqScanExec::new(
                    "union".to_string(),
                    schema.clone(),
                    all_rows,
                    None,
                    vec![],
                    None,
                )))
            }

            LogicalPlan::Update {
                input,
                set_exprs: _,
                schema: _,
            } => {
                let input_plan = self.plan_recursive(input)?;
                // For UPDATE, we need a special operator - for now, use input as is
                Ok(input_plan)
            }

            LogicalPlan::Delete { input, schema: _ } => {
                let input_plan = self.plan_recursive(input)?;
                Ok(input_plan)
            }

            LogicalPlan::CreateTable { name, schema } => {
                // CREATE TABLE - no physical plan needed
                Ok(Arc::new(SeqScanExec::new(
                    name.clone(),
                    schema.clone(),
                    vec![],
                    None,
                    vec![],
                    None,
                )))
            }

            LogicalPlan::DropTable { name, schema } => {
                // DROP TABLE - no physical plan needed
                Ok(Arc::new(SeqScanExec::new(
                    name.clone(),
                    schema.clone(),
                    vec![],
                    None,
                    vec![],
                    None,
                )))
            }
        }
    }

    fn convert_aggregate_exprs(
        &self,
        aggr_expr: &[super::Expr],
    ) -> SqlResult<Vec<(super::AggregateFunction, super::Expr)>> {
        let mut result = Vec::new();
        for expr in aggr_expr {
            match expr {
                super::Expr::AggregateFunction {
                    func,
                    args,
                    distinct: _,
                } => {
                    let arg = args
                        .first()
                        .cloned()
                        .unwrap_or(super::Expr::Literal(crate::types::Value::Null));
                    result.push((func.clone(), arg));
                }
                _ => {
                    // Non-aggregate expression - treat as pass-through
                    result.push((super::AggregateFunction::Count, expr.clone()));
                }
            }
        }
        Ok(result)
    }

    fn convert_join_on(
        &self,
        on: &[(super::Expr, super::Expr)],
    ) -> SqlResult<Vec<(super::Column, super::Column)>> {
        let mut result = Vec::new();
        for (left, right) in on {
            match (left, right) {
                (super::Expr::Column(l), super::Expr::Column(r)) => {
                    result.push((l.clone(), r.clone()));
                }
                _ => {
                    return Err(crate::types::SqlError::ExecutionError(
                        "Join ON clause must compare columns".to_string(),
                    ));
                }
            }
        }
        Ok(result)
    }

    fn evaluate_expr(&self, expr: &super::Expr) -> SqlResult<crate::types::Value> {
        match expr {
            super::Expr::Literal(val) => Ok(val.clone()),
            _ => Ok(crate::types::Value::Null),
        }
    }
}

/// No-op planner that returns an error
pub struct NoOpPlanner;

impl Planner for NoOpPlanner {
    fn create_physical_plan(&self, _: &LogicalPlan) -> SqlResult<Arc<dyn PhysicalPlan>> {
        Err(crate::types::SqlError::ExecutionError(
            "NoOpPlanner cannot create physical plans".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::{DataType, Field, Schema};

    fn test_schema() -> Schema {
        Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ])
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_default_planner_new() {
        let _planner = DefaultPlanner::new();
        assert!(true); // Just check it can be created
    }

    #[test]
    fn test_planner_table_scan() {
        let planner = DefaultPlanner::new();
        let logical_plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
            filters: vec![],
            limit: None,
            schema: test_schema(),
        };

        let physical_plan = planner.create_physical_plan(&logical_plan);
        assert!(physical_plan.is_ok());
    }

    #[test]
    fn test_planner_projection() {
        let planner = DefaultPlanner::new();

        let input = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
            filters: vec![],
            limit: None,
            schema: test_schema(),
        };

        let logical_plan = LogicalPlan::Projection {
            input: Box::new(input),
            expr: vec![super::super::Expr::Column(super::super::Column::new(
                "id".to_string(),
            ))],
            schema: test_schema(),
        };

        let physical_plan = planner.create_physical_plan(&logical_plan);
        assert!(physical_plan.is_ok());
    }

    #[test]
    fn test_planner_filter() {
        let planner = DefaultPlanner::new();

        let input = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
            filters: vec![],
            limit: None,
            schema: test_schema(),
        };

        let logical_plan = LogicalPlan::Filter {
            input: Box::new(input),
            predicate: super::super::Expr::BinaryExpr {
                left: Box::new(super::super::Expr::Column(super::super::Column::new(
                    "id".to_string(),
                ))),
                op: super::super::Operator::Gt,
                right: Box::new(super::super::Expr::Literal(crate::types::Value::Integer(
                    10,
                ))),
            },
        };

        let physical_plan = planner.create_physical_plan(&logical_plan);
        assert!(physical_plan.is_ok());
    }

    #[test]
    fn test_planner_limit() {
        let planner = DefaultPlanner::new();

        let input = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
            filters: vec![],
            limit: None,
            schema: test_schema(),
        };

        let logical_plan = LogicalPlan::Limit {
            input: Box::new(input),
            n: 10,
        };

        let physical_plan = planner.create_physical_plan(&logical_plan);
        assert!(physical_plan.is_ok());
    }

    #[test]
    fn test_noop_planner() {
        let planner = NoOpPlanner;
        let logical_plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
            filters: vec![],
            limit: None,
            schema: test_schema(),
        };

        let result = planner.create_physical_plan(&logical_plan);
        assert!(result.is_err());
    }
}
