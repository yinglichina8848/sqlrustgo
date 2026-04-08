//! Test FileStorage persistence with real data

use sqlrustgo_storage::{FileStorage, TableInfo, ColumnDefinition, StorageEngine};
use sqlrustgo_types::Value;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let db_path = PathBuf::from("/tmp/test_filestorage_db");
    let data_file = db_path.join("data");
    
    println!("==============================================");
    println!("  FileStorage Persistence Test");
    println!("==============================================");
    
    // Clean up
    let _ = std::fs::remove_dir_all(&db_path);
    
    // Create FileStorage
    println!("Creating FileStorage at {:?}...", db_path);
    let start = Instant::now();
    let mut storage = FileStorage::new(db_path.clone()).expect("Failed to create FileStorage");
    println!("  Created in {:.2}s", start.elapsed().as_secs_f64());
    
    // Create table
    let table_info = TableInfo {
        name: "users".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,
                is_primary_key: true,
                references: None,
                auto_increment: false,
            },
            ColumnDefinition {
                name: "name".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
            },
        ],
    };
    
    let start = Instant::now();
    storage.create_table(&table_info).expect("Failed to create table");
    println!("  Table created in {:.2}s", start.elapsed().as_secs_f64());
    
    // Insert 100 rows (small test)
    println!("Inserting 100 rows...");
    let start = Instant::now();
    for i in 1..=100 {
        let row = vec![
            Value::Integer(i as i64),
            Value::Text(format!("user_{}", i)),
        ];
        storage.insert("users", vec![row]).expect("Failed to insert");
    }
    println!("  Inserted in {:.2}s ({:.0} rows/s)", 
             start.elapsed().as_secs_f64(),
             100.0 / start.elapsed().as_secs_f64());
    
    // Flush to disk
    println!("Flushing to disk...");
    let start = Instant::now();
    storage.flush().expect("Failed to flush");
    println!("  Flushed in {:.2}s", start.elapsed().as_secs_f64());
    
    // Check data file exists
    if data_file.exists() {
        let metadata = std::fs::metadata(&data_file).unwrap();
        println!("  Data file size: {} bytes", metadata.len());
    }
    
    // Drop and recreate to verify persistence
    println!("Closing and reopening...");
    drop(storage);
    
    let start = Instant::now();
    let mut storage = FileStorage::new(db_path.clone()).expect("Failed to reopen");
    println!("  Reopened in {:.2}s", start.elapsed().as_secs_f64());
    
    // Scan to verify data
    let start = Instant::now();
    let rows = storage.scan("users").expect("Failed to scan");
    println!("Scanned {} rows in {:.2}s", rows.len(), start.elapsed().as_secs_f64());
    
    if rows.len() >= 100 {
        println!("✅ FileStorage persistence VERIFIED: {} rows persisted", rows.len());
    } else {
        println!("❌ FAILED: expected 100, got {}", rows.len());
        std::process::exit(1);
    }
    
    // Clean up
    let _ = std::fs::remove_dir_all(&db_path);
    
    println!("==============================================");
    println!("  All tests passed!");
    println!("==============================================");
}
