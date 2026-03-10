//! Physical Plan Module
//!
//! Defines the physical execution representation of query plans.

#![allow(dead_code)]

use crate::Expr;
use crate::Schema;

/// Physical plan trait - common interface for all physical operators
pub trait PhysicalPlan: Send + Sync {
    /// Get the schema of this physical plan
    fn schema(&self) -> &Schema;

    /// Get children of this plan
    fn children(&self) -> Vec<&dyn PhysicalPlan>;

    /// Get the name of this plan node
    fn name(&self) -> &str;
}

/// Sequential scan execution operator
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SeqScanExec {
    table_name: String,
    schema: Schema,
    projection: Option<Vec<usize>>,
}

impl SeqScanExec {
    pub fn new(table_name: String, schema: Schema) -> Self {
        Self {
            table_name,
            schema: schema.clone(),
            projection: None,
        }
    }

    pub fn with_projection(mut self, projection: Vec<usize>) -> Self {
        self.projection = Some(projection);
        self
    }
}

impl PhysicalPlan for SeqScanExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![]
    }

    fn name(&self) -> &str {
        "SeqScan"
    }
}

/// Projection execution operator
#[allow(dead_code)]
pub struct ProjectionExec {
    input: Box<dyn PhysicalPlan>,
    expr: Vec<Expr>,
    schema: Schema,
}

impl ProjectionExec {
    pub fn new(input: Box<dyn PhysicalPlan>, expr: Vec<Expr>, schema: Schema) -> Self {
        Self {
            input,
            expr,
            schema,
        }
    }
}

impl PhysicalPlan for ProjectionExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.input.as_ref()]
    }

    fn name(&self) -> &str {
        "Projection"
    }
}

/// Filter execution operator
#[allow(dead_code)]
pub struct FilterExec {
    input: Box<dyn PhysicalPlan>,
    predicate: Expr,
}

impl FilterExec {
    pub fn new(input: Box<dyn PhysicalPlan>, predicate: Expr) -> Self {
        Self { input, predicate }
    }
}

impl PhysicalPlan for FilterExec {
    fn schema(&self) -> &Schema {
        self.input.schema()
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.input.as_ref()]
    }

    fn name(&self) -> &str {
        "Filter"
    }
}

/// Aggregate execution operator
#[allow(dead_code)]
pub struct AggregateExec {
    input: Box<dyn PhysicalPlan>,
    group_expr: Vec<Expr>,
    aggregate_expr: Vec<Expr>,
    schema: Schema,
}

impl AggregateExec {
    pub fn new(
        input: Box<dyn PhysicalPlan>,
        group_expr: Vec<Expr>,
        aggregate_expr: Vec<Expr>,
        schema: Schema,
    ) -> Self {
        Self {
            input,
            group_expr,
            aggregate_expr,
            schema,
        }
    }
}

impl PhysicalPlan for AggregateExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.input.as_ref()]
    }

    fn name(&self) -> &str {
        "Aggregate"
    }
}

/// Hash join execution operator
#[allow(dead_code)]
pub struct HashJoinExec {
    left: Box<dyn PhysicalPlan>,
    right: Box<dyn PhysicalPlan>,
    join_type: crate::JoinType,
    condition: Option<Expr>,
    schema: Schema,
}

impl HashJoinExec {
    pub fn new(
        left: Box<dyn PhysicalPlan>,
        right: Box<dyn PhysicalPlan>,
        join_type: crate::JoinType,
        condition: Option<Expr>,
        schema: Schema,
    ) -> Self {
        Self {
            left,
            right,
            join_type,
            condition,
            schema,
        }
    }
}

impl PhysicalPlan for HashJoinExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.left.as_ref(), self.right.as_ref()]
    }

    fn name(&self) -> &str {
        "HashJoin"
    }
}

/// Sort execution operator
#[allow(dead_code)]
pub struct SortExec {
    input: Box<dyn PhysicalPlan>,
    sort_expr: Vec<crate::SortExpr>,
}

impl SortExec {
    pub fn new(input: Box<dyn PhysicalPlan>, sort_expr: Vec<crate::SortExpr>) -> Self {
        Self { input, sort_expr }
    }
}

impl PhysicalPlan for SortExec {
    fn schema(&self) -> &Schema {
        self.input.schema()
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.input.as_ref()]
    }

    fn name(&self) -> &str {
        "Sort"
    }
}

/// Limit execution operator
#[allow(dead_code)]
pub struct LimitExec {
    input: Box<dyn PhysicalPlan>,
    limit: usize,
    offset: Option<usize>,
}

impl LimitExec {
    pub fn new(input: Box<dyn PhysicalPlan>, limit: usize, offset: Option<usize>) -> Self {
        Self {
            input,
            limit,
            offset,
        }
    }
}

