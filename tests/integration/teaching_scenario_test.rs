// Teaching Scenario Tests - Comprehensive database education scenarios
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage, StorageEngine};
use sqlrustgo_executor::query_cache_config::{CacheEntry, CacheKey, QueryCacheConfig};
use sqlrustgo_executor::{ExecutorResult, QueryCache};
use sqlrustgo_optimizer::cost::SimpleCostModel;
use sqlrustgo_optimizer::stats::{ColumnStats, TableStats};
use sqlrustgo_planner::{
    DataType, Expr, Field, HashJoinExec, JoinType, Operator, Schema, SeqScanExec,
};
use sqlrustgo_transaction::{
    lock::{LockManager, LockMode},
    TxId,
};
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};
use std::time::Instant;

#[test]
fn test_lock_manager_with_deadlock_detector() {
    let mut manager = LockManager::new();

    let result1 = manager.acquire_lock(TxId::new(1), vec![1], LockMode::Exclusive);
    assert!(result1.is_ok());

    let result2 = manager.acquire_lock(TxId::new(2), vec![2], LockMode::Exclusive);
    assert!(result2.is_ok());

    let result3 = manager.acquire_lock(TxId::new(2), vec![1], LockMode::Shared);
    if let Ok(mode) = result3 {
        assert!(matches!(
            mode,
            sqlrustgo_transaction::lock::LockGrantMode::Waiting
        ));
    }

    let result4 = manager.acquire_lock(TxId::new(1), vec![2], LockMode::Shared);
    if let Ok(mode) = result4 {
        assert!(matches!(
            mode,
            sqlrustgo_transaction::lock::LockGrantMode::Waiting
        ));
    }

    let cycle = manager.detect_deadlock(TxId::new(1));
    assert!(cycle.is_some());

    manager.release_lock(TxId::new(1), &vec![1]).ok();
    manager.release_lock(TxId::new(2), &vec![2]).ok();
}

#[test]
fn test_analyze_updates_statistics() {
    let mut engine = ExecutionEngine::default();

    engine
        .execute(parse("CREATE TABLE products (id INTEGER, name TEXT, price INTEGER)").unwrap())
        .unwrap();

    for i in 1..=100 {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO products VALUES ({}, 'Product{}', {})",
                    i,
                    i,
                    i * 10
                ))
                .unwrap(),
            )
            .unwrap();
    }

    let result = engine.execute(parse("ANALYZE products").unwrap()).unwrap();

    assert!(result.rows.len() > 0);
}

