//! Compare Binary vs JSON storage performance

use sqlrustgo_storage::binary_storage::BinaryTableStorage;
use sqlrustgo_storage::{FileStorage, StorageEngine};
use sqlrustgo_types::Value;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let data_dir = PathBuf::from("/tmp/tpch_import_test");

    println!("==============================================");
    println!("  Binary vs JSON Storage Performance");
    println!("==============================================");

    // Load from JSON first
    println!("\n[1] Loading from JSON...");
    let start = Instant::now();
    let storage = FileStorage::new(data_dir.clone()).expect("Failed to open FileStorage");
    let rows = storage.scan("lineitem").expect("Failed to scan");
    let json_time = start.elapsed();
    println!(
        "  JSON load: {} rows in {:.2}ms",
        rows.len(),
        json_time.as_secs_f64() * 1000.0
    );

    // Save as binary
    println!("\n[2] Saving as binary...");
    let binary_dir = PathBuf::from("/tmp/binary_storage");
    let binary_storage =
        BinaryTableStorage::new(binary_dir.clone()).expect("Failed to create binary storage");

    // Get table data
    let table_data = storage.get_table("lineitem").expect("No table").clone();
    let rows_count = table_data.rows.len();

    let start = Instant::now();
    binary_storage
        .save("lineitem", &table_data)
        .expect("Failed to save binary");
    let binary_save_time = start.elapsed();
    println!(
        "  Binary save: {} rows in {:.2}ms",
        rows_count,
        binary_save_time.as_secs_f64() * 1000.0
    );

    // Load from binary
    println!("\n[3] Loading from binary...");
    let binary_storage =
        BinaryTableStorage::new(binary_dir).expect("Failed to open binary storage");
    let start = Instant::now();
    let loaded = binary_storage
        .load("lineitem")
        .expect("Failed to load binary");
    let binary_load_time = start.elapsed();
    println!(
        "  Binary load: {} rows in {:.2}ms",
        loaded.rows.len(),
        binary_load_time.as_secs_f64() * 1000.0
    );

    // Summary
    println!("\n==============================================");
    println!("  Summary");
    println!("==============================================");
    println!("{:<20} {:>12} {:>12}", "Method", "Time (ms)", "Throughput");
    println!("----------------------------------------");

    let json_rate = rows_count as f64 / json_time.as_secs_f64();
    let binary_rate = rows_count as f64 / binary_load_time.as_secs_f64();

    println!(
        "{:<20} {:>12.2} {:>12.0}",
        "JSON (current)",
        json_time.as_secs_f64() * 1000.0,
        json_rate
    );
    println!(
        "{:<20} {:>12.2} {:>12.0}",
        "Binary (new)",
        binary_load_time.as_secs_f64() * 1000.0,
        binary_rate
    );

    if binary_load_time < json_time {
        let speedup = json_time.as_secs_f64() / binary_load_time.as_secs_f64();
        println!("\n✅ Binary is {:.1}x FASTER than JSON!", speedup);
    } else {
        let slowdown = binary_load_time.as_secs_f64() / json_time.as_secs_f64();
        println!("\n❌ Binary is {:.1}x SLOWER than JSON", slowdown);
    }
    println!("==============================================");
}
