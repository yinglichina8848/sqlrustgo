//! ExecutionEngine - high-level SQL execution API
//! Provides a simple interface for executing SQL statements against a storage backend.

#![allow(unused_variables, unused_imports)]

use crate::{parse, SqlError, SqlResult, Value};
use sqlrustgo_catalog::stored_proc::{ParamMode, StoredProcParam, StoredProcStatement};
use sqlrustgo_catalog::{auth::UserIdentity, Catalog, StoredProcedure};
use sqlrustgo_executor::query_cache::{should_cache, QueryCache};
use sqlrustgo_executor::query_cache_config::{CacheEntry, CacheKey, QueryCacheConfig};
use sqlrustgo_executor::stored_proc::StoredProcExecutor;
use sqlrustgo_executor::trigger::{
    TriggerEvent as ExecTriggerEvent, TriggerExecutor, TriggerTiming as ExecTriggerTiming,
};
use sqlrustgo_executor::ExecutorResult;
use sqlrustgo_parser::parser::{
    AggregateCall, AggregateFunction, CallStatement, CreateIndexStatement,
    CreateProcedureStatement, CreateRoleStatement, CreateTableStatement, CreateTriggerStatement,
    DropRoleStatement, DropTableStatement, GrantRoleStatement, GrantStatement, InsertStatement,
    ObjectType as ParserObjectType, Privilege as ParserPrivilege, RevokeRoleStatement,
    RevokeStatement, SelectStatement, SetRoleStatement, StoredProcParam as ParserStoredProcParam,
    StoredProcParamMode as ParserParamMode, StoredProcStatement as ParserStatement,
    TruncateStatement,
};
use sqlrustgo_parser::transaction::IsolationLevel as ParserIsolationLevel;
use sqlrustgo_parser::JoinType; // For join type matching
use sqlrustgo_parser::{
    DeleteStatement, Expression, Statement, TransactionStatement, UpdateStatement,
};
use sqlrustgo_storage::{ColumnDefinition, MemoryStorage, StorageEngine, TableInfo};
use sqlrustgo_transaction::{IsolationLevel as TmIsolationLevel, TransactionManager, TxId};
use sqlrustgo_types::Value as SqlValue;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Execution engine for SQL statements
pub struct ExecutionEngine<S: StorageEngine> {
    storage: Arc<RwLock<S>>,
    catalog: Option<Arc<RwLock<Catalog>>>,
    stats: Arc<RwLock<ExecutionStats>>,
    cbo_enabled: bool,
    transaction_manager: TransactionManager,
    current_tx_id: Option<TxId>,
    default_isolation: TmIsolationLevel,
    current_role: Option<String>,
    query_cache: Arc<RwLock<QueryCache>>,
    cache_config: QueryCacheConfig,
}

/// Execution statistics for CBO
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    pub table_stats: HashMap<String, TableStatistics>,
}

/// Table-level statistics for query optimization
#[derive(Debug, Clone)]
pub struct TableStatistics {
    pub row_count: u64,
    pub column_stats: HashMap<String, ColumnStatistics>,
}

/// Column-level statistics
#[derive(Debug, Clone)]
pub struct ColumnStatistics {
    pub null_count: u64,
    pub distinct_count: u64,
    pub min_value: Option<SqlValue>,
    pub max_value: Option<SqlValue>,
}

/// Type alias for MemoryStorage-backed execution engine
pub type MemoryExecutionEngine = ExecutionEngine<MemoryStorage>;

