//! UPDATE/DELETE Micro-Benchmark
//!
//! Purpose: Establish baseline for Issue #1156 UPDATE/DELETE performance regression
//! Measures: UPDATE, DELETE, INSERT throughput with various row counts
//!
//! Run: cargo test -p sqlrustgo-storage --test update_delete_benchmark -- --nocapture

use sqlrustgo_storage::engine::{ColumnDefinition, StorageEngine, TableInfo};
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::time::Instant;

fn setup_table(storage: &mut dyn StorageEngine, name: &str, rows: usize) -> Result<(), String> {
    storage
        .create_table(&TableInfo {
            name: name.to_string(),
            columns: vec![
                ColumnDefinition::new("id", "INTEGER"),
                ColumnDefinition::new("val", "INTEGER"),
            ],
            ..Default::default()
        })
        .map_err(|e| format!("create_table: {}", e))?;

    for i in 0..rows {
        storage
            .insert(
                name,
                vec![vec![
                    Value::Integer(i as i64),
                    Value::Integer(i as i64 * 10),
                ]],
            )
            .map_err(|e| format!("insert failed: {}", e))?;
    }
    Ok(())
}

#[test]
fn benchmark_update_indexed_column_1k() {
    let mut storage = MemoryStorage::new();
    setup_table(&mut storage, "t", 1000).unwrap();

    // Warm up
    let _ = storage.update("t", &[Value::Integer(0)], &[(1, Value::Integer(99))]);

    // Benchmark
    let start = Instant::now();
    for i in 0..1000 {
        storage
            .update(
                "t",
                &[Value::Integer(i as i64)],
                &[(1, Value::Integer(i as i64 * 10))],
            )
            .unwrap();
    }
    let elapsed = start.elapsed();

    println!(
        "UPDATE 1k rows x1000: {:.2}µs/op, total {:.3}s",
        elapsed.as_micros() as f64 / 1000.0,
        elapsed.as_secs_f64()
    );
}

#[test]
fn benchmark_update_all_rows_1k() {
    let mut storage = MemoryStorage::new();
    setup_table(&mut storage, "t", 1000).unwrap();

    // UPDATE ALL rows (no WHERE)
    let start = Instant::now();
    for _ in 0..100 {
        storage
            .update("t", &[], &[(1, Value::Integer(42))])
            .unwrap();
    }
    let elapsed = start.elapsed();

    println!(
        "UPDATE ALL 1k rows x100: {:.2}µs/op, total {:.3}s",
        elapsed.as_micros() as f64 / 100.0,
        elapsed.as_secs_f64()
    );
}

#[test]
fn benchmark_delete_indexed_1k() {
    let mut storage = MemoryStorage::new();
    setup_table(&mut storage, "t", 1000).unwrap();

    // Warm up
    let _ = storage.delete("t", &[Value::Integer(0)]);

    // Benchmark
    let start = Instant::now();
    for i in 1..1000 {
        storage.delete("t", &[Value::Integer(i as i64)]).unwrap();
    }
    let elapsed = start.elapsed();

    println!(
        "DELETE 1 row x999: {:.2}µs/op, total {:.3}s",
        elapsed.as_micros() as f64 / 999.0,
        elapsed.as_secs_f64()
    );
}

#[test]
fn benchmark_delete_all_1k() {
    let mut storage = MemoryStorage::new();
    setup_table(&mut storage, "t", 1000).unwrap();

    // DELETE ALL (no WHERE)
    let start = Instant::now();
    for _ in 0..100 {
        storage.delete("t", &[]).unwrap();
    }
    let elapsed = start.elapsed();

    println!(
        "DELETE ALL 1k rows x100: {:.2}µs/op, total {:.3}s",
        elapsed.as_micros() as f64 / 100.0,
        elapsed.as_secs_f64()
    );
}
