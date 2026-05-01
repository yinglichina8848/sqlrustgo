-- === SKIP ===

-- SQLCorpus: Expression Tests
-- Tests for various SQL expressions and operators

-- === SETUP ===
CREATE TABLE expr_test (id INTEGER PRIMARY KEY, a INTEGER, b INTEGER, c INTEGER);
INSERT INTO expr_test VALUES (1, 10, 20, 30), (2, 5, 15, 25), (3, 100, 50, 75);

-- === CASE: arithmetic_add ===
SELECT a + b FROM expr_test WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: arithmetic_subtract ===
SELECT a - b FROM expr_test WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: arithmetic_multiply ===
SELECT a * b FROM expr_test WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: arithmetic_divide ===
SELECT a / b FROM expr_test WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: arithmetic_modulo ===
SELECT a % b FROM expr_test WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: comparison_eq ===
SELECT * FROM expr_test WHERE a = 10;
-- EXPECT: 1 rows

-- === CASE: comparison_ne ===
SELECT * FROM expr_test WHERE a != 10;
-- EXPECT: 2 rows

-- === CASE: comparison_lt ===
SELECT * FROM expr_test WHERE a < 10;
-- EXPECT: 1 rows

-- === CASE: comparison_lte ===
SELECT * FROM expr_test WHERE a <= 10;
-- EXPECT: 2 rows

-- === CASE: comparison_gt ===
SELECT * FROM expr_test WHERE a > 10;
-- EXPECT: 1 rows

-- === CASE: comparison_gte ===
SELECT * FROM expr_test WHERE a >= 10;
-- EXPECT: 2 rows

-- === CASE: logical_and ===
SELECT * FROM expr_test WHERE a > 5 AND b < 30;
-- EXPECT: 2 rows

-- === CASE: logical_or ===
SELECT * FROM expr_test WHERE a > 50 OR b > 40;
-- EXPECT: 2 rows

-- === CASE: logical_not ===
SELECT * FROM expr_test WHERE NOT a > 50;
-- EXPECT: 2 rows

-- === CASE: between ===
SELECT * FROM expr_test WHERE a BETWEEN 5 AND 15;
-- EXPECT: 2 rows

-- === CASE: not_between ===
SELECT * FROM expr_test WHERE a NOT BETWEEN 5 AND 15;
-- EXPECT: 1 rows

-- === CASE: like ===
CREATE TABLE like_test (id INTEGER, name TEXT);
INSERT INTO like_test VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Allen');
SELECT * FROM like_test WHERE name LIKE 'A%';
-- EXPECT: 2 rows

-- === CASE: not_like ===
SELECT * FROM like_test WHERE name NOT LIKE 'A%';
-- EXPECT: 1 rows

-- === CASE: in_list ===
SELECT * FROM expr_test WHERE a IN (5, 10, 15);
-- EXPECT: 2 rows

-- === CASE: not_in_list ===
SELECT * FROM expr_test WHERE a NOT IN (5, 10, 15);
-- EXPECT: 1 rows

-- === CASE: case_expression ===
SELECT CASE WHEN a > 50 THEN 'HIGH' WHEN a > 20 THEN 'MED' ELSE 'LOW' END FROM expr_test WHERE id = 1;
-- EXPECT: 1 rows
