//! Physical Plan Module
//!
//! Defines the physical execution representation of query plans.

#![allow(dead_code)]

use crate::AggregateFunction;
use crate::Expr;
use crate::Schema;
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

    /// Execute this physical plan and return results
    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        Ok(vec![])
    }

    /// Get table name for scan operators
    fn table_name(&self) -> &str {
        ""
    }

    /// Get as Any for downcasting
    fn as_any(&self) -> &dyn Any;
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

    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    pub fn projection(&self) -> Option<&Vec<usize>> {
        self.projection.as_ref()
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

    fn table_name(&self) -> &str {
        &self.table_name
    }

    fn as_any(&self) -> &dyn Any {
        self
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

    pub fn expr(&self) -> &Vec<Expr> {
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

    fn evaluate_expr(&self, expr: &Expr, row: &[Value], schema: &Schema) -> Value {
        match expr {
            Expr::Column(col) => {
                if let Some(idx) = schema.field_index(&col.name) {
                    row.get(idx).cloned().unwrap_or(Value::Null)
                } else {
                    Value::Null
                }
            }
            Expr::Literal(val) => val.clone(),
            Expr::Wildcard => Value::Integer(row.len() as i64),
            _ => Value::Null,
        }
    }

    fn compute_aggregate(&self, func: &AggregateFunction, values: &[Value]) -> Value {
        match func {
            AggregateFunction::Count => Value::Integer(values.len() as i64),
            AggregateFunction::Sum => {
                let mut sum: i64 = 0;
                for v in values {
                    if let Value::Integer(n) = v {
                        sum += n;
                    }
                }
                Value::Integer(sum)
            }
            AggregateFunction::Avg => {
                let mut sum: i64 = 0;
                let mut count = 0;
                for v in values {
                    if let Value::Integer(n) = v {
                        sum += n;
                        count += 1;
                    }
                }
                if count > 0 {
                    Value::Integer(sum / count as i64)
                } else {
                    Value::Null
                }
            }
            AggregateFunction::Min => {
                let mut min_val: Option<i64> = None;
                for v in values {
                    if let Value::Integer(n) = v {
                        match min_val {
                            Some(m) if *n < m => min_val = Some(*n),
                            None => min_val = Some(*n),
                            _ => {}
                        }
                    }
                }
                min_val.map(Value::Integer).unwrap_or(Value::Null)
            }
            AggregateFunction::Max => {
                let mut max_val: Option<i64> = None;
                for v in values {
                    if let Value::Integer(n) = v {
                        match max_val {
                            Some(m) if *n > m => max_val = Some(*n),
                            None => max_val = Some(*n),
                            _ => {}
                        }
                    }
                }
                max_val.map(Value::Integer).unwrap_or(Value::Null)
            }
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

    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        let input_rows = self.input.execute()?;

        if self.group_expr.is_empty() {
            let mut results = vec![];
            let mut agg_results = vec![];

            for agg_expr in &self.aggregate_expr {
                if let Expr::AggregateFunction { func, args, .. } = agg_expr {
                    let values: Vec<Value> = input_rows
                        .iter()
                        .map(|row| {
                            self.evaluate_expr(
                                args.first().unwrap_or(&Expr::Wildcard),
                                row,
                                self.input.schema(),
                            )
                        })
                        .collect();
                    let result = self.compute_aggregate(func, &values);
                    agg_results.push(result);
                }
            }

            if !agg_results.is_empty() {
                results.push(agg_results);
            }

            Ok(results)
        } else {
            let mut groups: HashMap<Vec<Value>, Vec<Vec<Value>>> = HashMap::new();

            for row in &input_rows {
                let key: Vec<Value> = self
                    .group_expr
                    .iter()
                    .map(|expr| self.evaluate_expr(expr, row, self.input.schema()))
                    .collect();
                groups.entry(key).or_insert_with(Vec::new).push(row.clone());
            }

            let mut results = vec![];
            for (key, group_rows) in groups {
                let mut row = key;
                for agg_expr in &self.aggregate_expr {
                    if let Expr::AggregateFunction { func, args, .. } = agg_expr {
                        let values: Vec<Value> = group_rows
                            .iter()
                            .map(|r| {
                                self.evaluate_expr(
                                    args.first().unwrap_or(&Expr::Wildcard),
                                    r,
                                    self.input.schema(),
                                )
                            })
                            .collect();
                        let result = self.compute_aggregate(func, &values);
                        row.push(result);
                    }
                }
                results.push(row);
            }

            Ok(results)
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataType, Expr, Field, Schema, SortExpr};
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
    fn test_projection_exec_schema() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = ProjectionExec::new(Box::new(child), vec![], schema.clone());
        assert_eq!(exec.schema().fields.len(), 1);
    }

    #[test]
    fn test_projection_exec_children() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = ProjectionExec::new(Box::new(child), vec![], schema);
        assert_eq!(exec.children().len(), 1);
    }

    #[test]
    fn test_filter_exec_schema() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let predicate = Expr::column("id");
        let exec = FilterExec::new(Box::new(child), predicate);
        assert_eq!(exec.schema().fields.len(), 1);
    }

    #[test]
    fn test_filter_exec_children() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let predicate = Expr::column("id");
        let exec = FilterExec::new(Box::new(child), predicate);
        assert_eq!(exec.children().len(), 1);
    }

    #[test]
    fn test_aggregate_exec_new() {
        let schema = Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = AggregateExec::new(Box::new(child), vec![], vec![], schema);
        assert_eq!(exec.name(), "Aggregate");
    }

    #[test]
    fn test_aggregate_exec_schema() {
        let schema = Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = AggregateExec::new(Box::new(child), vec![], vec![], schema);
        assert_eq!(exec.schema().fields.len(), 1);
    }

    #[test]
    fn test_hash_join_exec_children() {
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
        assert_eq!(exec.children().len(), 2);
    }

    #[test]
    fn test_sort_exec_new() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = SortExec::new(Box::new(child), vec![]);
        assert_eq!(exec.name(), "Sort");
    }

    #[test]
    fn test_sort_exec_schema() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = SortExec::new(Box::new(child), vec![]);
        assert_eq!(exec.schema().fields.len(), 1);
    }
}
