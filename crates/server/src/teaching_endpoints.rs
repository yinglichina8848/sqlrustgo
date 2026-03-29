//! Teaching Enhanced API Endpoints
//!
//! Provides Web UI endpoints for visualizing query execution pipelines
//! and operator-level profiling. This is a key differentiation feature
//! that MySQL doesn't provide - making SQLRustGo 2.0 ideal for teaching
//! and learning query optimization.

use crate::metrics_endpoint::MetricsRegistry;
use serde::{Deserialize, Serialize};
use sqlrustgo_executor::{OperatorProfile, QueryTrace, GLOBAL_PROFILER, GLOBAL_TRACE_COLLECTOR};
use sqlrustgo_parser::{parse, Expression, Statement, TransactionCommand};
use sqlrustgo_storage::engine::{StorageEngine, Value};
use std::sync::{Arc, RwLock};

/// Teaching Enhanced endpoints configuration
#[derive(Clone)]
pub struct TeachingEndpoints {
    /// Enable pipeline visualization
    pub enable_pipeline_viz: bool,
    /// Enable profiling UI
    pub enable_profiling: bool,
    /// Enable trace logging
    pub enable_trace: bool,
    /// Maximum traces to keep
    pub max_traces: usize,
    /// Maximum profiles to keep
    pub max_profiles: usize,
}

impl Default for TeachingEndpoints {
    fn default() -> Self {
        Self {
            enable_pipeline_viz: true,
            enable_profiling: true,
            enable_trace: true,
            max_traces: 1000,
            max_profiles: 100,
        }
    }
}

/// Extended HTTP server with teaching enhanced features
#[derive(Clone)]
pub struct TeachingHttpServer {
    host: String,
    port: u16,
    actual_port: Arc<RwLock<u16>>,
    version: String,
    metrics_registry: Arc<RwLock<MetricsRegistry>>,
    teaching_endpoints: TeachingEndpoints,
    storage: Option<Arc<RwLock<dyn StorageEngine>>>,
}

impl TeachingHttpServer {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            actual_port: Arc::new(RwLock::new(port)),
            version: "2.0.0".to_string(),
            metrics_registry: Arc::new(RwLock::new(MetricsRegistry::new())),
            teaching_endpoints: TeachingEndpoints::default(),
            storage: None,
        }
    }

    pub fn with_storage(mut self, storage: Arc<RwLock<dyn StorageEngine>>) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn with_teaching_endpoints(mut self, endpoints: TeachingEndpoints) -> Self {
        self.teaching_endpoints = endpoints;
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn with_metrics_registry(mut self, registry: Arc<RwLock<MetricsRegistry>>) -> Self {
        self.metrics_registry = registry;
        self
    }

    /// Get server version
    pub fn get_version(&self) -> String {
        self.version.clone()
    }

    /// Get server port (actual port after binding, or configured port if already bound)
    pub fn get_port(&self) -> u16 {
        *self.actual_port.read().unwrap()
    }

    /// Bind to an available port (when port is 0) and return the actual port
    pub fn bind_to_available_port(&self) -> u16 {
        if self.port == 0 {
            if let Ok(listener) = std::net::TcpListener::bind(format!("{}:0", self.host)) {
                if let Ok(addr) = listener.local_addr() {
                    return addr.port();
                }
            }
        }
        self.port
    }

    /// Start the teaching enhanced HTTP server
    pub fn start(&self) -> Result<(), std::io::Error> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = std::net::TcpListener::bind(&addr)?;

        // Update actual_port after binding
        if let Ok(local_addr) = listener.local_addr() {
            *self.actual_port.write().unwrap() = local_addr.port();
        }

        let actual_addr = format!("{}:{}", self.host, *self.actual_port.read().unwrap());
        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║          SQLRustGo 2.0 - Teaching Enhanced Server               ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!(
            "║  Server started on http://{}                                ║",
            actual_addr
        );
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║  Standard Endpoints:                                             ║");
        println!("║    - GET /health/live     - Liveness probe                       ║");
        println!("║    - GET /health/ready    - Readiness probe                      ║");
        println!("║    - GET /metrics         - Prometheus metrics                   ║");

        if self.teaching_endpoints.enable_pipeline_viz {
            println!("║                                                                   ║");
            println!("║  Teaching Enhanced Endpoints:                                    ║");
            println!("║    - GET /teaching/pipeline       - Query pipeline visualization ║");
            println!("║    - GET /teaching/pipeline/json   - Pipeline as JSON            ║");
            println!("║    - GET /teaching/trace          - Vectorized trace log         ║");
        }

        if self.teaching_endpoints.enable_profiling {
            println!("║    - GET /teaching/profile        - Operator profiling report   ║");
            println!("║    - GET /teaching/profile/json   - Profile as JSON             ║");
            println!("║    - GET /teaching/profile/operators - Individual operator stats ║");
        }

        println!("╚══════════════════════════════════════════════════════════════════╝");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let version = self.version.clone();
                    let metrics_registry = Arc::clone(&self.metrics_registry);
                    let teaching = self.teaching_endpoints.clone();
                    let storage = self.storage.clone();

                    std::thread::spawn(move || {
                        let _ = handle_teaching_request(
                            &mut stream,
                            &version,
                            &metrics_registry,
                            &teaching,
                            &storage,
                        );
                    });
                }
                Err(e) => {
                    log::error!("Connection error: {}", e);
                }
            }
        }

        Ok(())
    }
}

/// SQL Execution request body
#[derive(Debug, Deserialize)]
struct SqlRequest {
    sql: String,
}

