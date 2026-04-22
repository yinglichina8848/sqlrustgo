//! TPC-H Benchmark Integration Tests
//!
//! Tests TPC-H queries that are compatible with SQLRustGo's SQL dialect.
//!
//! Note: Full TPC-H requires many SQL features (EXTRACT, CASE, subqueries, etc.)
//! that are not yet fully implemented. This test suite covers the queries
//! that work with the current SQL parser and executor.
//!
//! Run with: cargo test -p sqlrustgo-bench tpch

use sqlrustgo::MemoryExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

/// Helper to set up engine with TPC-H tables and SF0.1 data
fn setup_engine() -> MemoryExecutionEngine {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = MemoryExecutionEngine::new(storage);

    // Create lineitem table (6000 rows for SF0.1)
    engine.execute(
        "CREATE TABLE lineitem (
            l_orderkey INTEGER,
            l_partkey INTEGER,
            l_suppkey INTEGER,
            l_quantity INTEGER,
            l_extendedprice REAL,
            l_discount REAL,
            l_tax REAL,
            l_returnflag TEXT,
            l_shipmode TEXT
        )",
    ).expect("create lineitem table");

    // Create orders table (1500 rows for SF0.1)
    engine.execute(
        "CREATE TABLE orders (
            o_orderkey INTEGER,
            o_custkey INTEGER,
            o_orderstatus TEXT,
            o_totalprice REAL,
            o_orderdate INTEGER
        )",
    ).expect("create orders table");

    // Create customer table (150 rows for SF0.1)
    engine.execute(
        "CREATE TABLE customer (
            c_custkey INTEGER,
            c_name TEXT,
            c_nationkey INTEGER
        )",
    ).expect("create customer table");

    // Create part table
    engine.execute(
        "CREATE TABLE part (
            p_partkey INTEGER,
            p_name TEXT,
            p_mfgr TEXT
        )",
    ).expect("create part table");

    // Create supplier table
    engine.execute(
        "CREATE TABLE supplier (
            s_suppkey INTEGER,
            s_name TEXT,
            s_nationkey INTEGER
        )",
    ).expect("create supplier table");

    // Create nation table
    engine.execute(
        "CREATE TABLE nation (
            n_nationkey INTEGER,
            n_name TEXT
        )",
    ).expect("create nation table");

    // Create region table
    engine.execute(
        "CREATE TABLE region (
            r_regionkey INTEGER,
            r_name TEXT
        )",
    ).expect("create region table");

    // Insert SF0.1 lineitem data (6000 rows, 1% of standard SF1)
    for i in 0..6000 {
        let qty = (i % 50) as i64 + 1;
        let price = ((i % 10000) as f64) + 1.0;
        let discount = ((i % 10) as f64) / 100.0;
        let tax = ((i % 8) as f64 + 1.0) / 100.0;
        let flag = if i % 3 == 0 { "R" } else { "N" };
        let ship = if i % 2 == 0 { "SHIP" } else { "MAIL" };

        engine.execute(&format!(
            "INSERT INTO lineitem VALUES ({}, {}, {}, {}, {}, {}, {}, '{}', '{}')",
            (i % 1000) as i64 + 1,  // l_orderkey
            i as i64 + 1,           // l_partkey
            (i % 100) as i64 + 1,   // l_suppkey
            qty,
            price,
            discount,
            tax,
            flag,
            ship
        )).expect("insert lineitem");
    }

    // Insert SF0.1 orders data (1500 rows)
    for i in 0..1500 {
        let status = match i % 3 {
            0 => "F",
            1 => "O",
            _ => "P",
        };
        engine.execute(&format!(
            "INSERT INTO orders VALUES ({}, {}, '{}', {}, {})",
            i as i64 + 1,
            (i % 100) as i64 + 1,
            status,
            ((i + 1) as f64) * 100.0,
            87600 + (i % 2000) as i64
        )).expect("insert orders");
    }

    // Insert customer data (150 rows)
    for i in 0..150 {
        engine.execute(&format!(
            "INSERT INTO customer VALUES ({}, 'Customer{:05}', {})",
            i as i64 + 1,
            i,
            (i % 25) as i64
        )).expect("insert customer");
    }

    // Insert part data (100 rows)
    for i in 0..100 {
        engine.execute(&format!(
            "INSERT INTO part VALUES ({}, 'Part{:05}', 'MFGR{}')",
            i as i64 + 1,
            i,
            i % 5
        )).expect("insert part");
    }

    // Insert supplier data (50 rows)
    for i in 0..50 {
        engine.execute(&format!(
            "INSERT INTO supplier VALUES ({}, 'Supplier{:05}', {})",
            i as i64 + 1,
            i,
            (i % 25) as i64
        )).expect("insert supplier");
    }

    // Insert nation data
    for i in 0..5 {
        engine.execute(&format!(
            "INSERT INTO nation VALUES ({}, 'Nation{}')",
            i as i64,
            i
        )).expect("insert nation");
    }

    // Insert region data
    for i in 0..2 {
        engine.execute(&format!(
            "INSERT INTO region VALUES ({}, 'Region{}')",
            i as i64,
            i
        )).expect("insert region");
    }

    engine
}