impl<S: StorageEngine + 'static> ExecutionEngine<S> {
    /// Create a new execution engine with CBO enabled by default
    pub fn new(storage: Arc<RwLock<S>>) -> Self {
        Self {
            storage,
            catalog: None,
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
            cbo_enabled: true,
            transaction_manager: TransactionManager::new(),
            current_tx_id: None,
            default_isolation: TmIsolationLevel::default(),
            current_role: None,
            query_cache: Arc::new(RwLock::new(QueryCache::new(QueryCacheConfig::default()))),
            cache_config: QueryCacheConfig::default(),
        }
    }

    /// Create a new execution engine with CBO configuration
    pub fn with_cbo(storage: Arc<RwLock<S>>, cbo_enabled: bool) -> Self {
        Self {
            storage,
            catalog: None,
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
            cbo_enabled,
            transaction_manager: TransactionManager::new(),
            current_tx_id: None,
            default_isolation: TmIsolationLevel::default(),
            current_role: None,
            query_cache: Arc::new(RwLock::new(QueryCache::new(QueryCacheConfig::default()))),
            cache_config: QueryCacheConfig::default(),
        }
    }

    /// Create a new execution engine with a catalog
    pub fn with_catalog(storage: Arc<RwLock<S>>, catalog: Arc<RwLock<Catalog>>) -> Self {
        Self {
            storage,
            catalog: Some(catalog),
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
            cbo_enabled: true,
            transaction_manager: TransactionManager::new(),
            current_tx_id: None,
            default_isolation: TmIsolationLevel::default(),
            current_role: None,
            query_cache: Arc::new(RwLock::new(QueryCache::new(QueryCacheConfig::default()))),
            cache_config: QueryCacheConfig::default(),
        }
    }

    /// Check if CBO is enabled
    pub fn is_cbo_enabled(&self) -> bool {
        self.cbo_enabled
    }

    /// Enable or disable CBO
    pub fn set_cbo_enabled(&mut self, enabled: bool) {
        self.cbo_enabled = enabled;
    }

    /// Get table statistics for CBO
    pub fn get_table_stats(&self) -> Arc<RwLock<ExecutionStats>> {
        self.stats.clone()
    }

    /// Estimate the number of rows returned by a query based on statistics
    /// This is a building block for cost-based optimization
    pub fn estimate_row_count(&self, table_name: &str) -> u64 {
        let stats = self.stats.read().unwrap();
        stats
            .table_stats
            .get(table_name)
            .map(|s| s.row_count)
            .unwrap_or(1000) // Default estimate
    }

    /// Estimate the selectivity of a predicate based on column statistics
    /// Returns a value between 0.0 and 1.0 representing the fraction of rows that match
    pub fn estimate_selectivity(&self, table_name: &str, column_name: &str) -> f64 {
        let stats = self.stats.read().unwrap();
        if let Some(table_stats) = stats.table_stats.get(table_name) {
            if let Some(col_stats) = table_stats.column_stats.get(column_name) {
                if col_stats.distinct_count > 0 {
                    return 1.0 / col_stats.distinct_count as f64;
                }
            }
        }
        0.1 // Default: assume 10% selectivity
    }

    /// Estimate the cost of a sequential scan
    pub fn estimate_seq_scan_cost(&self, table_name: &str) -> f64 {
        let rows = self.estimate_row_count(table_name);
        rows as f64 * 1.0 // Each row has unit cost
    }

    /// Estimate the cost of an index scan
    /// selectivity: fraction of rows that match the predicate
    pub fn estimate_index_scan_cost(&self, table_name: &str, selectivity: f64) -> f64 {
        let rows = self.estimate_row_count(table_name);
        // Index scan cost = index lookup cost + random I/O for matching rows
        let index_lookup_cost = 10.0; // Fixed overhead for index access
        let random_io_cost = (rows as f64 * selectivity) * 0.5; // Random I/O per match
        index_lookup_cost + random_io_cost
    }

    /// Estimate the benefit (cost reduction) of using an index vs sequential scan
    /// Returns positive value if index is beneficial, negative if sequential scan is better
    pub fn estimate_index_benefit(&self, table_name: &str, selectivity: f64) -> f64 {
        let seq_cost = self.estimate_seq_scan_cost(table_name);
        let index_cost = self.estimate_index_scan_cost(table_name, selectivity);
        seq_cost - index_cost
    }

    /// Decide whether to use index scan or sequential scan based on cost estimation
    /// Returns true if index scan is recommended
    pub fn should_use_index(&self, table_name: &str, column_name: &str) -> bool {
        let selectivity = self.estimate_selectivity(table_name, column_name);
        let benefit = self.estimate_index_benefit(table_name, selectivity);
        benefit > 0.0
    }

    /// Estimate the cost of a join between two tables
    /// join_type: "hash", "nested_loop", "merge"
    pub fn estimate_join_cost(&self, left_table: &str, right_table: &str, join_type: &str) -> f64 {
        let left_rows = self.estimate_row_count(left_table);
        let right_rows = self.estimate_row_count(right_table);

        match join_type {
            "hash" => {
                // Hash join cost = build + probe
                let build_cost = right_rows as f64 * 0.8;
                let probe_cost = left_rows as f64 * 0.8;
                build_cost + probe_cost
            }
            "merge" => {
                // Merge join cost = sort + merge
                let left_sort = left_rows as f64 * 0.5 * (left_rows as f64).log2();
                let right_sort = right_rows as f64 * 0.5 * (right_rows as f64).log2();
                left_sort + right_sort + (left_rows + right_rows) as f64 * 0.1
            }
            _ => {
                // Nested loop: outer * inner
                let outer_cost = left_rows as f64;
                let inner_cost = right_rows as f64 * 0.1; // Assuming index on inner
                outer_cost + outer_cost * inner_cost
            }
        }
    }

    /// Find the optimal join order using a greedy algorithm
    /// Returns tables in optimal join order (smallest first)
    pub fn optimize_join_order<'a>(&self, tables: &'a [&str]) -> Vec<&'a str> {
        if tables.len() <= 1 {
            return tables.to_vec();
        }

        let mut remaining: Vec<&str> = tables.to_vec();
        let mut result: Vec<&str> = Vec::new();

        while !remaining.is_empty() {
            let candidate = if result.is_empty() {
                remaining
                    .iter()
                    .min_by(|a, b| self.estimate_row_count(a).cmp(&self.estimate_row_count(b)))
                    .copied()
            } else {
                remaining
                    .iter()
                    .min_by(|a, b| {
                        let cost_a = self.estimate_join_cost(result.last().unwrap(), a, "hash");
                        let cost_b = self.estimate_join_cost(result.last().unwrap(), b, "hash");
                        cost_a.partial_cmp(&cost_b).unwrap()
                    })
                    .copied()
            };

            if let Some(t) = candidate {
                result.push(t);
                remaining.retain(|x| *x != t);
            } else {
                break;
            }
        }

        result
    }

    /// Collect statistics for a table (ANALYZE)
    fn collect_table_stats(&self, table: &str) -> SqlResult<TableStatistics> {
        let storage = self.storage.read().unwrap();
        let rows = storage.scan(table)?;
        let row_count = rows.len() as u64;

        let table_info = storage.get_table_info(table)?;

        let mut column_stats = HashMap::new();
        for col in &table_info.columns {
            let mut null_count = 0u64;
            let mut distinct_values = std::collections::HashSet::new();
            let mut min_value: Option<SqlValue> = None;
            let mut max_value: Option<SqlValue> = None;

            let col_idx = table_info
                .columns
                .iter()
                .position(|c| c.name == col.name)
                .unwrap_or(0);

            for row in &rows {
                if let Some(val) = row.get(col_idx) {
                    if val == &SqlValue::Null {
                        null_count += 1;
                    } else {
                        distinct_values.insert(format!("{:?}", val));
                        match val {
                            SqlValue::Integer(n) => {
                                min_value = Some(SqlValue::Integer(*n));
                                max_value = Some(SqlValue::Integer(*n));
                            }
                            SqlValue::Text(s) => {
                                let cmp_min = min_value
                                    .as_ref()
                                    .and_then(|v| {
                                        if let SqlValue::Text(ms) = v {
                                            Some(ms < s)
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or(true);
                                let cmp_max = max_value
                                    .as_ref()
                                    .and_then(|v| {
                                        if let SqlValue::Text(ms) = v {
                                            Some(ms > s)
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or(true);
                                if cmp_min {
                                    min_value = Some(SqlValue::Text(s.clone()));
                                }
                                if cmp_max {
                                    max_value = Some(SqlValue::Text(s.clone()));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            column_stats.insert(
                col.name.clone(),
                ColumnStatistics {
                    null_count,
                    distinct_count: distinct_values.len() as u64,
                    min_value,
                    max_value,
                },
            );
        }

        Ok(TableStatistics {
            row_count,
            column_stats,
        })
    }

    /// Execute a pre-parsed SQL statement and return results
    /// This avoids re-parsing on every execute for prepared statements
    pub fn execute_statement(&mut self, statement: Statement) -> SqlResult<ExecutorResult> {
        match statement {
            Statement::Select(ref select) => self.execute_select(select),
            Statement::Insert(ref insert) => self.execute_insert(insert),
            Statement::Update(ref update) => self.execute_update(update),
            Statement::Delete(ref delete) => self.execute_delete(delete),
            Statement::CreateTable(ref create) => self.execute_create_table(create),
            Statement::DropTable(ref drop) => self.execute_drop_table(drop),
            Statement::Truncate(ref truncate) => self.execute_truncate(truncate),
            Statement::CreateIndex(ref idx) => self.execute_create_index(idx),
            Statement::Analyze(ref analyze) => {
                let table_name = analyze.table_name.as_ref().ok_or_else(|| {
                    SqlError::ExecutionError("ANALYZE: table name is required".to_string())
                })?;
                let stats = self.collect_table_stats(table_name)?;
                let row_count = stats.row_count;

                let mut stats_guard = self.stats.write().unwrap();
                stats_guard.table_stats.insert(table_name.clone(), stats);

                Ok(ExecutorResult::new(
                    vec![vec![Value::Integer(row_count as i64)]],
                    1,
                ))
            }
            Statement::Union(ref union_stmt) => {
                let left_select = match union_stmt.left.as_ref() {
                    Statement::Select(s) => s,
                    _ => {
                        return Err(SqlError::ExecutionError(
                            "UNION left side must be a SELECT".to_string(),
                        ))
                    }
                };
                let right_select = match union_stmt.right.as_ref() {
                    Statement::Select(s) => s,
                    _ => {
                        return Err(SqlError::ExecutionError(
                            "UNION right side must be a SELECT".to_string(),
                        ))
                    }
                };

                let mut left_result = self.execute_select(left_select)?;
                let right_result = self.execute_select(right_select)?;

                left_result.rows.extend(right_result.rows);

                if !union_stmt.union_all {
                    left_result.rows.sort();
                    left_result.rows.dedup();
                }

                left_result.affected_rows = left_result.rows.len();
                Ok(left_result)
            }
            Statement::CreateTrigger(ref create_trigger) => {
                self.execute_create_trigger(create_trigger)
            }
            Statement::Call(ref call) => self.execute_call(call),
            Statement::CreateProcedure(ref create_proc) => {
                self.execute_create_procedure(create_proc)
            }
            Statement::Transaction(ref txn) => self.execute_transaction(txn),
            Statement::Grant(ref grant) => self.execute_grant(grant),
            Statement::Revoke(ref revoke) => self.execute_revoke(revoke),
            Statement::CreateRole(ref stmt) => self.execute_create_role(stmt),
            Statement::DropRole(ref stmt) => self.execute_drop_role(stmt),
            Statement::GrantRole(ref stmt) => self.execute_grant_role(stmt),
            Statement::RevokeRole(ref stmt) => self.execute_revoke_role(stmt),
            Statement::SetRole(ref stmt) => self.execute_set_role(stmt),
            Statement::ShowRoles => self.execute_show_roles(),
            Statement::ShowGrantsFor(ref user) => self.execute_show_grants_for(user),
            _ => Err(SqlError::ExecutionError(
                format!("Unsupported statement type: {:?}", std::mem::discriminant(&statement)),
            )),
        }
    }

    /// Execute a SQL statement and return results
    pub fn execute(&mut self, sql: &str) -> SqlResult<ExecutorResult> {
        if !sql.trim().is_empty() && self.cache_config.enabled {
            let cache_key = self.get_cache_key(sql);
            if let Some(result) = self.query_cache.write().unwrap().get(&cache_key) {
                return Ok(result);
            }

            let statement = parse(sql).map_err(|e| SqlError::ParseError(e.to_string()))?;
            let result = self.execute_statement(statement)?;

            if should_cache(&result) {
                let entry = CacheEntry {
                    result: result.clone(),
                    tables: vec![],
                    created_at: std::time::Instant::now(),
                    size_bytes: result.rows.iter().map(|r| r.len()).sum(),
                    last_access: 0,
                };
                self.query_cache.write().unwrap().put(cache_key, entry, vec![]);
            }

            return Ok(result);
        }

        let statement = parse(sql).map_err(|e| SqlError::ParseError(e.to_string()))?;
        self.execute_statement(statement)
    }

    fn get_cache_key(&self, sql: &str) -> CacheKey {
        CacheKey {
            normalized_sql: sql.trim().to_lowercase(),
            params_hash: 0,
        }
    }

    fn execute_select(&self, select: &SelectStatement) -> SqlResult<ExecutorResult> {
        let storage = self.storage.read().unwrap();

        if select.table.is_empty() {
            return Ok(ExecutorResult::new(vec![], 0));
        }

        // Step 1: FROM/JOIN - get initial rows and schema
        let (mut rows, table_info) = if select.join_clause.is_some() {
            self.execute_join(select)?
        } else {
            let rows = storage.scan(&select.table)?;
            let table_info = storage.get_table_info(&select.table)?;
            (rows, table_info)
        };

        // Step 2: WHERE
        if let Some(ref where_expr) = select.where_clause {
            rows.retain(|row| eval_predicate(where_expr, row, &table_info));
        }

        // Step 3: GROUP BY + AGGREGATE
        if !select.aggregates.is_empty() {
            let group_exprs = &select.group_by;
            if group_exprs.is_empty() {
                let mut agg_values =
                    self.compute_aggregates(&select.aggregates, &rows, &table_info)?;

                if let Some(ref having_expr) = select.having {
                    let having_schema = build_aggregate_schema(&[], &select.aggregates)?;
                    if !eval_predicate(having_expr, &agg_values, &having_schema) {
                        return Ok(ExecutorResult::new(vec![], 0));
                    }
                }

                return Ok(ExecutorResult::new(vec![agg_values], 1));
            } else {
                let mut groups: std::collections::HashMap<String, Vec<Vec<Value>>> =
                    std::collections::HashMap::new();
                for row in &rows {
                    let key = group_exprs
                        .iter()
                        .map(|expr| evaluate_expr_to_string(expr, row, &table_info))
                        .collect::<Vec<_>>()
                        .join("\x00");
                    groups.entry(key).or_default().push(row.clone());
                }

                let mut agg_result_rows: Vec<Vec<Value>> = Vec::new();

                for (key, group_rows) in groups.iter() {
                    let key_values: Vec<Value> = key
                        .split('\x00')
                        .map(|s| {
                            if s == "NULL" {
                                Value::Null
                            } else if let Ok(n) = s.parse::<i64>() {
                                Value::Integer(n)
                            } else {
                                Value::Text(s.to_string())
                            }
                        })
                        .collect();
                    let agg_values =
                        self.compute_aggregates(&select.aggregates, group_rows, &table_info)?;
                    let mut combined = key_values;
                    combined.extend(agg_values);
                    agg_result_rows.push(combined);
                }

                if let Some(ref having_expr) = select.having {
                    let having_schema = build_aggregate_schema(group_exprs, &select.aggregates)?;
                    agg_result_rows.retain(|row| eval_predicate(having_expr, row, &having_schema));
                }

                let row_count = agg_result_rows.len();
                return Ok(ExecutorResult::new(agg_result_rows, row_count));
            }
        }

        // Step 4: LIMIT / OFFSET
        let limited_rows = if let Some(limit) = select.limit {
            let offset = select.offset.unwrap_or(0);
            if offset as usize >= rows.len() {
                vec![]
            } else {
                rows.into_iter()
                    .skip(offset as usize)
                    .take(limit as usize)
                    .collect()
            }
        } else {
            rows
        };

        let row_count = limited_rows.len();
        Ok(ExecutorResult::new(limited_rows, row_count))
    }

    fn compute_aggregates(
        &self,
        aggregates: &[AggregateCall],
        rows: &[Vec<Value>],
        table_info: &TableInfo,
    ) -> SqlResult<Vec<Value>> {
        let mut results = Vec::with_capacity(aggregates.len());
        for agg in aggregates {
            let values: Vec<Value> = if let Some(arg) = agg.args.first() {
                rows.iter()
                    .map(|row| evaluate_expression(arg, row, table_info).unwrap_or(Value::Null))
                    .collect()
            } else {
                vec![Value::Integer(rows.len() as i64)]
            };

            let result = match agg.func {
                AggregateFunction::Count => {
                    if agg.args.is_empty() {
                        // COUNT(*) - count all rows
                        Value::Integer(rows.len() as i64)
                    } else {
                        // COUNT(col) - count non-NULL values
                        let non_null_count =
                            values.iter().filter(|v| !matches!(v, Value::Null)).count();
                        Value::Integer(non_null_count as i64)
                    }
                }
                AggregateFunction::Sum => {
                    let int_values: Vec<i64> = values
                        .iter()
                        .filter_map(|v| {
                            if let Value::Integer(n) = v {
                                Some(*n)
                            } else {
                                None
                            }
                        })
                        .collect();
                    if int_values.is_empty() {
                        Value::Null
                    } else {
                        Value::Integer(int_values.iter().sum())
                    }
                }
                AggregateFunction::Avg => {
                    let sum: i64 = values
                        .iter()
                        .filter_map(|v| {
                            if let Value::Integer(n) = v {
                                Some(*n)
                            } else {
                                None
                            }
                        })
                        .sum();
                    let count = values
                        .iter()
                        .filter(|v| matches!(v, Value::Integer(_)))
                        .count();
                    if count > 0 {
                        Value::Integer(sum / count as i64)
                    } else {
                        Value::Null
                    }
                }
                AggregateFunction::Min => {
                    let min = values
                        .iter()
                        .filter_map(|v| {
                            if let Value::Integer(n) = v {
                                Some(*n)
                            } else {
                                None
                            }
                        })
                        .min();
                    min.map(Value::Integer).unwrap_or(Value::Null)
                }
                AggregateFunction::Max => {
                    let max = values
                        .iter()
                        .filter_map(|v| {
                            if let Value::Integer(n) = v {
                                Some(*n)
                            } else {
                                None
                            }
                        })
                        .max();
                    max.map(Value::Integer).unwrap_or(Value::Null)
                }
            };
            results.push(result);
        }
        Ok(results)
    }

    /// Execute JOIN and return (rows, combined_schema)
    /// This function only generates joined rows, does NOT apply WHERE/AGG/HAVING
    fn execute_join(&self, select: &SelectStatement) -> SqlResult<(Vec<Vec<Value>>, TableInfo)> {
        use sqlrustgo_parser::JoinType as ParserJoinType;
        use std::collections::HashMap;

        let join_clause = select.join_clause.as_ref().unwrap();
        let left_table_name = select.table.clone();
        let right_table_name = join_clause.table.clone();

        let storage = self.storage.read().unwrap();

        // Scan both tables
        let left_rows = storage.scan(&left_table_name)?;
        let right_rows = storage.scan(&right_table_name)?;

        // Get table info for column indices
        let left_table_info = storage.get_table_info(&left_table_name)?;
        let right_table_info = storage.get_table_info(&right_table_name)?;

        // Extract join key column index from ON clause
        // For "t1.id = t2.id", we need to find which column "id" refers to in each table
        let left_key_idx =
            self.find_join_key_index(&join_clause.on_clause, &left_table_info, &select.table)?;
        let right_key_idx =
            self.find_join_key_index(&join_clause.on_clause, &right_table_info, &right_table_name)?;

        // Determine join type
        let join_type = match join_clause.join_type {
            ParserJoinType::Inner => JoinType::Inner,
            ParserJoinType::Left => JoinType::Left,
            ParserJoinType::Right => JoinType::Right,
            ParserJoinType::Full => JoinType::Full,
            ParserJoinType::Cross => JoinType::Cross,
        };

        let left_col_count = left_table_info.columns.len();
        let right_col_count = right_table_info.columns.len();

        let mut matched_results = match join_type {
            JoinType::Inner | JoinType::Left | JoinType::Right | JoinType::Full => {
                // Hash-based matching
                // SQL semantics: NULL = NULL is UNKNOWN (not a match), so skip NULL keys
                let mut right_hash: HashMap<String, Vec<Vec<Value>>> = HashMap::new();
                for right_row in &right_rows {
                    if matches!(right_row[right_key_idx], Value::Null) {
                        // NULL keys can never match in a join
                        continue;
                    }
                    let key = format!("{:?}", right_row[right_key_idx]);
                    right_hash.entry(key).or_default().push(right_row.clone());
                }

                let mut matched: Vec<Vec<Value>> = Vec::new();
                let mut left_matched: std::collections::HashSet<usize> =
                    std::collections::HashSet::new();
                let mut right_matched: std::collections::HashSet<usize> =
                    std::collections::HashSet::new();

                // Match left rows to right
                for (li, left_row) in left_rows.iter().enumerate() {
                    // SQL semantics: NULL keys never match
                    if matches!(left_row[left_key_idx], Value::Null) {
                        // For LEFT JOIN, this row will be added as unmatched later
                        continue;
                    }
                    let key = format!("{:?}", left_row[left_key_idx]);
                    if let Some(right_match_rows) = right_hash.get(&key) {
                        left_matched.insert(li);
                        for right_row in right_match_rows {
                            // Find the original right row index
                            if let Some(ri) = right_rows.iter().position(|r| r == right_row) {
                                right_matched.insert(ri);
                            }
                            let mut combined = left_row.clone();
                            combined.extend(right_row.clone());
                            matched.push(combined);
                        }
                    }
                }

                // For LEFT/RIGHT/FULL, add unmatched rows
                if matches!(join_type, JoinType::Left | JoinType::Full) {
                    for (li, left_row) in left_rows.iter().enumerate() {
                        if !left_matched.contains(&li) {
                            let mut combined = left_row.clone();
                            combined.extend(vec![Value::Null; right_col_count]);
                            matched.push(combined);
                        }
                    }
                }

                if matches!(join_type, JoinType::Right | JoinType::Full) {
                    for (ri, right_row) in right_rows.iter().enumerate() {
                        if !right_matched.contains(&ri) {
                            let mut combined = vec![Value::Null; left_col_count];
                            combined.extend(right_row.clone());
                            matched.push(combined);
                        }
                    }
                }

                matched
            }
            JoinType::Cross => {
                let mut results = Vec::new();
                for left_row in &left_rows {
                    for right_row in &right_rows {
                        let mut combined = left_row.clone();
                        combined.extend(right_row.clone());
                        results.push(combined);
                    }
                }
                results
            }
        };

        let combined_schema =
            build_combined_schema(&left_table_info, &right_table_name, &right_table_info)?;
        Ok((matched_results, combined_schema))
    }

    /// Find the column index for a join key in a table
    /// Handles both simple column names and qualified names (e.g., "t1.id")
    fn find_join_key_index(
        &self,
        expr: &Expression,
        table_info: &TableInfo,
        table_name: &str,
    ) -> SqlResult<usize> {
        match expr {
            Expression::Identifier(name) => {
                // Check if it's a qualified name like "t1.id"
                if let Some((qualifier, col_name)) = name.split_once('.') {
                    // If qualifier matches our table name, use the column name part
                    if qualifier == table_name {
                        table_info
                            .columns
                            .iter()
                            .position(|c| c.name.as_str() == col_name)
                            .ok_or_else(|| {
                                SqlError::ExecutionError(format!(
                                    "Column '{}.{}' not found in {}",
                                    qualifier, col_name, table_name
                                ))
                            })
                    } else {
                        // Qualifier doesn't match this table - column not in this table
                        Err(SqlError::ExecutionError(format!(
                            "Column '{}' not found in {}",
                            name, table_name
                        )))
                    }
                } else {
                    // Simple column name - find its index
                    table_info
                        .columns
                        .iter()
                        .position(|c| c.name.as_str() == name.as_str())
                        .ok_or_else(|| {
                            SqlError::ExecutionError(format!(
                                "Column '{}' not found in {}",
                                name, table_name
                            ))
                        })
                }
            }
            Expression::BinaryOp(left, _, right) => {
                // Try left side first
                let left_result = self.find_join_key_index(left, table_info, table_name);
                if left_result.is_ok() {
                    return left_result;
                }
                // Try right side
                self.find_join_key_index(right, table_info, table_name)
            }
            _ => Err(SqlError::ExecutionError(
                "Unsupported join condition expression".to_string(),
            )),
        }
    }

    fn execute_insert(&self, insert: &InsertStatement) -> SqlResult<ExecutorResult> {
        let table_name = insert.table.clone();

        // Get table info first (need it for triggers and FK validation)
        let table_info = {
            let storage = self.storage.read().unwrap();
            storage.get_table_info(&table_name)?.clone()
        };

        // Convert expressions to records
        let mut all_records: Vec<Vec<Value>> = Vec::new();
        for row_exprs in &insert.values {
            let mut record = Vec::with_capacity(row_exprs.len());
            for (_i, expr) in row_exprs.iter().enumerate() {
                let val = expression_to_value(expr);
                record.push(val);
            }
            all_records.push(record);
        }

        // For REPLACE INTO: if insert.values has a unique/key conflict, delete old row first
        if insert.is_replace {
            {
                let mut storage = self.storage.write().unwrap();
                for record in &all_records {
                    // Find existing rows with matching unique key (primary key or unique index)
                    let existing_rows = storage.scan(&table_name)?;
                    for existing_row in existing_rows {
                        if self.record_matches_unique_key(&existing_row, record, &table_info) {
                            // Delete the existing row
                            storage.delete(&table_name, &[])?;
                            break;
                        }
                    }
                }
            }
        }

        // Execute BEFORE INSERT triggers
        let trigger_executor = TriggerExecutor::new(self.storage.clone());
        let before_triggers = trigger_executor.get_triggers_for_operation(
            &table_name,
            ExecTriggerTiming::Before,
            ExecTriggerEvent::Insert,
        );

        let processed_records: Vec<Vec<Value>> = if !before_triggers.is_empty() {
            let mut processed = Vec::new();
            for record in &all_records {
                let modified = trigger_executor.execute_before_insert(&table_name, record)?;
                processed.push(modified);
            }
            processed
        } else {
            all_records.clone()
        };

        // Validate FK and CHECK constraints, then insert
        {
            let mut storage = self.storage.write().unwrap();
            let col_names: Vec<String> =
                table_info.columns.iter().map(|c| c.name.clone()).collect();
            for record in &processed_records {
                if !table_info.foreign_keys.is_empty() {
                    validate_foreign_keys(&*storage, &table_info, record, &insert.columns)?;
                }
                // Validate CHECK constraints
                if !table_info.check_constraints.is_empty() {
                    for constraint in &table_info.check_constraints {
                        let valid = sqlrustgo_storage::evaluate_check_constraint(
                            constraint, &col_names, record,
                        )?;
                        if !valid {
                            return Err(format!(
                                "CHECK constraint '{}' violated: {}",
                                constraint.name.as_deref().unwrap_or("unnamed"),
                                constraint.expression
                            )
                            .into());
                        }
                    }
                }
            }
            storage.insert(&table_name, processed_records)?;
        }

        // Execute AFTER INSERT triggers
        let after_triggers = trigger_executor.get_triggers_for_operation(
            &table_name,
            ExecTriggerTiming::After,
            ExecTriggerEvent::Insert,
        );

        if !after_triggers.is_empty() {
            for record in &all_records {
                trigger_executor.execute_after_insert(&table_name, record)?;
            }
        }

        self.query_cache.write().unwrap().invalidate_table(&table_name);

        Ok(ExecutorResult::new(vec![], insert.values.len()))
    }

    /// Check if a new record matches an existing row based on primary key or unique constraints
    fn record_matches_unique_key(
        &self,
        existing: &[Value],
        new: &[Value],
        table_info: &TableInfo,
    ) -> bool {
        // Find primary key column(s)
        for (col_idx, col) in table_info.columns.iter().enumerate() {
            if col.primary_key {
                // Compare by primary key column index
                if col_idx < existing.len() && col_idx < new.len() {
                    if existing[col_idx] != new[col_idx] {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }
        true
    }

    fn execute_update(&self, update: &UpdateStatement) -> SqlResult<ExecutorResult> {
        let table_name = update.table.clone();

        // If no WHERE clause, use the simple storage.update() path
        if update.where_clause.is_none() {
            let mut storage = self.storage.write().unwrap();
            let count = storage.update(&table_name, &[], &[])?;
            return Ok(ExecutorResult::new(vec![], count));
        }

        // Get table info and scan rows
        let table_info = {
            let storage = self.storage.read().unwrap();
            storage.get_table_info(&table_name)?.clone()
        };

        let all_rows = {
            let storage = self.storage.read().unwrap();
            storage.scan(&table_name)?
        };

        let where_clause = update.where_clause.as_ref().unwrap();

        // Filter rows that match the WHERE clause
        let rows_to_update: Vec<Vec<Value>> = all_rows
            .into_iter()
            .filter(|row| evaluate_where_clause(where_clause, row, &table_info))
            .collect();

        let count = rows_to_update.len();

        if count == 0 {
            return Ok(ExecutorResult::new(vec![], 0));
        }

        // Build column index map for SET clauses
        let set_col_indices: Vec<(usize, &Expression)> = update
            .set_clauses
            .iter()
            .filter_map(|(col_name, expr)| {
                find_column_index(col_name, &table_info).map(|idx| (idx, expr))
            })
            .collect();

        // Apply SET expressions to each matching row
        let updated_rows: Vec<Vec<Value>> = rows_to_update
            .iter()
            .map(|row| {
                let mut new_row = row.clone();
                for &(col_idx, ref set_expr) in &set_col_indices {
                    let new_val =
                        evaluate_expression(set_expr, &new_row, &table_info).unwrap_or(Value::Null);
                    if col_idx < new_row.len() {
                        new_row[col_idx] = new_val;
                    }
                }
                new_row
            })
            .collect();

        // Execute BEFORE UPDATE triggers
        let trigger_executor = TriggerExecutor::new(self.storage.clone());
        let before_triggers = trigger_executor.get_triggers_for_operation(
            &table_name,
            ExecTriggerTiming::Before,
            ExecTriggerEvent::Update,
        );

        let trigger_modified_rows: Vec<Vec<Value>> = if !before_triggers.is_empty() {
            let mut modified = Vec::new();
            for (i, updated_row) in updated_rows.iter().enumerate() {
                let old_row = &rows_to_update[i];
                let result =
                    trigger_executor.execute_before_update(&table_name, old_row, updated_row)?;
                modified.push(result);
            }
            modified
        } else {
            updated_rows.clone()
        };

        // Get rows to keep (non-matching rows)
        let rows_to_keep: Vec<Vec<Value>> = {
            let storage = self.storage.read().unwrap();
            let all_rows = storage.scan(&table_name)?;
            all_rows
                .into_iter()
                .filter(|row| !evaluate_where_clause(where_clause, row, &table_info))
                .collect()
        };

        // Delete all rows and re-insert updated + kept rows
        {
            let mut storage = self.storage.write().unwrap();
            let col_names: Vec<String> =
                table_info.columns.iter().map(|c| c.name.clone()).collect();

            // Validate CHECK constraints on updated rows before inserting
            if !table_info.check_constraints.is_empty() {
                for record in &trigger_modified_rows {
                    for constraint in &table_info.check_constraints {
                        let valid = sqlrustgo_storage::evaluate_check_constraint(
                            constraint, &col_names, record,
                        )?;
                        if !valid {
                            return Err(format!(
                                "CHECK constraint '{}' violated: {}",
                                constraint.name.as_deref().unwrap_or("unnamed"),
                                constraint.expression
                            )
                            .into());
                        }
                    }
                }
            }

            storage.delete(&table_name, &[])?;
            if !rows_to_keep.is_empty() {
                storage.insert(&table_name, rows_to_keep)?;
            }
            if !trigger_modified_rows.is_empty() {
                storage.insert(&table_name, trigger_modified_rows)?;
            }
        }

        // Execute AFTER UPDATE triggers
        let after_triggers = trigger_executor.get_triggers_for_operation(
            &table_name,
            ExecTriggerTiming::After,
            ExecTriggerEvent::Update,
        );

        if !after_triggers.is_empty() {
            for (i, updated_row) in updated_rows.iter().enumerate() {
                let old_row = &rows_to_update[i];
                trigger_executor.execute_after_update(&table_name, old_row, updated_row)?;
            }
        }

        self.query_cache.write().unwrap().invalidate_table(&table_name);
        Ok(ExecutorResult::new(vec![], count))
    }

    fn execute_delete(&self, delete: &DeleteStatement) -> SqlResult<ExecutorResult> {
        let table_name = delete.table.clone();

        // If no WHERE clause, delete all rows (current behavior is correct)
        if delete.where_clause.is_none() {
            let mut storage = self.storage.write().unwrap();
            let count = storage.delete(&table_name, &[])?;
            return Ok(ExecutorResult::new(vec![], count));
        }

        // Scan all rows from the table
        let all_rows = {
            let storage = self.storage.read().unwrap();
            storage.scan(&table_name)?
        };

        // Get table info to find column indices
        let table_info = {
            let storage = self.storage.read().unwrap();
            storage.get_table_info(&table_name)?.clone()
        };

        // Filter rows based on WHERE clause
        let where_clause = delete.where_clause.as_ref().unwrap();
        let rows_to_delete: Vec<Vec<Value>> = all_rows
            .into_iter()
            .filter(|row| evaluate_where_clause(where_clause, row, &table_info))
            .collect();

        let count = rows_to_delete.len();

        if count == 0 {
            return Ok(ExecutorResult::new(vec![], 0));
        }

        // Execute BEFORE DELETE triggers
        let trigger_executor = TriggerExecutor::new(self.storage.clone());
        let before_triggers = trigger_executor.get_triggers_for_operation(
            &table_name,
            ExecTriggerTiming::Before,
            ExecTriggerEvent::Delete,
        );

        if !before_triggers.is_empty() {
            for row in &rows_to_delete {
                trigger_executor.execute_before_delete(&table_name, row)?;
            }
        }

        // Delete all rows and re-insert non-matching ones
        let rows_to_keep: Vec<Vec<Value>> = {
            let storage = self.storage.read().unwrap();
            let all_rows = storage.scan(&table_name)?;
            all_rows
                .into_iter()
                .filter(|row| !evaluate_where_clause(where_clause, row, &table_info))
                .collect()
        };

        {
            let mut storage = self.storage.write().unwrap();
            storage.delete(&table_name, &[])?; // Delete all
            if !rows_to_keep.is_empty() {
                storage.insert(&table_name, rows_to_keep)?;
            }
        }

        // Execute AFTER DELETE triggers
        let after_triggers = trigger_executor.get_triggers_for_operation(
            &table_name,
            ExecTriggerTiming::After,
            ExecTriggerEvent::Delete,
        );

        if !after_triggers.is_empty() {
            for row in &rows_to_delete {
                trigger_executor.execute_after_delete(&table_name, row)?;
            }
        }

        Ok(ExecutorResult::new(vec![], count))
    }

    fn execute_create_table(&self, create: &CreateTableStatement) -> SqlResult<ExecutorResult> {
        let mut storage = self.storage.write().unwrap();
        let columns: Vec<ColumnDefinition> = create
            .columns
            .iter()
            .map(|c| ColumnDefinition {
                name: c.name.clone(),
                data_type: c.data_type.clone(),
                nullable: !c.primary_key,
                primary_key: c.primary_key,
            })
            .collect();
        let info = TableInfo {
            name: create.name.clone(),
            columns,
            foreign_keys: vec![],
            unique_constraints: vec![],
            check_constraints: vec![],
            partition_info: None,
        };
        storage.create_table(&info)?;
        Ok(ExecutorResult::empty())
    }

    fn execute_drop_table(&self, drop: &DropTableStatement) -> SqlResult<ExecutorResult> {
        let mut storage = self.storage.write().unwrap();
        storage.drop_table(&drop.name)?;
        Ok(ExecutorResult::empty())
    }

    fn execute_truncate(&self, truncate: &TruncateStatement) -> SqlResult<ExecutorResult> {
        let mut storage = self.storage.write().unwrap();
        // Check if table exists
        if !storage.has_table(&truncate.name) {
            return Err(SqlError::ExecutionError(format!(
                "Table not found: {}",
                truncate.name
            )));
        }
        // Delete all rows but keep the table structure
        // Using empty filter slice to delete all rows
        storage.delete(&truncate.name, &[])?;
        Ok(ExecutorResult::empty())
    }

    fn execute_create_index(&self, idx: &CreateIndexStatement) -> SqlResult<ExecutorResult> {
        let mut storage = self.storage.write().unwrap();
        let table_name = &idx.table;
        let col_name = idx
            .columns
            .first()
            .ok_or_else(|| SqlError::ExecutionError("No columns in index".to_string()))?;
        let table_info = storage.get_table_info(table_name)?;
        let col_idx = table_info
            .columns
            .iter()
            .position(|c| c.name == *col_name)
            .ok_or_else(|| SqlError::ExecutionError("Column not found".to_string()))?;
        storage.create_index(table_name, col_name, col_idx)?;
        Ok(ExecutorResult::empty())
    }

    fn execute_create_trigger(&self, stmt: &CreateTriggerStatement) -> SqlResult<ExecutorResult> {
        use sqlrustgo_storage::engine::{TriggerEvent, TriggerInfo, TriggerTiming};

        let mut storage = self.storage.write().unwrap();
        let timing = match stmt.timing.to_uppercase().as_str() {
            "BEFORE" => TriggerTiming::Before,
            "AFTER" => TriggerTiming::After,
            _ => {
                return Err(SqlError::ExecutionError(format!(
                    "Invalid trigger timing: {}",
                    stmt.timing
                )))
            }
        };
        // Handle first event (triggers support one event per trigger in storage)
        let event_str = stmt
            .events
            .first()
            .ok_or_else(|| SqlError::ExecutionError("No trigger event specified".to_string()))?;
        let event = match event_str.to_uppercase().as_str() {
            "INSERT" => TriggerEvent::Insert,
            "UPDATE" => TriggerEvent::Update,
            "DELETE" => TriggerEvent::Delete,
            _ => {
                return Err(SqlError::ExecutionError(format!(
                    "Invalid trigger event: {}",
                    event_str
                )))
            }
        };
        let trigger_info = TriggerInfo {
            name: stmt.name.clone(),
            table_name: stmt.table.clone(),
            timing,
            event,
            body: stmt.body.clone(),
        };
        storage.create_trigger(trigger_info)?;
        Ok(ExecutorResult::empty())
    }

    fn execute_call(&self, call: &CallStatement) -> SqlResult<ExecutorResult> {
        let catalog_guard = self.catalog.as_ref().ok_or_else(|| {
            SqlError::ExecutionError("CALL statement requires stored procedure catalog".to_string())
        })?;
        let catalog = catalog_guard.read().unwrap();

        let procedure = catalog
            .get_stored_procedure(&call.procedure_name)
            .ok_or_else(|| {
                SqlError::ExecutionError(format!(
                    "Stored procedure '{}' not found",
                    call.procedure_name
                ))
            })?;

        let executor = StoredProcExecutor::new(Arc::new(catalog.clone()), self.storage.clone());

        let args: Vec<Value> = call
            .args
            .iter()
            .map(|arg| expression_to_value_from_string(arg))
            .collect();

        let result = executor
            .execute_call(&call.procedure_name, args)
            .map_err(SqlError::ExecutionError)?;

        Ok(result)
    }

    fn execute_create_procedure(
        &self,
        stmt: &CreateProcedureStatement,
    ) -> SqlResult<ExecutorResult> {
        let catalog_guard = self.catalog.as_ref().ok_or_else(|| {
            SqlError::ExecutionError(
                "CREATE PROCEDURE requires stored procedure catalog".to_string(),
            )
        })?;
        let mut catalog = catalog_guard.write().unwrap();

        let params: Vec<sqlrustgo_catalog::stored_proc::StoredProcParam> = stmt
            .params
            .iter()
            .map(|p| {
                let mode = match p.mode {
                    ParserParamMode::In => ParamMode::In,
                    ParserParamMode::Out => ParamMode::Out,
                    ParserParamMode::InOut => ParamMode::InOut,
                };
                sqlrustgo_catalog::stored_proc::StoredProcParam {
                    name: p.name.clone(),
                    mode,
                    data_type: p.data_type.clone(),
                }
            })
            .collect();

        let body: Vec<StoredProcStatement> = stmt
            .body
            .iter()
            .map(|s| match s {
                ParserStatement::RawSql(sql) => StoredProcStatement::RawSql(sql.clone()),
            })
            .collect();

        let procedure = StoredProcedure::new(stmt.name.clone(), params, body);

        catalog.add_stored_procedure(procedure).map_err(|e| {
            SqlError::ExecutionError(format!("Failed to create procedure: {:?}", e))
        })?;

        Ok(ExecutorResult::empty())
    }

    fn execute_transaction(&mut self, stmt: &TransactionStatement) -> SqlResult<ExecutorResult> {
        match stmt {
            TransactionStatement::Begin {
                work: _,
                isolation_level,
            } => {
                let iso = isolation_level
                    .as_ref()
                    .map(|il| match il {
                        ParserIsolationLevel::ReadCommitted => TmIsolationLevel::SnapshotIsolation,
                        ParserIsolationLevel::ReadUncommitted => {
                            TmIsolationLevel::SnapshotIsolation
                        }
                        ParserIsolationLevel::SnapshotIsolation => {
                            TmIsolationLevel::SnapshotIsolation
                        }
                        ParserIsolationLevel::Serializable => TmIsolationLevel::Serializable,
                    })
                    .unwrap_or(self.default_isolation);
                self.begin_transaction(iso)
            }
            TransactionStatement::Commit { work: _ } => self.commit_transaction(),
            TransactionStatement::Rollback { work: _ } => self.rollback_transaction(),
            TransactionStatement::SetTransaction { isolation_level } => {
                self.default_isolation = match isolation_level {
                    ParserIsolationLevel::ReadCommitted => TmIsolationLevel::SnapshotIsolation,
                    ParserIsolationLevel::ReadUncommitted => TmIsolationLevel::SnapshotIsolation,
                    ParserIsolationLevel::SnapshotIsolation => TmIsolationLevel::SnapshotIsolation,
                    ParserIsolationLevel::Serializable => TmIsolationLevel::Serializable,
                };
                Ok(ExecutorResult::empty())
            }
            TransactionStatement::StartTransaction { isolation_level } => {
                let iso = isolation_level
                    .as_ref()
                    .map(|il| match il {
                        ParserIsolationLevel::ReadCommitted => TmIsolationLevel::SnapshotIsolation,
                        ParserIsolationLevel::ReadUncommitted => {
                            TmIsolationLevel::SnapshotIsolation
                        }
                        ParserIsolationLevel::SnapshotIsolation => {
                            TmIsolationLevel::SnapshotIsolation
                        }
                        ParserIsolationLevel::Serializable => TmIsolationLevel::Serializable,
                    })
                    .unwrap_or(self.default_isolation);
                self.begin_transaction(iso)
            }
        }
    }

    fn begin_transaction(&mut self, isolation: TmIsolationLevel) -> SqlResult<ExecutorResult> {
        if self.current_tx_id.is_some() {
            return Err(SqlError::ExecutionError(
                "Transaction already in progress".to_string(),
            ));
        }
        let tx_id = self
            .transaction_manager
            .begin_transaction(isolation)
            .map_err(|e| {
                SqlError::ExecutionError(format!("Failed to begin transaction: {:?}", e))
            })?;
        self.current_tx_id = Some(tx_id);
        Ok(ExecutorResult::new(
            vec![vec![Value::Integer(tx_id.as_u64() as i64)]],
            1,
        ))
    }

    fn commit_transaction(&mut self) -> SqlResult<ExecutorResult> {
        let tx_id = self
            .current_tx_id
            .ok_or_else(|| SqlError::ExecutionError("No transaction in progress".to_string()))?;
        self.transaction_manager.commit(tx_id).map_err(|e| {
            SqlError::ExecutionError(format!("Failed to commit transaction: {:?}", e))
        })?;
        self.current_tx_id = None;
        Ok(ExecutorResult::empty())
    }

    fn rollback_transaction(&mut self) -> SqlResult<ExecutorResult> {
        let tx_id = self
            .current_tx_id
            .ok_or_else(|| SqlError::ExecutionError("No transaction in progress".to_string()))?;
        self.transaction_manager.rollback(tx_id).map_err(|e| {
            SqlError::ExecutionError(format!("Failed to rollback transaction: {:?}", e))
        })?;
        self.current_tx_id = None;
        Ok(ExecutorResult::empty())
    }

    fn execute_grant(&mut self, grant: &GrantStatement) -> SqlResult<ExecutorResult> {
        let catalog_guard = self.catalog.as_ref().ok_or_else(|| {
            SqlError::ExecutionError("Catalog not available for GRANT".to_string())
        })?;
        let mut catalog = catalog_guard.write().unwrap();

        for privilege in &grant.privileges {
            let priv_str = match privilege {
                ParserPrivilege::Select => "SELECT",
                ParserPrivilege::Insert => "INSERT",
                ParserPrivilege::Update => "UPDATE",
                ParserPrivilege::Delete => "DELETE",
                ParserPrivilege::Read => "READ",
                ParserPrivilege::Write => "WRITE",
                ParserPrivilege::Execute => "EXECUTE",
                ParserPrivilege::Usage => "USAGE",
                ParserPrivilege::All => "ALL",
            };
            let priv_obj =
                sqlrustgo_catalog::auth::Privilege::from_str(priv_str).ok_or_else(|| {
                    SqlError::ExecutionError(format!("Unknown privilege: {}", priv_str))
                })?;

            let obj_type = match &grant.object_type {
                ParserObjectType::Table => sqlrustgo_catalog::auth::ObjectType::Table,
                ParserObjectType::Database => sqlrustgo_catalog::auth::ObjectType::Database,
                ParserObjectType::Column => sqlrustgo_catalog::auth::ObjectType::Column,
                ParserObjectType::Procedure => sqlrustgo_catalog::auth::ObjectType::Table,
                ParserObjectType::Function => sqlrustgo_catalog::auth::ObjectType::Table,
            };

            for recipient in &grant.recipients {
                let identity = sqlrustgo_catalog::auth::UserIdentity::new(recipient, "%");
                if grant.object_type == ParserObjectType::Column {
                    for column in &grant.columns {
                        catalog
                            .grant_column_privilege(&identity, priv_obj, &grant.object_name, column)
                            .map_err(|e| {
                                SqlError::ExecutionError(format!("GRANT failed: {}", e))
                            })?;
                    }
                } else {
                    catalog
                        .grant_privilege(
                            &identity,
                            priv_obj,
                            obj_type,
                            &grant.object_name,
                            grant.with_grant_option,
                        )
                        .map_err(|e| SqlError::ExecutionError(format!("GRANT failed: {}", e)))?;
                }
            }
        }

        Ok(ExecutorResult::new(
            vec![vec![Value::Integer(grant.recipients.len() as i64)]],
            1,
        ))
    }

    fn execute_revoke(&mut self, revoke: &RevokeStatement) -> SqlResult<ExecutorResult> {
        let catalog_guard = self.catalog.as_ref().ok_or_else(|| {
            SqlError::ExecutionError("Catalog not available for REVOKE".to_string())
        })?;
        let mut catalog = catalog_guard.write().unwrap();

        for privilege in &revoke.privileges {
            let priv_str = match privilege {
                ParserPrivilege::Select => "SELECT",
                ParserPrivilege::Insert => "INSERT",
                ParserPrivilege::Update => "UPDATE",
                ParserPrivilege::Delete => "DELETE",
                ParserPrivilege::Read => "READ",
                ParserPrivilege::Write => "WRITE",
                ParserPrivilege::Execute => "EXECUTE",
                ParserPrivilege::Usage => "USAGE",
                ParserPrivilege::All => "ALL",
            };
            let priv_obj =
                sqlrustgo_catalog::auth::Privilege::from_str(priv_str).ok_or_else(|| {
                    SqlError::ExecutionError(format!("Unknown privilege: {}", priv_str))
                })?;

            let obj_type = match &revoke.object_type {
                ParserObjectType::Table => sqlrustgo_catalog::auth::ObjectType::Table,
                ParserObjectType::Database => sqlrustgo_catalog::auth::ObjectType::Database,
                ParserObjectType::Column => sqlrustgo_catalog::auth::ObjectType::Column,
                ParserObjectType::Procedure => sqlrustgo_catalog::auth::ObjectType::Table,
                ParserObjectType::Function => sqlrustgo_catalog::auth::ObjectType::Table,
            };

            for user in &revoke.from_users {
                let identity = sqlrustgo_catalog::auth::UserIdentity::new(user, "%");
                catalog
                    .revoke_privilege(&identity, priv_obj, obj_type, &revoke.object_name)
                    .map_err(|e| SqlError::ExecutionError(format!("REVOKE failed: {}", e)))?;
            }
        }

        Ok(ExecutorResult::new(
            vec![vec![Value::Integer(revoke.from_users.len() as i64)]],
            1,
        ))
    }

    fn execute_create_role(&mut self, stmt: &CreateRoleStatement) -> SqlResult<ExecutorResult> {
        let catalog = self
            .catalog
            .as_ref()
            .ok_or_else(|| SqlError::ExecutionError("No catalog available".to_string()))?;
        let mut catalog_guard = catalog.write().unwrap();

        let parent_role_id = if let Some(ref parent_name) = stmt.parent_role {
            let parent_role = catalog_guard
                .auth_manager()
                .find_role_by_name(parent_name)
                .ok_or_else(|| {
                    SqlError::ExecutionError(format!("Parent role '{}' not found", parent_name))
                })?;
            Some(parent_role.id)
        } else {
            None
        };

        catalog_guard
            .create_role(&stmt.name, parent_role_id)
            .map_err(|e| SqlError::ExecutionError(format!("CREATE ROLE failed: {}", e)))?;

        Ok(ExecutorResult::new(
            vec![vec![Value::Text(format!("Role {} created", stmt.name))]],
            1,
        ))
    }

    fn execute_drop_role(&mut self, stmt: &DropRoleStatement) -> SqlResult<ExecutorResult> {
        let catalog = self
            .catalog
            .as_ref()
            .ok_or_else(|| SqlError::ExecutionError("No catalog available".to_string()))?;
        let mut catalog_guard = catalog.write().unwrap();

        let role_id = {
            let role = catalog_guard
                .auth_manager()
                .find_role_by_name(&stmt.name)
                .ok_or_else(|| {
                    SqlError::ExecutionError(format!("Role '{}' not found", stmt.name))
                })?;
            role.id
        };

        catalog_guard
            .drop_role(role_id)
            .map_err(|e| SqlError::ExecutionError(format!("DROP ROLE failed: {}", e)))?;

        Ok(ExecutorResult::new(
            vec![vec![Value::Text(format!("Role {} dropped", stmt.name))]],
            1,
        ))
    }

    fn execute_grant_role(&mut self, stmt: &GrantRoleStatement) -> SqlResult<ExecutorResult> {
        let catalog = self
            .catalog
            .as_ref()
            .ok_or_else(|| SqlError::ExecutionError("No catalog available".to_string()))?;
        let mut catalog_guard = catalog.write().unwrap();

        let role_id = {
            let role = catalog_guard
                .auth_manager()
                .find_role_by_name(&stmt.role_name)
                .ok_or_else(|| {
                    SqlError::ExecutionError(format!("Role '{}' not found", stmt.role_name))
                })?;
            role.id
        };

        let user_identity = UserIdentity::new(&stmt.user_name, stmt.host.as_deref().unwrap_or("%"));

        let user_id = {
            catalog_guard
                .auth_manager()
                .get_user_id_by_identity(&user_identity)
                .ok_or_else(|| {
                    SqlError::ExecutionError(format!("User '{}' not found", stmt.user_name))
                })?
        };

        catalog_guard
            .grant_role_to_user(user_id, role_id, 0)
            .map_err(|e| SqlError::ExecutionError(format!("GRANT ROLE failed: {}", e)))?;

        Ok(ExecutorResult::new(
            vec![vec![Value::Text(format!(
                "Grant {} to {}",
                stmt.role_name, stmt.user_name
            ))]],
            1,
        ))
    }

    fn execute_revoke_role(&mut self, stmt: &RevokeRoleStatement) -> SqlResult<ExecutorResult> {
        let catalog = self
            .catalog
            .as_ref()
            .ok_or_else(|| SqlError::ExecutionError("No catalog available".to_string()))?;
        let mut catalog_guard = catalog.write().unwrap();

        let role_id = {
            let role = catalog_guard
                .auth_manager()
                .find_role_by_name(&stmt.role_name)
                .ok_or_else(|| {
                    SqlError::ExecutionError(format!("Role '{}' not found", stmt.role_name))
                })?;
            role.id
        };

        let user_identity = UserIdentity::new(&stmt.user_name, stmt.host.as_deref().unwrap_or("%"));

        let user_id = {
            catalog_guard
                .auth_manager()
                .get_user_id_by_identity(&user_identity)
                .ok_or_else(|| {
                    SqlError::ExecutionError(format!("User '{}' not found", stmt.user_name))
                })?
        };

        catalog_guard
            .revoke_role_from_user(user_id, role_id)
            .map_err(|e| SqlError::ExecutionError(format!("REVOKE ROLE failed: {}", e)))?;

        Ok(ExecutorResult::new(
            vec![vec![Value::Text(format!(
                "Revoke {} from {}",
                stmt.role_name, stmt.user_name
            ))]],
            1,
        ))
    }

    fn execute_set_role(&mut self, stmt: &SetRoleStatement) -> SqlResult<ExecutorResult> {
        let catalog = self
            .catalog
            .as_ref()
            .ok_or_else(|| SqlError::ExecutionError("No catalog available".to_string()))?;

        let role_name = {
            let catalog_guard = catalog.read().unwrap();
            let role = catalog_guard
                .auth_manager()
                .find_role_by_name(&stmt.role_name)
                .ok_or_else(|| {
                    SqlError::ExecutionError(format!("Role '{}' not found", stmt.role_name))
                })?;
            role.name.clone()
        };

        self.current_role = Some(stmt.role_name.clone());

        Ok(ExecutorResult::new(
            vec![vec![Value::Text(format!("SET ROLE to {}", role_name))]],
            1,
        ))
    }

    fn execute_show_roles(&self) -> SqlResult<ExecutorResult> {
        let catalog = self
            .catalog
            .as_ref()
            .ok_or_else(|| SqlError::ExecutionError("No catalog available".to_string()))?;
        let catalog_guard = catalog.read().unwrap();

        let roles = catalog_guard.auth_manager().list_roles();
        let rows: Vec<Vec<Value>> = roles
            .iter()
            .map(|r| {
                vec![
                    Value::Integer(r.id as i64),
                    Value::Text(r.name.clone()),
                    r.parent_role_id
                        .map(|id| Value::Integer(id as i64))
                        .unwrap_or(Value::Null),
                ]
            })
            .collect();

        Ok(ExecutorResult::new(rows, 3))
    }

    fn execute_show_grants_for(&self, user_spec: &str) -> SqlResult<ExecutorResult> {
        let catalog = self
            .catalog
            .as_ref()
            .ok_or_else(|| SqlError::ExecutionError("No catalog available".to_string()))?;
        let catalog_guard = catalog.read().unwrap();

        let parts: Vec<&str> = user_spec.split('@').collect();
        let username = parts[0];
        let host = parts.get(1).unwrap_or(&"%");

        let identity = UserIdentity::new(username, host);
        let grants = catalog_guard
            .auth_manager()
            .get_all_grants_for_user(&identity);

        let rows: Vec<Vec<Value>> = grants
            .iter()
            .map(|g| {
                vec![
                    Value::Text(format!("{}@{}", g.user.username, g.user.host)),
                    Value::Text(g.privilege.to_string()),
                    Value::Text(format!("{:?}", g.object.object_type)),
                    Value::Text(g.object.object_name.clone()),
                ]
            })
            .collect();

        Ok(ExecutorResult::new(rows, 4))
    }
}

impl ExecutionEngine<MemoryStorage> {
    /// Create a new execution engine backed by MemoryStorage with CBO enabled
    pub fn with_memory() -> Self {
        Self {
            storage: Arc::new(RwLock::new(MemoryStorage::new())),
            catalog: None,
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
            cbo_enabled: true,
            transaction_manager: TransactionManager::new(),
            current_tx_id: None,
            default_isolation: TmIsolationLevel::default(),
            current_role: None,
            cache_config: QueryCacheConfig::default(),
            query_cache: Arc::new(RwLock::new(QueryCache::new(QueryCacheConfig::default()))),
        }
    }

    /// Create a new execution engine backed by MemoryStorage with custom CBO setting
    pub fn with_memory_and_cbo(cbo_enabled: bool) -> Self {
        Self {
            storage: Arc::new(RwLock::new(MemoryStorage::new())),
            catalog: None,
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
            cbo_enabled,
            transaction_manager: TransactionManager::new(),
            current_tx_id: None,
            default_isolation: TmIsolationLevel::default(),
            current_role: None,
            cache_config: QueryCacheConfig::default(),
            query_cache: Arc::new(RwLock::new(QueryCache::new(QueryCacheConfig::default()))),
        }
    }

    /// Create a new execution engine with catalog
    pub fn with_memory_and_catalog(catalog: Arc<RwLock<Catalog>>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(MemoryStorage::new())),
            catalog: Some(catalog),
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
            cbo_enabled: true,
            transaction_manager: TransactionManager::new(),
            current_tx_id: None,
            default_isolation: TmIsolationLevel::default(),
            current_role: None,
            cache_config: QueryCacheConfig::default(),
            query_cache: Arc::new(RwLock::new(QueryCache::new(QueryCacheConfig::default()))),
        }
    }
}

fn expression_to_string(expr: &sqlrustgo_parser::Expression) -> String {
    match expr {
        sqlrustgo_parser::Expression::Literal(s) => s.clone(),
        sqlrustgo_parser::Expression::Identifier(name) => name.clone(),
        sqlrustgo_parser::Expression::BinaryOp(left, op, right) => {
            format!(
                "({} {} {})",
                expression_to_string(left),
                op,
                expression_to_string(right)
            )
        }
        sqlrustgo_parser::Expression::IsNull(inner) => {
            format!("{} IS NULL", expression_to_string(inner))
        }
        sqlrustgo_parser::Expression::IsNotNull(inner) => {
            format!("{} IS NOT NULL", expression_to_string(inner))
        }
        sqlrustgo_parser::Expression::Aggregate(agg) => match agg.func {
            sqlrustgo_parser::AggregateFunction::Count => {
                if agg.args.is_empty() {
                    "COUNT(*)".to_string()
                } else {
                    format!(
                        "COUNT({})",
                        agg.args
                            .iter()
                            .map(expression_to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            sqlrustgo_parser::AggregateFunction::Sum => {
                format!(
                    "SUM({})",
                    agg.args
                        .iter()
                        .map(expression_to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            sqlrustgo_parser::AggregateFunction::Avg => {
                format!(
                    "AVG({})",
                    agg.args
                        .iter()
                        .map(expression_to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            sqlrustgo_parser::AggregateFunction::Min => {
                format!(
                    "MIN({})",
                    agg.args
                        .iter()
                        .map(expression_to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            sqlrustgo_parser::AggregateFunction::Max => {
                format!(
                    "MAX({})",
                    agg.args
                        .iter()
                        .map(expression_to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        },
        _ => "?".to_string(),
    }
}

/// Convert a parser Expression to a Value (simple literal evaluation)
fn expression_to_value(expr: &sqlrustgo_parser::Expression) -> Value {
    match expr {
        sqlrustgo_parser::Expression::Literal(s) => {
            let s = s.trim();
            if s.eq_ignore_ascii_case("NULL") {
                Value::Null
            } else if let Ok(n) = s.parse::<i64>() {
                Value::Integer(n)
            } else if let Ok(f) = s.parse::<f64>() {
                Value::Float(f)
            } else if s.starts_with('\'') && s.ends_with('\'') {
                Value::Text(s[1..s.len() - 1].to_string())
            } else {
                Value::Text(s.to_string())
            }
        }
        sqlrustgo_parser::Expression::Identifier(name) => Value::Text(name.clone()),
        _ => Value::Null,
    }
}

/// Convert a string argument to a Value (for CALL arguments)
fn expression_to_value_from_string(s: &str) -> Value {
    let s = s.trim();
    if s.eq_ignore_ascii_case("NULL") {
        Value::Null
    } else if let Ok(n) = s.parse::<i64>() {
        Value::Integer(n)
    } else if let Ok(f) = s.parse::<f64>() {
        Value::Float(f)
    } else if s.starts_with('\'') && s.ends_with('\'') {
        Value::Text(s[1..s.len() - 1].to_string())
    } else {
        Value::Text(s.to_string())
    }
}

/// Validate foreign key constraints for a row before insert
fn validate_foreign_keys(
    storage: &dyn StorageEngine,
    table_info: &sqlrustgo_storage::TableInfo,
    row: &[Value],
    insert_columns: &[String],
) -> SqlResult<()> {
    for fk in &table_info.foreign_keys {
        // Collect FK column values from the row
        let fk_values: Vec<Value> = fk
            .columns
            .iter()
            .filter_map(|col_name| {
                let col_idx = if insert_columns.is_empty() {
                    table_info
                        .columns
                        .iter()
                        .position(|c| c.name.eq_ignore_ascii_case(col_name))
                } else {
                    insert_columns
                        .iter()
                        .position(|c| c.eq_ignore_ascii_case(col_name))
                };
                col_idx.and_then(|idx| row.get(idx).cloned())
            })
            .collect();

        // Skip if any FK value is NULL (NULL FKs are allowed)
        if fk_values.iter().any(|v| matches!(v, Value::Null)) {
            continue;
        }

        // Scan parent table to verify referenced row exists
        let parent_rows = storage.scan(&fk.referenced_table)?;

        // Find referenced column indices in parent table
        let ref_col_indices: Vec<usize> = fk
            .referenced_columns
            .iter()
            .filter_map(|col_name| {
                storage
                    .get_table_info(&fk.referenced_table)
                    .ok()?
                    .columns
                    .iter()
                    .position(|c| c.name.eq_ignore_ascii_case(col_name))
            })
            .collect();

        let parent_has_match = parent_rows.iter().any(|parent_row| {
            ref_col_indices
                .iter()
                .enumerate()
                .all(|(i, &col_idx)| parent_row.get(col_idx) == fk_values.get(i))
        });

        if !parent_has_match {
            return Err(SqlError::ExecutionError(format!(
                "Foreign key constraint failed: {} ({}) references {} ({}) which does not exist",
                table_info.name,
                fk.columns.join(", "),
                fk.referenced_table,
                fk.referenced_columns.join(", ")
            )));
        }
    }
    Ok(())
}

/// Evaluate a WHERE clause expression against a row
/// Returns true if the row matches the WHERE condition
/// Evaluate a predicate expression to a boolean result
/// Phase 1: UNKNOWN is folded to FALSE for WHERE filtering
/// All NULL handling is centralized here - no NULL logic in individual operators
fn eval_predicate(expr: &Expression, row: &[Value], table_info: &TableInfo) -> bool {
    match expr {
        // AND short-circuits on false
        Expression::BinaryOp(left, op, right) if op.to_uppercase() == "AND" => {
            eval_predicate(left, row, table_info) && eval_predicate(right, row, table_info)
        }
        // OR short-circuits on true
        Expression::BinaryOp(left, op, right) if op.to_uppercase() == "OR" => {
            eval_predicate(left, row, table_info) || eval_predicate(right, row, table_info)
        }
        // IS NULL - always goes through evaluate_expression for value extraction
        Expression::IsNull(inner) => match evaluate_expression(inner, row, table_info) {
            Ok(val) => matches!(val, Value::Null),
            Err(_) => false,
        },
        // IS NOT NULL
        Expression::IsNotNull(inner) => match evaluate_expression(inner, row, table_info) {
            Ok(val) => !matches!(val, Value::Null),
            Err(_) => false,
        },
        // Legacy IS NULL (col IS NULL) - now uses new Expression::IsNull
        Expression::BinaryOp(left, op, right)
            if op.to_uppercase() == "IS"
                && matches!(right.as_ref(), Expression::Literal(s) if s.to_uppercase() == "NULL") =>
        {
            eval_predicate(&Expression::IsNull(left.clone()), row, table_info)
        }
        // Legacy IS NOT NULL
        Expression::BinaryOp(left, op, right)
            if op.to_uppercase() == "IS NOT"
                && matches!(right.as_ref(), Expression::Literal(s) if s.to_uppercase() == "NULL") =>
        {
            eval_predicate(&Expression::IsNotNull(left.clone()), row, table_info)
        }
        // All comparison operators go through sql_compare
        Expression::BinaryOp(left, op, right) => {
            let left_val = evaluate_expression(left, row, table_info).unwrap_or(Value::Null);
            let right_val = evaluate_expression(right, row, table_info).unwrap_or(Value::Null);
            sql_compare(op, &left_val, &right_val)
        }
        // For other expressions, evaluate and check if truthy
        _ => match evaluate_expression(expr, row, table_info) {
            Ok(val) => {
                matches!(val, Value::Boolean(true))
            }
            Err(_) => false,
        },
    }
}

/// Legacy alias for compatibility
#[allow(dead_code)]
fn evaluate_where_clause(expr: &Expression, row: &[Value], table_info: &TableInfo) -> bool {
    eval_predicate(expr, row, table_info)
}

/// SQL comparison operator
/// Returns false if either operand is NULL (UNKNOWN semantics)
/// This is Phase 1: UNKNOWN is folded to FALSE for WHERE filtering
fn sql_compare(op: &str, left: &Value, right: &Value) -> bool {
    if matches!(left, Value::Null) || matches!(right, Value::Null) {
        return false;
    }

    match op.to_uppercase().as_str() {
        "=" | "==" => left == right,
        "!=" | "<>" => left != right,
        ">" => compare_values(left, right) > 0,
        ">=" => compare_values(left, right) >= 0,
        "<" => compare_values(left, right) < 0,
        "<=" => compare_values(left, right) <= 0,
        _ => false,
    }
}

/// Evaluate an expression and return a Value
fn evaluate_expression(
    expr: &Expression,
    row: &[Value],
    table_info: &TableInfo,
) -> Result<Value, String> {
    match expr {
        Expression::Literal(_) => Ok(expression_to_value(expr)),
        Expression::Identifier(name) => {
            if let Some(col_idx) = find_column_index(name, table_info) {
                Ok(row.get(col_idx).cloned().unwrap_or(Value::Null))
            } else {
                Ok(expression_to_value(expr))
            }
        }
        Expression::BinaryOp(left, op, right) => {
            let left_val = evaluate_expression(left, row, table_info).unwrap_or(Value::Null);
            let right_val = evaluate_expression(right, row, table_info).unwrap_or(Value::Null);
            Ok(evaluate_binary_op(&left_val, &right_val, op))
        }
        Expression::IsNull(inner) => {
            let val = evaluate_expression(inner, row, table_info)?;
            Ok(Value::Boolean(matches!(val, Value::Null)))
        }
        Expression::Aggregate(agg) => {
            let agg_name = expression_to_string(&Expression::Aggregate(agg.clone()));
            if let Some(col_idx) = find_column_index(&agg_name, table_info) {
                Ok(row.get(col_idx).cloned().unwrap_or(Value::Null))
            } else {
                Err(format!("Aggregate not found in schema: {}", agg_name))
            }
        }
        _ => Ok(Value::Null),
    }
}

/// Evaluate a binary operation and return a boolean Value
fn evaluate_binary_op(left: &Value, right: &Value, op: &str) -> Value {
    match op.to_uppercase().as_str() {
        "=" | "==" | "IS" => Value::Boolean(left == right),
        "!=" | "<>" => Value::Boolean(left != right),
        ">" => Value::Boolean(compare_values(left, right) > 0),
        ">=" => Value::Boolean(compare_values(left, right) >= 0),
        "<" => Value::Boolean(compare_values(left, right) < 0),
        "<=" => Value::Boolean(compare_values(left, right) <= 0),
        "AND" | "&&" => {
            if let (Value::Boolean(l), Value::Boolean(r)) = (left, right) {
                Value::Boolean(*l && *r)
            } else {
                Value::Boolean(false)
            }
        }
        "OR" | "||" => {
            if let (Value::Boolean(l), Value::Boolean(r)) = (left, right) {
                Value::Boolean(*l || *r)
            } else {
                Value::Boolean(false)
            }
        }
        _ => Value::Null,
    }
}

/// Compare two values and return -1, 0, or 1
fn compare_values(left: &Value, right: &Value) -> i32 {
    match (left, right) {
        (Value::Integer(l), Value::Integer(r)) => l.cmp(r) as i32,
        (Value::Float(l), Value::Float(r)) => {
            if l < r {
                -1
            } else if l > r {
                1
            } else {
                0
            }
        }
        (Value::Text(l), Value::Text(r)) => l.cmp(r) as i32,
        (Value::Null, Value::Null) => 0,
        (Value::Null, _) => -1,
        (_, Value::Null) => 1,
        _ => 0,
    }
}

/// Evaluate expression to string (for GROUP BY key)
fn evaluate_expr_to_string(expr: &Expression, row: &[Value], table_info: &TableInfo) -> String {
    let val = evaluate_expression(expr, row, table_info).unwrap_or(Value::Null);
    match val {
        Value::Null => "NULL".to_string(),
        Value::Integer(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Text(s) => s,
        Value::Boolean(b) => b.to_string(),
        _ => "?".to_string(),
    }
}

/// Find the index of a column in the table info
/// For JOIN queries with combined tables, handles qualified names like "t2.id"
/// by routing to the correct portion of the combined schema.
/// Combined table naming: left_table.col, right_table.col
fn find_column_index(col_name: &str, table_info: &TableInfo) -> Option<usize> {
    if let Some((qualifier, col)) = col_name.split_once('.') {
        for (i, c) in table_info.columns.iter().enumerate() {
            if c.name.eq_ignore_ascii_case(col_name) {
                return Some(i);
            }
        }
        table_info
            .columns
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(col))
    } else {
        table_info
            .columns
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(col_name))
    }
}

fn build_combined_schema(
    left_info: &TableInfo,
    right_table_name: &str,
    right_info: &TableInfo,
) -> SqlResult<TableInfo> {
    let mut columns = Vec::new();

    for c in &left_info.columns {
        columns.push(ColumnDefinition {
            name: format!("{}.{}", left_info.name, c.name),
            data_type: c.data_type.clone(),
            nullable: c.nullable,
            primary_key: c.primary_key,
        });
    }

    for c in &right_info.columns {
        columns.push(ColumnDefinition {
            name: format!("{}.{}", right_table_name, c.name),
            data_type: c.data_type.clone(),
            nullable: c.nullable,
            primary_key: c.primary_key,
        });
    }

    Ok(TableInfo {
        name: format!("{}_join_{}", left_info.name, right_table_name),
        columns,
        foreign_keys: vec![],
        unique_constraints: vec![],
        check_constraints: vec![],
        partition_info: None,
    })
}

fn build_aggregate_schema(
    group_by: &[Expression],
    aggregates: &[AggregateCall],
) -> SqlResult<TableInfo> {
    let mut columns = Vec::new();

    for expr in group_by {
        columns.push(ColumnDefinition {
            name: expression_to_string(expr),
            data_type: "INTEGER".to_string(),
            nullable: false,
            primary_key: false,
        });
    }

    for agg in aggregates {
        let name = match agg.func {
            AggregateFunction::Count => {
                if agg.args.is_empty() {
                    "COUNT(*)".to_string()
                } else {
                    format!(
                        "COUNT({})",
                        agg.args
                            .iter()
                            .map(expression_to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            AggregateFunction::Sum => {
                format!(
                    "SUM({})",
                    agg.args
                        .iter()
                        .map(expression_to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            AggregateFunction::Avg => {
                format!(
                    "AVG({})",
                    agg.args
                        .iter()
                        .map(expression_to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            AggregateFunction::Min => {
                format!(
                    "MIN({})",
                    agg.args
                        .iter()
                        .map(expression_to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            AggregateFunction::Max => {
                format!(
                    "MAX({})",
                    agg.args
                        .iter()
                        .map(expression_to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        };
        columns.push(ColumnDefinition {
            name,
            data_type: "INTEGER".to_string(),
            nullable: false,
            primary_key: false,
        });
    }

    Ok(TableInfo {
        name: "aggregate".to_string(),
        columns,
        foreign_keys: vec![],
        unique_constraints: vec![],
        check_constraints: vec![],
        partition_info: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_table_stats() {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let mut engine = ExecutionEngine::new(storage);

        engine
            .execute("CREATE TABLE users (id INTEGER, name TEXT, age INTEGER)")
            .unwrap();
        engine
            .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
            .unwrap();
        engine
            .execute("INSERT INTO users VALUES (2, 'Bob', 25)")
            .unwrap();
        engine
            .execute("INSERT INTO users VALUES (3, 'Charlie', 30)")
            .unwrap();

        let result = engine.execute("ANALYZE users").unwrap();
        assert_eq!(result.affected_rows, 1);
        assert_eq!(result.rows[0][0], Value::Integer(3));

        let stats = engine.get_table_stats();
        let stats_guard = stats.read().unwrap();
        let table_stats = stats_guard.table_stats.get("users").unwrap();
        assert_eq!(table_stats.row_count, 3);
    }

    #[test]
    fn test_execution_stats_default() {
        let stats = ExecutionStats::default();
        assert!(stats.table_stats.is_empty());
    }

    #[test]
    fn test_table_statistics() {
        let mut stats = HashMap::new();
        stats.insert(
            "users".to_string(),
            TableStatistics {
                row_count: 100,
                column_stats: HashMap::new(),
            },
        );

        let exec_stats = ExecutionStats { table_stats: stats };
        assert_eq!(exec_stats.table_stats.get("users").unwrap().row_count, 100);
    }

    #[test]
    fn test_estimate_row_count() {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let mut engine = ExecutionEngine::new(storage);

        engine
            .execute("CREATE TABLE users (id INTEGER, name TEXT)")
            .unwrap();
        engine
            .execute("INSERT INTO users VALUES (1, 'Alice')")
            .unwrap();
        engine
            .execute("INSERT INTO users VALUES (2, 'Bob')")
            .unwrap();
        engine
            .execute("INSERT INTO users VALUES (3, 'Charlie')")
            .unwrap();

        // Before ANALYZE, should return default estimate
        assert_eq!(engine.estimate_row_count("users"), 1000);

        // After ANALYZE, should return actual count
        engine.execute("ANALYZE users").unwrap();
        assert_eq!(engine.estimate_row_count("users"), 3);
    }

    #[test]
    fn test_estimate_selectivity() {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let mut engine = ExecutionEngine::new(storage);

        engine
            .execute("CREATE TABLE users (id INTEGER, name TEXT)")
            .unwrap();
        for i in 0..100 {
            engine
                .execute(&format!("INSERT INTO users VALUES ({}, 'User{}')", i, i))
                .unwrap();
        }

        // Before ANALYZE, should return default selectivity
        let selectivity = engine.estimate_selectivity("users", "id");
        assert_eq!(selectivity, 0.1); // Default 10%

        // After ANALYZE with distinct_count, should return better estimate
        engine.execute("ANALYZE users").unwrap();
        let selectivity = engine.estimate_selectivity("users", "id");
        assert_eq!(selectivity, 0.01); // 1/100 distinct values
    }

    #[test]
    fn test_optimize_join_order() {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let mut engine = ExecutionEngine::new(storage);

        // Create tables of different sizes
        engine.execute("CREATE TABLE large (id INTEGER)").unwrap();
        engine.execute("CREATE TABLE medium (id INTEGER)").unwrap();
        engine.execute("CREATE TABLE small (id INTEGER)").unwrap();

        for i in 0..1000 {
            engine
                .execute(&format!("INSERT INTO large VALUES ({})", i))
                .unwrap();
        }
        for i in 0..100 {
            engine
                .execute(&format!("INSERT INTO medium VALUES ({})", i))
                .unwrap();
        }
        for i in 0..10 {
            engine
                .execute(&format!("INSERT INTO small VALUES ({})", i))
                .unwrap();
        }

        // Analyze to get accurate row counts
        engine.execute("ANALYZE large").unwrap();
        engine.execute("ANALYZE medium").unwrap();
        engine.execute("ANALYZE small").unwrap();

        let tables = vec!["large", "medium", "small"];
        let optimal = engine.optimize_join_order(&tables);

        // Smallest table should be first after ANALYZE
        assert_eq!(optimal[0], "small");
        // Should have all tables
        assert_eq!(optimal.len(), 3);
    }

    #[test]
    fn test_cbo_disable() {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let mut engine = ExecutionEngine::with_cbo(storage, false);
        assert!(!engine.is_cbo_enabled());
        engine.set_cbo_enabled(true);
        assert!(engine.is_cbo_enabled());
    }

    #[test]
    fn test_memory_engine_with_cbo() {
        let engine = ExecutionEngine::with_memory();
        assert!(engine.is_cbo_enabled());

        let engine_disabled = ExecutionEngine::with_memory_and_cbo(false);
        assert!(!engine_disabled.is_cbo_enabled());
    }

    #[test]
    fn test_estimate_index_benefit() {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let mut engine = ExecutionEngine::new(storage);

        engine
            .execute("CREATE TABLE users (id INTEGER, name TEXT)")
            .unwrap();
        for i in 0..1000 {
            engine
                .execute(&format!("INSERT INTO users VALUES ({}, 'User{}')", i, i))
                .unwrap();
        }

        // High selectivity (1/1000) - index should be very beneficial
        let high_sel = engine.estimate_selectivity("users", "id");
        let benefit = engine.estimate_index_benefit("users", high_sel);
        assert!(benefit > 0.0); // Index should be beneficial

        // With ANALYZE, we get actual stats
        engine.execute("ANALYZE users").unwrap();
        let benefit_after_analyze = engine.estimate_index_benefit("users", high_sel);
        assert!(benefit_after_analyze > 0.0);
    }

    #[test]
    fn test_should_use_index() {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let mut engine = ExecutionEngine::new(storage);

        engine
            .execute("CREATE TABLE users (id INTEGER, name TEXT)")
            .unwrap();
        for i in 0..10000 {
            engine
                .execute(&format!("INSERT INTO users VALUES ({}, 'User{}')", i, i))
                .unwrap();
        }

        // With low selectivity (high cardinality), index is beneficial
        let use_index = engine.should_use_index("users", "id");
        assert!(use_index);

        // After ANALYZE, should still recommend index for high cardinality
        engine.execute("ANALYZE users").unwrap();
        let use_index_after = engine.should_use_index("users", "id");
        assert!(use_index_after);
    }

    #[test]
    fn test_estimate_join_cost() {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let mut engine = ExecutionEngine::new(storage);

        engine
            .execute("CREATE TABLE orders (id INTEGER, user_id INTEGER)")
            .unwrap();
        engine
            .execute("CREATE TABLE users (id INTEGER, name TEXT)")
            .unwrap();

        for i in 0..100 {
            engine
                .execute(&format!("INSERT INTO orders VALUES ({}, {})", i, i % 10))
                .unwrap();
        }
        for i in 0..10 {
            engine
                .execute(&format!("INSERT INTO users VALUES ({}, 'User{}')", i, i))
                .unwrap();
        }

        let hash_cost = engine.estimate_join_cost("orders", "users", "hash");
        let nl_cost = engine.estimate_join_cost("orders", "users", "nested_loop");
        let merge_cost = engine.estimate_join_cost("orders", "users", "merge");

        // Hash join should be reasonable
        assert!(hash_cost > 0.0);
        assert!(nl_cost > 0.0);
        assert!(merge_cost > 0.0);
    }

    #[test]
    fn test_optimize_join_order_after_analyze() {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let mut engine = ExecutionEngine::new(storage);

        engine.execute("CREATE TABLE t1 (id INTEGER)").unwrap();
        engine.execute("CREATE TABLE t2 (id INTEGER)").unwrap();
        engine.execute("CREATE TABLE t3 (id INTEGER)").unwrap();

        for i in 0..500 {
            engine
                .execute(&format!("INSERT INTO t1 VALUES ({})", i))
                .unwrap();
        }
        for i in 0..50 {
            engine
                .execute(&format!("INSERT INTO t2 VALUES ({})", i))
                .unwrap();
        }
        for i in 0..5 {
            engine
                .execute(&format!("INSERT INTO t3 VALUES ({})", i))
                .unwrap();
        }

        // Analyze to get accurate stats
        engine.execute("ANALYZE t1").unwrap();
        engine.execute("ANALYZE t2").unwrap();
        engine.execute("ANALYZE t3").unwrap();

        let tables = vec!["t1", "t2", "t3"];
        let optimal = engine.optimize_join_order(&tables);

        // Smallest (t3 with 5 rows) should be first after ANALYZE
        assert_eq!(optimal[0], "t3");
    }
}
