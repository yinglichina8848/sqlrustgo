//! TPC-H Data Import Test
//! 
//! Imports SF=0.1 TPC-H data (60K lineitems) into FileStorage
//! to verify disk persistence with real data.

use sqlrustgo_storage::{FileStorage, StorageEngine, TableInfo, ColumnDefinition};
use sqlrustgo_types::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let data_dir = PathBuf::from("/tmp/tpch_import_test_sf1");
    let tpch_dir = PathBuf::from("/Users/liying/workspace/dev/heartopen/SQLRustGo/data/tpch-sf1-generated");
    
    println!("==============================================");
    println!("  TPC-H Data Import Test (SF=0.1)");
    println!("==============================================");
    println!("Data dir: {:?}", data_dir);
    println!("TPC-H dir: {:?}", tpch_dir);
    
    // Clean up
    let _ = std::fs::remove_dir_all(&data_dir);
    std::fs::create_dir_all(&data_dir).unwrap();
    
    let mut storage = FileStorage::new(data_dir.clone()).expect("Failed to create FileStorage");
    
    // Create TPC-H tables
    create_tables(&mut storage);
    
    // Import data
    let total_start = Instant::now();
    
    import_nation_region(&mut storage, &tpch_dir);
    import_supplier(&mut storage, &tpch_dir);
    import_part(&mut storage, &tpch_dir);
    import_partsupp(&mut storage, &tpch_dir);
    import_customer(&mut storage, &tpch_dir);
    import_orders(&mut storage, &tpch_dir);
    import_lineitem(&mut storage, &tpch_dir);
    
    let total_time = total_start.elapsed();
    
    // Flush to disk
    println!("\nFlushing to disk...");
    let flush_start = Instant::now();
    storage.flush().expect("Failed to flush");
    println!("Flush time: {:.2}s", flush_start.elapsed().as_secs_f64());
    
    // Close storage
    drop(storage);
    
    // Reopen and verify
    println!("\nReopening storage to verify persistence...");
    let mut storage = FileStorage::new(data_dir.clone()).expect("Failed to reopen");
    
    // Count rows in each table
    let tables = ["nation", "region", "supplier", "part", "partsupp", "customer", "orders", "lineitem"];
    let expected_rows = [25, 5, 1000, 20000, 80000, 15000, 150000, 600572];
    
    println!("\nVerification:");
    let mut all_passed = true;
    for (i, table) in tables.iter().enumerate() {
        let rows = storage.scan(table).expect(&format!("Failed to scan {}", table));
        let expected = expected_rows[i];
        let status = if rows.len() >= expected { "✅" } else { "❌" };
        println!("  {}: {} rows (expected {}) {}", table, rows.len(), expected, status);
        if rows.len() < expected {
            all_passed = false;
        }
    }
    
    drop(storage);
    
    println!("\n==============================================");
    if all_passed {
        println!("  ✅ ALL TESTS PASSED!");
        println!("  Total import time: {:.2}s", total_time.as_secs_f64());
        println!("  {:.0} rows/s", 866602 as f64 / total_time.as_secs_f64());
    } else {
        println!("  ❌ SOME TESTS FAILED!");
        std::process::exit(1);
    }
    println!("==============================================");
}

