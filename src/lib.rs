//! SQLRustGo Database System Library
//!
//! A Rust implementation of a SQL-92 compliant database system.
//! This crate re-exports functionality from the modular crates/ workspace.

pub use sqlrustgo_executor::{Executor, ExecutorResult};
pub use sqlrustgo_optimizer::Optimizer as QueryOptimizer;
pub use sqlrustgo_parser::lexer::tokenize;
pub use sqlrustgo_parser::{
    parse, Expression, GrantStatement, Lexer, Privilege, RevokeStatement, SetOperation, Statement,
    Token,
};
pub use sqlrustgo_planner::{LogicalPlan, Optimizer, PhysicalPlan, Planner, SetOperationType};
pub use sqlrustgo_storage::{
    BPlusTree, BufferPool, FileStorage, MemoryStorage, Page, StorageEngine, ViewInfo,
};
pub use sqlrustgo_types::{SqlError, SqlResult, Value};

use std::sync::{Arc, RwLock};

/// Evaluate a WHERE clause expression against a row
fn evaluate_where_clause(
    expr: &sqlrustgo_parser::Expression,
    row: &[Value],
    columns: &[sqlrustgo_storage::ColumnDefinition],
) -> bool {
    match expr {
        sqlrustgo_parser::Expression::BinaryOp(left, op, right) => {
            let left_val = evaluate_expr(left, row, columns);
            let right_val = evaluate_expr(right, row, columns);
            compare_values(&left_val, op, &right_val)
        }
        sqlrustgo_parser::Expression::Identifier(name) => {
            // Single identifier in WHERE - treat as boolean (for EXISTS subqueries, etc)
            if let Some(idx) = columns.iter().position(|c| c.name.eq_ignore_ascii_case(name)) {
                if let Some(val) = row.get(idx) {
                    return val.to_bool();
                }
            }
            false
        }
        sqlrustgo_parser::Expression::Literal(s) => {
            // For IN clauses, we might get a comma-separated list
            s.to_uppercase() != "FALSE" && s != "0"
        }
        sqlrustgo_parser::Expression::Wildcard => true,
    }
}

