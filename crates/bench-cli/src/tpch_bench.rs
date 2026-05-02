//! TPC-H Benchmark Tool
//!
//! Imports TPC-H .tbl data and runs Q1-Q22 queries.
//!
//! Usage:
//!   cargo run -p sqlrustgo-bench-cli -- tpch-bench \\
//!     --ddl scripts/sqlite_tpch_setup.sql \\
//!     --data data/tpch-sf01 \\
//!     --queries all

use crate::cli::TpchBenchArgs;
use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::{MemoryStorage, StorageEngine};
use sqlrustgo_storage::engine::Record;
use sqlrustgo_types::Value;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
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

/// Normalize data type names
fn normalize_data_type(dt: &str) -> String {
    let dt_upper = dt.to_uppercase();
    if dt_upper.contains("INT") || dt_upper == "INTEGER" || dt_upper == "BIGINT" || dt_upper == "SMALLINT" {
        "INTEGER".to_string()
    } else if dt_upper.contains("FLOAT") || dt_upper.contains("DOUBLE") || dt_upper.contains("REAL") {
        "FLOAT".to_string()
    } else if dt_upper.contains("DECIMAL") || dt_upper.contains("NUMERIC") {
        "DECIMAL".to_string()
    } else if dt_upper.contains("VARCHAR") || dt_upper.contains("CHAR") || dt_upper.contains("TEXT") {
        "TEXT".to_string()
    } else if dt_upper.contains("DATE") {
        "DATE".to_string()
    } else if dt_upper.contains("BLOB") || dt_upper.contains("BINARY") {
        "BLOB".to_string()
    } else {
        dt.to_string()
    }
}

/// Parse DDL file and extract table schemas
fn parse_ddl(ddl_path: &str) -> Result<Vec<TableSchema>, String> {
    use sqlrustgo_parser::parse;
    use sqlrustgo_parser::Statement;

    let content = fs::read_to_string(ddl_path)
        .map_err(|e| format!("Failed to read DDL file: {}", e))?;

    let mut schemas = Vec::new();

    let mut remaining = content.as_str();
    loop {
        // Try ";\n" first (normal case), then fallback to ";" at EOF
        let pos = remaining.find(";\n").or_else(|| remaining.find(");"));
        let (stmt, rest) = match pos {
            Some(p) => (&remaining[..p + 1], &remaining[p + 2..]),
            None if !remaining.trim().is_empty() => {
                // Last statement without trailing newline
                (remaining.trim(), "")
            }
            None => break,
        };

        let trimmed = stmt.trim();
        if trimmed.is_empty() {
            remaining = rest;
            continue;
        }

        // Remove leading comment lines
        let content_no_comments = trimmed
            .split('\n')
            .filter(|line| !line.trim().starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();
        if content_no_comments.is_empty() {
            remaining = rest;
            continue;
        }

        let upper = content_no_comments.to_uppercase();
        if upper.contains("DROP ")
            || upper.contains("CREATE DATABASE")
            || upper.contains("VACUUM")
            || upper.contains("ANALYZE")
            || upper.contains("CREATE INDEX")
        {
            remaining = rest;
            continue;
        }

        if let Ok(Statement::CreateTable(stmt)) = parse(&content_no_comments) {
            eprintln!("DEBUG: parse_ddl got {} columns for table {}: col_names={:?}", stmt.columns.len(), stmt.name, stmt.columns.iter().map(|c|c.name.clone()).collect::<Vec<_>>());
            schemas.push(convert_create_table(&stmt)?);
        } else {
            eprintln!("DEBUG: parse_ddl failed to parse as CREATE TABLE: {}", &content_no_comments[..content_no_comments.len().min(120)]);
        }
        remaining = rest;
    }

    Ok(schemas)
}

/// Convert parser's CreateTableStatement to our TableSchema
fn convert_create_table(stmt: &sqlrustgo_parser::CreateTableStatement) -> Result<TableSchema, String> {
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

    Ok(TableSchema {
        name: stmt.name.clone(),
        columns,
        primary_key,
    })
}

/// Import a single table from .tbl file
fn import_table(
    storage: &mut MemoryStorage,
    schema: &TableSchema,
    data_path: &Path,
    batch_size: usize,
) -> Result<usize, String> {
    use sqlrustgo_storage::engine::{ColumnDefinition, TableInfo};
    use sqlrustgo_types::Value;

    let table_path = data_path.join(format!("{}.tbl", schema.name));
    if !table_path.exists() {
        return Err(format!("Table file not found: {}", table_path.display()));
    }

    // Create table
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
        .map_err(|e| format!("Failed to create table: {:?}", e))?;

    let file = File::open(&table_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::with_capacity(1024 * 1024, file);

    let mut count = 0;
    let mut batch: Vec<Record> = Vec::with_capacity(batch_size);

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
        let fields: Vec<&str> = line.trim_end_matches('|').split('|').collect();

        if fields.len() != schema.columns.len() && fields.len() != schema.columns.len() + 1 {
            continue;
        }

        let record: Record = fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let col = &schema.columns[i];
                parse_value(field, &col.data_type)
            })
            .collect();

        batch.push(record);
        count += 1;

        if batch.len() >= batch_size {
            storage
                .insert(&schema.name, batch.clone())
                .map_err(|e| format!("Insert failed: {:?}", e))?;
            batch.clear();
        }
    }

    if !batch.is_empty() {
        storage
            .insert(&schema.name, batch)
            .map_err(|e| format!("Insert failed: {:?}", e))?;
    }

    Ok(count)
}

