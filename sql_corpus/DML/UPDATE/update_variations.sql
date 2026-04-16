-- === SKIP ===

-- === SKIP ===

-- SQLCorpus: UPDATE Variations
-- Tests for various UPDATE patterns

-- === SETUP ===
CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT, value INTEGER, status TEXT);
INSERT INTO items VALUES (1, 'Item1', 100, 'active'), (2, 'Item2', 200, 'active'), (3, 'Item3', 300, 'inactive'), (4, 'Item4', 400, 'active');

-- === CASE: update_single ===
UPDATE items SET value = 150 WHERE id = 1;
SELECT value FROM items WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: update_multiple ===
UPDATE items SET value = value + 50 WHERE status = 'active';
SELECT COUNT(*) FROM items WHERE value > 200;
-- EXPECT: 2 rows

-- === CASE: update_text ===
UPDATE items SET name = 'Updated' WHERE id = 2;
SELECT name FROM items WHERE id = 2;
-- EXPECT: 1 rows

-- === CASE: update_with_expression ===
UPDATE items SET value = value * 2 WHERE id = 3;
SELECT value FROM items WHERE id = 3;
-- EXPECT: 1 rows

-- === CASE: update_with_concat ===
UPDATE items SET name = name || '_v2' WHERE id = 4;
SELECT name FROM items WHERE id = 4;
-- EXPECT: 1 rows

-- === CASE: update_with_null ===
UPDATE items SET status = NULL WHERE id = 1;
SELECT status FROM items WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: update_multiple_columns ===
UPDATE items SET name = 'Multi', value = 999 WHERE id = 2;
SELECT name, value FROM items WHERE id = 2;
-- EXPECT: 1 rows

-- === CASE: update_all ===
UPDATE items SET status = 'archived';
SELECT COUNT(*) FROM items WHERE status = 'archived';
-- EXPECT: 4 rows

-- === CASE: update_with_in ===
UPDATE items SET value = 0 WHERE id IN (1, 3);
SELECT COUNT(*) FROM items WHERE value = 0;
-- EXPECT: 2 rows

-- === CASE: update_with_between ===
UPDATE items SET value = 500 WHERE value BETWEEN 100 AND 300;
SELECT COUNT(*) FROM items WHERE value = 500;
-- EXPECT: 3 rows

-- === CASE: update_with_like ===
UPDATE items SET status = 'updated' WHERE name LIKE 'Item%';
SELECT COUNT(*) FROM items WHERE status = 'updated';
-- EXPECT: 4 rows

-- === CASE: update_with_subquery ===
UPDATE items SET value = (SELECT MAX(value) FROM items) WHERE id = 1;
SELECT value FROM items WHERE id = 1;
-- EXPECT: 1 rows