impl PhysicalPlan for LimitExec {
    fn schema(&self) -> &Schema {
        self.input.schema()
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.input.as_ref()]
    }

    fn name(&self) -> &str {
        "Limit"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataType, Expr, Field};

    #[test]
    fn test_seq_scan_exec_new() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let exec = SeqScanExec::new("users".to_string(), schema.clone());
        assert_eq!(exec.schema(), &schema);
        assert_eq!(exec.name(), "SeqScan");
    }

    #[test]
    fn test_seq_scan_exec_with_projection() {
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let exec = SeqScanExec::new("users".to_string(), schema).with_projection(vec![0]);
        assert!(exec.projection.is_some());
    }

    #[test]
    fn test_seq_scan_children() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let exec = SeqScanExec::new("users".to_string(), schema);
        assert!(exec.children().is_empty());
    }

    #[test]
    fn test_projection_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = ProjectionExec::new(Box::new(child), vec![], schema);
        assert_eq!(exec.name(), "Projection");
    }

    #[test]
    fn test_filter_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let predicate = Expr::column("id");
        let exec = FilterExec::new(Box::new(child), predicate);
        assert_eq!(exec.name(), "Filter");
    }

    #[test]
    fn test_aggregate_exec() {
        let schema = Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = AggregateExec::new(Box::new(child), vec![], vec![], schema);
        assert_eq!(exec.name(), "Aggregate");
    }

    #[test]
    fn test_hash_join_exec() {
        let schema = Schema::new(vec![]);
        let left = SeqScanExec::new("users".to_string(), schema.clone());
        let right = SeqScanExec::new("orders".to_string(), schema.clone());
        let exec = HashJoinExec::new(
            Box::new(left),
            Box::new(right),
            crate::JoinType::Inner,
            None,
            schema,
        );
        assert_eq!(exec.name(), "HashJoin");
    }

    #[test]
    fn test_physical_plan_send_sync() {
        fn _check<T: Send + Sync>() {}
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let exec = SeqScanExec::new("users".to_string(), schema);
        _check::<Box<dyn PhysicalPlan>>();
        _check::<SeqScanExec>();
    }

    #[test]
    fn test_sort_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let sort_expr = vec![crate::SortExpr {
            expr: Expr::column("id"),
            asc: true,
            nulls_first: true,
        }];
        let exec = SortExec::new(Box::new(child), sort_expr);
        assert_eq!(exec.name(), "Sort");
        assert_eq!(exec.schema(), &schema);
    }

    #[test]
    fn test_sort_exec_children() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema);
        let exec = SortExec::new(Box::new(child), vec![]);
        assert!(!exec.children().is_empty());
    }

    #[test]
    fn test_limit_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = LimitExec::new(Box::new(child), 10, Some(5));
        assert_eq!(exec.name(), "Limit");
        assert_eq!(exec.schema(), &schema);
    }

    #[test]
    fn test_limit_exec_no_offset() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema);
        let exec = LimitExec::new(Box::new(child), 100, None);
        assert_eq!(exec.name(), "Limit");
    }

    #[test]
    fn test_limit_exec_children() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema);
        let exec = LimitExec::new(Box::new(child), 10, None);
        assert!(!exec.children().is_empty());
    }

    #[test]
    fn test_filter_exec_children() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let predicate = Expr::column("id");
        let exec = FilterExec::new(Box::new(child), predicate.clone());
        let children = exec.children();
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn test_aggregate_exec_children() {
        let schema = Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = AggregateExec::new(Box::new(child), vec![], vec![], schema);
        assert!(!exec.children().is_empty());
    }

    #[test]
    fn test_hash_join_exec_left_right() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let left = SeqScanExec::new("users".to_string(), schema.clone());
        let right = SeqScanExec::new("orders".to_string(), schema.clone());
        let exec = HashJoinExec::new(
            Box::new(left),
            Box::new(right),
            crate::JoinType::Left,
            None,
            schema,
        );
        let children = exec.children();
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_hash_join_exec_right() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let left = SeqScanExec::new("users".to_string(), schema.clone());
        let right = SeqScanExec::new("orders".to_string(), schema.clone());
        let exec = HashJoinExec::new(
            Box::new(left),
            Box::new(right),
            crate::JoinType::Right,
            None,
            schema,
        );
        assert_eq!(exec.name(), "HashJoin");
    }

    #[test]
    fn test_hash_join_exec_full() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let left = SeqScanExec::new("users".to_string(), schema.clone());
        let right = SeqScanExec::new("orders".to_string(), schema.clone());
        let exec = HashJoinExec::new(
            Box::new(left),
            Box::new(right),
            crate::JoinType::Full,
            None,
            schema,
        );
        assert_eq!(exec.name(), "HashJoin");
    }

    #[test]
    fn test_projection_exec_children() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = ProjectionExec::new(Box::new(child), vec![], schema);
        assert!(!exec.children().is_empty());
    }

    #[test]
    fn test_physical_plan_debug() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let exec = SeqScanExec::new("users".to_string(), schema);
        let debug_str = format!("{:?}", exec);
        assert!(debug_str.contains("SeqScanExec"));
    }
}
