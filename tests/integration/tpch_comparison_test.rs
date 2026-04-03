//! TPC-H Q1-Q6 Performance Comparison Test
//! Run with: cargo test --test tpch_comparison_test -- --nocapture --ignored

use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use std::path::Path;
use std::sync::{Arc, RwLock};

fn setup_engine(data_dir: &str) -> Option<ExecutionEngine> {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    
    // Create schema
    engine.execute(parse("CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER, l_linenumber INTEGER, l_quantity INTEGER, l_extendedprice REAL, l_discount REAL, l_tax REAL, l_returnflag TEXT, l_linestatus TEXT, l_shipdate TEXT, l_commitdate TEXT, l_receiptdate TEXT, l_shipinstruct TEXT, l_shipmode TEXT, l_comment TEXT)").unwrap()).ok()?;
    
    let filepath = format!("{}/lineitem.tbl", data_dir);
    if Path::new(&filepath).exists() {
        let mut storage = engine.storage.write().unwrap();
        if let Ok(count) = storage.bulk_load_tbl_file("lineitem", &filepath) {
            eprintln!("Loaded {} rows", count);
        }
    } else {
        eprintln!("Data file not found: {}", filepath);
        return None;
    }
    
    Some(engine)
}

fn run_query(engine: &mut ExecutionEngine, sql: &str, name: &str) {
    let start = std::time::Instant::now();
    match engine.execute(parse(sql).unwrap()) {
        Ok(result) => {
            let elapsed = start.elapsed();
            eprintln!("{}: {} rows in {:?}", name, result.rows.len(), elapsed);
        }
        Err(e) => {
            let elapsed = start.elapsed();
            eprintln!("{}: ERROR in {:?} - {:?}", name, elapsed, e);
        }
    }
}

#[test]
fn test_sf01_q1_q6() {
    eprintln!("\n=== SF=0.1 Performance ===");
    if let Some(mut engine) = setup_engine("data/tpch-sf01") {
        run_query(&mut engine, "SELECT COUNT(*) FROM lineitem WHERE l_shipdate <= '1998-09-02'", "Q1: COUNT filtered");
        run_query(&mut engine, "SELECT COUNT(*) FROM lineitem WHERE l_shipdate >= '1994-01-01' AND l_shipdate < '1995-01-01' AND l_discount >= 0.05 AND l_discount <= 0.07 AND l_quantity < 24", "Q6: COUNT complex filter");
    }
}

#[test]
#[ignore]
fn test_sf03_q1_q6() {
    eprintln!("\n=== SF=0.3 Performance ===");
    if let Some(mut engine) = setup_engine("data/tpch-sf03") {
        run_query(&mut engine, "SELECT COUNT(*) FROM lineitem WHERE l_shipdate <= '1998-09-02'", "Q1: COUNT filtered");
        run_query(&mut engine, "SELECT COUNT(*) FROM lineitem WHERE l_shipdate >= '1994-01-01' AND l_shipdate < '1995-01-01' AND l_discount >= 0.05 AND l_discount <= 0.07 AND l_quantity < 24", "Q6: COUNT complex filter");
    }
}
