//! Integration tests for the executor test framework
//!
//! These tests demonstrate how to use the test framework components
//! and verify they work correctly together.

use sqlrustgo_executor::{
    harness::{assertions, compare, fixtures, helpers, ExecutorTestCase, TestHarness},
    mock_storage::MockStorage,
    test_data::{RowBuilder, TestDataGenerator, TestDataSet, TestTableBuilder},
    ExecutorResult,
};

use sqlrustgo_planner::{DataType, Field, PhysicalPlan, Schema, SeqScanExec};
use sqlrustgo_storage::{ColumnDefinition, StorageEngine, TableInfo};
use sqlrustgo_types::Value;

// Helper to create a simple seq scan physical plan for testing
fn create_test_plan(table_name: &str) -> Box<dyn PhysicalPlan> {
    let schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
    ]);
    Box::new(SeqScanExec::new(table_name.to_string(), schema))
}

#[test]
fn test_mock_storage_with_test_data_set() {
    // Use predefined test data
    let storage = MockStorage::with_data("users", TestDataSet::simple_users());

    assert!(storage.has_table("users"));
    assert_eq!(storage.row_count("users"), 3);

    let rows = storage.get_table_data("users").unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0][0], Value::Integer(1));
    assert_eq!(rows[0][1], Value::Text("Alice".to_string()));
}

#[test]
fn test_test_data_generator() {
    let mut generator = TestDataGenerator::new();

    // Generate users table data
    let users = generator.generate_users_table(5);
    assert_eq!(users.len(), 5);

    // Each row should have 4 columns
    for row in &users {
        assert_eq!(row.len(), 4);
    }
}

#[test]
fn test_test_table_builder() {
    let builder = TestTableBuilder::new("products")
        .add_integer_column("id")
        .add_text_column("name")
        .add_integer_column("price");

    let schema = builder.build_schema();
    assert_eq!(schema.fields.len(), 3);
    assert_eq!(schema.fields[0].name, "id");
    assert_eq!(schema.fields[1].name, "name");
    assert_eq!(schema.fields[2].name, "price");
}

#[test]
fn test_harness_with_mock_storage() {
    let storage = MockStorage::with_data("users", TestDataSet::simple_users());
    let harness = TestHarness::<MockStorage>::new(storage);

    // Verify storage is accessible
    assert!(harness.storage().has_table("users"));
}

#[test]
fn test_harness_execute_seq_scan() {
    let storage = MockStorage::with_data("users", TestDataSet::simple_users());
    let harness = TestHarness::<MockStorage>::new(storage);

    let plan = create_test_plan("users");
    let result = harness.execute(plan.as_ref()).unwrap();

    assertions::assert_row_count(&result, 3);
    assertions::assert_has_rows(&result);
}

#[test]
fn test_helpers_users_schema() {
    let schema = helpers::users_schema();
    assert_eq!(schema.fields.len(), 2);
    assert_eq!(schema.fields[0].name, "id");
}

#[test]
fn test_helpers_orders_schema() {
    let schema = helpers::orders_schema();
    assert_eq!(schema.fields.len(), 3);
    assert_eq!(schema.fields[1].name, "user_id");
}

#[test]
fn test_fixtures_setup_users_table() {
    let mut storage = MockStorage::new();
    fixtures::setup_users_table(&mut storage).unwrap();

    assert!(storage.has_table("users"));
    assert_eq!(storage.row_count("users"), 3);
}

#[test]
fn test_fixtures_setup_orders_table() {
    let mut storage = MockStorage::new();
    fixtures::setup_orders_table(&mut storage).unwrap();

    assert!(storage.has_table("orders"));
    assert_eq!(storage.row_count("orders"), 3);
}

#[test]
fn test_compare_rows_match() {
    let actual = vec![Value::Integer(1), Value::Text("Alice".to_string())];
    let expected = vec![Value::Integer(1), Value::Text("Alice".to_string())];

    assert!(compare::rows_match(&actual, &expected));
}

