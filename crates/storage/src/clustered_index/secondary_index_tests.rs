//! Integration tests for secondary index coordination with clustered indexes.

use crate::clustered_index::{ClusteredLeafPage, SecondaryIndex};
use crate::row_format::types::ClusterKey;
use sqlrustgo_types::Value;

/// Test that inserting into clustered index and updating secondary index works correctly.
#[test]
fn test_secondary_index_insert_coordination() {
    let mut clustered_page = ClusteredLeafPage::new();

    let mut secondary_index = SecondaryIndex::new(
        "idx_email".to_string(),
        "users".to_string(),
        vec!["email".to_string()],
        true,
    );

    for i in 1..=3 {
        let cluster_key = ClusterKey::PrimaryKey(Value::Integer(i as i64));
        let email = format!("user{}@example.com", i);

        let fixed = vec![Value::Integer(i as i64), Value::Text(email.clone())];
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls = vec![false, false];

        let slot_idx = clustered_page
            .insert(&cluster_key, &fixed, &varlen, &nulls)
            .expect("should insert into clustered page");

        secondary_index
            .insert(Value::Text(email), cluster_key)
            .expect("should insert into secondary index");

        assert_eq!(slot_idx, (i - 1) as u16);
    }

    assert_eq!(secondary_index.len(), 3);
    assert!(!secondary_index.is_empty());
}

/// Test that searching via secondary index returns correct cluster keys.
#[test]
fn test_secondary_index_search_coordination() {
    let mut clustered_page = ClusteredLeafPage::new();

    let mut secondary_index = SecondaryIndex::new(
        "idx_email".to_string(),
        "users".to_string(),
        vec!["email".to_string()],
        true,
    );

    let user1_email = "alice@example.com".to_string();
    let user1_key = ClusterKey::PrimaryKey(Value::Integer(1));

    let user2_email = "bob@example.com".to_string();
    let user2_key = ClusterKey::PrimaryKey(Value::Integer(2));

    clustered_page
        .insert(
            &user1_key,
            &[Value::Integer(1), Value::Text(user1_email.clone())],
            &[],
            &[false, false],
        )
        .unwrap();
    secondary_index
        .insert(Value::Text(user1_email.clone()), user1_key.clone())
        .unwrap();

    clustered_page
        .insert(
            &user2_key,
            &[Value::Integer(2), Value::Text(user2_email.clone())],
            &[],
            &[false, false],
        )
        .unwrap();
    secondary_index
        .insert(Value::Text(user2_email.clone()), user2_key.clone())
        .unwrap();

    let results = secondary_index.search(&Value::Text("alice@example.com".to_string()));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], user1_key);

    if let ClusterKey::PrimaryKey(Value::Integer(id)) = results[0] {
        assert_eq!(id, 1);
    } else {
        panic!("Expected PrimaryKey(Integer) cluster key");
    }
}

/// Test that secondary index deletion updates correctly.
#[test]
fn test_secondary_index_delete_coordination() {
    let mut clustered_page = ClusteredLeafPage::new();

    let mut secondary_index = SecondaryIndex::new(
        "idx_email".to_string(),
        "users".to_string(),
        vec!["email".to_string()],
        false,
    );

    let email = "delete@example.com".to_string();
    let cluster_key = ClusterKey::PrimaryKey(Value::Integer(1));

    clustered_page
        .insert(
            &cluster_key,
            &[Value::Integer(1), Value::Text(email.clone())],
            &[],
            &[false, false],
        )
        .unwrap();
    secondary_index
        .insert(Value::Text(email.clone()), cluster_key.clone())
        .unwrap();

    assert_eq!(secondary_index.len(), 1);

    let deleted = secondary_index.delete(&cluster_key);
    assert_eq!(deleted, 1);
    assert_eq!(secondary_index.len(), 0);

    let results = secondary_index.search(&Value::Text(email));
    assert!(results.is_empty());
}

/// Test unique constraint violation detection.
#[test]
fn test_secondary_index_unique_constraint() {
    let mut secondary_index = SecondaryIndex::new(
        "idx_email".to_string(),
        "users".to_string(),
        vec!["email".to_string()],
        true,
    );

    let email = "duplicate@example.com".to_string();
    let key1 = ClusterKey::PrimaryKey(Value::Integer(1));
    let key2 = ClusterKey::PrimaryKey(Value::Integer(2));

    secondary_index
        .insert(Value::Text(email.clone()), key1)
        .expect("first insert should succeed");

    let result = secondary_index.insert(Value::Text(email), key2);
    assert!(result.is_err());
}

/// Test index-only scan optimization (covers_query).
#[test]
fn test_secondary_index_covers_query() {
    let single_col_index = SecondaryIndex::new(
        "idx_email".to_string(),
        "users".to_string(),
        vec!["email".to_string()],
        false,
    );

    assert!(single_col_index.covers_query(&["email".to_string()]));
    assert!(!single_col_index.covers_query(&["name".to_string()]));
    assert!(!single_col_index.covers_query(&["email".to_string(), "name".to_string()]));

    let composite_index = SecondaryIndex::new(
        "idx_name_email".to_string(),
        "users".to_string(),
        vec!["name".to_string(), "email".to_string()],
        false,
    );

    assert!(composite_index.covers_query(&["name".to_string()]));
    assert!(composite_index.covers_query(&["email".to_string()]));
    assert!(composite_index.covers_query(&["name".to_string(), "email".to_string()]));
    assert!(!composite_index.covers_query(&["phone".to_string()]));
}