/// SQL Execution response body
#[derive(Debug, Serialize)]
struct SqlResponse {
    columns: Option<Vec<String>>,
    rows: Option<Vec<Vec<serde_json::Value>>>,
    affected_rows: usize,
    error: Option<String>,
}

/// Handle teaching enhanced HTTP requests
fn handle_teaching_request<T: std::io::Read + std::io::Write>(
    stream: &mut T,
    version: &str,
    metrics_registry: &Arc<RwLock<MetricsRegistry>>,
    teaching: &TeachingEndpoints,
    storage: &Option<Arc<RwLock<dyn StorageEngine>>>,
) -> Result<(), std::io::Error> {
    let mut buffer = [0u8; 2048];
    let bytes_read = stream.read(&mut buffer)?;

    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let lines: Vec<&str> = request.lines().collect();

    let (status, content_type, body) = if let Some(request_line) = lines.first() {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() >= 2 {
            let method = parts[0];
            let path = parts[1];

            // Handle POST /sql endpoint
            if method == "POST" && path == "/sql" {
                // Find the request body (after blank line)
                if let Some(body_start) = request.find("\r\n\r\n") {
                    let body_str = &request[body_start + 4..];
                    match serde_json::from_str::<SqlRequest>(body_str) {
                        Ok(sql_req) => {
                            if let Some(ref storage) = storage {
                                let result = execute_sql(&sql_req.sql, storage);
                                let response = match result {
                                    Ok(exec_result) => SqlResponse {
                                        columns: Some(exec_result.columns),
                                        rows: Some(exec_result.rows),
                                        affected_rows: exec_result.affected_rows,
                                        error: None,
                                    },
                                    Err(e) => SqlResponse {
                                        columns: None,
                                        rows: None,
                                        affected_rows: 0,
                                        error: Some(e.to_string()),
                                    },
                                };
                                let json = serde_json::to_string(&response).unwrap_or_else(|_| {
                                    r#"{"error":"Serialization error"}"#.to_string()
                                });
                                ("HTTP/1.1 200 OK", "application/json", json)
                            } else {
                                let json = serde_json::json!({
                                    "error": "Storage not configured. Use with_storage() to enable SQL execution"
                                }).to_string();
                                (
                                    "HTTP/1.1 500 Internal Server Error",
                                    "application/json",
                                    json,
                                )
                            }
                        }
                        Err(e) => {
                            let json = serde_json::json!({
                                "error": format!("Invalid request: {}", e)
                            })
                            .to_string();
                            ("HTTP/1.1 400 Bad Request", "application/json", json)
                        }
                    }
                } else {
                    let json = serde_json::json!({
                        "error": "Missing request body"
                    })
                    .to_string();
                    ("HTTP/1.1 400 Bad Request", "application/json", json)
                }
            } else {
                // Handle GET endpoints
                match path {
                    // Standard endpoints
                    "/health/live" => {
                        let body = serde_json::json!({
                            "status": "healthy",
                            "feature": "teaching_enhanced"
                        })
                        .to_string();
                        ("HTTP/1.1 200 OK", "application/json", body)
                    }
                    "/health/ready" => {
                        let body = serde_json::json!({
                            "status": "ready",
                            "version": version,
                            "edition": "SQLRustGo 2.0 - Teaching Enhanced"
                        })
                        .to_string();
                        ("HTTP/1.1 200 OK", "application/json", body)
                    }
                    "/metrics" => {
                        let registry = metrics_registry.read().unwrap();
                        let prometheus_output = registry.to_prometheus_format();
                        (
                            "HTTP/1.1 200 OK",
                            "text/plain; version=0.0.4",
                            prometheus_output,
                        )
                    }

                    // Teaching enhanced endpoints
                    "/teaching/pipeline" if teaching.enable_pipeline_viz => {
                        let html = generate_pipeline_html();
                        ("HTTP/1.1 200 OK", "text/html; charset=utf-8", html)
                    }

                    "/teaching/pipeline/json" if teaching.enable_pipeline_viz => {
                        let traces = GLOBAL_TRACE_COLLECTOR.get_traces();
                        let json = serde_json::to_string_pretty(&traces).unwrap_or_default();
                        ("HTTP/1.1 200 OK", "application/json", json)
                    }

                    "/teaching/pipeline/latest" if teaching.enable_pipeline_viz => {
                        let trace = GLOBAL_TRACE_COLLECTOR.latest();
                        match trace {
                            Some(t) => {
                                let html = generate_pipeline_detail_html(&t);
                                ("HTTP/1.1 200 OK", "text/html; charset=utf-8", html)
                            }
                            None => {
                                let body = serde_json::json!({
                                    "message": "No traces available",
                                    "hint": "Execute a query to see pipeline visualization"
                                })
                                .to_string();
                                ("HTTP/1.1 200 OK", "application/json", body)
                            }
                        }
                    }

                    "/teaching/trace" if teaching.enable_trace => {
                        let traces = GLOBAL_TRACE_COLLECTOR.get_traces();
                        let html = generate_trace_html(&traces);
                        ("HTTP/1.1 200 OK", "text/html; charset=utf-8", html)
                    }

                    "/teaching/trace/json" if teaching.enable_trace => {
                        let traces = GLOBAL_TRACE_COLLECTOR.get_traces();
                        let json = serde_json::to_string_pretty(&traces).unwrap_or_default();
                        ("HTTP/1.1 200 OK", "application/json", json)
                    }

                    "/teaching/profile" if teaching.enable_profiling => {
                        let report = GLOBAL_PROFILER.generate_report();
                        let html = generate_profile_html(&report);
                        ("HTTP/1.1 200 OK", "text/html; charset=utf-8", html)
                    }

                    "/teaching/profile/json" if teaching.enable_profiling => {
                        let json = GLOBAL_PROFILER.to_json();
                        ("HTTP/1.1 200 OK", "application/json", json)
                    }

                    "/teaching/profile/operators" if teaching.enable_profiling => {
                        let profiles = GLOBAL_PROFILER.get_sorted_profiles();
                        let html = generate_operators_html(&profiles);
                        ("HTTP/1.1 200 OK", "text/html; charset=utf-8", html)
                    }

                    "/teaching" => {
                        let html = generate_teaching_index_html();
                        ("HTTP/1.1 200 OK", "text/html; charset=utf-8", html)
                    }

                    _ => (
                        "HTTP/1.1 404 Not Found",
                        "application/json",
                        serde_json::json!({
                            "error": "Not Found",
                            "message": format!("Path '{}' not found", path)
                        })
                        .to_string(),
                    ),
                } // End of GET handler match
            } // End of else (POST /sql check)
        } else {
            (
                "HTTP/1.1 400 Bad Request",
                "application/json",
                r#"{"error": "Bad Request"}"#.to_string(),
            )
        }
    } else {
        (
            "HTTP/1.1 400 Bad Request",
            "application/json",
            r#"{"error": "Bad Request"}"#.to_string(),
        )
    };

    let response = format!(
        "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );

    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}

/// Execute SQL query and return result
fn execute_sql(
    sql: &str,
    storage: &Arc<RwLock<dyn StorageEngine>>,
) -> Result<SqlExecResult, String> {
    // Parse the SQL statement
    let statement = parse(sql).map_err(|e| format!("Parse error: {:?}", e))?;

    let mut storage = storage
        .write()
        .map_err(|e| format!("Storage lock error: {}", e))?;

    match statement {
        Statement::Insert(insert) => {
            let table_name = &insert.table;
            if !storage.has_table(table_name) {
                return Err(format!("Table '{}' not found", table_name));
            }

            let table_info = storage.get_table_info(table_name).ok();
            let num_columns = table_info
                .as_ref()
                .map(|i| i.columns.len())
                .unwrap_or(insert.values.first().map(|r| r.len()).unwrap_or(0));

            let records: Vec<Vec<sqlrustgo_storage::engine::Value>> = insert
                .values
                .iter()
                .map(|row| {
                    let mut new_row: Vec<sqlrustgo_storage::engine::Value> =
                        vec![sqlrustgo_storage::engine::Value::Null; num_columns];

                    if insert.columns.is_empty() {
                        for (col_idx, expr) in row.iter().enumerate() {
                            if col_idx < num_columns {
                                new_row[col_idx] = evaluate_literal_expr(expr);
                            }
                        }
                    } else {
                        for (value_idx, col_name) in insert.columns.iter().enumerate() {
                            if value_idx < row.len() {
                                if let Some(ref info) = table_info {
                                    if let Some(target_idx) = info
                                        .columns
                                        .iter()
                                        .position(|c| c.name.eq_ignore_ascii_case(col_name))
                                    {
                                        new_row[target_idx] =
                                            evaluate_literal_expr(&row[value_idx]);
                                    }
                                }
                            }
                        }
                    }
                    new_row
                })
                .collect();

            storage
                .insert(table_name, records)
                .map_err(|e| e.to_string())?;
            Ok(SqlExecResult {
                columns: vec![],
                rows: vec![],
                affected_rows: insert.values.len(),
            })
        }

        Statement::CreateTable(create) => {
            let columns: Vec<sqlrustgo_storage::engine::ColumnDefinition> = create
                .columns
                .iter()
                .map(|col| sqlrustgo_storage::engine::ColumnDefinition {
                    name: col.name.clone(),
                    data_type: col.data_type.clone(),
                    nullable: col.nullable,
                    is_unique: col.primary_key,
                    is_primary_key: col.primary_key,
                    references: None,
                    auto_increment: col.auto_increment,
                })
                .collect();

            let table_info = sqlrustgo_storage::engine::TableInfo {
                name: create.name.clone(),
                columns,
            };

            storage
                .create_table(&table_info)
                .map_err(|e| e.to_string())?;
            Ok(SqlExecResult {
                columns: vec![],
                rows: vec![],
                affected_rows: 0,
            })
        }

        Statement::Select(select) => {
            if !storage.has_table(&select.table) {
                return Err(format!("Table '{}' not found", select.table));
            }

            let table_info = storage.get_table_info(&select.table).ok();
            let columns = table_info
                .map(|info| info.columns.clone())
                .unwrap_or_default();

            let rows = storage.scan(&select.table).map_err(|e| e.to_string())?;

            // Apply WHERE clause filter if present
            let filtered_rows: Vec<Vec<sqlrustgo_storage::engine::Value>> =
                if let Some(ref where_clause) = select.where_clause {
                    rows.into_iter()
                        .filter(|row| evaluate_where_clause(where_clause, row, &columns))
                        .collect()
                } else {
                    rows
                };

            let column_names: Vec<String> = if select.columns.is_empty()
                || (select.columns.len() == 1 && select.columns[0].name == "*")
            {
                columns.iter().map(|c| c.name.clone()).collect()
            } else {
                select.columns.iter().map(|c| c.name.clone()).collect()
            };

            let result_rows: Vec<Vec<serde_json::Value>> = filtered_rows
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|v| serde_json::json!(value_to_json(v)))
                        .collect()
                })
                .collect();

            Ok(SqlExecResult {
                columns: column_names,
                rows: result_rows,
                affected_rows: 0,
            })
        }

        Statement::Delete(delete) => {
            if !storage.has_table(&delete.table) {
                return Err(format!("Table '{}' not found", delete.table));
            }

            let table_info = storage.get_table_info(&delete.table).ok();
            let columns = table_info
                .map(|info| info.columns.clone())
                .unwrap_or_default();

            let all_rows = storage.scan(&delete.table).unwrap_or_default();

            let rows_to_delete: Vec<&Vec<sqlrustgo_storage::engine::Value>> =
                if delete.where_clause.is_none() {
                    all_rows.iter().collect()
                } else {
                    all_rows
                        .iter()
                        .filter(|row| {
                            if let Some(ref where_clause) = delete.where_clause {
                                evaluate_where_clause(where_clause, row, &columns)
                            } else {
                                false
                            }
                        })
                        .collect()
                };

            let deleted_count = rows_to_delete.len();

            if deleted_count > 0 {
                let remaining_rows: Vec<Vec<sqlrustgo_storage::engine::Value>> = all_rows
                    .into_iter()
                    .filter(|row| {
                        if let Some(ref where_clause) = delete.where_clause {
                            !evaluate_where_clause(where_clause, row, &columns)
                        } else {
                            false
                        }
                    })
                    .collect();

                let _ = storage.delete(&delete.table, &[]);
                if !remaining_rows.is_empty() {
                    storage
                        .insert(&delete.table, remaining_rows)
                        .map_err(|e| e.to_string())?;
                }
            }

            Ok(SqlExecResult {
                columns: vec![],
                rows: vec![],
                affected_rows: deleted_count,
            })
        }

        Statement::Update(update) => {
            if !storage.has_table(&update.table) {
                return Err(format!("Table '{}' not found", update.table));
            }

            let table_info = storage.get_table_info(&update.table).ok();
            let columns = table_info
                .map(|info| info.columns.clone())
                .unwrap_or_default();

            let all_rows = storage.scan(&update.table).unwrap_or_default();

            let rows_to_update: Vec<(usize, Vec<sqlrustgo_storage::engine::Value>)> = all_rows
                .iter()
                .enumerate()
                .filter(|(_, row)| {
                    if let Some(ref where_clause) = update.where_clause {
                        evaluate_where_clause(where_clause, row, &columns)
                    } else {
                        true
                    }
                })
                .map(|(idx, row)| {
                    let mut new_row = row.clone();
                    for (col_name, expr) in &update.set_clauses {
                        if let Some(col_idx) = columns
                            .iter()
                            .position(|c| c.name.eq_ignore_ascii_case(col_name))
                        {
                            new_row[col_idx] = evaluate_expr(expr, &new_row, &columns);
                        }
                    }
                    (idx, new_row)
                })
                .collect();

            let updated_count = rows_to_update.len();

            if updated_count > 0 {
                let mut final_rows = all_rows;
                for (idx, new_row) in rows_to_update {
                    final_rows[idx] = new_row;
                }
                let _ = storage.delete(&update.table, &[]);
                storage
                    .insert(&update.table, final_rows)
                    .map_err(|e| e.to_string())?;
            }

            Ok(SqlExecResult {
                columns: vec![],
                rows: vec![],
                affected_rows: updated_count,
            })
        }

        Statement::Transaction(tx) => match tx.command {
            TransactionCommand::Begin => Ok(SqlExecResult {
                columns: vec![],
                rows: vec![],
                affected_rows: 0,
            }),
            TransactionCommand::Commit => Ok(SqlExecResult {
                columns: vec![],
                rows: vec![],
                affected_rows: 0,
            }),
            TransactionCommand::Rollback => Ok(SqlExecResult {
                columns: vec![],
                rows: vec![],
                affected_rows: 0,
            }),
            _ => Ok(SqlExecResult {
                columns: vec![],
                rows: vec![],
                affected_rows: 0,
            }),
        },

        Statement::DropTable(drop) => {
            if !storage.has_table(&drop.name) {
                return Err(format!("Table '{}' not found", drop.name));
            }
            storage.drop_table(&drop.name).map_err(|e| e.to_string())?;
            Ok(SqlExecResult {
                columns: vec![],
                rows: vec![],
                affected_rows: 0,
            })
        }

        _ => Err("Unsupported statement type".to_string()),
    }
}

