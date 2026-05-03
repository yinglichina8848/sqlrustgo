//! TPC-H data import into SQLRustGo storage
//!
//! Usage:
//!     sqlrustgo-bench-cli tpch-import \
//!         --ddl data/tpch-sf01/schema.sql \
//!         --data data/tpch-sf01/ \
//!         --output storage/tpch-sf01

use crate::cli::TpchImportArgs;
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use sqlrustgo_parser::parse;
use sqlrustgo_parser::{CreateTableStatement, Statement, TableConstraint};
use sqlrustgo_storage::engine::{ColumnDefinition as EngineColDef, TableInfo};
use sqlrustgo_storage::{MemoryStorage, StorageEngine};
use sqlrustgo_types::Value;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

/// Schema of a TPC-H table
#[derive(Debug, Clone)]
struct TableSchema {
    name: String,
    columns: Vec<ColDef>,
    primary_key: Option<String>,
}

#[derive(Debug, Clone)]
struct ColDef {
    name: String,
    data_type: String,
}

/// Parse DDL file and extract table schemas
fn parse_ddl(ddl_path: &str) -> Result<Vec<TableSchema>, String> {
    let content =
        fs::read_to_string(ddl_path).map_err(|e| format!("Failed to read DDL file: {}", e))?;

    let mut schemas = Vec::new();

    for line in content.lines() {
        let upper = line.to_uppercase().trim().to_string();

        if upper.is_empty()
            || !upper.starts_with("CREATE TABLE")
            || upper.contains("CREATE DATABASE")
            || upper.contains("VACUUM")
            || upper.contains("ANALYZE")
            || upper.contains("CREATE INDEX")
        {
            continue;
        }

        let statements = parse(line).map_err(|e| format!("Parse error: {:?}", e))?;

        if let Statement::CreateTable(create) = statements {
            let schema = convert_create_table(&create)?;
            schemas.push(schema);
        }
    }

    Ok(schemas)
}

fn convert_create_table(stmt: &CreateTableStatement) -> Result<TableSchema, String> {
    let mut primary_key = None;

    for constraint in &stmt.constraints {
        if let TableConstraint::PrimaryKey { columns } = constraint {
            primary_key = columns.first().cloned();
        }
    }

    let columns: Vec<ColDef> = stmt
        .columns
        .iter()
        .map(|col| ColDef {
            name: col.name.clone(),
            data_type: col.data_type.clone(),
        })
        .collect();

    Ok(TableSchema {
        name: stmt.name.clone(),
        columns,
        primary_key,
    })
}

/// Convert .tbl field to Value
fn parse_field(field: &str, col: &ColDef) -> Value {
    let field = field.trim();
    if field.is_empty() || field == "NULL" {
        return Value::Null;
    }

    let value = match col.data_type.to_uppercase().as_str() {
        "INTEGER" | "INT" | "BIGINT" | "SMALLINT" => match field.parse::<i64>() {
            Ok(v) => Value::Integer(v),
            Err(_) => Value::Null,
        },
        "FLOAT" | "REAL" | "DOUBLE" | "DECIMAL" | "NUMERIC" => match field.parse::<f64>() {
            Ok(v) => Value::Float(v),
            Err(_) => Value::Null,
        },
        "TEXT" | "VARCHAR" | "CHAR" | "DATE" => Value::Text(field.to_string()),
        "BLOB" => Value::Blob(field.as_bytes().to_vec()),
        _ => Value::Text(field.to_string()),
    };

    if matches!(value, Value::Null) && !field.is_empty() && field != "NULL" {
        return Value::Text(field.to_string());
    }

    value
}

/// Count lines in a .tbl file
fn count_tbl_lines(path: &Path) -> Result<usize, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count())
}

/// Progress tracker for import
struct ProgressTracker {
    total: usize,
    current: usize,
    last_percent: usize,
}

impl ProgressTracker {
    fn new(total: usize) -> Self {
        Self {
            total,
            current: 0,
            last_percent: 0,
        }
    }

    fn update(&mut self, count: usize) {
        self.current += count;
        let percent = (self.current * 100) / self.total;
        if percent >= self.last_percent + 10 {
            println!("  Progress: {}%", percent);
            self.last_percent = percent;
        }
    }
}

