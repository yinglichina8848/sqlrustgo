-- === DIFFERENTIAL TEST: Type Boundaries and Precision Comparison ===
-- Purpose: Compare type handling between SQLRustGo and MySQL 5.7
-- Critical: Float precision, integer overflow, and decimal precision may differ

-- === SETUP ===
CREATE TABLE type_test (id INT PRIMARY KEY, int_col INT, bigint_col BIGINT, float_col FLOAT, double_col DOUBLE, decimal_col DECIMAL(10,2), varchar_col VARCHAR(10));
CREATE TABLE numeric_ops (a DECIMAL(5,2), b DECIMAL(5,2));

INSERT INTO type_test VALUES (1, 0, 0, 0.0, 0.0, 0.00, 'text');
INSERT INTO type_test VALUES (2, 127, 9223372036854775807, 3.14, 3.14159265358979, 99999.99, 'short');
INSERT INTO type_test VALUES (3, -128, -9223372036854775808, -3.14, -3.14159265358979, -99999.99, 'longertext');
INSERT INTO type_test VALUES (4, 32767, 2147483647, 1.7976931348623157e+308, 1.7976931348623157e+308, 12345.67, 'abc');
INSERT INTO type_test VALUES (5, -32768, -2147483648, -1.7976931348623157e+308, -1.7976931348623157e+308, -12345.67, 'xyz');

INSERT INTO numeric_ops VALUES (10.25, 3.75);
INSERT INTO numeric_ops VALUES (100.00, 50.00);
INSERT INTO numeric_ops VALUES (0.1, 0.2);
INSERT INTO numeric_ops VALUES (1.005, 2.010);
INSERT INTO numeric_ops VALUES (100, 50);

-- === CASE: Integer maximum value
SELECT * FROM type_test WHERE int_col = 32767;
-- EXPECT: rows 1

-- === CASE: Integer minimum value
SELECT * FROM type_test WHERE int_col = -32768;
-- EXPECT: rows 1

-- === CASE: Float with precision
SELECT float_col FROM type_test WHERE id = 2;
-- EXPECT: rows 1

-- === CASE: Double with precision
SELECT double_col FROM type_test WHERE id = 2;
-- EXPECT: rows 1

-- === CASE: Decimal precision in arithmetic
SELECT a + b AS add_result FROM numeric_ops WHERE a = 10.25;
-- EXPECT: rows 1

-- === CASE: Decimal precision in multiplication
SELECT a * b AS mul_result FROM numeric_ops WHERE a = 10.25;
-- EXPECT: rows 1

-- === CASE: Decimal division
SELECT a / b AS div_result FROM numeric_ops WHERE a = 100.00;
-- EXPECT: rows 1

-- === CASE: Floating point precision issue (0.1 + 0.2)
-- This exposes floating point representation differences
SELECT a + b AS sum_0_1_0_2 FROM numeric_ops WHERE a = 0.1;
-- EXPECT: rows 1

-- === CASE: Round half up for decimals
SELECT ROUND(2.5, 0) AS round_half_up;
-- EXPECT: rows 1

-- === CASE: Round half down for decimals
SELECT ROUND(2.5, 0) AS round_half_down;
-- EXPECT: rows 1

-- === CASE: Ceiling function
SELECT CEIL(2.3) AS ceil_result;
-- EXPECT: rows 1

-- === CASE: Floor function
SELECT FLOOR(2.7) AS floor_result;
-- EXPECT: rows 1

-- === CASE: Absolute value of negative
SELECT ABS(-10) AS abs_result;
-- EXPECT: rows 1

-- === CASE: Modulo operation
SELECT MOD(10, 3) AS mod_result;
-- EXPECT: rows 1

-- === CASE: Integer overflow in addition
SELECT int_col + 1 AS overflow_test FROM type_test WHERE int_col = 32767;
-- EXPECT: rows 1

-- === CASE: Integer underflow in subtraction
SELECT int_col - 1 AS underflow_test FROM type_test WHERE int_col = -32768;
-- EXPECT: rows 1

-- === CASE: VARCHAR truncation
SELECT varchar_col FROM type_test WHERE id = 3;
-- EXPECT: rows 1 (should truncate 'longertext' to 'longertext' - 10 chars)

-- === CASE: String length
SELECT LENGTH(varchar_col) AS str_len FROM type_test WHERE id = 3;
-- EXPECT: rows 1

-- === CASE: DECIMAL overflow (exceed precision)
SELECT decimal_col FROM type_test WHERE decimal_col > 100000;
-- EXPECT: rows 0

-- === CASE: Zero division
SELECT 1 / 0 AS zero_div;
-- EXPECT: rows 1 (MySQL returns NULL for div by zero, may error in SQLRustGo)

-- === CASE: MOD by zero
SELECT MOD(10, 0) AS mod_zero;
-- EXPECT: rows 1 (MySQL returns NULL, SQLRustGo may error)

-- === CASE: Float infinity representation
SELECT * FROM type_test WHERE float_col > 1.0e+308;
-- EXPECT: rows 2

-- === CASE: Negative infinity
SELECT * FROM type_test WHERE float_col < -1.0e+308;
-- EXPECT: rows 2

-- === CASE: NaN handling
SELECT ISNAN(0.0/0.0) AS is_nan;
-- EXPECT: rows 1

-- === CASE: Square root of negative
SELECT SQRT(-1) AS sqrt_negative;
-- EXPECT: rows 1 (MySQL returns NULL, SQLRustGo may error)

-- === CASE: Log of negative
SELECT LOG(-1) AS log_negative;
-- EXPECT: rows 1 (MySQL returns NULL, SQLRustGo may error)

-- === CASE: Power operation edge cases
SELECT POW(10, 308) AS pow_max;
-- EXPECT: rows 1

-- === CASE: Power with negative exponent
SELECT POW(10, -308) AS pow_min;
-- EXPECT: rows 1

-- === CASE: Bitwise AND on integers
SELECT 10 & 12 AS bitwise_and;
-- EXPECT: rows 1

-- === CASE: Bitwise OR on integers
SELECT 10 | 12 AS bitwise_or;
-- EXPECT: rows 1

-- === CASE: Bitwise XOR on integers
SELECT 10 ^ 12 AS bitwise_xor;
-- EXPECT: rows 1

-- === CASE: Bitwise NOT
SELECT ~10 AS bitwise_not;
-- EXPECT: rows 1

-- === CASE: Left shift
SELECT 1 << 5 AS left_shift;
-- EXPECT: rows 1

-- === CASE: Right shift
SELECT 32 >> 5 AS right_shift;
-- EXPECT: rows 1

-- === CASE: Cast string to int
SELECT CAST('123' AS INT) AS cast_result;
-- EXPECT: rows 1

-- === CASE: Cast int to string
SELECT CAST(123 AS VARCHAR(10)) AS cast_result;
-- EXPECT: rows 1

-- === CASE: Cast string to decimal
SELECT CAST('123.45' AS DECIMAL(10,2)) AS cast_result;
-- EXPECT: rows 1

-- === CASE: Cast with invalid string
SELECT CAST('abc' AS INT) AS cast_invalid;
-- EXPECT: rows 1 (MySQL returns 0, SQLRustGo may error)

-- === CASE: HEX function
SELECT HEX(255) AS hex_result;
-- EXPECT: rows 1

-- === CASE: CONV function (base conversion)
SELECT CONV(255, 10, 16) AS conv_result;
-- EXPECT: rows 1

-- === CASE: INET_ATON and INET_NTOA
SELECT INET_NTOA(INET_ATON('192.168.1.1')) AS inet_result;
-- EXPECT: rows 1
