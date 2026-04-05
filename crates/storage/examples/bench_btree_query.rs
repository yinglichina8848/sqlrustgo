//! B+Tree Query Performance Benchmark

use sqlrustgo_storage::{FileStorage, binary_storage::BinaryTableStorage};
use sqlrustgo_storage::bplus_tree::index::BTreeIndex;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let json_dir = PathBuf::from("/tmp/tpch_import_test");
    let binary_dir = PathBuf::from("/tmp/binary_storage");
    
    println!("==============================================");
    println!("  B+Tree Query Performance");
    println!("==============================================");
    
    // Setup
    let storage = FileStorage::new(json_dir).expect("Failed to open JSON");
    let table = storage.get_table("lineitem").expect("No table").clone();
    drop(storage);
    
    let binary = BinaryTableStorage::new(binary_dir).expect("Failed to create binary");
    binary.save("lineitem", &table).expect("Failed to save binary");
    let table = binary.load("lineitem").expect("Failed to load binary");
    
    println!("Loaded {} rows\n", table.rows.len());
    
    // Build index on l_orderkey
    let mut index = BTreeIndex::new();
    for (i, row) in table.rows.iter().enumerate() {
        if let sqlrustgo_types::Value::Integer(key) = row[0] {
            index.insert(key, i as u32);
        }
    }
    println!("Index built\n");
    
    // Test: exact key lookup
    let test_key = 12345i64;
    
    println!("[1] Point Query: key = {}", test_key);
    let scan_start = Instant::now();
    let mut scan_count = 0;
    for row in &table.rows {
        if let sqlrustgo_types::Value::Integer(key) = row[0] {
            if key == test_key { scan_count += 1; }
        }
    }
    let scan_time = scan_start.elapsed();
    
    let index_start = Instant::now();
    let index_count = index.search_all(test_key).len();
    let index_time = index_start.elapsed();
    
    println!("  Full Scan: {} rows in {:.4}ms", scan_count, scan_time.as_secs_f64() * 1000.0);
    println!("  B+Tree:   {} rows in {:.4}ms", index_count, index_time.as_secs_f64() * 1000.0);
    
    // Aggregate query (SUM)
    println!("\n[2] Aggregate: SUM(l_quantity) for key = {}", test_key);
    let scan_start = Instant::now();
    let mut scan_sum = 0.0;
    for row in &table.rows {
        if let sqlrustgo_types::Value::Integer(key) = row[0] {
            if key == test_key {
                if let sqlrustgo_types::Value::Float(qty) = row[4] {
                    scan_sum += qty;
                }
            }
        }
    }
    let scan_time = scan_start.elapsed();
    println!("  Full Scan: sum = {:.2} in {:.4}ms", scan_sum, scan_time.as_secs_f64() * 1000.0);
    
    // COUNT(*) query
    println!("\n[3] COUNT(*) for all rows");
    let scan_start = Instant::now();
    let total_count = table.rows.len();
    let scan_time = scan_time;
    println!("  Full Scan: {} rows in {:.4}ms", total_count, scan_time.as_secs_f64() * 1000.0);
    
    // Summary
    println!("\n==============================================");
    println!("  Summary");
    println!("==============================================");
    println!("Point Query (B+Tree): {:.0}x faster", 
             scan_time.as_secs_f64() / index_time.as_secs_f64());
    println!("Full Table Scan: {:.0} rows/s", 
             table.rows.len() as f64 / scan_time.as_secs_f64());
    println!("==============================================");
}
