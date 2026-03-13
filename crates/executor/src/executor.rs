//! Volcano Executor trait - 统一的查询执行接口

use sqlrustgo_planner::{PhysicalPlan, Schema};
use sqlrustgo_types::{SqlResult, Value, SqlError};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct ExecutorResult {
    pub rows: Vec<Vec<Value>>,
    pub affected_rows: usize,
}

impl ExecutorResult {
    pub fn new(rows: Vec<Vec<Value>>, affected_rows: usize) -> Self { Self { rows, affected_rows } }
    pub fn empty() -> Self { Self { rows: vec![], affected_rows: 0 } }
}

pub trait VolcanoExecutor: Send + Sync {
    fn init(&mut self) -> SqlResult<()>;
    fn next(&mut self) -> SqlResult<Option<Vec<Value>>>;
    fn close(&mut self) -> SqlResult<()>;
    fn schema(&self) -> &Schema;
    fn name(&self) -> &str;
    fn is_initialized(&self) -> bool;
    fn as_any(&self) -> &dyn Any;
}

pub type VolIterator = Box<dyn VolcanoExecutor>;

pub fn execute_collect(executor: &mut dyn VolcanoExecutor) -> SqlResult<ExecutorResult> {
    executor.init()?;
    let mut rows = Vec::new();
    while let Some(row) = executor.next()? { rows.push(row); }
    executor.close()?;
    Ok(ExecutorResult::new(rows, 0))
}

pub trait Executor: Send + Sync {
    fn execute(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult>;
    fn name(&self) -> &str;
    fn is_ready(&self) -> bool;
}

pub struct SeqScanVolcanoExecutor {
    table_name: String,
    schema: Schema,
    storage: std::sync::Arc<dyn Storage>,
    initialized: bool,
    current_idx: usize,
    rows: Vec<Vec<Value>>,
}

impl SeqScanVolcanoExecutor {
    pub fn new(table_name: String, schema: Schema, storage: std::sync::Arc<dyn Storage>) -> Self {
        Self { table_name, schema, storage, initialized: false, current_idx: 0, rows: Vec::new() }
    }
}

impl VolcanoExecutor for SeqScanVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> {
        if self.initialized { return Ok(()); }
        self.rows = self.storage.scan(&self.table_name).unwrap_or_default();
        self.current_idx = 0;
        self.initialized = true;
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized { return Err(SqlError::ExecutionError("Not initialized".to_string())); }
        if self.current_idx >= self.rows.len() { return Ok(None); }
        let row = self.rows[self.current_idx].clone();
        self.current_idx += 1;
        Ok(Some(row))
    }

    fn close(&mut self) -> SqlResult<()> { self.rows.clear(); self.current_idx = 0; self.initialized = false; Ok(()) }
    fn schema(&self) -> &Schema { &self.schema }
    fn name(&self) -> &str { "SeqScan" }
    fn is_initialized(&self) -> bool { self.initialized }
    fn as_any(&self) -> &dyn Any { self }
}

pub struct ProjectionVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    expr: Vec<sqlrustgo_planner::Expr>,
    schema: Schema,
    input_schema: Schema,
    initialized: bool,
}

impl ProjectionVolcanoExecutor {
    pub fn new(child: Box<dyn VolcanoExecutor>, expr: Vec<sqlrustgo_planner::Expr>, schema: Schema, input_schema: Schema) -> Self {
        Self { child, expr, schema, input_schema, initialized: false }
    }
}

impl VolcanoExecutor for ProjectionVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> { self.child.init()?; self.initialized = true; Ok(()) }
    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if let Some(row) = self.child.next()? {
            let projected_row: Vec<Value> = self.expr.iter().map(|expr| expr.evaluate(&row, &self.input_schema).unwrap_or(Value::Null)).collect();
            Ok(Some(projected_row))
        } else { Ok(None) }
    }
    fn close(&mut self) -> SqlResult<()> { self.child.close()?; self.initialized = false; Ok(()) }
    fn schema(&self) -> &Schema { &self.schema }
    fn name(&self) -> &str { "Projection" }
    fn is_initialized(&self) -> bool { self.initialized }
    fn as_any(&self) -> &dyn Any { self }
}

pub struct FilterVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    predicate: sqlrustgo_planner::Expr,
    schema: Schema,
    input_schema: Schema,
    initialized: bool,
}

