//! Physical Plan Module
//!
//! Defines the physical execution representation of query plans.

#![allow(dead_code)]

use crate::Expr;
use crate::Schema;
use crate::SortExpr;
use sqlrustgo_types::Value;
use std::any::Any;
use std::collections::HashMap;

/// Physical plan trait - common interface for all physical operators
pub trait PhysicalPlan: Send + Sync {
    /// Get the schema of this physical plan
    fn schema(&self) -> &Schema;

    /// Get children of this plan
    fn children(&self) -> Vec<&dyn PhysicalPlan>;

    /// Get the name of this plan node
    fn name(&self) -> &str;

    /// Downcast to concrete type
    fn as_any(&self) -> &dyn Any;

    /// Estimated number of rows output by this plan node
    fn row_count(&self) -> u64 {
        0
    }

    /// Estimated number of pages accessed by this plan node
    fn page_count(&self) -> u64 {
        0
    }
}

/// Sequential scan execution operator
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SeqScanExec {
    table_name: String,
    schema: Schema,
    projection: Option<Vec<usize>>,
    row_count: u64,
    page_count: u64,
}

impl SeqScanExec {
    pub fn new(table_name: String, schema: Schema) -> Self {
        Self {
            table_name,
            schema: schema.clone(),
            projection: None,
            row_count: 0,
            page_count: 0,
        }
    }

    pub fn with_projection(mut self, projection: Vec<usize>) -> Self {
        self.projection = Some(projection);
        self
    }

    pub fn with_stats(mut self, row_count: u64, page_count: u64) -> Self {
        self.row_count = row_count;
        self.page_count = page_count;
        self
    }

    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    pub fn projection(&self) -> Option<&[usize]> {
        self.projection.as_deref()
    }

    pub fn execute(&self) -> Result<Vec<HashMap<String, Value>>, String> {
        Ok(vec![])
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn row_count(&self) -> u64 {
        self.row_count
    }

    fn page_count(&self) -> u64 {
        self.page_count
    }
}

/// Index scan execution operator - uses an index to retrieve rows
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IndexScanExec {
    table_name: String,
    column: String,
    index_name: String,
    schema: Schema,
    projection: Option<Vec<usize>>,
    row_count: u64,
    page_count: u64,
    for_update: bool,
    key_expr: Option<Expr>,
    key_range: Option<(i64, i64)>,
}

impl IndexScanExec {
    pub fn new(table_name: String, column: String, index_name: String, schema: Schema) -> Self {
        Self {
            table_name,
            column,
            index_name,
            schema,
            projection: None,
            row_count: 0,
            page_count: 0,
            for_update: false,
            key_expr: None,
            key_range: None,
        }
    }

    pub fn with_projection(mut self, projection: Vec<usize>) -> Self {
        self.projection = Some(projection);
        self
    }

    pub fn with_stats(mut self, row_count: u64, page_count: u64) -> Self {
        self.row_count = row_count;
        self.page_count = page_count;
        self
    }

    pub fn with_for_update(mut self, for_update: bool) -> Self {
        self.for_update = for_update;
        self
    }

    pub fn with_key_expr(mut self, key_expr: Expr) -> Self {
        self.key_expr = Some(key_expr);
        self
    }

    pub fn with_key_range(mut self, start: i64, end: i64) -> Self {
        self.key_range = Some((start, end));
        self
    }

    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    pub fn column(&self) -> &str {
        &self.column
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }

    pub fn for_update(&self) -> bool {
        self.for_update
    }

    pub fn key_expr(&self) -> Option<&Expr> {
        self.key_expr.as_ref()
    }

