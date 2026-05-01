-- === SKIP ===

-- SQLCorpus: Numeric Functions
-- Tests for numeric/mathematical functions

-- === SETUP ===
CREATE TABLE numbers (id INTEGER PRIMARY KEY, n INTEGER, f REAL);
INSERT INTO numbers VALUES (1, 10, 3.14), (2, -5, 2.71), (3, 0, 0.0), (4, 100, 1.5);

-- === CASE: abs ===
SELECT ABS(n) FROM numbers;
-- EXPECT: 4 rows

-- === CASE: abs_negative ===
SELECT ABS(n) FROM numbers WHERE n < 0;
-- EXPECT: 1 rows

-- === CASE: round ===
SELECT ROUND(f, 1) FROM numbers;
-- EXPECT: 4 rows

-- === CASE: ceil ===
SELECT CEIL(f) FROM numbers;
-- EXPECT: 4 rows

-- === CASE: floor ===
SELECT FLOOR(f) FROM numbers;
-- EXPECT: 4 rows

-- === CASE: random ===
SELECT RANDOM();
-- EXPECT: 1 rows

-- === CASE: sqrt ===
SELECT SQRT(n) FROM numbers WHERE n >= 0;
-- EXPECT: 3 rows

-- === CASE: power ===
SELECT POWER(n, 2) FROM numbers WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: mod ===
SELECT MOD(n, 3) FROM numbers WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: sign_positive ===
SELECT SIGN(n) FROM numbers WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: sign_negative ===
SELECT SIGN(n) FROM numbers WHERE id = 2;
-- EXPECT: 1 rows

-- === CASE: sign_zero ===
SELECT SIGN(n) FROM numbers WHERE id = 3;
-- EXPECT: 1 rows

-- === CASE: max_int ===
SELECT MAX(n) FROM numbers;
-- EXPECT: 1 rows

-- === CASE: min_int ===
SELECT MIN(n) FROM numbers;
-- EXPECT: 1 rows

-- === CASE: max_real ===
SELECT MAX(f) FROM numbers;
-- EXPECT: 1 rows

-- === CASE: min_real ===
SELECT MIN(f) FROM numbers;
-- EXPECT: 1 rows