impl FilterVolcanoExecutor {
    pub fn new(child: Box<dyn VolcanoExecutor>, predicate: sqlrustgo_planner::Expr, schema: Schema, input_schema: Schema) -> Self {
        Self { child, predicate, schema, input_schema, initialized: false }
    }
}

impl VolcanoExecutor for FilterVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> { self.child.init()?; self.initialized = true; Ok(()) }
    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        while let Some(row) = self.child.next()? {
            let predicate_val = self.predicate.evaluate(&row, &self.input_schema).unwrap_or(Value::Null);
            if let Value::Boolean(true) = predicate_val { return Ok(Some(row)); }
        }
        Ok(None)
    }
    fn close(&mut self) -> SqlResult<()> { self.child.close()?; self.initialized = false; Ok(()) }
    fn schema(&self) -> &Schema { &self.schema }
    fn name(&self) -> &str { "Filter" }
    fn is_initialized(&self) -> bool { self.initialized }
    fn as_any(&self) -> &dyn Any { self }
}

pub struct AggregateVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    group_expr: Vec<sqlrustgo_planner::Expr>,
    aggregate_expr: Vec<sqlrustgo_planner::Expr>,
    schema: Schema,
    input_schema: Schema,
    initialized: bool,
    groups: std::collections::HashMap<Vec<Value>, Vec<Vec<Value>>>,
    group_keys: Vec<Vec<Value>>,
    current_group_idx: usize,
}

impl AggregateVolcanoExecutor {
    pub fn new(child: Box<dyn VolcanoExecutor>, group_expr: Vec<sqlrustgo_planner::Expr>, aggregate_expr: Vec<sqlrustgo_planner::Expr>, schema: Schema, input_schema: Schema) -> Self {
        Self { child, group_expr, aggregate_expr, schema, input_schema, initialized: false, groups: std::collections::HashMap::new(), group_keys: Vec::new(), current_group_idx: 0 }
    }

    fn compute_aggregate(&self, group_rows: &[Vec<Value>]) -> Vec<Value> {
        let mut results = Vec::new();
        for agg_expr in &self.aggregate_expr {
            if let sqlrustgo_planner::Expr::AggregateFunction { func, args, .. } = agg_expr {
                let agg_values: Vec<Value> = group_rows.iter().flat_map(|row| {
                    if args.is_empty() { vec![Value::Integer(group_rows.len() as i64)] }
                    else { args.iter().map(|arg| arg.evaluate(row, &self.input_schema).unwrap_or(Value::Null)).collect() }
                }).collect();
                
                let result = match func {
                    sqlrustgo_planner::AggregateFunction::Count => Value::Integer(agg_values.len() as i64),
                    sqlrustgo_planner::AggregateFunction::Sum => { let mut sum: i64 = 0; for v in &agg_values { if let Value::Integer(n) = v { sum += n; } } Value::Integer(sum) }
                    sqlrustgo_planner::AggregateFunction::Avg => { let mut sum: i64 = 0; let mut count = 0; for v in &agg_values { if let Value::Integer(n) = v { sum += n; count += 1; } } if count > 0 { Value::Integer(sum / count as i64) } else { Value::Null } }
                    sqlrustgo_planner::AggregateFunction::Min => { let mut min_val: Option<i64> = None; for v in &agg_values { if let Value::Integer(n) = v { match min_val { Some(m) if *n < m => min_val = Some(*n), None => min_val = Some(*n), _ => {} } } } min_val.map(Value::Integer).unwrap_or(Value::Null) }
                    sqlrustgo_planner::AggregateFunction::Max => { let mut max_val: Option<i64> = None; for v in &agg_values { if let Value::Integer(n) = v { match max_val { Some(m) if *n > m => max_val = Some(*n), None => max_val = Some(*n), _ => {} } } } max_val.map(Value::Integer).unwrap_or(Value::Null) }
                };
                results.push(result);
            }
        }
        results
    }
}