fn create_tables(storage: &mut FileStorage) {
    // Nation
    storage.create_table(&TableInfo {
        name: "nation".to_string(),
        columns: vec![
            col_def("n_nationkey", "INTEGER"),
            col_def("n_name", "TEXT"),
            col_def("n_regionkey", "INTEGER"),
            col_def("n_comment", "TEXT"),
        ],
    }).unwrap();
    
    // Region
    storage.create_table(&TableInfo {
        name: "region".to_string(),
        columns: vec![
            col_def("r_regionkey", "INTEGER"),
            col_def("r_name", "TEXT"),
            col_def("r_comment", "TEXT"),
        ],
    }).unwrap();
    
    // Supplier
    storage.create_table(&TableInfo {
        name: "supplier".to_string(),
        columns: vec![
            col_def("s_suppkey", "INTEGER"),
            col_def("s_name", "TEXT"),
            col_def("s_address", "TEXT"),
            col_def("s_nationkey", "INTEGER"),
            col_def("s_phone", "TEXT"),
            col_def("s_acctbal", "REAL"),
            col_def("s_comment", "TEXT"),
        ],
    }).unwrap();
    
    // Part
    storage.create_table(&TableInfo {
        name: "part".to_string(),
        columns: vec![
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
    }).unwrap();
    
    // PartSupp
    storage.create_table(&TableInfo {
        name: "partsupp".to_string(),
        columns: vec![
            col_def("ps_partkey", "INTEGER"),
            col_def("ps_suppkey", "INTEGER"),
            col_def("ps_availqty", "INTEGER"),
            col_def("ps_supplycost", "REAL"),
            col_def("ps_comment", "TEXT"),
        ],
    }).unwrap();
    
    // Customer
    storage.create_table(&TableInfo {
        name: "customer".to_string(),
        columns: vec![
            col_def("c_custkey", "INTEGER"),
            col_def("c_name", "TEXT"),
            col_def("c_address", "TEXT"),
            col_def("c_nationkey", "INTEGER"),
            col_def("c_phone", "TEXT"),
            col_def("c_acctbal", "REAL"),
            col_def("c_mktsegment", "TEXT"),
            col_def("c_comment", "TEXT"),
        ],
    }).unwrap();
    
    // Orders
    storage.create_table(&TableInfo {
        name: "orders".to_string(),
        columns: vec![
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
    }).unwrap();
    
    // Lineitem
    storage.create_table(&TableInfo {
        name: "lineitem".to_string(),
        columns: vec![
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
    }).unwrap();
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
    }
}

fn parse_row(line: &str, num_cols: usize) -> Vec<Value> {
    let fields: Vec<&str> = line.split('|').collect();
    let mut values = Vec::new();
    
    for (i, field) in fields.iter().enumerate() {
        if i >= num_cols {
            break;
        }
        let field = field.trim();
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

fn import_file(storage: &mut FileStorage, table: &str, path: &PathBuf, num_cols: usize, batch_size: usize) {
    println!("Importing {}...", table);
    let start = Instant::now();
    
    let file = File::open(path).expect(&format!("Failed to open {:?}", path));
    let reader = BufReader::new(file);
    
    let mut batch = Vec::new();
    let mut total_rows = 0;
    
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let row = parse_row(&line, num_cols);
        batch.push(row);
        total_rows += 1;
        
        if batch.len() >= batch_size {
            storage.insert(table, batch.drain(..).collect::<Vec<_>>()).expect("Failed to insert");
            if total_rows % 100000 == 0 {
                println!("  {}: {} rows...", table, total_rows);
            }
        }
    }
    
    // Insert remaining
    if !batch.is_empty() {
        storage.insert(table, batch).expect("Failed to insert");
    }
    
    println!("  {}: {} rows in {:.2}s ({:.0} rows/s)", 
             table, total_rows, start.elapsed().as_secs_f64(), 
             total_rows as f64 / start.elapsed().as_secs_f64());
}

fn import_nation_region(storage: &mut FileStorage, dir: &PathBuf) {
    import_file(storage, "nation", &dir.join("nation.tbl"), 4, 1000);
    import_file(storage, "region", &dir.join("region.tbl"), 3, 100);
}

fn import_supplier(storage: &mut FileStorage, dir: &PathBuf) {
    import_file(storage, "supplier", &dir.join("supplier.tbl"), 7, 1000);
}

fn import_part(storage: &mut FileStorage, dir: &PathBuf) {
    import_file(storage, "part", &dir.join("part.tbl"), 9, 5000);
}

fn import_partsupp(storage: &mut FileStorage, dir: &PathBuf) {
    import_file(storage, "partsupp", &dir.join("partsupp.tbl"), 5, 10000);
}

fn import_customer(storage: &mut FileStorage, dir: &PathBuf) {
    import_file(storage, "customer", &dir.join("customer.tbl"), 8, 5000);
}

fn import_orders(storage: &mut FileStorage, dir: &PathBuf) {
    import_file(storage, "orders", &dir.join("orders.tbl"), 9, 10000);
}

fn import_lineitem(storage: &mut FileStorage, dir: &PathBuf) {
    import_file(storage, "lineitem", &dir.join("lineitem.tbl"), 16, 50000);
}
