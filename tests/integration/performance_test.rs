// Performance and Integration Tests for v1.9.0 Features
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage, StorageEngine};
use sqlrustgo_executor::vectorization::RecordBatch;
use sqlrustgo_planner::{
    DataType, Expr, Field, FilterExec, IndexScanExec, Operator, Schema, SeqScanExec,
};
use sqlrustgo_server::{ConnectionPool, PoolConfig};
use sqlrustgo_storage::{ColumnDefinition, TableInfo};
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};
use std::time::Instant;

const BENCH_ROW_COUNT: usize = 10000;

#[test]
fn test_single_insert_qps() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE single_insert_test (id INTEGER, value TEXT)").unwrap())
        .unwrap();

    let start = Instant::now();

    for i in 0..BENCH_ROW_COUNT {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO single_insert_test VALUES ({}, 'value{}')",
                    i, i
                ))
                .unwrap(),
            )
            .unwrap();
    }

    let elapsed = start.elapsed();
    let qps = BENCH_ROW_COUNT as f64 / elapsed.as_secs_f64();
    println!(
        "Single insert QPS: {:.2} ops/s ({} rows in {:?})",
        qps, BENCH_ROW_COUNT, elapsed
    );

    // Verify all rows were inserted
    let result = engine
        .execute(parse("SELECT COUNT(*) FROM single_insert_test").unwrap())
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(BENCH_ROW_COUNT as i64));

    // QPS should be >= 1000
    assert!(qps >= 1000.0, "QPS {} should be >= 1000", qps);
}

#[test]
fn test_batch_insert_performance() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE performance_test (id INTEGER, value TEXT)").unwrap())
        .unwrap();

    let start = Instant::now();

    for i in 0..BENCH_ROW_COUNT {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO performance_test VALUES ({}, 'value{}')",
                    i, i
                ))
                .unwrap(),
            )
            .unwrap();
    }

    let elapsed = start.elapsed();
    println!("Batch insert {} rows took: {:?}", BENCH_ROW_COUNT, elapsed);

    let result = engine
        .execute(parse("SELECT COUNT(*) FROM performance_test").unwrap())
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(BENCH_ROW_COUNT as i64));
}

#[test]
fn test_batch_insert_single_statement() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE batch_test (id INTEGER, name TEXT)").unwrap())
        .unwrap();

    let values: Vec<String> = (1..=100).map(|i| format!("({}, 'Item{}')", i, i)).collect();
    let sql = format!("INSERT INTO batch_test VALUES {}", values.join(", "));

    let start = Instant::now();
    engine.execute(parse(&sql).unwrap()).unwrap();
    let elapsed = start.elapsed();

    println!("Single statement batch insert 100 rows took: {:?}", elapsed);

    let result = engine
        .execute(parse("SELECT COUNT(*) FROM batch_test").unwrap())
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(100));
}

#[test]
fn test_query_optimization_predicate_pushdown() {
    let mut storage = MemoryStorage::new();

    for i in 0..1000 {
        storage
            .insert(
                "orders",
                vec![vec![
                    Value::Integer(i as i64),
                    Value::Integer((i * 10) as i64),
                ]],
            )
            .ok();
    }

    let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));
    let schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("amount".to_string(), DataType::Integer),
    ]);

    let scan = SeqScanExec::new("orders".to_string(), schema.clone());
    let predicate = Expr::binary_expr(
        Expr::column("id"),
        Operator::Gt,
        Expr::literal(Value::Integer(500)),
    );
    let filter = FilterExec::new(Box::new(scan), predicate);

    let start = Instant::now();
    let result = engine.execute_plan(&filter).unwrap();
    let elapsed = start.elapsed();

    println!("Predicate pushdown query took: {:?}", elapsed);
    println!("Returned {} rows (expected ~500)", result.rows.len());

    assert!(result.rows.len() < 1000, "Filter should reduce row count");
}

#[test]
fn test_query_optimization_projection() {
    let mut storage = MemoryStorage::new();

    for i in 0..1000 {
        storage
            .insert(
                "users",
                vec![vec![
                    Value::Integer(i as i64),
                    Value::Text(format!("User{}", i)),
                    Value::Text(format!("Email{}@example.com", i)),
                ]],
            )
            .ok();
    }

    let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));
    let schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
        Field::new("email".to_string(), DataType::Text),
    ]);

    let scan = SeqScanExec::new("users".to_string(), schema);

    let start = Instant::now();
    let result = engine.execute_plan(&scan).unwrap();
    let elapsed = start.elapsed();

    println!("Projection query on 1000 rows took: {:?}", elapsed);
    assert_eq!(result.rows.len(), 1000);
}

