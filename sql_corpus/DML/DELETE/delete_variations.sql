-- === SKIP ===

-- SQLCorpus: DELETE Variations
-- Tests for various DELETE patterns

-- === SETUP ===
CREATE TABLE records (id INTEGER PRIMARY KEY, name TEXT, value INTEGER);
INSERT INTO records VALUES (1, 'A', 100), (2, 'B', 200), (3, 'C', 300), (4, 'D', 400), (5, 'E', 500);

-- === CASE: delete_single ===
DELETE FROM records WHERE id = 1;
SELECT COUNT(*) FROM records;
-- EXPECT: 4 rows

-- === CASE: delete_multiple ===
DELETE FROM records WHERE value > 350;
SELECT COUNT(*) FROM records;
-- EXPECT: 2 rows

-- === CASE: delete_with_in ===
DELETE FROM records WHERE id IN (2, 4);
SELECT COUNT(*) FROM records;
-- EXPECT: 1 rows

-- === CASE: delete_with_between ===
DELETE FROM records WHERE value BETWEEN 150 AND 250;
SELECT COUNT(*) FROM records;
-- EXPECT: 1 rows

-- === CASE: delete_with_like ===
DELETE FROM records WHERE name LIKE 'C%';
SELECT COUNT(*) FROM records;
-- EXPECT: 1 rows

-- === CASE: delete_with_subquery ===
DELETE FROM records WHERE value > (SELECT AVG(value) FROM records);
SELECT COUNT(*) FROM records;
-- EXPECT: 2 rows

-- === CASE: delete_all ===
DELETE FROM records;
SELECT COUNT(*) FROM records;
-- EXPECT: 0 rows

-- === CASE: delete_with_and ===
DELETE FROM records WHERE value > 100 AND name = 'B';
SELECT COUNT(*) FROM records;
-- EXPECT: 3 rows

-- === CASE: delete_with_or ===
DELETE FROM records WHERE value < 100 OR value > 400;
SELECT COUNT(*) FROM records;
-- EXPECT: 3 rows

-- === CASE: delete_with_exists ===
DELETE FROM records WHERE EXISTS (SELECT 1 FROM records WHERE value > 400);
SELECT COUNT(*) FROM records;
-- EXPECT: 3 rows

-- === CASE: delete_with_not ===
DELETE FROM records WHERE NOT value > 200;
SELECT COUNT(*) FROM records;
-- EXPECT: 2 rows

-- === CASE: delete_limit ===
DELETE FROM records WHERE id = (SELECT id FROM records ORDER BY id DESC LIMIT 1);
SELECT COUNT(*) FROM records;
-- EXPECT: 4 rows