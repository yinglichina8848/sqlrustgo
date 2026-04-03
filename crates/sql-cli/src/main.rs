//! SQLRustGo Interactive CLI
//!
//! A MySQL-compatible interactive SQL shell

use rustyline::history::FileHistory;
use rustyline::Editor;
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use sqlrustgo_executor::ExecutorResult;
use sqlrustgo_parser::parser::{
    CreateTableStatement, DropTableStatement, Expression, InsertStatement, KillStatement, KillType,
    SelectStatement,
};
use sqlrustgo_parser::Statement;
use sqlrustgo_security::SessionManager;
use sqlrustgo_storage::{ColumnDefinition, StorageEngine, TableInfo};
use sqlrustgo_types::Value;
use std::env;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

/// Check if teaching mode is enabled via environment variable
fn is_teaching_mode() -> bool {
    env::var("SQLRUSTGO_TEACHING_MODE")
        .map(|v| v == "1")
        .unwrap_or(false)
}

mod commands;
use commands::{execute_command, print_result, CommandResult};

/// SQL CLI main entry point
fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Check for command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        // Command line mode - execute single SQL and exit
        let sql = args[1..].join(" ");

        // Initialize execution engine
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let mut engine = ExecutionEngine::new(storage);

        // Initialize session manager for CLI
        let session_manager = Arc::new(SessionManager::new());
        let cli_session_id =
            session_manager.create_session("sqlrustgo".to_string(), "localhost".to_string());

        // Execute the SQL
        match execute_sql(&sql, &mut engine, &session_manager, cli_session_id) {
            Ok(result) => {
                print_result(result);
            }
            Err(e) => {
                println!("Error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    println!("SQLRustGo Interactive SQL Shell");
    println!("Type '.help' for available commands, '.exit' to quit\n");

    // Initialize execution engine
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage.clone());

    // Initialize session manager and create CLI session
    let session_manager = Arc::new(SessionManager::new());
    let cli_session_id =
        session_manager.create_session("sqlrustgo".to_string(), "localhost".to_string());
    log::info!("CLI session started: id={}", cli_session_id);

    // Pre-populate with some sample data for testing
    setup_sample_data(&mut engine);

    // Create REPL editor
    let mut rl = Editor::<(), FileHistory>::new().expect("Failed to create editor");
    let _ = rl.load_history(".sql_history");

    // Main REPL loop
    loop {
        let readline = rl.readline("sqlrustgo> ");
        match readline {
            Ok(line) => {
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(&trimmed);

                // Handle meta-commands (starting with dot)
                if trimmed.starts_with('.') {
                    let storage_ref: &dyn StorageEngine = &*storage.read().unwrap();
                    match execute_command(&trimmed, storage_ref) {
                        CommandResult::Ok(output) => {
                            if !output.is_empty() {
                                println!("{}", output);
                            }
                        }
                        CommandResult::Err(msg) => {
                            println!("Error: {}", msg);
                        }
                        CommandResult::Exit => {
                            println!("Goodbye!");
                            break;
                        }
                        CommandResult::Sql(_) => {
                            unreachable!("execute_command should not return Sql variant");
                        }
                    }
                } else {
                    // Execute SQL query
                    match execute_sql(&trimmed, &mut engine, &session_manager, cli_session_id) {
                        Ok(result) => {
                            print_result(result);
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("^C");
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    let _ = rl.save_history(".sql_history");
}

/// Execute SQL query
fn execute_sql(
    sql: &str,
    engine: &mut ExecutionEngine,
    _session_manager: &SessionManager,
    _current_session_id: u64,
) -> Result<ExecutorResult, String> {
    let statement = parse(sql).map_err(|e| format!("Parse error: {:?}", e))?;
    engine
        .execute(statement)
        .map_err(|e: sqlrustgo_types::SqlError| e.to_string())
}

/// Execute SELECT query
fn execute_select(
    select: &SelectStatement,
    storage: &dyn StorageEngine,
) -> Result<ExecutorResult, String> {
    let table_name = &select.table;

    // Check if table exists
    if !storage.has_table(table_name) {
        return Err(format!("Table '{}' not found", table_name));
    }

    // In teaching mode, return EXPLAIN output instead of actual data
    if is_teaching_mode() {
        return generate_explain_output(select, storage);
    }

    // Get table info for schema
    let table_info = storage
        .get_table_info(table_name)
        .map_err(|e| e.to_string())?;

    // Scan all rows
    let mut rows = storage.scan(table_name).map_err(|e| e.to_string())?;

    // Apply OFFSET (skip first N rows)
    if let Some(offset) = select.offset {
        let offset = offset.min(rows.len());
        rows = rows.into_iter().skip(offset).collect();
    }

    // Apply LIMIT (take first N rows)
    if let Some(limit) = select.limit {
        let limit = limit.min(rows.len());
        rows = rows.into_iter().take(limit).collect();
    }

    // If there are columns specified, filter them
    let result_rows: Vec<Vec<Value>> = if select.columns.is_empty() {
        rows
    } else {
        // Map column names to indices
        let col_indices: Vec<usize> = select
            .columns
            .iter()
            .filter_map(|col| table_info.columns.iter().position(|c| c.name == col.name))
            .collect();

        rows.into_iter()
            .map(|row| {
                col_indices
                    .iter()
                    .map(|&i| row.get(i).cloned().unwrap_or(Value::Null))
                    .collect()
            })
            .collect()
    };

    Ok(ExecutorResult::new(result_rows, 0))
}

/// Generate EXPLAIN output for teaching mode
fn generate_explain_output(
    select: &SelectStatement,
    storage: &dyn StorageEngine,
) -> Result<ExecutorResult, String> {
    let table_name = &select.table;

    // Get table info for schema
    let table_info = storage
        .get_table_info(table_name)
        .map_err(|e| e.to_string())?;

    // Build EXPLAIN output
    let mut explain_output = String::new();

    explain_output.push_str("+-----------------------------+\n");
    explain_output.push_str("| ID | Operation              |\n");
    explain_output.push_str("+-----------------------------+\n");

    // Generate plan based on query structure
    if select.columns.is_empty() {
        explain_output.push_str(&format!("| 1  | SeqScan on {}        |\n", table_name));
    } else {
        explain_output.push_str(&format!("| 1  | SeqScan on {}        |\n", table_name));
        let cols_str = if select.columns.len() > 3 {
            format!("{} columns...", select.columns.len())
        } else {
            select
                .columns
                .iter()
                .map(|c| c.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        };
        explain_output.push_str(&format!("| 2  | Projection: {} |\n", cols_str));
    }

    // Check WHERE clause presence
    if select.where_clause.is_some() {
        explain_output.push_str("| 3  | Filter                |\n");
    }

    // Add aggregates info if present
    if !select.aggregates.is_empty() {
        explain_output.push_str(&format!(
            "| {}  | Aggregate              |\n",
            if select.where_clause.is_some() { 4 } else { 3 }
        ));
    }

    explain_output.push_str("+-----------------------------+\n");

    // Add table structure info
    explain_output.push_str("\nTable structure:\n");
    explain_output.push_str("+-------------+---------------+\n");
    explain_output.push_str("| Column      | Type          |\n");
    explain_output.push_str("+-------------+---------------+\n");

    for col in &table_info.columns {
        let col_type = format!("{:?}", col.data_type);
        explain_output.push_str(&format!(
            "| {:11} | {:13} |\n",
            if col.name.len() > 11 {
                &col.name[..11]
            } else {
                &col.name
            },
            if col_type.len() > 13 {
                &col_type[..13]
            } else {
                &col_type
            }
        ));
    }

    explain_output.push_str("+-------------+---------------+\n");

    // Add note about teaching mode
    explain_output
        .push_str("\nNote: Teaching mode is enabled. Set SQLRUSTGO_TEACHING_MODE=0 to disable.\n");

    Ok(ExecutorResult::new(
        vec![vec![Value::Text(explain_output)]],
        0,
    ))
}

/// Execute INSERT statement
fn execute_insert(
    insert: &InsertStatement,
    storage: &mut dyn StorageEngine,
) -> Result<ExecutorResult, String> {
    let table_name = &insert.table;

    if !storage.has_table(table_name) {
        return Err(format!("Table '{}' not found", table_name));
    }

    // Convert values to records
    let records: Vec<Vec<Value>> = insert
        .values
        .iter()
        .map(|row| {
            row.iter()
                .map(|expr| match expr {
                    Expression::Literal(value) => sqlrustgo_types::Value::Text(value.clone()),
                    _ => Value::Null,
                })
                .collect()
        })
        .collect();

    storage
        .insert(table_name, records)
        .map_err(|e| e.to_string())?;

    Ok(ExecutorResult::new(vec![], insert.values.len()))
}

/// Execute CREATE TABLE statement
fn execute_create_table(
    create: &CreateTableStatement,
    storage: &mut dyn StorageEngine,
) -> Result<ExecutorResult, String> {
    let columns: Vec<ColumnDefinition> = create
        .columns
        .iter()
        .map(|col| ColumnDefinition {
            name: col.name.clone(),
            data_type: col.data_type.clone(),
            nullable: col.nullable,
            is_unique: false,
            is_primary_key: false,
            references: None,
            auto_increment: false,
        })
        .collect();

    let table_info = TableInfo {
        name: create.name.clone(),
        columns,
    };

    storage
        .create_table(&table_info)
        .map_err(|e| e.to_string())?;

    Ok(ExecutorResult::new(vec![], 0))
}

/// Execute DROP TABLE statement
fn execute_drop_table(
    drop: &DropTableStatement,
    storage: &mut dyn StorageEngine,
) -> Result<ExecutorResult, String> {
    storage.drop_table(&drop.name).map_err(|e| e.to_string())?;

    Ok(ExecutorResult::new(vec![], 0))
}

/// Execute SHOW STATUS statement
fn execute_show_status(storage: &dyn StorageEngine) -> Result<ExecutorResult, String> {
    let mut rows = Vec::new();

    // System metrics
    rows.push(vec![
        Value::Text("uptime".to_string()),
        Value::Text(format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        )),
    ]);

    // Table count
    let table_count = storage.list_tables().len();
    rows.push(vec![
        Value::Text("table_count".to_string()),
        Value::Text(table_count.to_string()),
    ]);

    // Database version
    rows.push(vec![
        Value::Text("version".to_string()),
        Value::Text("1.9.0".to_string()),
    ]);

    // Storage path (mock)
    rows.push(vec![
        Value::Text("datadir".to_string()),
        Value::Text("./data".to_string()),
    ]);

    Ok(ExecutorResult::new(rows, 0))
}

/// Execute SHOW PROCESSLIST statement
fn execute_show_processlist(
    session_manager: &SessionManager,
    current_session_id: u64,
) -> Result<ExecutorResult, String> {
    let current_session = session_manager
        .get_session(current_session_id)
        .ok_or_else(|| "Not in a valid session".to_string())?;

    if !current_session.can_view_processlist() {
        return Err(
            "Access denied: you need PROCESS or SUPER privilege to view processlist".to_string(),
        );
    }

    let mut rows = Vec::new();

    // Get all active sessions from SessionManager
    let sessions = session_manager.get_active_sessions();

    if sessions.is_empty() {
        rows.push(vec![
            Value::Text("0".to_string()),                // Id
            Value::Text("system".to_string()),           // User
            Value::Text("localhost".to_string()),        // Host
            Value::Text("".to_string()),                 // DB
            Value::Text("Daemon".to_string()),           // Command
            Value::Text("0".to_string()),                // Time
            Value::Text("".to_string()),                 // State
            Value::Text("SHOW PROCESSLIST".to_string()), // Info
        ]);
    } else {
        for session in sessions {
            let command = match session.status {
                sqlrustgo_security::SessionStatus::Active => "Query",
                sqlrustgo_security::SessionStatus::Idle => "Sleep",
                sqlrustgo_security::SessionStatus::Closing => "Closing",
                sqlrustgo_security::SessionStatus::Closed => "Dead",
            };

            let time = session.idle_time_seconds().to_string();
            let db = session.database.clone().unwrap_or_default();

            rows.push(vec![
                Value::Text(session.id.to_string()), // Id
                Value::Text(session.user.clone()),   // User
                Value::Text(session.ip.clone()),     // Host
                Value::Text(db),                     // DB
                Value::Text(command.to_string()),    // Command
                Value::Text(time),                   // Time
                Value::Text("".to_string()),         // State
                Value::Text("".to_string()),         // Info
            ]);
        }
    }

    Ok(ExecutorResult::new(rows, 0))
}

/// Execute SELECT * FROM information_schema.processlist
fn execute_information_schema_processlist(
    session_manager: &SessionManager,
    current_session_id: u64,
) -> Result<ExecutorResult, String> {
    let current_session = session_manager
        .get_session(current_session_id)
        .ok_or_else(|| "Not in a valid session".to_string())?;

    if !current_session.can_view_processlist() {
        return Err(
            "Access denied: you need PROCESS or SUPER privilege to view processlist".to_string(),
        );
    }

    let mut rows = Vec::new();

    let sessions = session_manager.get_active_sessions();

    if sessions.is_empty() {
        rows.push(vec![
            Value::Text("0".to_string()),
            Value::Text("system".to_string()),
            Value::Text("localhost".to_string()),
            Value::Text("".to_string()),
            Value::Text("Daemon".to_string()),
            Value::Text("0".to_string()),
            Value::Text("".to_string()),
            Value::Text("".to_string()),
        ]);
    } else {
        for session in sessions {
            let command = match session.status {
                sqlrustgo_security::SessionStatus::Active => "Query",
                sqlrustgo_security::SessionStatus::Idle => "Sleep",
                sqlrustgo_security::SessionStatus::Closing => "Closing",
                sqlrustgo_security::SessionStatus::Closed => "Dead",
            };

            let time = session.idle_time_seconds().to_string();
            let db = session.database.clone().unwrap_or_default();

            rows.push(vec![
                Value::Text(session.id.to_string()),
                Value::Text(session.user.clone()),
                Value::Text(session.ip.clone()),
                Value::Text(db),
                Value::Text(command.to_string()),
                Value::Text(time),
                Value::Text("".to_string()),
                Value::Text("".to_string()),
            ]);
        }
    }

    Ok(ExecutorResult::new(rows, 0))
}

/// Execute KILL statement
fn execute_kill(
    kill: &KillStatement,
    session_manager: &SessionManager,
    current_session_id: u64,
) -> Result<ExecutorResult, String> {
    let kill_type_str = match kill.kill_type {
        KillType::Connection => "CONNECTION",
        KillType::Query => "QUERY",
    };

    let target_session_id = kill.process_id;

    // Get current session for privilege check
    let current_session = session_manager
        .get_session(current_session_id)
        .ok_or_else(|| "Not in a valid session".to_string())?;

    // Cannot kill self
    if target_session_id == current_session_id {
        return Err(format!(
            "Cannot KILL {} {} (cannot kill self session)",
            kill_type_str, target_session_id
        ));
    }

    // Check if target session exists
    let target_session = session_manager.get_session(target_session_id);
    if target_session.is_none() {
        return Err(format!("Unknown thread id: {}", target_session_id));
    }

    let target_session = target_session.unwrap();

    // Permission check: can kill if current user has SUPER privilege
    // or if killing own session (same user)
    let is_own_session = target_session.user == current_session.user;
    if !is_own_session && !current_session.can_kill() {
        return Err(format!(
            "Access denied: cannot KILL {} {} (need SUPER privilege to kill other user's sessions)",
            kill_type_str, target_session_id
        ));
    }

    // Perform the kill based on type
    match kill.kill_type {
        KillType::Connection => {
            // Close the entire connection
            log::info!(
                "KILL CONNECTION {} - closing session (user: {}, ip: {}) by {}",
                target_session_id,
                target_session.user,
                target_session.ip,
                current_session.user
            );
            session_manager.close_session(target_session_id);
        }
        KillType::Query => {
            // Just interrupt the query, keep connection alive
            log::info!(
                "KILL QUERY {} - interrupting query in session (user: {}, ip: {}) by {}",
                target_session_id,
                target_session.user,
                target_session.ip,
                current_session.user
            );
            // In a real implementation, we would signal the query to interrupt
            // For now, we just log it
        }
    }

    Ok(ExecutorResult::new(
        vec![vec![Value::Text(format!(
            "{} {} {}",
            kill_type_str, target_session_id, "executed"
        ))]],
        0,
    ))
}

/// Setup sample data for testing CLI commands
fn setup_sample_data(engine: &mut ExecutionEngine) {
    let sample_sqls = vec![
        "CREATE TABLE IF NOT EXISTS users (id INTEGER, name TEXT, email TEXT)",
        "INSERT INTO users VALUES (1, 'Alice', 'alice@example.com')",
        "INSERT INTO users VALUES (2, 'Bob', 'bob@example.com')",
        "INSERT INTO users VALUES (3, 'Charlie', NULL)",
        "CREATE TABLE IF NOT EXISTS orders (id INTEGER, user_id INTEGER, amount REAL)",
        "INSERT INTO orders VALUES (1, 1, 100.5)",
        "INSERT INTO orders VALUES (2, 1, 250.0)",
        "INSERT INTO orders VALUES (3, 2, 75.25)",
    ];

    for sql in sample_sqls {
        let stmt = parse(sql);
        if let Ok(stmt) = stmt {
            let _ = engine.execute(stmt);
        }
    }
}
