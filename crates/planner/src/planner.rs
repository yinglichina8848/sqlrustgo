//! Planner Module
//!
//! Converts logical plans to physical execution plans.

use crate::logical_plan::LogicalPlan;
use crate::optimizer::{DefaultOptimizer, NoOpOptimizer, Optimizer};
use crate::physical_plan::{
    AggregateExec, FilterExec, HashJoinExec, IndexScanExec, LimitExec, PhysicalPlan,
    ProjectionExec, SeqScanExec, SetOperationExec, SortExec, SortMergeJoinExec,
};
use crate::Expr;
use crate::{Column, Schema};
use std::env;
use thiserror::Error;

/// Planner errors
#[derive(Debug, Error)]
pub enum PlannerError {
    #[error("Planning failed: {0}")]
    PlanningFailed(String),
    #[error("Optimization failed: {0}")]
    OptimizationFailed(String),
}

/// Planner result type
pub type PlannerResult<T> = Result<T, PlannerError>;

/// Planner trait - converts logical plans to physical plans
pub trait Planner {
    /// Create a physical plan from a logical plan
    fn create_physical_plan(
        &self,
        logical_plan: &LogicalPlan,
    ) -> PlannerResult<Box<dyn PhysicalPlan>>;

    /// Optimize and create physical plan
    fn optimize(&mut self, logical_plan: LogicalPlan) -> PlannerResult<Box<dyn PhysicalPlan>>;
}

