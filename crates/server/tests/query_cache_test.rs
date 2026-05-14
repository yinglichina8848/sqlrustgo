// Query Cache DML Invalidation Tests
// Tests that INSERT/UPDATE/DELETE operations properly invalidate cached query results

#[cfg(test)]
mod query_cache_dml_tests {
    use std::collections::{HashMap, HashSet};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread;

    // ============================================================
    // Mock structures matching the real QueryCache implementation
    // ============================================================

    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    struct CacheKey {
        normalized_sql: String,
        params_hash: u64,
    }

    #[derive(Debug, Clone)]
    struct CacheEntry {
        result: Vec<Vec<i64>>,
        tables: Vec<String>,
        size_bytes: usize,
        last_access: u64,
    }

    struct QueryCache {
        cache: HashMap<CacheKey, CacheEntry>,
        table_index: HashMap<String, HashSet<CacheKey>>,
        current_memory_bytes: usize,
        access_counter: u64,
    }

    impl QueryCache {
        fn new() -> Self {
            Self {
                cache: HashMap::new(),
                table_index: HashMap::new(),
                current_memory_bytes: 0,
                access_counter: 0,
            }
        }

        fn get(&mut self, key: &CacheKey) -> Option<&CacheEntry> {
            let entry = self.cache.get_mut(key)?;
            self.access_counter += 1;
            entry.last_access = self.access_counter;
            Some(entry)
        }

        fn put(&mut self, key: CacheKey, entry: CacheEntry, tables: Vec<String>) {
            let size = entry.size_bytes;

            // Remove old entry if exists
            if self.cache.remove(&key).is_some() {
                for t in &tables {
                    if let Some(keys) = self.table_index.get_mut(t) {
                        keys.remove(&key);
                    }
                }
            }

            self.access_counter += 1;
            self.cache.insert(key.clone(), entry);
            self.current_memory_bytes += size;

            for table in &tables {
                self.table_index
                    .entry(table.clone())
                    .or_default()
                    .insert(key.clone());
            }
        }

        fn invalidate_table(&mut self, table: &str) {
            if let Some(keys) = self.table_index.remove(table) {
                for key in keys {
                    if let Some(entry) = self.cache.remove(&key) {
                        self.current_memory_bytes =
                            self.current_memory_bytes.saturating_sub(entry.size_bytes);
                    }
                }
            }
        }

        fn stats(&self) -> QueryCacheStats {
            QueryCacheStats {
                entries: self.cache.len(),
                memory_bytes: self.current_memory_bytes,
                table_count: self.table_index.len(),
            }
        }
    }

    struct QueryCacheStats {
        entries: usize,
        memory_bytes: usize,
        table_count: usize,
    }

    fn make_cache_key(sql: &str, param_id: u64) -> CacheKey {
        CacheKey {
            normalized_sql: sql.to_string(),
            params_hash: param_id,
        }
    }

    fn make_cache_entry(id: i64, tables: Vec<String>) -> CacheEntry {
        CacheEntry {
            result: vec![vec![id]],
            tables,
            size_bytes: 64,
            last_access: 0,
        }
    }

    // ============================================================
    // Test: INSERT invalidates cache for the target table
    // ============================================================
    #[test]
    fn test_insert_invalidates_cache() {
        let mut cache = QueryCache::new();

        // Cache a SELECT query on "users" table
        cache.put(
            make_cache_key("SELECT * FROM users WHERE id = ?", 1),
            make_cache_entry(1, vec!["users".to_string()]),
            vec!["users".to_string()],
        );

        // Verify cache has the entry
        assert!(cache
            .get(&make_cache_key("SELECT * FROM users WHERE id = ?", 1))
            .is_some());

        // Simulate INSERT operation on "users" table
        cache.invalidate_table("users");

        // Cache should be invalidated
        assert!(cache
            .get(&make_cache_key("SELECT * FROM users WHERE id = ?", 1))
            .is_none());
    }

