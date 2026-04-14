//! SF=1 Binary Import Benchmark

use sqlrustgo_storage::binary_storage::BinaryTableStorage;
use sqlrustgo_storage::{ColumnDefinition, TableData, TableInfo};
use sqlrustgo_types::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let tpch_dir =
        PathBuf::from("/Users/liying/workspace/dev/heartopen/SQLRustGo/data/tpch-sf1-generated");
    let data_dir = PathBuf::from("/tmp/sf1_binary");

    println!("==============================================");
    println!("  SF=1 Binary Import Benchmark");
    println!("==============================================");

    let _ = std::fs::remove_dir_all(&data_dir);
    std::fs::create_dir_all(&data_dir).unwrap();

    let storage = BinaryTableStorage::new(data_dir).unwrap();

    let lineitem_path = tpch_dir.join("lineitem.tbl");
    println!("\nImporting lineitem.tbl (~6M rows)...");

    let start = Instant::now();
    let file = File::open(&lineitem_path).unwrap();
    let reader = BufReader::new(file);

    let mut all_rows: Vec<Vec<Value>> = Vec::new();
    let mut count = 0usize;

    for line in reader.lines() {
        if let Ok(line) = line {
            let fields: Vec<&str> = line.split('|').collect();
            if fields.len() >= 6 {
                all_rows.push(vec![
                    Value::Integer(fields[0].trim().parse().unwrap_or(0)),
                    Value::Integer(fields[1].trim().parse().unwrap_or(0)),
                    Value::Float(fields[4].trim().parse().unwrap_or(0.0)),
                    Value::Float(fields[5].trim().parse().unwrap_or(0.0)),
                    Value::Float(fields[6].trim().parse().unwrap_or(0.0)),
                ]);
                count += 1;
                if count % 500000 == 0 {
                    println!("  Parsed {} rows", count);
                }
            }
        }
    }

    let parse_time = start.elapsed();
    println!(
        "\nParsed {} rows in {:.1}s",
        count,
        parse_time.as_secs_f64()
    );

    println!("\nSaving as binary...");
    let save_start = Instant::now();

    let cols = vec![
        ColumnDefinition {
            name: "l_orderkey".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            is_unique: false,
            is_primary_key: false,
            references: None,
            auto_increment: false,
            compression: None,
        },
        ColumnDefinition {
            name: "l_partkey".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            is_unique: false,
            is_primary_key: false,
            references: None,
            auto_increment: false,
            compression: None,
        },
        ColumnDefinition {
            name: "l_quantity".to_string(),
            data_type: "REAL".to_string(),
            nullable: false,
            is_unique: false,
            is_primary_key: false,
            references: None,
            auto_increment: false,
            compression: None,
        },
        ColumnDefinition {
            name: "l_extendedprice".to_string(),
            data_type: "REAL".to_string(),
            nullable: false,
            is_unique: false,
            is_primary_key: false,
            references: None,
            auto_increment: false,
            compression: None,
        },
        ColumnDefinition {
            name: "l_discount".to_string(),
            data_type: "REAL".to_string(),
            nullable: false,
            is_unique: false,
            is_primary_key: false,
            references: None,
            auto_increment: false,
            compression: None,
        },
    ];
    let info = TableInfo {
        name: "lineitem".to_string(),
        columns: cols,
        ..Default::default()
    };
    let data = TableData {
        info,
        rows: all_rows,
    };

    storage.save("lineitem", &data).unwrap();
    let save_time = save_start.elapsed();

    let bin_path = PathBuf::from("/tmp/sf1_binary/lineitem.bin");
    let bin_size = std::fs::metadata(&bin_path).unwrap().len() as f64;
    let txt_size = std::fs::metadata(&lineitem_path).unwrap().len() as f64;
    println!(
        "Binary save: {:.1}s, size: {:.1} GB",
        save_time.as_secs_f64(),
        bin_size / 1e9
    );
    println!("Compression: {:.1}x", txt_size / bin_size);

    println!("\nLoading binary...");
    let load_start = Instant::now();
    let loaded = storage.load("lineitem").unwrap();
    let load_time = load_start.elapsed();
    println!(
        "Loaded {} rows in {:.1}s ({:.0} rows/s)",
        loaded.rows.len(),
        load_time.as_secs_f64(),
        loaded.rows.len() as f64 / load_time.as_secs_f64()
    );

    println!("\n==============================================");
    println!("  SF=1 Results (~6M rows)");
    println!("==============================================");
    println!(
        "Parse: {:.1}s ({:.0} rows/s)",
        parse_time.as_secs_f64(),
        count as f64 / parse_time.as_secs_f64()
    );
    println!("Binary save: {:.1}s", save_time.as_secs_f64());
    println!(
        "Binary load: {:.1}s ({:.0} rows/s)",
        load_time.as_secs_f64(),
        loaded.rows.len() as f64 / load_time.as_secs_f64()
    );
    println!("Total: {:.1}s", start.elapsed().as_secs_f64());
    println!("==============================================");
}