#[test]
fn test_compare_find_matching_row() {
    let result = vec![
        vec![Value::Integer(1), Value::Text("Alice".to_string())],
        vec![Value::Integer(2), Value::Text("Bob".to_string())],
        vec![Value::Integer(3), Value::Text("Charlie".to_string())],
    ];

    let expected = vec![Value::Null, Value::Text("Bob".to_string())];
    let found = compare::find_matching_row(&result, &expected);

    assert!(found.is_some());
    assert_eq!(found.unwrap()[0], Value::Integer(2));
}

#[test]
fn test_assertions_first_row_equals() {
    let result = ExecutorResult::new(
        vec![vec![Value::Integer(1), Value::Text("Alice".to_string())]],
        0,
    );

    assertions::assert_first_row_equals(
        &result,
        &[Value::Integer(1), Value::Text("Alice".to_string())],
    );
}

#[test]
fn test_assertions_contains_value() {
    let result = ExecutorResult::new(
        vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string())],
            vec![Value::Integer(2), Value::Text("Bob".to_string())],
        ],
        0,
    );

    assertions::assert_contains_value(&result, &Value::Text("Bob".to_string()));
}

#[test]
fn test_executor_test_case() {
    let storage = MockStorage::with_data("users", TestDataSet::simple_users());
    let harness = TestHarness::<MockStorage>::new(storage);

    let plan = create_test_plan("users");
    let _test_case = ExecutorTestCase::new("test_users", plan)
        .expect_rows(3)
        .expect_first_row(vec![Value::Integer(1), Value::Text("Alice".to_string())]);

    // Re-create plan for test case
    let plan2 = create_test_plan("users");
    let result = harness.execute(plan2.as_ref()).unwrap();
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn test_sequential_integers() {
    let mut generator = TestDataGenerator::with_seed(123);
    let integers = generator.generate_sequential_integers(1, 10);

    assert_eq!(integers.len(), 10);
    assert_eq!(integers[0], Value::Integer(1));
    assert_eq!(integers[9], Value::Integer(10));
}

#[test]
fn test_aggregate_test_data() {
    let data = TestDataSet::aggregate_test_data();

    // Should have 5 rows: 2 for A, 2 for B, 1 for C
    assert_eq!(data.len(), 5);

    // Count groups
    let mut groups = std::collections::HashMap::new();
    for row in &data {
        if let Value::Text(grp) = &row[0] {
            *groups.entry(grp.clone()).or_insert(0) += 1;
        }
    }

    assert_eq!(groups.get("A"), Some(&2));
    assert_eq!(groups.get("B"), Some(&2));
    assert_eq!(groups.get("C"), Some(&1));
}

#[test]
fn test_mock_storage_clone() {
    let storage = MockStorage::with_data("users", TestDataSet::simple_users());

    let cloned = storage.clone();
    assert!(cloned.has_table("users"));
    assert_eq!(cloned.row_count("users"), 3);
}

#[test]
fn test_mock_storage_clear() {
    let storage = MockStorage::with_data("users", TestDataSet::simple_users());
    storage.clear();

    assert_eq!(storage.table_count(), 0);
}

#[test]
fn test_mock_storage_insert() {
    let mut storage = MockStorage::new();
    storage
        .create_table(&TableInfo {
            name: "test".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            }],
        })
        .unwrap();

    storage
        .insert("test", vec![vec![Value::Integer(1)]])
        .unwrap();

    assert_eq!(storage.row_count("test"), 1);
}

#[test]
fn test_row_builder() {
    let row = RowBuilder::new()
        .add_integer(1)
        .add_text("Test")
        .add_float(3.14)
        .add_boolean(true)
        .build();

    assert_eq!(row.len(), 4);
    assert_eq!(row[0], Value::Integer(1));
    assert_eq!(row[1], Value::Text("Test".to_string()));
    assert_eq!(row[2], Value::Float(3.14));
    assert_eq!(row[3], Value::Boolean(true));
}

#[test]
fn test_data_set_with_nulls() {
    let data = TestDataSet::with_nulls();

    assert_eq!(data.len(), 3);
    assert_eq!(data[1][1], Value::Null);
}
