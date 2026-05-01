-- === SKIP ===

-- === Advanced SQL Patterns Test Suite ===

-- === CASE: Pivot/Matrix transformation ===
-- EXPECT: 5 rows
SELECT
  user_id,
  SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) as pending,
  SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END) as completed,
  SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END) as cancelled
FROM orders
GROUP BY user_id;

-- === CASE: Unpivot/Vertical table ===
-- EXPECT: 15 rows
SELECT user_id, 'pending' as status_type, COUNT(*) as cnt FROM orders WHERE status = 'pending' GROUP BY user_id
UNION ALL
SELECT user_id, 'completed', COUNT(*) FROM orders WHERE status = 'completed' GROUP BY user_id
UNION ALL
SELECT user_id, 'cancelled', COUNT(*) FROM orders WHERE status = 'cancelled' GROUP BY user_id;

-- === CASE: Running total ===
-- EXPECT: 10 rows
SELECT id, total,
  (SELECT SUM(total) FROM orders o2 WHERE o2.id <= o1.id) as running_total
FROM orders o1
ORDER BY id
LIMIT 10;

-- === CASE: Moving average ===
-- EXPECT: 5 rows
SELECT id, total,
  (SELECT AVG(total) FROM orders o2 WHERE o2.id BETWEEN o1.id - 2 AND o1.id) as moving_avg
FROM orders o1
WHERE o1.id <= 5;

-- === CASE: Rank within groups ===
-- EXPECT: 10 rows
SELECT user_id, total,
  RANK() OVER (PARTITION BY user_id ORDER BY total DESC) as rank_within_user
FROM orders;

-- === CASE: Percent of total ===
-- EXPECT: 5 rows
SELECT id, total,
  ROUND(total * 100.0 / (SELECT SUM(total) FROM orders), 2) as pct_of_total
FROM orders
WHERE id <= 5;

-- === CASE: Difference between rows ===
-- EXPECT: 9 rows
SELECT id, total,
  total - LAG(total) OVER (ORDER BY id) as diff_from_prev
FROM orders
ORDER BY id;

-- === CASE: Cumulative distribution ===
-- EXPECT: 10 rows
SELECT id, total,
  ROUND(CUME_DIST() OVER (ORDER BY total), 2) as cumdist
FROM orders;

-- === CASE: First/last in group ===
-- EXPECT: 5 rows
SELECT user_id,
  (SELECT total FROM orders o2 WHERE o2.user_id = o1.user_id ORDER BY order_date LIMIT 1) as first_order,
  (SELECT total FROM orders o2 WHERE o2.user_id = o1.user_id ORDER BY order_date DESC LIMIT 1) as last_order
FROM orders o1
GROUP BY user_id;

-- === CASE: Conditional aggregation with GROUP BY ===
-- EXPECT: 3 rows
SELECT user_id,
  COUNT(*) as total_orders,
  SUM(CASE WHEN total > 150 THEN 1 ELSE 0 END) as high_value,
  AVG(CASE WHEN total > 150 THEN total END) as avg_high_value
FROM orders
GROUP BY user_id
HAVING COUNT(*) > 2;

-- === CASE: Hierarchical data traversal ===
-- EXPECT: 5 rows
SELECT e.id, e.name, m.name as manager_name, 1 as level
FROM employees e
LEFT JOIN employees m ON e.manager_id = m.id
WHERE e.id <= 5
UNION ALL
SELECT e.id, e.name, 'CEO' as manager_name, 0 as level
FROM employees e
WHERE manager_id IS NULL;

-- === CASE: Relational division ===
-- EXPECT: 2 rows
SELECT DISTINCT user_id FROM orders o1
WHERE NOT EXISTS (
  SELECT 1 FROM (SELECT DISTINCT product_id FROM order_items) p
  WHERE NOT EXISTS (
    SELECT 1 FROM order_items o2
    WHERE o2.user_id = o1.user_id AND o2.product_id = p.product_id
  )
);

-- === CASE: Top N per group ===
-- EXPECT: 6 rows
SELECT * FROM (
  SELECT user_id, total, ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY total DESC) as rn
  FROM orders
) WHERE rn <= 2;

-- === CASE: Running count of occurrences ===
-- EXPECT: 10 rows
SELECT id, name,
  (SELECT COUNT(*) FROM users u2 WHERE u2.name <= u1.name) as name_rank
FROM users u1
ORDER BY name;

-- === CASE: Histogram using width buckets ===
-- EXPECT: 5 rows
SELECT
  CASE
    WHEN total < 100 THEN '0-99'
    WHEN total < 200 THEN '100-199'
    WHEN total < 300 THEN '200-299'
    ELSE '300+'
  END as bucket,
  COUNT(*) as frequency
FROM orders
GROUP BY bucket;
