-- SQLCorpus: DISTINCT and ALL
-- Tests for duplicate removal

-- === SETUP ===
CREATE TABLE test_data (id INTEGER PRIMARY KEY, a INTEGER, b TEXT, c INTEGER);
INSERT INTO test_data VALUES (1, 1, 'x', 10);
INSERT INTO test_data VALUES (2, 1, 'x', 10);
INSERT INTO test_data VALUES (3, 1, 'y', 10);
INSERT INTO test_data VALUES (4, 2, 'y', 20);
INSERT INTO test_data VALUES (5, 2, 'z', 20);
INSERT INTO test_data VALUES (6, 3, 'z', 30);

-- === CASE: distinct_single ===
SELECT DISTINCT a FROM test_data;
-- EXPECT: 3 rows

-- === CASE: distinct_multiple ===
SELECT DISTINCT a, b FROM test_data;
-- EXPECT: 5 rows

-- === CASE: distinct_all ===
SELECT ALL a FROM test_data;
-- EXPECT: 6 rows

-- === CASE: distinct_with_null ===
SELECT DISTINCT c FROM test_data;
-- EXPECT: 3 rows

-- === CASE: distinct_count ===
SELECT COUNT(DISTINCT a) FROM test_data;
-- EXPECT: 1 rows

-- === CASE: distinct_with_where ===
SELECT DISTINCT a FROM test_data WHERE c > 15;
-- EXPECT: 2 rows

-- === CASE: distinct_with_order ===
SELECT DISTINCT b FROM test_data ORDER BY b;
-- EXPECT: 3 rows

-- === CASE: distinct_with_group ===
SELECT a, COUNT(DISTINCT b) FROM test_data GROUP BY a;
-- EXPECT: 3 rows