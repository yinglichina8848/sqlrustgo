//! SF=1 TPC-H LineItem Import and Benchmark

use sqlrustgo_storage::{ColumnDefinition, FileStorage, TableData, TableInfo};
use sqlrustgo_types::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let data_dir = PathBuf::from("/tmp/tpch_sf1_test");
    let tpch_dir =
        PathBuf::from("/Users/liying/workspace/dev/heartopen/SQLRustGo/data/tpch-sf1-generated");

    println!("==============================================");
    println!("  SF=1 TPC-H Import Test (6M rows)");
    println!("==============================================");

    // Clean up
    let _ = std::fs::remove_dir_all(&data_dir);
    std::fs::create_dir_all(&data_dir).unwrap();

    // Create FileStorage
    let mut storage = FileStorage::new(data_dir).expect("Failed to create storage");

    // Import lineitem
    let lineitem_path = tpch_dir.join("lineitem.tbl");
    println!("\nImporting lineitem.tbl (~6M rows)...");
    println!(
        "File size: {:.1} GB",
        std::fs::metadata(&lineitem_path).unwrap().len() as f64 / 1e9
    );

    let start = Instant::now();
    let file = File::open(&lineitem_path).unwrap();
    let reader = BufReader::with_capacity(8 << 20, file);

    let mut batch = Vec::new();
    let mut count = 0;
    let mut total_rows = 0;

    for line in reader.lines() {
        let line = line.unwrap();
        let fields: Vec<&str> = line.split('|').collect();
        if fields.len() >= 2 {
            let orderkey = fields[0].trim().parse::<i64>().unwrap_or(0);
            let partkey = fields[1].trim().parse::<i64>().unwrap_or(0);
            batch.push(vec![Value::Integer(orderkey), Value::Integer(partkey)]);
            total_rows += 1;

            if batch.len() >= 10000 {
                let rows = std::mem::replace(&mut batch, Vec::with_capacity(10000));
                let table_data = TableData {
                    info: TableInfo {
                        name: "lineitem".to_string(),
                        columns: vec![
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
                        ],
                    },
                    rows,
                };
                storage
                    .insert_table("lineitem".to_string(), table_data)
                    .unwrap();
                count += 1;

                if count % 100 == 0 {
                    let elapsed = start.elapsed().as_secs_f64();
                    println!(
                        "  {} rows imported ({:.0} rows/s)",
                        total_rows,
                        total_rows as f64 / elapsed
                    );
                }
            }
        }
    }

    // Insert remaining
    if !batch.is_empty() {
        let table_data = TableData {
            info: TableInfo {
                name: "lineitem".to_string(),
                columns: vec![
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
                ],
            },
            rows: batch.clone(),
        };
        storage
            .insert_table("lineitem".to_string(), table_data)
            .unwrap();
        total_rows += batch.len();
    }

    let import_time = start.elapsed();
    println!("\n==============================================");
    println!(
        "Import complete: {} rows in {:.1}s",
        total_rows,
        import_time.as_secs_f64()
    );
    println!(
        "Throughput: {:.0} rows/s",
        total_rows as f64 / import_time.as_secs_f64()
    );
    println!("==============================================");
}
