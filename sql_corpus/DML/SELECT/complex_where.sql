-- === Complex WHERE Test Suite ===

-- === CASE: Multiple AND conditions ===
-- EXPECT: 4 rows
SELECT id, name, email, created_at
FROM users
WHERE id > 3 AND email LIKE '%@example.com' AND created_at > '2024-01-01';

-- === CASE: OR with AND precedence ===
-- EXPECT: 7 rows
SELECT id, name, email
FROM users
WHERE (id < 3 OR id > 8) AND email LIKE '%@example.com';

-- === CASE: BETWEEN operator ===
-- EXPECT: 5 rows
SELECT id, name, email
FROM users
WHERE id BETWEEN 2 AND 6;

-- === CASE: NOT BETWEEN ===
-- EXPECT: 5 rows
SELECT id, name, email
FROM users
WHERE id NOT BETWEEN 2 AND 6;

-- === CASE: LIKE with wildcard ===
-- EXPECT: 6 rows
SELECT id, name, email
FROM users
WHERE name LIKE 'J%';

-- === CASE: LIKE with multiple wildcards ===
-- EXPECT: 3 rows
SELECT id, name, email
FROM users
WHERE email LIKE '%@%.com';

-- === CASE: NOT LIKE ===
-- EXPECT: 4 rows
SELECT id, name, email
FROM users
WHERE name NOT LIKE 'A%';

-- === CASE: IN with list ===
-- EXPECT: 4 rows
SELECT id, name, email
FROM users
WHERE id IN (1, 3, 5, 7);

-- === CASE: NOT IN with list ===
-- EXPECT: 6 rows
SELECT id, name, email
FROM users
WHERE id NOT IN (1, 3, 5, 7);

-- === CASE: IS NULL ===
-- EXPECT: 2 rows
SELECT id, name, email
FROM users
WHERE email IS NULL;

-- === CASE: IS NOT NULL ===
-- EXPECT: 8 rows
SELECT id, name, email
FROM users
WHERE email IS NOT NULL;

-- === CASE: Complex with AND/OR/NOT ===
-- EXPECT: 5 rows
SELECT id, name, email, created_at
FROM users
WHERE (id > 5 AND email IS NOT NULL) OR (id < 3 AND name LIKE 'A%');

-- === CASE: Comparison operators ===
-- EXPECT: 6 rows
SELECT id, name, email
FROM users
WHERE id >= 4 AND id <= 9;

-- === CASE: String functions in WHERE ===
-- EXPECT: 5 rows
SELECT id, name, email
FROM users
WHERE UPPER(name) LIKE 'J%';

-- === CASE: Numeric comparison ===
-- EXPECT: 4 rows
SELECT id, name, email
FROM users
WHERE LENGTH(email) > 15;

-- === CASE: Combined conditions with parentheses ===
-- EXPECT: 3 rows
SELECT id, name, email
FROM users
WHERE (id IN (1, 2, 3) AND email LIKE '%@example.com')
   OR (id IN (7, 8, 9) AND name LIKE 'A%');
