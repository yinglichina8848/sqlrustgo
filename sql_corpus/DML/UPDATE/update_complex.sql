-- === Complex UPDATE Test Suite ===

-- === CASE: UPDATE with subquery ===
-- EXPECT: 3 rows affected
UPDATE users SET email = 'subquery_' || email WHERE id IN (SELECT id FROM users WHERE id > 200);

-- === CASE: UPDATE with JOIN ===
-- EXPECT: 2 rows affected
UPDATE users u
SET u.email = 'joined_' || u.email
FROM (SELECT id FROM users WHERE id BETWEEN 150 AND 151) AS t
WHERE u.id = t.id;

-- === CASE: UPDATE with multiple columns ===
-- EXPECT: 5 rows affected
UPDATE users SET email = 'multi_' || email, name = 'updated_' || name WHERE id BETWEEN 100 AND 104;

-- === CASE: UPDATE with CASE ===
-- EXPECT: 10 rows affected
UPDATE users SET email = CASE
  WHEN id <= 3 THEN 'low_' || email
  WHEN id <= 7 THEN 'mid_' || email
  ELSE 'high_' || email
END WHERE id <= 10;

-- === CASE: UPDATE with ORDER BY ===
-- EXPECT: 3 rows affected
UPDATE users SET email = 'ordered_' || email WHERE id >= 80 ORDER BY id DESC LIMIT 3;

-- === CASE: UPDATE with WHERE EXISTS ===
-- EXPECT: 4 rows affected
UPDATE users SET email = 'exists_' || email
WHERE EXISTS (SELECT 1 FROM orders o WHERE o.user_id = users.id AND o.total > 150);

-- === CASE: UPDATE with WHERE NOT EXISTS ===
-- EXPECT: 2 rows affected
UPDATE users SET email = 'not_exists_' || email
WHERE NOT EXISTS (SELECT 1 FROM orders o WHERE o.user_id = users.id) AND id > 500;

-- === CASE: UPDATE with IN and subquery ===
-- EXPECT: 5 rows affected
UPDATE users SET email = 'in_subquery_' || email
WHERE id IN (SELECT user_id FROM orders WHERE total > 100);

-- === CASE: UPDATE with multiple conditions ===
-- EXPECT: 2 rows affected
UPDATE users SET email = 'multi_cond_' || email
WHERE id > 50 AND id < 53 AND email LIKE '%@example.com';

-- === CASE: UPDATE with BETWEEN ===
-- EXPECT: 3 rows affected
UPDATE users SET email = 'between_' || email WHERE id BETWEEN 40 AND 42;

-- === CASE: UPDATE with LIKE ===
-- EXPECT: 2 rows affected
UPDATE users SET email = 'like_' || email WHERE email LIKE '%test%';

-- === CASE: UPDATE with NULL ===
-- EXPECT: 1 row affected
UPDATE users SET email = NULL WHERE id = 30;

-- === CASE: UPDATE with correlated subquery ===
-- EXPECT: 3 rows affected
UPDATE users SET email = (SELECT MAX(email) FROM users WHERE id != users.id)
WHERE id BETWEEN 20 AND 22;

-- === CASE: UPDATE with arithmetic ===
-- EXPECT: 5 rows affected
UPDATE orders SET total = total * 1.1 WHERE user_id <= 3 AND total < 200;

-- === CASE: UPDATE with string function ===
-- EXPECT: 5 rows affected
UPDATE users SET name = UPPER(name) WHERE id <= 15;

-- === CASE: UPDATE with coalesce ===
-- EXPECT: 3 rows affected
UPDATE users SET email = COALESCE(email, 'unknown@example.com') WHERE email IS NULL AND id > 600;
