//! Network Benchmark Tests
//!
//! Benchmarks for network packet processing performance.

use criterion::{criterion_group, criterion_main, Criterion};
use sqlrustgo::network::{ErrPacket, OkPacket, RowData};
use sqlrustgo::types::Value;

// ==================== Packet Benchmarks ====================

fn bench_packet_ok_new(c: &mut Criterion) {
    c.bench_function("packet_ok_new", |b| {
        b.iter(|| {
            OkPacket {
                affected_rows: 1,
                last_insert_id: 0,
                status_flags: 2,
                warnings: 0,
                message: String::new(),
            }
        });
    });
}

fn bench_packet_ok_to_bytes(c: &mut Criterion) {
    let packet = OkPacket {
        affected_rows: 1,
        last_insert_id: 0,
        status_flags: 2,
        warnings: 0,
        message: String::new(),
    };
    c.bench_function("packet_ok_to_bytes", |b| {
        b.iter(|| packet.to_bytes());
    });
}

fn bench_packet_err_new(c: &mut Criterion) {
    c.bench_function("packet_err_new", |b| {
        b.iter(|| {
            ErrPacket {
                error_code: 1146,
                sql_state: "42S02".to_string(),
                message: "Table not found".to_string(),
            }
        });
    });
}

fn bench_packet_err_to_bytes(c: &mut Criterion) {
    let packet = ErrPacket {
        error_code: 1146,
        sql_state: "42S02".to_string(),
        message: "Table not found".to_string(),
    };
    c.bench_function("packet_err_to_bytes", |b| {
        b.iter(|| packet.to_bytes());
    });
}

// ==================== Row Data Benchmarks ====================

fn bench_row_data_new(c: &mut Criterion) {
    c.bench_function("row_data_new_5_columns", |b| {
        b.iter(|| {
            RowData {
                values: vec![
                    Value::Integer(1),
                    Value::Text("Alice".to_string()),
                    Value::Text("alice@example.com".to_string()),
                    Value::Integer(25),
                    Value::Integer(1),
                ],
            }
        });
    });
}

fn bench_row_data_to_bytes(c: &mut Criterion) {
    let row = RowData {
        values: vec![
            Value::Integer(1),
            Value::Text("Alice".to_string()),
            Value::Text("alice@example.com".to_string()),
            Value::Integer(25),
            Value::Integer(1),
        ],
    };
    c.bench_function("row_data_to_bytes_5_columns", |b| {
        b.iter(|| row.to_bytes());
    });
}

fn bench_row_data_many_columns(c: &mut Criterion) {
    c.bench_function("row_data_new_20_columns", |b| {
        b.iter(|| {
            RowData {
                values: vec![
                    Value::Integer(1),
                    Value::Text("Alice".to_string()),
                    Value::Text("alice@example.com".to_string()),
                    Value::Integer(25),
                    Value::Integer(1),
                    Value::Text("active".to_string()),
                    Value::Text("2024-01-01".to_string()),
                    Value::Text("2024-01-15".to_string()),
                    Value::Text("admin".to_string()),
                    Value::Text("".to_string()),
                    Value::Integer(0),
                    Value::Integer(100),
                    Value::Integer(50),
                    Value::Text("en".to_string()),
                    Value::Text("UTC".to_string()),
                    Value::Integer(1),
                    Value::Integer(0),
                    Value::Integer(0),
                    Value::Text("".to_string()),
                    Value::Null,
                ],
            }
        });
    });
}

fn bench_row_data_to_bytes_many(c: &mut Criterion) {
    let row = RowData {
        values: vec![
            Value::Integer(1),
            Value::Text("Alice".to_string()),
            Value::Text("alice@example.com".to_string()),
            Value::Integer(25),
            Value::Integer(1),
            Value::Text("active".to_string()),
            Value::Text("2024-01-01".to_string()),
            Value::Text("2024-01-15".to_string()),
            Value::Text("admin".to_string()),
            Value::Text("".to_string()),
            Value::Integer(0),
            Value::Integer(100),
            Value::Integer(50),
            Value::Text("en".to_string()),
            Value::Text("UTC".to_string()),
            Value::Integer(1),
            Value::Integer(0),
            Value::Integer(0),
            Value::Text("".to_string()),
            Value::Null,
        ],
    };
    c.bench_function("row_data_to_bytes_20_columns", |b| {
        b.iter(|| row.to_bytes());
    });
}

// ==================== Value Benchmarks ====================

fn bench_value_integer(c: &mut Criterion) {
    c.bench_function("value_integer_new", |b| {
        b.iter(|| Value::Integer(12345));
    });
}

fn bench_value_text_small(c: &mut Criterion) {
    c.bench_function("value_text_small_new", |b| {
        b.iter(|| Value::Text("hello".to_string()));
    });
}

fn bench_value_text_medium(c: &mut Criterion) {
    c.bench_function("value_text_medium_new", |b| {
        b.iter(|| Value::Text("This is a medium length string".to_string()));
    });
}

fn bench_value_text_long(c: &mut Criterion) {
    c.bench_function("value_text_long_new", |b| {
        b.iter(|| {
            Value::Text(
                "This is a much longer string that represents a typical database value with more content for testing serialization performance".to_string()
            )
        });
    });
}

fn bench_value_null(c: &mut Criterion) {
    c.bench_function("value_null_new", |b| {
        b.iter(|| Value::Null);
    });
}

fn bench_value_to_string(c: &mut Criterion) {
    let val = Value::Integer(12345);
    c.bench_function("value_integer_to_string", |b| {
        b.iter(|| val.to_string());
    });
}

// ==================== Batch Processing Benchmarks ====================

fn bench_batch_row_serialization(c: &mut Criterion) {
    c.bench_function("batch_rows_10_serialize", |b| {
        b.iter(|| {
            for _ in 0..10 {
                let row = RowData {
                    values: vec![
                        Value::Integer(1),
                        Value::Text("Alice".to_string()),
                        Value::Text("alice@example.com".to_string()),
                        Value::Integer(25),
                        Value::Integer(1),
                    ],
                };
                row.to_bytes();
            }
        });
    });
}

fn bench_batch_row_100_serialization(c: &mut Criterion) {
    c.bench_function("batch_rows_100_serialize", |b| {
        b.iter(|| {
            for i in 0..100 {
                let row = RowData {
                    values: vec![
                        Value::Integer(i),
                        Value::Text(format!("User{}", i)),
                        Value::Text(format!("user{}@example.com", i)),
                        Value::Integer((i % 50) + 18),
                        Value::Integer(1),
                    ],
                };
                row.to_bytes();
            }
        });
    });
}

criterion_group!(
    benches,
    // Packet
    bench_packet_ok_new,
    bench_packet_ok_to_bytes,
    bench_packet_err_new,
    bench_packet_err_to_bytes,
    // Row Data
    bench_row_data_new,
    bench_row_data_to_bytes,
    bench_row_data_many_columns,
    bench_row_data_to_bytes_many,
    // Value
    bench_value_integer,
    bench_value_text_small,
    bench_value_text_medium,
    bench_value_text_long,
    bench_value_null,
    bench_value_to_string,
    // Batch
    bench_batch_row_serialization,
    bench_batch_row_100_serialization
);
criterion_main!(benches);