/// Import a single table from .tbl file
fn import_table(
    storage: &mut MemoryStorage,
    schema: &TableSchema,
    data_path: &Path,
    batch_size: usize,
) -> Result<usize, String> {
    let table_path = data_path.join(format!("{}.tbl", schema.name));

    let total_lines = count_tbl_lines(&table_path)?;
    let mut progress = ProgressTracker::new(total_lines);

    let file = File::open(&table_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::with_capacity(1024 * 1024, file); // 1MB buffer

    let mut batch: Vec<Vec<Value>> = Vec::with_capacity(batch_size);
    let mut total_imported = 0;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Read error: {}", e))?;

        // TPC-H .tbl format: fields separated by |
        let fields: Vec<&str> = line.split('|').collect();

        let record: Vec<Value> = schema
            .columns
            .iter()
            .enumerate()
            .map(|(idx, col)| {
                let field = fields.get(idx).copied().unwrap_or("");
                parse_field(field, col)
            })
            .collect();

        batch.push(record);
        progress.update(1);

        if batch.len() >= batch_size {
            storage
                .insert(&schema.name, batch.clone())
                .map_err(|e| format!("Insert error: {:?}", e))?;
            total_imported += batch.len();
            batch.clear();
        }
    }

    // Flush remaining
    if !batch.is_empty() {
        storage
            .insert(&schema.name, batch.clone())
            .map_err(|e| format!("Insert error: {:?}", e))?;
        total_imported += batch.len();
    }

    Ok(total_imported)
}

/// Save storage metadata to disk
fn save_metadata(
    storage: &MemoryStorage,
    schemas: &[TableSchema],
    output_path: &Path,
) -> Result<(), String> {
    println!("\n=== Saving Metadata ===");

    fs::create_dir_all(output_path)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    for schema in schemas {
        let info = storage
            .get_table_info(&schema.name)
            .map_err(|e| format!("Failed to get table info for {}: {:?}", schema.name, e))?;

        let info_path = output_path.join(format!("{}.schema.json", schema.name));
        let json = serde_json::to_string_pretty(&info)
            .map_err(|e| format!("Failed to serialize schema: {}", e))?;
        fs::write(&info_path, json).map_err(|e| format!("Failed to write schema file: {}", e))?;
        println!("  Saved: {}.schema.json", schema.name);
    }

    let metadata_path = output_path.join("metadata.json");
    let metadata = serde_json::json!({
        "version": "1.0",
        "table_count": schemas.len(),
        "tables": schemas.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
    });
    fs::write(
        &metadata_path,
        serde_json::to_string_pretty(&metadata).unwrap(),
    )
    .map_err(|e| format!("Failed to write metadata: {}", e))?;

    println!("\nMetadata saved to: {}", output_path.display());

    Ok(())
}

/// Export records to a Parquet file
fn export_table_to_parquet(
    path: &Path,
    records: &[Vec<Value>],
    column_names: &[String],
) -> Result<(), String> {
    use arrow::array::{ArrayRef, StringBuilder};
    use arrow::datatypes::DataType;
    use arrow::datatypes::Field;

    if records.is_empty() {
        return Err("Cannot export empty records to Parquet".to_string());
    }

    let num_columns = records[0].len();
    if num_columns != column_names.len() {
        return Err(format!(
            "Column count mismatch: {} values but {} column names",
            num_columns,
            column_names.len()
        ));
    }

    // Build Arrow schema - all columns as Utf8 for simplicity
    let fields: Vec<Field> = column_names
        .iter()
        .map(|name| Field::new(name, DataType::Utf8, true))
        .collect();
    let schema = arrow::datatypes::Schema::new(fields);

    // Build arrays from records
    let num_rows = records.len();
    let num_cols = column_names.len();

    let mut arrays: Vec<ArrayRef> = Vec::with_capacity(num_cols);

    for col_idx in 0..num_cols {
        let mut builder = StringBuilder::new();

        for row_idx in 0..num_rows {
            if col_idx < records[row_idx].len() {
                let value = &records[row_idx][col_idx];
                let s = match value {
                    Value::Integer(i) => i.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Text(t) => t.clone(),
                    Value::Boolean(b) => b.to_string(),
                    Value::Blob(b) => format!("{:?}", b),
                    Value::Null => {
                        builder.append_null();
                        continue;
                    }
                    _ => format!("{:?}", value),
                };
                builder.append_value(&s);
            } else {
                builder.append_value("");
            }
        }

        arrays.push(Arc::new(builder.finish()) as ArrayRef);
    }

    let batch = RecordBatch::try_new(Arc::new(schema.clone()), arrays)
        .map_err(|e| format!("Failed to create record batch: {}", e))?;

    // Write to Parquet
    let file = File::create(path).map_err(|e| format!("Failed to create Parquet file: {}", e))?;
    let props = WriterProperties::builder().build();

    let mut writer = ArrowWriter::try_new(file, Arc::new(schema), Some(props))
        .map_err(|e| format!("Failed to create Arrow writer: {}", e))?;

    writer
        .write(&batch)
        .map_err(|e| format!("Failed to write batch: {}", e))?;

    writer
        .close()
        .map_err(|e| format!("Failed to close writer: {}", e))?;

    Ok(())
}

