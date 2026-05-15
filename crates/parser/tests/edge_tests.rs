//! Parser boundary/edge case tests (Layer 5).
//! Tests for edge conditions: empty, very long, many items, etc.

use sqlrustgo_parser::parse;
use sqlrustgo_parser::parser::ExplainStatement;
use sqlrustgo_parser::Statement;

#[test]
fn test_single_char_identifier() {
    let result = parse("SELECT a FROM t");
    assert!(
        result.is_ok(),
        "Single char identifier should work: {:?}",
        result
    );
}

#[test]
fn test_long_identifier() {
    let sql = format!("SELECT {} FROM t", "a".repeat(100));
    let result = parse(&sql);
    assert!(result.is_ok(), "Long identifier should work: {:?}", result);
}

#[test]
fn test_very_long_identifier() {
    let sql = format!("SELECT {} FROM t", "a".repeat(1000));
    let result = parse(&sql);
    assert!(
        result.is_ok(),
        "Very long identifier should work: {:?}",
        result
    );
}

#[test]
fn test_many_columns() {
    let cols: Vec<String> = (0..50).map(|i| format!("col{}", i)).collect();
    let sql = format!("SELECT {} FROM t", cols.join(", "));
    let result = parse(&sql);
    assert!(result.is_ok(), "Many columns should work: {:?}", result);
}

#[test]
fn test_deeply_nested_parens() {
    let sql = "SELECT (((((1)))))";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Deeply nested parens should be supported: {:?}",
        result
    );
}

#[test]
fn test_long_string_literal() {
    let sql = format!("SELECT '{}'", "x".repeat(1000));
    let result = parse(&sql);
    assert!(
        result.is_ok(),
        "Long string literal should work: {:?}",
        result
    );
}

#[test]
fn test_many_digits_in_number() {
    let sql = "SELECT 123456789012345678901234567890";
    let result = parse(sql);
    assert!(result.is_ok(), "Long number should work: {:?}", result);
}

#[test]
fn test_negative_number() {
    let result = parse("SELECT -1 FROM t");
    assert!(
        result.is_ok(),
        "Negative numbers should be supported: {:?}",
        result
    );
}

#[test]
fn test_decimal_number() {
    let result = parse("SELECT 3.14159");
    assert!(result.is_ok(), "Decimal number should work: {:?}", result);
}

#[test]
fn test_negative_decimal() {
    let result = parse("SELECT -3.14159 FROM t");
    assert!(
        result.is_ok(),
        "Negative decimals should be supported: {:?}",
        result
    );
}

#[test]
fn test_hex_number() {
    let result = parse("SELECT 0xFF");
    assert!(result.is_ok(), "Hex number should work: {:?}", result);
}

#[test]
fn test_leading_zeros() {
    let result = parse("SELECT 007");
    assert!(result.is_ok(), "Leading zeros should work: {:?}", result);
}

#[test]
fn test_scientific_notation() {
    let result = parse("SELECT 1e10");
    assert!(
        result.is_ok(),
        "Scientific notation should work: {:?}",
        result
    );
}

#[test]
fn test_double_star_exponent() {
    let result = parse("SELECT 1e-5");
    assert!(
        result.is_ok(),
        "Negative exponent should work: {:?}",
        result
    );
}

#[test]
fn test_quoted_identifier() {
    let result = parse("SELECT `column name` FROM t");
    assert!(
        result.is_err(),
        "Quoted identifiers not supported: {:?}",
        result
    );
}

#[test]
fn test_quoted_identifier_with_special_chars() {
    let result = parse("SELECT `column-name.with.dots` FROM t");
    assert!(
        result.is_err(),
        "Quoted identifiers not supported: {:?}",
        result
    );
}

#[test]
fn test_alias_with_as() {
    let result = parse("SELECT col AS alias FROM t");
    assert!(result.is_ok(), "Alias with AS should work: {:?}", result);
}

#[test]
fn test_alias_without_as() {
    let result = parse("SELECT col alias FROM t");
    assert!(result.is_ok(), "Alias without AS should work: {:?}", result);
}

