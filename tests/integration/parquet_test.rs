//! Parquet Import/Export Integration Tests
//!
//! These tests verify end-to-end functionality of Parquet COPY operations:
//! - COPY table TO 'path' (FORMAT PARQUET)
//! - COPY table FROM 'path' (FORMAT PARQUET)
//! - Round-trip data integrity

use sqlrustgo_storage::parquet::{export_to_parquet, import_from_parquet};
use sqlrustgo_types::Value;
use tempfile::TempDir;

fn create_test_records() -> Vec<Vec<Value>> {
    vec![
        vec![
            Value::Integer(1),
            Value::Text("Alice".to_string()),
            Value::Float(3.14),
            Value::Boolean(true),
        ],
        vec![
            Value::Integer(2),
            Value::Text("Bob".to_string()),
            Value::Float(2.71),
            Value::Boolean(false),
        ],
        vec![
            Value::Integer(3),
            Value::Text("Charlie".to_string()),
            Value::Float(1.41),
            Value::Boolean(true),
        ],
    ]
}

fn create_column_names() -> Vec<String> {
    vec![
        "id".to_string(),
        "name".to_string(),
        "score".to_string(),
        "active".to_string(),
    ]
}

#[test]
fn test_export_to_parquet_basic() {
    let temp_dir = TempDir::new().unwrap();
    let parquet_path = temp_dir.path().join("test_export.parquet");

    let records = create_test_records();
    let column_names = create_column_names();

    let result = export_to_parquet(parquet_path.to_str().unwrap(), &records, &column_names);

    assert!(result.is_ok(), "Export to Parquet should succeed");
    assert!(parquet_path.exists(), "Parquet file should be created");
}

#[test]
fn test_import_from_parquet_basic() {
    let temp_dir = TempDir::new().unwrap();
    let parquet_path = temp_dir.path().join("test_import.parquet");

    let records = create_test_records();
    let column_names = create_column_names();

    export_to_parquet(parquet_path.to_str().unwrap(), &records, &column_names).unwrap();

    let imported = import_from_parquet(parquet_path.to_str().unwrap(), &column_names).unwrap();

    assert_eq!(imported.len(), 3, "Should import 3 records");
    assert_eq!(imported[0].len(), 4, "Each record should have 4 columns");
}

#[test]
fn test_parquet_roundtrip_data_integrity() {
    let temp_dir = TempDir::new().unwrap();
    let parquet_path = temp_dir.path().join("roundtrip.parquet");

    let original_records = create_test_records();
    let column_names = create_column_names();

    export_to_parquet(
        parquet_path.to_str().unwrap(),
        &original_records,
        &column_names,
    )
    .unwrap();

    let imported = import_from_parquet(parquet_path.to_str().unwrap(), &column_names).unwrap();

    assert_eq!(original_records.len(), imported.len());

    for (original, imported_row) in original_records.iter().zip(imported.iter()) {
        for (orig_val, imp_val) in original.iter().zip(imported_row.iter()) {
            assert_eq!(
                orig_val, imp_val,
                "Value mismatch: original {:?} vs imported {:?}",
                orig_val, imp_val
            );
        }
    }
}

#[test]
fn test_parquet_null_handling() {
    let temp_dir = TempDir::new().unwrap();
    let parquet_path = temp_dir.path().join("null_test.parquet");

    let records_with_null = vec![
        vec![Value::Integer(1), Value::Null, Value::Float(3.14)],
        vec![
            Value::Integer(2),
            Value::Text("Bob".to_string()),
            Value::Null,
        ],
    ];

    let column_names = vec!["id".to_string(), "name".to_string(), "value".to_string()];

    export_to_parquet(
        parquet_path.to_str().unwrap(),
        &records_with_null,
        &column_names,
    )
    .unwrap();

    let imported = import_from_parquet(parquet_path.to_str().unwrap(), &column_names).unwrap();

    assert_eq!(imported.len(), 2);
    assert_eq!(imported[0][1], Value::Null);
    assert_eq!(imported[1][2], Value::Null);
}

#[test]
fn test_parquet_large_records() {
    let temp_dir = TempDir::new().unwrap();
    let parquet_path = temp_dir.path().join("large.parquet");

    let column_names = vec!["id".to_string(), "data".to_string()];

    let mut large_records = Vec::new();
    for i in 0..1000 {
        large_records.push(vec![
            Value::Integer(i as i64),
            Value::Text(format!("text_content_{}", i)),
        ]);
    }

    export_to_parquet(
        parquet_path.to_str().unwrap(),
        &large_records,
        &column_names,
    )
    .unwrap();

    let imported = import_from_parquet(parquet_path.to_str().unwrap(), &column_names).unwrap();

    assert_eq!(imported.len(), 1000, "Should import all 1000 records");
    assert_eq!(imported[999][0], Value::Integer(999));
    assert_eq!(
        imported[999][1],
        Value::Text("text_content_999".to_string())
    );
}

#[test]
fn test_parquet_various_integer_types() {
    let temp_dir = TempDir::new().unwrap();
    let parquet_path = temp_dir.path().join("types.parquet");

    let records = vec![
        vec![Value::Integer(42), Value::Integer(-100)],
        vec![Value::Integer(i64::MAX), Value::Integer(i64::MIN)],
    ];

    let column_names = vec!["col1".to_string(), "col2".to_string()];

    export_to_parquet(parquet_path.to_str().unwrap(), &records, &column_names).unwrap();

    let imported = import_from_parquet(parquet_path.to_str().unwrap(), &column_names).unwrap();

    assert_eq!(imported[0][0], Value::Integer(42));
    assert_eq!(imported[0][1], Value::Integer(-100));
    assert_eq!(imported[1][0], Value::Integer(i64::MAX));
    assert_eq!(imported[1][1], Value::Integer(i64::MIN));
}

#[test]
fn test_parquet_text_types() {
    let temp_dir = TempDir::new().unwrap();
    let parquet_path = temp_dir.path().join("text_types.parquet");

    let records = vec![
        vec![
            Value::Text("".to_string()),
            Value::Text("Hello World".to_string()),
        ],
        vec![
            Value::Text("Multi\nLine\nText".to_string()),
            Value::Text("Special: !@#$%^&*()".to_string()),
        ],
    ];

    let column_names = vec!["col1".to_string(), "col2".to_string()];

    export_to_parquet(parquet_path.to_str().unwrap(), &records, &column_names).unwrap();

    let imported = import_from_parquet(parquet_path.to_str().unwrap(), &column_names).unwrap();

    assert_eq!(imported[0][0], Value::Text("".to_string()));
    assert_eq!(imported[0][1], Value::Text("Hello World".to_string()));
    assert_eq!(imported[1][0], Value::Text("Multi\nLine\nText".to_string()));
    assert_eq!(
        imported[1][1],
        Value::Text("Special: !@#$%^&*()".to_string())
    );
}
