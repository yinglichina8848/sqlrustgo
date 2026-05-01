-- === SKIP ===

-- === Common Aggregate Patterns Test Suite ===

-- === CASE: COUNT with GROUP BY ===
-- EXPECT: 5 rows
SELECT user_id, COUNT(*) as order_count FROM orders GROUP BY user_id;

-- === CASE: SUM with GROUP BY ===
-- EXPECT: 5 rows
SELECT user_id, SUM(total) as total_spent FROM orders GROUP BY user_id;

-- === CASE: AVG with GROUP BY ===
-- EXPECT: 5 rows
SELECT user_id, AVG(total) as avg_order FROM orders GROUP BY user_id;

-- === CASE: MIN with GROUP BY ===
-- EXPECT: 5 rows
SELECT user_id, MIN(total) as smallest_order FROM orders GROUP BY user_id;

-- === CASE: MAX with GROUP BY ===
-- EXPECT: 5 rows
SELECT user_id, MAX(total) as largest_order FROM orders GROUP BY user_id;

-- === CASE: COUNT DISTINCT ===
-- EXPECT: 1 row
SELECT COUNT(DISTINCT user_id) as unique_users FROM orders;

-- === CASE: SUM DISTINCT ===
-- EXPECT: 1 row
SELECT SUM(DISTINCT total) as sum_unique_totals FROM orders WHERE total < 500;

-- === CASE: AVG with ROUND ===
-- EXPECT: 5 rows
SELECT user_id, ROUND(AVG(total), 2) as avg_order FROM orders GROUP BY user_id;

-- === CASE: Multiple aggregates ===
-- EXPECT: 5 rows
SELECT user_id, COUNT(*), SUM(total), AVG(total), MIN(total), MAX(total) FROM orders GROUP BY user_id;

-- === CASE: GROUP BY with HAVING COUNT ===
-- EXPECT: 3 rows
SELECT user_id, COUNT(*) as cnt FROM orders GROUP BY user_id HAVING COUNT(*) > 2;

-- === CASE: GROUP BY with HAVING SUM ===
-- EXPECT: 2 rows
SELECT user_id, SUM(total) as total FROM orders GROUP BY user_id HAVING SUM(total) > 500;

-- === CASE: GROUP BY with HAVING AVG ===
-- EXPECT: 4 rows
SELECT user_id, AVG(total) as avg FROM orders GROUP BY user_id HAVING AVG(total) > 100;

-- === CASE: GROUP BY with ORDER BY ===
-- EXPECT: 5 rows
SELECT user_id, COUNT(*) as cnt FROM orders GROUP BY user_id ORDER BY cnt DESC;

-- === CASE: GROUP BY with ORDER BY and LIMIT ===
-- EXPECT: 3 rows
SELECT user_id, SUM(total) as total FROM orders GROUP BY user_id ORDER BY total DESC LIMIT 3;

-- === CASE: GROUP BY with WHERE ===
-- EXPECT: 4 rows
SELECT user_id, COUNT(*) as cnt FROM orders WHERE total > 100 GROUP BY user_id;

-- === CASE: GROUP BY with WHERE and HAVING ===
-- EXPECT: 2 rows
SELECT user_id, COUNT(*) as cnt FROM orders WHERE total > 100 GROUP BY user_id HAVING COUNT(*) > 1;

-- === CASE: GROUP BY multiple columns ===
-- EXPECT: 10 rows
SELECT user_id, status, COUNT(*) as cnt FROM orders GROUP BY user_id, status;

-- === CASE: COUNT with CASE ===
-- EXPECT: 1 row
SELECT COUNT(CASE WHEN total > 200 THEN 1 END) as high_value_orders, COUNT(CASE WHEN total <= 200 THEN 1 END) as low_value_orders FROM orders;

-- === CASE: Aggregate without GROUP BY ===
-- EXPECT: 1 row
SELECT COUNT(*) as total_orders, SUM(total) as total_revenue, AVG(total) as avg_order FROM orders;