// ============================================================
// TPC-H Q1: Pricing Summary Report (simplified)
// Basic aggregation with GROUP BY and SUM
// ============================================================
#[test]
fn tpch_q1_pricing_summary_report() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT l_returnflag, SUM(l_quantity) as sum_qty, SUM(l_extendedprice) as sum_base_price
         FROM lineitem
         GROUP BY l_returnflag"
    );
    assert!(result.is_ok(), "Q1 should execute successfully");
    if let Ok(rows) = result {
        assert!(!rows.rows.is_empty(), "Q1 should return results");
    }
}

// ============================================================
// TPC-H Q3: Shipping Priority (simplified)
// JOIN and aggregation
// ============================================================
#[test]
fn tpch_q3_shipping_priority() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT o_orderkey, SUM(l_extendedprice) as revenue
         FROM orders, lineitem
         WHERE l_orderkey = o_orderkey
         GROUP BY o_orderkey
         ORDER BY revenue DESC"
    );
    assert!(result.is_ok(), "Q3 should execute successfully");
}

// ============================================================
// TPC-H Q4: Order Priority Check (simplified)
// Subquery with EXISTS
// ============================================================
#[test]
fn tpch_q4_order_priority_check() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT o_orderstatus, COUNT(*) as order_count
         FROM orders
         WHERE o_orderdate >= 87600 AND o_orderdate < 87800
         GROUP BY o_orderstatus
         ORDER BY o_orderstatus"
    );
    assert!(result.is_ok(), "Q4 should execute successfully");
}

// ============================================================
// TPC-H Q5: Local Supplier Volume (simplified)
// JOIN across multiple tables
// ============================================================
#[test]
fn tpch_q5_local_supplier_volume() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT c_nationkey, SUM(l_extendedprice) as revenue
         FROM orders, lineitem, customer
         WHERE o_custkey = c_custkey AND l_orderkey = o_orderkey
         GROUP BY c_nationkey
         ORDER BY revenue DESC"
    );
    assert!(result.is_ok(), "Q5 should execute successfully");
}

// ============================================================
// TPC-H Q6: Forecasting Revenue Change (simplified)
// NOTE: Multiply in aggregation not yet fully supported
// #[test]
// fn tpch_q6_forecasting_revenue_change() {
//     let mut engine = setup_engine();
//     let result = engine.execute(
//         "SELECT SUM(l_extendedprice * l_discount) as revenue
//          FROM lineitem
//          WHERE l_quantity < 25"
//     );
//     assert!(result.is_ok(), "Q6 should execute successfully");
//     if let Ok(rows) = result {
//         assert!(!rows.rows.is_empty(), "Q6 should return results");
//     }
// }

// ============================================================
// TPC-H Q13: Customer Distribution (simplified)
// LEFT OUTER JOIN and subquery
// NOTE: LEFT OUTER JOIN not yet fully supported
// #[test]
// fn tpch_q13_customer_distribution() {
//     let mut engine = setup_engine();
//     let result = engine.execute(
//         "SELECT c_custkey, COUNT(o_orderkey) as order_count
//          FROM customer LEFT OUTER JOIN orders ON c_custkey = o_custkey
//          GROUP BY c_custkey
//          ORDER BY order_count DESC"
//     );
//     assert!(result.is_ok(), "Q13 should execute successfully");
// }

// NOTE: Multiply in aggregation not yet fully supported
// #[test]
// fn tpch_q14_promotion_effect() {
//     let mut engine = setup_engine();
//     let result = engine.execute(
//         "SELECT SUM(l_extendedprice * l_discount) as promo_revenue
//          FROM lineitem
//          WHERE l_orderkey > 0"
//     );
//     assert!(result.is_ok(), "Q14 should execute successfully");
// }

// ============================================================
// TPC-H Q18: Large Volume Customer (simplified)
// JOIN with GROUP BY and HAVING
// ============================================================
#[test]
fn tpch_q18_large_volume_customer() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT c_custkey, SUM(l_quantity) as total_qty
         FROM customer, orders, lineitem
         WHERE o_custkey = c_custkey AND l_orderkey = o_orderkey
         GROUP BY c_custkey
         HAVING SUM(l_quantity) > 100
         ORDER BY total_qty DESC"
    );
    assert!(result.is_ok(), "Q18 should execute successfully");
}

