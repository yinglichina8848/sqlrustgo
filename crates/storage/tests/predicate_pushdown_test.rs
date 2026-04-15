use sqlrustgo_storage::predicate::Predicate;
use sqlrustgo_storage::{ColumnDefinition, MemoryStorage, StorageEngine, TableInfo};
use sqlrustgo_types::Value;

fn create_test_table() -> (MemoryStorage, TableInfo) {
    let mut storage = MemoryStorage::new();

    let table_info = TableInfo {
        name: "users".to_string(),
        columns: vec![
            ColumnDefinition::new("id", "INTEGER"),
            ColumnDefinition::new("name", "TEXT"),
            ColumnDefinition::new("age", "INTEGER"),
        ],
        ..Default::default()
    };

    storage.create_table(&table_info).unwrap();

    storage
        .insert(
            "users",
            vec![
                vec![
                    Value::Integer(1),
                    Value::Text("Alice".to_string()),
                    Value::Integer(25),
                ],
                vec![
                    Value::Integer(2),
                    Value::Text("Bob".to_string()),
                    Value::Integer(30),
                ],
                vec![
                    Value::Integer(3),
                    Value::Text("Charlie".to_string()),
                    Value::Integer(35),
                ],
                vec![
                    Value::Integer(4),
                    Value::Text("Diana".to_string()),
                    Value::Integer(28),
                ],
                vec![
                    Value::Integer(5),
                    Value::Text("Eve".to_string()),
                    Value::Integer(40),
                ],
            ],
        )
        .unwrap();

    (storage, table_info)
}

#[test]
fn test_scan_predicate_eq() {
    let (storage, _) = create_test_table();

    let predicate = Predicate::eq("id", Value::Integer(2));
    let result = storage.scan_predicate("users", &predicate).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0][0], Value::Integer(2));
    assert_eq!(result[0][1], Value::Text("Bob".to_string()));
}

#[test]
fn test_scan_predicate_lt() {
    let (storage, _) = create_test_table();

    let predicate = Predicate::lt("age", Value::Integer(30));
    let result = storage.scan_predicate("users", &predicate).unwrap();

    assert_eq!(result.len(), 2);
    assert!(result
        .iter()
        .all(|r| matches!(r[0], Value::Integer(1) | Value::Integer(4))));
}

#[test]
fn test_scan_predicate_lte() {
    let (storage, _) = create_test_table();

    let predicate = Predicate::lte("age", Value::Integer(30));
    let result = storage.scan_predicate("users", &predicate).unwrap();

    assert_eq!(result.len(), 3);
}

#[test]
fn test_scan_predicate_gt() {
    let (storage, _) = create_test_table();

    let predicate = Predicate::gt("age", Value::Integer(30));
    let result = storage.scan_predicate("users", &predicate).unwrap();

    assert_eq!(result.len(), 2);
    assert!(result
        .iter()
        .all(|r| matches!(r[0], Value::Integer(3) | Value::Integer(5))));
}

#[test]
fn test_scan_predicate_gte() {
    let (storage, _) = create_test_table();

    let predicate = Predicate::gte("age", Value::Integer(35));
    let result = storage.scan_predicate("users", &predicate).unwrap();

    assert_eq!(result.len(), 2);
}

#[test]
fn test_scan_predicate_and() {
    let (storage, _) = create_test_table();

    let predicate = Predicate::and(
        Predicate::gt("age", Value::Integer(25)),
        Predicate::lt("age", Value::Integer(35)),
    );
    let result = storage.scan_predicate("users", &predicate).unwrap();

    assert_eq!(result.len(), 2);
}

#[test]
fn test_scan_predicate_or() {
    let (storage, _) = create_test_table();

    let predicate = Predicate::or(
        Predicate::eq("id", Value::Integer(1)),
        Predicate::eq("id", Value::Integer(5)),
    );
    let result = storage.scan_predicate("users", &predicate).unwrap();

    assert_eq!(result.len(), 2);
}

#[test]
fn test_scan_predicate_not() {
    let (storage, _) = create_test_table();

    let predicate = Predicate::not(Predicate::eq("id", Value::Integer(1)));
    let result = storage.scan_predicate("users", &predicate).unwrap();

    assert_eq!(result.len(), 4);
}

#[test]
fn test_scan_predicate_no_match() {
    let (storage, _) = create_test_table();

    let predicate = Predicate::gt("id", Value::Integer(100));
    let result = storage.scan_predicate("users", &predicate).unwrap();

    assert_eq!(result.len(), 0);
}
