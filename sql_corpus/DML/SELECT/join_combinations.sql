-- === SKIP ===

-- === Cross Join ===
-- EXPECT: 50 rows (5 users x 10 products)
SELECT u.id as user_id, p.id as product_id, u.name, p.name as product_name
FROM users u CROSS JOIN products p;

-- === Natural Join ===
-- EXPECT: 5 rows
SELECT * FROM orders NATURAL JOIN order_items;

-- === Join with aggregate ===
-- EXPECT: 6 rows
SELECT u.name, COUNT(o.order_id) as order_count, SUM(o.total) as total_spent
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
GROUP BY u.id, u.name;

-- === Join with multiple conditions ===
-- EXPECT: 3 rows
SELECT u.name, o.order_id, o.total
FROM users u
JOIN orders o ON u.id = o.user_id AND o.total > 150 AND o.order_date > '2024-01-01';

-- === Join with GROUP BY and HAVING ===
-- EXPECT: 3 rows
SELECT u.name, COUNT(*) as order_count
FROM users u
JOIN orders o ON u.id = o.user_id
GROUP BY u.id, u.name
HAVING COUNT(*) > 2;

-- === Join with ORDER BY ===
-- EXPECT: 15 rows
SELECT u.id, u.name, o.order_id, o.total
FROM users u
JOIN orders o ON u.id = o.user_id
ORDER BY u.id, o.total DESC;

-- === Join with LIMIT ===
-- EXPECT: 5 rows
SELECT u.id, u.name, o.order_id, o.total
FROM users u
JOIN orders o ON u.id = o.user_id
ORDER BY o.total DESC
LIMIT 5;

-- === Join with DISTINCT ===
-- EXPECT: 4 rows
SELECT DISTINCT u.id, u.name
FROM users u
JOIN orders o ON u.id = o.user_id
WHERE o.total > 100;

-- === Join with subquery in SELECT ===
-- EXPECT: 5 rows
SELECT u.id, u.name,
  (SELECT COUNT(*) FROM orders WHERE user_id = u.id) as order_count
FROM users u;

-- === Join with subquery in FROM ===
-- EXPECT: 5 rows
SELECT t.order_count, t.total_spent, u.name
FROM users u
JOIN (
  SELECT user_id, COUNT(*) as order_count, SUM(total) as total_spent
  FROM orders GROUP BY user_id
) t ON u.id = t.user_id;

-- === Join with UNION ===
-- EXPECT: 8 rows
SELECT u.id, u.name, o.order_id FROM users u
JOIN orders o ON u.id = o.user_id
UNION
SELECT u.id, u.name, NULL as order_id FROM users u WHERE id > 8;

-- === Join with COALESCE ===
-- EXPECT: 10 rows
SELECT u.id, u.name, COALESCE(o.order_id, 0) as order_id
FROM users u
LEFT JOIN orders o ON u.id = o.user_id;

-- === Join with CASE in SELECT ===
-- EXPECT: 6 rows
SELECT u.id, u.name,
  CASE WHEN o.order_id IS NULL THEN 'No Orders' ELSE 'Has Orders' END as status
FROM users u
LEFT JOIN orders o ON u.id = o.user_id;

-- === Join with IN clause ===
-- EXPECT: 4 rows
SELECT u.id, u.name, o.order_id
FROM users u
JOIN orders o ON u.id = o.user_id
WHERE o.total IN (100, 200, 300);

-- === Join with BETWEEN ===
-- EXPECT: 5 rows
SELECT u.id, u.name, o.order_id, o.total
FROM users u
JOIN orders o ON u.id = o.user_id
WHERE o.total BETWEEN 100 AND 300;

-- === Join with LIKE ===
-- EXPECT: 3 rows
SELECT u.id, u.name, o.order_id
FROM users u
JOIN orders o ON u.id = o.user_id
WHERE u.email LIKE '%@example.com';

-- === Join with NULL comparison ===
-- EXPECT: 2 rows
SELECT u.id, u.name, o.order_id
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
WHERE o.order_id IS NULL;

-- === Three table join ===
-- EXPECT: 12 rows
SELECT u.name, o.order_id, p.name as product_name
FROM users u
JOIN orders o ON u.id = o.user_id
JOIN order_items oi ON o.order_id = oi.order_id
JOIN products p ON oi.product_id = p.id;

-- === Self join ===
-- EXPECT: 5 rows
SELECT e.name as employee, m.name as manager
FROM employees e
LEFT JOIN employees m ON e.manager_id = m.id;
