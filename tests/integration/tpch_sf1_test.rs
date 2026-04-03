//! TPC-H SF=1 Real Data Test - Simple queries only
//! Run with: cargo test --test tpch_sf1_test -- --nocapture --ignored
//!
//! NOTE: SF=1 requires ~5GB memory. Only run on machines with 16GB+ RAM.

use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use std::path::Path;
use std::sync::{Arc, RwLock};

const TPCK_DATA_DIR: &str = "data/tpch-sf1";

fn create_engine() -> ExecutionEngine {
    ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
}

fn setup_schema(engine: &mut ExecutionEngine) {
    engine.execute(parse("CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER, l_linenumber INTEGER, l_quantity INTEGER, l_extendedprice REAL, l_discount REAL, l_tax REAL, l_returnflag TEXT, l_linestatus TEXT, l_shipdate TEXT, l_commitdate TEXT, l_receiptdate TEXT, l_shipinstruct TEXT, l_shipmode TEXT, l_comment TEXT)").unwrap()).unwrap();
}

fn setup_sqlrustgo_engine_sf1_lineitem() -> ExecutionEngine {
    let mut engine = create_engine();
    setup_schema(&mut engine);
    
    let filepath = format!("{}/lineitem.tbl", TPCK_DATA_DIR);
    if Path::new(&filepath).exists() {
        let mut storage = engine.storage.write().unwrap();
        match storage.bulk_load_tbl_file("lineitem", &filepath) {
            Ok(count) => println!("Loaded lineitem: {} rows", count),
            Err(e) => println!("Failed to load: {:?}", e),
        }
    } else {
        println!("SF=1 data not found at {}. Run: cd /tmp/tpch-dbgen && ./dbgen -s 1 -f && cp *.tbl ~/workspace/yinglichina/sqlrustgo/data/tpch-sf1/", TPCK_DATA_DIR);
    }
    
    engine
}

#[test]
#[ignore] // SF=1 requires ~5GB memory, may OOM on 16GB systems
fn test_sqlrustgo_sf1_count() {
    let mut engine = setup_sqlrustgo_engine_sf1_lineitem();
    
    // COUNT(*) query
    println!("\nCOUNT(*) query (SF=1, 6M rows):");
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
}

#[test]
#[ignore] // SF=1 requires ~5GB memory, may OOM on 16GB systems
fn test_sqlrustgo_sf1_sum_filtered() {
    let mut engine = setup_sqlrustgo_engine_sf1_lineitem();
    
    // SUM with filter
    println!("\nSUM(l_quantity) with filter (SF=1):");
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
}