impl VolcanoExecutor for AggregateVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> {
        self.child.init()?;
        let mut all_rows = Vec::new();
        while let Some(row) = self.child.next()? { all_rows.push(row); }
        self.child.close()?;
        for row in &all_rows {
            let key: Vec<Value> = self.group_expr.iter().map(|expr| expr.evaluate(row, &self.input_schema).unwrap_or(Value::Null)).collect();
            self.groups.entry(key).or_insert_with(Vec::new).push(row.clone());
        }
        self.group_keys = self.groups.keys().cloned().collect();
        self.group_keys.sort_by(|a, b| {
            for (av, bv) in a.iter().zip(b.iter()) { match (av, bv) { (Value::Integer(ai), Value::Integer(bi)) => { let cmp = ai.cmp(bi); if cmp != std::cmp::Ordering::Equal { return cmp; } } (Value::Text(ai), Value::Text(bi)) => { let cmp = ai.cmp(bi); if cmp != std::cmp::Ordering::Equal { return cmp; } } _ => {} } }
            a.len().cmp(&b.len())
        });
        self.current_group_idx = 0;
        self.initialized = true;
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized { return Err(SqlError::ExecutionError("Not initialized".to_string())); }
        if self.current_group_idx >= self.group_keys.len() { return Ok(None); }
        let key = &self.group_keys[self.current_group_idx];
        let group_rows = &self.groups[key];
        let mut result = key.clone();
        result.extend(self.compute_aggregate(group_rows));
        self.current_group_idx += 1;
        Ok(Some(result))
    }

    fn close(&mut self) -> SqlResult<()> { self.groups.clear(); self.group_keys.clear(); self.initialized = false; Ok(()) }
    fn schema(&self) -> &Schema { &self.schema }
    fn name(&self) -> &str { "Aggregate" }
    fn is_initialized(&self) -> bool { self.initialized }
    fn as_any(&self) -> &dyn Any { self }
}

pub struct HashJoinVolcanoExecutor {
    left: Box<dyn VolcanoExecutor>,
    right: Box<dyn VolcanoExecutor>,
    join_type: sqlrustgo_planner::JoinType,
    left_schema: Schema,
    right_schema: Schema,
    schema: Schema,
    initialized: bool,
    right_hash: std::collections::HashMap<Vec<Value>, Vec<Vec<Value>>>,
    current_left_rows: Vec<Vec<Value>>,
    left_idx: usize,
    right_idx: usize,
}

impl HashJoinVolcanoExecutor {
    pub fn new(left: Box<dyn VolcanoExecutor>, right: Box<dyn VolcanoExecutor>, join_type: sqlrustgo_planner::JoinType, left_schema: Schema, right_schema: Schema, schema: Schema) -> Self {
        Self { left, right, join_type, left_schema, right_schema, schema, initialized: false, right_hash: std::collections::HashMap::new(), current_left_rows: Vec::new(), left_idx: 0, right_idx: 0 }
    }

    fn next_inner(&mut self) -> SqlResult<Option<Vec<Value>>> {
        while self.left_idx < self.current_left_rows.len() {
            let left_row = &self.current_left_rows[self.left_idx];
            let key = if !left_row.is_empty() { vec![left_row[0].clone()] } else { vec![Value::Null] };
            if let Some(right_rows) = self.right_hash.get(&key) {
                if self.right_idx < right_rows.len() {
                    let mut result = left_row.clone();
                    result.extend(right_rows[self.right_idx].clone());
                    self.right_idx += 1;
                    return Ok(Some(result));
                }
            }
            self.left_idx += 1;
            self.right_idx = 0;
        }
        Ok(None)
    }

    fn next_left(&mut self) -> SqlResult<Option<Vec<Value>>> {
        while self.left_idx < self.current_left_rows.len() {
            let left_row = &self.current_left_rows[self.left_idx];
            let key = if !left_row.is_empty() { vec![left_row[0].clone()] } else { vec![Value::Null] };
            if let Some(right_rows) = self.right_hash.get(&key) {
                if self.right_idx < right_rows.len() {
                    let mut result = left_row.clone();
                    result.extend(right_rows[self.right_idx].clone());
                    self.right_idx += 1;
                    return Ok(Some(result));
                }
            } else {
                let mut result = left_row.clone();
                for _ in 0..self.right_schema.fields.len() { result.push(Value::Null); }
                self.left_idx += 1;
                self.right_idx = 0;
                return Ok(Some(result));
            }
            self.left_idx += 1;
            self.right_idx = 0;
        }
        Ok(None)
    }
}

