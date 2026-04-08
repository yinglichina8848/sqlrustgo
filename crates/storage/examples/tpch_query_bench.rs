//! TPC-H Query Benchmark on FileStorage
//!
//! Runs TPC-H queries on the imported SF=0.1 data and reports performance.

use sqlrustgo::{parse, ExecutionEngine};
use sqlrustgo_storage::FileStorage;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Instant;

fn main() {
    let data_dir = PathBuf::from("/tmp/tpch_import_test");
    
    println!("==============================================");
    println!("  TPC-H Query Benchmark (SF=0.1)");
    println!("==============================================");
    println!("Data dir: {:?}", data_dir);
    
    // Check if data exists
    let storage = match FileStorage::new(data_dir.clone()) {
        Ok(s) => s,
        Err(e) => {
            println!("❌ Failed to open FileStorage: {:?}", e);
            println!("Please run: cargo run -p sqlrustgo-storage --example tpch_import");
            std::process::exit(1);
        }
    };
    drop(storage);
    
    // Run queries
    let queries = vec![
        ("Q1", "SELECT l_returnflag, SUM(l_quantity) FROM lineitem GROUP BY l_returnflag"),
        ("Q2", "SELECT s_acctbal, s_name, n_name, p_partkey FROM part, supplier, partsupp, nation, region WHERE p_partkey = ps_partkey AND s_suppkey = ps_suppkey AND p_size = 15 AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'EUROPE' ORDER BY s_acctbal DESC LIMIT 10"),
        ("Q3", "SELECT o_orderkey, SUM(l_extendedprice) FROM orders JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate < '1995-03-15' GROUP BY o_orderkey"),
        ("Q4", "SELECT o_orderpriority, COUNT(*) FROM orders WHERE o_orderdate >= '1993-07-01' AND o_orderdate < '1993-10-01' GROUP BY o_orderpriority"),
        ("Q5", "SELECT n_name, SUM(l_extendedprice) FROM customer, orders, lineitem, supplier, nation, region WHERE c_custkey = o_custkey AND l_orderkey = o_orderkey AND l_suppkey = s_suppkey AND c_nationkey = s_nationkey AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'ASIA' GROUP BY n_name"),
        ("Q6", "SELECT SUM(l_extendedprice) FROM lineitem WHERE l_quantity < 24 AND l_shipdate >= '1994-01-01'"),
    ];
    
    println!("\nRunning {} queries...\n", queries.len());
    
    let mut results = Vec::new();
    let total_start = Instant::now();
    
    for (name, sql) in &queries {
        println!("Running {}...", name);
        
        let storage = Arc::new(RwLock::new(FileStorage::new(data_dir.clone()).expect("Failed to open storage")));
        let mut engine = ExecutionEngine::new(storage.clone());
        
        let start = Instant::now();
        let result = engine.execute(parse(sql).expect("Failed to parse SQL"));
        let elapsed = start.elapsed();
        
        let rows = result.as_ref().map(|r| r.rows().len()).unwrap_or(0);
        let status = if result.is_ok() { "✅" } else { "❌" };
        
        println!("  {} -> {} rows in {:.2}ms {}", name, rows, elapsed.as_secs_f64() * 1000.0, status);
        
        if let Err(e) = result {
            println!("  Error: {:?}", e);
        }
        
        results.push((name.clone(), rows, elapsed, result.is_ok()));
    }
    
    let total_elapsed = total_start.elapsed();
    
    println!("\n==============================================");
    println!("  Benchmark Summary");
    println!("==============================================");
    println!("{:<6} {:>10} {:>12} {}", "Query", "Rows", "Time (ms)", "Status");
    println!("----------------------------------------");
    
    for (name, rows, elapsed, ok) in &results {
        let status = if *ok { "✅" } else { "❌" };
        println!("{:<6} {:>10} {:>12.2} {}", name, rows, elapsed.as_secs_f64() * 1000.0, status);
    }
    
    println!("----------------------------------------");
    println!("Total time: {:.2}s", total_elapsed.as_secs_f64());
    
    let avg_ms: f64 = results.iter().map(|(_, _, e, _)| e.as_secs_f64() * 1000.0).sum::<f64>() / results.len() as f64;
    println!("Average query time: {:.2}ms", avg_ms);
    println!("==============================================");
}