/// Evaluate a literal expression to a Value
fn evaluate_literal_expr(expr: &Expression) -> sqlrustgo_storage::engine::Value {
    match expr {
        Expression::Literal(s) => {
            if let Ok(n) = s.parse::<i64>() {
                sqlrustgo_storage::engine::Value::Integer(n)
            } else if let Ok(n) = s.parse::<f64>() {
                sqlrustgo_storage::engine::Value::Float(n)
            } else if s.eq_ignore_ascii_case("true") {
                sqlrustgo_storage::engine::Value::Boolean(true)
            } else if s.eq_ignore_ascii_case("false") {
                sqlrustgo_storage::engine::Value::Boolean(false)
            } else {
                sqlrustgo_storage::engine::Value::Text(s.clone())
            }
        }
        _ => sqlrustgo_storage::engine::Value::Null,
    }
}

/// Evaluate an expression to a Value
fn evaluate_expr(
    expr: &Expression,
    row: &[sqlrustgo_storage::engine::Value],
    columns: &[sqlrustgo_storage::engine::ColumnDefinition],
) -> sqlrustgo_storage::engine::Value {
    match expr {
        Expression::Literal(s) => {
            if let Ok(n) = s.parse::<i64>() {
                sqlrustgo_storage::engine::Value::Integer(n)
            } else if let Ok(n) = s.parse::<f64>() {
                sqlrustgo_storage::engine::Value::Float(n)
            } else if s.eq_ignore_ascii_case("true") {
                sqlrustgo_storage::engine::Value::Boolean(true)
            } else if s.eq_ignore_ascii_case("false") {
                sqlrustgo_storage::engine::Value::Boolean(false)
            } else {
                sqlrustgo_storage::engine::Value::Text(s.clone())
            }
        }
        Expression::Identifier(name) => {
            if let Some(idx) = columns
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(name))
            {
                row.get(idx)
                    .cloned()
                    .unwrap_or(sqlrustgo_storage::engine::Value::Null)
            } else {
                sqlrustgo_storage::engine::Value::Null
            }
        }
        _ => sqlrustgo_storage::engine::Value::Null,
    }
}

