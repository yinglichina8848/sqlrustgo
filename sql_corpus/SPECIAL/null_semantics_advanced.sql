-- === SKIP ===

-- === NULL Semantics Test Suite ===

-- === CASE: IS NULL ===
-- EXPECT: 2 rows
SELECT * FROM users WHERE email IS NULL;

-- === CASE: IS NOT NULL ===
-- EXPECT: 8 rows
SELECT * FROM users WHERE email IS NOT NULL;

-- === CASE: = NULL (always false) ===
-- EXPECT: 0 rows
SELECT * FROM users WHERE email = NULL;

-- === CASE: != NULL (always false) ===
-- EXPECT: 0 rows
SELECT * FROM users WHERE email != NULL;

-- === CASE: COALESCE with NULLs ===
-- EXPECT: 10 rows
SELECT id, name, COALESCE(email, 'unknown@example.com') as email FROM users;

-- === CASE: IFNULL with NULL ===
-- EXPECT: 10 rows
SELECT id, name, IFNULL(email, 'no_email') as email FROM users;

-- === CASE: NULLIF with NULL ===
-- EXPECT: 1 row
SELECT NULLIF(NULL, 'value') as result;

-- === CASE: NULLIF equal values ===
-- EXPECT: 1 row
SELECT NULLIF('same', 'same') as result;

-- === CASE: NULLIF different values ===
-- EXPECT: 1 row
SELECT NULLIF('first', 'second') as result;

-- === CASE: NULL in aggregate (COUNT with NULL) ===
-- EXPECT: 1 row
SELECT COUNT(*) as total, COUNT(email) as with_email, COUNT(*) - COUNT(email) as without_email FROM users;

-- === CASE: SUM with NULL values ===
-- EXPECT: 1 row
SELECT SUM(total) as total_sum FROM orders;

-- === CASE: AVG with NULL values ===
-- EXPECT: 1 row
SELECT AVG(total) as avg_order FROM orders;

-- === CASE: NULL in CASE expression ===
-- EXPECT: 10 rows
SELECT id, name, CASE WHEN email IS NULL THEN 'No Email' ELSE email END as contact FROM users;

-- === CASE: NULL with IN ===
-- EXPECT: 0 rows
SELECT * FROM users WHERE NULL IN (SELECT email FROM users);

-- === CASE: NULL with NOT IN ===
-- EXPECT: 0 rows
SELECT * FROM users WHERE NULL NOT IN (SELECT email FROM users);

-- === CASE: NULL with BETWEEN ===
-- EXPECT: 0 rows
SELECT * FROM users WHERE email BETWEEN NULL AND 'z';

-- === CASE: NULL with LIKE ===
-- EXPECT: 0 rows
SELECT * FROM users WHERE email LIKE NULL;

-- === CASE: NULL with ORDER BY ===
-- EXPECT: 10 rows
SELECT * FROM users ORDER BY email;

-- === CASE: NULL handling in JOIN ===
-- EXPECT: 5 rows
SELECT a.id, b.id FROM users a LEFT JOIN users b ON a.id = b.id WHERE a.id <= 5;

-- === CASE: NULL in subquery ===
-- EXPECT: 2 rows
SELECT * FROM users WHERE email IN (SELECT email FROM users WHERE email IS NULL);
