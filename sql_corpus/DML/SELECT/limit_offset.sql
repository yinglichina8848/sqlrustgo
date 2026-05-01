-- SQLCorpus: LIMIT and OFFSET
-- Tests for pagination

-- === SETUP ===
CREATE TABLE numbers (id INTEGER PRIMARY KEY, value INTEGER);
INSERT INTO numbers VALUES (1, 10);
INSERT INTO numbers VALUES (2, 20);
INSERT INTO numbers VALUES (3, 30);
INSERT INTO numbers VALUES (4, 40);
INSERT INTO numbers VALUES (5, 50);
INSERT INTO numbers VALUES (6, 60);
INSERT INTO numbers VALUES (7, 70);
INSERT INTO numbers VALUES (8, 80);

-- === CASE: limit_3 ===
SELECT * FROM numbers LIMIT 3;
-- EXPECT: 3 rows

-- === CASE: limit_5 ===
SELECT * FROM numbers LIMIT 5;
-- EXPECT: 5 rows

-- === CASE: limit_all ===
SELECT * FROM numbers LIMIT 100;
-- EXPECT: 8 rows

-- === CASE: offset_2 ===
SELECT * FROM numbers OFFSET 2;
-- EXPECT: 6 rows

-- === CASE: offset_5 ===
SELECT * FROM numbers OFFSET 5;
-- EXPECT: 3 rows

-- === CASE: limit_offset ===
SELECT * FROM numbers LIMIT 3 OFFSET 2;
-- EXPECT: 3 rows

-- === CASE: limit_0 ===
SELECT * FROM numbers LIMIT 0;
-- EXPECT: 0 rows

-- === CASE: limit_with_order ===
SELECT * FROM numbers ORDER BY value DESC LIMIT 3;
-- EXPECT: 3 rows

-- === CASE: limit_with_where ===
SELECT * FROM numbers WHERE value > 30 LIMIT 2;
-- EXPECT: 2 rows

-- === CASE: limit_with_group ===
SELECT COUNT(*) FROM numbers GROUP BY value > 30 LIMIT 2;
-- EXPECT: 2 rows