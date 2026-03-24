//! CLI Commands Module
//!
//! Implements MySQL-style meta commands

use sqlrustgo_executor::ExecutorResult;
use sqlrustgo_storage::{StorageEngine, TableInfo};
use sqlrustgo_types::SqlError;

/// Command execution result
#[derive(Debug)]
pub enum CommandResult {
    Ok(String),
    Err(String),
    Exit,
    Sql(Result<ExecutorResult, SqlError>),
}

/// Execute a meta-command (starting with dot)
pub fn execute_command(cmd: &str, storage: &dyn StorageEngine) -> CommandResult {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let cmd_name = parts.first().unwrap_or(&"");

    match *cmd_name {
        ".help" => CommandResult::Ok(get_help()),
        ".tables" => execute_tables(storage),
        ".schema" => {
            if parts.len() < 2 {
                CommandResult::Err("Usage: .schema <table_name>".to_string())
            } else {
                execute_schema(parts[1], storage)
            }
        }
        ".indexes" => {
            if parts.len() < 2 {
                CommandResult::Err("Usage: .indexes <table_name>".to_string())
            } else {
                execute_indexes(parts[1], storage)
            }
        }
        ".exit" | ".quit" => CommandResult::Exit,
        _ => CommandResult::Err(format!(
            "Unknown command: {}. Type .help for available commands.",
            cmd_name
        )),
    }
}

/// Get help text
fn get_help() -> String {
    r#"SQLRustGo CLI Commands:
  .tables           List all tables
  .schema <table>   Show schema for a table
  .indexes <table>  Show indexes for a table
  .help             Show this help message
  .exit, .quit      Exit the CLI

SQL Commands:
  SELECT, INSERT, CREATE TABLE, DROP TABLE are supported
"#
    .to_string()
}

/// Execute .tables command - list all tables
fn execute_tables(storage: &dyn StorageEngine) -> CommandResult {
    let tables = storage.list_tables();

    if tables.is_empty() {
        CommandResult::Ok("No tables found".to_string())
    } else {
        let output = tables
            .iter()
            .map(|t| format!("  {}", t))
            .collect::<Vec<_>>()
            .join("\n");
        CommandResult::Ok(format!("Tables:\n{}", output))
    }
}

/// Execute .schema command - show table schema
fn execute_schema(table_name: &str, storage: &dyn StorageEngine) -> CommandResult {
    match storage.get_table_info(table_name) {
        Ok(info) => {
            let output = format_table_schema(&info);
            CommandResult::Ok(output)
        }
        Err(e) => CommandResult::Err(format!("Error: {}", e)),
    }
}

/// Execute .indexes command - show table indexes
fn execute_indexes(table_name: &str, storage: &dyn StorageEngine) -> CommandResult {
    // Check if table exists
    if !storage.has_table(table_name) {
        return CommandResult::Err(format!("Table '{}' not found", table_name));
    }

    // Try to get index information from storage
    // For now, we'll use a simple approach that checks if the storage supports it
    let indexes = get_table_indexes(storage, table_name);

    if indexes.is_empty() {
        CommandResult::Ok(format!("No indexes found for table '{}'", table_name))
    } else {
        let output = format_table_indexes(table_name, &indexes);
        CommandResult::Ok(output)
    }
}

/// Get table indexes from storage
/// This is a workaround since StorageEngine doesn't have a list_indexes method
fn get_table_indexes(storage: &dyn StorageEngine, table_name: &str) -> Vec<IndexInfo> {
    // Try to get table info first
    if let Ok(info) = storage.get_table_info(table_name) {
        // For each column, try to see if there's an index
        // Since MemoryStorage doesn't really track indexes, we'll return empty
        // But this can be extended in the future
        let mut indexes = Vec::new();

        // Check if we can find any index metadata
        // For now, return empty - this can be enhanced when StorageEngine is extended
        for (i, col) in info.columns.iter().enumerate() {
            // Try search_index to see if column has an index
            if let Some(_row_id) = storage.search_index(table_name, &col.name, 0) {
                indexes.push(IndexInfo {
                    name: format!("idx_{}_{}", table_name, col.name),
                    column: col.name.clone(),
                    column_index: i,
                    is_unique: false,
                });
            }
        }

        indexes
    } else {
        Vec::new()
    }
}

/// Index information
#[derive(Debug, Clone)]
struct IndexInfo {
    name: String,
    column: String,
    column_index: usize,
    is_unique: bool,
}

/// Format table schema for display
fn format_table_schema(info: &TableInfo) -> String {
    let mut output = format!("Table: {}\n", info.name);
    output.push_str("Columns:\n");

    for col in &info.columns {
        let nullable_str = if col.nullable { "YES" } else { "NO" };
        output.push_str(&format!(
            "  {:20} {:15} Nullable: {}\n",
            col.name, col.data_type, nullable_str
        ));
    }

    output
}