impl VolcanoExecutor for HashJoinVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> {
        self.left.init()?;
        self.right.init()?;
        while let Some(row) = self.right.next()? { let key = if !row.is_empty() { vec![row[0].clone()] } else { vec![Value::Null] }; self.right_hash.entry(key).or_insert_with(Vec::new).push(row); }
        while let Some(row) = self.left.next()? { self.current_left_rows.push(row); }
        self.left_idx = 0;
        self.right_idx = 0;
        self.initialized = true;
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized { return Err(SqlError::ExecutionError("Not initialized".to_string())); }
        match self.join_type { sqlrustgo_planner::JoinType::Inner => self.next_inner(), sqlrustgo_planner::JoinType::Left => self.next_left(), _ => Ok(None) }
    }

    fn close(&mut self) -> SqlResult<()> { self.left.close()?; self.right.close()?; self.right_hash.clear(); self.current_left_rows.clear(); self.initialized = false; Ok(()) }
    fn schema(&self) -> &Schema { &self.schema }
    fn name(&self) -> &str { "HashJoin" }
    fn is_initialized(&self) -> bool { self.initialized }
    fn as_any(&self) -> &dyn Any { self }
}

pub struct SortVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    sort_expr: Vec<sqlrustgo_planner::SortExpr>,
    schema: Schema,
    input_schema: Schema,
    initialized: bool,
    sorted_rows: Vec<Vec<Value>>,
    current_idx: usize,
}

impl SortVolcanoExecutor {
    pub fn new(child: Box<dyn VolcanoExecutor>, sort_expr: Vec<sqlrustgo_planner::SortExpr>, schema: Schema, input_schema: Schema) -> Self {
        Self { child, sort_expr, schema, input_schema, initialized: false, sorted_rows: Vec::new(), current_idx: 0 }
    }
}

impl VolcanoExecutor for SortVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> {
        self.child.init()?;
        while let Some(row) = self.child.next()? { self.sorted_rows.push(row); }
        self.child.close()?;
        self.sorted_rows.sort_by(|a, b| {
            for sort_expr in &self.sort_expr {
                let a_val = sort_expr.expr.evaluate(a, &self.input_schema).unwrap_or(Value::Null);
                let b_val = sort_expr.expr.evaluate(b, &self.input_schema).unwrap_or(Value::Null);
                let cmp = match (&a_val, &b_val) { (Value::Integer(ai), Value::Integer(bi)) => ai.cmp(bi), (Value::Text(ai), Value::Text(bi)) => ai.cmp(bi), _ => std::cmp::Ordering::Equal };
                let result = if sort_expr.asc { cmp } else { cmp.reverse() };
                if result != std::cmp::Ordering::Equal { return result; }
            }
            std::cmp::Ordering::Equal
        });
        self.current_idx = 0;
        self.initialized = true;
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized { return Err(SqlError::ExecutionError("Not initialized".to_string())); }
        if self.current_idx >= self.sorted_rows.len() { return Ok(None); }
        let row = self.sorted_rows[self.current_idx].clone();
        self.current_idx += 1;
        Ok(Some(row))
    }

    fn close(&mut self) -> SqlResult<()> { self.sorted_rows.clear(); self.current_idx = 0; self.initialized = false; Ok(()) }
    fn schema(&self) -> &Schema { &self.schema }
    fn name(&self) -> &str { "Sort" }
    fn is_initialized(&self) -> bool { self.initialized }
    fn as_any(&self) -> &dyn Any { self }
}

pub struct LimitVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    limit: usize,
    offset: usize,
    schema: Schema,
    initialized: bool,
    current_idx: usize,
}

impl LimitVolcanoExecutor {
    pub fn new(child: Box<dyn VolcanoExecutor>, limit: usize, offset: usize, schema: Schema) -> Self {
        Self { child, limit, offset, schema, initialized: false, current_idx: 0 }
    }
}

impl VolcanoExecutor for LimitVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> { self.child.init()?; self.current_idx = 0; self.initialized = true; Ok(()) }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized { return Err(SqlError::ExecutionError("Not initialized".to_string())); }
        while self.current_idx < self.offset { if self.child.next()?.is_none() { return Ok(None); } self.current_idx += 1; }
        if self.current_idx >= self.offset + self.limit { return Ok(None); }
        let result = self.child.next()?;
        if result.is_some() { self.current_idx += 1; }
        Ok(result)
    }

    fn close(&mut self) -> SqlResult<()> { self.child.close()?; self.initialized = false; Ok(()) }
    fn schema(&self) -> &Schema { &self.schema }
    fn name(&self) -> &str { "Limit" }
    fn is_initialized(&self) -> bool { self.initialized }
    fn as_any(&self) -> &dyn Any { self }
}

pub struct VolExecutorBuilder { storage: std::sync::Arc<dyn Storage> }

impl VolExecutorBuilder {
    pub fn new(storage: std::sync::Arc<dyn Storage>) -> Self { Self { storage } }

