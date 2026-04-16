-- SQLCorpus: ORDER BY Variations
-- Tests for result ordering

-- === SETUP ===
CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT, value INTEGER, category TEXT);
INSERT INTO items VALUES (1, 'Alpha', 100, 'A');
INSERT INTO items VALUES (2, 'Beta', 50, 'B');
INSERT INTO items VALUES (3, 'Gamma', 200, 'A');
INSERT INTO items VALUES (4, 'Delta', 75, 'C');
INSERT INTO items VALUES (5, 'Epsilon', 150, 'B');
INSERT INTO items VALUES (6, 'Zeta', 25, 'C');

-- === CASE: order_by_single_asc ===
SELECT * FROM items ORDER BY value ASC;
-- EXPECT: 6 rows

-- === CASE: order_by_single_desc ===
SELECT * FROM items ORDER BY value DESC;
-- EXPECT: 6 rows

-- === CASE: order_by_text_asc ===
SELECT * FROM items ORDER BY name ASC;
-- EXPECT: 6 rows

-- === CASE: order_by_text_desc ===
SELECT * FROM items ORDER BY name DESC;
-- EXPECT: 6 rows

-- === CASE: order_by_multiple_columns ===
SELECT * FROM items ORDER BY category ASC, value DESC;
-- EXPECT: 6 rows

-- === CASE: order_by_with_limit ===
SELECT * FROM items ORDER BY value DESC LIMIT 3;
-- EXPECT: 3 rows

-- === CASE: order_by_with_offset ===
SELECT * FROM items ORDER BY value ASC LIMIT 3 OFFSET 2;
-- EXPECT: 3 rows

-- === CASE: order_by_with_where ===
SELECT * FROM items WHERE category = 'A' ORDER BY value DESC;
-- EXPECT: 2 rows

-- === CASE: order_by_expression ===
SELECT * FROM items ORDER BY value * 2 DESC;
-- EXPECT: 6 rows

-- === CASE: order_by_nulls_first ===
SELECT * FROM items ORDER BY value ASC NULLS FIRST;
-- EXPECT: 6 rows

-- === CASE: order_by_nulls_last ===
SELECT * FROM items ORDER BY value ASC NULLS LAST;
-- EXPECT: 6 rows

-- === CASE: order_by_alias ===
SELECT name, value as val FROM items ORDER BY val DESC;
-- EXPECT: 6 rows

-- === CASE: order_by_position ===
SELECT name, value, category FROM items ORDER BY 2 DESC;
-- EXPECT: 6 rows

-- === CASE: order_by_grouped ===
SELECT category, SUM(value) FROM items GROUP BY category ORDER BY SUM(value) DESC;
-- EXPECT: 3 rows

-- === CASE: order_by_distinct ===
SELECT DISTINCT category FROM items ORDER BY category ASC;
-- EXPECT: 3 rows