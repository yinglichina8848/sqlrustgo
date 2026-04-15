-- SQLCorpus: UPDATE Tests
-- Tests for various UPDATE operations

-- === SETUP ===
CREATE TABLE update_test (id INTEGER PRIMARY KEY, name TEXT, value INTEGER);
INSERT INTO update_test VALUES (1, 'A', 100), (2, 'B', 200), (3, 'C', 300), (4, 'D', 400);

-- === CASE: update_single_row ===
UPDATE update_test SET value = 999 WHERE id = 1;
SELECT value FROM update_test WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: update_multiple_rows ===
UPDATE update_test SET value = 0 WHERE value > 200;
SELECT COUNT(*) FROM update_test WHERE value = 0;
-- EXPECT: 2 rows

-- === CASE: update_with_expression ===
UPDATE update_test SET value = value * 2 WHERE id = 2;
SELECT value FROM update_test WHERE id = 2;
-- EXPECT: 1 rows

-- === CASE: update_to_null ===
UPDATE update_test SET value = NULL WHERE id = 3;
SELECT value FROM update_test WHERE id = 3;
-- EXPECT: 1 rows

-- === CASE: update_all_rows ===
UPDATE update_test SET name = 'X';
SELECT COUNT(*) FROM update_test WHERE name = 'X';
-- EXPECT: 4 rows
