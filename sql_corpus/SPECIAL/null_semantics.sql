-- SQLCorpus: NULL Semantics Tests

-- === SETUP ===
CREATE TABLE test_null (id INTEGER PRIMARY KEY, val INTEGER);
INSERT INTO test_null VALUES (1, 10), (2, NULL), (3, 30);

-- === CASE: null_in_where ===
SELECT * FROM test_null WHERE val = 10;
-- EXPECT: 1 rows

-- === CASE: null_comparison ===
SELECT * FROM test_null WHERE val IS NULL;
-- EXPECT: 1 rows

-- === CASE: not_null ===
SELECT * FROM test_null WHERE val IS NOT NULL;
-- EXPECT: 2 rows

-- === CASE: coalesce ===
SELECT COALESCE(val, 0) FROM test_null WHERE id = 2;
-- EXPECT: 1 rows
