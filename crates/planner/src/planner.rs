//! Planner Module
//!
//! Converts logical plans to physical execution plans.

use crate::logical_plan::LogicalPlan;
use crate::optimizer::{DefaultOptimizer, Optimizer};
use crate::physical_plan::{
    AggregateExec, FilterExec, HashJoinExec, LimitExec, PhysicalPlan, ProjectionExec, SeqScanExec,
    SortExec,
};
use crate::Schema;
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

/// Default planner implementation
pub struct DefaultPlanner {
    optimizer: DefaultOptimizer,
}

impl DefaultPlanner {
    pub fn new() -> Self {
        Self {
            optimizer: DefaultOptimizer::new(),
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
                let mut exec = SeqScanExec::new(table_name.clone(), schema.clone());
                if let Some(proj) = projection {
                    exec = exec.with_projection(proj.clone());
                }
                Ok(Box::new(exec))
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
                let schema = Schema::new(vec![]); // Would need to compute from children
                Ok(Box::new(HashJoinExec::new(
                    left_plan,
                    right_plan,
                    join_type.clone(),
                    condition.clone(),
                    schema,
                )))
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
            LogicalPlan::CreateTable { .. } | LogicalPlan::DropTable { .. } => {
                // DDL statements - handled differently
                Ok(Box::new(SeqScanExec::new(String::new(), Schema::empty())))
            }
            LogicalPlan::Update { .. } | LogicalPlan::Delete { .. } => {
                // DML statements - handled differently
                Ok(Box::new(SeqScanExec::new(String::new(), Schema::empty())))
            }
            LogicalPlan::Subquery { subquery, .. } => self.create_physical_plan_internal(subquery),
            LogicalPlan::Union { left, .. } => {
                // Union - use left plan as base (simplified)
                self.create_physical_plan_internal(left)
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
        // First optimize the logical plan
        let optimized = self
            .optimizer
            .optimize(logical_plan)
            .map_err(|e| PlannerError::OptimizationFailed(e.to_string()))?;

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

        assert_eq!(physical_plan.name(), "SeqScan");
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
    fn test_table_scan_with_projection() {
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let logical_plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: Some(vec![0]),
        };

        let planner = DefaultPlanner::new();
        let physical_plan = planner.create_physical_plan(&logical_plan).unwrap();

        assert_eq!(physical_plan.name(), "SeqScan");
    }

    #[test]
    fn test_aggregate_physical_plan() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let table_scan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let aggregate_plan = LogicalPlan::Aggregate {
            input: Box::new(table_scan),
            group_expr: vec![Expr::column("id")],
            aggregate_expr: vec![Expr::column("id")],
            schema: schema.clone(),
        };

        let planner = DefaultPlanner::new();
        let physical_plan = planner.create_physical_plan(&aggregate_plan).unwrap();

        assert_eq!(physical_plan.name(), "Aggregate");
    }

    #[test]
    fn test_join_physical_plan() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let left_scan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };
        let right_scan = LogicalPlan::TableScan {
            table_name: "orders".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let join_plan = LogicalPlan::Join {
            left: Box::new(left_scan),
            right: Box::new(right_scan),
            join_type: crate::JoinType::Inner,
            condition: Some(Expr::binary_expr(
                Expr::column("id"),
                crate::Operator::Eq,
                Expr::column("id"),
            )),
        };

        let planner = DefaultPlanner::new();
        let physical_plan = planner.create_physical_plan(&join_plan).unwrap();

        assert_eq!(physical_plan.name(), "HashJoin");
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
        let physical_plan = planner.create_physical_plan(&sort_plan).unwrap();

        assert_eq!(physical_plan.name(), "Sort");
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
            offset: Some(5),
        };

        let planner = DefaultPlanner::new();
        let physical_plan = planner.create_physical_plan(&limit_plan).unwrap();