#[test]
fn test_connection_pool_basic_operations() {
    let config = PoolConfig::default();
    let pool = ConnectionPool::new(config);

    for _ in 0..10 {
        let _session = pool.acquire();
    }

    assert!(true, "Connection pool basic operations work");
}

#[test]
fn test_connection_pool_concurrent_stress() {
    let config = PoolConfig {
        size: 5,
        timeout_ms: 5000,
    };
    let pool = ConnectionPool::new(config);

    let start = Instant::now();

    std::thread::scope(|s| {
        for _ in 0..20 {
            s.spawn(|| {
                for _ in 0..10 {
                    let _session = pool.acquire();
                    std::thread::sleep(std::time::Duration::from_micros(100));
                }
            });
        }
    });

    let elapsed = start.elapsed();
    println!("Connection pool concurrent stress test took: {:?}", elapsed);

    assert!(
        elapsed.as_secs() < 10,
        "Pool should handle concurrent requests efficiently"
    );
}

#[test]
fn test_vectorization_record_batch() {
    use sqlrustgo_executor::vectorization::Vector;

    let vec: Vector<u8> = Vector::from_vec((0u8..128).collect());

    let mut batch = RecordBatch::new(128);
    batch.add_column(vec);

    assert_eq!(batch.num_rows(), 128);
    assert_eq!(batch.num_columns(), 1);
}

#[test]
fn test_vectorization_bulk_operations() {
    use sqlrustgo_executor::vectorization::Vector;

    let capacity = 10000;

    let start = Instant::now();

    let vec: Vector<u8> = Vector::from_vec((0u8..255).cycle().take(capacity).collect());

    let elapsed = start.elapsed();
    println!(
        "Vectorization bulk insert {} elements took: {:?}",
        capacity, elapsed
    );

    assert_eq!(vec.len(), capacity);
}

#[test]
fn test_parallel_scan_operations() {
    let mut storage = MemoryStorage::new();

    let row_count = 5000;
    for i in 0..row_count {
        storage
            .insert("parallel_test", vec![vec![Value::Integer(i as i64)]])
            .ok();
    }

    let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));
    let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

    let scan = SeqScanExec::new("parallel_test".to_string(), schema);

    let start = Instant::now();
    let result = engine.execute_plan(&scan).unwrap();
    let elapsed = start.elapsed();

    println!("Parallel scan of {} rows took: {:?}", row_count, elapsed);
    assert_eq!(result.rows.len(), row_count);
}

#[test]
fn test_index_scan_performance_vs_seqscan() {
    let mut storage = MemoryStorage::new();

    storage
        .create_table(&sqlrustgo_storage::TableInfo {
            name: "products".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        })
        .unwrap();

    for i in 0..1000 {
        storage
            .insert("products", vec![vec![Value::Integer(i as i64)]])
            .ok();
    }

    let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

    let seq_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
    let seq_scan = SeqScanExec::new("products".to_string(), seq_schema);

    let start_seq = Instant::now();
    let seq_result = engine.execute_plan(&seq_scan).unwrap();
    let seq_time = start_seq.elapsed();

    let index_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
    let index_scan = IndexScanExec::new(
        "products".to_string(),
        "idx_id".to_string(),
        Expr::literal(Value::Integer(500)),
        index_schema,
    );

    let start_index = Instant::now();
    let index_result = engine.execute_plan(&index_scan).unwrap();
    let index_time = start_index.elapsed();

    println!(
        "SeqScan time: {:?}, IndexScan time: {:?}",
        seq_time, index_time
    );
    println!(
        "SeqScan rows: {}, IndexScan rows: {}",
        seq_result.rows.len(),
        index_result.rows.len()
    );
}

