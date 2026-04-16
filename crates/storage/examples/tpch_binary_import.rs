//! Fast TPC-H Binary Import
//!
//! Directly imports .tbl files to BinaryTableStorage for fast loading.
//!
//! Usage:
//!   cargo run --example tpch_binary_import -- /path/to/tpch-sf1

use sqlrustgo_storage::binary_storage::BinaryTableStorage;
use sqlrustgo_storage::{ColumnDefinition, FileStorage, StorageEngine, TableData, TableInfo};
use sqlrustgo_types::Value;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let tpch_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp/tpch-dbgen/sf1"));

    let binary_dir = PathBuf::from("/tmp/tpch_binary");

    println!("==============================================");
    println!("  TPC-H Fast Binary Import");
    println!("==============================================");
    println!("Source: {:?}", tpch_dir);
    println!("Binary dir: {:?}", binary_dir);

    // Clean up
    let _ = std::fs::remove_dir_all(&binary_dir);
    std::fs::create_dir_all(&binary_dir).unwrap();

    let binary = BinaryTableStorage::new(binary_dir.clone()).unwrap();

    let total_start = Instant::now();

    // Create tables and import
    create_and_import(&binary, "nation", &tpch_dir.join("nation.tbl"), 4);
    create_and_import(&binary, "region", &tpch_dir.join("region.tbl"), 3);
    create_and_import(&binary, "supplier", &tpch_dir.join("supplier.tbl"), 7);
    create_and_import(&binary, "part", &tpch_dir.join("part.tbl"), 9);
    create_and_import(&binary, "partsupp", &tpch_dir.join("partsupp.tbl"), 5);
    create_and_import(&binary, "customer", &tpch_dir.join("customer.tbl"), 8);
    create_and_import(&binary, "orders", &tpch_dir.join("orders.tbl"), 9);
    create_and_import(&binary, "lineitem", &tpch_dir.join("lineitem.tbl"), 16);

    let total_time = total_start.elapsed();

    println!();
    println!("==============================================");
    println!("  Import Complete!");
    println!("  Total time: {:.2}s", total_time.as_secs_f64());
    println!("  Binary files in: {:?}", binary_dir);
    println!("==============================================");
}

fn col_def(name: &str, dtype: &str) -> ColumnDefinition {
    ColumnDefinition {
        name: name.to_string(),
        data_type: dtype.to_string(),
        nullable: true,
        is_unique: false,
        is_primary_key: false,
        references: None,
        auto_increment: false,
        compression: None,
    }
}

fn create_table_info(name: &str, num_cols: usize) -> TableInfo {
    let columns = match (name, num_cols) {
        ("nation", 4) => vec![
            col_def("n_nationkey", "INTEGER"),
            col_def("n_name", "TEXT"),
            col_def("n_regionkey", "INTEGER"),
            col_def("n_comment", "TEXT"),
        ],
        ("region", 3) => vec![
            col_def("r_regionkey", "INTEGER"),
            col_def("r_name", "TEXT"),
            col_def("r_comment", "TEXT"),
        ],
        ("supplier", 7) => vec![
            col_def("s_suppkey", "INTEGER"),
            col_def("s_name", "TEXT"),
            col_def("s_address", "TEXT"),
            col_def("s_nationkey", "INTEGER"),
            col_def("s_phone", "TEXT"),
            col_def("s_acctbal", "REAL"),
            col_def("s_comment", "TEXT"),
        ],
        ("part", 9) => vec![
            col_def("p_partkey", "INTEGER"),
            col_def("p_name", "TEXT"),
            col_def("p_mfgr", "TEXT"),
            col_def("p_brand", "TEXT"),
            col_def("p_type", "TEXT"),
            col_def("p_size", "INTEGER"),
            col_def("p_container", "TEXT"),
            col_def("p_retailprice", "REAL"),
            col_def("p_comment", "TEXT"),
        ],
        ("partsupp", 5) => vec![
            col_def("ps_partkey", "INTEGER"),
            col_def("ps_suppkey", "INTEGER"),
            col_def("ps_availqty", "INTEGER"),
            col_def("ps_supplycost", "REAL"),
            col_def("ps_comment", "TEXT"),
        ],
        ("customer", 8) => vec![
            col_def("c_custkey", "INTEGER"),
            col_def("c_name", "TEXT"),
            col_def("c_address", "TEXT"),
            col_def("c_nationkey", "INTEGER"),
            col_def("c_phone", "TEXT"),
            col_def("c_acctbal", "REAL"),
            col_def("c_mktsegment", "TEXT"),
            col_def("c_comment", "TEXT"),
        ],
        ("orders", 9) => vec![
            col_def("o_orderkey", "INTEGER"),
            col_def("o_custkey", "INTEGER"),
            col_def("o_orderstatus", "TEXT"),
            col_def("o_totalprice", "REAL"),
            col_def("o_orderdate", "TEXT"),
            col_def("o_orderpriority", "TEXT"),
            col_def("o_clerk", "TEXT"),
            col_def("o_shippriority", "INTEGER"),
            col_def("o_comment", "TEXT"),
        ],
        ("lineitem", 16) => vec![
            col_def("l_orderkey", "INTEGER"),
            col_def("l_partkey", "INTEGER"),
            col_def("l_suppkey", "INTEGER"),
            col_def("l_linenumber", "INTEGER"),
            col_def("l_quantity", "REAL"),
            col_def("l_extendedprice", "REAL"),
            col_def("l_discount", "REAL"),
            col_def("l_tax", "REAL"),
            col_def("l_returnflag", "TEXT"),
            col_def("l_linestatus", "TEXT"),
            col_def("l_shipdate", "TEXT"),
            col_def("l_commitdate", "TEXT"),
            col_def("l_receiptdate", "TEXT"),
            col_def("l_shipinstruct", "TEXT"),
            col_def("l_shipmode", "TEXT"),
            col_def("l_comment", "TEXT"),
        ],
        _ => panic!("Unknown table: {} with {} cols", name, num_cols),
    };

    TableInfo {
        name: name.to_string(),
        columns,
        table_foreign_keys: None,
    }
}

fn parse_row(line: &str, num_cols: usize) -> Vec<Value> {
    let fields: Vec<&str> = line.split('|').collect();
    let mut values = Vec::new();

    for i in 0..num_cols {
        if i >= fields.len() {
            values.push(Value::Null);
            continue;
        }
        let field = fields[i].trim();
        if field.is_empty() {
            values.push(Value::Null);
        } else if let Ok(i) = field.parse::<i64>() {
            values.push(Value::Integer(i));
        } else if let Ok(f) = field.parse::<f64>() {
            values.push(Value::Float(f));
        } else {
            values.push(Value::Text(field.to_string()));
        }
    }
    values
}

fn create_and_import(binary: &BinaryTableStorage, table: &str, path: &PathBuf, num_cols: usize) {
    println!("Importing {}...", table);
    let start = Instant::now();

    let file = File::open(path).expect(&format!("Failed to open {:?}", path));
    let reader = BufReader::new(file);

    let mut rows: Vec<Vec<Value>> = Vec::new();
    let mut total_rows = 0usize;

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let row = parse_row(&line, num_cols);
        rows.push(row);
        total_rows += 1;
    }

    let table_info = create_table_info(table, num_cols);
    let table_data = TableData {
        info: table_info,
        rows,
    };

    binary.save(table, &table_data).expect("Failed to save");

    println!(
        "  {}: {} rows in {:.2}s ({:.0} rows/s)",
        table,
        total_rows,
        start.elapsed().as_secs_f64(),
        total_rows as f64 / start.elapsed().as_secs_f64()
    );
}