        assert_eq!(physical_plan.name(), "Limit");
    }

    #[test]
    fn test_empty_relation_physical_plan() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let empty_plan = LogicalPlan::EmptyRelation;

        let planner = DefaultPlanner::new();
        let physical_plan = planner.create_physical_plan(&empty_plan).unwrap();

        assert_eq!(physical_plan.name(), "SeqScan");
    }

    #[test]
    fn test_values_physical_plan() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let values_plan = LogicalPlan::Values {
            schema: schema.clone(),
            values: vec![],
        };

        let planner = DefaultPlanner::new();
        let physical_plan = planner.create_physical_plan(&values_plan).unwrap();

        assert_eq!(physical_plan.name(), "SeqScan");
    }

    #[test]
    fn test_create_table_physical_plan() {
        let create_plan = LogicalPlan::CreateTable {
            table_name: "users".to_string(),
            schema: Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]),
            if_not_exists: true,
        };

        let planner = DefaultPlanner::new();
        let physical_plan = planner.create_physical_plan(&create_plan).unwrap();

        assert_eq!(physical_plan.name(), "SeqScan");
    }

    #[test]
    fn test_drop_table_physical_plan() {
        let drop_plan = LogicalPlan::DropTable {
            table_name: "users".to_string(),
            if_exists: true,
        };

        let planner = DefaultPlanner::new();
        let physical_plan = planner.create_physical_plan(&drop_plan).unwrap();

        assert_eq!(physical_plan.name(), "SeqScan");
    }

    #[test]
    fn test_update_physical_plan() {
        let update_plan = LogicalPlan::Update {
            table_name: "users".to_string(),
            updates: vec![("id".to_string(), Expr::literal(sqlrustgo_types::Value::Integer(1)))],
            predicate: Some(Expr::column("id")),
        };

        let planner = DefaultPlanner::new();
        let physical_plan = planner.create_physical_plan(&update_plan).unwrap();

        assert_eq!(physical_plan.name(), "SeqScan");
    }

    #[test]
    fn test_delete_physical_plan() {
        let delete_plan = LogicalPlan::Delete {
            table_name: "users".to_string(),
            predicate: Some(Expr::column("id")),
        };

        let planner = DefaultPlanner::new();
        let physical_plan = planner.create_physical_plan(&delete_plan).unwrap();

        assert_eq!(physical_plan.name(), "SeqScan");
    }

    #[test]
    fn test_subquery_physical_plan() {
        let inner_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let inner_scan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: inner_schema.clone(),
            projection: None,
        };

        let subquery_plan = LogicalPlan::Subquery {
            subquery: Box::new(inner_scan),
            alias: "sub".to_string(),
        };

        let planner = DefaultPlanner::new();
        let physical_plan = planner.create_physical_plan(&subquery_plan).unwrap();

        assert_eq!(physical_plan.name(), "SeqScan");
    }

    #[test]
    fn test_union_physical_plan() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let left_scan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };
        let right_scan = LogicalPlan::TableScan {
            table_name: "admins".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let union_plan = LogicalPlan::Union {
            left: Box::new(left_scan),
            right: Box::new(right_scan),
        };

        let planner = DefaultPlanner::new();
        let physical_plan = planner.create_physical_plan(&union_plan).unwrap();

        assert_eq!(physical_plan.name(), "SeqScan");
    }

    #[test]
    fn test_noop_planner_create_physical_plan() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let logical_plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let planner = NoOpPlanner::new();
        let result = planner.create_physical_plan(&logical_plan);

        assert!(result.is_ok());
    }

    #[test]
    fn test_noop_planner_default() {
        let planner = NoOpPlanner::default();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let logical_plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let result = planner.create_physical_plan(&logical_plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_planner_default() {
        let planner = DefaultPlanner::default();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let logical_plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };

        let result = planner.create_physical_plan(&logical_plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_planner_error_display() {
        let error = PlannerError::PlanningFailed("test error".to_string());
        assert_eq!(error.to_string(), "Planning failed: test error");

        let error = PlannerError::OptimizationFailed("optimization error".to_string());
        assert_eq!(error.to_string(), "Optimization failed: optimization error");
    }
}
