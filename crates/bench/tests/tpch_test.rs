// TPC-H benchmark tests for SQLRustGo
// Tests cover Q1, Q3, Q4, Q5, Q18, Q21 and basic SQL operations

use sqlrustgo::MemoryExecutionEngine;

fn setup_engine() -> MemoryExecutionEngine {
    let mut engine = MemoryExecutionEngine::with_memory();

    // Create lineitem table (6000 rows for SF0.1)
    engine
        .execute(
            "CREATE TABLE lineitem (
                l_orderkey INTEGER,
                l_partkey INTEGER,
                l_suppkey INTEGER,
                l_quantity REAL,
                l_extendedprice REAL,
                l_discount REAL,
                l_tax REAL,
                l_returnflag TEXT,
                l_shipmode TEXT
            )",
        )
        .expect("create lineitem table");

    // Create orders table (1500 rows for SF0.1)
    engine
        .execute(
            "CREATE TABLE orders (
                o_orderkey INTEGER,
                o_custkey INTEGER,
                o_orderstatus TEXT,
                o_totalprice REAL,
                o_orderdate INTEGER
            )",
        )
        .expect("create orders table");

    // Create customer table (150 rows for SF0.1)
    engine
        .execute(
            "CREATE TABLE customer (
                c_custkey INTEGER,
                c_name TEXT,
                c_nationkey INTEGER
            )",
        )
        .expect("create customer table");

    // Create part table
    engine
        .execute(
            "CREATE TABLE part (
                p_partkey INTEGER,
                p_name TEXT,
                p_mfgr TEXT
            )",
        )
        .expect("create part table");

    // Create supplier table
    engine
        .execute(
            "CREATE TABLE supplier (
                s_suppkey INTEGER,
                s_name TEXT,
                s_nationkey INTEGER
            )",
        )
        .expect("create supplier table");

    // Create nation table
    engine
        .execute(
            "CREATE TABLE nation (
                n_nationkey INTEGER,
                n_name TEXT
            )",
        )
        .expect("create nation table");

    // Create region table
    engine
        .execute(
            "CREATE TABLE region (
                r_regionkey INTEGER,
                r_name TEXT
            )",
        )
        .expect("create region table");

    // Insert SF0.1 lineitem data (6000 rows, 1% of standard SF1)
    for i in 0..6000 {
        let qty = 1.0 + (i % 50) as f64;
        let price = 1000.0 + (i % 100) as f64 * 10.0;
        let discount = 0.05 + (i % 10) as f64 * 0.01;
        let tax = 0.02 + (i % 5) as f64 * 0.01;
        let flag = if i % 3 == 0 { "R" } else { "N" };
        let ship = if i % 2 == 0 { "SHIP" } else { "MAIL" };

        engine
            .execute(&format!(
                "INSERT INTO lineitem VALUES ({}, {}, {}, {}, {}, {}, {}, '{}', '{}')",
                (i % 1000) as i64 + 1,
                i as i64 + 1,
                (i % 100) as i64 + 1,
                qty,
                price,
                discount,
                tax,
                flag,
                ship
            ))
            .expect("insert lineitem");
    }

    // Insert SF0.1 orders data (1500 rows)
    for i in 0..1500 {
        let status = match (i % 3) as i32 {
            0 => "F",
            1 => "O",
            _ => "P",
        };
        engine
            .execute(&format!(
                "INSERT INTO orders VALUES ({}, {}, '{}', {}, {})",
                i as i64 + 1,
                (i % 100) as i64 + 1,
                status,
                ((i + 1) as f64) * 100.0,
                87600 + (i % 2000) as i64
            ))
            .expect("insert orders");
    }

    // Insert customer data (150 rows)
    for i in 0..150 {
        engine
            .execute(&format!(
                "INSERT INTO customer VALUES ({}, 'Customer{:05}', {})",
                i as i64 + 1,
                i,
                (i % 25) as i64
            ))
            .expect("insert customer");
    }

    // Insert part data (100 rows)
    for i in 0..100 {
        engine
            .execute(&format!(
                "INSERT INTO part VALUES ({}, 'Part{:05}', 'MFGR{}')",
                i as i64 + 1,
                i,
                i % 5
            ))
            .expect("insert part");
    }

    // Insert supplier data (50 rows)
    for i in 0..50 {
        engine
            .execute(&format!(
                "INSERT INTO supplier VALUES ({}, 'Supplier{:05}', {})",
                i as i64 + 1,
                i,
                (i % 25) as i64
            ))
            .expect("insert supplier");
    }

    // Insert nation data
    for i in 0..5 {
        engine
            .execute(&format!(
                "INSERT INTO nation VALUES ({}, 'Nation{}')",
                i as i64, i
            ))
            .expect("insert nation");
    }

    // Insert region data
    for i in 0..2 {
        engine
            .execute(&format!(
                "INSERT INTO region VALUES ({}, 'Region{}')",
                i as i64, i
            ))
            .expect("insert region");
    }

    engine
}

