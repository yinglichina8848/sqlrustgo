-- === SKIP ===

-- === Outer Join Test Suite ===

-- === CASE: Left Join Basic ===
-- EXPECT: 10 rows
SELECT u.id, u.name, o.order_id, o.total
FROM users u LEFT JOIN orders o ON u.id = o.user_id;

-- === CASE: Left Join with NULL handling ===
-- EXPECT: 5 rows
SELECT u.id, u.name, o.order_id
FROM users u LEFT JOIN orders o ON u.id = o.user_id
WHERE o.order_id IS NULL;

-- === CASE: Right Join Basic ===
-- EXPECT: 15 rows
SELECT u.id, u.name, o.order_id, o.total
FROM users u RIGHT JOIN orders o ON u.id = o.user_id;

-- === CASE: Right Join with NULL handling ===
-- EXPECT: 3 rows
SELECT u.id, u.name, o.order_id
FROM users u RIGHT JOIN orders o ON u.id = o.user_id
WHERE u.id IS NULL;

-- === CASE: Full Outer Join Basic ===
-- EXPECT: 20 rows
SELECT u.id, u.name, o.order_id, o.total
FROM users u FULL OUTER JOIN orders o ON u.id = o.user_id;

-- === CASE: Full Outer Join with WHERE filter ===
-- EXPECT: 18 rows
SELECT u.id, u.name, o.order_id, o.total
FROM users u FULL OUTER JOIN orders o ON u.id = o.user_id
WHERE u.id > 5;

-- === CASE: Left Join with aggregate ===
-- EXPECT: 5 rows
SELECT u.id, u.name, COUNT(o.order_id) as order_count, SUM(o.total) as total_spent
FROM users u LEFT JOIN orders o ON u.id = o.user_id
GROUP BY u.id, u.name
ORDER BY u.id;

-- === CASE: Right Join with aggregate ===
-- EXPECT: 10 rows
SELECT u.id, u.name, COUNT(o.order_id) as order_count
FROM users u RIGHT JOIN orders o ON u.id = o.user_id
GROUP BY u.id, u.name;

-- === CASE: Full Outer Join with COALESCE ===
-- EXPECT: 20 rows
SELECT COALESCE(u.id, 0) as user_id, COALESCE(u.name, 'Unknown') as user_name, o.order_id
FROM users u FULL OUTER JOIN orders o ON u.id = o.user_id;

-- === CASE: Multi-table Left Join ===
-- EXPECT: 30 rows
SELECT u.id, u.name, o.order_id, p.product_id, p.name as product_name
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
LEFT JOIN order_items oi ON o.order_id = oi.order_id
LEFT JOIN products p ON oi.product_id = p.product_id;

-- === CASE: Left Join with subquery in ON clause ===
-- EXPECT: 10 rows
SELECT u.id, u.name, o.order_id
FROM users u
LEFT JOIN orders o ON u.id = o.user_id AND o.total > (SELECT AVG(total) FROM orders);

-- === CASE: Self Join with Left Join ===
-- EXPECT: 5 rows
SELECT e.id, e.name as employee, m.name as manager
FROM employees e
LEFT JOIN employees m ON e.manager_id = m.id;

-- === CASE: Left Join with CASE expression ===
-- EXPECT: 10 rows
SELECT u.id, u.name,
  CASE WHEN o.order_id IS NULL THEN 'No Orders' ELSE 'Has Orders' END as order_status
FROM users u LEFT JOIN orders o ON u.id = o.user_id;

-- === CASE: Right Join with DISTINCT ===
-- EXPECT: 8 rows
SELECT DISTINCT o.user_id, u.name
FROM users u RIGHT JOIN orders o ON u.id = o.user_id;

-- === CASE: Full Outer Join with UNION ===
-- EXPECT: 15 rows
SELECT u.id, u.name, 'User' as type FROM users u
UNION ALL
SELECT 0 as id, 'Anonymous' as name, 'Guest' as type FROM orders LIMIT 5;

-- === CASE: Left Join using USING clause ===
-- EXPECT: 5 rows
SELECT order_id, user_name, total
FROM users LEFT JOIN orders USING (user_id);

-- === CASE: Left Join with IN clause ===
-- EXPECT: 3 rows
SELECT u.id, u.name, o.order_id
FROM users u LEFT JOIN orders o ON u.id = o.user_id
WHERE u.id IN (1, 2, 3);

-- === CASE: Left Join with BETWEEN ===
-- EXPECT: 6 rows
SELECT u.id, u.name, o.order_id, o.total
FROM users u LEFT JOIN orders o ON u.id = o.user_id
WHERE o.total BETWEEN 100 AND 500;
