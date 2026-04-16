-- === Complex DELETE Test Suite ===

-- === CASE: DELETE with subquery ===
-- EXPECT: 3 rows affected
DELETE FROM users WHERE id IN (SELECT id FROM users WHERE id > 500);

-- === CASE: DELETE with JOIN ===
-- EXPECT: 2 rows affected
DELETE FROM users u
USING (SELECT id FROM users WHERE id BETWEEN 300 AND 301) AS t
WHERE u.id = t.id;

-- === CASE: DELETE with LIMIT ===
-- EXPECT: 2 rows affected
DELETE FROM users WHERE id >= 400 LIMIT 2;

-- === CASE: DELETE with ORDER BY ===
-- EXPECT: 3 rows affected
DELETE FROM users WHERE id >= 350 ORDER BY id DESC LIMIT 3;

-- === CASE: DELETE with ORDER BY and LIMIT ===
-- EXPECT: 2 rows affected
DELETE FROM users WHERE id >= 320 ORDER BY id ASC LIMIT 2;

-- === CASE: DELETE with WHERE EXISTS ===
-- EXPECT: 2 rows affected
DELETE FROM users u
WHERE EXISTS (SELECT 1 FROM orders o WHERE o.user_id = u.id AND o.total < 50);

-- === CASE: DELETE with WHERE NOT EXISTS ===
-- EXPECT: 3 rows affected
DELETE FROM users u
WHERE NOT EXISTS (SELECT 1 FROM orders o WHERE o.user_id = u.id);

-- === CASE: DELETE with IN and subquery ===
-- EXPECT: 4 rows affected
DELETE FROM users WHERE id IN (SELECT user_id FROM orders WHERE total < 100);

-- === CASE: DELETE with multiple conditions ===
-- EXPECT: 2 rows affected
DELETE FROM users WHERE id >= 250 AND id <= 251 AND email LIKE '%test%';

-- === CASE: DELETE with BETWEEN ===
-- EXPECT: 3 rows affected
DELETE FROM users WHERE id BETWEEN 230 AND 232;

-- === CASE: DELETE with LIKE ===
-- EXPECT: 2 rows affected
DELETE FROM users WHERE email LIKE '%delete_test%';

-- === CASE: DELETE with NULL condition ===
-- EXPECT: 1 row affected
DELETE FROM users WHERE email IS NULL AND id >= 600;

-- === CASE: DELETE with CASE in WHERE ===
-- EXPECT: 2 rows affected
DELETE FROM users WHERE CASE WHEN id > 550 THEN 1 ELSE 0 END = 1;

-- === CASE: DELETE with correlated subquery ===
-- EXPECT: 3 rows affected
DELETE FROM users u
WHERE (SELECT COUNT(*) FROM orders o WHERE o.user_id = u.id) = 0 AND u.id > 500;

-- === CASE: DELETE with CTID (if supported) ===
-- EXPECT: 1 row affected
DELETE FROM users WHERE id IN (SELECT id FROM users WHERE id >= 580 LIMIT 1);
