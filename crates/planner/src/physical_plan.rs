//! Physical Plan Module
//!
//! Defines the physical execution representation of query plans.

#![allow(dead_code)]

use crate::AggregateFunction;
use crate::Expr;
use crate::Operator;
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

    /// Downcast to concrete type
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

    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        Ok(vec![])
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

    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        let input_rows = self.input.execute()?;

        if self.expr.is_empty() {
            return Ok(input_rows);
        }

        let input_schema = self.input.schema();
        let mut results = vec![];

        for row in input_rows {
            let mut projected_row = vec![];
            for expr in &self.expr {
                let value = self.evaluate_expr(expr, &row, input_schema);
                projected_row.push(value);
            }
            results.push(projected_row);
        }

        Ok(results)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ProjectionExec {
    pub fn input(&self) -> &dyn PhysicalPlan {
        self.input.as_ref()
    }

    pub fn expr(&self) -> &Vec<Expr> {
        &self.expr
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
            Expr::Wildcard => Value::Text(
                row.iter()
                    .map(|v| format!("{:?}", v))
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            Expr::Alias { expr, .. } => self.evaluate_expr(expr, row, schema),
            Expr::BinaryExpr { left, op, right } => {
                let lval = self.evaluate_expr(left, row, schema);
                let rval = self.evaluate_expr(right, row, schema);
                self.evaluate_arithmetic(op, &lval, &rval)
            }
            _ => Value::Null,
        }
    }

    fn evaluate_arithmetic(&self, op: &Operator, left: &Value, right: &Value) -> Value {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => match op {
                Operator::Plus => Value::Integer(l + r),
                Operator::Minus => Value::Integer(l - r),
                Operator::Multiply => Value::Integer(l * r),
                Operator::Divide => {
                    if *r != 0 {
                        Value::Integer(l / r)
                    } else {
                        Value::Null
                    }
                }
                Operator::Modulo => {
                    if *r != 0 {
                        Value::Integer(l % r)
                    } else {
                        Value::Null
                    }
                }
                _ => Value::Null,
            },
            (Value::Float(l), Value::Float(r)) => match op {
                Operator::Plus => Value::Float(l + r),
                Operator::Minus => Value::Float(l - r),
                Operator::Multiply => Value::Float(l * r),
                Operator::Divide => {
                    if *r != 0.0 {
                        Value::Float(l / r)
                    } else {
                        Value::Null
                    }
                }
                _ => Value::Null,
            },
            _ => Value::Null,
        }
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

    pub fn input(&self) -> &dyn PhysicalPlan {
        self.input.as_ref()
    }

    pub fn predicate(&self) -> &Expr {
        &self.predicate
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

    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        let input_rows = self.input.execute()?;
        let input_schema = self.input.schema();

        let filtered: Vec<Vec<Value>> = input_rows
            .into_iter()
            .filter(|row| self.evaluate_predicate(&self.predicate, row, input_schema))
            .collect();

        Ok(filtered)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl FilterExec {
    fn evaluate_predicate(&self, expr: &Expr, row: &[Value], schema: &Schema) -> bool {
        match expr {
            Expr::BinaryExpr { left, op, right } => {
                let lval = self.evaluate_expr(left, row, schema);
                let rval = self.evaluate_expr(right, row, schema);
                self.compare_values(&lval, op, &rval)
            }
            Expr::Literal(Value::Integer(n)) => *n != 0,
            _ => true,
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
            _ => Value::Null,
        }
    }

    fn compare_values(&self, left: &Value, op: &Operator, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => match op {
                Operator::Eq => l == r,
                Operator::NotEq => l != r,
                Operator::Gt => l > r,
                Operator::Lt => l < r,
                Operator::GtEq => l >= r,
                Operator::LtEq => l <= r,
                _ => false,
            },
            _ => false,
        }
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

    pub fn input(&self) -> &dyn PhysicalPlan {
        self.input.as_ref()
    }

    pub fn group_expr(&self) -> &Vec<Expr> {
        &self.group_expr
    }

    pub fn aggregate_expr(&self) -> &Vec<Expr> {
        &self.aggregate_expr
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
            Expr::Alias { expr, .. } => self.evaluate_expr(expr, row, schema),
            _ => Value::Null,
        }
    }

    fn compute_aggregate(
        &self,
        func: &AggregateFunction,
        args: &[Expr],
        values: &[Value],
    ) -> Value {
        match func {
            AggregateFunction::Count => {
                if args.is_empty() {
                    Value::Integer(values.len() as i64)
                } else {
                    let non_null_count =
                        values.iter().filter(|v| !matches!(v, Value::Null)).count();
                    Value::Integer(non_null_count as i64)
                }
            }
            AggregateFunction::Sum => {
                let mut sum: i64 = 0;
                let mut sum_float: f64 = 0.0;
                let mut has_float = false;
                for v in values {
                    if let Value::Null = v {
                        continue;
                    }
                    if let Value::Integer(n) = v {
                        sum += n;
                    } else if let Value::Float(n) = v {
                        has_float = true;
                        sum_float += n;
                    }
                }
                if has_float {
                    Value::Float(sum_float + sum as f64)
                } else {
                    Value::Integer(sum)
                }
            }
            AggregateFunction::Avg => {
                let mut sum: i64 = 0;
                let mut sum_float: f64 = 0.0;
                let mut count = 0;
                let mut has_float = false;
                for v in values {
                    if let Value::Null = v {
                        continue;
                    }
                    if let Value::Integer(n) = v {
                        sum += n;
                        count += 1;
                    } else if let Value::Float(n) = v {
                        has_float = true;
                        sum_float += n;
                        count += 1;
                    }
                }
                if count > 0 {
                    if has_float {
                        Value::Float((sum_float + sum as f64) / count as f64)
                    } else {
                        Value::Integer(sum / count as i64)
                    }
                } else {
                    Value::Null
                }
            }
            AggregateFunction::Min => {
                let mut min_val: Option<(bool, i64, f64)> = None;
                for v in values {
                    if let Value::Null = v {
                        continue;
                    }
                    if let Value::Integer(n) = v {
                        let n = *n;
                        match min_val {
                            Some((false, m, _)) if n < m => min_val = Some((false, n, 0.0)),
                            None => min_val = Some((false, n, 0.0)),
                            _ => {}
                        }
                    } else if let Value::Float(n) = v {
                        let n = *n;
                        match min_val {
                            Some((true, _, m)) if n < m => min_val = Some((true, 0, n)),
                            None => min_val = Some((true, 0, n)),
                            Some((false, _, _)) => min_val = Some((true, 0, n)),
                            _ => {}
                        }
                    }
                }
                match min_val {
                    Some((true, _, n)) => Value::Float(n),
                    Some((false, n, _)) => Value::Integer(n),
                    None => Value::Null,
                }
            }
            AggregateFunction::Max => {
                let mut max_val: Option<(bool, i64, f64)> = None;
                for v in values {
                    if let Value::Null = v {
                        continue;
                    }
                    if let Value::Integer(n) = v {
                        let n = *n;
                        match max_val {
                            Some((false, m, _)) if n > m => max_val = Some((false, n, 0.0)),
                            None => max_val = Some((false, n, 0.0)),
                            _ => {}
                        }
                    } else if let Value::Float(n) = v {
                        let n = *n;
                        match max_val {
                            Some((true, _, m)) if n > m => max_val = Some((true, 0, n)),
                            None => max_val = Some((true, 0, n)),
                            Some((false, _, _)) => max_val = Some((true, 0, n)),
                            _ => {}
                        }
                    }
                }
                match max_val {
                    Some((true, _, n)) => Value::Float(n),
                    Some((false, n, _)) => Value::Integer(n),
                    None => Value::Null,
                }
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
                    let values: Vec<Value> = if args.is_empty() {
                        input_rows.iter().map(|_| Value::Null).collect()
                    } else {
                        input_rows
                            .iter()
                            .map(|row| {
                                self.evaluate_expr(
                                    args.first().unwrap_or(&Expr::Wildcard),
                                    row,
                                    self.input.schema(),
                                )
                            })
                            .collect()
                    };
                    let result = self.compute_aggregate(func, args, &values);
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
                        let values: Vec<Value> = if args.is_empty() {
                            group_rows.iter().map(|_| Value::Null).collect()
                        } else {
                            group_rows
                                .iter()
                                .map(|r| {
                                    self.evaluate_expr(
                                        args.first().unwrap_or(&Expr::Wildcard),
                                        r,
                                        self.input.schema(),
                                    )
                                })
                                .collect()
                        };
                        let result = self.compute_aggregate(func, args, &values);
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

    pub fn left(&self) -> &dyn PhysicalPlan {
        self.left.as_ref()
    }

    pub fn right(&self) -> &dyn PhysicalPlan {
        self.right.as_ref()
    }

    pub fn join_type(&self) -> crate::JoinType {
        self.join_type.clone()
    }

    pub fn condition(&self) -> Option<&Expr> {
        self.condition.as_ref()
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

    #[test]
    fn test_projection_exec_column() {
        let input_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let output_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let proj = ProjectionExec::new(input, vec![Expr::column("id")], output_schema);

        assert_eq!(proj.name(), "Projection");
        assert_eq!(proj.schema().fields.len(), 1);
    }

    #[test]
    fn test_projection_exec_alias() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let aliased_expr = Expr::Alias {
            expr: Box::new(Expr::column("id")),
            name: "my_id".to_string(),
        };
        let proj = ProjectionExec::new(input, vec![aliased_expr], schema);

        assert_eq!(proj.name(), "Projection");
    }

    #[test]
    fn test_aggregate_exec_count_star() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("test_table".to_string(), schema.clone());
        let agg = AggregateExec::new(
            Box::new(child),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Count,
                args: vec![],
                distinct: false,
            }],
            Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]),
        );

        assert_eq!(agg.name(), "Aggregate");
    }

    #[test]
    fn test_aggregate_exec_count_column() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("test_table".to_string(), schema.clone());
        let agg = AggregateExec::new(
            Box::new(child),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Count,
                args: vec![Expr::column("id")],
                distinct: false,
            }],
            Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]),
        );

        assert_eq!(agg.name(), "Aggregate");
    }

    #[test]
    fn test_aggregate_exec_sum() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("test_table".to_string(), schema.clone());
        let agg = AggregateExec::new(
            Box::new(child),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Sum,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            Schema::new(vec![Field::new("sum".to_string(), DataType::Integer)]),
        );

        assert_eq!(agg.name(), "Aggregate");
    }

    #[test]
    fn test_aggregate_exec_group_by() {
        let schema = Schema::new(vec![
            Field::new("category".to_string(), DataType::Text),
            Field::new("amount".to_string(), DataType::Integer),
        ]);
        let child = SeqScanExec::new("test_table".to_string(), schema.clone());
        let agg = AggregateExec::new(
            Box::new(child),
            vec![Expr::column("category")],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Sum,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            Schema::new(vec![
                Field::new("category".to_string(), DataType::Text),
                Field::new("sum".to_string(), DataType::Integer),
            ]),
        );

        assert_eq!(agg.name(), "Aggregate");
    }
}
