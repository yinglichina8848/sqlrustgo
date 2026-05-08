-- === DIFFERENTIAL TEST: Division by Zero ===
-- Purpose: Compare division by zero handling between SQLRustGo and standard SQL
-- Critical: Different engines return different values for division by zero

-- === SETUP ===
CREATE TABLE division_test (id INT PRIMARY KEY, val INT, divisor INT, fval FLOAT, dval DECIMAL(10,2));

INSERT INTO division_test VALUES (1, 10, 2, 10.0, 10.00);
INSERT INTO division_test VALUES (2, 10, 0, 10.0, 10.00);
INSERT INTO division_test VALUES (3, 0, 0, 0.0, 0.00);
INSERT INTO division_test VALUES (4, -10, 2, -10.0, -10.00);
INSERT INTO division_test VALUES (5, 10, 3, 10.0, 10.00);

-- === CASE: Division by non-zero ===
SELECT val / divisor AS normal_division FROM division_test WHERE id = 1;
-- EXPECT: 5

-- === CASE: Integer division by zero ===
SELECT val / divisor AS div_by_zero_int FROM division_test WHERE id = 2;
-- EXPECT: error or NULL depending on engine

-- === CASE: Float division by zero ===
SELECT fval / 0.0 AS div_by_zero_float FROM division_test WHERE id = 1;
-- EXPECT: error or INF depending on engine

-- === CASE: Decimal division by zero ===
SELECT dval / 0 AS div_by_zero_decimal FROM division_test WHERE id = 1;
-- EXPECT: error or NULL depending on engine

-- === CASE: Zero divided by number ===
SELECT 0 / divisor AS zero_divided FROM division_test WHERE id = 1;
-- EXPECT: 0

-- === CASE: Zero divided by zero ===
SELECT 0 / 0 AS zero_div_zero;
-- EXPECT: error or NULL depending on engine

-- === CASE: Division with NULL divisor ===
SELECT val / NULL AS div_null_divisor FROM division_test WHERE id = 1;
-- EXPECT: NULL

-- === CASE: Division with NULL dividend ===
SELECT NULL / divisor AS null_div_dividend FROM division_test WHERE id = 1;
-- EXPECT: NULL

-- === CASE: Modulo by zero ===
SELECT val % divisor AS mod_by_zero FROM division_test WHERE id = 2;
-- EXPECT: error or NULL depending on engine

-- === CASE: Negative divided by positive ===
SELECT -10 / 3 AS neg_div_pos;
-- EXPECT: varies by engine (-3 or -3.333...)

-- === CASE: Positive divided by negative ===
SELECT 10 / -3 AS pos_div_neg;
-- EXPECT: varies by engine (-3 or -3.333...)

-- === CASE: Negative divided by negative ===
SELECT -10 / -3 AS neg_div_neg;
-- EXPECT: varies by engine (3 or 3.333...)

-- === CASE: Division in WHERE clause ===
SELECT * FROM division_test WHERE val / divisor > 2;
-- EXPECT: id 1, 4, 5

-- === CASE: Division with arithmetic ===
SELECT (val + 5) / divisor AS arith_division FROM division_test WHERE id = 1;
-- EXPECT: 7