-- === Logical Operators Test Suite ===

-- === CASE: AND operator ===
-- EXPECT: 3 rows
SELECT id, name, email FROM users WHERE id > 5 AND email LIKE '%@example.com';

-- === CASE: OR operator ===
-- EXPECT: 7 rows
SELECT id, name, email FROM users WHERE id < 3 OR name LIKE 'A%';

-- === CASE: NOT operator ===
-- EXPECT: 7 rows
SELECT id, name, email FROM users WHERE NOT id > 8;

-- === CASE: AND with OR precedence ===
-- EXPECT: 5 rows
SELECT id, name, email FROM users WHERE (id > 5 AND email LIKE '%@example.com') OR id < 3;

-- === CASE: NOT with AND ===
-- EXPECT: 4 rows
SELECT id, name, email FROM users WHERE NOT (id > 5 AND email LIKE '%@example.com');

-- === CASE: Multiple AND ===
-- EXPECT: 2 rows
SELECT id, name, email FROM users WHERE id > 3 AND name LIKE 'A%' AND email IS NOT NULL;

-- === CASE: Multiple OR ===
-- EXPECT: 6 rows
SELECT id, name, email FROM users WHERE id = 1 OR id = 3 OR id = 5 OR id = 7 OR id = 9;

-- === CASE: NOT with OR ===
-- EXPECT: 5 rows
SELECT id, name, email FROM users WHERE NOT (id = 1 OR id = 3 OR id = 5);

-- === CASE: Complex AND/OR/NOT ===
-- EXPECT: 4 rows
SELECT id, name, email FROM users WHERE (id > 3 AND name LIKE 'A%') OR NOT (id < 8 AND email LIKE '%@test%');

-- === CASE: IS NULL with AND ===
-- EXPECT: 2 rows
SELECT id, name, email FROM users WHERE email IS NULL AND name LIKE 'A%';

-- === CASE: IN with OR ===
-- EXPECT: 4 rows
SELECT id, name, email FROM users WHERE id IN (1, 3, 5, 7);

-- === CASE: BETWEEN with NOT ===
-- EXPECT: 5 rows
SELECT id, name, email FROM users WHERE id NOT BETWEEN 3 AND 7;

-- === CASE: LIKE with NOT ===
-- EXPECT: 5 rows
SELECT id, name, email FROM users WHERE name NOT LIKE 'A%';

-- === CASE: EXISTS with NOT ===
-- EXPECT: 3 rows
SELECT id, name FROM users u WHERE NOT EXISTS (SELECT 1 FROM orders o WHERE o.user_id = u.id AND o.total > 200);

-- === CASE: Comparison with logical operators ===
-- EXPECT: 5 rows
SELECT id, name, email FROM users WHERE id >= 3 AND id <= 7 AND name IS NOT NULL;
