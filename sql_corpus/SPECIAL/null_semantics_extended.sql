-- SQLCorpus: NULL Semantics Tests
-- Tests for NULL handling in various operations

-- === SETUP ===
CREATE TABLE test_null (id INTEGER PRIMARY KEY, val TEXT, num INTEGER);
INSERT INTO test_null VALUES (1, 'A', 100), (2, NULL, 200), (3, 'C', NULL), (4, NULL, NULL), (5, 'E', 500);

-- === CASE: is_null ===
SELECT id FROM test_null WHERE val IS NULL;
-- EXPECT: 2 rows

-- === CASE: is_not_null ===
SELECT id FROM test_null WHERE val IS NOT NULL;
-- EXPECT: 3 rows

-- === CASE: coalesce ===
SELECT COALESCE(val, 'DEFAULT') FROM test_null;
-- EXPECT: 5 rows

-- === CASE: coalesce_multiple ===
SELECT COALESCE(val, num, -1) FROM test_null;
-- EXPECT: 5 rows

-- === CASE: nullif ===
SELECT NULLIF(val, 'C') FROM test_null;
-- EXPECT: 5 rows

-- === CASE: ifnull ===
SELECT IFNULL(val, 'MISSING') FROM test_null;
-- EXPECT: 5 rows

-- === CASE: null_in_where ===
SELECT id FROM test_null WHERE num = NULL;
-- EXPECT: 0 rows

-- === CASE: null_comparison ===
SELECT id FROM test_null WHERE val = NULL;
-- EXPECT: 0 rows

-- === CASE: not_null_comparison ===
SELECT id FROM test_null WHERE val != NULL;
-- EXPECT: 0 rows

-- === CASE: null_in_join ===
CREATE TABLE join_null (id INTEGER, name TEXT);
INSERT INTO join_null VALUES (1, 'A'), (2, NULL), (3, 'C');
SELECT t.id, j.name FROM test_null t LEFT JOIN join_null j ON t.val = j.name;
-- EXPECT: 5 rows

-- === CASE: null_aggregate ===
SELECT COUNT(*), COUNT(val), COUNT(num), SUM(num) FROM test_null;
-- EXPECT: 1 rows

-- === CASE: null_in_group_by ===
SELECT val, COUNT(*) FROM test_null GROUP BY val;
-- EXPECT: 3 rows

-- === CASE: null_ordering ===
SELECT val FROM test_null ORDER BY val NULLS FIRST;
-- EXPECT: 5 rows