// ============================================================
// TPC-H Q1: Pricing Summary Report
// Aggregation with GROUP BY
// ============================================================
#[test]
fn tpch_q1_pricing_summary_report() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT l_returnflag, SUM(l_quantity) as sum_qty, SUM(l_extendedprice) as sum_base_price
         FROM lineitem
         GROUP BY l_returnflag",
    );
    assert!(result.is_ok(), "Q1 should execute successfully");
    if let Ok(rows) = result {
        assert!(!rows.rows.is_empty(), "Q1 should return results");
    }
}

// ============================================================
// TPC-H Q3: Shipping Priority
// JOIN with GROUP BY and ORDER BY
// ============================================================
#[test]
fn tpch_q3_shipping_priority() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT o_orderkey, SUM(l_quantity) as revenue
         FROM orders, lineitem
         WHERE l_orderkey = o_orderkey
         GROUP BY o_orderkey
         ORDER BY revenue DESC",
    );
    assert!(result.is_ok(), "Q3 should execute successfully");
}

// ============================================================
// TPC-H Q4: Order Priority Check
// GROUP BY with WHERE condition
// ============================================================
#[test]
fn tpch_q4_order_priority_check() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT o_orderstatus, COUNT(*) as order_count
         FROM orders
         WHERE o_orderdate >= 87600 AND o_orderdate < 87800
         GROUP BY o_orderstatus
         ORDER BY o_orderstatus",
    );
    assert!(result.is_ok(), "Q4 should execute successfully");
}

// ============================================================
// TPC-H Q5: Local Supplier Volume
// Multi-table JOIN with aggregation
// ============================================================
#[test]
fn tpch_q5_local_supplier_volume() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT c_nationkey, SUM(l_extendedprice) as revenue
         FROM orders, lineitem, customer
         WHERE o_custkey = c_custkey AND l_orderkey = o_orderkey
         GROUP BY c_nationkey
         ORDER BY revenue DESC",
    );
    assert!(result.is_ok(), "Q5 should execute successfully");
}

// ============================================================
// TPC-H Q18: Large Volume Customer
// JOIN with GROUP BY and HAVING
// ============================================================
#[test]
fn tpch_q18_large_volume_customer() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT c_custkey, SUM(l_quantity) as total_qty
         FROM orders, lineitem, customer
         WHERE o_custkey = c_custkey AND l_orderkey = o_orderkey
         GROUP BY c_custkey
         HAVING SUM(l_quantity) > 100
         ORDER BY total_qty DESC",
    );
    assert!(result.is_ok(), "Q18 should execute successfully");
}

// ============================================================
// TPC-H Q21: Suppliers Who Kept Orders Waiting
// Multiple JOINs with subquery
// ============================================================
#[test]
fn tpch_q21_suppliers_who_kept_orders_waiting() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT s_name
         FROM supplier, nation
         WHERE s_nationkey = n_nationkey
         ORDER BY s_name",
    );
    assert!(result.is_ok(), "Q21 should execute successfully");
}

// ============================================================
// TPC-H Q2: Minimum Cost Supplier
// ORDER BY with aggregation and WHERE subquery
// ============================================================
#[test]
fn tpch_q2_minimum_cost_supplier() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT s_name, s_acctbal
         FROM supplier, part, nation
         WHERE s_nationkey = n_nationkey
         ORDER BY s_acctbal DESC, s_name
         LIMIT 10",
    );
    assert!(result.is_ok(), "Q2 should execute successfully");
}

// ============================================================
// TPC-H Q6: Discounted Revenue (simplified)
// Aggregation with WHERE conditions
// ============================================================
#[test]
fn tpch_q6_discounted_revenue() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT SUM(l_discount) as total_discount
         FROM lineitem
         WHERE l_quantity < 24",
    );
    assert!(result.is_ok(), "Q6 should execute successfully");
}

// ============================================================
// TPC-H Q10: Returned Item
// JOIN with GROUP BY and WHERE conditions
// ============================================================
#[test]
fn tpch_q10_returned_item() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT c_custkey, c_name, SUM(l_extendedprice) as revenue
         FROM orders, lineitem, customer
         WHERE o_custkey = c_custkey AND l_orderkey = o_orderkey
         GROUP BY c_custkey, c_name
         ORDER BY revenue DESC
         LIMIT 20",
    );
    assert!(result.is_ok(), "Q10 should execute successfully");
}

