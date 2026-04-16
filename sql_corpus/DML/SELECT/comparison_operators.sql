-- === SKIP ===

-- === Comparison Operators Test Suite ===

-- === CASE: Equal to ===
-- EXPECT: 1 row
SELECT id, name, email FROM users WHERE id = 5;

-- === CASE: Not equal to != ===
-- EXPECT: 9 rows
SELECT id, name, email FROM users WHERE id != 5;

-- === CASE: Not equal to <> ===
-- EXPECT: 9 rows
SELECT id, name, email FROM users WHERE id <> 5;

-- === CASE: Greater than ===
-- EXPECT: 5 rows
SELECT id, name, email FROM users WHERE id > 5;

-- === CASE: Greater than or equal ===
-- EXPECT: 6 rows
SELECT id, name, email FROM users WHERE id >= 5;

-- === CASE: Less than ===
-- EXPECT: 5 rows
SELECT id, name, email FROM users WHERE id < 5;

-- === CASE: Less than or equal ===
-- EXPECT: 6 rows
SELECT id, name, email FROM users WHERE id <= 5;

-- === CASE: BETWEEN ===
-- EXPECT: 5 rows
SELECT id, name, email FROM users WHERE id BETWEEN 3 AND 7;

-- === CASE: NOT BETWEEN ===
-- EXPECT: 5 rows
SELECT id, name, email FROM users WHERE id NOT BETWEEN 3 AND 7;

-- === CASE: IS NULL ===
-- EXPECT: 2 rows
SELECT id, name, email FROM users WHERE email IS NULL;

-- === CASE: IS NOT NULL ===
-- EXPECT: 8 rows
SELECT id, name, email FROM users WHERE email IS NOT NULL;

-- === CASE: LIKE pattern match ===
-- EXPECT: 3 rows
SELECT id, name, email FROM users WHERE name LIKE 'A%';

-- === CASE: NOT LIKE ===
-- EXPECT: 7 rows
SELECT id, name, email FROM users WHERE name NOT LIKE 'A%';

-- === CASE: IN list ===
-- EXPECT: 3 rows
SELECT id, name, email FROM users WHERE id IN (1, 3, 5);

-- === CASE: NOT IN list ===
-- EXPECT: 7 rows
SELECT id, name, email FROM users WHERE id NOT IN (1, 3, 5);

-- === CASE: EXISTS ===
-- EXPECT: 5 rows
SELECT id, name FROM users u WHERE EXISTS (SELECT 1 FROM orders o WHERE o.user_id = u.id);

-- === CASE: NOT EXISTS ===
-- EXPECT: 5 rows
SELECT id, name FROM users u WHERE NOT EXISTS (SELECT 1 FROM orders o WHERE o.user_id = u.id);

-- === CASE: String comparison ===
-- EXPECT: 2 rows
SELECT id, name, email FROM users WHERE name > 'John';

-- === CASE: Multiple comparisons ===
-- EXPECT: 2 rows
SELECT id, name, email FROM users WHERE id > 3 AND id < 8;

-- === CASE: Chained comparisons ===
-- EXPECT: 4 rows
SELECT id, name, email FROM users WHERE id < 3 OR id = 5 OR id > 8;
