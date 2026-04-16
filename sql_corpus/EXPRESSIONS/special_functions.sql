-- === Special Functions Test Suite ===

-- === CASE: COALESCE with NULLs ===
-- EXPECT: 10 rows
SELECT id, name, COALESCE(email, 'no_email@example.com') as email FROM users;

-- === CASE: COALESCE multiple arguments ===
-- EXPECT: 10 rows
SELECT id, COALESCE(email, name, 'unknown') as fallback FROM users;

-- === CASE: NULLIF equal values ===
-- EXPECT: 1 row
SELECT NULLIF(5, 5) as null_result;

-- === CASE: NULLIF different values ===
-- EXPECT: 1 row
SELECT NULLIF(5, 10) as non_null_result;

-- === CASE: NULLIF with NULL ===
-- EXPECT: 1 row
SELECT NULLIF(NULL, 5) as null_result;

-- === CASE: IFNULL ===
-- EXPECT: 10 rows
SELECT id, name, IFNULL(email, 'fallback@example.com') as email FROM users;

-- === CASE: IIF function ===
-- EXPECT: 1 row
SELECT IIF(1 > 0, 'true', 'false') as result;

-- === CASE: IIF false condition ===
-- EXPECT: 1 row
SELECT IIF(1 < 0, 'true', 'false') as result;

-- === CASE: IF with aggregate ===
-- EXPECT: 1 row
SELECT IF(COUNT(*) > 5, 'Many', 'Few') as result FROM users;

-- === CASE: ABS function ===
-- EXPECT: 1 row
SELECT ABS(-42) as absolute_value;

-- === CASE: MOD function ===
-- EXPECT: 1 row
SELECT MOD(10, 3) as remainder;

-- === CASE: ROUND function ===
-- EXPECT: 1 row
SELECT ROUND(3.14159, 2) as rounded;

-- === CASE: LENGTH with COALESCE ===
-- EXPECT: 10 rows
SELECT id, LENGTH(COALESCE(name, '')) as name_length FROM users;

-- === CASE: UPPER/LOWER with IFNULL ===
-- EXPECT: 10 rows
SELECT id, UPPER(IFNULL(name, 'UNKNOWN')) as upper_name FROM users;

-- === CASE: SUBSTR with NULL handling ===
-- EXPECT: 10 rows
SELECT id, SUBSTR(COALESCE(name, 'UNKNOWN'), 1, 3) as short_name FROM users;

-- === CASE: INLINE IF in SELECT ===
-- EXPECT: 10 rows
SELECT id, name, IIF(id > 5, 'High', 'Low') as category FROM users;

-- === CASE: Nested COALESCE ===
-- EXPECT: 10 rows
SELECT id, COALESCE(email, name, 'anonymous') as identifier FROM users;

-- === CASE: IFNULL with numeric ===
-- EXPECT: 1 row
SELECT IFNULL(NULL, 100) + 50 as numeric_result;
