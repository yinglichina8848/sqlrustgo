//! TPC-H Data Import Tool
//!
//! Imports TPC-H .tbl files into SQLRustGo Storage with verification.
//!
//! Usage:
//!   cargo run -p sqlrustgo-bench-cli -- tpch-import \
//!     --ddl scripts/pg_tpch_setup.sql \
//!     --data data/tpch-sf01 \
//!     --output storage/tpch-sf01

use crate::cli::TpchImportArgs;
use sqlrustgo_parser::parse;
use sqlrustgo_parser::{CreateTableStatement, Statement, TableConstraint};
use sqlrustgo_storage::engine::{ColumnDefinition, Record, TableInfo};
use sqlrustgo_storage::{MemoryStorage, StorageEngine};
use sqlrustgo_types::Value;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Table schema mapping
#[derive(Clone)]
struct TableSchema {
    name: String,
    columns: Vec<ColumnSchema>,
    primary_key: Vec<String>,
}

/// Column schema with type info
#[derive(Clone)]
struct ColumnSchema {
    name: String,
    data_type: String,
    nullable: bool,
}

/// Progress tracker
struct ProgressTracker {
    total: usize,
    processed: usize,
    start_time: Instant,
}

impl ProgressTracker {
    fn new(total: usize) -> Self {
        Self {
            total,
            processed: 0,
            start_time: Instant::now(),
        }
    }

    fn update(&mut self, count: usize) {
        self.processed += count;
    }

    fn print(&self, table: &str) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let rate = if elapsed > 0.0 {
            self.processed as f64 / elapsed
        } else {
            0.0
        };
        let pct = 100.0 * self.processed as f64 / self.total as f64;
        println!(
            "  {}: {} rows ({:.1}%) | {:.0} rows/s",
            table, self.processed, pct, rate
        );
    }
}

/// Parse DDL file and extract table schemas
fn parse_ddl(ddl_path: &str) -> Result<Vec<TableSchema>, String> {
    let content = fs::read_to_string(ddl_path)
        .map_err(|e| format!("Failed to read DDL file: {}", e))?;

    let mut schemas = Vec::new();

    // Split on ";\n" to separate statements, handling multi-line statements
    let mut remaining = content.as_str();
    loop {
        // Find next statement end
        if let Some(pos) = remaining.find(";\n") {
            let stmt = &remaining[..pos + 1]; // include the semicolon
            remaining = &remaining[pos + 2..]; // skip ";\n"

            // Skip empty lines
            let trimmed = stmt.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Remove leading comments (-- ... \n) to check actual statement content
            let content_no_comments = trimmed
                .split('\n')
                .filter(|line| !line.trim().starts_with("--"))
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();
            // If nothing left after removing comments, skip
            if content_no_comments.is_empty() {
                continue;
            }
            // Skip non-CREATE TABLE statements
            let upper = content_no_comments.to_uppercase();
            if upper.contains("DROP ")
                || upper.contains("CREATE DATABASE")
                || upper.contains("VACUUM")
                || upper.contains("ANALYZE")
                || upper.contains("CREATE INDEX") {
                continue;
            }

            if let Ok(Statement::CreateTable(stmt)) = parse(&content_no_comments) {
                schemas.push(convert_create_table(&stmt)?);
            }
        } else {
            break;
        }
    }

    Ok(schemas)
}

/// Convert parser's CreateTableStatement to our TableSchema
fn convert_create_table(stmt: &CreateTableStatement) -> Result<TableSchema, String> {
    let mut columns = Vec::new();
    let mut primary_key = Vec::new();

    for col in &stmt.columns {
        let data_type = normalize_data_type(&col.data_type);
        columns.push(ColumnSchema {
            name: col.name.clone(),
            data_type,
            nullable: col.nullable,
        });

        if col.primary_key {
            primary_key.push(col.name.clone());
        }
    }

    // Also check constraints for primary key
    for constraint in &stmt.constraints {
        match constraint {
            TableConstraint::PrimaryKey { columns: cols } => {
                primary_key.extend(cols.iter().cloned());
            }
            _ => {}
        }
    }

    Ok(TableSchema {
        name: stmt.name.clone(),
        columns,
        primary_key,
    })
}

