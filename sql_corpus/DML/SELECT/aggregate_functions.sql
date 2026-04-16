-- === Aggregate Functions Test Suite ===

-- === CASE: COUNT with GROUP BY ===
-- EXPECT: 5 rows
SELECT user_id, COUNT(*) as order_count
FROM orders
GROUP BY user_id;

-- === CASE: SUM with GROUP BY ===
-- EXPECT: 5 rows
SELECT user_id, SUM(total) as total_spent
FROM orders
GROUP BY user_id;

-- === CASE: AVG with GROUP BY ===
-- EXPECT: 5 rows
SELECT user_id, AVG(total) as avg_order_value
FROM orders
GROUP BY user_id;

-- === CASE: MIN with GROUP BY ===
-- EXPECT: 5 rows
SELECT user_id, MIN(total) as smallest_order
FROM orders
GROUP BY user_id;

-- === CASE: MAX with GROUP BY ===
-- EXPECT: 5 rows
SELECT user_id, MAX(total) as largest_order
FROM orders
GROUP BY user_id;

-- === CASE: COUNT with DISTINCT ===
-- EXPECT: 1 row
SELECT COUNT(DISTINCT user_id) as unique_customers
FROM orders;

-- === CASE: SUM with DISTINCT ===
-- EXPECT: 1 row
SELECT SUM(DISTINCT total) as sum_unique_totals
FROM orders
WHERE total < 100;

-- === CASE: HAVING with COUNT ===
-- EXPECT: 3 rows
SELECT user_id, COUNT(*) as order_count
FROM orders
GROUP BY user_id
HAVING COUNT(*) > 2;

-- === CASE: HAVING with SUM ===
-- EXPECT: 2 rows
SELECT user_id, SUM(total) as total_spent
FROM orders
GROUP BY user_id
HAVING SUM(total) > 500;

-- === CASE: HAVING with AVG ===
-- EXPECT: 4 rows
SELECT user_id, AVG(total) as avg_order_value
FROM orders
GROUP BY user_id
HAVING AVG(total) > 100;

-- === CASE: Multiple aggregates ===
-- EXPECT: 5 rows
SELECT user_id, COUNT(*) as cnt, SUM(total) as sum, AVG(total) as avg, MIN(total) as min, MAX(total) as max
FROM orders
GROUP BY user_id;

-- === CASE: COUNT with JOIN ===
-- EXPECT: 6 rows
SELECT u.name, COUNT(o.order_id) as order_count
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
GROUP BY u.id, u.name;

-- === CASE: Aggregate without GROUP BY ===
-- EXPECT: 1 row
SELECT COUNT(*) as total_orders, SUM(total) as total_revenue, AVG(total) as avg_order
FROM orders;

-- === CASE: GROUP BY with ORDER BY ===
-- EXPECT: 5 rows
SELECT user_id, COUNT(*) as order_count
FROM orders
GROUP BY user_id
ORDER BY order_count DESC;

-- === CASE: HAVING with multiple conditions ===
-- EXPECT: 2 rows
SELECT user_id, COUNT(*) as cnt, SUM(total) as sum
FROM orders
GROUP BY user_id
HAVING COUNT(*) > 2 AND SUM(total) > 400;

-- === CASE: COUNT with CASE ===
-- EXPECT: 1 row
SELECT
  COUNT(*) as total,
  COUNT(CASE WHEN total > 200 THEN 1 END) as high_value,
  COUNT(CASE WHEN total <= 200 THEN 1 END) as low_value
FROM orders;

-- === CASE: Aggregate with NULL handling ===
-- EXPECT: 1 row
SELECT COUNT(*) as total, COUNT(user_id) as with_user, SUM(total) as sum_total
FROM orders;
