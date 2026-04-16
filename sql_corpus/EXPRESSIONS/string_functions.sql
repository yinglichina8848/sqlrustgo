-- SQLCorpus: String Functions
-- Tests for string manipulation functions

-- === SETUP ===
CREATE TABLE strings (id INTEGER PRIMARY KEY, s TEXT);
INSERT INTO strings VALUES (1, 'Hello World'), (2, 'SQL Rust'), (3, 'UPPERCASE'), (4, 'lowercase'), (5, '  spaces  ');

-- === CASE: length ===
SELECT LENGTH(s) FROM strings WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: substr ===
SELECT SUBSTR(s, 1, 5) FROM strings WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: substr_from_end ===
SELECT SUBSTR(s, -5) FROM strings WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: trim ===
SELECT TRIM(s) FROM strings WHERE id = 5;
-- EXPECT: 1 rows

-- === CASE: ltrim ===
SELECT LTRIM(s) FROM strings WHERE id = 5;
-- EXPECT: 1 rows

-- === CASE: rtrim ===
SELECT RTRIM(s) FROM strings WHERE id = 5;
-- EXPECT: 1 rows

-- === CASE: upper ===
SELECT UPPER(s) FROM strings WHERE id = 4;
-- EXPECT: 1 rows

-- === CASE: lower ===
SELECT LOWER(s) FROM strings WHERE id = 3;
-- EXPECT: 1 rows

-- === CASE: replace ===
SELECT REPLACE(s, 'World', 'Rust') FROM strings WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: instr ===
SELECT INSTR(s, 'World') FROM strings WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: printf ===
SELECT PRINTF('%s=%d', 'value', 42);
-- EXPECT: 1 rows

-- === CASE: like_case_sensitive ===
SELECT * FROM strings WHERE s LIKE 'hello%';
-- EXPECT: 0 rows

-- === CASE: glob ===
SELECT * FROM strings WHERE s GLOB 'Hello*';
-- EXPECT: 1 rows

-- === CASE: quote ===
SELECT QUOTE(s) FROM strings WHERE id = 1;
-- EXPECT: 1 rows