/// Normalize data type strings
fn normalize_data_type(dtype: &str) -> String {
    let upper = dtype.to_uppercase();
    if upper.contains("INT") {
        "INTEGER".to_string()
    } else if upper.contains("VARCHAR") || upper.contains("TEXT") || upper.contains("CHAR") {
        "TEXT".to_string()
    } else if upper.contains("DATE") {
        "DATE".to_string()
    } else if upper.contains("DECIMAL") || upper.contains("NUMERIC") {
        "DECIMAL".to_string()
    } else if upper.contains("FLOAT") || upper.contains("REAL") || upper.contains("DOUBLE") {
        "FLOAT".to_string()
    } else if upper.contains("BLOB") || upper.contains("BYTE") {
        "BLOB".to_string()
    } else {
        dtype.to_uppercase()
    }
}

/// Parse a .tbl line into Values based on schema
fn parse_tbl_line(line: &str, schema: &TableSchema) -> Result<Vec<Value>, String> {
    let fields: Vec<&str> = line.trim().split('|').collect();

    if fields.is_empty() || fields[0].is_empty() {
        return Err("Empty line".to_string());
    }

    let mut values = Vec::with_capacity(schema.columns.len());

    for (i, col) in schema.columns.iter().enumerate() {
        let field = fields.get(i).unwrap_or(&"");

        if field.is_empty() || field == &"" {
            values.push(Value::Null);
            continue;
        }

        let value = match col.data_type.as_str() {
            "INTEGER" | "INT" | "BIGINT" | "SMALLINT" => {
                match field.parse::<i64>() {
                    Ok(v) => Value::Integer(v),
                    Err(_) => Value::Null,
                }
            }
            "FLOAT" | "REAL" | "DOUBLE" | "DECIMAL" | "NUMERIC" => {
                match field.parse::<f64>() {
                    Ok(v) => Value::Float(v),
                    Err(_) => Value::Null,
                }
            }
            "TEXT" | "VARCHAR" | "CHAR" | "DATE" => {
                Value::Text(field.to_string())
            }
            "BLOB" => {
                Value::Blob(field.as_bytes().to_vec())
            }
            _ => Value::Text(field.to_string()),
        };

        values.push(value);
    }

    Ok(values)
}

/// Count lines in a .tbl file
fn count_tbl_lines(path: &Path) -> Result<usize, String> {
    let file = File::open(path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count())
}

/// Import a single table from .tbl file
fn import_table(
    storage: &mut MemoryStorage,
    schema: &TableSchema,
    data_path: &Path,
    batch_size: usize,
) -> Result<usize, String> {
    let table_path = data_path.join(format!("{}.tbl", schema.name));

    if !table_path.exists() {
        return Err(format!("Table file not found: {}", table_path.display()));
    }

    // Count total lines for progress
    let total_lines = count_tbl_lines(&table_path)?;
    let mut progress = ProgressTracker::new(total_lines);

    let file = File::open(&table_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::with_capacity(1024 * 1024, file); // 1MB buffer

    let mut batch: Vec<Record> = Vec::with_capacity(batch_size);
    let mut total_imported = 0;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        if line.trim().is_empty() {
            continue;
        }

        match parse_tbl_line(&line, schema) {
            Ok(values) => {
                batch.push(values);

                if batch.len() >= batch_size {
                    // Insert batch
                    if let Err(e) = storage.insert(&schema.name, batch.clone()) {
                        return Err(format!("Insert failed: {:?}", e));
                    }
                    total_imported += batch.len();
                    batch.clear();
                    progress.update(batch_size);
                    progress.print(&schema.name);
                }
            }
            Err(_) => continue,
        }
    }

    // Insert remaining
    if !batch.is_empty() {
        if let Err(e) = storage.insert(&schema.name, batch.clone()) {
            return Err(format!("Insert failed: {:?}", e));
        }
        total_imported += batch.len();
    }

    progress.update(total_imported % batch_size);
    progress.print(&schema.name);

    Ok(total_imported)
}

/// Verify data by checking row counts
fn verify_data(
    storage: &MemoryStorage,
    schemas: &[TableSchema],
    data_path: &Path,
) -> Result<(), String> {
    println!("\n=== Data Verification ===");

    let mut all_pass = true;

    for schema in schemas {
        let table_path = data_path.join(format!("{}.tbl", schema.name));
        if !table_path.exists() {
            println!("  {}: SKIP (file not found)", schema.name);
            continue;
        }

        let expected_count = count_tbl_lines(&table_path)?;
        let actual_count = storage
            .scan(&schema.name)
            .map(|rows| rows.len())
            .unwrap_or(0);

        let status = if expected_count == actual_count {
            "PASS"
        } else {
            "FAIL"
        };

        println!(
            "  {}: expected={}, actual={} [{}]",
            schema.name, expected_count, actual_count, status
        );

        if expected_count != actual_count {
            all_pass = false;
        }
    }

    if all_pass {
        println!("\nAll verifications PASSED!");
    } else {
        println!("\nSome verifications FAILED!");
    }

    Ok(())
}

