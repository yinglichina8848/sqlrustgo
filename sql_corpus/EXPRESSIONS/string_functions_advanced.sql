-- === String Functions Advanced Test Suite ===

-- === CASE: ASCII - get ASCII code ===
-- EXPECT: 1 row
SELECT ASCII('A') as ascii_a, ASCII('a') as ascii_lower_a;

-- === CASE: CHAR - convert ASCII to string ===
-- EXPECT: 1 row
SELECT CHAR(65, 66, 67) as chars;

-- === CASE: CHARINDEX - find substring position ===
-- EXPECT: 1 row
SELECT CHARINDEX('world', 'hello world') as pos;

-- === CASE: CONCAT_WS - concatenate with separator ===
-- EXPECT: 1 row
SELECT CONCAT_WS(', ', 'Alice', 'Bob', 'Charlie') as names;

-- === CASE: DIFFERENCE - soundex difference ===
-- EXPECT: 1 row
SELECT DIFFERENCE('Smith', 'Smyth') as soundex_diff;

-- === CASE: FORMAT - format number ===
-- EXPECT: 1 row
SELECT FORMAT(1234567.89, 2) as formatted;

-- === CASE: LEFT - left substring ===
-- EXPECT: 1 row
SELECT LEFT('Hello World', 5) as left_5;

-- === CASE: RIGHT - right substring ===
-- EXPECT: 1 row
SELECT RIGHT('Hello World', 5) as right_5;

-- === CASE: LEN - length (excludes trailing spaces) ===
-- EXPECT: 1 row
SELECT LEN('hello   ') as len_val;

-- === CASE: DATALENGTH - data length (includes trailing spaces) ===
-- EXPECT: 1 row
SELECT DATALENGTH('hello   ') as datalen_val;

-- === CASE: LOWER - lowercase ===
-- EXPECT: 1 row
SELECT LOWER('HELLO World') as lower_val;

-- === CASE: UPPER - uppercase ===
-- EXPECT: 1 row
SELECT UPPER('hello World') as upper_val;

-- === CASE: LTRIM - left trim ===
-- EXPECT: 1 row
SELECT LTRIM('   hello') as ltrimmed;

-- === CASE: RTRIM - right trim ===
-- EXPECT: 1 row
SELECT RTRIM('hello   ') as rtrimmed;

-- === CASE: TRIM - both sides trim ===
-- EXPECT: 1 row
SELECT TRIM('   hello   ') as trimmed;

-- === CASE: NCHAR - national character ===
-- EXPECT: 1 row
SELECT NCHAR(65) as nchar_val;

-- === CASE: PATINDEX - pattern index ===
-- EXPECT: 1 row
SELECT PATINDEX('%world%', 'hello world') as pattern_pos;

-- === CASE: QUOTENAME - quote identifier ===
-- EXPECT: 1 row
SELECT QUOTENAME('table_name') as quoted_name;

-- === CASE: REPLACE - replace substring ===
-- EXPECT: 1 row
SELECT REPLACE('hello world', 'world', 'SQL') as replaced;

-- === CASE: REPLICATE - replicate string ===
-- EXPECT: 1 row
SELECT REPLICATE('ab', 3) as replicated;

-- === CASE: REVERSE - reverse string ===
-- EXPECT: 1 row
SELECT REVERSE('hello') as reversed;

-- === CASE: RIGHT with NULL ===
-- EXPECT: 1 row
SELECT RIGHT(NULL, 5) as null_result;

-- === CASE: RTRIM with custom chars ===
-- EXPECT: 1 row
SELECT RTRIM('hello!!!', '!') as rtrim_custom;

-- === CASE: SOUNDEX - soundex code ===
-- EXPECT: 1 row
SELECT SOUNDEX('Smith') as soundex_val;

-- === CASE: SPACE - generate spaces ===
-- EXPECT: 1 row
SELECT 'hello' || SPACE(5) || 'world' as spaced;

-- === CASE: STR - number to string ===
-- EXPECT: 1 row
SELECT STR(123.45, 8, 2) as str_val;

-- === CASE: STUFF - stuff string ===
-- EXPECT: 1 row
SELECT STUFF('hello world', 7, 5, 'SQL') as stuffed;

-- === CASE: TRANSLATE - translate characters ===
-- EXPECT: 1 row
SELECT TRANSLATE('hello', 'elo', '321') as translated;

-- === CASE: UNICODE - unicode code point ===
-- EXPECT: 1 row
SELECT UNICODE('A') as unicode_a, UNICODE('Á') as unicode_acute;

-- === CASE: SUBSTRING with NULL ===
-- EXPECT: 1 row
SELECT SUBSTRING(NULL, 1, 5) as null_substr;

-- === CASE: SUBSTRING with expressions ===
-- EXPECT: 1 row
SELECT SUBSTRING('hello world', LEN('hello') + 2, 5) as expr_substr;
