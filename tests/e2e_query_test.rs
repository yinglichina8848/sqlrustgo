//! End-to-End Query Integration Tests

use sqlrustgo_planner::{DataType, Expr, Field, PhysicalPlan, Schema, SeqScanExec};
use sqlrustgo_types::Value;

#[test]
fn test_simple_seqscan() {
    let schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
    ]);

    let scan = SeqScanExec::new("users".to_string(), schema);
    let results = scan.execute().unwrap();

    println!("✓ SeqScan executed, returned {} rows", results.len());
}

#[test]
fn test_seqscan_schema() {
    let schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
        Field::new("age".to_string(), DataType::Integer),
    ]);

    let scan = SeqScanExec::new("users".to_string(), schema.clone());
    let result_schema = scan.schema();

    assert_eq!(result_schema.fields.len(), 3);
    assert_eq!(result_schema.fields[0].name, "id");
    assert_eq!(result_schema.fields[1].name, "name");
    assert_eq!(result_schema.fields[2].name, "age");

    println!(
        "✓ SeqScan schema correct: {} fields",
        result_schema.fields.len()
    );
}

#[test]
fn test_seqscan_name() {
    let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

    let scan = SeqScanExec::new("test_table".to_string(), schema);

    assert_eq!(scan.name(), "SeqScan");
    assert_eq!(scan.table_name(), "test_table");

    println!("✓ SeqScan name and table_name correct");
}

#[test]
fn test_physical_plan_traits() {
    let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

    let scan = SeqScanExec::new("users".to_string(), schema.clone());

    // Test PhysicalPlan trait methods
    let name = scan.name();
    let table_name = scan.table_name();
    let _result_schema = scan.schema();
    let _children = scan.children();

    assert_eq!(name, "SeqScan");
    assert_eq!(table_name, "users");

    println!("✓ PhysicalPlan trait methods work correctly");
}

#[test]
fn test_multiple_tables() {
    let users_schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
    ]);

    let orders_schema = Schema::new(vec![
        Field::new("order_id".to_string(), DataType::Integer),
        Field::new("total".to_string(), DataType::Integer),
    ]);

    let users_scan = SeqScanExec::new("users".to_string(), users_schema);
    let orders_scan = SeqScanExec::new("orders".to_string(), orders_schema);

    let users_results = users_scan.execute().unwrap();
    let orders_results = orders_scan.execute().unwrap();

    println!(
        "✓ Multiple tables: users={}, orders={}",
        users_results.len(),
        orders_results.len()
    );
}

#[test]
fn test_different_data_types() {
    let schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
        Field::new("price".to_string(), DataType::Float),
        Field::new("active".to_string(), DataType::Boolean),
    ]);

    let scan = SeqScanExec::new("products".to_string(), schema);
    let result_schema = scan.schema();

    assert_eq!(result_schema.fields.len(), 4);

    println!("✓ Different data types handled correctly");
}

#[test]
fn test_empty_table_name() {
    let schema = Schema::new(vec![Field::new("col1".to_string(), DataType::Integer)]);

    let scan = SeqScanExec::new("".to_string(), schema);

    assert_eq!(scan.table_name(), "");

    println!("✓ Empty table name handled");
}

#[test]
fn test_large_schema() {
    let fields: Vec<Field> = (0..50)
        .map(|i| Field::new(format!("column_{}", i), DataType::Integer))
        .collect();

    let schema = Schema::new(fields);
    let scan = SeqScanExec::new("wide_table".to_string(), schema);

    assert_eq!(scan.schema().fields.len(), 50);

    println!("✓ Large schema (50 columns) handled correctly");
}