/// Save storage metadata to disk
fn save_metadata(storage: &MemoryStorage, schemas: &[TableSchema], output_path: &Path) -> Result<(), String> {
    println!("\n=== Saving Metadata ===");

    // Create output directory
    fs::create_dir_all(output_path)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Save table info for each schema
    for schema in schemas {
        let info = storage.get_table_info(&schema.name)
            .map_err(|e| format!("Failed to get table info for {}: {:?}", schema.name, e))?;
        
        let info_path = output_path.join(format!("{}.schema.json", schema.name));
        let json = serde_json::to_string_pretty(&info)
            .map_err(|e| format!("Failed to serialize schema: {}", e))?;
        fs::write(&info_path, json)
            .map_err(|e| format!("Failed to write schema file: {}", e))?;
        println!("  Saved: {}.schema.json", schema.name);
    }

    // Save overall metadata
    let metadata_path = output_path.join("storage_metadata.json");
    let metadata = serde_json::json!({
        "version": "1.0",
        "table_count": schemas.len(),
        "tables": schemas.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
    });
    fs::write(&metadata_path, serde_json::to_string_pretty(&metadata).unwrap())
        .map_err(|e| format!("Failed to write metadata: {}", e))?;

    println!("\nMetadata saved to: {}", output_path.display());

    Ok(())
}

/// Main import function
pub fn run(args: TpchImportArgs) -> Result<(), String> {
    println!("==============================================");
    println!("  TPC-H Data Import Tool");
    println!("==============================================");
    println!("DDL: {}", args.ddl);
    println!("Data: {}", args.data);
    println!("Output: {}", args.output);
    println!("Batch size: {}", args.batch_size);
    println!();

    // Parse DDL
    println!("=== Parsing DDL ===");
    let schemas = parse_ddl(&args.ddl)?;
    println!("Found {} tables:", schemas.len());
    for schema in &schemas {
        println!("  - {} ({} columns, PK: {:?})",
            schema.name,
            schema.columns.len(),
            schema.primary_key
        );
    }
    println!();

    let data_path = PathBuf::from(&args.data);
    let output_path = PathBuf::from(&args.output);

    // Create storage
    let mut storage = MemoryStorage::new();

    // Create tables
    println!("=== Creating tables ===");
    for schema in &schemas {
        let columns: Vec<ColumnDefinition> = schema
            .columns
            .iter()
            .map(|c| {
                let mut col = ColumnDefinition::new(&c.name, &c.data_type);
                col.nullable = c.nullable;
                if schema.primary_key.contains(&c.name) {
                    col.primary_key = true;
                }
                col
            })
            .collect();

        let info = TableInfo {
            name: schema.name.clone(),
            columns,
            ..Default::default()
        };

        storage.create_table(&info)
            .map_err(|e| format!("Failed to create table {}: {:?}", schema.name, e))?;
        println!("  Created: {}", schema.name);
    }
    println!();

    // Import data
    println!("=== Importing data ===");
    let total_start = Instant::now();

    for schema in &schemas {
        let start = Instant::now();
        match import_table(&mut storage, schema, &data_path, args.batch_size) {
            Ok(count) => {
                println!("  {}: imported {} rows in {:.2}s",
                    schema.name, count, start.elapsed().as_secs_f64());
            }
            Err(e) => {
                println!("  {}: ERROR - {}", schema.name, e);
            }
        }
    }

    println!("\nTotal import time: {:.2}s", total_start.elapsed().as_secs_f64());
    println!();

    // Verify
    let schemas_clone = schemas.clone();
    verify_data(&storage, &schemas_clone, &data_path)?;

    if args.verify_only {
        println!("\nVerify-only mode, skipping save.");
        return Ok(());
    }

    // Save metadata to disk
    save_metadata(&storage, &schemas_clone, &output_path)?;

    println!("\n==============================================");
    println!("  Import Complete!");
    println!("==============================================");

    Ok(())
}
