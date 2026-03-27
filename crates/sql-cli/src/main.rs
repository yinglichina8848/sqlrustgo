//! SQLRustGo Interactive CLI
//!
//! A MySQL-compatible interactive SQL shell

use rustyline::history::FileHistory;
use rustyline::Editor;
use sqlrustgo_executor::ExecutorResult;
use sqlrustgo_parser::parser::{
    CreateTableStatement, DropTableStatement, Expression, InsertStatement, SelectStatement,
};
use sqlrustgo_parser::{parse, Statement};
use sqlrustgo_storage::{ColumnDefinition, MemoryStorage, StorageEngine, TableInfo};
use sqlrustgo_types::Value;
use std::env;

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

        // Initialize storage engine
        let mut storage = MemoryStorage::new();

        // Execute the SQL
        match execute_sql(&sql, &mut storage) {
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

    // Initialize storage engine
    let mut storage = MemoryStorage::new();

    // Pre-populate with some sample data for testing
    setup_sample_data(&mut storage);

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
                    match execute_command(&trimmed, &storage) {
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
                        CommandResult::Sql(result) => {
                            // This was actually a SQL query, not a command
                            match result {
                                Ok(rows) => {
                                    print_result(rows);
                                }
                                Err(e) => {
                                    println!("Error: {}", e);
                                }
                            }
                        }
                    }
                } else {
                    // Execute SQL query
                    match execute_sql(&trimmed, &mut storage) {
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
fn execute_sql(sql: &str, storage: &mut dyn StorageEngine) -> Result<ExecutorResult, String> {
    // Parse the SQL statement
    let statement = parse(sql).map_err(|e| format!("Parse error: {:?}", e))?;

    // For now, implement simple execution for SELECT queries
    match statement {
        Statement::Select(select) => execute_select(&select, storage),
        Statement::Insert(insert) => execute_insert(&insert, storage),
        Statement::CreateTable(create) => execute_create_table(&create, storage),
        Statement::DropTable(drop) => execute_drop_table(&drop, storage),
        Statement::ShowStatus => execute_show_status(storage),
        Statement::ShowProcesslist => execute_show_processlist(storage),
        _ => Err("Only SELECT, INSERT, CREATE TABLE, DROP TABLE, SHOW STATUS, SHOW PROCESSLIST are supported".to_string()),
    }
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
            references: None,
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
fn execute_show_processlist(_storage: &dyn StorageEngine) -> Result<ExecutorResult, String> {
    let mut rows = Vec::new();

    // Currently no active connections in single-user mode
    // This would be expanded in multi-threaded server mode
    rows.push(vec![
        Value::Text("1".to_string()),                // Id
        Value::Text("system".to_string()),           // User
        Value::Text("localhost".to_string()),        // Host
        Value::Text("".to_string()),                 // DB
        Value::Text("Query".to_string()),            // Command
        Value::Text("0".to_string()),                // Time
        Value::Text("".to_string()),                 // State
        Value::Text("SHOW PROCESSLIST".to_string()), // Info
    ]);

    Ok(ExecutorResult::new(rows, 0))
}

/// Setup sample data for testing CLI commands
fn setup_sample_data(storage: &mut dyn StorageEngine) {
    // Create users table
    let users_info = TableInfo {
        name: "users".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                references: None,
            },
            ColumnDefinition {
                name: "name".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                is_unique: false,
                references: None,
            },
            ColumnDefinition {
                name: "email".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                is_unique: false,
                references: None,
            },
        ],
    };

    // Create table (ignore error if already exists)
    let _ = storage.create_table(&users_info);

    // Insert sample data
    let user_records = vec![
        vec![
            Value::Integer(1),
            Value::Text("Alice".to_string()),
            Value::Text("alice@example.com".to_string()),
        ],
        vec![
            Value::Integer(2),
            Value::Text("Bob".to_string()),
            Value::Text("bob@example.com".to_string()),
        ],
        vec![
            Value::Integer(3),
            Value::Text("Charlie".to_string()),
            Value::Null,
        ],
    ];
    let _ = storage.insert("users", user_records);

    // Create orders table
    let orders_info = TableInfo {
        name: "orders".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                references: None,
            },
            ColumnDefinition {
                name: "user_id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                references: None,
            },
            ColumnDefinition {
                name: "amount".to_string(),
                data_type: "REAL".to_string(),
                nullable: false,
                is_unique: false,
                references: None,
            },
        ],
    };

    let _ = storage.create_table(&orders_info);

    let order_records = vec![
        vec![Value::Integer(1), Value::Integer(1), Value::Float(100.5)],
        vec![Value::Integer(2), Value::Integer(1), Value::Float(250.0)],
        vec![Value::Integer(3), Value::Integer(2), Value::Float(75.25)],
    ];
    let _ = storage.insert("orders", order_records);
}
