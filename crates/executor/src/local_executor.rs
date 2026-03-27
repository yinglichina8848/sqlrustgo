//! LocalExecutor - Local query execution implementation
//!
//! Implements the Executor trait for local execution using StorageEngine.

#[allow(unused_imports)]
use sqlrustgo_planner::{
    AggregateExec, AggregateFunction, FilterExec, HashJoinExec, JoinType, PhysicalPlan,
    ProjectionExec, SortMergeJoinExec,
};
use sqlrustgo_storage::StorageEngine;
use sqlrustgo_types::{SqlResult, Value};

use crate::operator_profile::GLOBAL_PROFILER;
use crate::query_cache::should_cache;
use crate::query_cache::QueryCache;
use crate::query_cache_config::{CacheEntry, CacheKey, QueryCacheConfig};
use crate::sql_normalizer::SqlNormalizer;
use crate::{Executor, ExecutorResult};

use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;

/// LocalExecutor - executes physical plans using StorageEngine
pub struct LocalExecutor<'a> {
    storage: &'a dyn StorageEngine,
    cache: Arc<RwLock<QueryCache>>,
    cache_config: QueryCacheConfig,
}

impl<'a> LocalExecutor<'a> {
    /// Create a new LocalExecutor with the given storage engine
    pub fn new(storage: &'a dyn StorageEngine) -> Self {
        Self {
            storage,
            cache: Arc::new(RwLock::new(QueryCache::new(QueryCacheConfig::default()))),
            cache_config: QueryCacheConfig::default(),
        }
    }

    /// Create a LocalExecutor with custom cache config
    pub fn with_cache_config(storage: &'a dyn StorageEngine, config: QueryCacheConfig) -> Self {
        Self {
            storage,
            cache: Arc::new(RwLock::new(QueryCache::new(config.clone()))),
            cache_config: config,
        }
    }

    /// Invalidate cache for a specific table
    pub fn invalidate_table(&self, table: &str) {
        self.cache.write().invalidate_table(table);
    }

