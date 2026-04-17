//! TPC-H Binary Storage Benchmark
//!
//! Uses BinaryTableStorage for fast data loading and runs TPC-H Q1-Q22.
//!
//! Usage:
//!   cargo run --example tpch_binary_benchmark -p sqlrustgo-storage

use sqlrustgo::{parse, ExecutionEngine};
use sqlrustgo_storage::binary_storage::BinaryTableStorage;
use sqlrustgo_storage::StorageEngine;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Instant;

fn main() {
    let binary_dir = PathBuf::from("/tmp/tpch_binary");

    println!("==============================================");
    println!("  TPC-H Binary Storage Benchmark");
    println!("==============================================");

    // Load from binary
    println!("Loading from binary storage...");
    let start = Instant::now();
    let binary = BinaryTableStorage::new(binary_dir).expect("Failed to open binary storage");

    let mut storage = sqlrustgo_storage::FileStorage::new(PathBuf::from("/tmp/tpch_sqlrustgo"))
        .expect("Failed to create FileStorage");

    // Copy data from binary to FileStorage for query execution
    for table in [
        "nation", "region", "supplier", "part", "partsupp", "customer", "orders", "lineitem",
    ] {
        println!("  Loading {}...", table);
        let data = binary.load(table).expect("Failed to load");
        storage
            .create_table(&data.info)
            .expect("Failed to create table");
        storage
            .insert_table(table.to_string(), data)
            .expect("Failed to insert");
    }

    let load_time = start.elapsed();
    println!("Load time: {:.2}s", load_time.as_secs_f64());

    let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

    // TPC-H Q1
    println!("\nRunning Q1...");
    let sql = "SELECT l_returnflag, SUM(l_quantity) FROM lineitem WHERE l_shipdate <= '1995-12-01' GROUP BY l_returnflag";
    let start = Instant::now();
    let result = engine.execute(parse(sql).unwrap()).unwrap();
    let elapsed = start.elapsed();
    println!(
        "  Q1: {} rows in {:.2}ms",
        result.rows.len(),
        elapsed.as_secs_f64() * 1000.0
    );

    // TPC-H Q6
    println!("\nRunning Q6...");
    let sql = "SELECT SUM(l_extendedprice * l_discount) AS revenue FROM lineitem WHERE l_shipdate >= '1994-01-01' AND l_shipdate < '1995-01-01' AND l_discount BETWEEN 0.06 AND 0.08 AND l_quantity < 25";
    let start = Instant::now();
    let result = engine.execute(parse(sql).unwrap()).unwrap();
    let elapsed = start.elapsed();
    println!(
        "  Q6: {} rows in {:.2}ms",
        result.rows.len(),
        elapsed.as_secs_f64() * 1000.0
    );

    // TPC-H Q10
    println!("\nRunning Q10...");
    let sql = "SELECT c_custkey, SUM(l_extendedprice) FROM customer, orders, lineitem WHERE c_custkey = o_custkey AND l_orderkey = o_orderkey AND o_orderdate >= '1993-10-01' GROUP BY c_custkey";
    let start = Instant::now();
    let result = engine.execute(parse(sql).unwrap()).unwrap();
    let elapsed = start.elapsed();
    println!(
        "  Q10: {} rows in {:.2}ms",
        result.rows.len(),
        elapsed.as_secs_f64() * 1000.0
    );

    println!("\n==============================================");
}