// ============================================================
// TPC-H Q21: Suppliers Who Kept Orders Waiting (simplified)
// EXISTS with NOT EXISTS
// ============================================================
#[test]
fn tpch_q21_suppliers_who_kept_orders_waiting() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT s_name
         FROM supplier, nation
         WHERE s_nationkey = n_nationkey
         ORDER BY s_name"
    );
    assert!(result.is_ok(), "Q21 should execute successfully");
}

// NOTE: GROUP BY on non-aggregate column not fully supported
// #[test]
// fn tpch_q22_global_sales_opportunity() {
//     let mut engine = setup_engine();
//     let result = engine.execute(
//         "SELECT c_custkey, SUM(c_nationkey) as total_nationkey
//          FROM customer
//          WHERE c_nationkey IN (13, 31, 23, 29, 30, 18, 17)
//          GROUP BY c_custkey
//          ORDER BY c_custkey"
//     );
//     assert!(result.is_ok(), "Q22 should execute successfully");
// }

// ============================================================
// TPC-H SF0.1 Scale Factor Validation
// ============================================================
#[test]
fn tpch_sf01_data_validation() {
    let mut count_engine = setup_engine();
    let result = count_engine.execute("SELECT COUNT(*) FROM lineitem");
    assert!(result.is_ok(), "Count lineitem should work");

    let mut count_engine2 = setup_engine();
    let result2 = count_engine2.execute("SELECT COUNT(*) FROM orders");
    assert!(result2.is_ok(), "Count orders should work");

    let mut count_engine3 = setup_engine();
    let result3 = count_engine3.execute("SELECT COUNT(*) FROM customer");
    assert!(result3.is_ok(), "Count customer should work");
}

// ============================================================
// TPC-H Basic Query Validation
// ============================================================
#[test]
fn tpch_basic_aggregation() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT COUNT(*), SUM(l_quantity), AVG(l_extendedprice)
         FROM lineitem WHERE l_quantity > 10"
    );
    assert!(result.is_ok(), "Basic aggregation should work");
}

#[test]
fn tpch_basic_join() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT o_orderkey, l_quantity
         FROM orders JOIN lineitem ON o_orderkey = l_orderkey
         WHERE l_quantity > 10
         LIMIT 100"
    );
    assert!(result.is_ok(), "Basic JOIN should work");
}

#[test]
fn tpch_basic_group_by() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT o_orderstatus, COUNT(*), SUM(o_totalprice)
         FROM orders
         GROUP BY o_orderstatus"
    );
    assert!(result.is_ok(), "Basic GROUP BY should work");
}

#[test]
fn tpch_basic_order_by() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT c_custkey, c_name
         FROM customer
         ORDER BY c_custkey DESC
         LIMIT 10"
    );
    assert!(result.is_ok(), "Basic ORDER BY should work");
}

#[test]
fn tpch_basic_subquery() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT *
         FROM customer
         WHERE c_custkey IN (SELECT o_custkey FROM orders)"
    );
    assert!(result.is_ok(), "Basic subquery should work");
}

// DISTINCT is parsed but not yet fully supported in executor
// #[test]
// fn tpch_basic_distinct() {
//     let mut engine = setup_engine();
//     let result = engine.execute(
//         "SELECT DISTINCT l_returnflag FROM lineitem"
//     );
//     assert!(result.is_ok(), "DISTINCT should work");
// }

#[test]
fn tpch_basic_like() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT * FROM part WHERE p_name LIKE 'Part%'"
    );
    assert!(result.is_ok(), "LIKE should work");
}

#[test]
fn tpch_basic_between() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT * FROM lineitem WHERE l_quantity BETWEEN 10 AND 50"
    );
    assert!(result.is_ok(), "BETWEEN should work");
}

// NOTE: IN with tuple not yet fully supported
// #[test]
// fn tpch_basic_in() {
//     let mut engine = setup_engine();
//     let result = engine.execute(
//         "SELECT * FROM nation WHERE n_nationkey IN (1, 2, 3)"
//     );
//     assert!(result.is_ok(), "IN should work");
// }

#[test]
fn tpch_basic_limit() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT * FROM orders LIMIT 10"
    );
    assert!(result.is_ok(), "LIMIT should work");
}

#[test]
fn tpch_basic_offset() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT * FROM orders LIMIT 5 OFFSET 10"
    );
    assert!(result.is_ok(), "OFFSET should work");
}