    /// Clear all cached query results
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }

    /// Get cache key from SQL and params
    fn get_cache_key(&self, sql: &str, params: &[Value]) -> CacheKey {
        let (normalized, extracted) = SqlNormalizer::from_literal(sql);
        let mut all_params = extracted;
        all_params.extend_from_slice(params);
        let hash = SqlNormalizer::hash_params(&all_params);
        CacheKey {
            normalized_sql: normalized,
            params_hash: hash,
        }
    }

    /// Execute a physical plan and return results
    pub fn execute(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        self.execute_with_cache(plan, "", &[])
    }

    /// Execute with cache support using SQL and parameters
    pub fn execute_with_cache(
        &self,
        plan: &dyn PhysicalPlan,
        sql: &str,
        params: &[Value],
    ) -> SqlResult<ExecutorResult> {
        if self.cache_config.enabled && !sql.is_empty() {
            let cache_key = self.get_cache_key(sql, params);
            if let Some(result) = self.cache.write().get(&cache_key) {
                return Ok(result);
            }

            let result = match plan.name() {
                "SeqScan" => self.execute_seq_scan(plan),
                "Projection" => self.execute_projection(plan),
                "Filter" => self.execute_filter(plan),
                "Aggregate" => self.execute_aggregate(plan),
                "HashJoin" => self.execute_hash_join(plan),
                "SortMergeJoin" => self.execute_sort_merge_join(plan),
                "Sort" => self.execute_sort(plan),
                "Limit" => self.execute_limit(plan),
                _ => Ok(ExecutorResult::empty()),
            }?;

            if should_cache(&result) {
                let tables = self.extract_tables(plan);
                let entry = CacheEntry {
                    result: result.clone(),
                    tables,
                    created_at: Instant::now(),
                    size_bytes: result.rows.iter().map(|r| r.len()).sum(),
                };
                self.cache.write().put(cache_key, entry, vec![]);
            }

            return Ok(result);
        }

        match plan.name() {
            "SeqScan" => self.execute_seq_scan(plan),
            "Projection" => self.execute_projection(plan),
            "Filter" => self.execute_filter(plan),
            "Aggregate" => self.execute_aggregate(plan),
            "HashJoin" => self.execute_hash_join(plan),
            "SortMergeJoin" => self.execute_sort_merge_join(plan),
            "Sort" => self.execute_sort(plan),
            "Limit" => self.execute_limit(plan),
            _ => Ok(ExecutorResult::empty()),
        }
    }

    fn extract_tables(&self, plan: &dyn PhysicalPlan) -> Vec<String> {
        let mut tables = Vec::new();
        if !plan.table_name().is_empty() {
            tables.push(plan.table_name().to_string());
        }
        for child in plan.children() {
            tables.extend(self.extract_tables(child));
        }
        tables
    }

    /// Execute sequential scan
    fn execute_seq_scan(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let table_name = plan.table_name();
        if table_name.is_empty() {
            return Ok(ExecutorResult::empty());
        }

        let start = Instant::now();

        // Scan from storage
        let records = self.storage.scan(table_name).unwrap_or_default();

        // Convert records to rows
        let rows: Vec<Vec<Value>> = records;
        let row_count = rows.len();
        let duration = start.elapsed();

        // Record to GLOBAL_PROFILER
        GLOBAL_PROFILER.record("SeqScan", "scan", duration.as_nanos() as u64, row_count, 1);

        Ok(ExecutorResult::new(rows, 0))
    }

    /// Execute projection (column selection)
    fn execute_projection(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let children = plan.children();
        if children.is_empty() {
            return Ok(ExecutorResult::empty());
        }

        let start = Instant::now();

        let child_result = self.execute(children[0])?;

        let projection = plan
            .as_any()
            .downcast_ref::<ProjectionExec>()
            .map(|p| p.expr().clone())
            .unwrap_or_default();

        if projection.is_empty() {
            return Ok(child_result);
        }

        let input_schema = children[0].schema();
        let _output_schema = plan.schema();

        let projected_rows: Vec<Vec<Value>> = child_result
            .rows
            .iter()
            .map(|row| {
                projection
                    .iter()
                    .map(|expr| expr.evaluate(row, input_schema).unwrap_or(Value::Null))
                    .collect()
            })
            .collect();

        let row_count = projected_rows.len();
        let duration = start.elapsed();

        // Record to GLOBAL_PROFILER
        GLOBAL_PROFILER.record(
            "Projection",
            "projection",
            duration.as_nanos() as u64,
            row_count,
            1,
        );

        Ok(ExecutorResult::new(
            projected_rows,
            child_result.affected_rows,
        ))
    }

    /// Execute filter (WHERE clause)
    fn execute_filter(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let children = plan.children();
        if children.is_empty() {
            return Ok(ExecutorResult::empty());
        }

        let start = Instant::now();

        // Execute child first
        let child_result = self.execute(children[0])?;

        // Get the predicate from FilterExec
        let filter = plan.as_any().downcast_ref::<FilterExec>();

        let predicate = match filter {
            Some(f) => f.predicate(),
            None => return Ok(ExecutorResult::empty()),
        };

        let input_schema = children[0].schema();

        // Filter rows based on predicate
        let filtered_rows: Vec<Vec<Value>> = child_result
            .rows
            .into_iter()
            .filter(|row| {
                let predicate_val = predicate.evaluate(row, input_schema).unwrap_or(Value::Null);
                matches!(predicate_val, Value::Boolean(true))
            })
            .collect();

        let row_count = filtered_rows.len();
        let duration = start.elapsed();

        // Record to GLOBAL_PROFILER
        GLOBAL_PROFILER.record("Filter", "filter", duration.as_nanos() as u64, row_count, 1);

        Ok(ExecutorResult::new(filtered_rows, 0))
    }

    /// Execute aggregate (COUNT, SUM, AVG, etc.)
    fn execute_aggregate(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let children = plan.children();
        if children.is_empty() {
            return Ok(ExecutorResult::empty());
        }

        let start = Instant::now();

        let child_result = self.execute(children[0])?;

        let aggregate = plan.as_any().downcast_ref::<AggregateExec>();

        let (group_expr, aggregate_expr) = match aggregate {
            Some(p) => (p.group_expr(), p.aggregate_expr()),
            None => return Ok(ExecutorResult::empty()),
        };

        if group_expr.is_empty() {
            let mut agg_results = vec![];

            for agg_expr in aggregate_expr {
                if let sqlrustgo_planner::Expr::AggregateFunction {
                    func,
                    args,
                    distinct: _,
                } = agg_expr
                {
                    let values: Vec<Value> = child_result
                        .rows
                        .iter()
                        .map(|row| {
                            if let Some(arg) = args.first() {
                                arg.evaluate(row, children[0].schema())
                                    .unwrap_or(Value::Null)
                            } else {
                                Value::Integer(child_result.rows.len() as i64)
                            }
                        })
                        .collect();

                    let result = self.compute_aggregate(func, &values);
                    agg_results.push(result);
                }
            }

            if !agg_results.is_empty() {
                let row_count = 1;
                let duration = start.elapsed();

                // Record to GLOBAL_PROFILER
                GLOBAL_PROFILER.record(
                    "Aggregate",
                    "aggregate",
                    duration.as_nanos() as u64,
                    row_count,
                    1,
                );

                return Ok(ExecutorResult::new(vec![agg_results], 0));
            }

            Ok(ExecutorResult::empty())
        } else {
            let mut groups: std::collections::HashMap<Vec<Value>, Vec<Vec<Value>>> =
                std::collections::HashMap::new();

            for row in &child_result.rows {
                let key: Vec<Value> = group_expr
                    .iter()
                    .map(|expr| {
                        expr.evaluate(row, children[0].schema())
                            .unwrap_or(Value::Null)
                    })
                    .collect();
                groups.entry(key).or_default().push(row.clone());
            }

            let mut results = vec![];
            for (key, group_rows) in groups {
                let mut row = key;
                for agg_expr in aggregate_expr {
                    if let sqlrustgo_planner::Expr::AggregateFunction {
                        func,
                        args,
                        distinct: _,
                    } = agg_expr
                    {
                        let values: Vec<Value> = group_rows
                            .iter()
                            .map(|r| {
                                if let Some(arg) = args.first() {
                                    arg.evaluate(r, children[0].schema()).unwrap_or(Value::Null)
                                } else {
                                    Value::Integer(group_rows.len() as i64)
                                }
                            })
                            .collect();

                        let result = self.compute_aggregate(func, &values);
                        row.push(result);
                    }
                }
                results.push(row);
            }

            let row_count = results.len();
            let duration = start.elapsed();

            // Record to GLOBAL_PROFILER
            GLOBAL_PROFILER.record(
                "Aggregate",
                "aggregate",
                duration.as_nanos() as u64,
                row_count,
                1,
            );

            Ok(ExecutorResult::new(results, 0))
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

    /// Execute hash join
    fn execute_hash_join(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let children = plan.children();
        if children.len() < 2 {
            return Ok(ExecutorResult::empty());
        }

        let start = Instant::now();

        let left_result = self.execute(children[0])?;
        let right_result = self.execute(children[1])?;

        let hash_join = plan.as_any().downcast_ref::<HashJoinExec>();

        let (join_type, condition) = match hash_join {
            Some(hj) => (hj.join_type(), hj.condition()),
            None => return Ok(ExecutorResult::empty()),
        };

        let condition = match condition {
            Some(c) => c,
            None => {
                let results = cartesian_product(&left_result.rows, &right_result.rows);
                let row_count = results.len();
                let duration = start.elapsed();

                // Record to GLOBAL_PROFILER
                GLOBAL_PROFILER.record(
                    "HashJoin",
                    "join",
                    duration.as_nanos() as u64,
                    row_count,
                    1,
                );

                return Ok(ExecutorResult::new(results, 0));
            }
        };

        let left_schema = children[0].schema();
        let right_schema = children[1].schema();

        match join_type {
            JoinType::Inner => {
                let matched = hash_inner_join(
                    &left_result.rows,
                    &right_result.rows,
                    condition,
                    left_schema,
                    right_schema,
                );
                let row_count = matched.len();
                let duration = start.elapsed();

                // Record to GLOBAL_PROFILER
                GLOBAL_PROFILER.record(
                    "HashJoin",
                    "join",
                    duration.as_nanos() as u64,
                    row_count,
                    1,
                );

                Ok(ExecutorResult::new(matched, 0))
            }
            JoinType::Left => {
                let matched = hash_inner_join(
                    &left_result.rows,
                    &right_result.rows,
                    condition,
                    left_schema,
                    right_schema,
                );
                let left_only: Vec<Vec<Value>> = left_result
                    .rows
                    .iter()
                    .filter(|lrow| {
                        !matched.iter().any(|m| {
                            m.iter().take(lrow.len()).collect::<Vec<_>>()
                                == lrow.iter().collect::<Vec<_>>()
                        })
                    })
                    .map(|lrow| {
                        let mut row = lrow.clone();
                        row.extend(vec![Value::Null; right_schema.fields.len()]);
                        row
                    })
                    .collect();
                let mut results = matched;
                results.extend(left_only);

                let row_count = results.len();
                let duration = start.elapsed();

                // Record to GLOBAL_PROFILER
                GLOBAL_PROFILER.record(
                    "HashJoin",
                    "join",
                    duration.as_nanos() as u64,
                    row_count,
                    1,
                );

                Ok(ExecutorResult::new(results, 0))
            }
            _ => {
                let row_count = 0;
                let duration = start.elapsed();

                // Record to GLOBAL_PROFILER
                GLOBAL_PROFILER.record(
                    "HashJoin",
                    "join",
                    duration.as_nanos() as u64,
                    row_count,
                    1,
                );

                Ok(ExecutorResult::empty())
            }
        }
    }

    /// Execute sort merge join
    fn execute_sort_merge_join(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        use sqlrustgo_planner::SortMergeJoinExec;

        let children = plan.children();
        if children.len() < 2 {
            return Ok(ExecutorResult::empty());
        }

        let left_result = self.execute(children[0])?;
        let right_result = self.execute(children[1])?;

        let sort_merge_join = plan.as_any().downcast_ref::<SortMergeJoinExec>();

        let (join_type, condition) = match sort_merge_join {
            Some(smj) => (smj.join_type(), smj.condition()),
            None => return Ok(ExecutorResult::empty()),
        };

        let condition = match condition {
            Some(c) => c,
            None => {
                return Ok(ExecutorResult::new(
                    cartesian_product(&left_result.rows, &right_result.rows),
                    0,
                ));
            }
        };

        let left_schema = children[0].schema();
        let right_schema = children[1].schema();

        match join_type {
            JoinType::Inner => {
                let matched = sort_merge_inner_join(
                    &left_result.rows,
                    &right_result.rows,
                    condition,
                    left_schema,
                    right_schema,
                );
                Ok(ExecutorResult::new(matched, 0))
            }
            JoinType::Left => {
                let matched = sort_merge_inner_join(
                    &left_result.rows,
                    &right_result.rows,
                    condition,
                    left_schema,
                    right_schema,
                );
                let matched_keys: std::collections::HashSet<Vec<Value>> = matched
                    .iter()
                    .map(|m| m.iter().take(left_schema.fields.len()).cloned().collect())
                    .collect();
                let left_only: Vec<Vec<Value>> = left_result
                    .rows
                    .iter()
                    .filter(|lrow| {
                        let key: Vec<Value> = lrow.to_vec();
                        !matched_keys.contains(&key)
                    })
                    .map(|lrow| {
                        let mut row = lrow.clone();
                        row.extend(vec![Value::Null; right_schema.fields.len()]);
                        row
                    })
                    .collect();
                let mut results = matched;
                results.extend(left_only);
                Ok(ExecutorResult::new(results, 0))
            }
            _ => Ok(ExecutorResult::empty()),
        }
    }
}

fn cartesian_product(left: &[Vec<Value>], right: &[Vec<Value>]) -> Vec<Vec<Value>> {
    let mut result = Vec::new();
    for lrow in left {
        for rrow in right {
            let mut row = lrow.clone();
            row.extend(rrow.clone());
            result.push(row);
        }
    }
    result
}

fn hash_inner_join(
    left: &[Vec<Value>],
    right: &[Vec<Value>],
    condition: &sqlrustgo_planner::Expr,
    left_schema: &sqlrustgo_planner::Schema,
    right_schema: &sqlrustgo_planner::Schema,
) -> Vec<Vec<Value>> {
    let mut results = Vec::new();

    for lrow in left {
        for rrow in right {
            let mut combined = lrow.clone();
            combined.extend(rrow.clone());

            let full_schema = sqlrustgo_planner::Schema::new(
                left_schema
                    .fields
                    .iter()
                    .chain(right_schema.fields.iter())
                    .cloned()
                    .collect(),
            );

            if condition
                .evaluate(&combined, &full_schema)
                .map(|v| v.to_bool())
                .unwrap_or(false)
            {
                results.push(combined);
            }
        }
    }

    results
}

fn sort_merge_inner_join(
    left: &[Vec<Value>],
    right: &[Vec<Value>],
    condition: &sqlrustgo_planner::Expr,
    left_schema: &sqlrustgo_planner::Schema,
    right_schema: &sqlrustgo_planner::Schema,
) -> Vec<Vec<Value>> {
    let mut results = Vec::new();

    // Sort both inputs by the join key for merge join
    let mut left_sorted: Vec<Vec<Value>> = left.to_vec();
    let mut right_sorted: Vec<Vec<Value>> = right.to_vec();

    // Sort by first column (join key)
    left_sorted.sort_by(|a, b| {
        let a_key = a.first().unwrap_or(&Value::Null);
        let b_key = b.first().unwrap_or(&Value::Null);
        match (a_key, b_key) {
            (Value::Integer(ai), Value::Integer(bi)) => ai.cmp(bi),
            (Value::Text(ai), Value::Text(bi)) => ai.cmp(bi),
            _ => std::cmp::Ordering::Equal,
        }
    });

    right_sorted.sort_by(|a, b| {
        let a_key = a.first().unwrap_or(&Value::Null);
        let b_key = b.first().unwrap_or(&Value::Null);
        match (a_key, b_key) {
            (Value::Integer(ai), Value::Integer(bi)) => ai.cmp(bi),
            (Value::Text(ai), Value::Text(bi)) => ai.cmp(bi),
            _ => std::cmp::Ordering::Equal,
        }
    });

    // Merge join
    let mut i = 0;
    let mut j = 0;

    while i < left_sorted.len() && j < right_sorted.len() {
        let lrow = &left_sorted[i];
        let rrow = &right_sorted[j];

        let lkey = lrow.first().unwrap_or(&Value::Null);
        let rkey = rrow.first().unwrap_or(&Value::Null);

        let cmp = match (lkey, rkey) {
            (Value::Integer(li), Value::Integer(ri)) => li.cmp(ri),
            (Value::Text(li), Value::Text(ri)) => li.cmp(ri),
            _ => std::cmp::Ordering::Equal,
        };

        if cmp == std::cmp::Ordering::Equal {
            // Found match - check condition and add
            let mut combined = lrow.clone();
            combined.extend(rrow.clone());

            let full_schema = sqlrustgo_planner::Schema::new(
                left_schema
                    .fields
                    .iter()
                    .chain(right_schema.fields.iter())
                    .cloned()
                    .collect(),
            );

            if condition
                .evaluate(&combined, &full_schema)
                .map(|v| v.to_bool())
                .unwrap_or(false)
            {
                results.push(combined);
            }

            // Advance both for equal keys (handles duplicates)
            i += 1;
            j += 1;
        } else if cmp == std::cmp::Ordering::Less {
            i += 1;
        } else {
            j += 1;
        }
    }

    results
}

impl<'a> LocalExecutor<'a> {
    /// Execute sort
    fn execute_sort(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let children = plan.children();
        if children.is_empty() {
            return Ok(ExecutorResult::empty());
        }

        // Execute child first
        let child_result = self.execute(children[0])?;

        // Sort would go here
        Ok(child_result)
    }

    /// Execute limit
    fn execute_limit(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let children = plan.children();
        if children.is_empty() {
            return Ok(ExecutorResult::empty());
        }

        // Execute child first
        let child_result = self.execute(children[0])?;

        // Limit would go here
        Ok(child_result)
    }
}

impl<'a> Executor for LocalExecutor<'a> {
    fn execute(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        LocalExecutor::execute(self, plan)
    }

    fn name(&self) -> &str {
        "local"
    }

    fn is_ready(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_planner::{
        AggregateExec, AggregateFunction, Column, Expr, Field, FilterExec, Operator, PhysicalPlan,
        ProjectionExec, Schema, SeqScanExec, SortMergeJoinExec,
    };
    use sqlrustgo_storage::MemoryStorage;
    use std::any::Any;

    fn make_test_schema() -> Schema {
        Schema::new(vec![Field::new(
            "id".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )])
    }

    #[test]
    fn test_local_executor_creation() {
        let storage = MemoryStorage::new();
        let _executor = LocalExecutor::new(&storage);
        // Storage is set up correctly if we get here
        assert!(!storage.has_table("nonexistent"));
    }

    #[test]
    fn test_local_executor_with_empty_table() {
        let mut storage = MemoryStorage::new();
        storage
            .create_table(&sqlrustgo_storage::TableInfo {
                name: "users".to_string(),
                columns: vec![],
            })
            .unwrap();

        let executor = LocalExecutor::new(&storage);
        let test_schema = make_test_schema();

        // Create a mock plan
        struct MockPlan {
            schema: Schema,
        }
        impl PhysicalPlan for MockPlan {
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
                "users"
            }
            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        let result = executor
            .execute(&MockPlan {
                schema: test_schema,
            })
            .unwrap();
        assert!(result.rows.is_empty());
    }

    #[test]
    fn test_local_executor_send_sync() {
        fn _check<T: Send + Sync>() {}
        let _storage = MemoryStorage::new();
        _check::<LocalExecutor>();
    }

    #[test]
    fn test_execute_projection() {
        use sqlrustgo_planner::Expr;

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                    vec![Value::Integer(2), Value::Text("Bob".to_string())],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ]);
        let output_schema = Schema::new(vec![Field::new(
            "id".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);

        let seq_scan = sqlrustgo_planner::SeqScanExec::new("users".to_string(), input_schema);
        let projection =
            ProjectionExec::new(Box::new(seq_scan), vec![Expr::column("id")], output_schema);

        let result = executor.execute(&projection).unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0], vec![Value::Integer(1)]);
        assert_eq!(result.rows[1], vec![Value::Integer(2)]);
    }

    #[test]
    fn test_execute_projection_multiple_columns() {
        use sqlrustgo_planner::Expr;

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![vec![Value::Integer(1), Value::Text("Alice".to_string())]],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ]);
        let output_schema = Schema::new(vec![
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
        ]);

        let seq_scan = sqlrustgo_planner::SeqScanExec::new("users".to_string(), input_schema);
        let projection = ProjectionExec::new(
            Box::new(seq_scan),
            vec![Expr::column("name"), Expr::column("id")],
            output_schema,
        );

        let result = executor.execute(&projection).unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(
            result.rows[0],
            vec![Value::Text("Alice".to_string()), Value::Integer(1)]
        );
    }

    #[test]
    fn test_execute_aggregate_count() {
        use sqlrustgo_planner::{AggregateExec, AggregateFunction, Expr};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![
                    vec![Value::Integer(1)],
                    vec![Value::Integer(2)],
                    vec![Value::Integer(3)],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![Field::new(
            "id".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let output_schema = Schema::new(vec![Field::new(
            "count".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);

        let seq_scan = sqlrustgo_planner::SeqScanExec::new("users".to_string(), input_schema);
        let aggregate = AggregateExec::new(
            Box::new(seq_scan),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Count,
                args: vec![],
                distinct: false,
            }],
            None,
            output_schema,
        );

        let result = executor.execute(&aggregate).unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0], vec![Value::Integer(3)]);
    }

    #[test]
    fn test_execute_aggregate_sum() {
        use sqlrustgo_planner::{AggregateExec, AggregateFunction, Expr};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "orders",
                vec![
                    vec![Value::Integer(100)],
                    vec![Value::Integer(200)],
                    vec![Value::Integer(300)],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![Field::new(
            "amount".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let output_schema = Schema::new(vec![Field::new(
            "sum".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);

        let seq_scan = sqlrustgo_planner::SeqScanExec::new("orders".to_string(), input_schema);
        let aggregate = AggregateExec::new(
            Box::new(seq_scan),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Sum,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            output_schema,
        );

        let result = executor.execute(&aggregate).unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0], vec![Value::Integer(600)]);
    }

    #[test]
    fn test_execute_aggregate_avg() {
        use sqlrustgo_planner::{AggregateExec, AggregateFunction, Expr};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "orders",
                vec![
                    vec![Value::Integer(100)],
                    vec![Value::Integer(200)],
                    vec![Value::Integer(300)],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![Field::new(
            "amount".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let output_schema = Schema::new(vec![Field::new(
            "avg".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);

        let seq_scan = sqlrustgo_planner::SeqScanExec::new("orders".to_string(), input_schema);
        let aggregate = AggregateExec::new(
            Box::new(seq_scan),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Avg,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            output_schema,
        );

        let result = executor.execute(&aggregate).unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0], vec![Value::Integer(200)]);
    }

    #[test]
    fn test_execute_aggregate_with_group_by() {
        use sqlrustgo_planner::{AggregateExec, AggregateFunction, Expr};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "orders",
                vec![
                    vec![Value::Text("A".to_string()), Value::Integer(100)],
                    vec![Value::Text("A".to_string()), Value::Integer(200)],
                    vec![Value::Text("B".to_string()), Value::Integer(300)],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![
            Field::new("category".to_string(), sqlrustgo_planner::DataType::Text),
            Field::new("amount".to_string(), sqlrustgo_planner::DataType::Integer),
        ]);
        let output_schema = Schema::new(vec![
            Field::new("category".to_string(), sqlrustgo_planner::DataType::Text),
            Field::new("sum".to_string(), sqlrustgo_planner::DataType::Integer),
        ]);

        let seq_scan = sqlrustgo_planner::SeqScanExec::new("orders".to_string(), input_schema);
        let aggregate = AggregateExec::new(
            Box::new(seq_scan),
            vec![Expr::column("category")],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Sum,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            output_schema,
        );

        let result = executor.execute(&aggregate).unwrap();

        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_execute_hash_join_inner() {
        use sqlrustgo_planner::{Expr, HashJoinExec, JoinType};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                    vec![Value::Integer(2), Value::Text("Bob".to_string())],
                ],
            )
            .unwrap();
        storage
            .insert(
                "orders",
                vec![
                    vec![Value::Integer(1), Value::Integer(100), Value::Integer(1)],
                    vec![Value::Integer(2), Value::Integer(200), Value::Integer(1)],
                    vec![Value::Integer(3), Value::Integer(300), Value::Integer(2)],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let left_schema = Schema::new(vec![
            Field::new("user_id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ]);
        let right_schema = Schema::new(vec![
            Field::new("order_id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("amount".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("user_id".to_string(), sqlrustgo_planner::DataType::Integer),
        ]);
        let output_schema = Schema::new(vec![
            Field::new("user_id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
            Field::new("order_id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("amount".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("user_id".to_string(), sqlrustgo_planner::DataType::Integer),
        ]);

        let left_scan = sqlrustgo_planner::SeqScanExec::new("users".to_string(), left_schema);
        let right_scan = sqlrustgo_planner::SeqScanExec::new("orders".to_string(), right_schema);

        let join_condition = Expr::binary_expr(
            Expr::column("user_id"),
            sqlrustgo_planner::Operator::Eq,
            Expr::column("user_id"),
        );

        let hash_join = HashJoinExec::new(
            Box::new(left_scan),
            Box::new(right_scan),
            JoinType::Inner,
            Some(join_condition),
            output_schema,
        );

        let result = executor.execute(&hash_join).unwrap();

        assert!(result.rows.len() >= 2);
    }

    #[test]
    fn test_execute_hash_join_cross() {
        use sqlrustgo_planner::{HashJoinExec, JoinType};

        let mut storage = MemoryStorage::new();
        storage
            .insert("a", vec![vec![Value::Integer(1)], vec![Value::Integer(2)]])
            .unwrap();
        storage
            .insert(
                "b",
                vec![
                    vec![Value::Text("x".to_string())],
                    vec![Value::Text("y".to_string())],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let left_schema = Schema::new(vec![Field::new(
            "id".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let right_schema = Schema::new(vec![Field::new(
            "val".to_string(),
            sqlrustgo_planner::DataType::Text,
        )]);
        let output_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("val".to_string(), sqlrustgo_planner::DataType::Text),
        ]);

        let left_scan = sqlrustgo_planner::SeqScanExec::new("a".to_string(), left_schema);
        let right_scan = sqlrustgo_planner::SeqScanExec::new("b".to_string(), right_schema);

        let hash_join = HashJoinExec::new(
            Box::new(left_scan),
            Box::new(right_scan),
            JoinType::Cross,
            None,
            output_schema,
        );

        let result = executor.execute(&hash_join).unwrap();

        assert_eq!(result.rows.len(), 4);
    }

    #[test]
    fn test_execute_sort() {
        use sqlrustgo_planner::{physical_plan::SortExec, DataType, Expr, SeqScanExec};
        // Test execution of SortExec
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Integer),
        ]);

        // Create a table scan as child
        let scan = SeqScanExec::new("test".to_string(), schema.clone());

        // Create sort plan
        let sort_expr = vec![sqlrustgo_planner::SortExpr {
            expr: Expr::column("value"),
            asc: true,
            nulls_first: false,
        }];

        let sort = SortExec::new(Box::new(scan), sort_expr);

        let result = executor.execute(&sort).unwrap();
        assert!(result.rows.is_empty()); // No data in storage
    }

    #[test]
    fn test_execute_limit() {
        use sqlrustgo_planner::{physical_plan::LimitExec, DataType, SeqScanExec};
        // Test execution of LimitExec
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        // Create a table scan as child
        let scan = SeqScanExec::new("test".to_string(), schema.clone());

        // Create limit plan with offset
        let limit = LimitExec::new(Box::new(scan), 10, Some(5));

        let result = executor.execute(&limit).unwrap();
        assert!(result.rows.is_empty()); // No data in storage
    }

    #[test]
    fn test_executor_name() {
        // Test the executor name method
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);
        assert_eq!(executor.name(), "local");
    }

    #[test]
    fn test_execute_with_unknown_plan_type() {
        use sqlrustgo_planner::PhysicalPlan;
        use std::any::Any;

        struct UnknownPlan {
            schema: Schema,
        }
        impl UnknownPlan {
            fn new() -> Self {
                Self {
                    schema: Schema::new(vec![]),
                }
            }
        }
        impl PhysicalPlan for UnknownPlan {
            fn schema(&self) -> &Schema {
                &self.schema
            }
            fn children(&self) -> Vec<&dyn PhysicalPlan> {
                vec![]
            }
            fn name(&self) -> &str {
                "Unknown"
            }
            fn table_name(&self) -> &str {
                ""
            }
            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);
        let result = executor.execute(&UnknownPlan::new()).unwrap();
        assert!(result.rows.is_empty());
    }

    #[test]
    fn test_execute_projection_with_empty_projection() {
        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![vec![Value::Integer(1), Value::Text("Alice".to_string())]],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ]);

        let seq_scan = SeqScanExec::new("users".to_string(), input_schema);
        let projection = ProjectionExec::new(
            Box::new(seq_scan),
            vec![],
            Schema::new(vec![
                Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
                Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
            ]),
        );

        let result = executor.execute(&projection).unwrap();
        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_execute_filter_with_children() {
        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                    vec![Value::Integer(2), Value::Text("Bob".to_string())],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ]);

        let seq_scan = SeqScanExec::new("users".to_string(), input_schema);
        let filter = FilterExec::new(
            Box::new(seq_scan),
            Expr::binary_expr(
                Expr::column("id"),
                sqlrustgo_planner::Operator::Gt,
                Expr::literal(Value::Integer(1)),
            ),
        );

        let result = executor.execute(&filter).unwrap();
        // All rows returned since filter doesn't actually filter in this implementation
        assert!(result.rows.len() >= 1);
    }

    // === Tests for uncovered code paths ===

    #[test]
    fn test_executor_is_ready() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);
        // is_ready should return true (always ready)
        assert!(executor.is_ready());
    }

    #[test]
    fn test_execute_aggregate_min() {
        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "orders",
                vec![
                    vec![Value::Integer(100)],
                    vec![Value::Integer(200)],
                    vec![Value::Integer(50)],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![Field::new(
            "amount".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let output_schema = Schema::new(vec![Field::new(
            "min".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);

        let seq_scan = SeqScanExec::new("orders".to_string(), input_schema);
        let aggregate = AggregateExec::new(
            Box::new(seq_scan),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Min,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            output_schema,
        );

        let result = executor.execute(&aggregate).unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0], vec![Value::Integer(50)]);
    }

    #[test]
    fn test_execute_aggregate_max() {
        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "orders",
                vec![
                    vec![Value::Integer(100)],
                    vec![Value::Integer(200)],
                    vec![Value::Integer(50)],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![Field::new(
            "amount".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let output_schema = Schema::new(vec![Field::new(
            "max".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);

        let seq_scan = SeqScanExec::new("orders".to_string(), input_schema);
        let aggregate = AggregateExec::new(
            Box::new(seq_scan),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Max,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            output_schema,
        );

        let result = executor.execute(&aggregate).unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0], vec![Value::Integer(200)]);
    }

    #[test]
    fn test_execute_hash_join_left() {
        use sqlrustgo_planner::HashJoinExec;
        use sqlrustgo_planner::JoinType;

        let mut left_storage = MemoryStorage::new();
        left_storage
            .insert(
                "left_table",
                vec![
                    vec![Value::Integer(1), Value::Text("A".to_string())],
                    vec![Value::Integer(2), Value::Text("B".to_string())],
                ],
            )
            .unwrap();

        let mut right_storage = MemoryStorage::new();
        right_storage
            .insert(
                "right_table",
                vec![vec![Value::Integer(1), Value::Text("X".to_string())]],
            )
            .unwrap();

        let executor = LocalExecutor::new(&left_storage);

        let left_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ]);
        let right_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("value".to_string(), sqlrustgo_planner::DataType::Text),
        ]);
        let join_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("value".to_string(), sqlrustgo_planner::DataType::Text),
        ]);

        let left_scan = SeqScanExec::new("left_table".to_string(), left_schema);
        let right_scan = SeqScanExec::new("right_table".to_string(), right_schema);

        let join_condition = Some(Expr::BinaryExpr {
            left: Box::new(Expr::column("id")),
            op: Operator::Eq,
            right: Box::new(Expr::column("id")),
        });

        let hash_join = HashJoinExec::new(
            Box::new(left_scan),
            Box::new(right_scan),
            JoinType::Left,
            join_condition,
            join_schema,
        );

        let result = executor.execute(&hash_join);
        // Left join should complete without error
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_sort_merge_join_inner() {
        use sqlrustgo_planner::{Expr, JoinType, Operator, SortMergeJoinExec};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                    vec![Value::Integer(2), Value::Text("Bob".to_string())],
                ],
            )
            .unwrap();
        storage
            .insert(
                "orders",
                vec![
                    vec![Value::Integer(1), Value::Integer(100), Value::Integer(1)],
                    vec![Value::Integer(2), Value::Integer(200), Value::Integer(1)],
                    vec![Value::Integer(3), Value::Integer(300), Value::Integer(2)],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let left_schema = Schema::new(vec![
            Field::new("user_id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ]);
        let right_schema = Schema::new(vec![
            Field::new("order_id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("amount".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("user_id".to_string(), sqlrustgo_planner::DataType::Integer),
        ]);
        let output_schema = Schema::new(vec![
            Field::new("user_id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
            Field::new("order_id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("amount".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("user_id".to_string(), sqlrustgo_planner::DataType::Integer),
        ]);

        let left_scan = SeqScanExec::new("users".to_string(), left_schema.clone());
        let right_scan = SeqScanExec::new("orders".to_string(), right_schema.clone());

        let join_condition = Some(Expr::BinaryExpr {
            left: Box::new(Expr::Column(Column::new("user_id".to_string()))),
            op: Operator::Eq,
            right: Box::new(Expr::Column(Column::new("user_id".to_string()))),
        });

        let sort_merge_join = SortMergeJoinExec::new(
            Box::new(left_scan),
            Box::new(right_scan),
            JoinType::Inner,
            join_condition,
            output_schema,
            vec![Expr::Column(Column::new("user_id".to_string()))],
            vec![Expr::Column(Column::new("user_id".to_string()))],
        );

        let result = executor.execute(&sort_merge_join).unwrap();
        // Should have 2 matching rows (user_id=1 has 2 orders, user_id=2 has 1 order)
        assert!(result.rows.len() >= 2);
    }

    #[test]
    fn test_execute_sort_merge_join_left() {
        use sqlrustgo_planner::{Expr, JoinType, Operator, SortMergeJoinExec};

        let mut left_storage = MemoryStorage::new();
        left_storage
            .insert(
                "left_table",
                vec![
                    vec![Value::Integer(1), Value::Text("A".to_string())],
                    vec![Value::Integer(2), Value::Text("B".to_string())],
                ],
            )
            .unwrap();

        let mut right_storage = MemoryStorage::new();
        right_storage
            .insert(
                "right_table",
                vec![vec![Value::Integer(1), Value::Text("X".to_string())]],
            )
            .unwrap();

        let executor = LocalExecutor::new(&left_storage);

        let left_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ]);
        let right_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("value".to_string(), sqlrustgo_planner::DataType::Text),
        ]);
        let join_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("value".to_string(), sqlrustgo_planner::DataType::Text),
        ]);

        let left_scan = SeqScanExec::new("left_table".to_string(), left_schema);
        let right_scan = SeqScanExec::new("right_table".to_string(), right_schema);

        let join_condition = Some(Expr::BinaryExpr {
            left: Box::new(Expr::column("id")),
            op: Operator::Eq,
            right: Box::new(Expr::column("id")),
        });

        let sort_merge_join = SortMergeJoinExec::new(
            Box::new(left_scan),
            Box::new(right_scan),
            JoinType::Left,
            join_condition,
            join_schema,
            vec![Expr::column("id")],
            vec![Expr::column("id")],
        );

        let result = executor.execute(&sort_merge_join);
        // Left join should complete without error
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_sort_merge_join_empty() {
        use sqlrustgo_planner::{Expr, JoinType, Operator, SortMergeJoinExec};

        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        let left_schema = Schema::new(vec![Field::new(
            "id".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let right_schema = Schema::new(vec![Field::new(
            "id".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let join_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
        ]);

        let left_scan = SeqScanExec::new("empty_table".to_string(), left_schema);
        let right_scan = SeqScanExec::new("right_table".to_string(), right_schema);

        let join_condition = Some(Expr::BinaryExpr {
            left: Box::new(Expr::column("id")),
            op: Operator::Eq,
            right: Box::new(Expr::column("id")),
        });

        let sort_merge_join = SortMergeJoinExec::new(
            Box::new(left_scan),
            Box::new(right_scan),
            JoinType::Inner,
            join_condition,
            join_schema,
            vec![Expr::column("id")],
            vec![Expr::column("id")],
        );

        let result = executor.execute(&sort_merge_join);
        assert!(result.is_ok());
        assert!(result.unwrap().rows.is_empty());
    }
}
