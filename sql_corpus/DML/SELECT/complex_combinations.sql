-- === Query Combinations Test Suite ===

-- === CASE: Complex SELECT with multiple features ===
-- EXPECT: 3 rows
SELECT u.id, u.name, COUNT(o.order_id) AS order_count, SUM(o.total) AS total_spent
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
WHERE u.id <= 5 AND o.total > 100
GROUP BY u.id, u.name
HAVING COUNT(*) > 1
ORDER BY total_spent DESC
LIMIT 5;

-- === CASE: Nested subquery with JOIN ===
-- EXPECT: 3 rows
SELECT *
FROM (
  SELECT u.id, u.name, o.total
  FROM users u
  JOIN orders o ON u.id = o.user_id
  WHERE u.id <= 5
) sub
WHERE sub.total > 150;

-- === CASE: CTE with multiple features ===
-- EXPECT: 3 rows
WITH user_orders AS (
  SELECT user_id, COUNT(*) AS cnt, SUM(total) AS total
  FROM orders
  GROUP BY user_id
  HAVING SUM(total) > 200
)
SELECT u.name, uo.cnt, uo.total
FROM users u
JOIN user_orders uo ON u.id = uo.user_id
WHERE u.id <= 5;

-- === CASE: UNION with complex SELECTs ===
-- EXPECT: 10 rows
SELECT id, name, 'active' AS status FROM users WHERE id <= 5
UNION ALL
SELECT id, name, 'inactive' AS status FROM users WHERE id > 5 AND id <= 10;

-- === CASE: JOIN with subquery in SELECT ===
-- EXPECT: 5 rows
SELECT u.id, u.name,
  (SELECT COUNT(*) FROM orders WHERE user_id = u.id) AS order_count,
  (SELECT SUM(total) FROM orders WHERE user_id = u.id) AS total_spent
FROM users u
WHERE u.id <= 5;

-- === CASE: Multiple JOINs with conditions ===
-- EXPECT: 3 rows
SELECT u.id, u.name, o.order_id, o.total, p.name AS product_name
FROM users u
JOIN orders o ON u.id = o.user_id AND o.total > 100
JOIN order_items oi ON o.order_id = oi.order_id
JOIN products p ON oi.product_id = p.id
WHERE u.id <= 3;

-- === CASE: CASE with aggregate in GROUP BY ===
-- EXPECT: 4 rows
SELECT
  CASE WHEN id <= 3 THEN 'low' WHEN id <= 7 THEN 'medium' ELSE 'high' END AS category,
  COUNT(*) AS cnt,
  SUM(total) AS total
FROM orders
GROUP BY CASE WHEN id <= 3 THEN 'low' WHEN id <= 7 THEN 'medium' ELSE 'high' END;

-- === CASE: DISTINCT with aggregate ===
-- EXPECT: 5 rows
SELECT DISTINCT user_id, COUNT(*) AS order_count, SUM(total) AS total
FROM orders
WHERE user_id <= 5
GROUP BY user_id;

-- === CASE: ORDER BY with NULLS FIRST/LAST ===
-- EXPECT: 10 rows
SELECT * FROM users ORDER BY email NULLS FIRST;

-- === CASE: Complex HAVING with subquery ===
-- EXPECT: 2 rows
SELECT user_id, COUNT(*) AS cnt, SUM(total) AS total
FROM orders
GROUP BY user_id
HAVING SUM(total) > (SELECT AVG(total) FROM orders) AND COUNT(*) > 2;