/// Test range query on secondary index.
#[test]
fn test_secondary_index_range_query() {
    let mut secondary_index = SecondaryIndex::new(
        "idx_age".to_string(),
        "users".to_string(),
        vec!["age".to_string()],
        false,
    );

    for (i, &age) in [18, 25, 30, 35, 40].iter().enumerate() {
        let cluster_key = ClusterKey::PrimaryKey(Value::Integer(i as i64 + 1));
        secondary_index
            .insert(Value::Integer(age), cluster_key)
            .unwrap();
    }

    let results = secondary_index.range_query(&Value::Integer(20), &Value::Integer(35));
    assert_eq!(results.len(), 2);

    let mut ages: Vec<i64> = Vec::new();
    for ck in &results {
        if let ClusterKey::PrimaryKey(Value::Integer(id)) = ck {
            ages.push(*id);
        }
    }
    ages.sort();
    assert_eq!(ages, vec![2, 3]);
}

/// Test multiple secondary indexes on same table.
#[test]
fn test_multiple_secondary_indexes() {
    let mut clustered_page = ClusteredLeafPage::new();

    let mut email_index = SecondaryIndex::new(
        "idx_email".to_string(),
        "users".to_string(),
        vec!["email".to_string()],
        true,
    );

    let mut name_index = SecondaryIndex::new(
        "idx_name".to_string(),
        "users".to_string(),
        vec!["name".to_string()],
        false,
    );

    let user1_name = "Alice".to_string();
    let user1_email = "alice@example.com".to_string();
    let user1_key = ClusterKey::PrimaryKey(Value::Integer(1));

    clustered_page
        .insert(
            &user1_key,
            &[
                Value::Integer(1),
                Value::Text(user1_name.clone()),
                Value::Text(user1_email.clone()),
            ],
            &[],
            &[false, false, false],
        )
        .unwrap();

    email_index
        .insert(Value::Text(user1_email.clone()), user1_key.clone())
        .unwrap();
    name_index
        .insert(Value::Text(user1_name.clone()), user1_key.clone())
        .unwrap();

    let user2_name = "Alice".to_string();
    let user2_email = "alice2@example.com".to_string();
    let user2_key = ClusterKey::PrimaryKey(Value::Integer(2));

    clustered_page
        .insert(
            &user2_key,
            &[
                Value::Integer(2),
                Value::Text(user2_name.clone()),
                Value::Text(user2_email.clone()),
            ],
            &[],
            &[false, false, false],
        )
        .unwrap();

    email_index
        .insert(Value::Text(user2_email.clone()), user2_key.clone())
        .unwrap();
    name_index
        .insert(Value::Text(user2_name.clone()), user2_key.clone())
        .unwrap();

    let email_results = email_index.search(&Value::Text("alice@example.com".to_string()));
    assert_eq!(email_results.len(), 1);
    assert_eq!(email_results[0], user1_key);

    let name_results = name_index.search(&Value::Text("Alice".to_string()));
    assert_eq!(name_results.len(), 2);
    assert!(name_results.contains(&user1_key));
    assert!(name_results.contains(&user2_key));
}

/// Test secondary index with HiddenRowId cluster keys.
#[test]
fn test_secondary_index_hidden_row_id() {
    let mut secondary_index = SecondaryIndex::new(
        "idx_phone".to_string(),
        "contacts".to_string(),
        vec!["phone".to_string()],
        true,
    );

    let ck1 = ClusterKey::HiddenRowId(100);
    let ck2 = ClusterKey::HiddenRowId(200);

    secondary_index
        .insert(Value::Text("123-456-7890".to_string()), ck1.clone())
        .unwrap();
    secondary_index
        .insert(Value::Text("098-765-4321".to_string()), ck2.clone())
        .unwrap();

    let results = secondary_index.search(&Value::Text("123-456-7890".to_string()));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], ck1);
}

/// Test secondary index with null values.
#[test]
fn test_secondary_index_null_handling() {
    let mut secondary_index = SecondaryIndex::new(
        "idx_name".to_string(),
        "users".to_string(),
        vec!["name".to_string()],
        false,
    );

    let ck = ClusterKey::PrimaryKey(Value::Integer(1));

    secondary_index.insert(Value::Null, ck.clone()).unwrap();

    let results = secondary_index.search(&Value::Null);
    assert_eq!(results.len(), 1);

    let results = secondary_index.search(&Value::Text("John".to_string()));
    assert!(results.is_empty());
}

/// Test search_unique on unique secondary index.
#[test]
fn test_secondary_index_search_unique() {
    let mut secondary_index = SecondaryIndex::new(
        "idx_email".to_string(),
        "users".to_string(),
        vec!["email".to_string()],
        true,
    );

    let ck = ClusterKey::PrimaryKey(Value::Integer(42));
    secondary_index
        .insert(Value::Text("answer@example.com".to_string()), ck.clone())
        .unwrap();

    let result = secondary_index.search_unique(&Value::Text("answer@example.com".to_string()));
    assert!(result.is_some());
    assert_eq!(result.unwrap(), ck);

    let result = secondary_index.search_unique(&Value::Text("missing@example.com".to_string()));
    assert!(result.is_none());
}
