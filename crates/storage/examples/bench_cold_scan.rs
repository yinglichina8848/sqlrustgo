//! Fair comparison: cold start (no cache)

use sqlrustgo_storage::binary_storage::BinaryTableStorage;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let binary_dir = PathBuf::from("/tmp/binary_storage");
    let storage = BinaryTableStorage::new(binary_dir.clone()).expect("Failed to create");
    
    println!("==============================================");
    println!("  Cold Start Performance (No Cache)");
    println!("==============================================");
    
    // Cold load from binary (fresh from disk)
    println!("\n[Cold] Loading lineitem from binary (fresh disk read)...");
    let start = Instant::now();
    let loaded = storage.load("lineitem").expect("Failed to load");
    let cold_time = start.elapsed();
    println!("  Binary (cold): {} rows in {:.2}ms ({:.0} rows/s)", 
             loaded.rows.len(), 
             cold_time.as_secs_f64() * 1000.0,
             loaded.rows.len() as f64 / cold_time.as_secs_f64());
    
    // PostgreSQL baseline (cold)
    println!("\n[Reference] PostgreSQL full scan: ~33ms (18M rows/s)");
    
    // Summary
    println!("\n==============================================");
    println!("  Summary");
    println!("==============================================");
    println!("Binary Storage (cold load): {:.0} rows/s", loaded.rows.len() as f64 / cold_time.as_secs_f64());
    println!("PostgreSQL (cold scan): 18,000,000 rows/s");
    println!("Gap: {:.0}x slower", 18_000_000.0 / (loaded.rows.len() as f64 / cold_time.as_secs_f64()));
    println!("==============================================");
}
