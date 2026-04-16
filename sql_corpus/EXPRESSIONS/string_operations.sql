-- === String Operations Test Suite ===

-- === CASE: LENGTH function ===
-- EXPECT: 10 rows
SELECT id, name, LENGTH(name) as name_length FROM users;

-- === CASE: UPPER function ===
-- EXPECT: 10 rows
SELECT id, UPPER(name) as upper_name FROM users;

-- === CASE: LOWER function ===
-- EXPECT: 10 rows
SELECT id, LOWER(name) as lower_name FROM users;

-- === CASE: SUBSTR function ===
-- EXPECT: 10 rows
SELECT id, SUBSTR(name, 1, 3) as short_name FROM users;

-- === CASE: SUBSTR with negative length ===
-- EXPECT: 10 rows
SELECT id, SUBSTR(name, -3) as last_three FROM users;

-- === CASE: INSTR function ===
-- EXPECT: 5 rows
SELECT id, name, INSTR(name, 'oh') as pos FROM users WHERE id <= 5;

-- === CASE: REPLACE function ===
-- EXPECT: 10 rows
SELECT id, REPLACE(name, 'A', 'X') as replaced FROM users;

-- === CASE: TRIM function ===
-- EXPECT: 1 row
SELECT TRIM('   spaces   ') as trimmed;

-- === CASE: LTRIM function ===
-- EXPECT: 1 row
SELECT LTRIM('   left') as ltrimmed;

-- === CASE: RTRIM function ===
-- EXPECT: 1 row
SELECT RTRIM('right   ') as rtrimmed;

-- === CASE: LPAD function ===
-- EXPECT: 1 row
SELECT LPAD('pad', 7, '*') as lpad_result;

-- === CASE: RPAD function ===
-- EXPECT: 1 row
SELECT RPAD('pad', 7, '*') as rpad_result;

-- === CASE: REVERSE function ===
-- EXPECT: 1 row
SELECT REVERSE('stressed') as reversed;

-- === CASE: CHAR_LENGTH ===
-- EXPECT: 10 rows
SELECT id, CHAR_LENGTH(name) as char_len FROM users;

-- === CASE: CONCAT function ===
-- EXPECT: 10 rows
SELECT id, CONCAT(name, ' - ', email) as full_contact FROM users;

-- === CASE: CONCAT with NULL ===
-- EXPECT: 10 rows
SELECT id, CONCAT(name, ' ', COALESCE(email, '')) as contact FROM users;

-- === CASE: || operator ===
-- EXPECT: 10 rows
SELECT id, name || ' <' || email || '>' as formatted FROM users;

-- === CASE: QUOTE function ===
-- EXPECT: 1 row
SELECT QUOTE('single''s') as quoted;