/// Verify imported data
fn verify_data(
    storage: &MemoryStorage,
    schemas: &[TableSchema],
    data_path: &Path,
) -> Result<(), String> {
    println!("\n=== Verifying Data ===");

    for schema in schemas {
        let table_path = data_path.join(format!("{}.tbl", schema.name));
        let expected_count = count_tbl_lines(&table_path)?;
        let actual_count: usize = storage
            .scan(&schema.name)
            .map(|rows: Vec<Vec<Value>>| rows.len())
            .unwrap_or(0);

        if actual_count == expected_count {
            println!("  {}: {} rows [OK]", schema.name, actual_count);
        } else {
            println!(
                "  {}: {} rows [MISMATCH - expected {}]",
                schema.name, actual_count, expected_count
            );
        }
    }

    Ok(())
}

/// Run the TPC-H import
pub fn run(args: &TpchImportArgs) -> Result<(), String> {
    let total_start = Instant::now();

    let data_path = PathBuf::from(&args.data);
    let output_path = PathBuf::from(&args.output);

    println!("=== TPC-H Import ===");
    println!("DDL: {}", args.ddl);
    println!("Data: {}", args.data);
    println!("Output: {}", args.output);
    println!();

    let schemas = parse_ddl(&args.ddl)?;
    println!("Found {} tables:", schemas.len());
    for schema in &schemas {
        println!(
            "  - {} ({} columns, PK: {:?})",
            schema.name,
            schema.columns.len(),
            schema.primary_key
        );
    }

    // Create storage
    let mut storage = MemoryStorage::new();

    // Create tables
    println!("\n=== Creating Tables ===");
    for schema in &schemas {
        let columns: Vec<EngineColDef> = schema
            .columns
            .iter()
            .map(|c| EngineColDef {
                name: c.name.clone(),
                data_type: c.data_type.clone(),
                nullable: true,
                primary_key: schema.primary_key.as_ref() == Some(&c.name),
            })
            .collect();

        let info = TableInfo {
            name: schema.name.clone(),
            columns,
            foreign_keys: vec![],
            unique_constraints: vec![],
            check_constraints: vec![],
            partition_info: None,
        };

        storage
            .create_table(&info)
            .map_err(|e| format!("Failed to create table {}: {:?}", schema.name, e))?;
        println!("  Created: {}", schema.name);
    }

    // Import data
    println!("\n=== Importing Data ===");
    let schemas_clone = schemas.clone();
    for schema in &schemas_clone {
        let start = Instant::now();
        match import_table(&mut storage, schema, &data_path, args.batch_size) {
            Ok(count) => {
                println!(
                    "  {}: imported {} rows in {:.2}s",
                    schema.name,
                    count,
                    start.elapsed().as_secs_f64()
                );
            }
            Err(e) => {
                println!("  {}: ERROR - {}", schema.name, e);
            }
        }
    }

    println!(
        "\nTotal import time: {:.2}s",
        total_start.elapsed().as_secs_f64()
    );
    println!();

    // Verify
    verify_data(&storage, &schemas, &data_path)?;

    if args.verify_only {
        println!("\nVerify-only mode, skipping save.");
        return Ok(());
    }

    // Save metadata to disk
    save_metadata(&storage, &schemas_clone, &output_path)?;

    // Export each table to Parquet
    println!("\n=== Exporting to Parquet ===");
    let export_start = Instant::now();
    for schema in &schemas_clone {
        let records = storage
            .scan(&schema.name)
            .map_err(|e| format!("Scan failed for {}: {:?}", schema.name, e))?;

        if records.is_empty() {
            println!("  {}: empty, skipping", schema.name);
            continue;
        }

        let parquet_path = output_path.join(format!("{}.parquet", schema.name));
        let column_names: Vec<String> = schema.columns.iter().map(|c| c.name.clone()).collect();

        match export_table_to_parquet(&parquet_path, &records, &column_names) {
            Ok(()) => {
                println!(
                    "  {}: {} rows -> {} ({:.2}s)",
                    schema.name,
                    records.len(),
                    parquet_path.file_name().unwrap().to_string_lossy(),
                    export_start.elapsed().as_secs_f64()
                );
            }
            Err(e) => {
                println!("  {}: Parquet export failed: {}", schema.name, e);
            }
        }
    }
    println!(
        "  Parquet export done in {:.2}s",
        export_start.elapsed().as_secs_f64()
    );

    println!("\n==============================================");
    println!("  Import Complete!");
    println!("==============================================");

    Ok(())
}
