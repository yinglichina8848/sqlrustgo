-- SQLCorpus: UNION, INTERSECT, EXCEPT
-- Tests for set operations

-- === SETUP ===
CREATE TABLE set_a (id INTEGER PRIMARY KEY, val TEXT);
CREATE TABLE set_b (id INTEGER PRIMARY KEY, val TEXT);
INSERT INTO set_a VALUES (1, 'apple'), (2, 'banana'), (3, 'cherry');
INSERT INTO set_b VALUES (2, 'banana'), (3, 'cherry'), (4, 'date');

-- === CASE: union_all ===
SELECT val FROM set_a UNION ALL SELECT val FROM set_b;
-- EXPECT: 6 rows

-- === CASE: union ===
SELECT val FROM set_a UNION SELECT val FROM set_b;
-- EXPECT: 4 rows

-- === CASE: intersect ===
SELECT val FROM set_a INTERSECT SELECT val FROM set_b;
-- EXPECT: 2 rows

-- === CASE: except ===
SELECT val FROM set_a EXCEPT SELECT val FROM set_b;
-- EXPECT: 1 rows

-- === CASE: union_with_order ===
SELECT val FROM set_a UNION SELECT val FROM set_b ORDER BY val;
-- EXPECT: 4 rows

-- === CASE: union_with_limit ===
SELECT val FROM set_a UNION SELECT val FROM set_b LIMIT 2;
-- EXPECT: 2 rows

-- === CASE: nested_union ===
SELECT val FROM set_a UNION (SELECT val FROM set_b WHERE val > 'b');
-- EXPECT: 3 rows

-- === CASE: union_distinct ===
SELECT DISTINCT val FROM set_a UNION SELECT DISTINCT val FROM set_b;
-- EXPECT: 4 rows