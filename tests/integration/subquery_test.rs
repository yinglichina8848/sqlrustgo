//! TPC-H Subquery Tests
//!
//! Tests for TPC-H queries that use subqueries: EXISTS, IN, ANY/ALL

#[cfg(test)]
mod tests {
    use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
    use std::sync::{Arc, RwLock};

    fn create_engine() -> ExecutionEngine {
        ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
    }

    fn setup_schema(engine: &mut ExecutionEngine) {
        engine.execute(parse("CREATE TABLE orders (o_orderkey INTEGER, o_custkey INTEGER, o_orderstatus TEXT, o_totalprice REAL, o_orderdate TEXT, o_orderpriority TEXT)").unwrap()).unwrap();
        engine.execute(parse("CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER, l_quantity INTEGER, l_extendedprice REAL, l_discount REAL, l_tax REAL, l_returnflag TEXT, l_linestatus TEXT, l_shipdate TEXT, l_commitdate TEXT, l_receiptdate TEXT)").unwrap()).unwrap();
        engine.execute(parse("CREATE TABLE customer (c_custkey INTEGER, c_name TEXT, c_phone TEXT, c_acctbal REAL, c_comment TEXT)").unwrap()).unwrap();
        engine.execute(parse("CREATE TABLE part (p_partkey INTEGER, p_name TEXT, p_mfgr TEXT, p_brand TEXT, p_type TEXT, p_size INTEGER, p_container TEXT, p_retailprice REAL)").unwrap()).unwrap();
        engine.execute(parse("CREATE TABLE supplier (s_suppkey INTEGER, s_name TEXT, s_nationkey INTEGER, s_phone TEXT, s_acctbal REAL)").unwrap()).unwrap();
        engine.execute(parse("CREATE TABLE partsupp (ps_partkey INTEGER, ps_suppkey INTEGER, ps_availqty INTEGER, ps_supplycost REAL)").unwrap()).unwrap();
    }

