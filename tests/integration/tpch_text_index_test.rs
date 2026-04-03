//! Test TEXT field index
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use std::path::Path;
use std::sync::{Arc, RwLock};

#[test]
fn test_text_index() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    
    // Create table with TEXT column
    engine.execute(parse("CREATE TABLE orders (o_orderkey INTEGER, o_custkey INTEGER, o_orderstatus TEXT, o_totalprice REAL, o_orderdate TEXT)").unwrap()).unwrap();
    
    // Create index on TEXT column
    engine.execute(parse("CREATE INDEX idx_status ON orders(o_orderstatus)").unwrap()).unwrap();
    
    // Insert some data
    for i in 0..1000 {
        let status = if i % 2 == 0 { "P" } else { "O" };
        engine.execute(parse(&format!(
            "INSERT INTO orders VALUES ({}, {}, '{}', 100.0, '1998-01-01')",
            i, i, status
        )).unwrap()).unwrap();
    }
    
    // Test TEXT = query (should use index)
    println!("\nTest: WHERE o_orderstatus = 'P'");
    let start = std::time::Instant::now();
    match engine.execute(parse("SELECT COUNT(*) FROM orders WHERE o_orderstatus = 'P'").unwrap()) {
        Ok(result) => {
            let elapsed = start.elapsed();
            println!("  Result: {:?} in {:?}", result.rows, elapsed);
        }
        Err(e) => println!("  Error: {:?}", e),
    }
    
    // Test with bulk load
    println!("\nTest: Bulk load with TEXT index");
    drop(engine);
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    engine.execute(parse("CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER, l_linenumber INTEGER, l_quantity INTEGER, l_extendedprice REAL, l_discount REAL, l_tax REAL, l_returnflag TEXT, l_linestatus TEXT, l_shipdate TEXT, l_commitdate TEXT, l_receiptdate TEXT, l_shipinstruct TEXT, l_shipmode TEXT, l_comment TEXT)").unwrap()).unwrap();
    
    let filepath = "data/tpch-sf03/lineitem.tbl";
    if Path::new(&filepath).exists() {
        let mut storage = engine.storage.write().unwrap();
        match storage.bulk_load_tbl_file("lineitem", &filepath) {
            Ok(count) => {
                println!("  Loaded {} rows", count);
                
                // Test TEXT = query (l_returnflag = 'R')
                println!("\nTest: WHERE l_returnflag = 'R'");
                let start = std::time::Instant::now();
                drop(storage);
                match engine.execute(parse("SELECT COUNT(*) FROM lineitem WHERE l_returnflag = 'R'").unwrap()) {
                    Ok(result) => {
                        let elapsed = start.elapsed();
                        println!("  Result: {:?} in {:?}", result.rows, elapsed);
                    }
                    Err(e) => println!("  Error: {:?}", e),
                }
                
                // Test TEXT range (should NOT use index)
                println!("\nTest: WHERE l_shipdate <= '1998-09-02' (TEXT range)");
                let start = std::time::Instant::now();
                match engine.execute(parse("SELECT COUNT(*) FROM lineitem WHERE l_shipdate <= '1998-09-02'").unwrap()) {
                    Ok(result) => {
                        let elapsed = start.elapsed();
                        println!("  Result: {:?} in {:?}", result.rows, elapsed);
                    }
                    Err(e) => println!("  Error: {:?}", e),
                }
            }
            Err(e) => println!("  Load error: {:?}", e),
        }
    }
}