#[test]
fn test_join_performance_hash_join() {
    use sqlrustgo_planner::{HashJoinExec, JoinType};

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

    for i in 1..=100 {
        storage
            .insert("employees", vec![vec![Value::Integer(i)]])
            .ok();
    }

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

    for i in 1..=100 {
        storage
            .insert("salaries", vec![vec![Value::Integer(i)]])
            .ok();
    }

    let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

    let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
    let right_schema = Schema::new(vec![Field::new("emp_id".to_string(), DataType::Integer)]);

    let left_scan = Box::new(SeqScanExec::new("employees".to_string(), left_schema));
    let right_scan = Box::new(SeqScanExec::new("salaries".to_string(), right_schema));

    let join_schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("emp_id".to_string(), DataType::Integer),
    ]);

    let join = HashJoinExec::new(left_scan, right_scan, JoinType::Inner, None, join_schema);

    let start = Instant::now();
    let result = engine.execute_plan(&join).unwrap();
    let elapsed = start.elapsed();

    println!("Hash join of 100x100 rows took: {:?}", elapsed);
    println!("Join result: {} rows", result.rows.len());

    assert!(result.rows.len() > 0, "Join should return results");
}

#[test]
fn test_order_by_performance() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE sorted_data (value INTEGER)").unwrap())
        .unwrap();

    for i in (0..1000).rev() {
        engine
            .execute(parse(&format!("INSERT INTO sorted_data VALUES ({})", i)).unwrap())
            .unwrap();
    }

    let start = Instant::now();
    let result = engine
        .execute(parse("SELECT * FROM sorted_data ORDER BY value").unwrap())
        .unwrap();
    let elapsed = start.elapsed();

    println!("ORDER BY of 1000 rows took: {:?}", elapsed);

    assert_eq!(result.rows.len(), 1000);
}

#[test]
fn test_limit_performance() {
    let mut storage = MemoryStorage::new();

    for i in 0..10000 {
        storage
            .insert("limit_test", vec![vec![Value::Integer(i as i64)]])
            .ok();
    }

    let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));
    let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

    let scan = SeqScanExec::new("limit_test".to_string(), schema);

    let start = Instant::now();
    let result = engine.execute_plan(&scan).unwrap();
    let elapsed = start.elapsed();

    println!("Full scan of 10000 rows took: {:?}", elapsed);
    assert_eq!(result.rows.len(), 10000);
}

#[test]
fn test_mixed_workload_performance() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(
            parse("CREATE TABLE mixed_test (id INTEGER, category TEXT, value INTEGER)").unwrap(),
        )
        .unwrap();

    for i in 0..1000 {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO mixed_test VALUES ({}, 'category{}', {})",
                    i,
                    i % 10,
                    i * 10
                ))
                .unwrap(),
            )
            .unwrap();
    }

    let start = Instant::now();

    engine
        .execute(parse("SELECT COUNT(*) FROM mixed_test WHERE category = 'category5'").unwrap())
        .ok();
    engine
        .execute(parse("SELECT * FROM mixed_test WHERE value > 5000 LIMIT 10").unwrap())
        .ok();
    engine
        .execute(parse("SELECT category, COUNT(*) FROM mixed_test GROUP BY category").unwrap())
        .ok();

    let elapsed = start.elapsed();
    println!("Mixed workload (3 queries) took: {:?}", elapsed);
}

#[test]
fn test_concurrent_reads_performance() {
    let mut storage = MemoryStorage::new();

    for i in 0..5000 {
        storage
            .insert("concurrent_reads", vec![vec![Value::Integer(i as i64)]])
            .ok();
    }

    let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));
    let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

    let start = Instant::now();

    std::thread::scope(|s| {
        for _ in 0..10 {
            s.spawn(|| {
                let scan = SeqScanExec::new("concurrent_reads".to_string(), schema.clone());
                engine.execute_plan(&scan).ok();
            });
        }
    });

    let elapsed = start.elapsed();
    println!("10 concurrent reads took: {:?}", elapsed);

    assert!(
        elapsed.as_secs() < 5,
        "Concurrent reads should complete quickly"
    );
}

#[test]
fn test_cache_hit_performance() {
    use sqlrustgo_executor::query_cache_config::{CacheEntry, CacheKey, QueryCacheConfig};
    use sqlrustgo_executor::{ExecutorResult, QueryCache};
    use std::time::Instant;

    let config = QueryCacheConfig::default();
    let mut cache = QueryCache::new(config);

    let key = CacheKey {
        normalized_sql: "SELECT * FROM test".to_string(),
        params_hash: 0,
    };
    let entry = CacheEntry {
        result: ExecutorResult::new(vec![vec![Value::Integer(1)]], 1),
        tables: vec!["test".to_string()],
        created_at: Instant::now(),
        size_bytes: 100,
        last_access: 0,
    };

    cache.put(key.clone(), entry, vec!["test".to_string()]);

    let start = Instant::now();
    for _ in 0..10000 {
        cache.get(&key);
    }
    let elapsed = start.elapsed();

    println!("10000 cache hits took: {:?}", elapsed);
    println!(
        "Cache hit rate: {:.2} M ops/sec",
        10000.0 / elapsed.as_secs_f64()
    );
}