    pub fn build(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        match plan.name() {
            "SeqScan" => self.build_seq_scan(plan),
            "Projection" => self.build_projection(plan),
            "Filter" => self.build_filter(plan),
            "Aggregate" => self.build_aggregate(plan),
            "HashJoin" => self.build_hash_join(plan),
            "Sort" => self.build_sort(plan),
            "Limit" => self.build_limit(plan),
            _ => Err(SqlError::ExecutionError(format!("Unsupported plan type: {}", plan.name()))),
        }
    }

    fn build_seq_scan(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        Ok(Box::new(SeqScanVolcanoExecutor::new(plan.table_name().to_string(), plan.schema().clone(), self.storage.clone())))
    }

    fn build_projection(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        let children = plan.children();
        if children.is_empty() { return Err(SqlError::ExecutionError("Projection has no children".to_string())); }
        let child = self.build(children[0])?;
        let projection = plan.as_any().downcast_ref::<sqlrustgo_planner::ProjectionExec>().map(|p| p.expr().clone()).unwrap_or_default();
        Ok(Box::new(ProjectionVolcanoExecutor::new(child, projection, plan.schema().clone(), children[0].schema().clone())))
    }

    fn build_filter(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        let children = plan.children();
        if children.is_empty() { return Err(SqlError::ExecutionError("Filter has no children".to_string())); }
        let child = self.build(children[0])?;
        let filter = plan.as_any().downcast_ref::<sqlrustgo_planner::FilterExec>().map(|f| f.predicate().clone()).unwrap_or(sqlrustgo_planner::Expr::Literal(Value::Boolean(true)));
        Ok(Box::new(FilterVolcanoExecutor::new(child, filter, plan.schema().clone(), children[0].schema().clone())))
    }

    fn build_aggregate(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        let children = plan.children();
        if children.is_empty() { return Err(SqlError::ExecutionError("Aggregate has no children".to_string())); }
        let child = self.build(children[0])?;
        let aggregate = plan.as_any().downcast_ref::<sqlrustgo_planner::AggregateExec>().ok_or_else(|| SqlError::ExecutionError("Failed to cast to AggregateExec".to_string()))?;
        Ok(Box::new(AggregateVolcanoExecutor::new(child, aggregate.group_expr().clone(), aggregate.aggregate_expr().clone(), plan.schema().clone(), children[0].schema().clone())))
    }

    fn build_hash_join(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        let children = plan.children();
        if children.len() < 2 { return Err(SqlError::ExecutionError("HashJoin has less than 2 children".to_string())); }
        let left = self.build(children[0])?;
        let right = self.build(children[1])?;
        let hash_join = plan.as_any().downcast_ref::<sqlrustgo_planner::HashJoinExec>().ok_or_else(|| SqlError::ExecutionError("Failed to cast to HashJoinExec".to_string()))?;
        Ok(Box::new(HashJoinVolcanoExecutor::new(left, right, hash_join.join_type(), children[0].schema().clone(), children[1].schema().clone(), plan.schema().clone())))
    }

    fn build_sort(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        let children = plan.children();
        if children.is_empty() { return Err(SqlError::ExecutionError("Sort has no children".to_string())); }
        let child = self.build(children[0])?;
        let sort = plan.as_any().downcast_ref::<sqlrustgo_planner::physical_plan::SortExec>().map(|s| s.sort_expr().clone()).unwrap_or_default();
        Ok(Box::new(SortVolcanoExecutor::new(child, sort, plan.schema().clone(), children[0].schema().clone())))
    }

    fn build_limit(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        let children = plan.children();
        if children.is_empty() { return Err(SqlError::ExecutionError("Limit has no children".to_string())); }
        let child = self.build(children[0])?;
        let limit_exec = plan.as_any().downcast_ref::<sqlrustgo_planner::physical_plan::LimitExec>();
        let limit = limit_exec.map(|l| l.limit()).unwrap_or(usize::MAX);
        let offset = limit_exec.and_then(|l| l.offset()).unwrap_or(0);
        Ok(Box::new(LimitVolcanoExecutor::new(child, limit, offset, plan.schema().clone())))
    }
}

pub trait Storage: Send + Sync {
    fn scan(&self, table_name: &str) -> SqlResult<Vec<Vec<Value>>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_planner::{PhysicalPlan, Schema, Field, DataType};

