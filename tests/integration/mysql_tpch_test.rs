//! MySQL TPC-H Integration Tests
//!
//! Tests that TPC-H queries execute correctly on MySQL.
//! These tests will skip if MySQL is not available.

use mysql::prelude::Queryable;
use mysql::Pool;

fn get_mysql_pool() -> Option<Pool> {
    let config = sqlrustgo_bench::mysql_config::MySqlConfig::local();
    let conn_str = config.connection_string();
    mysql::Pool::new(conn_str.as_str()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mysql_connection() {
        if let Some(pool) = get_mysql_pool() {
            let mut conn = pool.get_conn().expect("Failed to get connection");
            let result: Result<Vec<u8>, _> = conn.query("SELECT 1");
            assert!(result.is_ok());
        } else {
            println!("MySQL not available, skipping test");
        }
    }

    #[test]
    fn test_mysql_tpch_q1() {
        let pool = match get_mysql_pool() {
            Some(p) => p,
            None => {
                println!("MySQL not available, skipping test");
                return;
            }
        };

        let mut conn = pool.get_conn().expect("Failed to get connection");

        // Create test table
        conn.query_drop("CREATE TABLE IF NOT EXISTS test_q1 (
            id INT,
            quantity INT,
            price DECIMAL(10,2)
        )").ok();

        // Insert test data
        conn.query_drop("INSERT INTO test_q1 VALUES (1, 10, 100.00)").ok();

        // Execute Q1-like aggregation
        let result: Result<Vec<(i32,)>, _> = conn.query(
            "SELECT SUM(quantity) as total_qty FROM test_q1"
        );

        assert!(result.is_ok());

        // Cleanup
        conn.query_drop("DROP TABLE test_q1").ok();
    }

    #[test]
    fn test_mysql_tpch_q6_aggregation() {
        let pool = match get_mysql_pool() {
            Some(p) => p,
            None => {
                println!("MySQL not available, skipping test");
                return;
            }
        };

        let mut conn = pool.get_conn().expect("Failed to get connection");

        // Create test table
        conn.query_drop("CREATE TABLE IF NOT EXISTS test_q6 (
            id INT,
            quantity INT,
            price DECIMAL(10,2),
            discount DECIMAL(10,2)
        )").ok();

        // Insert test data
        conn.query_drop("INSERT INTO test_q6 VALUES (1, 25, 1000.00, 0.10)").ok();
        conn.query_drop("INSERT INTO test_q6 VALUES (2, 30, 2000.00, 0.15)").ok();

        // Execute Q6-like query
        let result: Result<Vec<(mysql::Value,)>, _> = conn.query(
            "SELECT SUM(price * (1 - discount)) as revenue FROM test_q6 WHERE quantity > 20"
        );

        assert!(result.is_ok());

        // Cleanup
        conn.query_drop("DROP TABLE test_q6").ok();
    }

    #[test]
    fn test_mysql_tpch_join() {
        let pool = match get_mysql_pool() {
            Some(p) => p,
            None => {
                println!("MySQL not available, skipping test");
                return;
            }
        };

        let mut conn = pool.get_conn().expect("Failed to get connection");

        // Create test tables
        conn.query_drop("CREATE TABLE IF NOT EXISTS orders (
            order_id INT,
            customer_id INT,
            total_price DECIMAL(10,2)
        )").ok();

        conn.query_drop("CREATE TABLE IF NOT EXISTS customers (
            customer_id INT,
            name VARCHAR(100)
        )").ok();

        // Insert test data
        conn.query_drop("INSERT INTO orders VALUES (1, 100, 500.00)").ok();
        conn.query_drop("INSERT INTO customers VALUES (100, 'Alice')").ok();

        // Execute join query
        let result: Result<Vec<(String, mysql::Value)>, _> = conn.query(
            "SELECT c.name, o.total_price FROM orders o JOIN customers c ON o.customer_id = c.customer_id"
        );

        assert!(result.is_ok());

        // Cleanup
        conn.query_drop("DROP TABLE orders").ok();
        conn.query_drop("DROP TABLE customers").ok();
    }
}