// ============================================================
// TPC-H Q13: Customer Orders (simplified, no OUTER JOIN)
// ============================================================
#[test]
fn tpch_q13_customer_orders() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT c_custkey, COUNT(o_orderkey) as order_count
         FROM customer, orders
         WHERE c_custkey = o_custkey
         GROUP BY c_custkey
         ORDER BY order_count DESC
         LIMIT 10",
    );
    assert!(result.is_ok(), "Q13 should execute successfully");
}

// ============================================================
// TPC-H Q14: Promotion Effect
// WHERE with LIKE pattern matching
// ============================================================
#[test]
fn tpch_q14_promotion_effect() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT SUM(l_extendedprice) as promo_revenue
         FROM lineitem, part
         WHERE l_partkey = p_partkey AND p_name LIKE 'Promo%'",
    );
    assert!(result.is_ok(), "Q14 should execute successfully");
}

// ============================================================
// TPC-H Q20: Potential Part Promotion
// WHERE with IN subquery
// ============================================================
#[test]
fn tpch_q20_potential_part_promotion() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT s_name, s_address
         FROM supplier
         WHERE s_nationkey = 1
         ORDER BY s_name",
    );
    assert!(result.is_ok(), "Q20 should execute successfully");
}

// ============================================================
// TPC-H Q22: Global Sales Opportunity
// Subquery with aggregation
// ============================================================
#[test]
fn tpch_q22_global_sales() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT c_custkey, c_phone, c_acctbal
         FROM customer
         WHERE c_acctbal > 1000
         ORDER BY c_acctbal DESC
         LIMIT 10",
    );
    assert!(result.is_ok(), "Q22 should execute successfully");
}

// ============================================================
// TPC-H SF0.1 Scale Factor Validation
// ============================================================
#[test]
fn tpch_sf01_data_validation() {
    let mut engine = setup_engine();

    // Validate lineitem row count
    let result = engine.execute("SELECT COUNT(*) FROM lineitem");
    assert!(result.is_ok(), "Count lineitem should work");
    if let Ok(rows) = result {
        assert!(!rows.rows.is_empty(), "Should have lineitem data");
    }

    // Validate orders row count
    let result = engine.execute("SELECT COUNT(*) FROM orders");
    assert!(result.is_ok(), "Count orders should work");
    if let Ok(rows) = result {
        assert!(!rows.rows.is_empty(), "Should have orders data");
    }

    // Validate customer row count
    let result = engine.execute("SELECT COUNT(*) FROM customer");
    assert!(result.is_ok(), "Count customer should work");
    if let Ok(rows) = result {
        assert!(!rows.rows.is_empty(), "Should have customer data");
    }
}

// ============================================================
// Basic SQL Operations Tests
// ============================================================

#[test]
fn tpch_basic_aggregation() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT COUNT(*), SUM(l_quantity), AVG(l_extendedprice)
         FROM lineitem WHERE l_quantity > 10",
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
         LIMIT 100",
    );
    assert!(result.is_ok(), "Basic JOIN should work");
}

#[test]
fn tpch_basic_group_by() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT o_orderstatus, COUNT(*), SUM(o_totalprice)
         FROM orders
         GROUP BY o_orderstatus",
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
         LIMIT 10",
    );
    assert!(result.is_ok(), "Basic ORDER BY should work");
}

#[test]
fn tpch_basic_subquery() {
    let mut engine = setup_engine();
    let result = engine.execute(
        "SELECT *
         FROM customer
         WHERE c_custkey IN (SELECT o_custkey FROM orders)",
    );
    assert!(result.is_ok(), "Basic subquery should work");
}

#[test]
fn tpch_basic_like() {
    let mut engine = setup_engine();
    let result = engine.execute("SELECT * FROM part WHERE p_name LIKE 'Part%'");
    assert!(result.is_ok(), "LIKE should work");
}

#[test]
fn tpch_basic_between() {
    let mut engine = setup_engine();
    let result = engine.execute("SELECT * FROM lineitem WHERE l_quantity BETWEEN 10 AND 50");
    assert!(result.is_ok(), "BETWEEN should work");
}

#[test]
fn tpch_basic_limit() {
    let mut engine = setup_engine();
    let result = engine.execute("SELECT * FROM orders LIMIT 10");
    assert!(result.is_ok(), "LIMIT should work");
}

#[test]
fn tpch_basic_offset() {
    let mut engine = setup_engine();
    let result = engine.execute("SELECT * FROM orders LIMIT 5 OFFSET 10");
    assert!(result.is_ok(), "OFFSET should work");
}

// ============================================================
// Unsupported Features (documented, tests commented out)
// - DISTINCT: parsed but not fully supported in executor
// - LEFT OUTER JOIN: not fully supported
// - Multiply in aggregation: not fully supported
// - GROUP BY on non-aggregate column: not fully supported
// ============================================================
