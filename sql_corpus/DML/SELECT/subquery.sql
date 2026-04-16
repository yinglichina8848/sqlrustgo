-- === SKIP ===

-- === Subquery Test Suite ===

-- === CASE: Scalar Subquery in SELECT ===
-- EXPECT: 10 rows
SELECT id, name, (SELECT COUNT(*) FROM orders WHERE user_id = users.id) as order_count
FROM users;

-- === CASE: Scalar Subquery in WHERE ===
-- EXPECT: 5 rows
SELECT id, name, email
FROM users
WHERE id = (SELECT MIN(user_id) FROM orders WHERE total > 100);

-- === CASE: EXISTS with correlated subquery ===
-- EXPECT: 7 rows
SELECT id, name
FROM users u
WHERE EXISTS (SELECT 1 FROM orders o WHERE o.user_id = u.id AND o.total > 200);

-- === CASE: NOT EXISTS ===
-- EXPECT: 3 rows
SELECT id, name
FROM users u
WHERE NOT EXISTS (SELECT 1 FROM orders o WHERE o.user_id = u.id AND o.total > 500);

-- === CASE: IN with subquery ===
-- EXPECT: 8 rows
SELECT id, name, email
FROM users
WHERE id IN (SELECT user_id FROM orders WHERE total > 150);

-- === CASE: NOT IN with subquery ===
-- EXPECT: 2 rows
SELECT id, name, email
FROM users
WHERE id NOT IN (SELECT user_id FROM orders WHERE total > 300);

-- === CASE: Subquery with aggregate in HAVING ===
-- EXPECT: 4 rows
SELECT user_id, COUNT(*) as order_count, SUM(total) as total_spent
FROM orders
GROUP BY user_id
HAVING COUNT(*) > (SELECT AVG(order_count) FROM (SELECT user_id, COUNT(*) as order_count FROM orders GROUP BY user_id) t);

-- === CASE: Nested subquery ===
-- EXPECT: 3 rows
SELECT id, name
FROM users
WHERE id IN (
  SELECT user_id FROM orders
  WHERE total > (SELECT AVG(total) FROM orders)
);

-- === CASE: Subquery with JOIN ===
-- EXPECT: 12 rows
SELECT o.id, o.order_date, o.total
FROM orders o
WHERE o.user_id IN (
  SELECT id FROM users
  WHERE email LIKE '%@example.com'
);

-- === CASE: ALL with subquery ===
-- EXPECT: 2 rows
SELECT id, name, email
FROM users
WHERE id = ALL (SELECT user_id FROM orders WHERE total < 50);

-- === CASE: ANY with subquery ===
-- EXPECT: 6 rows
SELECT id, name, email
FROM users
WHERE id = ANY (SELECT user_id FROM orders WHERE total > 200);

-- === CASE: Subquery in FROM clause ===
-- EXPECT: 5 rows
SELECT t.order_count, t.avg_total, u.name
FROM (SELECT user_id, COUNT(*) as order_count, AVG(total) as avg_total FROM orders GROUP BY user_id) t
JOIN users u ON t.user_id = u.id;

-- === CASE: Correlated subquery with UPDATE ===
-- EXPECT: 10 rows affected
UPDATE users
SET email = (SELECT MAX(email) FROM users WHERE id != users.id)
WHERE id < 10;

-- === CASE: Subquery with DISTINCT ===
-- EXPECT: 4 rows
SELECT DISTINCT user_id
FROM orders
WHERE user_id IN (SELECT id FROM users WHERE id < 6);

-- === CASE: Subquery with ORDER BY ===
-- EXPECT: 5 rows
SELECT id, name
FROM users
WHERE id IN (SELECT user_id FROM orders ORDER BY total DESC LIMIT 5);

-- === CASE: Multiple subqueries in WHERE ===
-- EXPECT: 3 rows
SELECT id, name, email
FROM users
WHERE id IN (SELECT user_id FROM orders WHERE total > 100)
  AND id NOT IN (SELECT user_id FROM orders WHERE total < 50);
