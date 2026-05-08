-- === DIFFERENTIAL TEST: Integer Overflow ===
-- Purpose: Compare integer overflow behavior between SQLRustGo and standard SQL
-- Critical: Overflow behavior varies between engines

-- === SETUP ===
CREATE TABLE overflow_test (id INT PRIMARY KEY, val INT, big_val BIGINT);

INSERT INTO overflow_test VALUES (1, 2147483647, 9223372036854775807);
INSERT INTO overflow_test VALUES (2, -2147483648, -9223372036854775808);
INSERT INTO overflow_test VALUES (3, 0, 0);
INSERT INTO overflow_test VALUES (4, 1, 1);

-- === CASE: Maximum integer addition ===
SELECT val + 1 AS overflow_positive FROM overflow_test WHERE id = 1;
-- EXPECT: varies by engine (overflow/wrap/error)

-- === CASE: Minimum integer subtraction ===
SELECT val - 1 AS overflow_negative FROM overflow_test WHERE id = 2;
-- EXPECT: varies by engine

-- === CASE: Bigint maximum ===
SELECT big_val + 1 AS big_overflow FROM overflow_test WHERE id = 1;
-- EXPECT: varies by engine

-- === CASE: Bigint minimum ===
SELECT big_val - 1 AS big_underflow FROM overflow_test WHERE id = 2;
-- EXPECT: varies by engine

-- === CASE: Multiplication overflow ===
SELECT val * 2 AS multiply_overflow FROM overflow_test WHERE id = 1;
-- EXPECT: varies by engine

-- === CASE: Division at boundary ===
SELECT val / 2 AS divide_boundary FROM overflow_test WHERE id = 1;
-- EXPECT: rows 1

-- === CASE: Modulo at boundary ===
SELECT val % 2 AS modulo_boundary FROM overflow_test WHERE id = 1;
-- EXPECT: rows 1

-- === CASE: Addition with NULL ===
SELECT val + NULL AS add_null FROM overflow_test WHERE id = 1;
-- EXPECT: NULL

-- === CASE: Multiplication by zero ===
SELECT val * 0 AS multiply_zero FROM overflow_test WHERE id = 1;
-- EXPECT: rows 1

-- === CASE: Integer ABS at boundary ===
SELECT ABS(val) AS abs_boundary FROM overflow_test WHERE id = 2;
-- EXPECT: may overflow for minimum integer

-- === CASE: Integer negation at boundary ===
SELECT -val AS negate_boundary FROM overflow_test WHERE id = 2;
-- EXPECT: may overflow for minimum integer

-- === CASE: Cast overflow ===
SELECT CAST(2147483648 AS INT) AS cast_overflow;
-- EXPECT: varies by engine

-- === CASE: Subtraction with result out of range ===
SELECT val - (-2147483648) AS subtract_overflow FROM overflow_test WHERE id = 1;
-- EXPECT: varies by engine