/// Evaluate a WHERE clause expression against a row
fn evaluate_where_clause(
    expr: &Expression,
    row: &[sqlrustgo_storage::engine::Value],
    columns: &[sqlrustgo_storage::engine::ColumnDefinition],
) -> bool {
    match expr {
        Expression::BinaryOp(left, op, right) => {
            let left_val = evaluate_expr(left, row, columns);
            let right_val = evaluate_expr(right, row, columns);
            compare_values(&left_val, op, &right_val)
        }
        _ => true,
    }
}

/// Compare two values with the given operator
fn compare_values(
    left: &sqlrustgo_storage::engine::Value,
    op: &str,
    right: &sqlrustgo_storage::engine::Value,
) -> bool {
    match op {
        "=" | "==" | "EQ" => left == right,
        "!=" | "<>" | "NE" => left != right,
        ">" | "GT" => match (left, right) {
            (
                sqlrustgo_storage::engine::Value::Integer(l),
                sqlrustgo_storage::engine::Value::Integer(r),
            ) => l > r,
            (
                sqlrustgo_storage::engine::Value::Float(l),
                sqlrustgo_storage::engine::Value::Float(r),
            ) => l > r,
            (
                sqlrustgo_storage::engine::Value::Integer(l),
                sqlrustgo_storage::engine::Value::Float(r),
            ) => (*l as f64) > *r,
            (
                sqlrustgo_storage::engine::Value::Float(l),
                sqlrustgo_storage::engine::Value::Integer(r),
            ) => *l > (*r as f64),
            (
                sqlrustgo_storage::engine::Value::Text(l),
                sqlrustgo_storage::engine::Value::Text(r),
            ) => l > r,
            _ => false,
        },
        "<" | "LT" => match (left, right) {
            (
                sqlrustgo_storage::engine::Value::Integer(l),
                sqlrustgo_storage::engine::Value::Integer(r),
            ) => l < r,
            (
                sqlrustgo_storage::engine::Value::Float(l),
                sqlrustgo_storage::engine::Value::Float(r),
            ) => l < r,
            (
                sqlrustgo_storage::engine::Value::Integer(l),
                sqlrustgo_storage::engine::Value::Float(r),
            ) => (*l as f64) < *r,
            (
                sqlrustgo_storage::engine::Value::Float(l),
                sqlrustgo_storage::engine::Value::Integer(r),
            ) => *l < (*r as f64),
            (
                sqlrustgo_storage::engine::Value::Text(l),
                sqlrustgo_storage::engine::Value::Text(r),
            ) => l < r,
            _ => false,
        },
        ">=" | "GE" => match (left, right) {
            (
                sqlrustgo_storage::engine::Value::Integer(l),
                sqlrustgo_storage::engine::Value::Integer(r),
            ) => l >= r,
            (
                sqlrustgo_storage::engine::Value::Float(l),
                sqlrustgo_storage::engine::Value::Float(r),
            ) => l >= r,
            _ => false,
        },
        "<=" | "LE" => match (left, right) {
            (
                sqlrustgo_storage::engine::Value::Integer(l),
                sqlrustgo_storage::engine::Value::Integer(r),
            ) => l <= r,
            (
                sqlrustgo_storage::engine::Value::Float(l),
                sqlrustgo_storage::engine::Value::Float(r),
            ) => l <= r,
            _ => false,
        },
        _ => false,
    }
}

