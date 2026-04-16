-- === Numeric Operations Test Suite ===

-- === CASE: ABS function ===
-- EXPECT: 1 row
SELECT ABS(-42) as absolute_value;

-- === CASE: ABS with negative decimal ===
-- EXPECT: 1 row
SELECT ABS(-3.14) as absolute_decimal;

-- === CASE: CEIL function ===
-- EXPECT: 1 row
SELECT CEIL(3.14) as ceiling_value;

-- === CASE: FLOOR function ===
-- EXPECT: 1 row
SELECT FLOOR(3.14) as floor_value;

-- === CASE: ROUND function ===
-- EXPECT: 1 row
SELECT ROUND(3.14159, 2) as rounded;

-- === CASE: ROUND with 0 decimals ===
-- EXPECT: 1 row
SELECT ROUND(3.5, 0) as rounded_int;

-- === CASE: TRUNC function ===
-- EXPECT: 1 row
SELECT TRUNC(3.14159, 2) as truncated;

-- === CASE: MOD function ===
-- EXPECT: 1 row
SELECT MOD(10, 3) as remainder;

-- === CASE: POWER function ===
-- EXPECT: 1 row
SELECT POWER(2, 3) as power_value;

-- === CASE: SQRT function ===
-- EXPECT: 1 row
SELECT SQRT(16) as square_root;

-- === CASE: RANDOM function ===
-- EXPECT: 1 row
SELECT RANDOM() as random_value;

-- === CASE: RANDOM with seed ===
-- EXPECT: 1 row
SELECT RANDOM(42) as seeded_random;

-- === CASE: Addition operator ===
-- EXPECT: 1 row
SELECT 10 + 5 as sum_value;

-- === CASE: Subtraction operator ===
-- EXPECT: 1 row
SELECT 10 - 5 as diff_value;

-- === CASE: Multiplication operator ===
-- EXPECT: 1 row
SELECT 10 * 5 as product_value;

-- === CASE: Division operator ===
-- EXPECT: 1 row
SELECT 10 / 5 as quotient_value;

-- === CASE: Division by zero handling ===
-- EXPECT: 1 row
SELECT CASE WHEN 10 = 0 THEN 0 ELSE 10 / 10 END as safe_division;

-- === CASE: Modulo operator ===
-- EXPECT: 1 row
SELECT 10 % 3 as modulo_value;

-- === CASE: Numeric with aggregate ===
-- EXPECT: 1 row
SELECT SUM(total) / COUNT(*) as average_order FROM orders;

-- === CASE: Numeric comparison ===
-- EXPECT: 5 rows
SELECT * FROM orders WHERE total > 100 AND total < 500;

-- === CASE: BETWEEN with numeric ===
-- EXPECT: 5 rows
SELECT * FROM orders WHERE total BETWEEN 100 AND 500;