/// Parse a field value based on data type
fn parse_value(field: &str, data_type: &str) -> Value {

    if field.is_empty() {
        return Value::Null;
    }

    let dt_upper = data_type.to_uppercase();
    if dt_upper == "INTEGER" || dt_upper == "INT" || dt_upper == "BIGINT" || dt_upper == "SMALLINT" {
        match field.parse::<i64>() {
            Ok(v) => Value::Integer(v),
            Err(_) => Value::Null,
        }
    } else if dt_upper == "FLOAT" || dt_upper == "REAL" || dt_upper == "DOUBLE" || dt_upper == "DECIMAL" || dt_upper == "NUMERIC" {
        match field.parse::<f64>() {
            Ok(v) => Value::Float(v),
            Err(_) => Value::Null,
        }
    } else if dt_upper == "TEXT" || dt_upper == "VARCHAR" || dt_upper == "CHAR" || dt_upper == "DATE" {
        Value::Text(field.to_string())
    } else if dt_upper == "BLOB" {
        Value::Blob(field.as_bytes().to_vec())
    } else {
        Value::Text(field.to_string())
    }
}

/// TPC-H query definitions
fn get_query(q: &str) -> Option<&'static str> {
    match q {
        "Q1"  => Some("SELECT l_returnflag, SUM(l_quantity) FROM lineitem GROUP BY l_returnflag"),
        "Q2"  => Some("SELECT s_acctbal, s_name, n_name, p_partkey FROM part, supplier, partsupp, nation, region WHERE p_partkey = ps_partkey AND s_suppkey = ps_suppkey AND p_size = 15 AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'EUROPE' ORDER BY s_acctbal DESC LIMIT 10"),
        "Q3"  => Some("SELECT o_orderkey, SUM(l_extendedprice) FROM orders JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate < '1995-03-15' GROUP BY o_orderkey"),
        "Q4"  => Some("SELECT o_orderpriority, COUNT(*) FROM orders WHERE o_orderdate >= '1993-07-01' AND o_orderdate < '1993-10-01' GROUP BY o_orderpriority"),
        "Q5"  => Some("SELECT n_name, SUM(l_extendedprice) FROM customer, orders, lineitem, supplier, nation, region WHERE c_custkey = o_custkey AND l_orderkey = o_orderkey AND l_suppkey = s_suppkey AND c_nationkey = s_nationkey AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'ASIA' GROUP BY n_name"),
        "Q6"  => Some("SELECT SUM(l_extendedprice) FROM lineitem WHERE l_quantity < 24 AND l_shipdate >= '1994-01-01'"),
        "Q7"  => Some("SELECT n1.n_name AS supp_nation, n2.n_name AS cust_nation, SUM(l_extendedprice) FROM supplier, lineitem, orders, customer, nation n1, nation n2 WHERE s_suppkey = l_suppkey AND o_orderkey = l_orderkey AND c_custkey = o_custkey AND s_nationkey = n1.n_nationkey AND c_nationkey = n2.n_nationkey GROUP BY n1.n_name, n2.n_name"),
        "Q8"  => Some("SELECT extract(year FROM o_orderdate) AS o_year, SUM(l_extendedprice * (1 - l_discount)) AS volume FROM part, supplier, lineitem, orders, customer, nation n1, nation n2, region WHERE p_partkey = l_partkey AND s_suppkey = l_suppkey AND l_orderkey = o_orderkey AND o_custkey = c_custkey AND c_nationkey = n1.n_nationkey AND n1.n_regionkey = r_regionkey AND r_name = 'AMERICA' AND s_nationkey = n2.n_nationkey AND o_orderdate >= '1995-01-01' AND o_orderdate <= '1996-12-31' AND p_type = 'ECONOMY ANODIZED STEEL' GROUP BY o_year"),
        "Q9"  => Some("SELECT n_name AS nation, extract(year FROM o_orderdate) AS o_year, SUM(l_extendedprice * (1 - l_discount) - ps_supplycost * l_quantity) AS amount FROM part, supplier, lineitem, partsupp, orders, nation WHERE s_suppkey = l_suppkey AND ps_suppkey = l_suppkey AND ps_partkey = l_partkey AND p_partkey = l_partkey AND o_orderkey = l_orderkey AND s_nationkey = n_nationkey AND p_name LIKE '%green%' GROUP BY n_name, o_year"),
        "Q10" => Some("SELECT c_custkey, SUM(l_extendedprice) FROM customer JOIN orders ON c_custkey = o_custkey JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate >= '1993-10-01' GROUP BY c_custkey"),
        "Q11" => Some("SELECT ps_partkey, SUM(ps_supplycost * ps_availqty) AS value FROM partsupp, supplier, nation WHERE ps_suppkey = s_suppkey AND s_nationkey = n_nationkey AND n_name = 'GERMANY' GROUP BY ps_partkey"),
        "Q12" => Some("SELECT l_shipmode, COUNT(*) FROM orders, lineitem WHERE l_orderkey = o_orderkey AND l_shipmode IN ('MAIL', 'SHIP') AND l_commitdate < l_receiptdate AND l_shipdate < l_commitdate AND o_orderdate >= '1993-01-01' AND o_orderdate < '1994-01-01' GROUP BY l_shipmode"),
        "Q13" => Some("SELECT c_custkey, COUNT(*) FROM customer GROUP BY c_custkey"),
        "Q14" => Some("SELECT SUM(CASE WHEN p_type LIKE 'PROMO%' THEN l_extendedprice * (1 - l_discount) ELSE 0 END) AS promo_revenue FROM lineitem, part WHERE l_partkey = p_partkey AND l_shipdate >= '1995-09-01' AND l_shipdate < '1995-10-01'"),
        "Q15" => Some("SELECT s_suppkey, s_name, s_address, s_phone, SUM(l_extendedprice * (1 - l_discount)) AS total_revenue FROM supplier, lineitem WHERE l_suppkey = s_suppkey AND l_shipdate >= '1996-01-01' AND l_shipdate < '1996-04-01' GROUP BY s_suppkey, s_name, s_address, s_phone"),
        "Q16" => Some("SELECT p_brand, p_type, p_size, COUNT(DISTINCT ps_suppkey) AS supplier_cnt FROM partsupp, part WHERE p_partkey = ps_partkey AND p_brand <> 'Brand#45' AND p_type NOT LIKE 'MEDIUM POLISHED%' AND p_size IN (49, 14, 23, 45, 19, 3, 36, 9) GROUP BY p_brand, p_type, p_size"),
        "Q17" => Some("SELECT SUM(l_extendedprice) / 7.0 AS avg_yearly FROM lineitem, part WHERE p_partkey = l_partkey AND p_brand = 'Brand#23' AND p_container = 'MED BOX'"),
        "Q18" => Some("SELECT c_name, c_custkey, o_orderkey, o_orderdate, o_totalprice, SUM(l_quantity) FROM customer, orders, lineitem WHERE o_orderkey = l_orderkey AND c_custkey = o_custkey GROUP BY c_name, c_custkey, o_orderkey, o_orderdate, o_totalprice"),
        "Q19" => Some("SELECT SUM(l_extendedprice * (1 - l_discount)) AS revenue FROM lineitem, part WHERE p_partkey = l_partkey AND p_brand = 'Brand#12' AND p_container IN ('SM CASE', 'SM BOX', 'SM PACK', 'SM PKG') AND l_quantity >= 1 AND l_quantity <= 11 AND p_size BETWEEN 1 AND 5"),
        "Q20" => Some("SELECT s_name, s_address FROM supplier, nation WHERE s_nationkey = n_nationkey AND n_name = 'CANADA'"),
        "Q21" => Some("SELECT s_name, COUNT(*) AS numwait FROM supplier, lineitem, orders, nation WHERE s_suppkey = l_suppkey AND o_orderkey = l_orderkey AND o_orderstatus = 'F' AND s_nationkey = n_nationkey AND n_name = 'SAUDI ARABIA' GROUP BY s_name"),
        "Q22" => Some("SELECT SUBSTRING(c_phone FROM 1 FOR 2) AS cntrycode, COUNT(*) AS numcust, SUM(c_acctbal) AS totacctbal FROM customer WHERE SUBSTRING(c_phone FROM 1 FOR 2) IN ('13', '31', '23', '29', '30', '18', '17') AND c_acctbal > 0.00 GROUP BY cntrycode"),
        _ => None,
    }
}

