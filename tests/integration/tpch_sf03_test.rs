//! TPC-H SF=0.3 Simple Query Test
//! Run with: cargo test --test tpch_sf03_test -- --nocapture

use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use std::path::Path;
use std::sync::{Arc, RwLock};

const TPCK_DATA_DIR: &str = "data/tpch-sf03";

fn create_engine() -> ExecutionEngine {
    ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
}

fn setup_schema(engine: &mut ExecutionEngine) {
    engine.execute(parse("CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER, l_linenumber INTEGER, l_quantity INTEGER, l_extendedprice REAL, l_discount REAL, l_tax REAL, l_returnflag TEXT, l_linestatus TEXT, l_shipdate TEXT, l_commitdate TEXT, l_receiptdate TEXT, l_shipinstruct TEXT, l_shipmode TEXT, l_comment TEXT)").unwrap()).unwrap();
}

fn setup_sqlrustgo_engine_sf03_lineitem() -> ExecutionEngine {
    let mut engine = create_engine();
    setup_schema(&mut engine);
    
    let filepath = format!("{}/lineitem.tbl", TPCK_DATA_DIR);
    if Path::new(&filepath).exists() {
        let mut storage = engine.storage.write().unwrap();
        match storage.bulk_load_tbl_file("lineitem", &filepath) {
            Ok(count) => println!("Loaded lineitem: {} rows", count),
            Err(e) => println!("Failed to load: {:?}", e),
        }
    }
    
    engine
}

#[test]
fn test_sqlrustgo_sf03_count() {
    let mut engine = setup_sqlrustgo_engine_sf03_lineitem();
    
    // COUNT(*) query
    println!("\nCOUNT(*) query (SF=0.3, 1.8M rows):");
    let start = std::time::Instant::now();
    let result = engine.execute(parse("SELECT COUNT(*) FROM lineitem").unwrap());
    let elapsed = start.elapsed();
    
    match result {
        Ok(rows) => {
            println!("COUNT(*): {:?} rows in {:?}", rows.rows.len(), elapsed);
            for row in &rows.rows {
                println!("  {:?}", row);
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
    
    // Run 10 iterations for performance test
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let _ = engine.execute(parse("SELECT COUNT(*) FROM lineitem").unwrap());
    }
    let total = start.elapsed();
    let avg = total / 10;
    println!("COUNT(*) 10 iterations: {:?} total, {:?} avg", total, avg);
}

#[test]
fn test_sqlrustgo_sf03_sum_filtered() {
    let mut engine = setup_sqlrustgo_engine_sf03_lineitem();
    
    // SUM with filter
    println!("\nSUM(l_quantity) with filter (SF=0.3):");
    let start = std::time::Instant::now();
    let result = engine.execute(parse("SELECT SUM(l_quantity) FROM lineitem WHERE l_quantity < 10").unwrap());
    let elapsed = start.elapsed();
    
    match result {
        Ok(rows) => {
            println!("SUM: {:?} in {:?}", rows.rows, elapsed);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
    
    // Run 10 iterations
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let _ = engine.execute(parse("SELECT SUM(l_quantity) FROM lineitem WHERE l_quantity < 10").unwrap());
    }
    let total = start.elapsed();
    let avg = total / 10;
    println!("SUM 10 iterations: {:?} total, {:?} avg", total, avg);
}