#[test]
fn test_expression_in_select() {
    let result = parse("SELECT a + b * c FROM t");
    assert!(
        result.is_ok(),
        "Expression in select should work: {:?}",
        result
    );
}

#[test]
fn test_function_in_select() {
    let result = parse("SELECT COUNT(*) FROM t");
    assert!(
        result.is_ok(),
        "Function in select should work: {:?}",
        result
    );
}

#[test]
fn test_tpch_q1_parse() {
    let sql = "SELECT l_returnflag, l_linestatus, SUM(l_quantity) AS sum_qty, SUM(l_extendedprice) AS sum_base_price, SUM(l_extendedprice * (1 - l_discount)) AS sum_disc_price, SUM(l_extendedprice * (1 - l_discount) * (1 + l_tax)) AS sum_charge, AVG(l_quantity) AS avg_qty, AVG(l_extendedprice) AS avg_price, AVG(l_discount) AS avg_disc, COUNT(*) AS count_order FROM lineitem WHERE l_shipdate <= '1998-09-02' GROUP BY l_returnflag, l_linestatus ORDER BY l_returnflag, l_linestatus";
    let result = parse(sql);
    assert!(result.is_ok(), "Q1 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q3_parse() {
    let sql = "SELECT l_orderkey, SUM(l_extendedprice * (1 - l_discount)) AS revenue, o_orderdate, o_shippriority FROM customer, orders, lineitem WHERE c_mktsegment = 'BUILDING' AND c_custkey = o_custkey AND l_orderkey = o_orderkey AND o_orderdate < '1995-03-15' AND l_shipdate > '1995-03-15' GROUP BY l_orderkey, o_orderdate, o_shippriority ORDER BY revenue DESC, o_orderdate";
    let result = parse(sql);
    assert!(result.is_ok(), "Q3 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q4_parse() {
    let sql = "SELECT o_orderpriority, COUNT(*) AS order_count FROM orders WHERE o_orderdate >= '1993-07-01' AND o_orderdate < '1993-10-01' AND EXISTS (SELECT * FROM lineitem WHERE l_orderkey = o_orderkey AND l_commitdate < l_receiptdate) GROUP BY o_orderpriority ORDER BY o_orderpriority";
    let result = parse(sql);
    assert!(result.is_ok(), "Q4 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q5_parse() {
    let sql = "SELECT n_name, SUM(l_extendedprice * (1 - l_discount)) AS revenue FROM customer, orders, lineitem, supplier, nation, region WHERE c_custkey = o_custkey AND l_orderkey = o_orderkey AND l_suppkey = s_suppkey AND c_nationkey = s_nationkey AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'ASIA' AND o_orderdate >= '1994-01-01' AND o_orderdate < '1995-01-01' GROUP BY n_name ORDER BY revenue DESC";
    let result = parse(sql);
    assert!(result.is_ok(), "Q5 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q6_parse() {
    let sql = "SELECT SUM(l_extendedprice * l_discount) AS revenue FROM lineitem WHERE l_shipdate >= '1994-01-01' AND l_shipdate < '1995-01-01' AND l_discount >= 0.05 AND l_discount <= 0.07 AND l_quantity < 25";
    let result = parse(sql);
    assert!(result.is_ok(), "Q6 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q7_parse() {
    let sql = "SELECT supp_nation, cust_nation, l_year, SUM(volume) AS revenue FROM (SELECT n1.n_name AS supp_nation, n2.n_name AS cust_nation, CAST(SUBSTR(l_shipdate, 1, 4) AS INTEGER) AS l_year, l_extendedprice * (1 - l_discount) AS volume FROM supplier, lineitem, orders, customer, nation n1, nation n2 WHERE s_suppkey = l_suppkey AND o_orderkey = l_orderkey AND c_custkey = o_custkey AND s_nationkey = n1.n_nationkey AND c_nationkey = n2.n_nationkey AND ((n1.n_name = 'FRANCE' AND n2.n_name = 'GERMANY') OR (n1.n_name = 'GERMANY' AND n2.n_name = 'FRANCE')) AND l_shipdate BETWEEN '1995-01-01' AND '1996-12-31') AS shipping GROUP BY supp_nation, cust_nation, l_year ORDER BY supp_nation, cust_nation, l_year";
    let result = parse(sql);
    assert!(result.is_ok(), "Q7 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q8_parse() {
    let sql = "SELECT o_year, SUM(CASE WHEN nation = 'BRAZIL' THEN volume ELSE 0 END) / SUM(volume) AS mkt_share FROM (SELECT CAST(SUBSTR(o_orderdate, 1, 4) AS INTEGER) AS o_year, l_extendedprice * (1 - l_discount) AS volume, n2.n_name AS nation FROM part, supplier, lineitem, orders, customer, nation n1, nation n2, region WHERE p_partkey = l_partkey AND s_suppkey = l_suppkey AND l_orderkey = o_orderkey AND o_custkey = c_custkey AND c_nationkey = n1.n_nationkey AND n1.n_regionkey = r_regionkey AND r_name = 'AMERICA' AND s_nationkey = n2.n_nationkey AND o_orderdate BETWEEN '1995-01-01' AND '1996-12-31' AND p_type = 'ECONOMY ANODIZED STEEL') AS all_nations GROUP BY o_year ORDER BY o_year";
    let result = parse(sql);
    assert!(result.is_ok(), "Q8 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q9_parse() {
    let sql = "SELECT nation, o_year, SUM(amount) AS sum_profit FROM (SELECT n_name AS nation, CAST(SUBSTR(o_orderdate, 1, 4) AS INTEGER) AS o_year, l_extendedprice * (1 - l_discount) - ps_supplycost * l_quantity AS amount FROM part, supplier, lineitem, partsupp, orders, nation WHERE s_suppkey = l_suppkey AND ps_suppkey = l_suppkey AND ps_partkey = l_partkey AND p_partkey = l_partkey AND o_orderkey = l_orderkey AND s_nationkey = n_nationkey AND p_name LIKE '%green%') AS profit GROUP BY nation, o_year ORDER BY nation, o_year DESC";
    let result = parse(sql);
    assert!(result.is_ok(), "Q9 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q10_parse() {
    let sql = "SELECT c_custkey, c_name, SUM(l_extendedprice * (1 - l_discount)) AS revenue, c_acctbal, n_name, c_address, c_phone, c_comment FROM customer, orders, lineitem, nation WHERE c_custkey = o_custkey AND l_orderkey = o_orderkey AND o_orderdate >= '1993-10-01' AND o_orderdate < '1994-01-01' AND l_returnflag = 'R' AND c_nationkey = n_nationkey GROUP BY c_custkey, c_name, c_acctbal, n_name, c_address, c_phone, c_comment ORDER BY revenue DESC";
    let result = parse(sql);
    assert!(result.is_ok(), "Q10 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q12_parse() {
    let sql = "SELECT l_shipmode, SUM(CASE WHEN o_orderpriority = '1-URGENT' OR o_orderpriority = '2-HIGH' THEN 1 ELSE 0 END) AS high_line_count, SUM(CASE WHEN o_orderpriority <> '1-URGENT' AND o_orderpriority <> '2-HIGH' THEN 1 ELSE 0 END) AS low_line_count FROM orders, lineitem WHERE o_orderkey = l_orderkey AND l_shipmode IN ('MAIL', 'SHIP') AND l_commitdate < l_receiptdate AND l_shipdate < l_commitdate AND l_receiptdate >= '1994-01-01' AND l_receiptdate < '1995-01-01' GROUP BY l_shipmode ORDER BY l_shipmode";
    let result = parse(sql);
    assert!(result.is_ok(), "Q12 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q13_parse() {
    let sql = "SELECT c_count, COUNT(*) AS custdist FROM (SELECT c_custkey, COUNT(o_orderkey) AS c_count FROM customer LEFT JOIN orders ON c_custkey = o_custkey AND o_comment NOT LIKE '%special%requests%' GROUP BY c_custkey) AS c_orders GROUP BY c_count ORDER BY custdist DESC, c_count DESC";
    let result = parse(sql);
    assert!(result.is_ok(), "Q13 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q14_parse() {
    let sql = "SELECT 100.00 * SUM(CASE WHEN p_type LIKE 'PROMO%' THEN l_extendedprice * (1 - l_discount) ELSE 0 END) / SUM(l_extendedprice * (1 - l_discount)) AS promo_revenue FROM lineitem, part WHERE l_partkey = p_partkey AND l_shipdate >= '1995-09-01' AND l_shipdate < '1995-10-01'";
    let result = parse(sql);
    assert!(result.is_ok(), "Q14 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q15_parse() {
    let sql = "SELECT s_suppkey, s_name, s_address, s_phone, total_revenue FROM supplier, (SELECT l_suppkey AS supplier_no, SUM(l_extendedprice * (1 - l_discount)) AS total_revenue FROM lineitem WHERE l_shipdate >= '1996-01-01' AND l_shipdate < '1996-04-01' GROUP BY l_suppkey) AS revenue0 WHERE s_suppkey = supplier_no AND total_revenue = (SELECT MAX(total_revenue) FROM (SELECT l_suppkey, SUM(l_extendedprice * (1 - l_discount)) AS total_revenue FROM lineitem WHERE l_shipdate >= '1996-01-01' AND l_shipdate < '1996-04-01' GROUP BY l_suppkey) AS t) ORDER BY s_suppkey";
    let result = parse(sql);
    assert!(result.is_ok(), "Q15 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q17_parse() {
    let sql = "SELECT SUM(l_extendedprice) / 7.0 AS avg_yearly FROM lineitem, part WHERE p_partkey = l_partkey AND p_brand = 'Brand#23' AND p_container = 'MED BOX' AND l_quantity < (SELECT 0.2 * AVG(l_quantity) FROM lineitem WHERE l_partkey = p_partkey)";
    let result = parse(sql);
    assert!(result.is_ok(), "Q17 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q18_parse() {
    let sql = "SELECT c_name, c_custkey, o_orderkey, o_orderdate, o_totalprice, SUM(l_quantity) FROM customer, orders, lineitem WHERE o_orderkey IN (SELECT l_orderkey FROM lineitem GROUP BY l_orderkey HAVING SUM(l_quantity) > 300) AND c_custkey = o_custkey AND o_orderkey = l_orderkey GROUP BY c_name, c_custkey, o_orderkey, o_orderdate, o_totalprice ORDER BY o_totalprice DESC, o_orderdate";
    let result = parse(sql);
    assert!(result.is_ok(), "Q18 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q19_parse() {
    let sql = "SELECT SUM(l_extendedprice * (1 - l_discount)) AS revenue FROM lineitem, part WHERE (p_partkey = l_partkey AND p_brand = 'Brand#12' AND p_container IN ('SM CASE', 'SM BOX', 'SM PACK', 'SM PKG') AND l_quantity >= 1 AND l_quantity <= 11 AND p_size BETWEEN 1 AND 5 AND l_shipmode IN ('AIR', 'AIR REG') AND l_shipinstruct = 'DELIVER IN PERSON') OR (p_partkey = l_partkey AND p_brand = 'Brand#23' AND p_container IN ('MED BAG', 'MED BOX', 'MED PKG', 'MED PACK') AND l_quantity >= 10 AND l_quantity <= 20 AND p_size BETWEEN 1 AND 10 AND l_shipmode IN ('AIR', 'AIR REG') AND l_shipinstruct = 'DELIVER IN PERSON') OR (p_partkey = l_partkey AND p_brand = 'Brand#34' AND p_container IN ('LG CASE', 'LG BOX', 'LG PACK', 'LG PKG') AND l_quantity >= 20 AND l_quantity <= 30 AND p_size BETWEEN 1 AND 15 AND l_shipmode IN ('AIR', 'AIR REG') AND l_shipinstruct = 'DELIVER IN PERSON')";
    let result = parse(sql);
    assert!(result.is_ok(), "Q19 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q20_parse() {
    let sql = "SELECT s_name, s_address FROM supplier, nation WHERE s_suppkey IN (SELECT ps_suppkey FROM partsupp WHERE ps_partkey IN (SELECT p_partkey FROM part WHERE p_name LIKE 'forest%') AND ps_availqty > (SELECT 0.5 * SUM(l_quantity) FROM lineitem WHERE l_partkey = ps_partkey AND l_suppkey = ps_suppkey AND l_shipdate >= '1994-01-01' AND l_shipdate < '1995-01-01')) AND s_nationkey = n_nationkey AND n_name = 'CANADA' ORDER BY s_name";
    let result = parse(sql);
    assert!(result.is_ok(), "Q20 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q21_parse() {
    let sql = "SELECT s_name, COUNT(*) AS numwait FROM supplier, lineitem l1, orders, nation WHERE s_suppkey = l1.l_suppkey AND o_orderkey = l1.l_orderkey AND o_orderstatus = 'F' AND l1.l_receiptdate > l1.l_commitdate AND EXISTS (SELECT * FROM lineitem l2 WHERE l2.l_orderkey = l1.l_orderkey AND l2.l_suppkey <> l1.l_suppkey) AND NOT EXISTS (SELECT * FROM lineitem l3 WHERE l3.l_orderkey = l1.l_orderkey AND l3.l_suppkey <> l1.l_suppkey AND l3.l_receiptdate > l3.l_commitdate) AND s_nationkey = n_nationkey AND n_name = 'SAUDI ARABIA' GROUP BY s_name ORDER BY numwait DESC, s_name";
    let result = parse(sql);
    assert!(result.is_ok(), "Q21 parse failed: {:?}", result);
}

#[test]
fn test_tpch_q22_parse() {
    let sql = "SELECT cntrycode, COUNT(*) AS numcust, SUM(c_acctbal) AS totacctbal FROM (SELECT SUBSTR(c_phone, 1, 2) AS cntrycode, c_acctbal FROM customer WHERE SUBSTR(c_phone, 1, 2) IN ('13', '31', '23', '29', '30', '18', '17') AND c_acctbal > (SELECT AVG(c_acctbal) FROM customer WHERE c_acctbal > 0.00 AND SUBSTR(c_phone, 1, 2) IN ('13', '31', '23', '29', '30', '18', '17')) AND NOT EXISTS (SELECT * FROM orders WHERE o_custkey = c_custkey)) AS custsale GROUP BY cntrycode ORDER BY cntrycode";
    let result = parse(sql);
    assert!(result.is_ok(), "Q22 parse failed: {:?}", result);
}

#[test]
fn test_distinct_in_select() {
    let result = parse("SELECT DISTINCT a FROM t");
    assert!(
        result.is_ok(),
        "DISTINCT in select should work: {:?}",
        result
    );
}

#[test]
fn test_all_in_select() {
    let result = parse("SELECT ALL a FROM t");
    assert!(result.is_err(), "ALL keyword not supported: {:?}", result);
}

#[test]
fn test_multiple_statements() {
    let result = parse("SELECT 1; SELECT 2");
    assert!(
        result.is_err(),
        "Multiple statements not supported: {:?}",
        result
    );
}

#[test]
fn test_comment_before_select() {
    let result = parse("-- comment\nSELECT 1");
    assert!(result.is_err(), "Comments not supported: {:?}", result);
}

#[test]
fn test_comment_after_select() {
    let result = parse("SELECT 1 -- comment");
    assert!(
        result.is_err(),
        "Trailing comments not supported: {:?}",
        result
    );
}

#[test]
fn test_block_comment() {
    let result = parse("/* comment */ SELECT 1");
    assert!(
        result.is_err(),
        "Block comments not supported: {:?}",
        result
    );
}

#[test]
fn test_cte_basic() {
    let result = parse("WITH cte AS (SELECT 1) SELECT * FROM cte");
    assert!(result.is_ok(), "CTE should parse: {:?}", result);
}

#[test]
fn test_multiple_ctes() {
    let result = parse("WITH cte1 AS (SELECT 1), cte2 AS (SELECT 2) SELECT * FROM cte1, cte2");
    assert!(result.is_ok(), "Multiple CTEs should parse: {:?}", result);
}

#[test]
fn test_scalar_subquery() {
    let result = parse("SELECT (SELECT MAX(x) FROM t) FROM t2");
    assert!(
        result.is_err(),
        "Scalar subquery not supported: {:?}",
        result
    );
}

#[test]
fn test_exists_subquery() {
    let result = parse("SELECT * FROM t WHERE EXISTS (SELECT 1)");
    assert!(result.is_ok(), "EXISTS subquery should parse: {:?}", result);
}

#[test]
fn test_in_subquery() {
    let result = parse("SELECT * FROM t WHERE a IN (SELECT x FROM t2)");
    assert!(result.is_ok(), "IN subquery should work: {:?}", result);
}

#[test]
fn test_join_with_using() {
    let result = parse("SELECT * FROM t1 JOIN t2 USING (id)");
    assert!(result.is_ok(), "JOIN USING should work: {:?}", result);
}

#[test]
fn test_natural_join() {
    let result = parse("SELECT * FROM t1 NATURAL JOIN t2");
    assert!(result.is_ok(), "NATURAL JOIN should work: {:?}", result);
}

#[test]
fn test_cross_join() {
    let result = parse("SELECT * FROM t1 CROSS JOIN t2");
    assert!(result.is_ok(), "CROSS JOIN should work: {:?}", result);
}

#[test]
fn test_insert_single_value() {
    let result = parse("INSERT INTO t VALUES (1)");
    assert!(
        result.is_ok(),
        "Single value insert should work: {:?}",
        result
    );
}

#[test]
fn test_insert_multiple_values() {
    let result = parse("INSERT INTO t VALUES (1), (2), (3)");
    assert!(
        result.is_ok(),
        "Multiple values insert should work: {:?}",
        result
    );
}

#[test]
fn test_insert_with_columns() {
    let result = parse("INSERT INTO t (a, b) VALUES (1, 2)");
    assert!(
        result.is_ok(),
        "Insert with columns should work: {:?}",
        result
    );
}

#[test]
fn test_update_with_where() {
    let result = parse("UPDATE t SET a = 1 WHERE b = 2");
    assert!(
        result.is_ok(),
        "UPDATE with WHERE should work: {:?}",
        result
    );
}

#[test]
fn test_delete_with_where() {
    let result = parse("DELETE FROM t WHERE a = 1");
    assert!(
        result.is_ok(),
        "DELETE with WHERE should work: {:?}",
        result
    );
}

#[test]
fn test_create_table_with_primary_key() {
    let result = parse("CREATE TABLE t (a INT PRIMARY KEY, b TEXT)");
    assert!(
        result.is_ok(),
        "CREATE TABLE with PRIMARY KEY should work: {:?}",
        result
    );
}

#[test]
fn test_create_table_with_not_null() {
    let result = parse("CREATE TABLE t (a INT NOT NULL)");
    assert!(
        result.is_ok(),
        "CREATE TABLE with NOT NULL should work: {:?}",
        result
    );
}

#[test]
fn test_create_table_with_default() {
    let result = parse("CREATE TABLE t (a INT DEFAULT 0)");
    assert!(
        result.is_ok(),
        "CREATE TABLE with DEFAULT should work: {:?}",
        result
    );
}

#[test]
fn test_create_table_with_unique() {
    let result = parse("CREATE TABLE t (a INT UNIQUE)");
    assert!(
        result.is_ok(),
        "CREATE TABLE with UNIQUE should work: {:?}",
        result
    );
}

#[test]
fn test_create_table_with_check() {
    let result = parse("CREATE TABLE t (a INT CHECK (a > 0))");
    assert!(
        result.is_ok(),
        "CREATE TABLE with CHECK should work: {:?}",
        result
    );
}

#[test]
fn test_create_table_with_foreign_key() {
    let result = parse("CREATE TABLE t (a INT REFERENCES other(id))");
    assert!(
        result.is_ok(),
        "CREATE TABLE with FOREIGN KEY should work: {:?}",
        result
    );
}

#[test]
fn test_drop_table_if_exists() {
    let result = parse("DROP TABLE IF EXISTS t");
    assert!(
        result.is_ok(),
        "DROP TABLE IF EXISTS should work: {:?}",
        result
    );
}

#[test]
fn test_truncate_table() {
    let result = parse("TRUNCATE TABLE t");
    assert!(result.is_err(), "TRUNCATE not supported: {:?}", result);
}

#[test]
fn test_alter_table_add_column() {
    let result = parse("ALTER TABLE t ADD COLUMN a INT");
    assert!(
        result.is_ok(),
        "ALTER TABLE ADD COLUMN should work: {:?}",
        result
    );
}

#[test]
fn test_alter_table_drop_column() {
    let result = parse("ALTER TABLE t DROP COLUMN a");
    assert!(
        result.is_ok(),
        "ALTER TABLE DROP COLUMN should work: {:?}",
        result
    );
}

#[test]
fn test_create_index() {
    let result = parse("CREATE INDEX idx ON t (a)");
    assert!(result.is_ok(), "CREATE INDEX should work: {:?}", result);
}

#[test]
fn test_create_unique_index() {
    let result = parse("CREATE UNIQUE INDEX idx ON t (a)");
    assert!(
        result.is_ok(),
        "CREATE UNIQUE INDEX should work: {:?}",
        result
    );
}

#[test]
fn test_drop_index() {
    let result = parse("DROP INDEX idx");
    assert!(result.is_ok(), "DROP INDEX should work: {:?}", result);
}

#[test]
fn test_begin_transaction() {
    let result = parse("BEGIN");
    assert!(result.is_ok(), "BEGIN should work: {:?}", result);
}

#[test]
fn test_commit() {
    let result = parse("COMMIT");
    assert!(result.is_ok(), "COMMIT should work: {:?}", result);
}

#[test]
fn test_rollback() {
    let result = parse("ROLLBACK");
    assert!(result.is_ok(), "ROLLBACK should work: {:?}", result);
}

#[test]
fn test_set_variable() {
    let result = parse("SET a = 1");
    assert!(result.is_err(), "SET not supported: {:?}", result);
}

#[test]
fn test_show_tables() {
    let result = parse("SHOW TABLES");
    assert!(result.is_ok(), "SHOW TABLES should work: {:?}", result);
}

#[test]
fn test_describe_table() {
    let result = parse("DESCRIBE t");
    assert!(result.is_ok(), "DESCRIBE should work: {:?}", result);
}

#[test]
fn test_explain_select() {
    let result = parse("EXPLAIN SELECT * FROM t");
    assert!(result.is_ok(), "EXPLAIN should parse: {:?}", result);
    match result.unwrap() {
        Statement::Explain(ExplainStatement {
            analyze,
            statement,
            format: _,
        }) => {
            assert!(!analyze);
            match *statement {
                Statement::Select(s) => assert_eq!(s.table, "t"),
                _ => panic!("EXPLAIN should wrap a SELECT"),
            }
        }
        _ => panic!("Expected Explain statement"),
    }
}

#[test]
fn test_explain_analyze() {
    let result = parse("EXPLAIN ANALYZE SELECT * FROM t");
    assert!(result.is_ok(), "EXPLAIN ANALYZE should parse: {:?}", result);
    match result.unwrap() {
        Statement::Explain(ExplainStatement { analyze, .. }) => {
            assert!(analyze, "EXPLAIN ANALYZE should have analyze=true");
        }
        _ => panic!("Expected Explain statement"),
    }
}