/// Run TPC-H benchmark with real imported data
pub fn run(args: TpchBenchArgs) -> Result<(), String> {
    println!("==============================================");
    println!("  TPC-H Benchmark with Real Data");
    println!("==============================================");
    println!("DDL: {}", args.ddl);
    println!("Data: {}", args.data);
    println!("Queries: {}", args.queries);
    println!("Iterations: {}", args.iterations);
    println!();

    // Parse DDL
    println!("=== Parsing DDL ===");
    let schemas = parse_ddl(&args.ddl)?;
    println!("Found {} tables:", schemas.len());
    for schema in &schemas {
        println!("  - {} ({} columns)", schema.name, schema.columns.len());
    }
    println!();

    let data_path = PathBuf::from(&args.data);

    // Create storage and import data
    let mut storage = MemoryStorage::new();

    println!("=== Importing data ===");
    let import_start = Instant::now();
    let mut imported_count = 0;

    for schema in &schemas {
        let start = Instant::now();
        match import_table(&mut storage, schema, &data_path, args.batch_size) {
            Ok(count) => {
                println!("  {}: {} rows in {:.2}s", schema.name, count, start.elapsed().as_secs_f64());
                imported_count += count;
            }
            Err(e) => {
                println!("  {}: ERROR - {}", schema.name, e);
            }
        }
    }

    println!("\nTotal import: {} rows in {:.2}s", imported_count, import_start.elapsed().as_secs_f64());
    println!();

    // Determine which queries to run
    let query_names: Vec<String> = if args.queries == "all" {
        (1..=22).map(|i| format!("Q{}", i)).collect()
    } else {
        args.queries.split(',').map(|s| s.trim().to_string()).collect()
    };

    println!("=== Running {} TPC-H Queries ===", query_names.len());
    println!();

    let mut results: Vec<(String, f64, usize, String)> = Vec::new();

    for qname in &query_names {
        let sql = match get_query(qname) {
            Some(s) => s,
            None => {
                println!("  {}: UNKNOWN QUERY", qname);
                continue;
            }
        };

        // Timed runs (each iteration gets its own engine pointing to the shared populated storage)
        let mut total_time = 0.0;
        let mut result_rows = 0;

        for iter in 0..args.iterations {
            let iter_start = Instant::now();
            let storage = Arc::new(RwLock::new(storage.clone()));
            let mut engine = ExecutionEngine::new(storage);
            let result = engine.execute(sql);
            let elapsed = iter_start.elapsed().as_secs_f64() * 1000.0; // ms

            if iter == 0 {
                result_rows = result.map(|r| r.rows.len()).unwrap_or(0);
            }
            total_time += elapsed;
        }

        let avg_ms = total_time / args.iterations as f64;
        println!(
            "  {}: {:.2} ms ({} rows, {} iters)",
            qname, avg_ms, result_rows, args.iterations
        );

        results.push((qname.to_string(), avg_ms, result_rows, sql.to_string()));
    }

    println!();
    println!("==============================================");
    println!("  Benchmark Complete");
    println!("==============================================");
    println!();

    // Summary table
    println!("{:<6} {:>12} {:>10}", "Query", "Avg (ms)", "Rows");
    println!("{}", "-".repeat(30));
    let mut total_ms = 0.0;
    for (name, ms, rows, _) in &results {
        println!("{:<6} {:>12.2} {:>10}", name, ms, rows);
        total_ms += ms;
    }
    println!("{}", "-".repeat(30));
    println!("{:<6} {:>12.2}", "TOTAL", total_ms);
    println!();

    // Save results
    if let Some(ref path) = args.output {
        let json = serde_json::json!({
            "scale": "sf0.1",
            "import_rows": imported_count,
            "import_time_ms": import_start.elapsed().as_secs_f64() * 1000.0,
            "queries": results.iter().map(|(name, ms, rows, sql)| {
                serde_json::json!({
                    "name": name,
                    "avg_ms": ms,
                    "rows": rows,
                    "sql": sql
                })
            }).collect::<Vec<_>>(),
        });
        fs::write(path, serde_json::to_string_pretty(&json).unwrap())
            .map_err(|e| format!("Failed to write results: {}", e))?;
        println!("Results saved to: {}", path);
    }

    Ok(())
}
