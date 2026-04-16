-- === Data Type Conversion Functions Test Suite ===

-- === CASE: CAST as INTEGER ===
-- EXPECT: 1 row
SELECT CAST('123' AS INTEGER) + 100 as result;

-- === CASE: CAST as TEXT ===
-- EXPECT: 1 row
SELECT CAST(123 AS TEXT) || ' is text' as result;

-- === CASE: CAST as REAL ===
-- EXPECT: 1 row
SELECT CAST('3.14' AS REAL) * 2 as result;

-- === CASE: CAST as BLOB ===
-- EXPECT: 1 row
SELECT CAST('binary data' AS BLOB) as blob_val;

-- === CASE: CAST as NUMERIC ===
-- EXPECT: 1 row
SELECT CAST('123.45' AS NUMERIC) + 10 as result;

-- === CASE: CAST date to text ===
-- EXPECT: 1 row
SELECT CAST('2024-01-15' AS TEXT) as date_text;

-- === CASE: CAST text to date ===
-- EXPECT: 1 row
SELECT CAST('2024-01-15' AS DATE) as text_date;

-- === CASE: CONVERT from text ===
-- EXPECT: 1 row
SELECT CONVERT(INTEGER, '456') as converted;

-- === CASE: CONVERT to text with style ===
-- EXPECT: 1 row
SELECT CONVERT(TEXT, 123, 0) as converted_text;

-- === CASE: CONVERT datetime ===
-- EXPECT: 1 row
SELECT CONVERT(TEXT, DATETIME('now'), 120) as datetime_text;

-- === CASE: Implicit conversion integer to text ===
-- EXPECT: 1 row
SELECT 123 || '456' as implicit_concat;

-- === CASE: Implicit conversion text to integer ===
-- EXPECT: 1 row
SELECT '100' + 50 as implicit_add;

-- === CASE: TRY_CAST valid ===
-- EXPECT: 1 row
SELECT TRY_CAST('123' AS INTEGER) as try_cast_val;

-- === CASE: TRY_CAST invalid ===
-- EXPECT: 1 row
SELECT TRY_CAST('not_a_number' AS INTEGER) as try_cast_null;

-- === CASE: TRY_CONVERT valid ===
-- EXPECT: 1 row
SELECT TRY_CONVERT(INTEGER, '789') as try_convert_val;

-- === CASE: TRY_CONVERT invalid ===
-- EXPECT: 1 row
SELECT TRY_CONVERT(INTEGER, 'abc') as try_convert_null;

-- === CASE: Explicit CAST boolean ===
-- EXPECT: 1 row
SELECT CAST(1 AS BOOLEAN) as bool_val, CAST(0 AS BOOLEAN) as bool_zero;

-- === CASE: CAST between numeric types ===
-- EXPECT: 1 row
SELECT CAST(3.14159 AS INTEGER) as truncated, CAST(42 AS REAL) as as_real;

-- === CASE: CAST with expressions ===
-- EXPECT: 1 row
SELECT CAST(10 AS REAL) / CAST(3 AS REAL) as division;

-- === CASE: CAST with function calls ===
-- EXPECT: 1 row
SELECT CAST(UPPER('text') AS TEXT) as upper_cast;

-- === CASE: Hex to integer ===
-- EXPECT: 1 row
SELECT CAST('0xFF' AS INTEGER) as hex_val;

-- === CASE: Integer to hex ===
-- EXPECT: 1 row
SELECT HEX(255) as int_to_hex;