    fn insert_test_data(engine: &mut ExecutionEngine) {
        engine
            .execute(
                parse("INSERT INTO orders VALUES (1, 1, 'O', 15000.00, '1994-01-15', '1-URGENT')")
                    .unwrap(),
            )
            .unwrap();
        engine
            .execute(
                parse("INSERT INTO orders VALUES (2, 2, 'O', 5000.00, '1994-01-20', '5-LOW')")
                    .unwrap(),
            )
            .unwrap();
        engine
            .execute(
                parse("INSERT INTO orders VALUES (3, 3, 'F', 8000.00, '1994-02-01', '3-MEDIUM')")
                    .unwrap(),
            )
            .unwrap();

        engine.execute(parse("INSERT INTO lineitem VALUES (1, 1, 1, 15, 15000.00, 0.05, 1.2, 'N', 'O', '1994-01-20', '1994-01-18', '1994-01-25')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO lineitem VALUES (1, 2, 2, 20, 20000.00, 0.05, 1.6, 'N', 'O', '1994-01-20', '1994-01-18', '1994-01-25')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO lineitem VALUES (2, 3, 3, 5, 5000.00, 0.10, 0.4, 'N', 'O', '1994-01-25', '1994-01-23', '1994-01-30')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO lineitem VALUES (3, 1, 1, 8, 8000.00, 0.08, 0.64, 'N', 'O', '1994-02-10', '1994-02-08', '1994-02-15')").unwrap()).unwrap();

        engine.execute(parse("INSERT INTO customer VALUES (1, 'Customer#1', '10-1111111', 1000.00, 'comment1')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO customer VALUES (2, 'Customer#2', '10-2222222', 2000.00, 'specialrequests')").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO customer VALUES (3, 'Customer#3', '10-3333333', 3000.00, 'comment3')").unwrap()).unwrap();

        engine.execute(parse("INSERT INTO part VALUES (1, 'Part1', 'MFGR#1', 'Brand#1', 'ECONOMY', 10, 'MED PKG', 1000.00)").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO part VALUES (2, 'Part2', 'MFGR#1', 'Brand#2', 'PROMO', 20, 'LG CASE', 2000.00)").unwrap()).unwrap();
        engine.execute(parse("INSERT INTO part VALUES (3, 'forestPart', 'MFGR#2', 'Brand#3', 'STANDARD', 15, 'MED CASE', 1500.00)").unwrap()).unwrap();

        engine
            .execute(
                parse("INSERT INTO supplier VALUES (1, 'Supplier#1', 1, '10-1111111', 1000.00)")
                    .unwrap(),
            )
            .unwrap();
        engine
            .execute(
                parse("INSERT INTO supplier VALUES (2, 'Supplier#2', 1, '10-2222222', 2000.00)")
                    .unwrap(),
            )
            .unwrap();
        engine
            .execute(
                parse("INSERT INTO supplier VALUES (3, 'Supplier#3', 1, '10-3333333', 3000.00)")
                    .unwrap(),
            )
            .unwrap();

        engine
            .execute(parse("INSERT INTO partsupp VALUES (1, 1, 100, 500.00)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO partsupp VALUES (2, 2, 200, 600.00)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO partsupp VALUES (3, 3, 150, 700.00)").unwrap())
            .unwrap();
    }

    #[test]
    fn test_tpch_q10_exists_subquery() {
        let mut engine = create_engine();
        setup_schema(&mut engine);
        insert_test_data(&mut engine);

        let sql = "SELECT o_orderpriority, COUNT(*) AS order_count FROM orders WHERE o_orderdate >= '1994-01-01' AND o_orderdate < '1994-04-01' AND EXISTS (SELECT * FROM lineitem WHERE l_orderkey = o_orderkey AND l_commitdate < l_receiptdate) GROUP BY o_orderpriority ORDER BY o_orderpriority";
        let result = engine.execute(parse(sql).unwrap());
        assert!(
            result.is_ok(),
            "Q10 with EXISTS should succeed: {:?}",
            result
        );
    }

    #[test]
    fn test_tpch_q12_in_subquery() {
        let mut engine = create_engine();
        setup_schema(&mut engine);
        insert_test_data(&mut engine);

        let sql = "SELECT l_shipmode, COUNT(*) FROM orders, lineitem WHERE l_orderkey = o_orderkey AND l_shipmode IN ('MAIL', 'SHIP') AND l_commitdate < l_receiptdate AND l_shipdate < l_commitdate GROUP BY l_shipmode ORDER BY l_shipmode";
        let result = engine.execute(parse(sql).unwrap());
        assert!(
            result.is_ok(),
            "Q12 with IN subquery should succeed: {:?}",
            result
        );
    }

    #[test]
    fn test_tpch_q21_not_in_subquery() {
        let mut engine = create_engine();
        setup_schema(&mut engine);
        insert_test_data(&mut engine);

        let sql = "SELECT s_name, COUNT(*) FROM supplier, lineitem l1, orders WHERE s_suppkey = l1.l_suppkey AND o_orderkey = l1.l_orderkey AND o_orderstatus = 'F' AND NOT EXISTS (SELECT * FROM lineitem l2 WHERE l2.l_orderkey = l1.l_orderkey AND l2.l_suppkey <> l1.l_suppkey AND l2.l_receiptdate > l2.l_commitdate) GROUP BY s_name ORDER BY s_name LIMIT 100";
        let result = engine.execute(parse(sql).unwrap());
        assert!(
            result.is_ok(),
            "Q21 with NOT EXISTS should succeed: {:?}",
            result
        );
    }

    #[test]
    fn test_simple_scalar_subquery() {
        let mut engine = create_engine();
        setup_schema(&mut engine);
        insert_test_data(&mut engine);

        let sql = "SELECT * FROM customer WHERE c_acctbal > (SELECT AVG(c_acctbal) FROM customer WHERE c_acctbal > 0)";
        let result = engine.execute(parse(sql).unwrap());
        assert!(
            result.is_ok(),
            "Scalar subquery should succeed: {:?}",
            result
        );
    }

    #[test]
    fn test_correlated_exists() {
        let mut engine = create_engine();
        setup_schema(&mut engine);
        insert_test_data(&mut engine);

        let sql = "SELECT * FROM supplier s WHERE EXISTS (SELECT * FROM partsupp WHERE ps_suppkey = s.s_suppkey AND ps_partkey IN (SELECT p_partkey FROM part WHERE p_name LIKE 'forest%'))";
        let result = engine.execute(parse(sql).unwrap());
        assert!(
            result.is_ok(),
            "Correlated EXISTS with subquery should succeed: {:?}",
            result
        );
    }

    #[test]
    fn test_in_with_scalar_subquery() {
        let mut engine = create_engine();
        setup_schema(&mut engine);
        insert_test_data(&mut engine);

        let sql = "SELECT * FROM partsupp WHERE ps_suppkey NOT IN (SELECT s_suppkey FROM supplier WHERE s_acctbal < 1000.00)";
        let result = engine.execute(parse(sql).unwrap());
        assert!(
            result.is_ok(),
            "IN with scalar subquery should succeed: {:?}",
            result
        );
    }

    #[test]
    fn test_correlated_in_subquery() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE t1 (id INTEGER, t2_id INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("CREATE TABLE t2 (id INTEGER, name TEXT)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO t1 VALUES (1, 10)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO t1 VALUES (2, 20)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO t1 VALUES (3, 30)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO t2 VALUES (10, 'ten')").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO t2 VALUES (20, 'twenty')").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO t2 VALUES (40, 'forty')").unwrap())
            .unwrap();

        let sql = "SELECT * FROM t1 WHERE t2_id IN (SELECT id FROM t2 WHERE id < 25)";
        let result = engine.execute(parse(sql).unwrap());
        assert!(
            result.is_ok(),
            "Correlated IN subquery should succeed: {:?}",
            result
        );
    }

    #[test]
    fn test_correlated_any_all() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE outer_tbl (id INTEGER, val INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("CREATE TABLE inner_tbl (id INTEGER, val INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO outer_tbl VALUES (1, 10)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO outer_tbl VALUES (2, 25)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO inner_tbl VALUES (1, 15)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO inner_tbl VALUES (2, 20)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO inner_tbl VALUES (3, 30)").unwrap())
            .unwrap();

        let sql = "SELECT * FROM outer_tbl o WHERE o.val > ANY (SELECT i.val FROM inner_tbl i WHERE i.id = o.id)";
        let result = engine.execute(parse(sql).unwrap());
        assert!(
            result.is_ok(),
            "Correlated ANY subquery should succeed: {:?}",
            result
        );
    }

    #[test]
    fn test_non_correlated_exists() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE products (id INTEGER, name TEXT)").unwrap())
            .unwrap();
        engine
            .execute(parse("CREATE TABLE categories (id INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO products VALUES (1, 'Product1')").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO products VALUES (2, 'Product2')").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO categories VALUES (1)").unwrap())
            .unwrap();

        let sql = "SELECT * FROM products WHERE EXISTS (SELECT * FROM categories)";
        let result = engine.execute(parse(sql).unwrap());
        assert!(
            result.is_ok(),
            "Non-correlated EXISTS should succeed: {:?}",
            result
        );
    }
}
