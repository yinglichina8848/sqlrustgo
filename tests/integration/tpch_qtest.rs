//! TPC-H Q1-Q6 Performance Test
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::fs::File;
use std::io::Write;

fn main() {
    let data_dir = "data/tpch-sf01";
    let mut results = String::new();
    
    results.push_str("=== SF=0.1 ===\n");
    
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    engine.execute(parse("CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER, l_linenumber INTEGER, l_quantity INTEGER, l_extendedprice REAL, l_discount REAL, l_tax REAL, l_returnflag TEXT, l_linestatus TEXT, l_shipdate TEXT, l_commitdate TEXT, l_receiptdate TEXT, l_shipinstruct TEXT, l_shipmode TEXT, l_comment TEXT)").unwrap()).unwrap();
    
    let filepath = format!("{}/lineitem.tbl", data_dir);
    if Path::new(&filepath).exists() {
        let mut storage = engine.storage.write().unwrap();
        match storage.bulk_load_tbl_file("lineitem", &filepath) {
            Ok(count) => results.push_str(&format!("Loaded {} rows\n", count)),
            Err(e) => results.push_str(&format!("Error loading: {:?}\n", e)),
        }
    }
    
    // Q1
    let start = std::time::Instant::now();
    let r = engine.execute(parse("SELECT COUNT(*) FROM lineitem WHERE l_shipdate <= '1998-09-02'").unwrap());
    let elapsed = start.elapsed();
    match r {
        Ok(result) => results.push_str(&format!("Q1: {} rows in {:?}\n", result.rows.len(), elapsed)),
        Err(e) => results.push_str(&format!("Q1 Error: {:?}\n", e)),
    }
    
    // Q6
    let start = std::time::Instant::now();
    let r = engine.execute(parse("SELECT COUNT(*) FROM lineitem WHERE l_shipdate >= '1994-01-01' AND l_shipdate < '1995-01-01' AND l_discount >= 0.05 AND l_discount <= 0.07 AND l_quantity < 24").unwrap());
    let elapsed = start.elapsed();
    match r {
        Ok(result) => results.push_str(&format!("Q6: {} rows in {:?}\n", result.rows.len(), elapsed)),
        Err(e) => results.push_str(&format!("Q6 Error: {:?}\n", e)),
    }
    
    // Write to file
    let mut f = File::create("/tmp/sf01_results.txt").unwrap();
    f.write_all(results.as_bytes()).unwrap();
    println!("SF=0.1 done");
    
    // SF=0.3
    let mut results = String::new();
    results.push_str("=== SF=0.3 ===\n");
    
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    engine.execute(parse("CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER, l_linenumber INTEGER, l_quantity INTEGER, l_extendedprice REAL, l_discount REAL, l_tax REAL, l_returnflag TEXT, l_linestatus TEXT, l_shipdate TEXT, l_commitdate TEXT, l_receiptdate TEXT, l_shipinstruct TEXT, l_shipmode TEXT, l_comment TEXT)").unwrap()).unwrap();
    
    let filepath = "data/tpch-sf03/lineitem.tbl";
    if Path::new(&filepath).exists() {
        let mut storage = engine.storage.write().unwrap();
        match storage.bulk_load_tbl_file("lineitem", &filepath) {
            Ok(count) => results.push_str(&format!("Loaded {} rows\n", count)),
            Err(e) => results.push_str(&format!("Error loading: {:?}\n", e)),
        }
    }
    
    // Q1
    let start = std::time::Instant::now();
    let r = engine.execute(parse("SELECT COUNT(*) FROM lineitem WHERE l_shipdate <= '1998-09-02'").unwrap());
    let elapsed = start.elapsed();
    match r {
        Ok(result) => results.push_str(&format!("Q1: {} rows in {:?}\n", result.rows.len(), elapsed)),
        Err(e) => results.push_str(&format!("Q1 Error: {:?}\n", e)),
    }
    
    // Q6
    let start = std::time::Instant::now();
    let r = engine.execute(parse("SELECT COUNT(*) FROM lineitem WHERE l_shipdate >= '1994-01-01' AND l_shipdate < '1995-01-01' AND l_discount >= 0.05 AND l_discount <= 0.07 AND l_quantity < 24").unwrap());
    let elapsed = start.elapsed();
    match r {
        Ok(result) => results.push_str(&format!("Q6: {} rows in {:?}\n", result.rows.len(), elapsed)),
        Err(e) => results.push_str(&format!("Q6 Error: {:?}\n", e)),
    }
    
    let mut f = File::create("/tmp/sf03_results.txt").unwrap();
    f.write_all(results.as_bytes()).unwrap();
    println!("SF=0.3 done");
}