/// Evaluate an expression to a Value
fn evaluate_expr(
    expr: &sqlrustgo_parser::Expression,
    row: &[Value],
    columns: &[sqlrustgo_storage::ColumnDefinition],
) -> Value {
    match expr {
        sqlrustgo_parser::Expression::Literal(s) => {
            // Try to parse as number first
            if let Ok(n) = s.parse::<i64>() {
                Value::Integer(n)
            } else if let Ok(n) = s.parse::<f64>() {
                Value::Float(n)
            } else if s.eq_ignore_ascii_case("true") {
                Value::Boolean(true)
            } else if s.eq_ignore_ascii_case("false") {
                Value::Boolean(false)
            } else {
                Value::Text(s.clone())
            }
        }
        sqlrustgo_parser::Expression::Identifier(name) => {
            // Look up column by name (case-insensitive)
            if let Some(idx) = columns.iter().position(|c| c.name.eq_ignore_ascii_case(name)) {
                row.get(idx).cloned().unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }
        sqlrustgo_parser::Expression::BinaryOp(_, _, _) => {
            // For binary ops, we'd need to evaluate recursively
            // This shouldn't happen in a simple where clause evaluation
            Value::Null
        }
        sqlrustgo_parser::Expression::Wildcard => Value::Null,
    }
}

/// Compare two values with the given operator
fn compare_values(left: &Value, op: &str, right: &Value) -> bool {
    match op {
        "=" | "==" | "EQ" => left == right,
        "!=" | "<>" | "NE" => left != right,
        ">" | "GT" => {
            match (left, right) {
                (Value::Integer(l), Value::Integer(r)) => l > r,
                (Value::Float(l), Value::Float(r)) => l > r,
                (Value::Integer(l), Value::Float(r)) => (*l as f64) > *r,
                (Value::Float(l), Value::Integer(r)) => *l > (*r as f64),
                (Value::Text(l), Value::Text(r)) => l > r,
                _ => false,
            }
        }
        "<" | "LT" => {
            match (left, right) {
                (Value::Integer(l), Value::Integer(r)) => l < r,
                (Value::Float(l), Value::Float(r)) => l < r,
                (Value::Integer(l), Value::Float(r)) => (*l as f64) < *r,
                (Value::Float(l), Value::Integer(r)) => *l < (*r as f64),
                (Value::Text(l), Value::Text(r)) => l < r,
                _ => false,
            }
        }
        ">=" | "GE" => {
            match (left, right) {
                (Value::Integer(l), Value::Integer(r)) => l >= r,
                (Value::Float(l), Value::Float(r)) => l >= r,
                (Value::Integer(l), Value::Float(r)) => (*l as f64) >= *r,
                (Value::Float(l), Value::Integer(r)) => *l >= (*r as f64),
                (Value::Text(l), Value::Text(r)) => l >= r,
                _ => false,
            }
        }
        "<=" | "LE" => {
            match (left, right) {
                (Value::Integer(l), Value::Integer(r)) => l <= r,
                (Value::Float(l), Value::Float(r)) => l <= r,
                (Value::Integer(l), Value::Float(r)) => (*l as f64) <= *r,
                (Value::Float(l), Value::Integer(r)) => *l <= (*r as f64),
                (Value::Text(l), Value::Text(r)) => l <= r,
                _ => false,
            }
        }
        "LIKE" | "like" => {
            if let (Value::Text(pattern), Value::Text(text)) = (right, left) {
                like_match(text, pattern)
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Simple LIKE pattern matching (supports % and _)
fn like_match(text: &str, pattern: &str) -> bool {
    // Simple implementation for LIKE patterns
    // % matches any sequence of characters
    // _ matches any single character
    let mut text_chars: Vec<char> = text.chars().collect();
    let mut pattern_chars: Vec<char> = pattern.chars().collect();

    fn do_match(pi: usize, ti: usize, pc: &[char], tc: &[char]) -> bool {
        if pi == pc.len() {
            ti == tc.len()
        } else if pc[pi] == '%' {
            // % matches any sequence - try matching remaining pattern at each position
            // or skip the % and continue
            (ti < tc.len() && do_match(pi + 1, ti, pc, tc)) || (ti < tc.len() && do_match(pi, ti + 1, pc, tc))
        } else if pc[pi] == '_' {
            ti < tc.len() && do_match(pi + 1, ti + 1, pc, tc)
        } else if ti < tc.len() && pc[pi] == tc[ti] {
            do_match(pi + 1, ti + 1, pc, tc)
        } else {
            false
        }
    }

    do_match(0, 0, &pattern_chars, &text_chars)
}

/// Compute an aggregate function over a set of rows
fn compute_aggregate(
    agg: &sqlrustgo_parser::parser::AggregateCall,
    rows: &[Vec<Value>],
    columns: &[sqlrustgo_storage::ColumnDefinition],
) -> Value {
    use sqlrustgo_parser::parser::AggregateFunction;

    // Get the column index for the aggregate argument
    let col_idx: Option<usize> = if agg.args.is_empty() {
        // COUNT(*) has no arguments
        None
    } else {
        match &agg.args[0] {
            sqlrustgo_parser::Expression::Identifier(name) => {
                columns.iter().position(|c| c.name.eq_ignore_ascii_case(name))
            }
            sqlrustgo_parser::Expression::Wildcard => Some(0), // Will be ignored for COUNT
            _ => None,
        }
    };

    match agg.func {
        AggregateFunction::Count => {
            if agg.args.is_empty() || matches!(agg.args.get(0), Some(sqlrustgo_parser::Expression::Wildcard)) {
                // COUNT(*) - count all rows
                Value::Integer(rows.len() as i64)
            } else {
                // COUNT(column) - count non-null values
                let mut count = 0i64;
                for row in rows {
                    if let Some(idx) = col_idx {
                        if let Some(val) = row.get(idx) {
                            if *val != Value::Null {
                                count += 1;
                            }
                        }
                    }
                }
                Value::Integer(count)
            }
        }
        AggregateFunction::Sum => {
            let mut sum = 0i64;
            for row in rows {
                if let Some(idx) = col_idx {
                    if let Some(val) = row.get(idx) {
                        if let Value::Integer(n) = val {
                            sum += n;
                        } else if let Value::Float(n) = val {
                            sum += *n as i64;
                        }
                    }
                }
            }
            Value::Integer(sum)
        }
        AggregateFunction::Avg => {
            let mut sum = 0.0f64;
            let mut count = 0i64;
            for row in rows {
                if let Some(idx) = col_idx {
                    if let Some(val) = row.get(idx) {
                        if let Value::Integer(n) = val {
                            sum += *n as f64;
                            count += 1;
                        } else if let Value::Float(n) = val {
                            sum += *n;
                            count += 1;
                        }
                    }
                }
            }
            if count > 0 {
                Value::Float(sum / count as f64)
            } else {
                Value::Null
            }
        }
        AggregateFunction::Min => {
            let mut min: Option<Value> = None;
            for row in rows {
                if let Some(idx) = col_idx {
                    if let Some(val) = row.get(idx) {
                        if *val != Value::Null {
                            match &min {
                                None => min = Some(val.clone()),
                                Some(m) => {
                                    if val < m {
                                        min = Some(val.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            min.unwrap_or(Value::Null)
        }
        AggregateFunction::Max => {
            let mut max: Option<Value> = None;
            for row in rows {
                if let Some(idx) = col_idx {
                    if let Some(val) = row.get(idx) {
                        if *val != Value::Null {
                            match &max {
                                None => max = Some(val.clone()),
                                Some(m) => {
                                    if val > m {
                                        max = Some(val.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            max.unwrap_or(Value::Null)
        }
    }
}

pub struct ExecutionEngine {
    pub storage: Arc<RwLock<dyn StorageEngine>>,
}

impl ExecutionEngine {
    pub fn new(storage: Arc<RwLock<dyn StorageEngine>>) -> Self {
        Self { storage }
    }

    pub fn execute(&mut self, statement: Statement) -> Result<ExecutorResult, SqlError> {
        match statement {
            Statement::Insert(insert) => {
                let table_name = &insert.table;
                let mut storage = self.storage.write().unwrap();
                if !storage.has_table(table_name) {
                    return Err(SqlError::ExecutionError(format!(
                        "Table '{}' not found",
                        table_name
                    )));
                }

                // Get table info to determine column types
                let table_info = storage.get_table_info(table_name).ok();

                let records: Vec<Vec<Value>> = insert
                    .values
                    .iter()
                    .map(|row| {
                        row.iter()
                            .enumerate()
                            .map(|(col_idx, expr)| {
                                match expr {
                                    Expression::Literal(value) => {
                                        // Determine the column type from table schema
                                        if let Some(ref info) = table_info {
                                            if col_idx < info.columns.len() {
                                                let col_type = &info.columns[col_idx].data_type;
                                                let upper = col_type.to_uppercase();
                                                // Parse numeric values for appropriate types
                                                if upper.contains("INT") || upper == "BIGINT" || upper == "SMALLINT" {
                                                    if let Ok(n) = value.parse::<i64>() {
                                                        return Value::Integer(n);
                                                    }
                                                } else if upper == "FLOAT" || upper == "DOUBLE" || upper == "DECIMAL" {
                                                    if let Ok(n) = value.parse::<f64>() {
                                                        return Value::Float(n);
                                                    }
                                                } else if upper == "BOOLEAN" {
                                                    if value.to_uppercase() == "TRUE" {
                                                        return Value::Boolean(true);
                                                    } else if value.to_uppercase() == "FALSE" {
                                                        return Value::Boolean(false);
                                                    }
                                                }
                                            }
                                        }
                                        // Default to text for strings and unrecognized types
                                        Value::Text(value.clone())
                                    }
                                    _ => Value::Null,
                                }
                            })
                            .collect()
                    })
                    .collect();

                storage.insert(table_name, records)?;
                Ok(ExecutorResult::new(vec![], insert.values.len()))
            }
            Statement::CreateTable(create) => {
                let mut storage = self.storage.write().unwrap();

                // If IF NOT EXISTS is set, check if table already exists
                if create.if_not_exists && storage.has_table(&create.name) {
                    return Ok(ExecutorResult::new(vec![], 0));
                }

                let columns: Vec<sqlrustgo_storage::ColumnDefinition> = create
                    .columns
                    .iter()
                    .map(|col| sqlrustgo_storage::ColumnDefinition {
                        name: col.name.clone(),
                        data_type: col.data_type.clone(),
                        nullable: col.nullable,
                        is_unique: false,
                    })
                    .collect();

                let table_info = sqlrustgo_storage::TableInfo {
                    name: create.name.clone(),
                    columns,
                };

                storage.create_table(&table_info)?;
                Ok(ExecutorResult::new(vec![], 0))
            }
            Statement::CreateView(create) => {
                let mut storage = self.storage.write().unwrap();
                let view_info = sqlrustgo_storage::ViewInfo {
                    name: create.name.clone(),
                    query: create.query.clone(),
                    schema: sqlrustgo_storage::TableInfo {
                        name: create.name.clone(),
                        columns: vec![],
                    },
                    records: vec![],
                };
                storage.create_view(view_info)?;
                Ok(ExecutorResult::new(vec![], 0))
            }
            Statement::Analyze(analyze) => {
                let table_name = analyze.table_name.ok_or_else(|| {
                    SqlError::ExecutionError("ANALYZE requires a table name".to_string())
                })?;
                let storage = self.storage.read().unwrap();
                let stats = storage.analyze_table(&table_name)?;

                let mut rows: Vec<Vec<Value>> = vec![
                    vec![
                        Value::Text("table".to_string()),
                        Value::Text(stats.table_name.clone()),
                    ],
                    vec![
                        Value::Text("row_count".to_string()),
                        Value::Text(stats.row_count.to_string()),
                    ],
                ];

                for col_stat in &stats.column_stats {
                    let col_row = vec![
                        Value::Text(format!("column:{}", col_stat.column_name)),
                        Value::Text(format!(
                            "distinct:{}, null:{}",
                            col_stat.distinct_count, col_stat.null_count
                        )),
                    ];
                    rows.push(col_row);
                }

                let row_count = rows.len();
                Ok(ExecutorResult::new(rows, row_count))
            }
            Statement::Delete(delete) => {
                let mut storage = self.storage.write().unwrap();
                if !storage.has_table(&delete.table) {
                    return Err(SqlError::ExecutionError(format!(
                        "Table '{}' not found",
                        delete.table
                    )));
                }

                let table_info = storage.get_table_info(&delete.table).ok();
                let columns = table_info.map(|info| info.columns.clone()).unwrap_or_default();
                let mut rows = storage.scan(&delete.table).unwrap_or_default();

                // Filter rows by WHERE clause - keep rows that DON'T match the WHERE clause
                let original_count = rows.len();
                rows.retain(|row| {
                    if let Some(ref where_clause) = delete.where_clause {
                        !evaluate_where_clause(where_clause, row, &columns) // Invert: keep rows that don't match
                    } else {
                        true // Without WHERE, keep all rows (nothing to delete)
                    }
                });

                let deleted_count = original_count - rows.len();

                // Delete matching rows and insert remaining rows
                if deleted_count > 0 {
                    let _ = storage.delete(&delete.table, &[]);
                    if !rows.is_empty() {
                        storage.insert(&delete.table, rows)?;
                    }
                }

                Ok(ExecutorResult::new(vec![], deleted_count))
            }
            Statement::Update(update) => {
                let mut storage = self.storage.write().unwrap();
                if !storage.has_table(&update.table) {
                    return Err(SqlError::ExecutionError(format!(
                        "Table '{}' not found",
                        update.table
                    )));
                }

                let table_info = storage.get_table_info(&update.table).ok();
                let columns = table_info.map(|info| info.columns.clone()).unwrap_or_default();
                let mut rows = storage.scan(&update.table).unwrap_or_default();

                // Filter rows by WHERE clause and apply updates
                let mut updated_count = 0;
                for row in rows.iter_mut() {
                    if let Some(ref where_clause) = update.where_clause {
                        if !evaluate_where_clause(where_clause, row, &columns) {
                            continue;
                        }
                    } else {
                        // No WHERE clause - update all rows (but we need at least one filter)
                        // For UPDATE without WHERE, we still require some condition
                        // For now, skip rows without WHERE
                        continue;
                    }

                    // Apply SET clauses
                    for (col_name, expr) in &update.set_clauses {
                        if let Some(col_idx) = columns.iter().position(|c| c.name.eq_ignore_ascii_case(col_name)) {
                            let new_value = evaluate_expr(expr, row, &columns);
                            if col_idx < row.len() {
                                row[col_idx] = new_value;
                                updated_count += 1;
                            }
                        }
                    }
                }

                // Write back updated rows
                if updated_count > 0 {
                    // Delete all existing rows
                    let _ = storage.delete(&update.table, &[]);
                    // Insert updated rows
                    storage.insert(&update.table, rows)?;
                }

                Ok(ExecutorResult::new(vec![], updated_count))
            }
            Statement::Select(select) => {
                let storage = self.storage.read().unwrap();
                if !storage.has_table(&select.table) {
                    return Err(SqlError::ExecutionError(format!(
                        "Table '{}' not found",
                        select.table
                    )));
                }
                let table_info = storage.get_table_info(&select.table).ok();
                let columns = table_info.map(|info| info.columns.clone()).unwrap_or_default();
                let rows = storage.scan(&select.table).unwrap_or_default();

                // Apply WHERE clause filter if present
                let filtered_rows: Vec<Vec<Value>> = if let Some(ref where_clause) = select.where_clause {
                    rows.into_iter()
                        .filter(|row| {
                            evaluate_where_clause(where_clause, row, &columns)
                        })
                        .collect()
                } else {
                    rows
                };

                // Handle aggregates if present
                if !select.aggregates.is_empty() {
                    let result_row: Vec<Value> = select.aggregates.iter().map(|agg| {
                        compute_aggregate(agg, &filtered_rows, &columns)
                    }).collect();
                    return Ok(ExecutorResult::new(vec![result_row], 0));
                }

                // Apply column projection if specified (not SELECT *)
                // SELECT * has columns = [{"*", None}]
                let is_select_star = select.columns.len() == 1 && select.columns[0].name == "*";
                let projected_rows: Vec<Vec<Value>> = if is_select_star {
                    // SELECT * - return all columns
                    filtered_rows
                } else if select.columns.is_empty() {
                    // No columns specified (edge case) - return all columns
                    filtered_rows
                } else {
                    // Project only the selected columns
                    filtered_rows.into_iter().map(|row| {
                        select.columns.iter().filter_map(|col| {
                            columns.iter().position(|c| c.name.eq_ignore_ascii_case(&col.name))
                                .and_then(|idx| row.get(idx).cloned())
                        }).collect()
                    }).collect()
                };

                Ok(ExecutorResult::new(projected_rows, 0))
            }
            Statement::Explain(explain) => {
                let start = std::time::Instant::now();
                let result = self.execute(*explain.query)?;
                if explain.analyze {
                    let duration = start.elapsed();
                    let mut rows = result.rows;
                    rows.push(vec![
                        Value::Text("Execution Time".to_string()),
                        Value::Text(format!("{:?}", duration)),
                    ]);
                    return Ok(ExecutorResult::new(rows, result.affected_rows));
                }
                Ok(result)
            }
            _ => Ok(ExecutorResult::empty()),
        }
    }

    pub fn execute_plan(&self, plan: &dyn PhysicalPlan) -> Result<ExecutorResult, SqlError> {
        let storage = self.storage.read().unwrap();
        match plan.name() {
            "SeqScan" => {
                let rows = storage.scan(plan.table_name())?;
                Ok(ExecutorResult::new(rows, 0))
            }
            "Filter" => {
                let filter_plan = plan
                    .as_any()
                    .downcast_ref::<sqlrustgo_planner::FilterExec>()
                    .ok_or_else(|| {
                        SqlError::ExecutionError("Failed to downcast FilterExec".to_string())
                    })?;

                let child = filter_plan.input();
                let input_result = self.execute_plan(child)?;

                let predicate = filter_plan.predicate();
                let schema = child.schema();
                let filtered_rows: Vec<Vec<Value>> = input_result
                    .rows
                    .into_iter()
                    .filter(|row| predicate.matches(row, schema))
                    .collect();

                Ok(ExecutorResult::new(filtered_rows, 0))
            }
            "Projection" => {
                let proj_plan = plan
                    .as_any()
                    .downcast_ref::<sqlrustgo_planner::ProjectionExec>()
                    .ok_or_else(|| {
                        SqlError::ExecutionError("Failed to downcast ProjectionExec".to_string())
                    })?;

                let child = proj_plan.input();
                let input_result = self.execute_plan(child)?;

                let exprs = proj_plan.expr();
                let _output_schema = plan.schema();
                let projected_rows: Vec<Vec<Value>> = input_result
                    .rows
                    .iter()
                    .map(|row| {
                        exprs
                            .iter()
                            .filter_map(|expr| expr.evaluate(row, child.schema()))
                            .collect()
                    })
                    .collect();

                Ok(ExecutorResult::new(projected_rows, 0))
            }
            _ => Ok(ExecutorResult::empty()),
        }
    }
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self {
            storage: Arc::new(RwLock::new(MemoryStorage::new())),
        }
    }
}

pub fn init() {
    println!("SQLRustGo Database System initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        init();
    }

    #[test]
    fn test_module_exports() {
        let _ = tokenize("SELECT 1");
        let _ = parse("SELECT 1");
        let _ = Value::Integer(1);
    }

    #[test]
    fn test_sql_result_alias() {
        let result: SqlResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_optimizer_alias() {
        let _: Option<Box<dyn sqlrustgo_optimizer::Optimizer>> = None;
    }

    #[test]
    fn test_physical_plan_trait() {
        let _: Option<Box<dyn PhysicalPlan>> = None;
    }

    #[test]
    fn test_execution_engine_new() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(sqlrustgo_parser::parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();
        let stmt = sqlrustgo_parser::parse("SELECT * FROM users").unwrap();
        assert_eq!(engine.execute(stmt).unwrap().rows.len(), 0);
    }

    #[test]
    fn test_execution_engine_default() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(sqlrustgo_parser::parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();
        let stmt = sqlrustgo_parser::parse("SELECT * FROM users").unwrap();
        assert_eq!(engine.execute(stmt).unwrap().rows.len(), 0);
    }

    #[test]
    fn test_execute_plan_seqscan() {
        use sqlrustgo_planner::{DataType, Field, Schema, SeqScanExec};

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

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let plan = SeqScanExec::new("users".to_string(), schema);
        let result = engine.execute_plan(&plan).unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][0], Value::Integer(1));
        assert_eq!(result.rows[0][1], Value::Text("Alice".to_string()));
    }

    #[test]
    fn test_execute_plan_filter() {
        use sqlrustgo_planner::{DataType, Expr, Field, FilterExec, Operator, Schema, SeqScanExec};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                    vec![Value::Integer(2), Value::Text("Bob".to_string())],
                    vec![Value::Integer(3), Value::Text("Charlie".to_string())],
                ],
            )
            .unwrap();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);

        let scan = SeqScanExec::new("users".to_string(), schema.clone());
        let predicate = Expr::binary_expr(
            Expr::column("id"),
            Operator::Gt,
            Expr::literal(Value::Integer(1)),
        );
        let filter = FilterExec::new(Box::new(scan), predicate);
        let result = engine.execute_plan(&filter).unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][0], Value::Integer(2));
    }

    #[test]
    fn test_execute_plan_projection() {
        use sqlrustgo_planner::{DataType, Expr, Field, ProjectionExec, Schema, SeqScanExec};

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

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);

        let scan = SeqScanExec::new("users".to_string(), schema.clone());
        let proj_schema = Schema::new(vec![Field::new("name".to_string(), DataType::Text)]);
        let projection =
            ProjectionExec::new(Box::new(scan), vec![Expr::column("name")], proj_schema);
        let result = engine.execute_plan(&projection).unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][0], Value::Text("Alice".to_string()));
    }

    #[test]
    fn test_storage_engine_export() {
        let _: Option<Box<dyn StorageEngine>> = None;
    }

    #[test]
    fn test_executor_export() {
        let _: Option<Box<dyn Executor>> = None;
    }

    #[test]
    fn test_planner_export() {
        let _: Option<Box<dyn Planner>> = None;
    }
}
