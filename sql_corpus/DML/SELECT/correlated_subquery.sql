-- === SKIP ===

-- === Correlated Subquery Test Suite ===

-- === CASE: Correlated in SELECT ===
-- EXPECT: 10 rows
SELECT id, name,
  (SELECT COUNT(*) FROM orders WHERE user_id = users.id) as order_count
FROM users;

-- === CASE: Correlated in WHERE EXISTS ===
-- EXPECT: 5 rows
SELECT id, name FROM users u
WHERE EXISTS (SELECT 1 FROM orders o WHERE o.user_id = u.id AND o.total > 150);

-- === CASE: Correlated in WHERE IN ===
-- EXPECT: 4 rows
SELECT id, name FROM users u
WHERE id IN (SELECT user_id FROM orders WHERE total > (SELECT AVG(total) FROM orders));

-- === CASE: Correlated with aggregate in HAVING ===
-- EXPECT: 3 rows
SELECT user_id, SUM(total) as total_spent
FROM orders
GROUP BY user_id
HAVING SUM(total) > (SELECT AVG(total_spent) FROM (SELECT user_id, SUM(total) as total_spent FROM orders GROUP BY user_id));

-- === CASE: Correlated UPDATE ===
-- EXPECT: 5 rows affected
UPDATE users u
SET email = 'correlated_' || u.email
WHERE EXISTS (SELECT 1 FROM orders WHERE user_id = u.id AND total > 100);

-- === CASE: Correlated DELETE ===
-- EXPECT: 2 rows affected
DELETE FROM users u
WHERE EXISTS (SELECT 1 FROM orders WHERE user_id = u.id AND total < 50);

-- === CASE: Multiple correlated subqueries ===
-- EXPECT: 8 rows
SELECT id, name FROM users u
WHERE EXISTS (SELECT 1 FROM orders WHERE user_id = u.id)
  AND EXISTS (SELECT 1 FROM orders WHERE user_id = u.id AND total > 100);

-- === CASE: Correlated with NOT EXISTS ===
-- EXPECT: 5 rows
SELECT id, name FROM users u
WHERE NOT EXISTS (SELECT 1 FROM orders WHERE user_id = u.id AND total > 500);

-- === CASE: Correlated with JOIN ===
-- EXPECT: 6 rows
SELECT u.id, u.name, o.total
FROM users u
JOIN orders o ON EXISTS (SELECT 1 FROM orders WHERE user_id = u.id AND total > 100);

-- === CASE: Correlated with DISTINCT ===
-- EXPECT: 5 rows
SELECT DISTINCT user_id FROM orders o1
WHERE (SELECT COUNT(*) FROM orders o2 WHERE o2.user_id = o1.user_id) > 1;

-- === CASE: Correlated scalar with multiple columns ===
-- EXPECT: 10 rows
SELECT id, name,
  (SELECT MIN(total) FROM orders WHERE user_id = users.id) as min_order,
  (SELECT MAX(total) FROM orders WHERE user_id = users.id) as max_order
FROM users;

-- === CASE: Correlated in CASE ===
-- EXPECT: 10 rows
SELECT id, name,
  CASE WHEN (SELECT COUNT(*) FROM orders WHERE user_id = users.id) > 2
    THEN 'High' ELSE 'Low' END as activity_level
FROM users;

-- === CASE: Nested correlated subquery ===
-- EXPECT: 5 rows
SELECT id, name FROM users u
WHERE EXISTS (
  SELECT 1 FROM orders o
  WHERE o.user_id = u.id
    AND EXISTS (SELECT 1 FROM order_items WHERE order_id = o.order_id)
);

-- === CASE: Correlated with ORDER BY subquery ===
-- EXPECT: 3 rows
SELECT id, name FROM users u
WHERE id IN (
  SELECT user_id FROM orders
  WHERE total > (SELECT MAX(total) / 2 FROM orders)
  ORDER BY total DESC LIMIT 3
);

-- === CASE: Correlated with ALL/ANY ===
-- EXPECT: 4 rows
SELECT id, name FROM users u
WHERE total > ALL (SELECT total FROM orders WHERE user_id = u.id AND total < 100);