/// Format table indexes for display
fn format_table_indexes(table_name: &str, indexes: &[IndexInfo]) -> String {
    let mut output = format!("Indexes for {}:\n", table_name);

    if indexes.is_empty() {
        output.push_str("  (none)\n");
    } else {
        output.push_str("+---------------------------+-------------+----------+\n");
        output.push_str("| Key Name                  | Column      | Unique   |\n");
        output.push_str("+---------------------------+-------------+----------+\n");

        for idx in indexes {
            let unique_str = if idx.is_unique { "YES" } else { "NO" };
            output.push_str(&format!(
                "| {:25} | {:11} | {:8} |\n",
                idx.name, idx.column, unique_str
            ));
        }
        output.push_str("+---------------------------+-------------+----------+\n");
    }

    output
}

/// Print SQL query result
pub fn print_result(result: ExecutorResult) {
    if result.rows.is_empty() {
        if result.affected_rows > 0 {
            println!("Query OK, {} rows affected", result.affected_rows);
        } else {
            println!("Empty set");
        }
        return;
    }

    // Print rows
    for row in &result.rows {
        let row_str: Vec<String> = row.iter().map(|v| format_value(v)).collect();
        println!("{}", row_str.join("\t"));
    }

    println!("\n{} rows in set", result.rows.len());
}

/// Format a value for display
fn format_value(value: &sqlrustgo_types::Value) -> String {
    match value {
        sqlrustgo_types::Value::Null => "NULL".to_string(),
        sqlrustgo_types::Value::Integer(i) => i.to_string(),
        sqlrustgo_types::Value::Float(f) => f.to_string(),
        sqlrustgo_types::Value::Text(s) => s.clone(),
        sqlrustgo_types::Value::Boolean(b) => b.to_string(),
        sqlrustgo_types::Value::Blob(b) => format!("[BLOB: {} bytes]", b.len()),
        sqlrustgo_types::Value::Date(d) => d.to_string(),
        sqlrustgo_types::Value::Timestamp(t) => t.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_storage::{ColumnDefinition, MemoryStorage, TableInfo};

    #[test]
    fn test_command_help() {
        let storage = MemoryStorage::new();
        let result = execute_command(".help", &storage);

        match result {
            CommandResult::Ok(output) => {
                assert!(output.contains(".tables"));
                assert!(output.contains(".schema"));
                assert!(output.contains(".indexes"));
            }
            _ => panic!("Expected Ok"),
        }
    }

    #[test]
    fn test_command_tables_empty() {
        let storage = MemoryStorage::new();
        let result = execute_command(".tables", &storage);

        match result {
            CommandResult::Ok(output) => {
                assert!(output.contains("No tables found"));
            }
            _ => panic!("Expected Ok"),
        }
    }

    #[test]
    fn test_command_tables_with_data() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
            }],
        };
        storage.create_table(&info).unwrap();

        let result = execute_command(".tables", &storage);

        match result {
            CommandResult::Ok(output) => {
                assert!(output.contains("users"));
            }
            _ => panic!("Expected Ok"),
        }
    }

    #[test]
    fn test_command_schema() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                    is_unique: false,
                },
            ],
        };
        storage.create_table(&info).unwrap();

        let result = execute_command(".schema users", &storage);

        match result {
            CommandResult::Ok(output) => {
                assert!(output.contains("users"));
                assert!(output.contains("id"));
                assert!(output.contains("name"));
            }
            _ => panic!("Expected Ok"),
        }
    }

    #[test]
    fn test_command_schema_not_found() {
        let storage = MemoryStorage::new();
        let result = execute_command(".schema nonexistent", &storage);

        match result {
            CommandResult::Err(msg) => {
                // Error message contains "Error" from storage.get_table_info
                assert!(msg.contains("Error") || msg.contains("not found"));
            }
            _ => panic!("Expected Err"),
        }
    }

    #[test]
    fn test_command_indexes() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
            }],
        };
        storage.create_table(&info).unwrap();

        let result = execute_command(".indexes users", &storage);

        match result {
            CommandResult::Ok(output) => {
                assert!(output.contains("users"));
            }
            _ => panic!("Expected Ok"),
        }
    }

    #[test]
    fn test_command_exit() {
        let storage = MemoryStorage::new();
        let result = execute_command(".exit", &storage);

        match result {
            CommandResult::Exit => {}
            _ => panic!("Expected Exit"),
        }
    }

    #[test]
    fn test_command_quit() {
        let storage = MemoryStorage::new();
        let result = execute_command(".quit", &storage);

        match result {
            CommandResult::Exit => {}
            _ => panic!("Expected Exit"),
        }
    }

    #[test]
    fn test_command_unknown() {
        let storage = MemoryStorage::new();
        let result = execute_command(".unknown", &storage);

        match result {
            CommandResult::Err(msg) => {
                assert!(msg.contains("Unknown command"));
            }
            _ => panic!("Expected Err"),
        }
    }

    #[test]
    fn test_format_value() {
        assert_eq!(format_value(&sqlrustgo_types::Value::Integer(42)), "42");
        assert_eq!(
            format_value(&sqlrustgo_types::Value::Text("hello".to_string())),
            "hello"
        );
        assert_eq!(format_value(&sqlrustgo_types::Value::Null), "NULL");
        assert_eq!(format_value(&sqlrustgo_types::Value::Float(3.14)), "3.14");
        assert_eq!(format_value(&sqlrustgo_types::Value::Boolean(true)), "true");
    }
}