    pub struct MockExecutor;
    impl MockExecutor { pub fn new() -> Self { Self } }
    impl Default for MockExecutor { fn default() -> Self { Self::new() } }
    impl Executor for MockExecutor {
        fn execute(&self, _plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> { Ok(ExecutorResult::empty()) }
        fn name(&self) -> &str { "mock" }
        fn is_ready(&self) -> bool { true }
    }

    #[test]
    fn test_executor_result() {
        let result = ExecutorResult::new(vec![], 0); assert!(result.rows.is_empty());
        let result = ExecutorResult::new(vec![vec![Value::Integer(1)]], 1);
        assert_eq!(result.rows.len(), 1); assert_eq!(result.affected_rows, 1);
    }

    #[test]
    fn test_executor_result_empty() { let result = ExecutorResult::empty(); assert!(result.rows.is_empty()); assert_eq!(result.affected_rows, 0); }

    #[test]
    fn test_executor_result_with_rows() {
        let rows = vec![vec![Value::Integer(1), Value::Text("Alice".to_string())], vec![Value::Integer(2), Value::Text("Bob".to_string())]];
        let result = ExecutorResult::new(rows, 0); assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_executor_send_sync() { fn _check<T: Send + Sync>() {} _check::<MockExecutor>(); _check::<ExecutorResult>(); }

    #[test]
    fn test_volcano_executor_trait_object() { let _executor: Box<dyn VolcanoExecutor> = Box::new(MockVolcanoExecutor::new()); }

    pub struct MockVolcanoExecutor {
        data: Vec<Vec<Value>>, idx: usize, initialized: bool, schema: Schema,
    }
    impl MockVolcanoExecutor {
        pub fn new() -> Self { 
            Self { 
                data: vec![vec![Value::Integer(1), Value::Text("Alice".to_string())], vec![Value::Integer(2), Value::Text("Bob".to_string())]], 
                idx: 0, 
                initialized: false,
                schema: Schema::new(vec![Field::new("id".to_string(), DataType::Integer), Field::new("name".to_string(), DataType::Text)]),
            } 
        }
    }
    impl Default for MockVolcanoExecutor { fn default() -> Self { Self::new() } }
    impl VolcanoExecutor for MockVolcanoExecutor {
        fn init(&mut self) -> SqlResult<()> { self.initialized = true; self.idx = 0; Ok(()) }
        fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
            if !self.initialized { return Err(SqlError::ExecutionError("Not initialized".to_string())); }
            if self.idx >= self.data.len() { return Ok(None); }
            let row = self.data[self.idx].clone(); self.idx += 1; Ok(Some(row))
        }
        fn close(&mut self) -> SqlResult<()> { self.initialized = false; Ok(()) }
        fn schema(&self) -> &Schema { &self.schema }
        fn name(&self) -> &str { "MockVolcano" }
        fn is_initialized(&self) -> bool { self.initialized }
        fn as_any(&self) -> &dyn Any { self }
    }

    #[test]
    fn test_mock_volcano_executor() {
        let mut executor = MockVolcanoExecutor::new();
        let result = executor.next(); assert!(result.is_err());
        executor.init().unwrap(); assert!(executor.is_initialized());
        let row1 = executor.next().unwrap(); assert!(row1.is_some());
        let row2 = executor.next().unwrap(); assert!(row2.is_some());
        let row3 = executor.next().unwrap(); assert!(row3.is_none());
        executor.close().unwrap();
        executor.init().unwrap(); let row = executor.next().unwrap(); assert!(row.is_some());
    }

    #[test]
    fn test_mock_executor_name() { let executor = MockExecutor::new(); assert_eq!(executor.name(), "mock"); }
    #[test]
    fn test_mock_executor_is_ready() { let executor = MockExecutor::new(); assert!(executor.is_ready()); }
    #[test]
    fn test_volcano_executor_schema() { let executor = MockVolcanoExecutor::new(); let schema = executor.schema(); assert_eq!(schema.fields.len(), 2); }
    #[test]
    fn test_volcano_executor_name() { let executor = MockVolcanoExecutor::new(); assert_eq!(executor.name(), "MockVolcano"); }
    #[test]
    fn test_execute_collect() { let mut executor = MockVolcanoExecutor::new(); let result = execute_collect(&mut executor).unwrap(); assert_eq!(result.rows.len(), 2); }
    #[test]
    fn test_volcano_executor_send_sync() { fn _check<T: Send + Sync>() {} _check::<MockVolcanoExecutor>(); _check::<Box<dyn VolcanoExecutor>>(); }
}
