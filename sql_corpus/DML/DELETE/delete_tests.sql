-- SQLCorpus: DELETE Tests
-- Tests for various DELETE operations

-- === SETUP ===
CREATE TABLE delete_test (id INTEGER PRIMARY KEY, name TEXT, value INTEGER);
INSERT INTO delete_test VALUES (1, 'A', 100), (2, 'B', 200), (3, 'C', 300), (4, 'D', 400), (5, 'E', 500);

-- === CASE: delete_single_row ===
DELETE FROM delete_test WHERE id = 1;
SELECT COUNT(*) FROM delete_test;
-- EXPECT: 1 rows

-- === CASE: delete_multiple_rows ===
DELETE FROM delete_test WHERE value > 300;
SELECT COUNT(*) FROM delete_test;
-- EXPECT: 2 rows

-- === CASE: delete_with_subquery ===
DELETE FROM delete_test WHERE id IN (SELECT id FROM delete_test WHERE value < 250);
SELECT COUNT(*) FROM delete_test;
-- EXPECT: 1 rows

-- === CASE: delete_all ===
DELETE FROM delete_test;
SELECT COUNT(*) FROM delete_test;
-- EXPECT: 1 rows