/// Result from SQL execution for JSON serialization
#[derive(Debug)]
struct SqlExecResult {
    columns: Vec<String>,
    rows: Vec<Vec<serde_json::Value>>,
    affected_rows: usize,
}

/// Convert Value to JSON-compatible type
fn value_to_json(value: Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Boolean(b) => serde_json::json!(b),
        Value::Integer(i) => serde_json::json!(i),
        Value::Float(f) => serde_json::json!(f),
        Value::Text(s) => serde_json::json!(s),
        Value::Date(d) => serde_json::json!(d),
        Value::Timestamp(ts) => serde_json::json!(ts),
        Value::Blob(b) => serde_json::json!(base64_encode(&b)),
    }
}

/// Simple base64 encoding for Blob values
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b = match chunk.len() {
            1 => [chunk[0], 0, 0],
            2 => [chunk[0], chunk[1], 0],
            _ => [chunk[0], chunk[1], chunk[2]],
        };
        result.push(ALPHABET[(b[0] >> 2) as usize] as char);
        result.push(ALPHABET[((b[0] & 0x03) << 4 | b[1] >> 4) as usize] as char);
        if chunk.len() > 1 {
            result.push(ALPHABET[((b[1] & 0x0f) << 2 | b[2] >> 6) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(ALPHABET[(b[2] & 0x3f) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Generate HTML for pipeline visualization index
#[allow(clippy::useless_format)]
fn generate_teaching_index_html() -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>SQLRustGo 2.0 - Teaching Enhanced</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background: #1a1a2e;
            color: #eee;
        }}
        h1 {{
            color: #00d4ff;
            border-bottom: 2px solid #00d4ff;
            padding-bottom: 10px;
        }}
        .feature {{
            background: #16213e;
            border-radius: 8px;
            padding: 20px;
            margin: 20px 0;
            border-left: 4px solid #00d4ff;
        }}
        .feature h2 {{
            color: #00d4ff;
            margin-top: 0;
        }}
        .feature a {{
            color: #00d4ff;
            text-decoration: none;
            padding: 8px 16px;
            background: #0f3460;
            border-radius: 4px;
            display: inline-block;
            margin: 5px;
        }}
        .feature a:hover {{
            background: #00d4ff;
            color: #1a1a2e;
        }}
        .badge {{
            background: #e94560;
            color: white;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 12px;
        }}
        .code {{
            background: #0f0f23;
            padding: 10px;
            border-radius: 4px;
            font-family: monospace;
            overflow-x: auto;
        }}
    </style>
</head>
<body>
    <h1>🎓 SQLRustGo 2.0 - Teaching Enhanced</h1>
    <p>Differentiating features that MySQL doesn't provide!</p>
    
    <div class="feature">
        <h2>📊 Visual Pipeline Execution</h2>
        <p>Visualize how your SQL query is executed as a pipeline of operators.</p>
        <a href="/teaching/pipeline">Pipeline Visualization</a>
        <a href="/teaching/pipeline/json">JSON API</a>
        <a href="/teaching/pipeline/latest">Latest Query</a>
    </div>
    
    <div class="feature">
        <h2>🔍 Vectorized Trace Log</h2>
        <p>Detailed execution traces showing vectorized batch processing.</p>
        <a href="/teaching/trace">Trace Log</a>
        <a href="/teaching/trace/json">JSON API</a>
    </div>
    
    <div class="feature">
        <h2>⚡ Operator Profiling</h2>
        <p>Per-operator performance profiling to understand query bottlenecks.</p>
        <a href="/teaching/profile">Profiling Report</a>
        <a href="/teaching/profile/json">JSON API</a>
        <a href="/teaching/profile/operators">Operator Stats</a>
    </div>
    
    <div class="feature">
        <h2>✨ Key Differentiators</h2>
        <ul>
            <li><strong>MySQL can't do this:</strong> Real-time pipeline visualization</li>
            <li><strong>MySQL can't do this:</strong> Vectorized execution tracing</li>
            <li><strong>MySQL can't do this:</strong> Operator-level profiling UI</li>
            <li>Perfect for teaching query optimization</li>
            <li>Great for learning how databases work</li>
        </ul>
    </div>
</body>
</html>"#
    )
}

/// Generate HTML for pipeline visualization
fn generate_pipeline_html() -> String {
    let traces = GLOBAL_TRACE_COLLECTOR.get_traces();

    let mut trace_list = String::new();
    for trace in traces.iter().take(20) {
        trace_list.push_str(&format!(
            r#"<tr>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{} rows</td>
                <td><a href="/teaching/pipeline/latest">View</a></td>
            </tr>"#,
            &trace.query_id[..8],
            truncate_string(&trace.sql, 40),
            format_duration(trace.total_duration_ns),
            trace.total_rows
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Pipeline Visualization - SQLRustGo 2.0</title>
    <style>
        body {{
            font-family: 'Courier New', monospace;
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
            background: #0d1117;
            color: #c9d1d9;
        }}
        h1 {{ color: #58a6ff; }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }}
        th, td {{
            padding: 10px;
            text-align: left;
            border-bottom: 1px solid #30363d;
        }}
        th {{ background: #161b22; color: #58a6ff; }}
        tr:hover {{ background: #161b22; }}
        a {{ color: #58a6ff; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
        .empty {{ color: #8b949e; font-style: italic; }}
        .home-link {{
            display: inline-block;
            margin-bottom: 20px;
            padding: 8px 16px;
            background: #21262d;
            border-radius: 4px;
        }}
    </style>
</head>
<body>
    <a href="/teaching" class="home-link">← Back to Teaching Enhanced</a>
    <h1>📊 Query Pipeline Visualization</h1>
    <p>These are the queries that have been executed with pipeline tracing enabled.</p>
    
    <table>
        <thead>
            <tr>
                <th>Query ID</th>
                <th>SQL</th>
                <th>Duration</th>
                <th>Rows</th>
                <th>Action</th>
            </tr>
        </thead>
        <tbody>
            {trace_list}
        </tbody>
    </table>
    
    {empty_msg}
</body>
</html>"#,
        trace_list = if trace_list.is_empty() {
            r#"<p class="empty">No queries executed yet. Run a query to see the pipeline visualization.</p>"#.to_string()
        } else {
            String::new()
        },
        empty_msg = ""
    )
}

/// Generate HTML for detailed pipeline view
fn generate_pipeline_detail_html(trace: &QueryTrace) -> String {
    let viz = trace.visualize_pipeline();
    let escaped_viz = viz.replace("<", "&lt;").replace(">", "&gt;");

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Pipeline Detail - SQLRustGo 2.0</title>
    <style>
        body {{
            font-family: 'Courier New', monospace;
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
            background: #0d1117;
            color: #c9d1d9;
        }}
        h1 {{ color: #58a6ff; }}
        .pipeline {{
            background: #161b22;
            padding: 20px;
            border-radius: 8px;
            overflow-x: auto;
            white-space: pre;
            font-size: 14px;
            line-height: 1.6;
        }}
        .query-info {{
            background: #21262d;
            padding: 15px;
            border-radius: 8px;
            margin-bottom: 20px;
        }}
        a {{ color: #58a6ff; text-decoration: none; }}
        .json-link {{
            display: inline-block;
            margin-top: 20px;
            padding: 8px 16px;
            background: #238636;
            border-radius: 4px;
        }}
    </style>
</head>
<body>
    <a href="/teaching/pipeline" style="color: #58a6ff;">← Back to Pipeline List</a>
    <h1>📊 Pipeline Execution Detail</h1>
    
    <div class="query-info">
        <p><strong>Query ID:</strong> {query_id}</p>
        <p><strong>SQL:</strong> {sql}</p>
        <p><strong>Duration:</strong> {duration}</p>
        <p><strong>Operators:</strong> {operators}</p>
        <p><strong>Total Rows:</strong> {total_rows}</p>
    </div>
    
    <h2>Execution Tree</h2>
    <div class="pipeline">
{visualization}
    </div>
    
    <a href="/teaching/pipeline/json" class="json-link">View as JSON</a>
</body>
</html>"#,
        query_id = trace.query_id,
        sql = trace.sql,
        duration = format_duration(trace.total_duration_ns),
        operators = trace.operator_count,
        total_rows = trace.total_rows,
        visualization = escaped_viz
    )
}

/// Generate HTML for trace log
fn generate_trace_html(traces: &[QueryTrace]) -> String {
    let mut trace_entries = String::new();
    for trace in traces.iter().take(50) {
        trace_entries.push_str(&format!(
            r#"<div class="trace-entry">
                <div class="trace-header">
                    <span class="trace-id">#{trace_id}</span>
                    <span class="trace-sql">{sql}</span>
                    <span class="trace-time">{duration}</span>
                </div>
            </div>"#,
            trace_id = &trace.query_id[..8],
            sql = truncate_string(&trace.sql, 60),
            duration = format_duration(trace.total_duration_ns)
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Vectorized Trace Log - SQLRustGo 2.0</title>
    <style>
        body {{
            font-family: 'Courier New', monospace;
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
            background: #0d1117;
            color: #c9d1d9;
        }}
        h1 {{ color: #a371f7; }}
        .trace-entry {{
            background: #161b22;
            margin: 10px 0;
            padding: 15px;
            border-radius: 6px;
            border-left: 3px solid #a371f7;
        }}
        .trace-header {{
            display: flex;
            justify-content: space-between;
        }}
        .trace-id {{
            color: #a371f7;
            font-weight: bold;
        }}
        .trace-sql {{
            color: #c9d1d9;
        }}
        .trace-time {{
            color: #8b949e;
        }}
        a {{ color: #58a6ff; text-decoration: none; }}
        .empty {{ color: #8b949e; font-style: italic; }}
    </style>
</head>
<body>
    <a href="/teaching" style="color: #58a6ff;">← Back to Teaching Enhanced</a>
    <h1>🔍 Vectorized Trace Log</h1>
    <p>Detailed execution traces showing vectorized batch processing.</p>
    
    {trace_entries}
    
    {empty_msg}
</body>
</html>"#,
        trace_entries = if trace_entries.is_empty() {
            r#"<p class="empty">No traces available. Execute a query to generate traces.</p>"#
                .to_string()
        } else {
            trace_entries
        },
        empty_msg = ""
    )
}

/// Generate HTML for profiling report
fn generate_profile_html(report: &str) -> String {
    let escaped_report = report.replace("<", "&lt;").replace(">", "&gt;");

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Operator Profiling - SQLRustGo 2.0</title>
    <style>
        body {{
            font-family: 'Courier New', monospace;
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
            background: #0d1117;
            color: #c9d1d9;
        }}
        h1 {{ color: #3fb950; }}
        .report {{
            background: #161b22;
            padding: 20px;
            border-radius: 8px;
            overflow-x: auto;
            white-space: pre;
            font-size: 13px;
            line-height: 1.5;
        }}
        a {{ color: #58a6ff; text-decoration: none; }}
    </style>
</head>
<body>
    <a href="/teaching" style="color: #58a6ff;">← Back to Teaching Enhanced</a>
    <h1>⚡ Operator Profiling Report</h1>
    <p>Per-operator performance profiling to identify bottlenecks.</p>
    
    <div class="report">
{report}
    </div>
    
    <a href="/teaching/profile/json" style="display: inline-block; margin-top: 20px; padding: 8px 16px; background: #238636; border-radius: 4px;">View as JSON</a>
</body>
</html>"#,
        report = escaped_report
    )
}

/// Generate HTML for individual operator stats
#[allow(clippy::format_in_format_args)]
fn generate_operators_html(profiles: &[OperatorProfile]) -> String {
    let mut operator_rows = String::new();
    for profile in profiles.iter().take(50) {
        operator_rows.push_str(&format!(
            r#"<tr>
                <td>{name}</td>
                <td>{execs}</td>
                <td>{avg}</td>
                <td>{total}</td>
                <td>{min}</td>
                <td>{max}</td>
                <td>{rows}</td>
                <td>{rps}</td>
            </tr>"#,
            name = profile.operator_name,
            execs = profile.execution_count,
            avg = profile.format_avg_time(),
            total = profile.format_total_time(),
            min = profile.format_min_time(),
            max = profile.format_max_time(),
            rows = profile.rows_processed,
            rps = format!("{:.0}", profile.rows_per_second)
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Operator Statistics - SQLRustGo 2.0</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
            background: #0d1117;
            color: #c9d1d9;
        }}
        h1 {{ color: #3fb950; }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }}
        th, td {{
            padding: 10px;
            text-align: left;
            border-bottom: 1px solid #30363d;
        }}
        th {{ background: #161b22; color: #3fb950; }}
        tr:hover {{ background: #161b22; }}
        a {{ color: #58a6ff; text-decoration: none; }}
        .empty {{ color: #8b949e; font-style: italic; }}
    </style>
</head>
<body>
    <a href="/teaching" style="color: #58a6ff;">← Back to Teaching Enhanced</a>
    <h1>⚡ Individual Operator Statistics</h1>
    
    <table>
        <thead>
            <tr>
                <th>Operator</th>
                <th>Executions</th>
                <th>Avg Time</th>
                <th>Total Time</th>
                <th>Min Time</th>
                <th>Max Time</th>
                <th>Rows</th>
                <th>Rows/sec</th>
            </tr>
        </thead>
        <tbody>
            {operator_rows}
        </tbody>
    </table>
    
    {empty_msg}
</body>
</html>"#,
        operator_rows = if operator_rows.is_empty() {
            r#"<p class="empty">No profiling data available. Execute some queries first.</p>"#
                .to_string()
        } else {
            operator_rows
        },
        empty_msg = ""
    )
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn format_duration(ns: u64) -> String {
    if ns < 1_000 {
        format!("{}ns", ns)
    } else if ns < 1_000_000 {
        format!("{:.1}µs", ns as f64 / 1_000.0)
    } else if ns < 1_000_000_000 {
        format!("{:.2}ms", ns as f64 / 1_000_000.0)
    } else {
        format!("{:.2}s", ns as f64 / 1_000_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teaching_endpoints_default() {
        let endpoints = TeachingEndpoints::default();
        assert!(endpoints.enable_pipeline_viz);
        assert!(endpoints.enable_profiling);
        assert!(endpoints.enable_trace);
        assert_eq!(endpoints.max_traces, 1000);
        assert_eq!(endpoints.max_profiles, 100);
    }

    #[test]
    fn test_teaching_endpoints_custom() {
        let endpoints = TeachingEndpoints {
            enable_pipeline_viz: false,
            enable_profiling: true,
            enable_trace: true,
            max_traces: 100,
            max_profiles: 50,
        };
        assert!(!endpoints.enable_pipeline_viz);
        assert!(endpoints.enable_profiling);
        assert!(endpoints.enable_trace);
        assert_eq!(endpoints.max_traces, 100);
        assert_eq!(endpoints.max_profiles, 50);
    }

    #[test]
    fn test_teaching_endpoints_clone() {
        let endpoints = TeachingEndpoints::default();
        let cloned = endpoints.clone();
        assert_eq!(cloned.enable_pipeline_viz, endpoints.enable_pipeline_viz);
        assert_eq!(cloned.enable_profiling, endpoints.enable_profiling);
        assert_eq!(cloned.enable_trace, endpoints.enable_trace);
        assert_eq!(cloned.max_traces, endpoints.max_traces);
        assert_eq!(cloned.max_profiles, endpoints.max_profiles);
    }

    #[test]
    fn test_teaching_http_server_creation() {
        let server = TeachingHttpServer::new("127.0.0.1", 8080);
        assert_eq!(server.port, 8080);
        assert_eq!(server.host, "127.0.0.1");
    }

    #[test]
    fn test_teaching_http_server_with_endpoints() {
        let endpoints = TeachingEndpoints {
            enable_pipeline_viz: false,
            enable_profiling: true,
            enable_trace: true,
            max_traces: 100,
            max_profiles: 50,
        };
        let server = TeachingHttpServer::new("127.0.0.1", 8080).with_teaching_endpoints(endpoints);
        assert_eq!(server.port, 8080);
    }

    #[test]
    fn test_teaching_http_server_with_version() {
        let server = TeachingHttpServer::new("127.0.0.1", 8080).with_version("1.5.0");
        assert_eq!(server.get_version(), "1.5.0");
    }

    #[test]
    fn test_teaching_http_server_get_port() {
        let server = TeachingHttpServer::new("127.0.0.1", 8080);
        assert_eq!(server.get_port(), 8080);
    }

    #[test]
    fn test_teaching_http_server_bind_to_available_port() {
        let server = TeachingHttpServer::new("127.0.0.1", 0);
        let port = server.bind_to_available_port();
        assert!(port > 0, "Port should be > 0 when binding to port 0");
    }

    #[test]
    fn test_teaching_http_server_bind_preserves_configured_port() {
        let server = TeachingHttpServer::new("127.0.0.1", 8080);
        let port = server.bind_to_available_port();
        assert_eq!(port, 8080, "Non-zero port should not change");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello...");
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("", 5), "");
    }

    #[test]
    fn test_truncate_string_exact_length() {
        assert_eq!(truncate_string("hello", 5), "hello");
    }

    #[test]
    fn test_format_duration() {
        assert!(format_duration(500).contains("ns"));
        assert!(format_duration(50_000).contains("µs"));
        assert!(format_duration(5_000_000).contains("ms"));
    }

    #[test]
    fn test_format_duration_seconds() {
        let result = format_duration(5_000_000_000);
        assert!(
            result.contains("s"),
            "Should contain 's' for seconds: {}",
            result
        );
    }

    #[test]
    fn test_teaching_http_server_builder_pattern() {
        let server = TeachingHttpServer::new("0.0.0.0", 9090)
            .with_version("2.0.0")
            .with_teaching_endpoints(TeachingEndpoints {
                enable_pipeline_viz: true,
                enable_profiling: false,
                enable_trace: true,
                max_traces: 500,
                max_profiles: 200,
            });

        assert_eq!(server.get_version(), "2.0.0");
        assert_eq!(server.get_port(), 9090);
    }
}