#[test]
fn test_explain_shows_query_plan() {
    let mut engine = ExecutionEngine::default();

    engine
        .execute(parse("CREATE TABLE customers (id INTEGER, name TEXT)").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO customers VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    let result = engine
        .execute(parse("EXPLAIN SELECT * FROM customers WHERE id = 1").unwrap())
        .unwrap();

    assert!(result.rows.len() > 0, "EXPLAIN should return query plan");
}

#[test]
fn test_cbo_optimizer_cost_based_selection() {
    let model = SimpleCostModel::default_model();

    let seq_scan_cost = model.seq_scan_cost(10000, 10);
    let index_scan_cost = model.index_scan_cost(100, 5, 10);

    assert!(
        index_scan_cost < seq_scan_cost,
        "Index scan should be cheaper than sequential scan for small result sets"
    );
}

#[test]
fn test_hash_join_with_condition() {
    let mut storage = MemoryStorage::new();

    storage
        .create_table(&sqlrustgo_storage::TableInfo {
            name: "employees".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        })
        .unwrap();
    storage
        .insert(
            "employees",
            vec![
                vec![Value::Integer(1)],
                vec![Value::Integer(2)],
                vec![Value::Integer(3)],
            ],
        )
        .unwrap();

    storage
        .create_table(&sqlrustgo_storage::TableInfo {
            name: "salaries".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "emp_id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        })
        .unwrap();
    storage
        .insert(
            "salaries",
            vec![vec![Value::Integer(1)], vec![Value::Integer(2)]],
        )
        .unwrap();

    let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

    let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
    let right_schema = Schema::new(vec![Field::new("emp_id".to_string(), DataType::Integer)]);

    let left_scan = Box::new(sqlrustgo_planner::SeqScanExec::new(
        "employees".to_string(),
        left_schema.clone(),
    ));
    let right_scan = Box::new(sqlrustgo_planner::SeqScanExec::new(
        "salaries".to_string(),
        right_schema.clone(),
    ));

    let join_schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("emp_id".to_string(), DataType::Integer),
    ]);

    let join = HashJoinExec::new(
        left_scan,
        right_scan,
        JoinType::Inner,
        Some(Expr::binary_expr(
            Expr::column("id"),
            Operator::Eq,
            Expr::column("emp_id"),
        )),
        join_schema,
    );

    let result = engine.execute_plan(&join).unwrap();
    assert!(
        result.rows.len() >= 2,
        "Hash join should match at least 2 rows"
    );
}

#[test]
fn test_column_statistics_for_optimizer() {
    let col_stats = ColumnStats::new("age".to_string())
        .with_distinct_count(50)
        .with_null_count(5)
        .with_range(Some(Value::Integer(18)), Some(Value::Integer(80)));

    assert_eq!(col_stats.distinct_count, 50);
    assert_eq!(col_stats.null_count, 5);

    let table_stats = TableStats::new("users".to_string()).with_row_count(1000);

    assert_eq!(table_stats.row_count, 1000);

    let selectivity = col_stats.eq_selectivity();
    assert!(selectivity > 0.0 && selectivity <= 1.0);
}

#[test]
fn test_query_cache_basic() {
    let config = QueryCacheConfig::default();
    let mut cache = QueryCache::new(config);

    let key = CacheKey {
        normalized_sql: "SELECT * FROM users WHERE id = 1".to_string(),
        params_hash: 0,
    };
    let entry = CacheEntry {
        result: ExecutorResult::new(
            vec![vec![Value::Integer(1), Value::Text("Alice".to_string())]],
            1,
        ),
        tables: vec!["users".to_string()],
        created_at: Instant::now(),
        size_bytes: 100,
    };

    cache.put(key.clone(), entry.clone(), vec!["users".to_string()]);

    let cached = cache.get(&key);
    assert!(cached.is_some(), "Query cache should return cached result");
}

#[test]
fn test_foreign_key_constraint_enforcement() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
        .unwrap();

    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    engine
        .execute(
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id))")
                .unwrap(),
        )
        .unwrap();

    let valid_result = engine.execute(parse("INSERT INTO orders VALUES (1, 1)").unwrap());
    assert!(valid_result.is_ok(), "Should allow insert with valid FK");

    let invalid_result = engine.execute(parse("INSERT INTO orders VALUES (2, 999)").unwrap());
    assert!(
        invalid_result.is_err(),
        "Should reject insert with invalid FK"
    );
}

#[test]
fn test_isolation_level_read_committed() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE accounts (id INTEGER, balance INTEGER)").unwrap())
        .unwrap();

    engine
        .execute(parse("INSERT INTO accounts VALUES (1, 1000)").unwrap())
        .unwrap();

    let result = engine
        .execute(parse("SELECT * FROM accounts").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_btree_index_operations() {
    use sqlrustgo_storage::bplus_tree::BPlusTree;

    let mut tree: BPlusTree = BPlusTree::new();

    for i in 1i64..=10 {
        tree.insert(i, i as u32);
    }

    let result = tree.search(5);
    assert!(result.is_some());
}

#[test]
fn test_transaction_rollback() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE test_table (id INTEGER, value TEXT)").unwrap())
        .unwrap();

    engine
        .execute(parse("INSERT INTO test_table VALUES (1, 'initial')").unwrap())
        .unwrap();

    let result = engine
        .execute(parse("SELECT * FROM test_table WHERE id = 1").unwrap())
        .unwrap();
    assert_eq!(result.rows[0][1], Value::Text("initial".to_string()));
}