    pub fn key_range(&self) -> Option<&(i64, i64)> {
        self.key_range.as_ref()
    }
}

impl PhysicalPlan for IndexScanExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![]
    }

    fn name(&self) -> &str {
        "IndexScan"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn row_count(&self) -> u64 {
        self.row_count
    }

    fn page_count(&self) -> u64 {
        self.page_count
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

    pub fn expr(&self) -> &[Expr] {
        &self.expr
    }

    pub fn input(&self) -> &dyn PhysicalPlan {
        self.input.as_ref()
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

    fn as_any(&self) -> &dyn Any {
        self
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

    pub fn predicate(&self) -> &Expr {
        &self.predicate
    }

    pub fn input(&self) -> &dyn PhysicalPlan {
        self.input.as_ref()
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

    fn as_any(&self) -> &dyn Any {
        self
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

    fn as_any(&self) -> &dyn Any {
        self
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Sort-merge join execution operator
#[allow(dead_code)]
pub struct SortMergeJoinExec {
    left: Box<dyn PhysicalPlan>,
    right: Box<dyn PhysicalPlan>,
    join_type: crate::JoinType,
    condition: Option<Expr>,
    schema: Schema,
    left_sort_exprs: Vec<Expr>,
    right_sort_exprs: Vec<Expr>,
}

impl SortMergeJoinExec {
    pub fn new(
        left: Box<dyn PhysicalPlan>,
        right: Box<dyn PhysicalPlan>,
        join_type: crate::JoinType,
        condition: Option<Expr>,
        schema: Schema,
        left_sort_exprs: Vec<Expr>,
        right_sort_exprs: Vec<Expr>,
    ) -> Self {
        Self {
            left,
            right,
            join_type,
            condition,
            schema,
            left_sort_exprs,
            right_sort_exprs,
        }
    }

    pub fn left(&self) -> &dyn PhysicalPlan {
        self.left.as_ref()
    }

    pub fn right(&self) -> &dyn PhysicalPlan {
        self.right.as_ref()
    }

    pub fn join_type(&self) -> &crate::JoinType {
        &self.join_type
    }

    pub fn condition(&self) -> Option<&Expr> {
        self.condition.as_ref()
    }

    pub fn left_sort_exprs(&self) -> &[Expr] {
        &self.left_sort_exprs
    }

    pub fn right_sort_exprs(&self) -> &[Expr] {
        &self.right_sort_exprs
    }
}

impl PhysicalPlan for SortMergeJoinExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.left.as_ref(), self.right.as_ref()]
    }

    fn name(&self) -> &str {
        "SortMergeJoin"
    }

    fn as_any(&self) -> &dyn Any {
        self
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

    pub fn sort_expr(&self) -> &[crate::SortExpr] {
        &self.sort_expr
    }

    pub fn input(&self) -> &dyn PhysicalPlan {
        self.input.as_ref()
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

    fn as_any(&self) -> &dyn Any {
        self
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

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn offset(&self) -> Option<usize> {
        self.offset
    }

    pub fn input(&self) -> &dyn PhysicalPlan {
        self.input.as_ref()
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Delete execution operator
#[allow(dead_code)]
pub struct DeleteExec {
    table_name: String,
    predicate: Option<Expr>,
    schema: Schema,
}

impl DeleteExec {
    pub fn new(table_name: String, predicate: Option<Expr>) -> Self {
        Self {
            table_name,
            predicate,
            schema: Schema::empty(),
        }
    }

    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    pub fn predicate(&self) -> Option<&Expr> {
        self.predicate.as_ref()
    }
}

impl PhysicalPlan for DeleteExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![]
    }

    fn name(&self) -> &str {
        "Delete"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

use crate::logical_plan::{MergeMatchedClause, MergeNotMatchedClause};

/// Window function execution operator
#[allow(dead_code)]
pub struct WindowExec {
    input: Box<dyn PhysicalPlan>,
    window_exprs: Vec<Expr>,
    schema: Schema,
    input_schema: Schema,
    partition_by: Vec<Expr>,
    order_by: Vec<SortExpr>,
}

impl WindowExec {
    pub fn new(
        input: Box<dyn PhysicalPlan>,
        window_exprs: Vec<Expr>,
        schema: Schema,
        input_schema: Schema,
        partition_by: Vec<Expr>,
        order_by: Vec<SortExpr>,
    ) -> Self {
        Self {
            input,
            window_exprs,
            schema,
            input_schema,
            partition_by,
            order_by,
        }
    }
}

impl PhysicalPlan for WindowExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.input.as_ref()]
    }

    fn name(&self) -> &str {
        "Window"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Merge execution operator
#[allow(dead_code)]
pub struct MergeExec {
    target_table: String,
    source_table: String,
    on_condition: Expr,
    matched_clause: Option<MergeMatchedClause>,
    not_matched_clause: Option<MergeNotMatchedClause>,
    schema: Schema,
}

impl MergeExec {
    pub fn new(
        target_table: String,
        source_table: String,
        on_condition: Expr,
        matched_clause: Option<MergeMatchedClause>,
        not_matched_clause: Option<MergeNotMatchedClause>,
    ) -> Self {
        Self {
            target_table,
            source_table,
            on_condition,
            matched_clause,
            not_matched_clause,
            schema: Schema::empty(),
        }
    }

    pub fn target_table(&self) -> &str {
        &self.target_table
    }

    pub fn source_table(&self) -> &str {
        &self.source_table
    }

    pub fn on_condition(&self) -> &Expr {
        &self.on_condition
    }

    pub fn matched_clause(&self) -> Option<&MergeMatchedClause> {
        self.matched_clause.as_ref()
    }

    pub fn not_matched_clause(&self) -> Option<&MergeNotMatchedClause> {
        self.not_matched_clause.as_ref()
    }
}

impl PhysicalPlan for MergeExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![]
    }

    fn name(&self) -> &str {
        "Merge"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Expr, Field, Schema, SortExpr};
    use sqlrustgo_types::Value;

    #[test]
    fn test_seq_scan_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let scan = SeqScanExec::new("test_table".to_string(), schema.clone());

        assert_eq!(scan.name(), "SeqScan");
        assert_eq!(scan.schema().fields.len(), 1);
        assert!(scan.children().is_empty());
    }

    #[test]
    fn test_seq_scan_exec_with_projection() {
        let schema = Schema::new(vec![
            Field::new("id".to_string(), crate::DataType::Integer),
            Field::new("name".to_string(), crate::DataType::Text),
        ]);
        let scan = SeqScanExec::new("test_table".to_string(), schema).with_projection(vec![0]);

        assert!(scan.schema().fields.len() >= 1);
    }

    #[test]
    fn test_projection_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let proj = ProjectionExec::new(input, vec![Expr::column("id")], schema);

        assert_eq!(proj.name(), "Projection");
        assert_eq!(proj.schema().fields.len(), 1);
        assert!(!proj.children().is_empty());
    }

    #[test]
    fn test_filter_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let predicate = Expr::binary_expr(
            Expr::column("id"),
            crate::Operator::Gt,
            Expr::literal(Value::Integer(10)),
        );
        let filter = FilterExec::new(input, predicate);

        assert_eq!(filter.name(), "Filter");
        assert!(!filter.children().is_empty());
    }

    #[test]
    fn test_aggregate_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let agg = AggregateExec::new(
            input,
            vec![Expr::column("id")],
            vec![Expr::column("id")],
            schema,
        );

        assert_eq!(agg.name(), "Aggregate");
        assert_eq!(agg.schema().fields.len(), 1);
        assert!(!agg.children().is_empty());
    }

    #[test]
    fn test_hash_join_exec() {
        let left_schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let right_schema =
            Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let left = Box::new(SeqScanExec::new(
            "left_table".to_string(),
            left_schema.clone(),
        ));
        let right = Box::new(SeqScanExec::new(
            "right_table".to_string(),
            right_schema.clone(),
        ));

        let join_schema = Schema::new(vec![
            Field::new("id".to_string(), crate::DataType::Integer),
            Field::new("id".to_string(), crate::DataType::Integer),
        ]);
        let join = HashJoinExec::new(left, right, crate::JoinType::Inner, None, join_schema);

        assert_eq!(join.name(), "HashJoin");
        assert!(!join.children().is_empty());
    }

    #[test]
    fn test_sort_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let sort_expr = vec![SortExpr {
            expr: Expr::column("id"),
            asc: true,
            nulls_first: false,
        }];
        let sort = SortExec::new(input, sort_expr);

        assert_eq!(sort.name(), "Sort");
        assert_eq!(sort.schema().fields.len(), 1);
        assert!(!sort.children().is_empty());
    }

    #[test]
    fn test_limit_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let limit = LimitExec::new(input, 10, None);

        assert_eq!(limit.name(), "Limit");
        assert_eq!(limit.schema().fields.len(), 1);
        assert!(!limit.children().is_empty());
    }

    #[test]
    fn test_limit_exec_with_offset() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let limit = LimitExec::new(input, 10, Some(5));

        assert_eq!(limit.name(), "Limit");
        assert!(!limit.children().is_empty());
    }

    #[test]
    fn test_delete_exec() {
        let delete = DeleteExec::new("test_table".to_string(), None);

        assert_eq!(delete.name(), "Delete");
        assert_eq!(delete.table_name(), "test_table");
        assert!(delete.predicate().is_none());
        assert!(delete.children().is_empty());
    }

    #[test]
    fn test_delete_exec_with_predicate() {
        let predicate = Expr::binary_expr(
            Expr::column("id"),
            crate::Operator::Eq,
            Expr::literal(Value::Integer(1)),
        );
        let delete = DeleteExec::new("test_table".to_string(), Some(predicate.clone()));

        assert_eq!(delete.name(), "Delete");
        assert_eq!(delete.table_name(), "test_table");
        assert!(delete.predicate().is_some());
    }

    #[test]
    fn test_seq_scan_exec_accessors() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let scan = SeqScanExec::new("test_table".to_string(), schema.clone());
        assert_eq!(scan.table_name(), "test_table");
        assert!(scan.projection().is_none());

        let scan_with_proj = scan.with_projection(vec![0]);
        assert!(scan_with_proj.projection().is_some());
    }

    #[test]
    fn test_projection_exec_accessors() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let expr = vec![Expr::column("id")];
        let proj = ProjectionExec::new(input, expr.clone(), schema);

        assert_eq!(proj.expr(), expr);
        assert!(proj.input().name() == "SeqScan");
    }

    #[test]
    fn test_filter_exec_accessors() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let predicate = Expr::column("id");
        let filter = FilterExec::new(input, predicate.clone());

        assert!(filter.predicate() == &predicate);
        assert!(filter.input().name() == "SeqScan");
    }

    #[test]
    fn test_sort_exec_accessors() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let sort_expr = vec![SortExpr {
            expr: Expr::column("id"),
            asc: true,
            nulls_first: false,
        }];
        let sort = SortExec::new(input, sort_expr.clone());

        assert_eq!(sort.sort_expr(), sort_expr);
        assert!(sort.input().name() == "SeqScan");
    }

    #[test]
    fn test_limit_exec_accessors() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let limit = LimitExec::new(input, 10, Some(5));

        assert_eq!(limit.limit(), 10);
        assert_eq!(limit.offset(), Some(5));
        assert!(limit.input().name() == "SeqScan");
    }

    #[test]
    fn test_index_scan_exec_basic() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let scan = IndexScanExec::new(
            "test_table".to_string(),
            "id".to_string(),
            "idx_id".to_string(),
            schema.clone(),
        );

        assert_eq!(scan.name(), "IndexScan");
        assert_eq!(scan.table_name(), "test_table");
        assert_eq!(scan.column(), "id");
        assert_eq!(scan.index_name(), "idx_id");
        assert!(!scan.for_update());
        assert!(scan.key_expr().is_none());
        assert!(scan.key_range().is_none());
        assert!(scan.children().is_empty());
    }

    #[test]
    fn test_index_scan_exec_with_projection() {
        let schema = Schema::new(vec![
            Field::new("id".to_string(), crate::DataType::Integer),
            Field::new("name".to_string(), crate::DataType::Text),
        ]);
        let scan = IndexScanExec::new(
            "test_table".to_string(),
            "id".to_string(),
            "idx_id".to_string(),
            schema,
        )
        .with_projection(vec![0, 1]);

        // IndexScanExec doesn't expose projection() method publicly
        // but with_projection chain works - just verify the scan was created
        assert_eq!(scan.name(), "IndexScan");
    }

    #[test]
    fn test_index_scan_exec_with_stats() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let scan = IndexScanExec::new(
            "test_table".to_string(),
            "id".to_string(),
            "idx_id".to_string(),
            schema,
        )
        .with_stats(100, 5);

        assert_eq!(scan.row_count(), 100);
        assert_eq!(scan.page_count(), 5);
    }

    #[test]
    fn test_index_scan_exec_with_for_update() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let scan = IndexScanExec::new(
            "test_table".to_string(),
            "id".to_string(),
            "idx_id".to_string(),
            schema,
        )
        .with_for_update(true);

        assert!(scan.for_update());
    }

    #[test]
    fn test_index_scan_exec_with_key_expr() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let key_expr = Expr::literal(Value::Integer(42));
        let scan = IndexScanExec::new(
            "test_table".to_string(),
            "id".to_string(),
            "idx_id".to_string(),
            schema,
        )
        .with_key_expr(key_expr);

        assert!(scan.key_expr().is_some());
    }

    #[test]
    fn test_index_scan_exec_with_key_range() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let scan = IndexScanExec::new(
            "test_table".to_string(),
            "id".to_string(),
            "idx_id".to_string(),
            schema,
        )
        .with_key_range(10, 100);

        assert!(scan.key_range().is_some());
        let (start, end) = scan.key_range().unwrap();
        assert_eq!(*start, 10);
        assert_eq!(*end, 100);
    }

    #[test]
    fn test_index_scan_exec_clone() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let scan = IndexScanExec::new(
            "test_table".to_string(),
            "id".to_string(),
            "idx_id".to_string(),
            schema,
        )
        .with_stats(100, 5);

        let cloned = scan.clone();
        assert_eq!(cloned.table_name(), "test_table");
        assert_eq!(cloned.row_count(), 100);
    }

    #[test]
    fn test_sort_merge_join_exec() {
        let left_schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let right_schema =
            Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let left = Box::new(SeqScanExec::new(
            "left_table".to_string(),
            left_schema.clone(),
        ));
        let right = Box::new(SeqScanExec::new(
            "right_table".to_string(),
            right_schema.clone(),
        ));

        let join = SortMergeJoinExec::new(
            left,
            right,
            crate::JoinType::Inner,
            Some(Expr::column("id")),
            Schema::empty(),
            vec![Expr::column("id")],
            vec![Expr::column("id")],
        );

        assert_eq!(join.name(), "SortMergeJoin");
        assert_eq!(join.join_type(), &crate::JoinType::Inner);
        assert!(join.condition().is_some());
        assert!(!join.children().is_empty());
    }

    #[test]
    fn test_sort_merge_join_exec_accessors() {
        let left_schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let right_schema =
            Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let left = Box::new(SeqScanExec::new(
            "left_table".to_string(),
            left_schema.clone(),
        ));
        let right = Box::new(SeqScanExec::new(
            "right_table".to_string(),
            right_schema.clone(),
        ));

        let left_exprs = vec![Expr::column("id")];
        let right_exprs = vec![Expr::column("id")];
        let join = SortMergeJoinExec::new(
            left,
            right,
            crate::JoinType::Left,
            None,
            Schema::empty(),
            left_exprs.clone(),
            right_exprs.clone(),
        );

        assert!(join.left().name() == "SeqScan");
        assert!(join.right().name() == "SeqScan");
        assert_eq!(join.join_type(), &crate::JoinType::Left);
        assert!(join.condition().is_none());
        assert_eq!(join.left_sort_exprs(), left_exprs);
        assert_eq!(join.right_sort_exprs(), right_exprs);
    }

    #[test]
    fn test_window_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let window_exprs = vec![Expr::WindowFunction {
            func: crate::WindowFunction::RowNumber,
            args: vec![],
            partition_by: vec![],
            order_by: vec![],
            frame: None,
        }];

        let window = WindowExec::new(input, window_exprs, schema.clone(), schema, vec![], vec![]);

        assert_eq!(window.name(), "Window");
        assert!(!window.children().is_empty());
    }

    #[test]
    fn test_window_exec_accessors() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let window_exprs = vec![Expr::WindowFunction {
            func: crate::WindowFunction::Rank,
            args: vec![],
            partition_by: vec![Expr::column("id")],
            order_by: vec![],
            frame: None,
        }];
        let partition_by = vec![Expr::column("id")];
        let order_by = vec![SortExpr {
            expr: Expr::column("id"),
            asc: true,
            nulls_first: false,
        }];

        let window = WindowExec::new(
            input,
            window_exprs.clone(),
            schema.clone(),
            schema,
            partition_by.clone(),
            order_by.clone(),
        );

        assert_eq!(window.name(), "Window");
        assert_eq!(window.schema().fields.len(), 1);
    }

    #[test]
    fn test_merge_exec() {
        let merge = MergeExec::new(
            "target".to_string(),
            "source".to_string(),
            Expr::column("id"),
            None,
            None,
        );

        assert_eq!(merge.name(), "Merge");
        assert_eq!(merge.target_table(), "target");
        assert_eq!(merge.source_table(), "source");
        assert!(merge.matched_clause().is_none());
        assert!(merge.not_matched_clause().is_none());
        assert!(merge.children().is_empty());
    }

    #[test]
    fn test_merge_exec_with_clauses() {
        use crate::logical_plan::{MergeMatchedClause, MergeNotMatchedClause};
        let matched = MergeMatchedClause {
            update_columns: vec!["y".to_string()],
            update_values: vec![Expr::literal(Value::Integer(1))],
        };
        let not_matched = MergeNotMatchedClause {
            insert_columns: vec!["z".to_string()],
            insert_values: vec![Expr::literal(Value::Integer(2))],
        };

        let merge = MergeExec::new(
            "target".to_string(),
            "source".to_string(),
            Expr::column("id"),
            Some(matched),
            Some(not_matched),
        );

        assert!(merge.matched_clause().is_some());
        assert!(merge.not_matched_clause().is_some());
    }

    #[test]
    fn test_aggregate_exec_accessors() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let group_exprs = vec![Expr::column("id")];
        let agg_exprs = vec![Expr::AggregateFunction {
            func: crate::AggregateFunction::Count,
            args: vec![Expr::Wildcard],
            distinct: false,
        }];

        let agg = AggregateExec::new(input, group_exprs.clone(), agg_exprs.clone(), schema);

        assert_eq!(agg.name(), "Aggregate");
        assert_eq!(agg.schema().fields.len(), 1);
    }

    #[test]
    fn test_hash_join_exec_accessors() {
        let left_schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let right_schema =
            Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let left = Box::new(SeqScanExec::new(
            "left_table".to_string(),
            left_schema.clone(),
        ));
        let right = Box::new(SeqScanExec::new(
            "right_table".to_string(),
            right_schema.clone(),
        ));

        let condition = Some(Expr::binary_expr(
            Expr::column("id"),
            crate::Operator::Eq,
            Expr::column("id"),
        ));
        let join_schema = Schema::new(vec![
            Field::new("id".to_string(), crate::DataType::Integer),
            Field::new("id".to_string(), crate::DataType::Integer),
        ]);
        let join = HashJoinExec::new(
            left,
            right,
            crate::JoinType::Inner,
            condition.clone(),
            join_schema,
        );

        assert_eq!(join.name(), "HashJoin");
        assert!(join.children().len() == 2);
    }

    #[test]
    fn test_seq_scan_exec_execute() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let scan = SeqScanExec::new("test_table".to_string(), schema);

        let result = scan.execute();
        assert!(result.is_ok());
        let rows = result.unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn test_seq_scan_exec_with_stats() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let scan = SeqScanExec::new("test_table".to_string(), schema).with_stats(1000, 10);

        assert_eq!(scan.row_count(), 1000);
        assert_eq!(scan.page_count(), 10);
    }

    #[test]
    fn test_physical_plan_trait_default_methods() {
        use super::PhysicalPlan;
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let scan = SeqScanExec::new("test_table".to_string(), schema);

        assert_eq!(scan.row_count(), 0);
        assert_eq!(scan.page_count(), 0);
    }

    #[test]
    fn test_projection_exec_with_multiple_exprs() {
        let schema = Schema::new(vec![Field::new("a".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let exprs = vec![
            Expr::column("a"),
            Expr::binary_expr(
                Expr::column("a"),
                crate::Operator::Plus,
                Expr::literal(Value::Integer(1)),
            ),
        ];
        let proj = ProjectionExec::new(input, exprs, schema);

        assert_eq!(proj.name(), "Projection");
        assert_eq!(proj.expr().len(), 2);
    }

    #[test]
    fn test_filter_exec_with_complex_predicate() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let predicate = Expr::BinaryExpr {
            left: Box::new(Expr::column("id")),
            op: crate::Operator::Gt,
            right: Box::new(Expr::literal(Value::Integer(5))),
        };
        let filter = FilterExec::new(input, predicate);

        assert_eq!(filter.name(), "Filter");
        assert!(!filter.children().is_empty());
    }

    #[test]
    fn test_sort_exec_multiple_exprs() {
        let schema = Schema::new(vec![Field::new("a".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let sort_exprs = vec![
            SortExpr {
                expr: Expr::column("a"),
                asc: true,
                nulls_first: false,
            },
            SortExpr {
                expr: Expr::column("a"),
                asc: false,
                nulls_first: true,
            },
        ];
        let sort = SortExec::new(input, sort_exprs);

        assert_eq!(sort.name(), "Sort");
        assert_eq!(sort.sort_expr().len(), 2);
    }

    #[test]
    fn test_limit_exec_zero_limit() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let limit = LimitExec::new(input, 0, None);

        assert_eq!(limit.limit(), 0);
        assert!(limit.offset().is_none());
    }

    #[test]
    fn test_delete_exec_schema() {
        let delete = DeleteExec::new("test_table".to_string(), None);

        assert_eq!(delete.schema().fields.len(), 0);
        assert!(delete.children().is_empty());
    }

    #[test]
    fn test_delete_exec_with_complex_predicate() {
        let predicate = Expr::BinaryExpr {
            left: Box::new(Expr::column("id")),
            op: crate::Operator::Eq,
            right: Box::new(Expr::literal(Value::Integer(1))),
        };
        let delete = DeleteExec::new("test_table".to_string(), Some(predicate));

        assert_eq!(delete.name(), "Delete");
        assert!(delete.predicate().is_some());
    }
}