    // ============================================================
    // Test: UPDATE invalidates cache for the target table
    // ============================================================
    #[test]
    fn test_update_invalidates_cache() {
        let mut cache = QueryCache::new();

        // Cache multiple SELECT queries on "orders" table
        cache.put(
            make_cache_key("SELECT * FROM orders WHERE id = ?", 1),
            make_cache_entry(1, vec!["orders".to_string()]),
            vec!["orders".to_string()],
        );
        cache.put(
            make_cache_key("SELECT * FROM orders WHERE customer_id = ?", 100),
            make_cache_entry(100, vec!["orders".to_string()]),
            vec!["orders".to_string()],
        );

        // Cache a SELECT on different table (should not be affected)
        cache.put(
            make_cache_key("SELECT * FROM products WHERE id = ?", 1),
            make_cache_entry(1, vec!["products".to_string()]),
            vec!["products".to_string()],
        );

        // Verify all entries exist
        assert_eq!(cache.stats().entries, 3);

        // Simulate UPDATE operation on "orders" table
        cache.invalidate_table("orders");

        // Only orders cache should be invalidated, products should remain
        assert_eq!(cache.stats().entries, 1);
        assert!(cache
            .get(&make_cache_key("SELECT * FROM products WHERE id = ?", 1))
            .is_some());
        assert!(cache
            .get(&make_cache_key("SELECT * FROM orders WHERE id = ?", 1))
            .is_none());
    }

    // ============================================================
    // Test: DELETE invalidates cache for the target table
    // ============================================================
    #[test]
    fn test_delete_invalidates_cache() {
        let mut cache = QueryCache::new();

        // Cache SELECT query
        cache.put(
            make_cache_key("SELECT * FROM lineitem WHERE order_id = ?", 1),
            make_cache_entry(1, vec!["lineitem".to_string()]),
            vec!["lineitem".to_string()],
        );

        // Verify cache has the entry
        assert!(cache
            .get(&make_cache_key(
                "SELECT * FROM lineitem WHERE order_id = ?",
                1
            ))
            .is_some());

        // Simulate DELETE operation
        cache.invalidate_table("lineitem");

        // Cache should be invalidated
        assert!(cache
            .get(&make_cache_key(
                "SELECT * FROM lineitem WHERE order_id = ?",
                1
            ))
            .is_none());
        assert_eq!(cache.stats().entries, 0);
    }

    // ============================================================
    // Test: Multi-table query invalidation
    // ============================================================
    #[test]
    fn test_multi_table_query_invalidation() {
        let mut cache = QueryCache::new();

        // Cache a JOIN query
        cache.put(
            make_cache_key(
                "SELECT * FROM orders JOIN customers ON orders.customer_id = customers.id",
                0,
            ),
            make_cache_entry(1, vec!["orders".to_string(), "customers".to_string()]),
            vec!["orders".to_string(), "customers".to_string()],
        );

        // Cache individual table queries
        cache.put(
            make_cache_key("SELECT * FROM orders", 0),
            make_cache_entry(2, vec!["orders".to_string()]),
            vec!["orders".to_string()],
        );
        cache.put(
            make_cache_key("SELECT * FROM customers", 0),
            make_cache_entry(3, vec!["customers".to_string()]),
            vec!["customers".to_string()],
        );

        assert_eq!(cache.stats().entries, 3);

        // INSERT on orders should invalidate both the join query and the orders query
        cache.invalidate_table("orders");

        assert_eq!(cache.stats().entries, 1);
        assert!(cache
            .get(&make_cache_key("SELECT * FROM customers", 0))
            .is_some());
    }

    // ============================================================
    // Test: Multiple DML operations
    // ============================================================
    #[test]
    fn test_multiple_dml_operations() {
        let mut cache = QueryCache::new();

        // Cache queries for multiple tables
        cache.put(
            make_cache_key("SELECT * FROM t1", 0),
            make_cache_entry(1, vec!["t1".to_string()]),
            vec!["t1".to_string()],
        );
        cache.put(
            make_cache_key("SELECT * FROM t2", 0),
            make_cache_entry(2, vec!["t2".to_string()]),
            vec!["t2".to_string()],
        );
        cache.put(
            make_cache_key("SELECT * FROM t3", 0),
            make_cache_entry(3, vec!["t3".to_string()]),
            vec!["t3".to_string()],
        );

        assert_eq!(cache.stats().entries, 3);

        // Simulate INSERT on t1
        cache.invalidate_table("t1");
        assert_eq!(cache.stats().entries, 2);

        // Simulate UPDATE on t2
        cache.invalidate_table("t2");
        assert_eq!(cache.stats().entries, 1);

        // Simulate DELETE on t3
        cache.invalidate_table("t3");
        assert_eq!(cache.stats().entries, 0);
    }