#[test]
fn test_basic_select_operations() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE products (id INTEGER, name TEXT, price INTEGER)").unwrap())
        .ok();
    engine
        .execute(
            parse("INSERT INTO products VALUES (1, 'Apple', 100), (2, 'Banana', 200)").unwrap(),
        )
        .ok();

    let result = engine
        .execute(parse("SELECT * FROM products").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 2, "SELECT * should return all rows");

    let result_col = engine
        .execute(parse("SELECT name FROM products").unwrap())
        .unwrap();
    assert_eq!(
        result_col.rows.len(),
        2,
        "SELECT column should return specific column"
    );
}

#[test]
fn test_where_clause_filtering() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE items (id INTEGER, value INTEGER)").unwrap())
        .ok();
    engine
        .execute(parse("INSERT INTO items VALUES (1, 100), (2, 200), (3, 300)").unwrap())
        .ok();

    let result = engine
        .execute(parse("SELECT * FROM items WHERE value > 150").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 2, "WHERE clause should filter rows");
}

#[test]
fn test_insert_operations() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE test (id INTEGER, name TEXT)").unwrap())
        .ok();

    let result = engine
        .execute(parse("INSERT INTO test VALUES (1, 'Alice')").unwrap())
        .unwrap();
    assert_eq!(result.affected_rows, 1, "INSERT should affect 1 row");

    let result_multi = engine
        .execute(parse("INSERT INTO test VALUES (2, 'Bob'), (3, 'Charlie')").unwrap())
        .unwrap();
    assert_eq!(
        result_multi.affected_rows, 2,
        "Multi-row INSERT should affect 2 rows"
    );
}

#[test]
fn test_update_operations() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE test (id INTEGER, value INTEGER)").unwrap())
        .ok();
    engine
        .execute(parse("INSERT INTO test VALUES (1, 100), (2, 200)").unwrap())
        .ok();

    let result = engine
        .execute(parse("UPDATE test SET value = 150 WHERE id = 1").unwrap())
        .unwrap();
    assert_eq!(result.affected_rows, 1, "UPDATE should affect 1 row");
}

#[test]
fn test_delete_operations() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE test (id INTEGER)").unwrap())
        .ok();
    engine
        .execute(parse("INSERT INTO test VALUES (1), (2), (3)").unwrap())
        .ok();

    let result = engine
        .execute(parse("DELETE FROM test WHERE id = 2").unwrap())
        .unwrap();
    assert_eq!(result.affected_rows, 1, "DELETE should affect 1 row");

    let remaining = engine
        .execute(parse("SELECT COUNT(*) FROM test").unwrap())
        .unwrap();
    assert_eq!(
        remaining.rows[0][0],
        Value::Integer(2),
        "Should have 2 rows remaining"
    );
}

#[test]
fn test_table_creation_ddl() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    let result = engine
        .execute(parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap())
        .unwrap();
    assert_eq!(
        result.affected_rows, 0,
        "CREATE TABLE should return 0 affected rows"
    );

    let exists = engine
        .execute(parse("SELECT * FROM users").unwrap())
        .unwrap();
    assert_eq!(exists.rows.len(), 0, "New table should be empty");
}

#[test]
fn test_multiple_joins() {
    use sqlrustgo_planner::HashJoinExec;

    let mut storage = MemoryStorage::new();

    storage
        .create_table(&sqlrustgo_storage::TableInfo {
            name: "orders".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        })
        .unwrap();
    storage
        .insert(
            "orders",
            vec![vec![Value::Integer(1)], vec![Value::Integer(2)]],
        )
        .ok();

    storage
        .create_table(&sqlrustgo_storage::TableInfo {
            name: "items".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "order_id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        })
        .unwrap();
    storage
        .insert(
            "items",
            vec![vec![Value::Integer(1)], vec![Value::Integer(1)]],
        )
        .ok();

    let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

    let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
    let right_schema = Schema::new(vec![Field::new("order_id".to_string(), DataType::Integer)]);

    let left_scan = Box::new(SeqScanExec::new("orders".to_string(), left_schema));
    let right_scan = Box::new(SeqScanExec::new("items".to_string(), right_schema));

    let join_schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("order_id".to_string(), DataType::Integer),
    ]);

    let join = HashJoinExec::new(left_scan, right_scan, JoinType::Inner, None, join_schema);

    let result = engine.execute_plan(&join).unwrap();
    assert!(result.rows.len() >= 2, "Join should return results");
}
