-- SQLCorpus: INSERT Tests
-- Tests for various INSERT operations

-- === SETUP ===
CREATE TABLE insert_test (id INTEGER PRIMARY KEY, name TEXT, value INTEGER);
INSERT INTO insert_test VALUES (1, 'A', 100);

-- === CASE: insert_single_row ===
INSERT INTO insert_test VALUES (2, 'B', 200);
SELECT * FROM insert_test WHERE id = 2;
-- EXPECT: 1 rows

-- === CASE: insert_multiple_rows ===
INSERT INTO insert_test VALUES (3, 'C', 300), (4, 'D', 400);
SELECT COUNT(*) FROM insert_test;
-- EXPECT: 1 rows

-- === CASE: insert_with_null ===
INSERT INTO insert_test VALUES (5, NULL, NULL);
SELECT * FROM insert_test WHERE id = 5;
-- EXPECT: 1 rows

-- === CASE: insert_into_subset_columns ===
INSERT INTO insert_test (id, name) VALUES (6, 'F');
SELECT * FROM insert_test WHERE id = 6;
-- EXPECT: 1 rows

-- === CASE: insert_select ===
INSERT INTO insert_test (id, name, value) SELECT id + 10, name, value FROM insert_test WHERE id = 1;
SELECT * FROM insert_test WHERE id = 11;
-- EXPECT: 1 rows