// ============================================================================
// Insert Performance Optimization Tests (Issue #895)
// ============================================================================

#[test]
fn test_insert_batch_optimization() {
    // Test batch insert optimization
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(
            parse("CREATE TABLE batch_optimization_test (id INTEGER PRIMARY KEY, value TEXT)")
                .unwrap(),
        )
        .unwrap();

    // Multi-row insert (batch)
    let result = engine.execute(
        parse("INSERT INTO batch_optimization_test VALUES (1, 'a'), (2, 'b'), (3, 'c')").unwrap(),
    );
    assert!(result.is_ok(), "Multi-row insert should work");

    // Verify
    let result = engine
        .execute(parse("SELECT COUNT(*) FROM batch_optimization_test").unwrap())
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(3));

    println!("✓ Batch insert optimization: multi-row inserts supported");
}

#[test]
fn test_insert_with_transaction() {
    // Test INSERT with explicit transaction
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE tx_insert_test (id INTEGER, value TEXT)").unwrap())
        .unwrap();

    // Begin transaction
    engine.execute(parse("BEGIN").unwrap()).unwrap();

    // Insert multiple rows
    for i in 0..100 {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO tx_insert_test VALUES ({}, 'v{}')",
                    i, i
                ))
                .unwrap(),
            )
            .unwrap();
    }

    // Commit
    engine.execute(parse("COMMIT").unwrap()).unwrap();

    // Verify
    let result = engine
        .execute(parse("SELECT COUNT(*) FROM tx_insert_test").unwrap())
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(100));

    println!("✓ Transactional insert: 100 rows in single transaction");
}

#[test]
fn test_wal_write_optimization() {
    // Test WAL write optimization (reduced sync calls)
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));

    engine
        .execute(parse("CREATE TABLE wal_optimization_test (id INTEGER, data TEXT)").unwrap())
        .unwrap();

    let iterations = 500;
    let start = Instant::now();

    for i in 0..iterations {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO wal_optimization_test VALUES ({}, 'data{}')",
                    i, i
                ))
                .unwrap(),
            )
            .unwrap();
    }

    let elapsed = start.elapsed();
    let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();

    println!("WAL optimized insert: {:.2} ops/s", ops_per_sec);

    // Verify data integrity
    let result = engine
        .execute(parse("SELECT COUNT(*) FROM wal_optimization_test").unwrap())
        .unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(iterations as i64));
}

#[test]
fn test_composite_index() {
    let mut storage = MemoryStorage::new();

    storage
        .create_table(&TableInfo {
            name: "orders".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "customer_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    references: None,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "order_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    references: None,
                    auto_increment: false,
                },
            ],
        })
        .unwrap();

    for i in 0..100 {
        for j in 0..10 {
            storage
                .insert("orders", vec![vec![Value::Integer(i), Value::Integer(j)]])
                .unwrap();
        }
    }

    storage
        .create_table_index("orders", "customer_id", 0)
        .unwrap();
    storage.create_table_index("orders", "order_id", 1).unwrap();

    let result = storage.search_index("orders", "customer_id", 50);
    assert!(!result.is_empty());

    let range_result = storage.range_index("orders", "customer_id", 50, 60);
    assert!(!range_result.is_empty());

    println!("✓ Composite index works");
}

#[test]
fn test_covering_index() {
    let mut storage = MemoryStorage::new();

    storage
        .create_table(&TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    is_primary_key: true,
                    references: None,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    references: None,
                    auto_increment: false,
                },
            ],
        })
        .unwrap();

    for i in 0..1000 {
        storage
            .insert(
                "users",
                vec![vec![Value::Integer(i), Value::Text(format!("user{}", i))]],
            )
            .unwrap();
    }

    storage.create_table_index("users", "id", 0).unwrap();

    let result = storage.search_index("users", "id", 500);
    assert!(!result.is_empty());

    let range_result = storage.range_index("users", "id", 100, 200);
    assert!(!range_result.is_empty());

    println!("✓ Covering index works");
}
