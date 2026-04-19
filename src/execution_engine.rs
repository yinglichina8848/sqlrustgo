//! ExecutionEngine - high-level SQL execution API
//! Provides a simple interface for executing SQL statements against a storage backend.

#![allow(unused_variables, unused_imports)]

use crate::{parse, SqlError, SqlResult, Value};
use sqlrustgo_catalog::stored_proc::{ParamMode, StoredProcParam, StoredProcStatement};
use sqlrustgo_catalog::{Catalog, StoredProcedure};
use sqlrustgo_executor::stored_proc::StoredProcExecutor;
use sqlrustgo_executor::ExecutorResult;
use sqlrustgo_parser::parser::{
    AggregateCall, AggregateFunction, CallStatement, CreateIndexStatement,
    CreateProcedureStatement, CreateTableStatement, CreateTriggerStatement, DropTableStatement,
    InsertStatement, SelectStatement, StoredProcParam as ParserStoredProcParam,
    StoredProcParamMode as ParserParamMode, StoredProcStatement as ParserStatement,
};
use sqlrustgo_parser::{DeleteStatement, Expression, Statement, UpdateStatement};
use sqlrustgo_storage::{ColumnDefinition, MemoryStorage, StorageEngine, TableInfo};
use sqlrustgo_types::Value as SqlValue;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Execution engine for SQL statements
pub struct ExecutionEngine<S: StorageEngine> {
    storage: Arc<RwLock<S>>,
    catalog: Option<Arc<RwLock<Catalog>>>,
    stats: Arc<RwLock<ExecutionStats>>,
    cbo_enabled: bool,
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
        }
    }

    /// Create a new execution engine with CBO configuration
    pub fn with_cbo(storage: Arc<RwLock<S>>, cbo_enabled: bool) -> Self {
        Self {
            storage,
            catalog: None,
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
            cbo_enabled,
        }
    }

    /// Create a new execution engine with a catalog
    pub fn with_catalog(storage: Arc<RwLock<S>>, catalog: Arc<RwLock<Catalog>>) -> Self {
        Self {
            storage,
            catalog: Some(catalog),
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
            cbo_enabled: true,
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

    /// Execute a SQL statement and return results
    pub fn execute(&mut self, sql: &str) -> SqlResult<ExecutorResult> {
        let statement = parse(sql).map_err(|e| SqlError::ParseError(e.to_string()))?;

        match statement {
            Statement::Select(ref select) => self.execute_select(select),
            Statement::Insert(ref insert) => self.execute_insert(insert),
            Statement::Update(ref update) => self.execute_update(update),
            Statement::Delete(ref delete) => self.execute_delete(delete),
            Statement::CreateTable(ref create) => self.execute_create_table(create),
            Statement::DropTable(ref drop) => self.execute_drop_table(drop),
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
                // Extract left and right SelectStatements from the Union
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

                // Append rows from right to left
                left_result.rows.extend(right_result.rows);

                // If not UNION ALL, deduplicate
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
            _ => Err(SqlError::ExecutionError(
                "Unsupported statement type".to_string(),
            )),
        }
    }

    fn execute_select(&self, select: &SelectStatement) -> SqlResult<ExecutorResult> {
        let storage = self.storage.read().unwrap();

        // Get table info
        let table_info = storage.get_table_info(&select.table)?;

        // Scan all rows
        let mut rows = storage.scan(&select.table)?;

        // Apply WHERE clause filter
        if let Some(ref where_expr) = select.where_clause {
            rows.retain(|row| evaluate_where_clause(where_expr, row, &table_info));
        }

        // Handle aggregate functions
        if !select.aggregates.is_empty() {
            // Group rows by GROUP BY expressions
            let group_exprs = &select.group_by;
            if group_exprs.is_empty() {
                // No GROUP BY - compute aggregates over all filtered rows
                let mut agg_values =
                    self.compute_aggregates(&select.aggregates, &rows, &table_info)?;

                // Apply HAVING clause if present
                if let Some(ref having_expr) = select.having {
                    if !evaluate_where_clause(having_expr, &agg_values, &table_info) {
                        return Ok(ExecutorResult::new(vec![], 0));
                    }
                }

                return Ok(ExecutorResult::new(vec![agg_values], 1));
            } else {
                // GROUP BY - group rows first
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

                let mut agg_result_rows: Vec<Vec<Value>> = groups
                    .values()
                    .map(|group_rows| {
                        self.compute_aggregates(&select.aggregates, group_rows, &table_info)
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                // Apply HAVING clause if present (filters aggregated groups)
                if let Some(ref having_expr) = select.having {
                    agg_result_rows
                        .retain(|row| evaluate_where_clause(having_expr, row, &table_info));
                }

                let row_count = agg_result_rows.len();
                return Ok(ExecutorResult::new(agg_result_rows, row_count));
            }
        }

        let row_count = rows.len();

        // Apply ORDER BY
        if !select.order_by.is_empty() {
            let order_exprs = &select.order_by;
            rows.sort_by(|a, b| {
                for expr in order_exprs {
                    let a_val = evaluate_expression(&expr.expression, a, &table_info)
                        .unwrap_or(Value::Null);
                    let b_val = evaluate_expression(&expr.expression, b, &table_info)
                        .unwrap_or(Value::Null);
                    let cmp = compare_values_for_sort(&a_val, &b_val);
                    let result = if expr.ascending { cmp } else { -cmp };
                    if result != 0 {
                        return if result < 0 {
                            std::cmp::Ordering::Less
                        } else {
                            std::cmp::Ordering::Greater
                        };
                    }
                }
                std::cmp::Ordering::Equal
            });
        }

        // Apply OFFSET
        if let Some(offset_n) = select.offset {
            let offset_n = offset_n as usize;
            if offset_n < rows.len() {
                rows = rows[offset_n..].to_vec();
            } else {
                rows.clear();
            }
        }

        // Apply LIMIT
        if let Some(limit_n) = select.limit {
            let limit_n = limit_n as usize;
            if limit_n < rows.len() {
                rows.truncate(limit_n);
            }
        }

        // Apply DISTINCT - remove duplicate rows
        if select.distinct {
            rows.sort();
            rows.dedup();
        }

        let row_count = rows.len();
        Ok(ExecutorResult::new(rows, row_count))
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
                AggregateFunction::Count => Value::Integer(values.len() as i64),
                AggregateFunction::Sum => {
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
                    Value::Integer(sum)
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

    fn execute_insert(&self, insert: &InsertStatement) -> SqlResult<ExecutorResult> {
        let mut storage = self.storage.write().unwrap();

        // Get table info for FK validation
        let table_info = storage.get_table_info(&insert.table)?;

        // Build column name to index map
        let _col_indices: std::collections::HashMap<&str, usize> = if insert.columns.is_empty() {
            std::collections::HashMap::new()
        } else {
            insert
                .columns
                .iter()
                .enumerate()
                .map(|(i, c)| (c.as_str(), i))
                .collect()
        };

        // Convert expressions to records and validate FK constraints
        let mut all_records = Vec::new();
        for row_exprs in &insert.values {
            let mut record = Vec::with_capacity(row_exprs.len());
            for (_i, expr) in row_exprs.iter().enumerate() {
                let val = expression_to_value(expr);
                record.push(val);
            }

            // Validate foreign key constraints before insert
            if !table_info.foreign_keys.is_empty() {
                validate_foreign_keys(&*storage, &table_info, &record, &insert.columns)?;
            }

            all_records.push(record);
        }

        storage.insert(&insert.table, all_records)?;
        Ok(ExecutorResult::new(vec![], insert.values.len()))
    }

    fn execute_update(&self, update: &UpdateStatement) -> SqlResult<ExecutorResult> {
        // If no WHERE clause, use the simple storage.update() path
        if update.where_clause.is_none() {
            let mut storage = self.storage.write().unwrap();
            let count = storage.update(&update.table, &[], &[])?;
            return Ok(ExecutorResult::new(vec![], count));
        }

        // Get table info and scan rows
        let table_info = {
            let storage = self.storage.read().unwrap();
            storage.get_table_info(&update.table)?
        };

        let all_rows = {
            let storage = self.storage.read().unwrap();
            storage.scan(&update.table)?
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
            .into_iter()
            .map(|mut row| {
                for &(col_idx, ref set_expr) in &set_col_indices {
                    let new_val =
                        evaluate_expression(set_expr, &row, &table_info).unwrap_or(Value::Null);
                    if col_idx < row.len() {
                        row[col_idx] = new_val;
                    }
                }
                row
            })
            .collect();

        // Get rows to keep (non-matching rows)
        let rows_to_keep: Vec<Vec<Value>> = {
            let storage = self.storage.read().unwrap();
            let all_rows = storage.scan(&update.table)?;
            all_rows
                .into_iter()
                .filter(|row| !evaluate_where_clause(where_clause, row, &table_info))
                .collect()
        };

        // Delete all rows and re-insert updated + kept rows
        {
            let mut storage = self.storage.write().unwrap();
            storage.delete(&update.table, &[])?;
            if !rows_to_keep.is_empty() {
                storage.insert(&update.table, rows_to_keep)?;
            }
            if !updated_rows.is_empty() {
                storage.insert(&update.table, updated_rows)?;
            }
        }

        Ok(ExecutorResult::new(vec![], count))
    }

    fn execute_delete(&self, delete: &DeleteStatement) -> SqlResult<ExecutorResult> {
        // If no WHERE clause, delete all rows (current behavior is correct)
        if delete.where_clause.is_none() {
            let mut storage = self.storage.write().unwrap();
            let count = storage.delete(&delete.table, &[])?;
            return Ok(ExecutorResult::new(vec![], count));
        }

        // Scan all rows from the table
        let all_rows = {
            let storage = self.storage.read().unwrap();
            storage.scan(&delete.table)?
        };

        // Get table info to find column indices
        let table_info = {
            let storage = self.storage.read().unwrap();
            storage.get_table_info(&delete.table)?
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

        // Delete all rows and re-insert non-matching ones
        // Since storage.delete ignores filters, we need to:
        // 1. Scan remaining rows (those that don't match WHERE)
        // 2. Delete all
        // 3. Re-insert the non-matching ones
        let rows_to_keep: Vec<Vec<Value>> = {
            let storage = self.storage.read().unwrap();
            let all_rows = storage.scan(&delete.table)?;
            all_rows
                .into_iter()
                .filter(|row| !evaluate_where_clause(where_clause, row, &table_info))
                .collect()
        };

        {
            let mut storage = self.storage.write().unwrap();
            storage.delete(&delete.table, &[])?; // Delete all
            if !rows_to_keep.is_empty() {
                storage.insert(&delete.table, rows_to_keep)?;
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
        };
        storage.create_table(&info)?;
        Ok(ExecutorResult::empty())
    }

    fn execute_drop_table(&self, drop: &DropTableStatement) -> SqlResult<ExecutorResult> {
        let mut storage = self.storage.write().unwrap();
        storage.drop_table(&drop.name)?;
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
            .map_err(|e| SqlError::ExecutionError(e))?;

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
}

impl ExecutionEngine<MemoryStorage> {
    /// Create a new execution engine backed by MemoryStorage with CBO enabled
    pub fn with_memory() -> Self {
        Self {
            storage: Arc::new(RwLock::new(MemoryStorage::new())),
            catalog: None,
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
            cbo_enabled: true,
        }
    }

    /// Create a new execution engine backed by MemoryStorage with custom CBO setting
    pub fn with_memory_and_cbo(cbo_enabled: bool) -> Self {
        Self {
            storage: Arc::new(RwLock::new(MemoryStorage::new())),
            catalog: None,
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
            cbo_enabled,
        }
    }

    /// Create a new execution engine with catalog
    pub fn with_memory_and_catalog(catalog: Arc<RwLock<Catalog>>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(MemoryStorage::new())),
            catalog: Some(catalog),
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
            cbo_enabled: true,
        }
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
    } else if s.starts_with('@') {
        Value::Text(s.to_string())
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
fn evaluate_where_clause(expr: &Expression, row: &[Value], table_info: &TableInfo) -> bool {
    match expr {
        // Handle AND conditions
        Expression::BinaryOp(left, op, right) if op.to_uppercase() == "AND" => {
            evaluate_where_clause(left, row, table_info)
                && evaluate_where_clause(right, row, table_info)
        }
        // Handle OR conditions
        Expression::BinaryOp(left, op, right) if op.to_uppercase() == "OR" => {
            evaluate_where_clause(left, row, table_info)
                || evaluate_where_clause(right, row, table_info)
        }
        // Handle IS NULL
        Expression::BinaryOp(left, op, right)
            if op.to_uppercase() == "IS"
                && matches!(right.as_ref(), Expression::Literal(s) if s.to_uppercase() == "NULL") =>
        {
            if let Expression::Identifier(col_name) = left.as_ref() {
                if let Some(col_idx) = find_column_index(col_name, table_info) {
                    if let Some(row_val) = row.get(col_idx) {
                        return matches!(row_val, Value::Null);
                    }
                }
            }
            false
        }
        // Handle IS NOT NULL
        Expression::BinaryOp(left, op, right)
            if op.to_uppercase() == "IS NOT"
                && matches!(right.as_ref(), Expression::Literal(s) if s.to_uppercase() == "NULL") =>
        {
            if let Expression::Identifier(col_name) = left.as_ref() {
                if let Some(col_idx) = find_column_index(col_name, table_info) {
                    if let Some(row_val) = row.get(col_idx) {
                        return !matches!(row_val, Value::Null);
                    }
                }
            }
            false
        }
        // Handle comparison operators (=, !=, >, <, >=, <=)
        Expression::BinaryOp(left, op, right) => {
            evaluate_binary_comparison(left, op, right, row, table_info)
        }
        // For other expressions, try to evaluate as a condition
        _ => {
            if let Ok(val) = evaluate_expression(expr, row, table_info) {
                if let Value::Boolean(b) = val {
                    b
                } else {
                    val != Value::Null
                }
            } else {
                false
            }
        }
    }
}

/// Evaluate a binary comparison expression
fn evaluate_binary_comparison(
    left: &Expression,
    op: &str,
    right: &Expression,
    row: &[Value],
    table_info: &TableInfo,
) -> bool {
    let left_val = evaluate_expression(left, row, table_info).unwrap_or(Value::Null);
    let right_val = evaluate_expression(right, row, table_info).unwrap_or(Value::Null);

    match op.to_uppercase().as_str() {
        "=" | "==" | "IS" => left_val == right_val,
        "!=" | "<>" => left_val != right_val,
        ">" => compare_values(&left_val, &right_val) > 0,
        ">=" => compare_values(&left_val, &right_val) >= 0,
        "<" => compare_values(&left_val, &right_val) < 0,
        "<=" => compare_values(&left_val, &right_val) <= 0,
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
                // If column not found, treat as literal value
                Ok(expression_to_value(expr))
            }
        }
        Expression::BinaryOp(left, op, right) => {
            let left_val = evaluate_expression(left, row, table_info).unwrap_or(Value::Null);
            let right_val = evaluate_expression(right, row, table_info).unwrap_or(Value::Null);
            Ok(evaluate_binary_op(&left_val, &right_val, op))
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

/// Compare two values for ORDER BY sorting. Returns -1, 0, or 1.
fn compare_values_for_sort(a: &Value, b: &Value) -> i32 {
    use std::cmp::Ordering;
    match (a, b) {
        (Value::Null, Value::Null) => 0,
        (Value::Null, _) => 1, // NULL sorts last (ascending)
        (_, Value::Null) => -1,
        (Value::Integer(a_i), Value::Integer(b_i)) => a_i.cmp(b_i) as i32,
        (Value::Float(a_f), Value::Float(b_f)) => {
            if a_f < b_f {
                -1
            } else if a_f > b_f {
                1
            } else {
                0
            }
        }
        (Value::Text(a_s), Value::Text(b_s)) => a_s.cmp(b_s) as i32,
        (Value::Boolean(a_b), Value::Boolean(b_b)) => a_b.cmp(b_b) as i32,
        _ => 0,
    }
}

/// Find the index of a column in the table info
fn find_column_index(col_name: &str, table_info: &TableInfo) -> Option<usize> {
    table_info
        .columns
        .iter()
        .position(|c| c.name.eq_ignore_ascii_case(col_name))
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
