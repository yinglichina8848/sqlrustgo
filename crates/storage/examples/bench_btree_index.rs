//! Benchmark: Full Table Scan vs B+Tree Index

use sqlrustgo_storage::{FileStorage, binary_storage::BinaryTableStorage};
use sqlrustgo_storage::bplus_tree::index::BTreeIndex;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let json_dir = PathBuf::from("/tmp/tpch_import_test");
    let binary_dir = PathBuf::from("/tmp/binary_storage");
    
    println!("==============================================");
    println!("  B+Tree Index vs Full Table Scan Benchmark");
    println!("==============================================");
    
    // Load from JSON and save as binary
    println!("\n[Setup] Loading from JSON...");
    let storage = FileStorage::new(json_dir.clone()).expect("Failed to open JSON");
    let table = storage.get_table("lineitem").expect("No table").clone();
    let rows_count = table.rows.len();
    drop(storage);
    
    println!("[Setup] Saving as binary...");
    let binary = BinaryTableStorage::new(binary_dir.clone()).expect("Failed to create binary");
    binary.save("lineitem", &table).expect("Failed to save binary");
    println!("  {} rows saved\n", rows_count);
    
    // Load from binary
    println!("[Benchmark] Loading from binary...");
    let start = Instant::now();
    let table = binary.load("lineitem").expect("Failed to load binary");
    let load_time = start.elapsed();
    println!("  Loaded {} rows in {:.2}ms\n", table.rows.len(), load_time.as_secs_f64() * 1000.0);
    
    // Build index on l_orderkey (column 0)
    println!("Building B+Tree index on column 0 (l_orderkey)...");
    let mut index = BTreeIndex::new();
    let index_start = Instant::now();
    for (i, row) in table.rows.iter().enumerate() {
        if let sqlrustgo_types::Value::Integer(key) = row[0] {
            index.insert(key, i as u32);
        }
    }
    let index_time = index_start.elapsed();
    println!("  Index built in {:.2}ms", index_time.as_secs_f64() * 1000.0);
    
    // Benchmark: Full table scan for specific key
    let test_key = 1000i64;
    
    println!("\n[1] Full Table Scan for key = {}", test_key);
    let scan_start = Instant::now();
    let mut scan_count = 0;
    for row in &table.rows {
        if let sqlrustgo_types::Value::Integer(key) = row[0] {
            if key == test_key {
                scan_count += 1;
            }
        }
    }
    let scan_time = scan_start.elapsed();
    println!("  Found {} rows in {:.4}ms", scan_count, scan_time.as_secs_f64() * 1000.0);
    
    println!("\n[2] B+Tree Index lookup for key = {}", test_key);
    let index_start = Instant::now();
    let index_result = index.search(test_key);
    let index_lookup_time = index_start.elapsed();
    println!("  Found {} rows in {:.4}ms", if index_result.is_some() { 1 } else { 0 }, index_lookup_time.as_secs_f64() * 1000.0);
    
    // Summary
    println!("\n==============================================");
    println!("  Summary");
    println!("==============================================");
    println!("{:<30} {:>12}", "Method", "Time (ms)");
    println!("----------------------------------------");
    println!("{:<30} {:>12.4}", "Full Table Scan (lookup)", scan_time.as_secs_f64() * 1000.0);
    println!("{:<30} {:>12.4}", "B+Tree Index Lookup", index_lookup_time.as_secs_f64() * 1000.0);
    
    if index_lookup_time < scan_time {
        let speedup = scan_time.as_secs_f64() / index_lookup_time.as_secs_f64();
        println!("\n✅ B+Tree is {:.0}x FASTER for point lookup!", speedup);
    } else {
        println!("\n⚠️  B+Tree slower (overhead for small result sets)");
    }
    println!("==============================================");
}