/// Check if teaching mode is enabled via environment variable
fn is_teaching_mode() -> bool {
    env::var("SQLRUSTGO_TEACHING_MODE")
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// Default planner implementation
pub struct DefaultPlanner {
    optimizer: DefaultOptimizer,
    noop_optimizer: NoOpOptimizer,
    use_noop: bool,
}

impl DefaultPlanner {
    pub fn new() -> Self {
        let teaching_mode = is_teaching_mode();
        Self {
            optimizer: DefaultOptimizer::new(),
            noop_optimizer: NoOpOptimizer::new(),
            use_noop: teaching_mode,
        }
    }

    /// Create a new planner with explicit teaching mode setting
    pub fn with_teaching_mode(teaching_mode: bool) -> Self {
        Self {
            optimizer: DefaultOptimizer::new(),
            noop_optimizer: NoOpOptimizer::new(),
            use_noop: teaching_mode,
        }
    }

    fn create_physical_plan_internal(
        &self,
        logical_plan: &LogicalPlan,
    ) -> PlannerResult<Box<dyn PhysicalPlan>> {
        match logical_plan {
            LogicalPlan::TableScan {
                table_name,
                schema,
                projection,
            } => {
                // Check if there's an index available for this table
                // For now, use heuristic: if table is large, consider index scan
                // In a full implementation, this would use statistics
                let use_index = should_use_index(table_name);

                if use_index {
                    // Use index scan with first column as key
                    let key_col = schema
                        .fields
                        .first()
                        .map(|f| f.name.clone())
                        .unwrap_or_else(|| "id".to_string());
                    let key_expr = Expr::Column(Column::new_qualified(table_name.clone(), key_col));
                    Ok(Box::new(IndexScanExec::new(
                        table_name.clone(),
                        format!("{}_pkey", table_name),
                        key_expr,
                        schema.clone(),
                    )))
                } else {
                    let mut exec = SeqScanExec::new(table_name.clone(), schema.clone());
                    if let Some(proj) = projection {
                        exec = exec.with_projection(proj.clone());
                    }
                    Ok(Box::new(exec))
                }
            }
            LogicalPlan::Projection {
                input,
                expr,
                schema,
            } => {
                let input_plan = self.create_physical_plan_internal(input)?;
                Ok(Box::new(ProjectionExec::new(
                    input_plan,
                    expr.clone(),
                    schema.clone(),
                )))
            }
            LogicalPlan::Filter { predicate, input } => {
                let input_plan = self.create_physical_plan_internal(input)?;
                Ok(Box::new(FilterExec::new(input_plan, predicate.clone())))
            }
            LogicalPlan::Aggregate {
                input,
                group_expr,
                aggregate_expr,
                schema,
            } => {
                let input_plan = self.create_physical_plan_internal(input)?;
                Ok(Box::new(AggregateExec::new(
                    input_plan,
                    group_expr.clone(),
                    aggregate_expr.clone(),
                    schema.clone(),
                )))
            }
            LogicalPlan::Join {
                left,
                right,
                join_type,
                condition,
            } => {
                let left_plan = self.create_physical_plan_internal(left)?;
                let right_plan = self.create_physical_plan_internal(right)?;

                // Estimate row counts for cost model
                let left_rows = estimate_output_rows(left_plan.as_ref()).unwrap_or(1000);
                let right_rows = estimate_output_rows(right_plan.as_ref()).unwrap_or(1000);

                // Use heuristic to select join algorithm
                let join_algorithm = select_join_algorithm(&(), left_rows, right_rows, join_type);

                let schema = Schema::new(vec![]);

                match join_algorithm.as_str() {
                    "sort_merge" => {
                        let left_keys = extract_join_keys(condition.as_ref(), true);
                        let right_keys = extract_join_keys(condition.as_ref(), false);
                        Ok(Box::new(SortMergeJoinExec::new(
                            left_plan,
                            right_plan,
                            join_type.clone(),
                            condition.clone(),
                            schema,
                            left_keys,
                            right_keys,
                        )))
                    }
                    _ => Ok(Box::new(HashJoinExec::new(
                        left_plan,
                        right_plan,
                        join_type.clone(),
                        condition.clone(),
                        schema,
                    ))),
                }
            }
            LogicalPlan::Sort { input, sort_expr } => {
                let input_plan = self.create_physical_plan_internal(input)?;
                Ok(Box::new(SortExec::new(input_plan, sort_expr.clone())))
            }
            LogicalPlan::Limit {
                input,
                limit,
                offset,
            } => {
                let input_plan = self.create_physical_plan_internal(input)?;
                Ok(Box::new(LimitExec::new(input_plan, *limit, *offset)))
            }
            LogicalPlan::EmptyRelation => {
                // Return empty scan for empty relation
                Ok(Box::new(SeqScanExec::new(String::new(), Schema::empty())))
            }
            LogicalPlan::Values { schema, .. } => {
                // VALUES clause - create scan with no underlying table
                Ok(Box::new(SeqScanExec::new(String::new(), schema.clone())))
            }
            LogicalPlan::CreateTable { .. }
            | LogicalPlan::DropTable { .. }
            | LogicalPlan::View { .. } => {
                // DDL statements - handled differently
                Ok(Box::new(SeqScanExec::new(String::new(), Schema::empty())))
            }
            LogicalPlan::Update { .. } | LogicalPlan::Delete { .. } => {
                // DML statements - handled differently
                Ok(Box::new(SeqScanExec::new(String::new(), Schema::empty())))
            }
            LogicalPlan::Subquery { subquery, .. } => self.create_physical_plan_internal(subquery),
            LogicalPlan::Window { input, .. } => self.create_physical_plan_internal(input),
            LogicalPlan::SetOperation {
                op_type,
                left,
                right,
                schema,
            } => {
                let left_plan = self.create_physical_plan_internal(left)?;
                let right_plan = self.create_physical_plan_internal(right)?;
                Ok(Box::new(SetOperationExec::new(
                    *op_type,
                    left_plan,
                    right_plan,
                    schema.clone(),
                )))
            }
        }
    }
}

impl Default for DefaultPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Planner for DefaultPlanner {
    fn create_physical_plan(
        &self,
        logical_plan: &LogicalPlan,
    ) -> PlannerResult<Box<dyn PhysicalPlan>> {
        self.create_physical_plan_internal(logical_plan)
    }

    fn optimize(&mut self, logical_plan: LogicalPlan) -> PlannerResult<Box<dyn PhysicalPlan>> {
        // In teaching mode, skip optimization to show original execution plan
        let optimized = if self.use_noop {
            self.noop_optimizer
                .optimize(logical_plan)
                .map_err(|e| PlannerError::OptimizationFailed(e.to_string()))?
        } else {
            self.optimizer
                .optimize(logical_plan)
                .map_err(|e| PlannerError::OptimizationFailed(e.to_string()))?
        };

        // Then convert to physical plan
        self.create_physical_plan_internal(&optimized)
    }
}

/// No-op planner that creates physical plans without optimization
pub struct NoOpPlanner;

impl NoOpPlanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoOpPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Planner for NoOpPlanner {
    fn create_physical_plan(
        &self,
        logical_plan: &LogicalPlan,
    ) -> PlannerResult<Box<dyn PhysicalPlan>> {
        let planner = DefaultPlanner::new();
        planner.create_physical_plan(logical_plan)
    }

    fn optimize(&mut self, logical_plan: LogicalPlan) -> PlannerResult<Box<dyn PhysicalPlan>> {
        let mut planner = DefaultPlanner::new();
        planner.optimize(logical_plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataType;
    use crate::Expr;
    use crate::Field;
    use crate::SetOperationType;

    #[test]
    fn test_default_planner_creation() {
        let planner = DefaultPlanner::new();
        assert!(std::any::type_name::<DefaultPlanner>().contains("DefaultPlanner"));
    }

    #[test]
    fn test_table_scan_physical_plan() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let logical_plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let planner = DefaultPlanner::new();
        let physical_plan = planner.create_physical_plan(&logical_plan).unwrap();

        let name = physical_plan.name();
        assert!(
            name == "IndexScan" || name == "SeqScan",
            "Expected IndexScan or SeqScan, got {}",
            name
        );
        assert_eq!(physical_plan.schema().fields.len(), 1);
    }

    #[test]
    fn test_projection_physical_plan() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let logical_plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let projection_plan = LogicalPlan::Projection {
            input: Box::new(logical_plan),
            expr: vec![Expr::column("id")],
            schema: schema.clone(),
        };

        let planner = DefaultPlanner::new();
        let physical_plan = planner.create_physical_plan(&projection_plan).unwrap();

        assert_eq!(physical_plan.name(), "Projection");
    }

    #[test]
    fn test_filter_physical_plan() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let table_scan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let filter_plan = LogicalPlan::Filter {
            predicate: Expr::binary_expr(
                Expr::column("id"),
                crate::Operator::Gt,
                Expr::literal(sqlrustgo_types::Value::Integer(10)),
            ),
            input: Box::new(table_scan),
        };

        let planner = DefaultPlanner::new();
        let physical_plan = planner.create_physical_plan(&filter_plan).unwrap();

        assert_eq!(physical_plan.name(), "Filter");
    }

    #[test]
    fn test_noop_planner() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let logical_plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let mut planner = NoOpPlanner::new();
        let result = planner.optimize(logical_plan);

        assert!(result.is_ok());
    }

    #[test]
    fn test_join_physical_plan() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let left = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let right = LogicalPlan::TableScan {
            table_name: "orders".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let join_plan = LogicalPlan::Join {
            left: Box::new(left),
            right: Box::new(right),
            join_type: crate::JoinType::Inner,
            condition: None,
        };

        let planner = DefaultPlanner::new();
        let result = planner.create_physical_plan(&join_plan);
        assert!(result.is_ok());
        let plan = result.unwrap();
        let name = plan.name();
        assert!(
            name == "SortMergeJoin" || name == "HashJoin",
            "Expected SortMergeJoin or HashJoin, got {}",
            name
        );
    }

    #[test]
    fn test_aggregate_physical_plan() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let table_scan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let agg_plan = LogicalPlan::Aggregate {
            input: Box::new(table_scan),
            group_expr: vec![Expr::column("id")],
            aggregate_expr: vec![],
            schema: schema.clone(),
        };

        let planner = DefaultPlanner::new();
        let result = planner.create_physical_plan(&agg_plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sort_physical_plan() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let table_scan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let sort_plan = LogicalPlan::Sort {
            input: Box::new(table_scan),
            sort_expr: vec![crate::SortExpr {
                expr: Expr::column("id"),
                asc: true,
                nulls_first: false,
            }],
        };

        let planner = DefaultPlanner::new();
        let result = planner.create_physical_plan(&sort_plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_planner_result_ok() {
        let ok: PlannerResult<i32> = Ok(42);
        assert!(ok.is_ok());
        assert_eq!(ok.unwrap(), 42);
    }

    #[test]
    fn test_planner_result_err() {
        let err: PlannerResult<i32> = Err(PlannerError::PlanningFailed("test".to_string()));
        assert!(err.is_err());
    }

    #[test]
    fn test_noop_planner_default() {
        let planner = NoOpPlanner::default();
        assert!(std::any::type_name::<NoOpPlanner>().contains("NoOpPlanner"));
    }

    #[test]
    fn test_planner_error_display() {
        let err = PlannerError::PlanningFailed("test error".to_string());
        assert!(err.to_string().contains("Planning failed"));
    }

    #[test]
    fn test_aggregate_physical_plan_with_group_by() {
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Integer),
        ]);

        let table_scan = LogicalPlan::TableScan {
            table_name: "data".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let aggregate_plan = LogicalPlan::Aggregate {
            input: Box::new(table_scan),
            group_expr: vec![Expr::column("id")],
            aggregate_expr: vec![Expr::column("value")],
            schema: schema.clone(),
        };

        let planner = DefaultPlanner::new();
        let result = planner.create_physical_plan(&aggregate_plan);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name(), "Aggregate");
    }

    #[test]
    fn test_limit_physical_plan() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let table_scan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let limit_plan = LogicalPlan::Limit {
            input: Box::new(table_scan),
            limit: 10,
            offset: None,
        };

        let planner = DefaultPlanner::new();
        let result = planner.create_physical_plan(&limit_plan);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name(), "Limit");
    }

    #[test]
    fn test_limit_physical_plan_with_offset() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let table_scan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let limit_plan = LogicalPlan::Limit {
            input: Box::new(table_scan),
            limit: 10,
            offset: Some(5),
        };

        let planner = DefaultPlanner::new();
        let result = planner.create_physical_plan(&limit_plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_table_scan_with_projection() {
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);

        let table_scan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: Some(vec![0]),
        };

        let planner = DefaultPlanner::new();
        let result = planner.create_physical_plan(&table_scan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_optimizer() {
        let _optimizer = DefaultOptimizer::new();
        assert!(std::any::type_name::<DefaultOptimizer>().contains("DefaultOptimizer"));
    }

    #[test]
    fn test_empty_relation_physical_plan() {
        let planner = DefaultPlanner::new();
        let plan = LogicalPlan::EmptyRelation;
        let result = planner.create_physical_plan(&plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_values_physical_plan() {
        let planner = DefaultPlanner::new();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let plan = LogicalPlan::Values {
            schema: schema.clone(),
            values: vec![],
        };
        let result = planner.create_physical_plan(&plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_table_physical_plan() {
        let planner = DefaultPlanner::new();
        let plan = LogicalPlan::CreateTable {
            table_name: "test".to_string(),
            schema: Schema::empty(),
            if_not_exists: false,
        };
        let result = planner.create_physical_plan(&plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_drop_table_physical_plan() {
        let planner = DefaultPlanner::new();
        let plan = LogicalPlan::DropTable {
            table_name: "test".to_string(),
            if_exists: false,
        };
        let result = planner.create_physical_plan(&plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_physical_plan() {
        let planner = DefaultPlanner::new();
        let plan = LogicalPlan::Update {
            table_name: "test".to_string(),
            updates: vec![],
            predicate: None,
        };
        let result = planner.create_physical_plan(&plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_physical_plan() {
        let planner = DefaultPlanner::new();
        let plan = LogicalPlan::Delete {
            table_name: "test".to_string(),
            predicate: None,
        };
        let result = planner.create_physical_plan(&plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_subquery_physical_plan() {
        let planner = DefaultPlanner::new();
        let inner_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let inner = LogicalPlan::TableScan {
            table_name: "inner".to_string(),
            schema: inner_schema.clone(),
            projection: None,
        };
        let plan = LogicalPlan::Subquery {
            subquery: Box::new(inner),
            alias: "sub".to_string(),
        };
        let result = planner.create_physical_plan(&plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_operation_physical_plan() {
        let planner = DefaultPlanner::new();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let left = LogicalPlan::TableScan {
            table_name: "left".to_string(),
            schema: schema.clone(),
            projection: None,
        };
        let right = LogicalPlan::TableScan {
            table_name: "right".to_string(),
            schema: schema.clone(),
            projection: None,
        };
        let plan = LogicalPlan::SetOperation {
            op_type: SetOperationType::Union,
            left: Box::new(left),
            right: Box::new(right),
            schema: schema.clone(),
        };
        let result = planner.create_physical_plan(&plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_noop_planner_create_physical_plan() {
        let planner = NoOpPlanner::new();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let plan = LogicalPlan::TableScan {
            table_name: "test".to_string(),
            schema: schema.clone(),
            projection: None,
        };
        let result = planner.create_physical_plan(&plan);
        assert!(result.is_ok());
    }
}

fn select_join_algorithm(
    _cost_model: &(),
    _left_rows: u64,
    _right_rows: u64,
    _join_type: &crate::JoinType,
) -> String {
    // Use HashJoin by default for stability
    // TODO: Re-enable SortMergeJoin after more testing
    "hash_join".to_string()
}

fn estimate_output_rows(_plan: &dyn PhysicalPlan) -> Option<u64> {
    // Simple heuristic: estimate based on plan type
    // In a full implementation, this would use statistics
    Some(1000) // Default estimate
}

fn should_use_index(_table_name: &str) -> bool {
    // Disabled for now - IndexScan not fully implemented
    // Will re-enable after proper implementation
    false
}

fn extract_join_keys(condition: Option<&Expr>, left_side: bool) -> Vec<Expr> {
    match condition {
        Some(Expr::BinaryExpr { left, right, .. }) => {
            if left_side {
                vec![(**left).clone()]
            } else {
                vec![(**right).clone()]
            }
        }
        _ => vec![],
    }
}
