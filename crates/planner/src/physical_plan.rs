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