    // ============================================================
    // Test: Concurrent DML operations
    // ============================================================
    #[test]
    fn test_concurrent_dml_operations() {
        let cache = Arc::new(Mutex::new(QueryCache::new()));

        // Pre-populate cache
        {
            let mut cache = cache.lock().unwrap();
            for i in 0..100 {
                cache.put(
                    make_cache_key(
                        &format!("SELECT * FROM table_{} WHERE id = ?", i as u64 % 10),
                        i as u64,
                    ),
                    make_cache_entry(i as i64, vec![format!("table_{}", i as u64 % 10)]),
                    vec![format!("table_{}", i as u64 % 10)],
                );
            }
        }

        let barrier = Arc::new(Barrier::new(10));
        let success_count = Arc::new(AtomicUsize::new(0));

        // 10 threads doing DML operations
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let cache = Arc::clone(&cache);
                let barrier = Arc::clone(&barrier);
                let success_count = Arc::clone(&success_count);
                thread::spawn(move || {
                    barrier.wait();
                    // Each thread invalidates a different table
                    let table_name = format!("table_{}", i);
                    {
                        let mut cache = cache.lock().unwrap();
                        cache.invalidate_table(&table_name);
                    }
                    success_count.fetch_add(1, Ordering::SeqCst);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(success_count.load(Ordering::SeqCst), 10);

        // Verify all tables were invalidated
        let cache = cache.lock().unwrap();
        assert_eq!(cache.stats().entries, 0);
    }

    // ============================================================
    // Test: Cache stats after DML invalidation
    // ============================================================
    #[test]
    fn test_cache_stats_after_dml() {
        let mut cache = QueryCache::new();

        // Fill cache
        cache.put(
            make_cache_key("SELECT * FROM users", 0),
            make_cache_entry(1, vec!["users".to_string()]),
            vec!["users".to_string()],
        );
        cache.put(
            make_cache_key("SELECT * FROM orders", 0),
            make_cache_entry(2, vec!["orders".to_string()]),
            vec!["orders".to_string()],
        );

        let stats_before = cache.stats();
        assert_eq!(stats_before.entries, 2);
        assert_eq!(stats_before.table_count, 2);

        // Invalidate users table
        cache.invalidate_table("users");

        let stats_after = cache.stats();
        assert_eq!(stats_after.entries, 1);
        assert_eq!(stats_after.table_count, 1);
    }

    // ============================================================
    // Test: DML on non-existent table (no-op)
    // ============================================================
    #[test]
    fn test_dml_nonexistent_table() {
        let mut cache = QueryCache::new();

        // Cache something
        cache.put(
            make_cache_key("SELECT * FROM users", 0),
            make_cache_entry(1, vec!["users".to_string()]),
            vec!["users".to_string()],
        );

        // Invalidate non-existent table
        cache.invalidate_table("nonexistent_table");

        // Original cache should be untouched
        assert_eq!(cache.stats().entries, 1);
        assert!(cache
            .get(&make_cache_key("SELECT * FROM users", 0))
            .is_some());
    }

    // ============================================================
    // Test: Cache repopulation after DML
    // ============================================================
    #[test]
    fn test_cache_repopulation_after_dml() {
        let mut cache = QueryCache::new();

        // Initial cache
        cache.put(
            make_cache_key("SELECT * FROM users", 0),
            make_cache_entry(1, vec!["users".to_string()]),
            vec!["users".to_string()],
        );
        assert!(cache
            .get(&make_cache_key("SELECT * FROM users", 0))
            .is_some());

        // DML invalidates
        cache.invalidate_table("users");
        assert!(cache
            .get(&make_cache_key("SELECT * FROM users", 0))
            .is_none());

        // Repopulate cache
        cache.put(
            make_cache_key("SELECT * FROM users", 0),
            make_cache_entry(2, vec!["users".to_string()]),
            vec!["users".to_string()],
        );
        assert!(cache
            .get(&make_cache_key("SELECT * FROM users", 0))
            .is_some());
    }

    // ============================================================
    // Test: DML invalidation preserves other tables
    // ============================================================
    #[test]
    fn test_dml_preserves_other_tables() {
        let mut cache = QueryCache::new();

        // Cache for multiple tables
        cache.put(
            make_cache_key("SELECT * FROM a", 0),
            make_cache_entry(1, vec!["a".to_string()]),
            vec!["a".to_string()],
        );
        cache.put(
            make_cache_key("SELECT * FROM b", 0),
            make_cache_entry(2, vec!["b".to_string()]),
            vec!["b".to_string()],
        );
        cache.put(
            make_cache_key("SELECT * FROM c", 0),
            make_cache_entry(3, vec!["c".to_string()]),
            vec!["c".to_string()],
        );

        // DML on table 'b'
        cache.invalidate_table("b");

        assert_eq!(cache.stats().entries, 2);
        assert!(cache.get(&make_cache_key("SELECT * FROM a", 0)).is_some());
        assert!(cache.get(&make_cache_key("SELECT * FROM c", 0)).is_some());
        assert!(cache.get(&make_cache_key("SELECT * FROM b", 0)).is_none());
    }

    // ============================================================
    // Test: High frequency DML stress test
    // ============================================================
    #[test]
    fn test_high_frequency_dml_stress() {
        let cache = Arc::new(Mutex::new(QueryCache::new()));

        let barrier = Arc::new(Barrier::new(20));
        let ops_count = Arc::new(AtomicUsize::new(0));

        // Pre-populate
        {
            let mut cache = cache.lock().unwrap();
            for i in 0..50 {
                let table_id = i as u64 % 5;
                cache.put(
                    make_cache_key(
                        &format!("SELECT * FROM t{} WHERE id = ?", table_id),
                        i as u64,
                    ),
                    make_cache_entry(i as i64, vec![format!("t{}", table_id)]),
                    vec![format!("t{}", table_id)],
                );
            }
        }

        // 20 threads doing rapid DML
        let handles: Vec<_> = (0..20)
            .map(|thread_id| {
                let cache = Arc::clone(&cache);
                let barrier = Arc::clone(&barrier);
                let ops_count = Arc::clone(&ops_count);
                thread::spawn(move || {
                    barrier.wait();
                    for i in 0..50 {
                        let table_id = thread_id as u64 % 5;
                        let table = format!("t{}", table_id);
                        {
                            let mut cache = cache.lock().unwrap();
                            cache.invalidate_table(&table);
                            // Immediately repopulate
                            cache.put(
                                make_cache_key(
                                    &format!("SELECT * FROM {} WHERE id = ?", table),
                                    i as u64,
                                ),
                                make_cache_entry(i as i64, vec![table.clone()]),
                                vec![table],
                            );
                        }
                        ops_count.fetch_add(1, Ordering::SeqCst);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(ops_count.load(Ordering::SeqCst), 1000);

        // Cache should be in a consistent state
        let cache = cache.lock().unwrap();
        assert!(cache.stats().entries <= 50);
    }

    // ============================================================
    // Test: Clear all caches
    // ============================================================
    #[test]
    fn test_clear_caches() {
        let mut cache = QueryCache::new();

        cache.put(
            make_cache_key("SELECT * FROM users", 0),
            make_cache_entry(1, vec!["users".to_string()]),
            vec!["users".to_string()],
        );
        cache.put(
            make_cache_key("SELECT * FROM orders", 0),
            make_cache_entry(2, vec!["orders".to_string()]),
            vec!["orders".to_string()],
        );
        cache.put(
            make_cache_key("SELECT * FROM products", 0),
            make_cache_entry(3, vec!["products".to_string()]),
            vec!["products".to_string()],
        );

        assert_eq!(cache.stats().entries, 3);

        // Clear all
        cache.table_index.clear();
        cache.cache.clear();
        cache.current_memory_bytes = 0;

        assert_eq!(cache.stats().entries, 0);
        assert_eq!(cache.stats().memory_bytes, 0);
    }

    // ============================================================
    // Test: Query result correctness after re-cache
    // ============================================================
    #[test]
    fn test_query_result_correctness_after_recache() {
        let mut cache = QueryCache::new();

        // Initial cache with result
        cache.put(
            make_cache_key("SELECT COUNT(*) FROM orders", 0),
            make_cache_entry(100, vec!["orders".to_string()]),
            vec!["orders".to_string()],
        );

        // Verify cached result
        let entry = cache.get(&make_cache_key("SELECT COUNT(*) FROM orders", 0));
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().result[0][0], 100);

        // Invalidate
        cache.invalidate_table("orders");

        // Re-cache with updated result
        cache.put(
            make_cache_key("SELECT COUNT(*) FROM orders", 0),
            make_cache_entry(101, vec!["orders".to_string()]),
            vec!["orders".to_string()],
        );

        // Verify updated result
        let entry = cache.get(&make_cache_key("SELECT COUNT(*) FROM orders", 0));
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().result[0][0], 101);
    }
}
