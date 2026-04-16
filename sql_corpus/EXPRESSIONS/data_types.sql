-- === Data Types Test Suite ===

-- === CASE: INTEGER type ===
-- EXPECT: 1 row
SELECT CAST(42 AS INTEGER) as int_val;

-- === CASE: TEXT type ===
-- EXPECT: 1 row
SELECT CAST('hello' AS TEXT) as text_val;

-- === CASE: REAL type ===
-- EXPECT: 1 row
SELECT CAST(3.14 AS REAL) as real_val;

-- === CASE: BLOB type ===
-- EXPECT: 1 row
SELECT CAST('binary' AS BLOB) as blob_val;

-- === CASE: Boolean type ===
-- EXPECT: 1 row
SELECT CAST(1 AS BOOLEAN) as bool_val;

-- === CASE: Date type ===
-- EXPECT: 1 row
SELECT CAST('2024-01-15' AS DATE) as date_val;

-- === CASE: Timestamp type ===
-- EXPECT: 1 row
SELECT CAST('2024-01-15 10:30:00' AS TIMESTAMP) as timestamp_val;

-- === CASE: Type conversion from text to integer ===
-- EXPECT: 1 row
SELECT CAST('123' AS INTEGER) + 100 as result;

-- === CASE: Type conversion from integer to text ===
-- EXPECT: 1 row
SELECT CAST(123 AS TEXT) || ' is a number' as result;

-- === CASE: Type conversion from text to real ===
-- EXPECT: 1 row
SELECT CAST('3.14' AS REAL) * 2 as result;

-- === CASE: Type conversion from real to integer ===
-- EXPECT: 1 row
SELECT CAST(3.14 AS INTEGER) as result;

-- === CASE: NULL type handling ===
-- EXPECT: 1 row
SELECT CAST(NULL AS INTEGER) as null_val;

-- === CASE: String length with type cast ===
-- EXPECT: 1 row
SELECT LENGTH(CAST(12345 AS TEXT)) as len;

-- === CASE: Numeric operations with CAST ===
-- EXPECT: 1 row
SELECT CAST(10 AS REAL) / CAST(3 AS REAL) as division;

-- === CASE: Type in WHERE clause ===
-- EXPECT: 3 rows
SELECT * FROM users WHERE CAST(id AS TEXT) LIKE '%5%';

-- === CASE: Implicit type conversion ===
-- EXPECT: 1 row
SELECT 1 + 2.5 as implicit_conversion;

-- === CASE: Date arithmetic ===
-- EXPECT: 1 row
SELECT DATE('2024-01-15') + 7 as next_week;
