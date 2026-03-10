//! Logical Plan Module
//!
//! Defines the logical representation of query execution plans.

use crate::Expr;
use crate::Schema;
use sqlrustgo_types::Value;

/// Logical plan node types
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalPlan {
    /// Scan a table
    TableScan {
        table_name: String,
        schema: Schema,
        projection: Option<Vec<usize>>,
    },
    /// Projection (SELECT columns)
    Projection {
        input: Box<LogicalPlan>,
        expr: Vec<Expr>,
        schema: Schema,
    },
    /// Filter (WHERE clause)
    Filter {
        predicate: Expr,
        input: Box<LogicalPlan>,
    },
    /// Aggregate (GROUP BY)
    Aggregate {
        input: Box<LogicalPlan>,
        group_expr: Vec<Expr>,
        aggregate_expr: Vec<Expr>,
        schema: Schema,
    },
    /// Join operation
    Join {
        left: Box<LogicalPlan>,
        right: Box<LogicalPlan>,
        join_type: crate::JoinType,
        condition: Option<Expr>,
    },
    /// Sort (ORDER BY)
    Sort {
        input: Box<LogicalPlan>,
        sort_expr: Vec<crate::SortExpr>,
    },
    /// Limit
    Limit {
        input: Box<LogicalPlan>,
        limit: usize,
        offset: Option<usize>,
    },
    /// VALUES (for INSERT)
    Values {
        values: Vec<Vec<Value>>,
        schema: Schema,
    },
    /// Empty relation
    EmptyRelation,
    /// Subquery
    Subquery {
        subquery: Box<LogicalPlan>,
        alias: String,
    },
    /// Union
    Union {
        left: Box<LogicalPlan>,
        right: Box<LogicalPlan>,
    },
    /// Update statement
    Update {
        table_name: String,
        updates: Vec<(String, Expr)>,
        predicate: Option<Expr>,
    },
    /// Delete statement
    Delete {
        table_name: String,
        predicate: Option<Expr>,
    },
    /// Create table
    CreateTable {
        table_name: String,
        schema: Schema,
        if_not_exists: bool,
    },
    /// Drop table
    DropTable { table_name: String, if_exists: bool },
}

impl LogicalPlan {
    /// Get the schema of this plan (cloned)
    pub fn schema(&self) -> Schema {
        match self {
            LogicalPlan::TableScan { schema, .. } => schema.clone(),
            LogicalPlan::Projection { schema, .. } => schema.clone(),
            LogicalPlan::Aggregate { schema, .. } => schema.clone(),
            LogicalPlan::Values { schema, .. } => schema.clone(),
            LogicalPlan::CreateTable { schema, .. } => schema.clone(),
            LogicalPlan::EmptyRelation => Schema::empty(),
            LogicalPlan::Join { .. } => Schema::empty(),
            LogicalPlan::Filter { input, .. } => input.schema(),
            LogicalPlan::Sort { input, .. } => input.schema(),
            LogicalPlan::Limit { input, .. } => input.schema(),
            LogicalPlan::Subquery { subquery, .. } => subquery.schema(),
            LogicalPlan::Union { left, .. } => left.schema(),
            LogicalPlan::Update { .. } => Schema::empty(),
            LogicalPlan::Delete { .. } => Schema::empty(),
            LogicalPlan::DropTable { .. } => Schema::empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataType, Field};

    #[test]
    fn test_logical_plan_schema_table_scan() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };
        assert_eq!(plan.schema(), schema);
    }

    #[test]
    fn test_logical_plan_schema_projection() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };
        let plan = LogicalPlan::Projection {
            input: Box::new(input),
            expr: vec![],
            schema: schema.clone(),
        };
        assert_eq!(plan.schema(), schema);
    }

    #[test]
    fn test_logical_plan_schema_filter() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: schema.clone(),
            projection: None,
        };
        let plan = LogicalPlan::Filter {
            predicate: Expr::literal(Value::Null),
            input: Box::new(input),
        };
        assert_eq!(plan.schema(), schema);
    }

    #[test]
    fn test_logical_plan_schema_empty_relation() {
        let plan = LogicalPlan::EmptyRelation;
        assert_eq!(plan.schema(), Schema::empty());
    }

    #[test]
    fn test_logical_plan_schema_values() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let plan = LogicalPlan::Values {
            values: vec![],
            schema: schema.clone(),
        };
        assert_eq!(plan.schema(), schema);
    }

    #[test]
    fn test_logical_plan_schema_create_table() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let plan = LogicalPlan::CreateTable {
            table_name: "users".to_string(),
            schema: schema.clone(),
            if_not_exists: true,
        };
        assert_eq!(plan.schema(), schema);
    }

    #[test]
    fn test_logical_plan_schema_drop_table() {
        let plan = LogicalPlan::DropTable {
            table_name: "users".to_string(),
            if_exists: false,
        };
        assert_eq!(plan.schema(), Schema::empty());
    }

    #[test]
    fn test_logical_plan_schema_update() {
        let plan = LogicalPlan::Update {
            table_name: "users".to_string(),
            updates: vec![],
            predicate: None,
        };
        assert_eq!(plan.schema(), Schema::empty());
    }

    #[test]
    fn test_logical_plan_schema_delete() {
        let plan = LogicalPlan::Delete {
            table_name: "users".to_string(),
            predicate: None,
        };
        assert_eq!(plan.schema(), Schema::empty());
    }

    #[test]
    fn test_logical_plan_schema_join() {
        let plan = LogicalPlan::Join {
            left: Box::new(LogicalPlan::EmptyRelation),
            right: Box::new(LogicalPlan::EmptyRelation),
            join_type: crate::JoinType::Inner,
            condition: None,
        };
        assert_eq!(plan.schema(), Schema::empty());
    }

    #[test]
    fn test_logical_plan_debug() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema,
            projection: None,
        };
        let debug = format!("{:?}", plan);
        assert!(debug.contains("TableScan"));
    }
}
